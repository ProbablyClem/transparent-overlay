use std::{
    collections::VecDeque,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

use egui::{ColorImage, TextureOptions};

use crate::{
    events::AppEvent,
    media::{ActiveMedia, MediaChat, MediaType},
    ui::{decode_circular, decode_image, wake, CtxWaker},
    video::{spawn_video_decoder, VideoFrame},
};

// ─────────────────────────────────────────────────────────────────────────────
//  App state
// ─────────────────────────────────────────────────────────────────────────────

pub struct App {
    event_tx: Sender<AppEvent>,
    event_rx: Receiver<AppEvent>,

    /// FIFO — index 0 is the currently displayed item
    queue: VecDeque<MediaChat>,
    pub current: Option<ActiveMedia>,

    /// ffplay/paplay child process for the current audio (killed on advance)
    audio_child: Option<Child>,

    http: reqwest::blocking::Client,

    /// Whether Win32 overlay setup has been done (runs once after window is ready)
    pub(crate) win32_initialized: bool,

    waker: CtxWaker,

    /// Keep tray icon alive for the duration of the app
    _tray_icon: Option<tray_icon::TrayIcon>,
}

// ─────────────────────────────────────────────────────────────────────────────
//  App impl
// ─────────────────────────────────────────────────────────────────────────────

impl App {
    pub fn new(
        cc: &eframe::CreationContext,
        event_tx: Sender<AppEvent>,
        event_rx: Receiver<AppEvent>,
        waker: CtxWaker,
        tray_icon: Option<tray_icon::TrayIcon>,
    ) -> Self {
        // Register the egui context so background threads can request repaints
        let _ = waker.set(cc.egui_ctx.clone());

        Self {
            event_tx,
            event_rx,
            queue: VecDeque::new(),
            current: None,
            audio_child: None,
            http: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(),
            win32_initialized: false,
            waker,
            _tray_icon: tray_icon,
        }
    }

    // ── item lifecycle ────────────────────────────────────────────────────────

    fn start_item(&mut self, chat: MediaChat) {
        self.kill_audio();
        self.current = None; // triggers ActiveMedia::drop → temp file cleanup

        let active = ActiveMedia::new(chat.clone());

        if let Some(ref url) = chat.author.image {
            self.download_in_bg(url.clone(), AppEvent::AvatarLoaded);
        }

        if let Some(ref media) = chat.media {
            match media.media_type {
                MediaType::Image => {
                    self.download_in_bg(media.url.clone(), AppEvent::MediaImageLoaded);
                }
                MediaType::Video => {
                    spawn_video_decoder(
                        media.url.clone(),
                        self.event_tx.clone(),
                        self.waker.clone(),
                    );
                }
                MediaType::Sound => {
                    // ffplay handles HTTP URLs directly — no download needed
                    self.play_audio_url(&media.url);
                }
            }
        }

        self.current = Some(active);
    }

    pub fn advance(&mut self) {
        self.kill_audio();
        self.current = None; // triggers drop
        if let Some(next) = self.queue.pop_front() {
            self.start_item(next);
        }
    }

    // ── audio via ffplay subprocess ──────────────────────────────────────────

    fn kill_audio(&mut self) {
        if let Some(mut child) = self.audio_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    /// Play a URL (http/file) through ffplay in the background.
    fn play_audio_url(&mut self, url: &str) {
        match Command::new("ffplay")
            .args(["-nodisp", "-autoexit", "-loglevel", "quiet", url])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => self.audio_child = Some(child),
            Err(e) => log::warn!("Failed to spawn ffplay: {e}"),
        }
    }

    /// Play a local file path through ffplay.
    fn play_audio_file(&mut self, path: &str) {
        self.play_audio_url(path);
    }

    // ── background HTTP download ─────────────────────────────────────────────

    fn download_in_bg<F>(&self, url: String, make_event: F)
    where
        F: Fn(Vec<u8>) -> AppEvent + Send + 'static,
    {
        let http = self.http.clone();
        let tx = self.event_tx.clone();
        let waker = self.waker.clone();
        std::thread::spawn(
            move || match http.get(&url).send().and_then(|r| r.bytes()) {
                Ok(bytes) => {
                    let _ = tx.send(make_event(bytes.to_vec()));
                    wake(&waker);
                }
                Err(e) => log::warn!("Download failed for {url}: {e}"),
            },
        );
    }

    // ── event processing ─────────────────────────────────────────────────────

    pub fn process_events(&mut self, ctx: &egui::Context) {
        while let Ok(ev) = self.event_rx.try_recv() {
            match ev {
                AppEvent::NewMediaChat(mc) => {
                    if self.current.is_none() {
                        self.start_item(*mc);
                    } else {
                        self.queue.push_back(*mc);
                    }
                }

                AppEvent::Flush => {
                    self.queue.clear();
                    self.kill_audio();
                    self.current = None;
                }

                AppEvent::Skip => self.advance(),

                AppEvent::AvatarLoaded(data) => {
                    if let Some(active) = &mut self.current {
                        if let Some(ci) = decode_circular(&data) {
                            active.avatar_tex =
                                Some(ctx.load_texture("avatar", ci, TextureOptions::NEAREST));
                        }
                    }
                }

                AppEvent::MediaImageLoaded(data) => {
                    if let Some(active) = &mut self.current {
                        if let Some(ci) = decode_image(&data) {
                            active.media_tex =
                                Some(ctx.load_texture("media", ci, TextureOptions::LINEAR));
                        }
                    }
                }

                AppEvent::VideoReady {
                    frame_rx,
                    audio_path,
                } => {
                    if let Some(active) = &mut self.current {
                        active.frame_rx = Some(frame_rx);

                        // Start audio and record the precise instant it began.
                        // This instant is used as the video clock origin so that
                        // frame PTS values are measured from the same zero as audio.
                        if let Some(ref path) = audio_path {
                            self.play_audio_file(path);
                            if let Some(ref mut a) = self.current {
                                a.audio_started_at = Some(Instant::now());
                                a.temp_path = Some(PathBuf::from(path));
                            }
                        }
                    }
                }

                AppEvent::VideoEnded => {
                    if let Some(active) = &mut self.current {
                        active.video_ended = true;
                    }
                }
            }
        }
    }

    // ── video frame advancement ───────────────────────────────────────────────

    pub fn update_video_frame(&mut self, ctx: &egui::Context) {
        let active = match &mut self.current {
            Some(a) if a.is_video() => a,
            _ => return,
        };

        // Pull newly decoded frames from the decoder into our local queue
        if let Some(ref rx) = active.frame_rx {
            while let Ok(frame) = rx.try_recv() {
                if active.video_clock.is_none() {
                    // Use audio_started_at as the clock base so that frame PTS
                    // is measured against the same origin as audio playback.
                    // Fall back to now() for video-only streams (no audio).
                    active.video_clock = Some(active.audio_started_at.unwrap_or_else(Instant::now));
                }
                active.pending_frames.push_back(frame);
            }
        }

        let elapsed = match active.video_clock {
            Some(t) => t.elapsed().as_secs_f64(),
            None => return,
        };

        // Discard frames whose PTS has passed, keep the most recent one
        let mut last: Option<VideoFrame> = None;
        while active
            .pending_frames
            .front()
            .map(|f| f.pts_secs <= elapsed)
            .unwrap_or(false)
        {
            last = active.pending_frames.pop_front();
        }

        if let Some(frame) = last {
            let ci = ColorImage::from_rgba_unmultiplied(
                [frame.width as usize, frame.height as usize],
                &frame.data,
            );
            active.frame_tex = Some(ctx.load_texture("vframe", ci, TextureOptions::LINEAR));
        }
    }
}

use std::{collections::VecDeque, path::PathBuf, sync::mpsc::Receiver, time::Instant};

use egui::TextureHandle;

use crate::{
    media::{MediaChat, MediaType},
    video::VideoFrame,
};

pub struct ActiveMedia {
    pub chat: MediaChat,

    pub avatar_tex: Option<TextureHandle>,
    pub media_tex: Option<TextureHandle>, // image-type media
    pub frame_tex: Option<TextureHandle>, // current video frame

    /// Bounded receiver from the video decoder thread
    pub frame_rx: Option<Receiver<VideoFrame>>,
    /// Decoded frames waiting to be displayed at the right PTS
    pub pending_frames: VecDeque<VideoFrame>,
    pub video_ended: bool,
    /// Wall-clock instant when the first frame was received (video clock origin)
    pub video_clock: Option<Instant>,

    /// Wall-clock instant this item started displaying
    pub started_at: Instant,

    /// Wall-clock instant when ffplay audio was started (used as video clock base for A/V sync)
    pub audio_started_at: Option<Instant>,

    /// Temp file to clean up after the video finishes
    pub temp_path: Option<PathBuf>,
}

impl ActiveMedia {
    pub fn new(chat: MediaChat) -> Self {
        Self {
            chat,
            avatar_tex: None,
            media_tex: None,
            frame_tex: None,
            frame_rx: None,
            pending_frames: VecDeque::new(),
            video_ended: false,
            video_clock: None,
            started_at: Instant::now(),
            audio_started_at: None,
            temp_path: None,
        }
    }

    pub fn is_video(&self) -> bool {
        self.chat
            .media
            .as_ref()
            .map(|m| m.media_type == MediaType::Video)
            .unwrap_or(false)
    }

    pub fn should_advance(&self) -> bool {
        if self.is_video() {
            self.video_ended && self.pending_frames.is_empty()
        } else {
            let dur = self.chat.duration.unwrap_or(5.0);
            self.started_at.elapsed().as_secs_f64() >= dur
        }
    }
}

impl Drop for ActiveMedia {
    fn drop(&mut self) {
        // Remove the downloaded video temp file when this item is done
        if let Some(ref path) = self.temp_path {
            let _ = std::fs::remove_file(path);
        }
    }
}

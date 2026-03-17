/// Video decoder using FFmpeg subprocess (ffmpeg + ffprobe).
///
/// Requires `ffmpeg` and `ffprobe` to be on PATH at runtime.
/// (Add the FFmpeg bin directory to your system PATH.)
///
/// Audio is handled by spawning `ffplay -nodisp -autoexit` in app.rs.
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc::SyncSender;

use anyhow::{anyhow, Result};

use crate::events::AppEvent;
use crate::ui::{wake, CtxWaker};

/// Number of decoded frames buffered in the channel before back-pressure kicks in.
const FRAME_BUF: usize = 15;

/// Spawn the video decode pipeline in a background thread.
pub fn spawn_video_decoder(
    url: String,
    event_tx: std::sync::mpsc::Sender<AppEvent>,
    waker: CtxWaker,
) {
    std::thread::spawn(move || {
        if let Err(e) = pipeline(url, event_tx.clone(), waker.clone()) {
            log::error!("Video pipeline error: {e}");
            let _ = event_tx.send(AppEvent::VideoEnded);
            wake(&waker);
        }
    });
}

fn pipeline(
    url: String,
    event_tx: std::sync::mpsc::Sender<AppEvent>,
    waker: CtxWaker,
) -> Result<()> {
    // ── download ─────────────────────────────────────────────────────────────
    log::info!("Downloading video: {url}");
    let bytes = reqwest::blocking::get(&url)?.bytes()?;

    let tmp = tempfile::NamedTempFile::new()?;
    let (mut tmp_file, tmp_path) = tmp.keep()?;
    std::io::Write::write_all(&mut tmp_file, &bytes)?;
    drop(tmp_file);

    let path = tmp_path.to_string_lossy().to_string();

    // ── probe video info ──────────────────────────────────────────────────────
    let (width, height, fps, has_audio) = probe_video(&path)?;
    let audio_path = if has_audio { Some(path.clone()) } else { None };

    // ── hand off frame channel to the app ────────────────────────────────────
    let (frame_tx, frame_rx) = std::sync::mpsc::sync_channel::<VideoFrame>(FRAME_BUF);
    let _ = event_tx.send(AppEvent::VideoReady {
        frame_rx,
        audio_path,
    });
    wake(&waker);

    // ── decode video frames ───────────────────────────────────────────────────
    decode_video(&path, width, height, fps, frame_tx, waker.clone())?;
    let _ = event_tx.send(AppEvent::VideoEnded);
    wake(&waker);

    let _ = std::fs::remove_file(&tmp_path);
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────

/// Returns (width, height, fps, has_audio) from ffprobe JSON output.
fn probe_video(path: &str) -> Result<(u32, u32, f64, bool)> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "quiet",
            "-print_format",
            "json",
            "-show_streams",
            path,
        ])
        .output()?;

    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow!("ffprobe JSON parse error: {e}"))?;

    let streams = json["streams"]
        .as_array()
        .ok_or_else(|| anyhow!("ffprobe: no streams array"))?;

    let mut width = 0u32;
    let mut height = 0u32;
    let mut fps = 30.0f64;
    let mut has_video = false;
    let mut has_audio = false;

    for stream in streams {
        match stream["codec_type"].as_str().unwrap_or("") {
            "video" if !has_video => {
                has_video = true;
                width = stream["width"].as_u64().unwrap_or(0) as u32;
                height = stream["height"].as_u64().unwrap_or(0) as u32;
                // avg_frame_rate is a "num/den" string
                if let Some(r) = stream["avg_frame_rate"].as_str() {
                    fps = parse_ratio(r).unwrap_or(30.0);
                }
            }
            "audio" => has_audio = true,
            _ => {}
        }
    }

    if !has_video || width == 0 || height == 0 {
        return Err(anyhow!("ffprobe: no video stream found in {path}"));
    }

    Ok((width, height, fps, has_audio))
}

fn parse_ratio(s: &str) -> Option<f64> {
    let mut it = s.split('/');
    let num: f64 = it.next()?.parse().ok()?;
    let den: f64 = it.next()?.parse().ok()?;
    if den == 0.0 {
        None
    } else {
        Some(num / den)
    }
}

/// Pipes raw RGBA frames from ffmpeg stdout and forwards them to the channel.
fn decode_video(
    path: &str,
    width: u32,
    height: u32,
    fps: f64,
    tx: SyncSender<VideoFrame>,
    waker: CtxWaker,
) -> Result<()> {
    let mut child = Command::new("ffmpeg")
        .args(["-i", path, "-f", "rawvideo", "-pix_fmt", "rgba", "pipe:1"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;

    let frame_size = (width * height * 4) as usize;
    let mut buf = vec![0u8; frame_size];
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("ffmpeg: no stdout"))?;
    let mut frame_idx = 0u64;

    loop {
        match stdout.read_exact(&mut buf) {
            Ok(()) => {
                let pts_secs = frame_idx as f64 / fps.max(1.0);
                let frame = VideoFrame {
                    width,
                    height,
                    data: buf.clone(),
                    pts_secs,
                };
                if tx.send(frame).is_err() {
                    break; // receiver dropped (app navigated away)
                }
                wake(&waker);
                frame_idx += 1;
            }
            Err(_) => break, // EOF or pipe closed
        }
    }

    let _ = child.wait();
    Ok(())
}

pub struct VideoFrame {
    pub width: u32,
    pub height: u32,
    /// RGBA packed bytes, length = width * height * 4
    pub data: Vec<u8>,
    /// Presentation timestamp in seconds
    pub pts_secs: f64,
}

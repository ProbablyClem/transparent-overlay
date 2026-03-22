use std::sync::mpsc::Receiver;

use crate::{media::MediaChat, video::VideoFrame};

pub enum AppEvent {
    // Socket.IO events
    NewMediaChat(Box<MediaChat>),
    Flush,
    Skip,

    // Asset download results
    AvatarLoaded(Vec<u8>),
    MediaImageLoaded(Vec<u8>),

    /// Video decoder is ready.
    /// `frame_rx`  — receive decoded RGBA frames
    /// `audio_path` — temp file path to pass to ffplay for audio (None if no audio stream)
    VideoReady {
        frame_rx: Receiver<VideoFrame>,
        audio_path: Option<String>,
    },

    /// All video frames have been sent (channel may still have buffered frames)
    VideoEnded,
}

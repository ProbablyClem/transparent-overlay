# mediachat-native

Fullscreen transparent overlay for MediaChat, built with Rust + egui (OpenGL). Receives Socket.IO events from the backend and renders media (images, videos, audio, text) directly on screen — no browser, no webview.

## Prerequisites (Windows)

| Tool | Install |
|---|---|
| Rust MSVC | `winget install Rustlang.Rustup` → `rustup default stable-x86_64-pc-windows-msvc` |
| VS BuildTools 2022 (C++) | `winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --norestart"` |
| LLVM | `winget install LLVM.LLVM` |
| FFmpeg BtbN GPL Shared 7.1 | `winget install BtbN.FFmpeg.GPL.Shared.7.1` |

After installing FFmpeg, add its `bin/` folder to your **system PATH** (`ffmpeg`, `ffprobe`, `ffplay` must be reachable at runtime).

## Build

```bash
cargo build --release
# → target/release/mediachat-native.exe
```

## Run

```bash
./target/release/mediachat-native.exe --server <BACKEND_URL> --room <your-discord-username>
```

**Example:**
```bash
./target/release/mediachat-native.exe \
  --server "http://q0g4sgow8c040ookw80g0ogg.54.36.101.56.sslip.io" \
  --room "elkofy"
```

> The `--server` URL is the **backend** (Socket.IO) URL, not the frontend web URL.
> The `--room` is your Discord username — the same key used in the web viewer (`/viewer/:key`).

## Environment variable

`--server` can also be set via `MEDIACHAT_SERVER`:
```bash
export MEDIACHAT_SERVER=http://...
./mediachat-native.exe --room elkofy
```

## Notes

- Renderer: **glow (OpenGL/glutin)** for per-pixel alpha transparency on Windows
- Video decoding: piped through `ffmpeg` subprocess (no FFI/bindgen)
- Audio playback: `ffplay -nodisp -autoexit`
- The overlay is fullscreen, always-on-top, click-through (mouse passthrough enabled)

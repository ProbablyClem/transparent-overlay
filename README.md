# mediachat-native

Fullscreen transparent overlay for MediaChat, built with Rust + egui (OpenGL). Receives Socket.IO events from the backend and renders media (images, videos, audio, text) directly on screen — no browser, no webview.

## Install

Download the latest `MediaChat-Setup-x.y.z.exe` from the [Releases](../../releases) page and run it. The installer:

- Installs the app to `%LOCALAPPDATA%\MediaChat\`
- Registers it to **start automatically at Windows login**
- Adds an entry in Add/Remove Programs for clean uninstallation

No admin rights required.

### Configuration

Create a `.env` file in `%LOCALAPPDATA%\MediaChat\` (next to the exe):

```env
MEDIACHAT_SERVER=http://<your-backend>
```

Or pass it as a CLI flag: `--server http://<your-backend>`.

---

## Development

### Prerequisites (Windows)

| Tool | Install |
|---|---|
| Rust MSVC | `winget install Rustlang.Rustup` → `rustup default stable-x86_64-pc-windows-msvc` |
| VS BuildTools 2022 (C++) | `winget install Microsoft.VisualStudio.2022.BuildTools --override "--quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --norestart"` |

FFmpeg is **downloaded automatically** on first launch via [ffmpeg-sidecar](https://github.com/ProbablyClem/ffmpeg-sidecar) — no manual installation needed.

### Build

```bash
cargo build --release
# → target/release/mediachat-native.exe
```

### Run

```bash
./target/release/mediachat-native.exe --server <BACKEND_URL> --room <your-discord-username>
```

`--server` can also be set via the `MEDIACHAT_SERVER` environment variable or a `.env` file at the working directory.

---

## Notes

- Renderer: **glow (OpenGL/glutin)** — per-pixel alpha transparency on Windows via Win32 `SetLayeredWindowAttributes`
- Video decoding: piped through `ffmpeg` subprocess (no FFI/bindgen)
- Audio playback: `ffplay -nodisp -autoexit`
- The overlay is fullscreen, always-on-top, click-through (mouse passthrough enabled)

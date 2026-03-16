# PulseStream

Stream Windows audio to a PulseAudio server over TCP. Built with Rust and [iced](https://github.com/iced-rs/iced).

![Windows](https://img.shields.io/badge/platform-Windows-blue)
![Rust](https://img.shields.io/badge/language-Rust-orange)
[![CI](https://github.com/dasunNimantha/pulse-stream/actions/workflows/ci.yml/badge.svg)](https://github.com/dasunNimantha/pulse-stream/actions/workflows/ci.yml)

## Features

- **WASAPI loopback capture** — captures system audio with low latency using Windows Audio Session API
- **Per-app audio capture** — isolate and stream audio from a single application via process loopback
- **Auto server discovery** — scans the local subnet to find PulseAudio TCP servers
- **Real-time stats** — displays bandwidth, latency, capture format, and uptime
- **System volume integration** — reads Windows volume/mute state and applies it to the stream
- **System tray** — minimize to tray with restore and exit from the context menu
- **Dark / Light theme** — cyan-accented theme with a toggle in the header
- **Persistent settings** — server, port, audio format, and preferences saved to `settings.json`
- **Auto-reconnect** — retries the connection automatically on network failure

## Download

Grab the latest `.exe` from the [Releases](https://github.com/dasunNimantha/pulse-stream/releases) page — no installation required.

## Building from source

### Prerequisites

- [Rust](https://rustup.rs/) (stable, 2021 edition)
- Windows 10 or later

### Build

```bash
cargo build --release
```

The compiled binary will be at `target/release/pulse-stream-rs.exe`.

### Run

```bash
cargo run
```

### Test

```bash
cargo test
```

72 tests cover input validation, settings serialization, audio streamer lifecycle, and theme properties.

## Usage

1. Start a PulseAudio TCP module on the receiving machine:
   ```bash
   pactl load-module module-simple-protocol-tcp rate=48000 format=s16le channels=2 source=0 record=false port=4714
   ```
2. Launch PulseStream on Windows
3. Enter the server IP and port, or click the scan button to auto-detect
4. Select an audio device and optionally a specific application
5. Click **Connect**

### Configuration

Settings are stored at:

```
%LOCALAPPDATA%\PulseStream\data\settings.json
```

| Setting           | Default | Description                        |
| ----------------- | ------- | ---------------------------------- |
| `server`          | *(empty — triggers auto-scan)* | PulseAudio server IP |
| `port`            | `4714`  | TCP port                           |
| `rate`            | `48000` | Sample rate in Hz                  |
| `channels`        | `2`     | Channel count (1–8)                |
| `device_id`       | `null`  | Audio output device (null = default) |
| `auto_connect`    | `false` | Connect on startup                 |
| `minimize_to_tray`| `true`  | Minimize to system tray on close   |
| `dark_theme`      | `true`  | Dark mode enabled                  |

## Architecture

```
src/
├── main.rs        # Entry point and window configuration
├── lib.rs         # Module exports
├── app.rs         # Application state, update loop, subscriptions
├── audio.rs       # WASAPI capture, TCP streaming, device enumeration
├── view.rs        # UI layout and input validation
├── theme.rs       # Color schemes and widget styles
├── message.rs     # Iced message types
└── settings.rs    # Persistent settings (serde JSON)
```

**Key design decisions:**

- Audio capture runs on a dedicated thread communicating via `flume` channels to keep the UI responsive
- Pre-allocated reusable buffers in the capture loop eliminate per-frame heap allocations
- `TCP_NODELAY` with a small send buffer minimizes streaming latency
- COM is initialized per-thread to avoid apartment model conflicts with the iced/winit event loop

## CI / CD

| Workflow | Trigger | What it does |
| -------- | ------- | ------------ |
| **CI** | Push / PR to `main` | Format check, clippy, tests, release build |
| **Release** | Tag `v*` | Tests, release build, creates GitHub Release with `.exe` |

To publish a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## License

MIT

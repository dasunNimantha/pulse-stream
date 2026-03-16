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

## Problem & Motivation

Streaming audio from a Windows PC to a Linux machine running PulseAudio typically requires either running a full PulseAudio client on Windows or using third-party tools that add significant overhead. Existing solutions often suffer from:

- **High latency** — multiple layers of buffering between capture, encoding, network, and playback
- **No per-app isolation** — you stream everything or nothing, with no way to pick a single application
- **Heavy dependencies** — requiring full PulseAudio installations or virtual audio drivers on Windows

PulseStream solves this by using WASAPI loopback capture to read audio directly from the Windows audio engine and streaming raw PCM over a simple TCP socket to PulseAudio's `module-simple-protocol-tcp`. No encoding, no virtual drivers, no PulseAudio client on Windows.

## Latency

The end-to-end audio pipeline has several stages, each contributing delay:

| Stage | Typical delay | Notes |
| ----- | ------------- | ----- |
| WASAPI capture buffer | ~10 ms | Configurable; set to 10 ms (100,000 × 100 ns units) |
| PCM conversion | < 0.1 ms | Zero-copy i16 conversion via direct memory write |
| TCP transmission | 0.1–2 ms | `TCP_NODELAY` enabled, send buffer set to 1920 bytes |
| Network transit | 0.1–1 ms | Depends on your LAN (wired vs WiFi) |
| PulseAudio receive buffer | 10–50 ms | Controlled by `module-simple-protocol-tcp` config |
| ALSA playback buffer | 5–20 ms | Depends on the sound server / sink configuration |

**Realistic total: 25–80 ms** depending on network and PulseAudio configuration.

### Tips to reduce latency on the receiver

```bash
# Use a smaller PulseAudio buffer (e.g. 1024 fragments)
pactl load-module module-simple-protocol-tcp \
  rate=48000 format=s16le channels=2 \
  source=0 record=false port=4714

# Lower the default ALSA buffer in /etc/pulse/daemon.conf
default-fragments = 2
default-fragment-size-msec = 5
```

Use a wired Ethernet connection instead of WiFi for the most consistent latency.

## Limitations

- **Windows only** — WASAPI is a Windows API; the sender must run on Windows 10 or later
- **No audio encoding** — streams raw PCM (s16le), so bandwidth usage is proportional to sample rate and channel count (~1.5 Mbps at 48 kHz stereo). Not suitable over the internet or slow networks
- **No encryption** — audio is sent as plaintext TCP. Use only on trusted local networks
- **Receiver must run PulseAudio** — specifically `module-simple-protocol-tcp`. Does not work with plain ALSA, PipeWire (without PulseAudio compat), or other sound servers out of the box
- **Single receiver** — streams to one TCP endpoint at a time; no multicast or multi-client support
- **No sample rate conversion** — the sender captures at the device's native rate and sends as-is. The `rate` and `channels` fields configure the PulseAudio module expectation but do not resample
- **Per-app capture requires Windows 10 2004+** — process loopback (`AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK`) is only available on Windows 10 version 2004 and later
- **System tray requires a window manager** — the tray icon uses OS-level system tray APIs; headless or terminal-only Windows environments are not supported

## CI / CD

| Workflow | Trigger | What it does |
| -------- | ------- | ------------ |
| **CI** | Push / PR to `master` | Format check, clippy, tests, release build |
| **Release** | Tag `v*` | Tests, release build, creates GitHub Release with `.exe` |

To publish a release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

## License

MIT

# PulseStream

Stream Windows audio to a Linux machine over TCP using ALSA. Built with Rust and [iced](https://github.com/iced-rs/iced).

![Windows](https://img.shields.io/badge/platform-Windows-blue)
![Rust](https://img.shields.io/badge/language-Rust-orange)
[![CI](https://github.com/dasunNimantha/pulse-stream/actions/workflows/ci.yml/badge.svg)](https://github.com/dasunNimantha/pulse-stream/actions/workflows/ci.yml)

## Features

- **WASAPI loopback capture** — captures system audio with low latency using Windows Audio Session API
- **Per-app audio capture** — isolate and stream audio from a single application via process loopback
- **Auto server discovery** — scans the local subnet to find receivers
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

The compiled binary will be at `target/release/pulse-stream.exe`.

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

### Receiver setup (Linux)

The receiver script listens on a TCP port and pipes audio straight to ALSA with a small buffer for low-latency playback:

```bash
#!/bin/bash
# pulse-stream-recv.sh — low-latency TCP-to-ALSA receiver
PORT=${1:-4714}
RATE=48000
CHANNELS=2
FORMAT=S16_LE
# Buffer: 256 frames × 2 periods = ~10ms at 48kHz
BUFFER_FRAMES=256
PERIODS=2

echo "Listening on port $PORT (${RATE}Hz ${CHANNELS}ch ${FORMAT})"
echo "Press Ctrl+C to stop"

while true; do
  ncat -l -p "$PORT" | aplay \
    -t raw \
    -f "$FORMAT" \
    -r "$RATE" \
    -c "$CHANNELS" \
    --buffer-size=$((BUFFER_FRAMES * CHANNELS * 2 * PERIODS)) \
    --period-size=$((BUFFER_FRAMES * CHANNELS * 2)) \
    -D default \
    2>/dev/null
  echo "Client disconnected, waiting for reconnect..."
done
```

Make it executable and run:

```bash
chmod +x pulse-stream-recv.sh
./pulse-stream-recv.sh 4714
```

> Requires `ncat` (from nmap) and `alsa-utils`. Install with:
> `sudo apt install ncat alsa-utils` (Debian/Ubuntu) or
> `sudo pacman -S nmap alsa-utils` (Arch).

#### Persisting across reboots (systemd)

To run the ALSA receiver as a service that starts on boot and auto-reconnects:

1. Copy the script above to `/usr/local/bin/pulse-stream-recv.sh` and make it executable:

```bash
sudo chmod +x /usr/local/bin/pulse-stream-recv.sh
```

2. Create the service file:

```bash
sudo tee /etc/systemd/system/pulse-stream-recv.service > /dev/null << 'EOF'
[Unit]
Description=PulseStream low-latency ALSA receiver
After=network.target sound.target
Wants=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/pulse-stream-recv.sh 4714
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
EOF
```

3. Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable pulse-stream-recv.service
sudo systemctl start pulse-stream-recv.service
```

Check status with `systemctl status pulse-stream-recv.service`. The service automatically restarts when a stream disconnects and is ready for the next connection.

### Sender setup (Windows)

1. Launch PulseStream on Windows
2. Enter the server IP and port, or click the scan button to auto-detect
3. Select an audio device and optionally a specific application
4. Click **Connect**

### Configuration

Settings are stored at:

```
%LOCALAPPDATA%\PulseStream\data\settings.json
```

| Setting           | Default | Description                        |
| ----------------- | ------- | ---------------------------------- |
| `server`          | *(empty — triggers auto-scan)* | Receiver IP |
| `port`            | `4714`  | TCP port                           |
| `rate`            | `48000` | Sample rate in Hz                  |
| `channels`        | `2`     | Channel count (1–8)                |
| `device_id`       | `null`  | Audio output device (null = default) |
| `auto_connect`    | `false` | Connect on startup                 |
| `minimize_to_tray`| `true`  | Minimize to system tray on close   |
| `dark_theme`      | `true`  | Dark mode enabled                  |

**Key design decisions:**

- Audio capture runs on a dedicated thread communicating via `flume` channels to keep the UI responsive
- Pre-allocated reusable buffers in the capture loop eliminate per-frame heap allocations
- `TCP_NODELAY` with a small send buffer minimizes streaming latency
- COM is initialized per-thread to avoid apartment model conflicts with the iced/winit event loop

## Problem & Motivation

Streaming audio from a Windows PC to a Linux machine typically requires third-party tools that add significant overhead. Existing solutions often suffer from:

- **High latency** — multiple layers of buffering between capture, encoding, network, and playback
- **No per-app isolation** — you stream everything or nothing, with no way to pick a single application
- **Heavy dependencies** — requiring virtual audio drivers or complex audio server configurations on Windows

PulseStream solves this by using WASAPI loopback capture to read audio directly from the Windows audio engine and streaming raw PCM over a simple TCP socket to an ALSA receiver on Linux. No encoding, no virtual drivers, minimal dependencies on both ends.

## Latency

The end-to-end audio pipeline has several stages, each contributing delay:

| Stage | Typical delay | Notes |
| ----- | ------------- | ----- |
| WASAPI capture buffer | ~10 ms | Set to 10 ms (100,000 × 100 ns units) |
| PCM conversion | < 0.1 ms | Zero-copy i16 conversion via direct memory write |
| TCP send | 0.1–2 ms | `TCP_NODELAY` enabled, 1920-byte send buffer |
| Network transit | 0.1–1 ms | Wired LAN recommended |
| **ALSA receiver buffer** | **~10 ms** | 256 frames × 2 periods at 48 kHz |

### Where the delay comes from

The sender side (WASAPI capture → TCP send) adds only ~12 ms total. The remaining delay is the **ALSA receiver buffer** — configured at ~10 ms (256 frames × 2 periods at 48 kHz), bringing the total end-to-end latency to **~25 ms**, which is low enough that delay is not perceptible for most use cases.

### Additional tips

- Use a **wired Ethernet** connection — WiFi adds jitter and occasional 5–20 ms spikes

## Limitations

- **Windows only** — WASAPI is a Windows API; the sender must run on Windows 10 or later
- **No audio encoding** — streams raw PCM (s16le), so bandwidth usage is proportional to sample rate and channel count (~1.5 Mbps at 48 kHz stereo). Not suitable over the internet or slow networks
- **No encryption** — audio is sent as plaintext TCP. Use only on trusted local networks
- **Receiver must run ALSA** — uses the provided ALSA receiver script with `ncat` and `aplay`. No native PipeWire or macOS/Windows receiver support
- **Single receiver** — streams to one TCP endpoint at a time; no multicast or multi-client support
- **No sample rate conversion** — the sender captures at the device's native rate and sends as-is. The `rate` and `channels` fields must match the receiver's `aplay` configuration
- **Per-app capture requires Windows 10 2004+** — process loopback (`AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK`) is only available on Windows 10 version 2004 and later
- **System tray requires a window manager** — the tray icon uses OS-level system tray APIs; headless or terminal-only Windows environments are not supported

## License

MIT

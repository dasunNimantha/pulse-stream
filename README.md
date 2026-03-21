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
- **Start with Windows** — launches at boot minimized to tray, enabled by default
- **System tray** — starts hidden in tray; restore with a click, exit from the context menu
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

#### Watchdog (optional)

After long uptime the receiver can occasionally end up with nothing listening on port 4714 (the script is between `ncat` restarts). A watchdog restarts the service when the port has **no healthy activity**: no **LISTEN** and no **ESTABLISHED** socket.

**Why not match any line with `:4714`?** After a client disconnects, TCP can linger in states like **FIN-WAIT-2** or **TIME-WAIT**. A naive `ss | grep :4714` still finds those sockets, so the watchdog thinks everything is fine while `ncat` is stuck and **no longer accepts new connections**. The script below only treats **listening** and **established** sockets as healthy.

It must not restart while a client is connected (`ncat` is then ESTABLISHED).

1. Create the watchdog script:

```bash
sudo tee /usr/local/bin/pulse-stream-recv-watchdog.sh > /dev/null << 'EOF'
#!/bin/bash
# Healthy: LISTEN (waiting for clients) or ESTABLISHED (active stream).
# Stale FIN-WAIT-2 / TIME-WAIT / CLOSE-WAIT must NOT skip a restart.
ss -tnp state established state listening | grep -q ':4714 ' && exit 0
systemctl restart pulse-stream-recv.service
EOF
sudo chmod +x /usr/local/bin/pulse-stream-recv-watchdog.sh
```

2. Create the timer and oneshot service:

```bash
sudo tee /etc/systemd/system/pulse-stream-recv-watchdog.service > /dev/null << 'EOF'
[Unit]
Description=PulseStream receiver watchdog - restart if port 4714 not listening
After=pulse-stream-recv.service network.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/pulse-stream-recv-watchdog.sh
EOF

sudo tee /etc/systemd/system/pulse-stream-recv-watchdog.timer > /dev/null << 'EOF'
[Unit]
Description=Run PulseStream receiver watchdog every 2 minutes

[Timer]
OnCalendar=*:0/2
Persistent=true

[Install]
WantedBy=timers.target
EOF
```

3. Enable and start the timer:

```bash
sudo systemctl daemon-reload
sudo systemctl enable pulse-stream-recv-watchdog.timer
sudo systemctl start pulse-stream-recv-watchdog.timer
```

The watchdog runs every 2 minutes. If there is no **LISTEN** or **ESTABLISHED** socket on 4714, it restarts `pulse-stream-recv.service` so the sender can connect again without manual intervention.

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
| `start_with_windows` | `true` | Register in Windows startup       |
| `minimize_to_tray`| `true`  | Start hidden in tray; minimize on close |
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

The sender side (WASAPI capture → TCP send) adds ~12 ms total. The **ALSA receiver buffer** adds ~10 ms (256 frames × 2 periods at 48 kHz), for **~25 ms** total — low enough that delay is not perceptible for most use cases.

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

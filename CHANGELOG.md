# Changelog

## [0.1.1] - 2026-03-16

### Added
- **Start with Windows** — app registers itself in the Windows startup registry; enabled by default
- **Start minimized to tray** — window launches hidden when minimize-to-tray is enabled, ideal for boot startup
- **Auto-connect on startup** — automatically connects to the saved server when the app starts
- **ALSA receiver systemd service** — README includes instructions for persisting the receiver across reboots
- `start_with_windows` setting persisted in `settings.json`

### Changed
- Receiver documentation rewritten to focus exclusively on ALSA (PulseAudio option removed)
- README tagline, features, motivation, latency, and limitations updated to reflect ALSA-only approach
- Package renamed from `pulse-stream-rs` to `pulse-stream`; binary is now `pulse-stream.exe`
- CI workflow triggers on `master` branch; tag pushes excluded to prevent duplicate runs with release workflow

## [0.1.0] - 2026-03-16

### Added
- WASAPI loopback audio capture with low-latency 10ms buffer
- Per-app audio capture via process loopback (Windows 10 2004+)
- TCP streaming with `TCP_NODELAY` and tuned send buffer
- Auto server discovery via local subnet scan
- Real-time stats: bandwidth, latency, capture format, uptime
- System volume integration (reads Windows volume/mute state)
- Dark/light theme with cyan accents
- System tray with minimize, restore, and exit
- Persistent settings via `settings.json`
- Auto-reconnect on network failure
- Zero-copy PCM conversion for minimal capture overhead
- 72 integration tests covering validation, settings, audio, and theme
- GitHub Actions CI and release pipelines

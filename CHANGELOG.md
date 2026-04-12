# Changelog

## [0.1.5] - 2026-04-12

### Fixed
- **WASAPI GetBuffer/ReleaseBuffer misuse** — no longer calls `ReleaseBuffer` when `GetBuffer` fails, preventing potential undefined behavior
- **Drop counter always zero** — `Stats.drops` now tracks actual channel send failures instead of being hardcoded to zero
- **Device selection not persisted** — changing the audio device now saves to `settings.json` immediately

### Added
- **Audio device hot-plug detection** — device list refreshes every 3 seconds so newly plugged speakers/headphones appear without restart
- **Release profile optimizations** — LTO, symbol stripping, single codegen unit for smaller and faster binaries
- **179 tests** — comprehensive coverage for app state transitions, message handling, settings I/O, audio events, validation edge cases, and theme widget styles (up from 76)

### Changed
- Reduced hot-path allocations in capture loop using `Arc<[u8]>` instead of `Vec::clone()`
- Upgraded `AtomicBool` ordering from `Relaxed` to `Acquire`/`Release` for correct cross-thread visibility
- Trimmed tokio features from `full` to only `rt-multi-thread`, `macros`, `time`, `net`
- Removed dead `ToggleMinimizeToTray` message variant

## [0.1.4] - 2026-04-12

### Added
- **Auto-reconnect on connection loss** — detects when the TCP writer thread dies (router failure, power loss, receiver down) and automatically reconnects instead of silently dropping audio
- **TCP keepalive** — enables `SO_KEEPALIVE` on the streaming socket so the OS probes half-open connections and surfaces dead peers faster

### Changed
- Refactored streaming parameters into `StreamConfig` struct to satisfy clippy `too_many_arguments` lint

## [0.1.3] - 2026-03-23

### Added
- **Mute local output** — silences laptop speakers while streaming; volume buttons still control stream level, endpoint is re-muted automatically
- Receiver script now kills stale `ncat` processes between connection cycles
- `ncat` idle timeout (`-i 30s`) to recover from dead connections (e.g. PC sleep/lock)

### Changed
- Minimize to tray is now always enabled (removed toggle from UI)
- Removed external watchdog timer — receiver script handles recovery internally

### Fixed
- Volume buttons had no effect on stream when mute local output was enabled
- Receiver watchdog was restarting the service during active streams due to stale FIN-WAIT-2 sockets

## [0.1.2] - 2026-03-17

### Fixed
- **Audio stutter on dialog open** — moved TCP writes to a dedicated writer thread so the capture loop never blocks on network I/O
- **Stutter on first Save dialog** — inject silence when WASAPI stops delivering buffers during system dialog loading
- **Chunk drops under brief TCP delays** — increased bounded channel capacity (3 → 16) and TCP send buffer (1920 → 8192 bytes)
- **Device names not showing** — fixed PROPVARIANT string extraction to properly read `PKEY_Device_FriendlyName` from WASAPI; devices now show real names instead of "Audio Device 1"

### Added
- **Start minimized only at boot** — `--minimized` flag passed via registry so the window only hides when launched by Windows startup, not when the user opens the app manually
- Receiver watchdog systemd timer — restarts the ALSA receiver if port 4714 has no activity

### Changed
- WASAPI capture buffer restored to 10ms for lower latency
- README updated with watchdog setup and corrected latency figures

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

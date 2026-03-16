use pulse_stream::audio::{AudioStreamer, DeviceInfo, ProcessInfo, Stats, StreamState};
use std::time::Duration;

// ==================== Data structures ====================

#[test]
fn stream_state_debug_format() {
    let state = StreamState::Disconnected;
    assert_eq!(format!("{:?}", state), "Disconnected");

    let state = StreamState::Connecting;
    assert_eq!(format!("{:?}", state), "Connecting");

    let state = StreamState::Connected;
    assert_eq!(format!("{:?}", state), "Connected");

    let state = StreamState::Streaming;
    assert_eq!(format!("{:?}", state), "Streaming");
}

#[test]
fn stream_state_clone() {
    let state = StreamState::Streaming;
    let cloned = state.clone();
    assert!(matches!(cloned, StreamState::Streaming));
}

#[test]
fn device_info_creation() {
    let info = DeviceInfo {
        id: "dev-001".to_string(),
        name: "Speakers".to_string(),
    };
    assert_eq!(info.id, "dev-001");
    assert_eq!(info.name, "Speakers");
}

#[test]
fn device_info_clone() {
    let info = DeviceInfo {
        id: "dev-001".to_string(),
        name: "Speakers".to_string(),
    };
    let cloned = info.clone();
    assert_eq!(cloned.id, info.id);
    assert_eq!(cloned.name, info.name);
}

#[test]
fn device_info_empty_id_for_default() {
    let info = DeviceInfo {
        id: String::new(),
        name: "Default".to_string(),
    };
    assert!(info.id.is_empty());
    assert_eq!(info.name, "Default");
}

#[test]
fn process_info_creation() {
    let info = ProcessInfo {
        pid: 1234,
        name: "firefox".to_string(),
    };
    assert_eq!(info.pid, 1234);
    assert_eq!(info.name, "firefox");
}

#[test]
fn process_info_clone() {
    let info = ProcessInfo {
        pid: 5678,
        name: "chrome".to_string(),
    };
    let cloned = info.clone();
    assert_eq!(cloned.pid, info.pid);
    assert_eq!(cloned.name, info.name);
}

#[test]
fn stats_creation() {
    let stats = Stats {
        bytes_sent: 1024 * 1024,
        bitrate_kbps: 1536.0,
        uptime: Duration::from_secs(120),
        client_latency_ms: 15.5,
        drops: 0,
        capture_format: "48.0kHz 2ch 32bit".to_string(),
    };
    assert_eq!(stats.bytes_sent, 1_048_576);
    assert!((stats.bitrate_kbps - 1536.0).abs() < f64::EPSILON);
    assert_eq!(stats.uptime.as_secs(), 120);
    assert!((stats.client_latency_ms - 15.5).abs() < f64::EPSILON);
    assert_eq!(stats.drops, 0);
    assert_eq!(stats.capture_format, "48.0kHz 2ch 32bit");
}

#[test]
fn stats_clone() {
    let stats = Stats {
        bytes_sent: 500,
        bitrate_kbps: 256.0,
        uptime: Duration::from_secs(60),
        client_latency_ms: 10.0,
        drops: 2,
        capture_format: "44.1kHz 2ch 16bit".to_string(),
    };
    let cloned = stats.clone();
    assert_eq!(cloned.bytes_sent, stats.bytes_sent);
    assert_eq!(cloned.drops, stats.drops);
}

// ==================== AudioStreamer lifecycle ====================

#[test]
fn streamer_initial_state() {
    let streamer = AudioStreamer::new();
    assert!(!streamer.is_running());
}

#[test]
fn streamer_event_receiver_is_valid() {
    let streamer = AudioStreamer::new();
    let rx = streamer.event_receiver();
    assert!(rx.try_recv().is_err());
}

#[test]
fn streamer_stop_when_not_running() {
    let mut streamer = AudioStreamer::new();
    streamer.stop();
    assert!(!streamer.is_running());
}

#[test]
fn streamer_double_stop_is_safe() {
    let mut streamer = AudioStreamer::new();
    streamer.stop();
    streamer.stop();
    assert!(!streamer.is_running());
}

#[test]
fn streamer_drop_stops_cleanly() {
    let streamer = AudioStreamer::new();
    let _rx = streamer.event_receiver();
    drop(streamer);
}

#[test]
fn streamer_start_sets_running() {
    let mut streamer = AudioStreamer::new();
    let _rx = streamer.event_receiver();

    streamer.start("127.0.0.1".to_string(), 4714, 48000, 2, None, None);
    assert!(streamer.is_running());

    streamer.stop();
    assert!(!streamer.is_running());
}

#[test]
fn streamer_emits_events_on_connection_failure() {
    let mut streamer = AudioStreamer::new();
    let rx = streamer.event_receiver();

    streamer.start(
        "127.0.0.1".to_string(),
        1, // unlikely port
        48000,
        2,
        None,
        None,
    );

    std::thread::sleep(Duration::from_millis(500));
    streamer.stop();

    let mut got_connecting = false;
    let mut got_log = false;
    let mut got_disconnected = false;

    while let Ok(event) = rx.try_recv() {
        match event {
            pulse_stream::audio::AudioEvent::StateChanged(StreamState::Connecting) => {
                got_connecting = true
            }
            pulse_stream::audio::AudioEvent::Log(_) => got_log = true,
            pulse_stream::audio::AudioEvent::StateChanged(StreamState::Disconnected) => {
                got_disconnected = true
            }
            _ => {}
        }
    }

    assert!(got_connecting, "should emit Connecting state");
    assert!(got_log, "should emit log messages");
    assert!(got_disconnected, "should emit Disconnected on stop");
}

#[test]
fn streamer_ignores_double_start() {
    let mut streamer = AudioStreamer::new();
    let _rx = streamer.event_receiver();

    streamer.start("127.0.0.1".to_string(), 1, 48000, 2, None, None);
    assert!(streamer.is_running());

    streamer.start("127.0.0.1".to_string(), 2, 48000, 2, None, None);
    assert!(streamer.is_running());

    streamer.stop();
}

// ==================== Device enumeration ====================

#[test]
fn get_output_devices_always_has_default() {
    let devices = pulse_stream::audio::get_output_devices();
    assert!(!devices.is_empty());
    assert_eq!(devices[0].name, "Default");
    assert!(devices[0].id.is_empty());
}

#[test]
fn get_audio_processes_returns_vec() {
    let processes = pulse_stream::audio::get_audio_processes();
    // On CI or headless, this may be empty — just confirm it doesn't crash
    let _ = processes.len();
}

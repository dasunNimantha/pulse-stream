use pulse_stream::audio::{
    AudioEvent, AudioStreamer, CaptureMode, DeviceInfo, ProcessInfo, Stats, StreamConfig,
    StreamState,
};
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

    streamer.start(pulse_stream::audio::StreamConfig {
        server: "127.0.0.1".to_string(),
        port: 4714,
        rate: 48000,
        channels: 2,
        device_id: None,
        process_id: None,
        mute_local_output: false,
        capture_mode: CaptureMode::WasapiLoopback,
    });
    assert!(streamer.is_running());

    streamer.stop();
    assert!(!streamer.is_running());
}

#[test]
fn streamer_emits_events_on_connection_failure() {
    let mut streamer = AudioStreamer::new();
    let rx = streamer.event_receiver();

    streamer.start(pulse_stream::audio::StreamConfig {
        server: "127.0.0.1".to_string(),
        port: 1,
        rate: 48000,
        channels: 2,
        device_id: None,
        process_id: None,
        mute_local_output: false,
        capture_mode: CaptureMode::WasapiLoopback,
    });

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

    streamer.start(pulse_stream::audio::StreamConfig {
        server: "127.0.0.1".to_string(),
        port: 1,
        rate: 48000,
        channels: 2,
        device_id: None,
        process_id: None,
        mute_local_output: false,
        capture_mode: CaptureMode::WasapiLoopback,
    });
    assert!(streamer.is_running());

    streamer.start(pulse_stream::audio::StreamConfig {
        server: "127.0.0.1".to_string(),
        port: 2,
        rate: 48000,
        channels: 2,
        device_id: None,
        process_id: None,
        mute_local_output: false,
        capture_mode: CaptureMode::WasapiLoopback,
    });
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
    let _ = processes.len();
}

// ==================== StreamConfig ====================

#[test]
fn stream_config_construction() {
    let cfg = StreamConfig {
        server: "10.0.0.1".to_string(),
        port: 5000,
        rate: 96000,
        channels: 6,
        device_id: Some("dev-1".to_string()),
        process_id: Some(1234),
        mute_local_output: true,
        capture_mode: CaptureMode::WasapiLoopback,
    };
    assert_eq!(cfg.server, "10.0.0.1");
    assert_eq!(cfg.port, 5000);
    assert_eq!(cfg.rate, 96000);
    assert_eq!(cfg.channels, 6);
    assert_eq!(cfg.device_id, Some("dev-1".to_string()));
    assert_eq!(cfg.process_id, Some(1234));
    assert!(cfg.mute_local_output);
    assert_eq!(cfg.capture_mode, CaptureMode::WasapiLoopback);
}

#[test]
fn stream_config_none_fields() {
    let cfg = StreamConfig {
        server: "127.0.0.1".to_string(),
        port: 4714,
        rate: 48000,
        channels: 2,
        device_id: None,
        process_id: None,
        mute_local_output: false,
        capture_mode: CaptureMode::WasapiLoopback,
    };
    assert!(cfg.device_id.is_none());
    assert!(cfg.process_id.is_none());
    assert!(!cfg.mute_local_output);
}

// ==================== AudioEvent variants ====================

#[test]
fn audio_event_state_changed_clone() {
    let event = AudioEvent::StateChanged(StreamState::Streaming);
    let cloned = event.clone();
    assert!(matches!(
        cloned,
        AudioEvent::StateChanged(StreamState::Streaming)
    ));
}

#[test]
fn audio_event_log_clone() {
    let event = AudioEvent::Log("test message".to_string());
    let cloned = event.clone();
    if let AudioEvent::Log(msg) = cloned {
        assert_eq!(msg, "test message");
    } else {
        panic!("expected Log variant");
    }
}

#[test]
fn audio_event_stats_updated_clone() {
    let event = AudioEvent::StatsUpdated(Stats {
        bytes_sent: 1024,
        bitrate_kbps: 256.0,
        uptime: Duration::from_secs(10),
        client_latency_ms: 5.0,
        drops: 1,
        capture_format: "48.0kHz 2ch 16bit".to_string(),
    });
    let cloned = event.clone();
    if let AudioEvent::StatsUpdated(s) = cloned {
        assert_eq!(s.bytes_sent, 1024);
        assert_eq!(s.drops, 1);
    } else {
        panic!("expected StatsUpdated variant");
    }
}

#[test]
fn audio_event_volume_changed_clone() {
    let event = AudioEvent::VolumeChanged {
        volume: 0.75,
        muted: false,
    };
    let cloned = event.clone();
    if let AudioEvent::VolumeChanged { volume, muted } = cloned {
        assert!((volume - 0.75).abs() < f32::EPSILON);
        assert!(!muted);
    } else {
        panic!("expected VolumeChanged variant");
    }
}

#[test]
fn audio_event_debug_format() {
    let event = AudioEvent::Log("hello".to_string());
    let dbg = format!("{:?}", event);
    assert!(dbg.contains("Log"));
    assert!(dbg.contains("hello"));
}

// ==================== AudioStreamer::default ====================

#[test]
fn streamer_default_equivalent_to_new() {
    let a = AudioStreamer::new();
    let b = AudioStreamer::default();
    assert!(!a.is_running());
    assert!(!b.is_running());
}

// ==================== CaptureMode ====================

#[test]
fn capture_mode_clone() {
    let mode = CaptureMode::VbCable;
    let cloned = mode.clone();
    assert_eq!(cloned, CaptureMode::VbCable);
}

#[test]
fn capture_mode_eq() {
    assert_eq!(CaptureMode::WasapiLoopback, CaptureMode::WasapiLoopback);
    assert_eq!(CaptureMode::VbCable, CaptureMode::VbCable);
    assert_ne!(CaptureMode::WasapiLoopback, CaptureMode::VbCable);
}

#[test]
fn capture_mode_debug() {
    let dbg = format!("{:?}", CaptureMode::WasapiLoopback);
    assert!(dbg.contains("WasapiLoopback"));
    let dbg = format!("{:?}", CaptureMode::VbCable);
    assert!(dbg.contains("VbCable"));
}

#[test]
fn stream_config_with_vbcable_mode() {
    let cfg = StreamConfig {
        server: "10.0.0.1".to_string(),
        port: 4714,
        rate: 48000,
        channels: 2,
        device_id: Some("vb-cable-id".to_string()),
        process_id: None,
        mute_local_output: false,
        capture_mode: CaptureMode::VbCable,
    };
    assert_eq!(cfg.capture_mode, CaptureMode::VbCable);
    assert_eq!(cfg.device_id, Some("vb-cable-id".to_string()));
    assert!(!cfg.mute_local_output);
}

#[test]
fn detect_vb_cable_does_not_crash() {
    let result = pulse_stream::audio::detect_vb_cable();
    let _ = result;
}

// ==================== Connection failure event order ====================

#[test]
fn streamer_connection_failure_emits_events_in_order() {
    let mut streamer = AudioStreamer::new();
    let rx = streamer.event_receiver();

    streamer.start(StreamConfig {
        server: "127.0.0.1".to_string(),
        port: 1,
        rate: 48000,
        channels: 2,
        device_id: None,
        process_id: None,
        mute_local_output: false,
        capture_mode: CaptureMode::WasapiLoopback,
    });

    std::thread::sleep(Duration::from_millis(500));
    streamer.stop();

    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    let mut saw_connecting = false;
    let mut saw_log_after_connecting = false;
    let mut saw_disconnected_after_log = false;

    for event in &events {
        match event {
            AudioEvent::StateChanged(StreamState::Connecting) => {
                saw_connecting = true;
            }
            AudioEvent::Log(_) if saw_connecting => {
                saw_log_after_connecting = true;
            }
            AudioEvent::StateChanged(StreamState::Disconnected) if saw_log_after_connecting => {
                saw_disconnected_after_log = true;
            }
            _ => {}
        }
    }

    assert!(saw_connecting, "should emit Connecting first");
    assert!(saw_log_after_connecting, "should emit Log after Connecting");
    assert!(
        saw_disconnected_after_log,
        "should emit Disconnected after Log"
    );
}

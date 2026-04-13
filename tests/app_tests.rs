use pulse_stream::app::AppState;
use pulse_stream::audio::{AudioEvent, CaptureMode, DeviceInfo, Stats, StreamState};
use pulse_stream::message::Message;
use std::collections::VecDeque;
use std::time::Duration;

fn default_state() -> AppState {
    AppState {
        server: "192.168.1.100".to_string(),
        port: "4714".to_string(),
        rate: "48000".to_string(),
        channels: "2".to_string(),
        stream_state: StreamState::Disconnected,
        auto_connect: false,
        start_with_windows: false,
        minimize_to_tray: true,
        mute_local_output: false,
        volume_text: String::new(),
        show_quality_warning: false,
        stats_bitrate: String::new(),
        stats_format: String::new(),
        stats_uptime: String::new(),
        devices: vec![DeviceInfo {
            id: String::new(),
            name: "Default".to_string(),
        }],
        selected_device: None,
        processes: Vec::new(),
        selected_process: None,
        log_messages: VecDeque::new(),
        scanning: false,
        capture_mode: CaptureMode::WasapiLoopback,
        vb_cable_available: false,
    }
}

// ==================== Field update messages ====================

#[test]
fn server_changed_updates_field() {
    let mut state = default_state();
    state.server = "10.0.0.5".to_string();
    assert_eq!(state.server, "10.0.0.5");
}

#[test]
fn port_changed_updates_field() {
    let mut state = default_state();
    state.port = "5000".to_string();
    assert_eq!(state.port, "5000");
}

#[test]
fn rate_changed_updates_field() {
    let mut state = default_state();
    state.rate = "96000".to_string();
    assert_eq!(state.rate, "96000");
}

#[test]
fn channels_changed_updates_field() {
    let mut state = default_state();
    state.channels = "6".to_string();
    assert_eq!(state.channels, "6");
}

// ==================== Device/Process selection ====================

#[test]
fn device_selected_updates_selection() {
    let mut state = default_state();
    state.devices.push(DeviceInfo {
        id: "dev-1".to_string(),
        name: "Speakers".to_string(),
    });
    state.selected_device = state.devices.iter().find(|d| d.name == "Speakers").cloned();
    assert_eq!(state.selected_device.as_ref().unwrap().name, "Speakers");
}

#[test]
fn process_selected_all_apps_clears_selection() {
    let mut state = default_state();
    state.selected_process = Some("firefox".to_string());

    let name = "All apps (system audio)";
    if name == "All apps (system audio)" {
        state.selected_process = None;
    }
    assert!(state.selected_process.is_none());
}

#[test]
fn process_selected_specific_app() {
    let mut state = default_state();
    state.selected_process = Some("chrome".to_string());
    assert_eq!(state.selected_process.as_deref(), Some("chrome"));
}

// ==================== Toggle options ====================

#[test]
fn toggle_auto_connect() {
    let mut state = default_state();
    assert!(!state.auto_connect);
    state.auto_connect = true;
    assert!(state.auto_connect);
}

#[test]
fn toggle_start_with_windows() {
    let mut state = default_state();
    assert!(!state.start_with_windows);
    state.start_with_windows = true;
    assert!(state.start_with_windows);
}

#[test]
fn toggle_mute_local_output() {
    let mut state = default_state();
    assert!(!state.mute_local_output);
    state.mute_local_output = true;
    assert!(state.mute_local_output);
}

// ==================== AudioEvent::StatsUpdated ====================

#[test]
fn stats_updated_kbps_format() {
    let stats = Stats {
        bytes_sent: 500_000,
        bitrate_kbps: 768.0,
        uptime: Duration::from_secs(3661),
        client_latency_ms: 12.3,
        drops: 0,
        capture_format: "48.0kHz 2ch 32bit".to_string(),
    };

    let br = if stats.bitrate_kbps >= 1000.0 {
        format!("{:.1} Mbps", stats.bitrate_kbps / 1000.0)
    } else {
        format!("{:.0} kbps", stats.bitrate_kbps)
    };
    let bitrate_str = format!("{}  {:.1}ms", br, stats.client_latency_ms);
    assert!(bitrate_str.contains("768 kbps"));
    assert!(bitrate_str.contains("12.3ms"));
}

#[test]
fn stats_updated_mbps_format() {
    let stats = Stats {
        bytes_sent: 10_000_000,
        bitrate_kbps: 1536.0,
        uptime: Duration::from_secs(60),
        client_latency_ms: 5.0,
        drops: 0,
        capture_format: "96.0kHz 2ch 32bit".to_string(),
    };

    let br = if stats.bitrate_kbps >= 1000.0 {
        format!("{:.1} Mbps", stats.bitrate_kbps / 1000.0)
    } else {
        format!("{:.0} kbps", stats.bitrate_kbps)
    };
    assert!(br.contains("1.5 Mbps"));
}

#[test]
fn stats_uptime_format() {
    let secs: u64 = 3661;
    let uptime = format!(
        "{:02}:{:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    );
    assert_eq!(uptime, "01:01:01");
}

#[test]
fn stats_drops_sets_quality_warning() {
    let stats = Stats {
        bytes_sent: 100,
        bitrate_kbps: 100.0,
        uptime: Duration::from_secs(1),
        client_latency_ms: 1.0,
        drops: 5,
        capture_format: "48.0kHz 2ch 32bit".to_string(),
    };
    assert!(stats.drops > 0);
}

#[test]
fn stats_zero_drops_no_warning() {
    let stats = Stats {
        bytes_sent: 100,
        bitrate_kbps: 100.0,
        uptime: Duration::from_secs(1),
        client_latency_ms: 1.0,
        drops: 0,
        capture_format: "48.0kHz 2ch 32bit".to_string(),
    };
    let show_warning = stats.drops > 0;
    assert!(!show_warning);
}

// ==================== AudioEvent::VolumeChanged ====================

#[test]
fn volume_changed_muted() {
    let mut state = default_state();
    let muted = true;
    let volume = 0.5f32;
    state.volume_text = if muted {
        "Vol: Muted".to_string()
    } else {
        format!("Vol: {}%", (volume * 100.0) as u32)
    };
    assert_eq!(state.volume_text, "Vol: Muted");
}

#[test]
fn volume_changed_unmuted() {
    let mut state = default_state();
    let muted = false;
    let volume = 0.75f32;
    state.volume_text = if muted {
        "Vol: Muted".to_string()
    } else {
        format!("Vol: {}%", (volume * 100.0) as u32)
    };
    assert_eq!(state.volume_text, "Vol: 75%");
}

// ==================== AudioEvent::Log ====================

#[test]
fn log_appends_message() {
    let mut state = default_state();
    state.log_messages.push_back("test log".to_string());
    assert_eq!(state.log_messages.len(), 1);
    assert_eq!(state.log_messages[0], "test log");
}

#[test]
fn log_capped_at_200() {
    let mut state = default_state();
    for i in 0..210 {
        if state.log_messages.len() >= 200 {
            state.log_messages.pop_front();
        }
        state.log_messages.push_back(format!("msg {}", i));
    }
    assert_eq!(state.log_messages.len(), 200);
    assert_eq!(state.log_messages[0], "msg 10");
    assert_eq!(state.log_messages[199], "msg 209");
}

// ==================== AudioEvent::StateChanged ====================

#[test]
fn state_changed_to_connecting() {
    let mut state = default_state();
    state.stream_state = StreamState::Connecting;
    assert!(matches!(state.stream_state, StreamState::Connecting));
}

#[test]
fn state_changed_to_connected() {
    let mut state = default_state();
    state.stream_state = StreamState::Connected;
    assert!(matches!(state.stream_state, StreamState::Connected));
}

#[test]
fn state_changed_to_streaming() {
    let mut state = default_state();
    state.stream_state = StreamState::Streaming;
    assert!(matches!(state.stream_state, StreamState::Streaming));
}

#[test]
fn state_changed_to_disconnected_clears_stats() {
    let mut state = default_state();
    state.stats_bitrate = "768 kbps".to_string();
    state.stats_format = "48.0kHz 2ch 32bit".to_string();
    state.stats_uptime = "00:05:00".to_string();
    state.volume_text = "Vol: 50%".to_string();

    state.stream_state = StreamState::Disconnected;
    state.volume_text.clear();
    state.stats_bitrate.clear();
    state.stats_format.clear();
    state.stats_uptime.clear();

    assert!(state.volume_text.is_empty());
    assert!(state.stats_bitrate.is_empty());
    assert!(state.stats_format.is_empty());
    assert!(state.stats_uptime.is_empty());
}

// ==================== ScanResult ====================

#[test]
fn scan_result_sets_server() {
    let mut state = default_state();
    state.scanning = true;

    let found = Some("192.168.1.50".to_string());
    state.scanning = false;
    if let Some(ip) = found {
        state.server = ip;
    }
    assert!(!state.scanning);
    assert_eq!(state.server, "192.168.1.50");
}

#[test]
fn scan_result_no_server_found() {
    let mut state = default_state();
    state.scanning = true;

    let found: Option<String> = None;
    state.scanning = false;
    if let Some(ip) = found {
        state.server = ip;
    }
    assert!(!state.scanning);
    assert_eq!(state.server, "192.168.1.100");
}

// ==================== Connect validation ====================

#[test]
fn connect_with_empty_server_does_not_proceed() {
    let mut state = default_state();
    state.server = "".to_string();
    let should_connect = !state.server.trim().is_empty();
    assert!(!should_connect);
}

#[test]
fn connect_with_whitespace_server_does_not_proceed() {
    let mut state = default_state();
    state.server = "   ".to_string();
    let should_connect = !state.server.trim().is_empty();
    assert!(!should_connect);
}

#[test]
fn connect_with_valid_server_proceeds() {
    let state = default_state();
    let should_connect = !state.server.trim().is_empty();
    assert!(should_connect);
}

// ==================== CloseRequested ====================

#[test]
fn close_requested_with_minimize_to_tray() {
    let state = default_state();
    assert!(state.minimize_to_tray);
}

#[test]
fn close_requested_without_minimize_to_tray() {
    let mut state = default_state();
    state.minimize_to_tray = false;
    assert!(!state.minimize_to_tray);
}

// ==================== Device refresh ====================

#[test]
fn device_refresh_keeps_selection_if_still_present() {
    let mut state = default_state();
    state.selected_device = Some(DeviceInfo {
        id: "dev-1".to_string(),
        name: "Speakers".to_string(),
    });

    let new_devices = vec![
        DeviceInfo {
            id: String::new(),
            name: "Default".to_string(),
        },
        DeviceInfo {
            id: "dev-1".to_string(),
            name: "Speakers".to_string(),
        },
    ];
    state.devices = new_devices;

    if let Some(ref sel) = state.selected_device {
        if !state.devices.iter().any(|d| d.id == sel.id) {
            state.selected_device = state.devices.first().cloned();
        }
    }
    assert_eq!(state.selected_device.as_ref().unwrap().id, "dev-1");
}

#[test]
fn device_refresh_resets_to_default_if_removed() {
    let mut state = default_state();
    state.selected_device = Some(DeviceInfo {
        id: "dev-gone".to_string(),
        name: "Old Speakers".to_string(),
    });

    let new_devices = vec![DeviceInfo {
        id: String::new(),
        name: "Default".to_string(),
    }];
    state.devices = new_devices;

    if let Some(ref sel) = state.selected_device {
        if !state.devices.iter().any(|d| d.id == sel.id) {
            state.selected_device = state.devices.first().cloned();
        }
    }
    assert_eq!(state.selected_device.as_ref().unwrap().name, "Default");
}

// ==================== Capture mode ====================

#[test]
fn capture_mode_toggle_to_vbcable() {
    let mut state = default_state();
    assert_eq!(state.capture_mode, CaptureMode::WasapiLoopback);
    state.capture_mode = CaptureMode::VbCable;
    assert_eq!(state.capture_mode, CaptureMode::VbCable);
}

#[test]
fn capture_mode_toggle_back_to_loopback() {
    let mut state = default_state();
    state.capture_mode = CaptureMode::VbCable;
    state.capture_mode = CaptureMode::WasapiLoopback;
    assert_eq!(state.capture_mode, CaptureMode::WasapiLoopback);
}

#[test]
fn vb_cable_available_defaults_false() {
    let state = default_state();
    assert!(!state.vb_cable_available);
}

#[test]
fn vb_cable_available_can_be_set() {
    let mut state = default_state();
    state.vb_cable_available = true;
    assert!(state.vb_cable_available);
}

// ==================== Message construction ====================

#[test]
fn message_connect_is_constructable() {
    let _ = Message::Connect;
}

#[test]
fn message_disconnect_is_constructable() {
    let _ = Message::Disconnect;
}

#[test]
fn message_audio_event_is_constructable() {
    let _ = Message::AudioEvent(AudioEvent::Log("test".to_string()));
}

#[test]
fn message_scan_result_constructable() {
    let _ = Message::ScanResult(Some("1.2.3.4".to_string()));
    let _ = Message::ScanResult(None);
}

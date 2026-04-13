use pulse_stream::audio::{AudioEvent, CaptureMode, Stats, StreamState};
use pulse_stream::message::Message;
use std::time::Duration;

#[test]
fn message_connect_constructable() {
    let msg = Message::Connect;
    let _ = format!("{:?}", msg);
}

#[test]
fn message_disconnect_constructable() {
    let msg = Message::Disconnect;
    let _ = format!("{:?}", msg);
}

#[test]
fn message_server_changed_constructable() {
    let msg = Message::ServerChanged("10.0.0.1".to_string());
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("ServerChanged"));
}

#[test]
fn message_port_changed_constructable() {
    let msg = Message::PortChanged("5000".to_string());
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("PortChanged"));
}

#[test]
fn message_rate_changed_constructable() {
    let msg = Message::RateChanged("96000".to_string());
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("RateChanged"));
}

#[test]
fn message_channels_changed_constructable() {
    let msg = Message::ChannelsChanged("6".to_string());
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("ChannelsChanged"));
}

#[test]
fn message_device_selected_constructable() {
    let msg = Message::DeviceSelected("Speakers".to_string());
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("DeviceSelected"));
}

#[test]
fn message_process_selected_constructable() {
    let msg = Message::ProcessSelected("firefox".to_string());
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("ProcessSelected"));
}

#[test]
fn message_capture_mode_changed_constructable() {
    let msg = Message::CaptureModeChanged(CaptureMode::VbCable);
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("CaptureModeChanged"));
    assert!(dbg.contains("VbCable"));
}

#[test]
fn message_capture_mode_changed_loopback() {
    let msg = Message::CaptureModeChanged(CaptureMode::WasapiLoopback);
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("WasapiLoopback"));
}

#[test]
fn message_toggle_auto_connect_constructable() {
    let msg = Message::ToggleAutoConnect(true);
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("ToggleAutoConnect"));
}

#[test]
fn message_toggle_start_with_windows_constructable() {
    let msg = Message::ToggleStartWithWindows(false);
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("ToggleStartWithWindows"));
}

#[test]
fn message_toggle_mute_local_output_constructable() {
    let msg = Message::ToggleMuteLocalOutput(true);
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("ToggleMuteLocalOutput"));
}

#[test]
fn message_toggle_theme_constructable() {
    let msg = Message::ToggleTheme;
    let _ = format!("{:?}", msg);
}

#[test]
fn message_audio_event_state_changed() {
    let msg = Message::AudioEvent(AudioEvent::StateChanged(StreamState::Connecting));
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("AudioEvent"));
    assert!(dbg.contains("Connecting"));
}

#[test]
fn message_audio_event_log() {
    let msg = Message::AudioEvent(AudioEvent::Log("connected".to_string()));
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("connected"));
}

#[test]
fn message_audio_event_stats() {
    let msg = Message::AudioEvent(AudioEvent::StatsUpdated(Stats {
        bytes_sent: 100,
        bitrate_kbps: 256.0,
        uptime: Duration::from_secs(10),
        client_latency_ms: 5.0,
        drops: 0,
        capture_format: "48kHz".to_string(),
    }));
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("StatsUpdated"));
}

#[test]
fn message_audio_event_volume() {
    let msg = Message::AudioEvent(AudioEvent::VolumeChanged {
        volume: 0.5,
        muted: true,
    });
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("VolumeChanged"));
}

#[test]
fn message_scan_servers_constructable() {
    let msg = Message::ScanServers;
    let _ = format!("{:?}", msg);
}

#[test]
fn message_scan_result_some() {
    let msg = Message::ScanResult(Some("192.168.1.50".to_string()));
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("ScanResult"));
}

#[test]
fn message_scan_result_none() {
    let msg = Message::ScanResult(None);
    let dbg = format!("{:?}", msg);
    assert!(dbg.contains("None"));
}

#[test]
fn message_close_requested_constructable() {
    let _ = Message::CloseRequested;
}

#[test]
fn message_tray_restore_constructable() {
    let _ = Message::TrayRestore;
}

#[test]
fn message_exit_app_constructable() {
    let _ = Message::ExitApp;
}

#[test]
fn message_tick_constructable() {
    let _ = Message::Tick;
}

#[test]
fn message_noop_constructable() {
    let _ = Message::Noop;
}

#[test]
fn message_clone_works() {
    let msg = Message::ServerChanged("hello".to_string());
    let cloned = msg.clone();
    let dbg = format!("{:?}", cloned);
    assert!(dbg.contains("hello"));
}

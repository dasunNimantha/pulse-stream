use pulse_stream::app::AppState;
use pulse_stream::audio::{CaptureMode, DeviceInfo, StreamState};

use pulse_stream::view::{
    all_fields_valid, is_valid_channels, is_valid_port, is_valid_rate, is_valid_server,
};

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
        log_messages: std::collections::VecDeque::new(),
        scanning: false,
        capture_mode: CaptureMode::WasapiLoopback,
        vb_cable_available: false,
    }
}

// ==================== Server validation ====================

#[test]
fn valid_server_standard_ip() {
    assert!(is_valid_server("192.168.1.1"));
    assert!(is_valid_server("10.0.0.1"));
    assert!(is_valid_server("172.16.0.1"));
    assert!(is_valid_server("0.0.0.0"));
    assert!(is_valid_server("255.255.255.255"));
}

#[test]
fn valid_server_with_whitespace() {
    assert!(is_valid_server("  192.168.1.1  "));
    assert!(is_valid_server(" 10.0.0.1"));
}

#[test]
fn invalid_server_empty() {
    assert!(!is_valid_server(""));
    assert!(!is_valid_server("   "));
}

#[test]
fn invalid_server_too_few_octets() {
    assert!(!is_valid_server("192.168.1"));
    assert!(!is_valid_server("10.0"));
    assert!(!is_valid_server("1"));
}

#[test]
fn invalid_server_too_many_octets() {
    assert!(!is_valid_server("192.168.1.1.1"));
}

#[test]
fn invalid_server_octet_out_of_range() {
    assert!(!is_valid_server("256.0.0.1"));
    assert!(!is_valid_server("192.168.1.999"));
    assert!(!is_valid_server("192.168.300.1"));
}

#[test]
fn invalid_server_non_numeric() {
    assert!(!is_valid_server("abc.def.ghi.jkl"));
    assert!(!is_valid_server("192.168.1.x"));
    assert!(!is_valid_server("host.example.com"));
}

#[test]
fn invalid_server_negative_octet() {
    assert!(!is_valid_server("-1.0.0.1"));
    assert!(!is_valid_server("192.168.1.-5"));
}

// ==================== Port validation ====================

#[test]
fn valid_port_standard() {
    assert!(is_valid_port("4714"));
    assert!(is_valid_port("1"));
    assert!(is_valid_port("80"));
    assert!(is_valid_port("443"));
    assert!(is_valid_port("8080"));
    assert!(is_valid_port("65535"));
}

#[test]
fn invalid_port_zero() {
    assert!(!is_valid_port("0"));
}

#[test]
fn invalid_port_negative() {
    assert!(!is_valid_port("-1"));
    assert!(!is_valid_port("-4714"));
}

#[test]
fn invalid_port_overflow() {
    assert!(!is_valid_port("65536"));
    assert!(!is_valid_port("100000"));
}

#[test]
fn invalid_port_non_numeric() {
    assert!(!is_valid_port("abc"));
    assert!(!is_valid_port(""));
    assert!(!is_valid_port("80.5"));
}

// ==================== Sample rate validation ====================

#[test]
fn valid_rate_common_values() {
    assert!(is_valid_rate("44100"));
    assert!(is_valid_rate("48000"));
    assert!(is_valid_rate("96000"));
    assert!(is_valid_rate("192000"));
    assert!(is_valid_rate("384000"));
}

#[test]
fn valid_rate_boundaries() {
    assert!(is_valid_rate("1000"));
    assert!(is_valid_rate("384000"));
}

#[test]
fn invalid_rate_too_low() {
    assert!(!is_valid_rate("999"));
    assert!(!is_valid_rate("0"));
    assert!(!is_valid_rate("100"));
}

#[test]
fn invalid_rate_too_high() {
    assert!(!is_valid_rate("384001"));
    assert!(!is_valid_rate("1000000"));
}

#[test]
fn invalid_rate_non_numeric() {
    assert!(!is_valid_rate("abc"));
    assert!(!is_valid_rate(""));
    assert!(!is_valid_rate("48000.5"));
}

// ==================== Channels validation ====================

#[test]
fn valid_channels() {
    assert!(is_valid_channels("1"));
    assert!(is_valid_channels("2"));
    assert!(is_valid_channels("6"));
    assert!(is_valid_channels("8"));
}

#[test]
fn invalid_channels_zero() {
    assert!(!is_valid_channels("0"));
}

#[test]
fn invalid_channels_too_high() {
    assert!(!is_valid_channels("9"));
    assert!(!is_valid_channels("16"));
    assert!(!is_valid_channels("256"));
}

#[test]
fn invalid_channels_non_numeric() {
    assert!(!is_valid_channels("abc"));
    assert!(!is_valid_channels(""));
    assert!(!is_valid_channels("-1"));
}

// ==================== Combined validation ====================

#[test]
fn all_fields_valid_with_good_values() {
    let state = default_state();
    assert!(all_fields_valid(&state));
}

#[test]
fn all_fields_invalid_server() {
    let mut state = default_state();
    state.server = "invalid".to_string();
    assert!(!all_fields_valid(&state));
}

#[test]
fn all_fields_invalid_port() {
    let mut state = default_state();
    state.port = "0".to_string();
    assert!(!all_fields_valid(&state));
}

#[test]
fn all_fields_invalid_rate() {
    let mut state = default_state();
    state.rate = "500".to_string();
    assert!(!all_fields_valid(&state));
}

#[test]
fn all_fields_invalid_channels() {
    let mut state = default_state();
    state.channels = "0".to_string();
    assert!(!all_fields_valid(&state));
}

#[test]
fn all_fields_multiple_invalid() {
    let mut state = default_state();
    state.server = "".to_string();
    state.port = "abc".to_string();
    assert!(!all_fields_valid(&state));
}

// ==================== Whitespace-only inputs ====================

#[test]
fn whitespace_only_server() {
    assert!(!is_valid_server("   "));
    assert!(!is_valid_server("\t"));
    assert!(!is_valid_server(" \n "));
}

#[test]
fn whitespace_only_port() {
    assert!(!is_valid_port("   "));
    assert!(!is_valid_port("\t"));
}

#[test]
fn whitespace_only_rate() {
    assert!(!is_valid_rate("   "));
    assert!(!is_valid_rate("\t"));
}

#[test]
fn whitespace_only_channels() {
    assert!(!is_valid_channels("   "));
    assert!(!is_valid_channels("\t"));
}

// ==================== Boundary values ====================

#[test]
fn port_boundary_max() {
    assert!(is_valid_port("65535"));
}

#[test]
fn port_boundary_one_over_max() {
    assert!(!is_valid_port("65536"));
}

#[test]
fn rate_boundary_min() {
    assert!(is_valid_rate("1000"));
}

#[test]
fn rate_boundary_one_below_min() {
    assert!(!is_valid_rate("999"));
}

#[test]
fn rate_boundary_max() {
    assert!(is_valid_rate("384000"));
}

#[test]
fn rate_boundary_one_over_max() {
    assert!(!is_valid_rate("384001"));
}

#[test]
fn channels_boundary_max() {
    assert!(is_valid_channels("8"));
}

#[test]
fn channels_boundary_one_over_max() {
    assert!(!is_valid_channels("9"));
}

#[test]
fn channels_boundary_min() {
    assert!(is_valid_channels("1"));
}

// ==================== all_fields_valid edge cases ====================

#[test]
fn all_fields_valid_empty_server_other_valid() {
    let mut state = default_state();
    state.server = "".to_string();
    assert!(!all_fields_valid(&state));
}

#[test]
fn all_fields_valid_whitespace_server_other_valid() {
    let mut state = default_state();
    state.server = "   ".to_string();
    assert!(!all_fields_valid(&state));
}

// ==================== Special characters ====================

#[test]
fn server_with_special_chars() {
    assert!(!is_valid_server("192.168.1.1;"));
    assert!(!is_valid_server("192.168.1.1:80"));
    assert!(!is_valid_server("192.168.1.1/24"));
}

#[test]
fn port_with_leading_zeros() {
    assert!(is_valid_port("0080"));
}

#[test]
fn rate_with_leading_zeros() {
    assert!(is_valid_rate("048000"));
}

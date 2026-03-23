use pulse_stream::settings::AppSettings;

#[test]
fn default_settings_values() {
    let s = AppSettings::default();
    assert!(s.server.is_empty());
    assert_eq!(s.port, 4714);
    assert_eq!(s.rate, 48000);
    assert_eq!(s.channels, 2);
    assert!(s.device_id.is_none());
    assert!(!s.auto_connect);
    assert!(s.start_with_windows);
    assert!(s.minimize_to_tray);
    assert!(!s.mute_local_output);
    assert!(s.dark_theme);
}

#[test]
fn settings_serialization_roundtrip() {
    let original = AppSettings {
        server: "10.0.0.5".to_string(),
        port: 5000,
        rate: 96000,
        channels: 6,
        device_id: Some("device-123".to_string()),
        auto_connect: true,
        start_with_windows: true,
        minimize_to_tray: false,
        mute_local_output: true,
        dark_theme: false,
    };

    let json = serde_json::to_string(&original).unwrap();
    let restored: AppSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.server, "10.0.0.5");
    assert_eq!(restored.port, 5000);
    assert_eq!(restored.rate, 96000);
    assert_eq!(restored.channels, 6);
    assert_eq!(restored.device_id, Some("device-123".to_string()));
    assert!(restored.auto_connect);
    assert!(restored.start_with_windows);
    assert!(!restored.minimize_to_tray);
    assert!(restored.mute_local_output);
    assert!(!restored.dark_theme);
}

#[test]
fn settings_deserialization_with_missing_fields() {
    let json = r#"{"server": "192.168.1.50"}"#;
    let s: AppSettings = serde_json::from_str(json).unwrap();

    assert_eq!(s.server, "192.168.1.50");
    assert_eq!(s.port, 4714);
    assert_eq!(s.rate, 48000);
    assert_eq!(s.channels, 2);
    assert!(s.device_id.is_none());
    assert!(!s.auto_connect);
    assert!(s.start_with_windows);
    assert!(s.minimize_to_tray);
    assert!(!s.mute_local_output);
    assert!(s.dark_theme);
}

#[test]
fn settings_deserialization_empty_json() {
    let json = "{}";
    let s: AppSettings = serde_json::from_str(json).unwrap();
    let d = AppSettings::default();

    assert_eq!(s.server, d.server);
    assert_eq!(s.port, d.port);
    assert_eq!(s.rate, d.rate);
    assert_eq!(s.channels, d.channels);
    assert_eq!(s.start_with_windows, d.start_with_windows);
    assert_eq!(s.minimize_to_tray, d.minimize_to_tray);
    assert_eq!(s.mute_local_output, d.mute_local_output);
    assert_eq!(s.dark_theme, d.dark_theme);
}

#[test]
fn settings_deserialization_with_extra_fields() {
    let json = r#"{
        "server": "10.0.0.1",
        "port": 4714,
        "unknown_field": "some_value",
        "another": 42
    }"#;
    let result: Result<AppSettings, _> = serde_json::from_str(json);
    assert!(result.is_ok());
    let s = result.unwrap();
    assert_eq!(s.server, "10.0.0.1");
}

#[test]
fn settings_device_id_none_serializes() {
    let s = AppSettings::default();
    let json = serde_json::to_string(&s).unwrap();
    assert!(json.contains("\"device_id\":null"));
}

#[test]
fn settings_preserve_minimize_to_tray_false() {
    let json = r#"{"minimize_to_tray": false}"#;
    let s: AppSettings = serde_json::from_str(json).unwrap();
    assert!(!s.minimize_to_tray);
}

#[test]
fn settings_preserve_start_with_windows_true() {
    let json = r#"{"start_with_windows": true}"#;
    let s: AppSettings = serde_json::from_str(json).unwrap();
    assert!(s.start_with_windows);
}

#[test]
fn settings_start_with_windows_defaults_true() {
    let json = "{}";
    let s: AppSettings = serde_json::from_str(json).unwrap();
    assert!(s.start_with_windows);
}

#[test]
fn settings_mute_local_output_defaults_false() {
    let json = "{}";
    let s: AppSettings = serde_json::from_str(json).unwrap();
    assert!(!s.mute_local_output);
}

#[test]
fn settings_preserve_mute_local_output_true() {
    let json = r#"{"mute_local_output": true}"#;
    let s: AppSettings = serde_json::from_str(json).unwrap();
    assert!(s.mute_local_output);
}

#[test]
fn settings_pretty_json_is_valid() {
    let s = AppSettings::default();
    let json = serde_json::to_string_pretty(&s).unwrap();
    let restored: AppSettings = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.port, s.port);
    assert_eq!(restored.rate, s.rate);
}

use crate::audio::{AudioEvent, CaptureMode};

#[derive(Debug, Clone)]
pub enum Message {
    Connect,
    Disconnect,
    ServerChanged(String),
    PortChanged(String),
    RateChanged(String),
    ChannelsChanged(String),
    DeviceSelected(String),
    ProcessSelected(String),
    CaptureModeChanged(CaptureMode),

    ToggleAutoConnect(bool),
    ToggleStartWithWindows(bool),
    ToggleMuteLocalOutput(bool),

    ToggleTheme,

    AudioEvent(AudioEvent),

    ScanServers,
    ScanResult(Option<String>),

    CloseRequested,
    TrayRestore,
    ExitApp,

    Tick,
    Noop,
}

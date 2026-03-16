use crate::audio::AudioEvent;

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

    ToggleAutoConnect(bool),
    ToggleStartWithWindows(bool),
    ToggleMinimizeToTray(bool),

    ToggleTheme,

    AudioEvent(AudioEvent),

    Tick,
    Noop,
}

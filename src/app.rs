use crate::audio::{AudioEvent, AudioStreamer, DeviceInfo, ProcessInfo, StreamState};
use crate::message::Message;
use crate::settings::AppSettings;
use crate::theme::{pulse_theme, ThemeMode};
use crate::view::build_view;
use iced::{Application, Command, Element, Subscription, Theme};
use std::time::Duration;

pub struct AppState {
    pub server: String,
    pub port: String,
    pub rate: String,
    pub channels: String,
    pub stream_state: StreamState,
    pub auto_connect: bool,
    pub start_with_windows: bool,
    pub minimize_to_tray: bool,
    pub volume_text: String,
    pub show_quality_warning: bool,
    pub stats_bitrate: String,
    pub stats_format: String,
    pub stats_uptime: String,
    pub devices: Vec<DeviceInfo>,
    pub selected_device: Option<DeviceInfo>,
    pub processes: Vec<ProcessInfo>,
    pub selected_process: Option<String>,
    pub log_messages: Vec<String>,
    pub level_left: f32,
    pub level_right: f32,
}

pub struct PulseStreamApp {
    state: AppState,
    theme_mode: ThemeMode,
    settings: AppSettings,
    streamer: AudioStreamer,
    audio_rx: Option<flume::Receiver<AudioEvent>>,
}

impl Application for PulseStreamApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let settings = AppSettings::load();
        let streamer = AudioStreamer::new();
        let audio_rx = Some(streamer.event_receiver());

        let devices = crate::audio::get_output_devices();
        let processes = crate::audio::get_audio_processes();

        let selected_device = if let Some(ref saved_id) = settings.device_id {
            devices.iter().find(|d| &d.id == saved_id).cloned()
        } else {
            devices.first().cloned()
        };

        let theme_mode = if settings.dark_theme {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };

        let state = AppState {
            server: settings.server.clone(),
            port: settings.port.to_string(),
            rate: settings.rate.to_string(),
            channels: settings.channels.to_string(),
            stream_state: StreamState::Disconnected,
            auto_connect: settings.auto_connect,
            start_with_windows: false,
            minimize_to_tray: settings.minimize_to_tray,
            volume_text: String::new(),
            show_quality_warning: false,
            stats_bitrate: String::new(),
            stats_format: String::new(),
            stats_uptime: String::new(),
            devices,
            selected_device,
            processes,
            selected_process: None,
            log_messages: Vec::new(),
            level_left: 0.0,
            level_right: 0.0,
        };

        let app = Self {
            state,
            theme_mode,
            settings,
            streamer,
            audio_rx,
        };

        (app, Command::none())
    }

    fn title(&self) -> String {
        match &self.state.stream_state {
            StreamState::Disconnected => "PulseStream".to_string(),
            StreamState::Connecting => "PulseStream - Connecting".to_string(),
            StreamState::Connected => "PulseStream - Connected".to_string(),
            StreamState::Streaming => "PulseStream - Streaming".to_string(),
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Connect => {
                let server = self.state.server.trim().to_string();
                if server.is_empty() {
                    return Command::none();
                }
                let port: u16 = self.state.port.parse().unwrap_or(4714);
                let rate: u32 = self.state.rate.parse().unwrap_or(48000);
                let channels: u16 = self.state.channels.parse().unwrap_or(2);
                let device_id = self.state.selected_device.as_ref().and_then(|d| {
                    if d.id.is_empty() { None } else { Some(d.id.clone()) }
                });

                let process_id = self.state.selected_process.as_ref().and_then(|name| {
                    self.state.processes.iter().find(|p| &p.name == name).map(|p| p.pid)
                });

                self.save_settings();
                self.streamer.start(server, port, rate, channels, device_id, process_id);
            }

            Message::Disconnect => {
                self.streamer.stop();
            }

            Message::ServerChanged(s) => self.state.server = s,
            Message::PortChanged(s) => self.state.port = s,
            Message::RateChanged(s) => self.state.rate = s,
            Message::ChannelsChanged(s) => self.state.channels = s,

            Message::DeviceSelected(name) => {
                self.state.selected_device = self
                    .state
                    .devices
                    .iter()
                    .find(|d| d.name == name)
                    .cloned();
            }

            Message::ProcessSelected(name) => {
                let was_streaming = !matches!(self.state.stream_state, StreamState::Disconnected);

                if name == "All apps (system audio)" {
                    self.state.selected_process = None;
                } else {
                    self.state.selected_process = Some(name);
                }

                if was_streaming {
                    self.streamer.stop();
                    self.state.log_messages.clear();
                    return self.update(Message::Connect);
                }
            }

            Message::ToggleAutoConnect(v) => {
                self.state.auto_connect = v;
                self.save_settings();
            }
            Message::ToggleStartWithWindows(v) => {
                self.state.start_with_windows = v;
                #[cfg(windows)]
                toggle_startup_registry(v);
            }
            Message::ToggleMinimizeToTray(v) => {
                self.state.minimize_to_tray = v;
                self.save_settings();
            }

            Message::ToggleTheme => {
                self.theme_mode = match self.theme_mode {
                    ThemeMode::Dark => ThemeMode::Light,
                    ThemeMode::Light => ThemeMode::Dark,
                };
                self.settings.dark_theme = self.theme_mode == ThemeMode::Dark;
                self.save_settings();
            }

            Message::AudioEvent(event) => match event {
                AudioEvent::StateChanged(ref s) => {
                    if matches!(s, StreamState::Disconnected) {
                        self.state.volume_text.clear();
                        self.state.stats_bitrate.clear();
                        self.state.stats_format.clear();
                        self.state.stats_uptime.clear();
                        self.state.level_left = 0.0;
                        self.state.level_right = 0.0;
                    }
                    self.state.stream_state = s.clone();
                }
                AudioEvent::Log(msg) => {
                    if self.state.log_messages.len() >= 200 {
                        self.state.log_messages.remove(0);
                    }
                    self.state.log_messages.push(msg);
                }
                AudioEvent::LevelUpdated { left, right } => {
                    self.state.level_left = left;
                    self.state.level_right = right;
                }
                AudioEvent::StatsUpdated(stats) => {
                    let br = if stats.bitrate_kbps >= 1000.0 {
                        format!("{:.1} Mbps", stats.bitrate_kbps / 1000.0)
                    } else {
                        format!("{:.0} kbps", stats.bitrate_kbps)
                    };
                    self.state.stats_bitrate = format!("{}  {:.1}ms", br, stats.client_latency_ms);
                    self.state.stats_format = stats.capture_format;
                    let secs = stats.uptime.as_secs();
                    self.state.stats_uptime =
                        format!("{:02}:{:02}:{:02}", secs / 3600, (secs % 3600) / 60, secs % 60);
                    self.state.show_quality_warning = stats.drops > 0;
                }
                AudioEvent::VolumeChanged { volume, muted } => {
                    self.state.volume_text = if muted {
                        "Vol: Muted".to_string()
                    } else {
                        format!("Vol: {}%", (volume * 100.0) as u32)
                    };
                }
            },

            Message::Tick => {
                self.state.processes = crate::audio::get_audio_processes();
            }
            Message::Noop => {}
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Message> {
        build_view(&self.state, self.theme_mode)
    }

    fn theme(&self) -> Theme {
        pulse_theme(self.theme_mode)
    }

    fn subscription(&self) -> Subscription<Message> {
        let tick = iced::time::every(Duration::from_secs(3)).map(|_| Message::Tick);

        if let Some(rx) = &self.audio_rx {
            let rx = rx.clone();
            let audio = iced::subscription::unfold("audio-events", rx, |rx| async move {
                match rx.recv_async().await {
                    Ok(event) => (Message::AudioEvent(event), rx),
                    Err(_) => {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        (Message::Noop, rx)
                    }
                }
            });
            Subscription::batch([audio, tick])
        } else {
            tick
        }
    }
}

impl PulseStreamApp {
    fn save_settings(&self) {
        let settings = AppSettings {
            server: self.state.server.clone(),
            port: self.state.port.parse().unwrap_or(4714),
            rate: self.state.rate.parse().unwrap_or(48000),
            channels: self.state.channels.parse().unwrap_or(2),
            device_id: self.state.selected_device.as_ref().and_then(|d| {
                if d.id.is_empty() { None } else { Some(d.id.clone()) }
            }),
            auto_connect: self.state.auto_connect,
            minimize_to_tray: self.state.minimize_to_tray,
            dark_theme: self.theme_mode == ThemeMode::Dark,
        };
        let _ = settings.save();
    }
}

#[cfg(windows)]
fn toggle_startup_registry(enable: bool) {
    use std::env;

    let exe_path = env::current_exe().unwrap_or_default();
    let exe_str = exe_path.to_string_lossy();

    let key_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
    let value_name = "PulseStream";

    // Use reg.exe for simplicity
    if enable {
        let _ = std::process::Command::new("reg")
            .args(["add", &format!(r"HKCU\{}", key_path), "/v", value_name, "/t", "REG_SZ", "/d", &format!("\"{}\"", exe_str), "/f"])
            .output();
    } else {
        let _ = std::process::Command::new("reg")
            .args(["delete", &format!(r"HKCU\{}", key_path), "/v", value_name, "/f"])
            .output();
    }
}

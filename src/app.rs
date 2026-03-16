use crate::audio::{AudioEvent, AudioStreamer, DeviceInfo, ProcessInfo, StreamState};
use crate::message::Message;
use crate::settings::AppSettings;
use crate::theme::{pulse_theme, ThemeMode};
use crate::view::build_view;
use iced::window;
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
    pub log_messages: std::collections::VecDeque<String>,
    pub scanning: bool,
}

pub struct PulseStreamApp {
    state: AppState,
    theme_mode: ThemeMode,
    settings: AppSettings,
    streamer: AudioStreamer,
    audio_rx: Option<flume::Receiver<AudioEvent>>,
    _tray_icon: Option<tray_icon::TrayIcon>,
    tray_exit_id: Option<tray_icon::menu::MenuId>,
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
            log_messages: std::collections::VecDeque::new(),
            scanning: false,
        };

        let (tray_icon, tray_exit_id) = create_tray_icon();

        let app = Self {
            state,
            theme_mode,
            settings,
            streamer,
            audio_rx,
            _tray_icon: tray_icon,
            tray_exit_id,
        };

        let startup_cmd = if app.state.server.trim().is_empty() {
            Command::perform(async {}, |_| Message::ScanServers)
        } else {
            Command::none()
        };

        (app, startup_cmd)
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
                    }
                    self.state.stream_state = s.clone();
                }
                AudioEvent::Log(msg) => {
                    if self.state.log_messages.len() >= 200 {
                        self.state.log_messages.pop_front();
                    }
                    self.state.log_messages.push_back(msg);
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

            Message::ScanServers => {
                self.state.scanning = true;
                let port: u16 = self.state.port.parse().unwrap_or(4714);
                return Command::perform(
                    async move { scan_subnet(port).await },
                    Message::ScanResult,
                );
            }

            Message::ScanResult(found) => {
                self.state.scanning = false;
                if let Some(ip) = found {
                    self.state.server = ip;
                }
            }

            Message::CloseRequested => {
                if self.state.minimize_to_tray {
                    return window::change_mode(window::Id::MAIN, window::Mode::Hidden);
                } else {
                    self.streamer.stop();
                    return window::close(window::Id::MAIN);
                }
            }

            Message::TrayRestore => {
                return window::change_mode(window::Id::MAIN, window::Mode::Windowed);
            }

            Message::ExitApp => {
                self.streamer.stop();
                return window::close(window::Id::MAIN);
            }

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

        let close_events = iced::event::listen_with(|event, _status| {
            if let iced::Event::Window(id, window::Event::CloseRequested) = event {
                let _ = id;
                Some(Message::CloseRequested)
            } else {
                None
            }
        });

        let exit_id = self.tray_exit_id.clone();
        let tray_events = iced::subscription::unfold("tray-events", exit_id, |exit_id| async {
            loop {
                if let Ok(
                    tray_icon::TrayIconEvent::Click {
                        button: tray_icon::MouseButton::Left,
                        ..
                    }
                    | tray_icon::TrayIconEvent::DoubleClick {
                        button: tray_icon::MouseButton::Left,
                        ..
                    },
                ) = tray_icon::TrayIconEvent::receiver().try_recv()
                {
                    return (Message::TrayRestore, exit_id);
                }
                if let Ok(event) = tray_icon::menu::MenuEvent::receiver().try_recv() {
                    if exit_id.as_ref() == Some(&event.id) {
                        return (Message::ExitApp, exit_id);
                    } else {
                        return (Message::TrayRestore, exit_id);
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });

        let mut subs = vec![tick, close_events, tray_events];

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
            subs.push(audio);
        }

        Subscription::batch(subs)
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

fn create_tray_icon() -> (Option<tray_icon::TrayIcon>, Option<tray_icon::menu::MenuId>) {
    let icon = match tray_icon::Icon::from_resource(1, Some((32, 32))) {
        Ok(i) => i,
        Err(_) => return (None, None),
    };

    let menu = tray_icon::menu::Menu::new();
    let open_item = tray_icon::menu::MenuItem::new("Open PulseStream", true, None);
    let exit_item = tray_icon::menu::MenuItem::new("Exit", true, None);
    let exit_id = exit_item.id().clone();
    let _ = menu.append(&open_item);
    let _ = menu.append(&tray_icon::menu::PredefinedMenuItem::separator());
    let _ = menu.append(&exit_item);

    match tray_icon::TrayIconBuilder::new()
        .with_tooltip("PulseStream")
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .build()
    {
        Ok(tray) => (Some(tray), Some(exit_id)),
        Err(_) => (None, None),
    }
}

async fn scan_subnet(port: u16) -> Option<String> {
    use std::net::{IpAddr, Ipv4Addr};

    let local_ip = get_local_ip().unwrap_or(Ipv4Addr::new(192, 168, 1, 1));
    let octets = local_ip.octets();
    let base = format!("{}.{}.{}.", octets[0], octets[1], octets[2]);

    let mut handles = Vec::new();
    for i in 1u8..=254 {
        let addr_str = format!("{}{}", base, i);
        if let Ok(ip) = addr_str.parse::<IpAddr>() {
            if ip == IpAddr::V4(local_ip) {
                continue;
            }
        }
        let addr = format!("{}:{}", addr_str, port);
        handles.push(tokio::spawn(async move {
            match tokio::time::timeout(
                std::time::Duration::from_millis(200),
                tokio::net::TcpStream::connect(&addr),
            )
            .await
            {
                Ok(Ok(_)) => Some(addr_str),
                _ => None,
            }
        }));
    }

    for handle in handles {
        if let Ok(Some(ip)) = handle.await {
            return Some(ip);
        }
    }

    None
}

fn get_local_ip() -> Option<std::net::Ipv4Addr> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    match socket.local_addr().ok()?.ip() {
        std::net::IpAddr::V4(ip) => Some(ip),
        _ => None,
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

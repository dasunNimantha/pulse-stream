use crate::app::AppState;
use crate::message::Message;
use crate::theme::{
    get_colors, CardStyle, CheckStyle, DangerButtonStyle, InputStyle, MenuStyle,
    PanelStyle, PickListStyle, PrimaryButtonStyle, ToggleStyle, ThemeMode,
};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, text, text_input, Column,
    Row, Space,
};
use iced::{Alignment, Element, Length, Theme};
use iced_aw::core::icons::bootstrap::{icon_to_text, Bootstrap};

pub fn is_valid_server(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|p| p.parse::<u8>().is_ok())
}

pub fn is_valid_port(s: &str) -> bool {
    s.parse::<u16>().is_ok_and(|p| p > 0)
}

pub fn is_valid_rate(s: &str) -> bool {
    s.parse::<u32>().is_ok_and(|r| (1000..=384000).contains(&r))
}

pub fn is_valid_channels(s: &str) -> bool {
    s.parse::<u16>().is_ok_and(|c| (1..=8).contains(&c))
}

pub fn all_fields_valid(state: &AppState) -> bool {
    is_valid_server(&state.server)
        && is_valid_port(&state.port)
        && is_valid_rate(&state.rate)
        && is_valid_channels(&state.channels)
}

pub fn build_view(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);

    let header = build_header(state, mode);
    let panel = build_main_panel(state, mode);

    let main = column![header, Space::with_height(10), panel]
        .spacing(0)
        .width(Length::Fill);

    container(main)
        .width(Length::Fill)
        .padding([10, 12])
        .style(iced::theme::Container::Custom(Box::new(
            move |_: &Theme| iced::widget::container::Appearance {
                text_color: Some(colors.text_primary),
                background: Some(iced::Background::Color(colors.bg_primary)),
                border: iced::Border::default(),
                shadow: Default::default(),
            },
        )))
        .into()
}

fn build_header(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);

    let dot_color = match &state.stream_state {
        crate::audio::StreamState::Disconnected => colors.text_disabled,
        crate::audio::StreamState::Connecting => colors.yellow,
        crate::audio::StreamState::Connected | crate::audio::StreamState::Streaming => colors.green,
    };

    let status_label = match &state.stream_state {
        crate::audio::StreamState::Disconnected => "Disconnected",
        crate::audio::StreamState::Connecting => "Connecting...",
        crate::audio::StreamState::Connected => "Connected",
        crate::audio::StreamState::Streaming => "Streaming",
    };

    let status_color = match &state.stream_state {
        crate::audio::StreamState::Disconnected => colors.text_secondary,
        crate::audio::StreamState::Connecting => colors.yellow,
        _ => colors.green,
    };

    let mut status_items: Row<Message> = row![
        text("\u{25CF}")
            .size(10)
            .style(iced::theme::Text::Color(dot_color)),
        Space::with_width(5),
        text(status_label)
            .size(12)
            .style(iced::theme::Text::Color(status_color)),
    ]
    .align_items(Alignment::Center);

    if !state.volume_text.is_empty() {
        status_items = status_items.push(Space::with_width(10)).push(
            text(&state.volume_text)
                .size(11)
                .style(iced::theme::Text::Color(colors.text_secondary)),
        );
    }

    if state.show_quality_warning {
        status_items = status_items.push(Space::with_width(5)).push(
            icon_to_text(Bootstrap::ExclamationTriangleFill)
                .size(11.0)
                .style(iced::theme::Text::Color(colors.yellow)),
        );
    }

    container(
        row![
            row![
                icon_to_text(Bootstrap::BroadcastPin)
                    .size(16.0)
                    .style(iced::theme::Text::Color(colors.accent)),
                Space::with_width(8),
                text("PulseStream")
                    .size(15)
                    .style(iced::theme::Text::Color(colors.text_primary)),
            ]
            .spacing(0)
            .align_items(Alignment::Center),
            Space::with_width(16),
            status_items,
            Space::with_width(Length::Fill),
            row![
                text(if mode == ThemeMode::Dark { "Dark" } else { "Light" })
                    .size(11)
                    .style(iced::theme::Text::Color(colors.text_secondary)),
                Space::with_width(6),
                checkbox("", mode == ThemeMode::Light)
                    .on_toggle(|_| Message::ToggleTheme)
                    .style(iced::theme::Checkbox::Custom(Box::new(ToggleStyle {
                        mode,
                    }))),
            ]
            .spacing(0)
            .align_items(Alignment::Center),
        ]
        .spacing(0)
        .align_items(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .padding([8, 14])
    .style(iced::theme::Container::Custom(Box::new(CardStyle { mode })))
    .into()
}

fn build_main_panel(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let mut content: Column<Message> = column![].spacing(0).width(Length::Fill);

    content = content.push(section_header("Connection", Bootstrap::HddNetworkFill, mode));
    content = content.push(Space::with_height(8));
    content = content.push(build_connection_fields(state, mode));
    content = content.push(Space::with_height(12));
    content = content.push(divider(mode));

    content = content.push(Space::with_height(12));
    content = content.push(section_header("Audio Source", Bootstrap::SpeakerFill, mode));
    content = content.push(Space::with_height(8));
    content = content.push(build_audio_fields(state, mode));
    content = content.push(Space::with_height(12));
    content = content.push(divider(mode));

    content = content.push(Space::with_height(12));
    content = content.push(section_header("Format", Bootstrap::Sliders, mode));
    content = content.push(Space::with_height(8));
    content = content.push(build_format_fields(state, mode));
    content = content.push(Space::with_height(12));
    content = content.push(divider(mode));

    content = content.push(Space::with_height(10));
    content = content.push(build_options_row(state, mode));
    content = content.push(Space::with_height(14));
    content = content.push(build_action_button(state, mode));

    content = content.push(Space::with_height(10));
    content = content.push(divider(mode));
    content = content.push(Space::with_height(8));
    content = content.push(build_stats_footer(state, mode));

    container(content)
        .padding(12)
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(PanelStyle { mode })))
    .into()
}

fn section_header<'a>(title: &str, icon: Bootstrap, mode: ThemeMode) -> Element<'a, Message> {
    let colors = get_colors(mode);

    row![
        icon_to_text(icon)
            .size(12.0)
            .style(iced::theme::Text::Color(colors.accent)),
        Space::with_width(7),
        text(title)
            .size(13)
            .style(iced::theme::Text::Color(colors.text_primary)),
    ]
    .align_items(Alignment::Center)
    .into()
}

fn divider(mode: ThemeMode) -> Element<'static, Message> {
    let colors = get_colors(mode);
    container(Space::new(Length::Fill, 1))
        .style(iced::theme::Container::Custom(Box::new(
            move |_: &Theme| iced::widget::container::Appearance {
                background: Some(iced::Background::Color(colors.border)),
                ..Default::default()
            },
        )))
        .into()
}

fn field_label<'a>(label: &str, mode: ThemeMode) -> Element<'a, Message> {
    let colors = get_colors(mode);
    container(
        text(label)
            .size(11)
            .style(iced::theme::Text::Color(colors.text_secondary)),
    )
    .width(55)
    .into()
}

fn build_connection_fields(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);
    let server_err = !state.server.is_empty() && !is_valid_server(&state.server);
    let port_err = !state.port.is_empty() && !is_valid_port(&state.port);

    let scan_content = row![
        icon_to_text(if state.scanning { Bootstrap::ArrowRepeat } else { Bootstrap::Wifi })
            .size(11.0),
    ]
    .align_items(Alignment::Center);

    let mut scan_btn = button(scan_content)
        .padding([4, 8])
        .style(iced::theme::Button::Custom(Box::new(
            crate::theme::SecondaryButtonStyle { mode },
        )));

    if !state.scanning {
        scan_btn = scan_btn.on_press(Message::ScanServers);
    }

    row![
        field_label("Server", mode),
        text_input("192.168.1.x", &state.server)
            .on_input(Message::ServerChanged)
            .padding([6, 10])
            .size(13)
            .style(iced::theme::TextInput::Custom(Box::new(InputStyle { mode, error: server_err }))),
        Space::with_width(6),
        scan_btn,
        Space::with_width(10),
        container(
            text("Port")
                .size(11)
                .style(iced::theme::Text::Color(colors.text_secondary)),
        )
        .width(32),
        text_input("4714", &state.port)
            .on_input(Message::PortChanged)
            .padding([6, 10])
            .size(13)
            .width(70)
            .style(iced::theme::TextInput::Custom(Box::new(InputStyle { mode, error: port_err }))),
    ]
    .align_items(Alignment::Center)
    .into()
}

fn build_audio_fields(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let device_names: Vec<String> = state.devices.iter().map(|d| d.name.clone()).collect();
    let selected_device = state.selected_device.as_ref().map(|d| d.name.clone());

    let mut process_names: Vec<String> = vec!["All apps (system audio)".to_string()];
    process_names.extend(state.processes.iter().map(|p| p.name.clone()));
    let selected_process = state
        .selected_process
        .clone()
        .unwrap_or_else(|| "All apps (system audio)".to_string());

    column![
        row![
            field_label("Device", mode),
            pick_list(device_names, selected_device, Message::DeviceSelected)
                .padding([6, 10])
                .text_size(13)
                .width(Length::Fill)
                .style(iced::theme::PickList::Custom(
                    std::rc::Rc::new(PickListStyle { mode }),
                    std::rc::Rc::new(MenuStyle { mode }),
                )),
        ]
        .align_items(Alignment::Center),
        Space::with_height(6),
        row![
            field_label("App", mode),
            pick_list(process_names, Some(selected_process), Message::ProcessSelected)
                .padding([6, 10])
                .text_size(13)
                .width(Length::Fill)
                .style(iced::theme::PickList::Custom(
                    std::rc::Rc::new(PickListStyle { mode }),
                    std::rc::Rc::new(MenuStyle { mode }),
                )),
        ]
        .align_items(Alignment::Center),
    ]
    .spacing(0)
    .into()
}

fn build_format_fields(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);
    let rate_err = !state.rate.is_empty() && !is_valid_rate(&state.rate);
    let ch_err = !state.channels.is_empty() && !is_valid_channels(&state.channels);
    row![
        field_label("Rate", mode),
        text_input("48000", &state.rate)
            .on_input(Message::RateChanged)
            .padding([6, 10])
            .size(13)
            .style(iced::theme::TextInput::Custom(Box::new(InputStyle { mode, error: rate_err }))),
        Space::with_width(12),
        container(
            text("Channels")
                .size(11)
                .style(iced::theme::Text::Color(colors.text_secondary)),
        )
        .width(55),
        text_input("2", &state.channels)
            .on_input(Message::ChannelsChanged)
            .padding([6, 10])
            .size(13)
            .width(50)
            .style(iced::theme::TextInput::Custom(Box::new(InputStyle { mode, error: ch_err }))),
    ]
    .align_items(Alignment::Center)
    .into()
}

fn build_options_row(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    row![
        checkbox("Auto-connect", state.auto_connect)
            .on_toggle(Message::ToggleAutoConnect)
            .size(14)
            .spacing(5)
            .text_size(11)
            .style(iced::theme::Checkbox::Custom(Box::new(CheckStyle { mode }))),
        Space::with_width(Length::Fill),
        checkbox("Start with Windows", state.start_with_windows)
            .on_toggle(Message::ToggleStartWithWindows)
            .size(14)
            .spacing(5)
            .text_size(11)
            .style(iced::theme::Checkbox::Custom(Box::new(CheckStyle { mode }))),
        Space::with_width(Length::Fill),
        checkbox("Minimize to tray", state.minimize_to_tray)
            .on_toggle(Message::ToggleMinimizeToTray)
            .size(14)
            .spacing(5)
            .text_size(11)
            .style(iced::theme::Checkbox::Custom(Box::new(CheckStyle { mode }))),
    ]
    .align_items(Alignment::Center)
    .into()
}

fn build_action_button(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let is_connected = !matches!(state.stream_state, crate::audio::StreamState::Disconnected);

    if is_connected {
        button(
            container(
                row![
                    icon_to_text(Bootstrap::StopCircleFill).size(13.0),
                    Space::with_width(7),
                    text("Disconnect").size(13),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(),
        )
        .on_press(Message::Disconnect)
        .width(Length::Fill)
        .padding([9, 0])
        .style(iced::theme::Button::Custom(Box::new(DangerButtonStyle {
            mode,
        })))
        .into()
    } else {
        let valid = all_fields_valid(state);
        let mut btn = button(
            container(
                row![
                    icon_to_text(Bootstrap::PlayCircleFill).size(13.0),
                    Space::with_width(7),
                    text("Connect").size(13),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(),
        )
        .width(Length::Fill)
        .padding([9, 0])
        .style(iced::theme::Button::Custom(Box::new(PrimaryButtonStyle {
            mode,
        })));

        if valid {
            btn = btn.on_press(Message::Connect);
        }

        btn.into()
    }
}

fn build_stats_footer(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);
    let has_stats = !state.stats_bitrate.is_empty();

    let bitrate_text = if has_stats { &state.stats_bitrate } else { "-- kbps  --ms" };
    let format_text = if has_stats { &state.stats_format } else { "---" };
    let uptime_text = if has_stats { &state.stats_uptime } else { "--:--:--" };

    let dim = if has_stats { colors.text_secondary } else { colors.text_disabled };
    let accent = if has_stats { colors.accent } else { colors.text_disabled };

    row![
        icon_to_text(Bootstrap::Speedometer)
            .size(11.0)
            .style(iced::theme::Text::Color(colors.text_disabled)),
        Space::with_width(5),
        text(bitrate_text)
            .size(10)
            .style(iced::theme::Text::Color(dim)),
        Space::with_width(12),
        text(format_text)
            .size(10)
            .style(iced::theme::Text::Color(dim)),
        Space::with_width(Length::Fill),
        icon_to_text(Bootstrap::ClockHistory)
            .size(11.0)
            .style(iced::theme::Text::Color(accent)),
        Space::with_width(5),
        text(uptime_text)
            .size(10)
            .style(iced::theme::Text::Color(accent)),
    ]
    .align_items(Alignment::Center)
    .into()
}

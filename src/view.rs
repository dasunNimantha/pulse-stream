use crate::app::AppState;
use crate::message::Message;
use crate::theme::{
    get_colors, CardStyle, CheckStyle, DangerButtonStyle, GhostButtonStyle, InputStyle,
    LevelBarStyle, LevelTrackStyle, MenuStyle, PanelStyle, PickListStyle, PrimaryButtonStyle,
    StatusDotStyle, ThemeMode,
};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, text, text_input, Column,
    Row, Space,
};
use iced::{Alignment, Element, Length, Theme};
use iced_aw::core::icons::bootstrap::{icon_to_text, Bootstrap};

pub fn build_view(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);
    let is_streaming = matches!(
        state.stream_state,
        crate::audio::StreamState::Connected | crate::audio::StreamState::Streaming
    );

    let mut sections: Column<Message> = column![].spacing(10).width(Length::Fill);

    sections = sections.push(build_status_card(state, mode));

    if is_streaming {
        sections = sections.push(build_level_meters(state, mode));
    }

    sections = sections.push(build_connection_card(state, mode));
    sections = sections.push(build_audio_card(state, mode));
    sections = sections.push(build_options_row(state, mode));
    sections = sections.push(build_action_button(state, mode));

    if is_streaming && !state.log_messages.is_empty() {
        sections = sections.push(build_log_panel(state, mode));
    }

    if !state.stats_bitrate.is_empty() {
        sections = sections.push(build_stats_bar(state, mode));
    }

    container(
        scrollable(
            container(sections)
                .width(Length::Fill)
                .padding([16, 18, 20, 18]),
        )
        .height(Length::Fill)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
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

fn build_status_card(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
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

    let dot = container(Space::new(10, 10))
        .style(iced::theme::Container::Custom(Box::new(StatusDotStyle {
            color: dot_color,
        })));

    let mut info: Row<Message> = row![
        dot,
        Space::with_width(10),
        text(status_label)
            .size(15)
            .style(iced::theme::Text::Color(colors.text_primary)),
    ]
    .align_items(Alignment::Center);

    if !state.volume_text.is_empty() {
        info = info.push(Space::with_width(Length::Fill)).push(
            text(&state.volume_text)
                .size(12)
                .style(iced::theme::Text::Color(colors.text_secondary)),
        );
    } else {
        info = info.push(Space::with_width(Length::Fill));
    }

    if state.show_quality_warning {
        info = info.push(Space::with_width(6)).push(
            icon_to_text(Bootstrap::ExclamationTriangleFill)
                .size(13.0)
                .style(iced::theme::Text::Color(colors.yellow)),
        );
    }

    let theme_icon = if mode == ThemeMode::Dark {
        Bootstrap::MoonFill
    } else {
        Bootstrap::SunFill
    };

    info = info.push(Space::with_width(8)).push(
        button(
            icon_to_text(theme_icon)
                .size(14.0)
                .style(iced::theme::Text::Color(colors.text_secondary)),
        )
        .on_press(Message::ToggleTheme)
        .style(iced::theme::Button::Custom(Box::new(GhostButtonStyle { mode })))
        .padding([5, 7]),
    );

    container(info)
        .width(Length::Fill)
        .padding([12, 16])
        .style(iced::theme::Container::Custom(Box::new(CardStyle { mode })))
        .into()
}

fn build_level_meters(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);

    fn meter_row<'a>(
        label: &str,
        level: f32,
        colors: crate::theme::ColorScheme,
        mode: ThemeMode,
    ) -> Element<'a, Message> {
        let pct = (level.clamp(0.0, 1.0) * 100.0) as u16;
        let bar_color = if level > 0.9 {
            colors.red
        } else if level > 0.7 {
            colors.yellow
        } else {
            colors.green
        };

        let bar = if pct > 0 {
            row![
                container(Space::new(Length::FillPortion(pct), 4))
                    .style(iced::theme::Container::Custom(Box::new(LevelBarStyle {
                        color: bar_color,
                    }))),
                container(Space::new(Length::FillPortion(100u16.saturating_sub(pct).max(1)), 4))
                    .style(iced::theme::Container::Custom(Box::new(LevelTrackStyle {
                        mode,
                    }))),
            ]
        } else {
            row![container(Space::new(Length::Fill, 4))
                .style(iced::theme::Container::Custom(Box::new(LevelTrackStyle {
                    mode,
                })))]
        };

        row![
            text(label)
                .size(10)
                .width(14)
                .style(iced::theme::Text::Color(colors.text_disabled)),
            bar.width(Length::Fill),
        ]
        .spacing(6)
        .align_items(Alignment::Center)
        .into()
    }

    container(
        column![
            meter_row("L", state.level_left, colors, mode),
            Space::with_height(3),
            meter_row("R", state.level_right, colors, mode),
        ]
        .width(Length::Fill),
    )
    .padding([4, 16])
    .into()
}

fn build_connection_card(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);

    let header = row![
        icon_to_text(Bootstrap::HddNetworkFill)
            .size(12.0)
            .style(iced::theme::Text::Color(colors.accent)),
        Space::with_width(6),
        text("Connection")
            .size(11)
            .style(iced::theme::Text::Color(colors.text_secondary)),
    ]
    .align_items(Alignment::Center);

    let server_input = text_input("192.168.1.x", &state.server)
        .on_input(Message::ServerChanged)
        .padding([7, 10])
        .size(13)
        .style(iced::theme::TextInput::Custom(Box::new(InputStyle { mode })));

    let port_input = text_input("4714", &state.port)
        .on_input(Message::PortChanged)
        .padding([7, 10])
        .size(13)
        .width(72)
        .style(iced::theme::TextInput::Custom(Box::new(InputStyle { mode })));

    let inputs = row![
        column![
            text("Server")
                .size(10)
                .style(iced::theme::Text::Color(colors.text_disabled)),
            Space::with_height(3),
            server_input,
        ]
        .width(Length::Fill),
        Space::with_width(10),
        column![
            text("Port")
                .size(10)
                .style(iced::theme::Text::Color(colors.text_disabled)),
            Space::with_height(3),
            port_input,
        ],
    ]
    .align_items(Alignment::End);

    let inner = column![header, Space::with_height(8), inputs].width(Length::Fill);

    container(inner)
        .width(Length::Fill)
        .padding([12, 14])
        .style(iced::theme::Container::Custom(Box::new(CardStyle { mode })))
        .into()
}

fn build_audio_card(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);

    let header = row![
        icon_to_text(Bootstrap::SpeakerFill)
            .size(12.0)
            .style(iced::theme::Text::Color(colors.accent)),
        Space::with_width(6),
        text("Audio")
            .size(11)
            .style(iced::theme::Text::Color(colors.text_secondary)),
    ]
    .align_items(Alignment::Center);

    let device_names: Vec<String> = state.devices.iter().map(|d| d.name.clone()).collect();
    let selected_device = state.selected_device.as_ref().map(|d| d.name.clone());

    let device_picker = column![
        text("Device")
            .size(10)
            .style(iced::theme::Text::Color(colors.text_disabled)),
        Space::with_height(3),
        pick_list(device_names, selected_device, Message::DeviceSelected)
            .padding([7, 10])
            .text_size(13)
            .style(iced::theme::PickList::Custom(
                std::rc::Rc::new(PickListStyle { mode }),
                std::rc::Rc::new(MenuStyle { mode }),
            )),
    ]
    .width(Length::Fill);

    let mut process_names: Vec<String> = vec!["All apps (system audio)".to_string()];
    process_names.extend(state.processes.iter().map(|p| p.name.clone()));
    let selected_process = state
        .selected_process
        .clone()
        .unwrap_or_else(|| "All apps (system audio)".to_string());

    let process_picker = column![
        text("App")
            .size(10)
            .style(iced::theme::Text::Color(colors.text_disabled)),
        Space::with_height(3),
        pick_list(process_names, Some(selected_process), Message::ProcessSelected)
            .padding([7, 10])
            .text_size(13)
            .style(iced::theme::PickList::Custom(
                std::rc::Rc::new(PickListStyle { mode }),
                std::rc::Rc::new(MenuStyle { mode }),
            )),
    ]
    .width(Length::Fill);

    let divider = container(Space::new(Length::Fill, 1))
        .style(iced::theme::Container::Custom(Box::new(
            move |_: &Theme| iced::widget::container::Appearance {
                background: Some(iced::Background::Color(colors.border_light)),
                ..Default::default()
            },
        )));

    let rate_input = column![
        text("Sample rate")
            .size(10)
            .style(iced::theme::Text::Color(colors.text_disabled)),
        Space::with_height(3),
        text_input("48000", &state.rate)
            .on_input(Message::RateChanged)
            .padding([7, 10])
            .size(13)
            .width(110)
            .style(iced::theme::TextInput::Custom(Box::new(InputStyle { mode }))),
    ];

    let channels_input = column![
        text("Channels")
            .size(10)
            .style(iced::theme::Text::Color(colors.text_disabled)),
        Space::with_height(3),
        text_input("2", &state.channels)
            .on_input(Message::ChannelsChanged)
            .padding([7, 10])
            .size(13)
            .width(64)
            .style(iced::theme::TextInput::Custom(Box::new(InputStyle { mode }))),
    ];

    let inner = column![
        header,
        Space::with_height(8),
        device_picker,
        Space::with_height(6),
        process_picker,
        Space::with_height(8),
        divider,
        Space::with_height(8),
        row![rate_input, Space::with_width(10), channels_input, Space::with_width(Length::Fill)]
            .align_items(Alignment::End),
    ]
    .width(Length::Fill);

    container(inner)
        .width(Length::Fill)
        .padding([12, 14])
        .style(iced::theme::Container::Custom(Box::new(CardStyle { mode })))
        .into()
}

fn build_options_row(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);

    container(
        row![
            checkbox("Auto-connect", state.auto_connect)
                .on_toggle(Message::ToggleAutoConnect)
                .size(15)
                .spacing(6)
                .text_size(11)
                .style(iced::theme::Checkbox::Custom(Box::new(CheckStyle { mode }))),
            Space::with_width(Length::Fill),
            checkbox("Start with Windows", state.start_with_windows)
                .on_toggle(Message::ToggleStartWithWindows)
                .size(15)
                .spacing(6)
                .text_size(11)
                .style(iced::theme::Checkbox::Custom(Box::new(CheckStyle { mode }))),
            Space::with_width(Length::Fill),
            checkbox("Minimize to tray", state.minimize_to_tray)
                .on_toggle(Message::ToggleMinimizeToTray)
                .size(15)
                .spacing(6)
                .text_size(11)
                .style(iced::theme::Checkbox::Custom(Box::new(CheckStyle { mode }))),
        ]
        .align_items(Alignment::Center),
    )
    .padding([2, 4])
    .style(iced::theme::Container::Custom(Box::new(
        move |_: &Theme| iced::widget::container::Appearance {
            text_color: Some(colors.text_secondary),
            ..Default::default()
        },
    )))
    .into()
}

fn build_action_button(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let is_connected = !matches!(state.stream_state, crate::audio::StreamState::Disconnected);

    if is_connected {
        button(
            container(
                row![
                    icon_to_text(Bootstrap::StopCircleFill).size(14.0),
                    Space::with_width(8),
                    text("Disconnect").size(13),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(),
        )
        .on_press(Message::Disconnect)
        .width(Length::Fill)
        .padding([10, 0])
        .style(iced::theme::Button::Custom(Box::new(DangerButtonStyle {
            mode,
        })))
        .into()
    } else {
        button(
            container(
                row![
                    icon_to_text(Bootstrap::PlayCircleFill).size(14.0),
                    Space::with_width(8),
                    text("Connect").size(13),
                ]
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .center_x(),
        )
        .on_press(Message::Connect)
        .width(Length::Fill)
        .padding([10, 0])
        .style(iced::theme::Button::Custom(Box::new(PrimaryButtonStyle {
            mode,
        })))
        .into()
    }
}

fn build_log_panel(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);

    let mut log_col = column![].spacing(1);
    let start = state.log_messages.len().saturating_sub(30);
    for msg in &state.log_messages[start..] {
        log_col = log_col.push(
            text(msg)
                .size(10)
                .style(iced::theme::Text::Color(colors.text_disabled)),
        );
    }

    container(
        scrollable(
            container(log_col).padding([6, 10]).width(Length::Fill),
        )
        .height(65)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .style(iced::theme::Container::Custom(Box::new(PanelStyle { mode })))
    .into()
}

fn build_stats_bar(state: &AppState, mode: ThemeMode) -> Element<'_, Message> {
    let colors = get_colors(mode);

    container(
        row![
            icon_to_text(Bootstrap::Speedometer)
                .size(11.0)
                .style(iced::theme::Text::Color(colors.text_disabled)),
            Space::with_width(5),
            text(&state.stats_bitrate)
                .size(10)
                .style(iced::theme::Text::Color(colors.text_disabled)),
            Space::with_width(12),
            text(&state.stats_format)
                .size(10)
                .style(iced::theme::Text::Color(colors.text_disabled)),
            Space::with_width(Length::Fill),
            icon_to_text(Bootstrap::ClockHistory)
                .size(11.0)
                .style(iced::theme::Text::Color(colors.accent)),
            Space::with_width(4),
            text(&state.stats_uptime)
                .size(10)
                .style(iced::theme::Text::Color(colors.accent)),
        ]
        .align_items(Alignment::Center),
    )
    .padding([6, 4])
    .into()
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{Application, Font, Pixels, Settings};
use pulse_stream::PulseStreamApp;

fn main() -> iced::Result {
    PulseStreamApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(420.0, 465.0),
            resizable: false,
            exit_on_close_request: false,
            ..Default::default()
        },
        fonts: vec![iced_aw::BOOTSTRAP_FONT_BYTES.into()],
        default_font: Font::with_name("Segoe UI"),
        default_text_size: Pixels(13.0),
        ..Default::default()
    })
}

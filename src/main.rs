#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{Application, Font, Pixels, Settings};
use pulse_stream_rs::PulseStreamApp;

fn main() -> iced::Result {
    PulseStreamApp::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(420.0, 700.0),
            min_size: Some(iced::Size::new(380.0, 400.0)),
            resizable: true,
            ..Default::default()
        },
        fonts: vec![iced_aw::BOOTSTRAP_FONT_BYTES.into()],
        default_font: Font::with_name("Segoe UI"),
        default_text_size: Pixels(13.0),
        ..Default::default()
    })
}

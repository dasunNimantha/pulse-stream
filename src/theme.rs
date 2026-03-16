use iced::theme::{self, Theme};
use iced::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

#[derive(Clone, Copy)]
pub struct ColorScheme {
    pub accent: Color,
    pub accent_hover: Color,
    pub accent_dim: Color,

    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_tertiary: Color,
    pub bg_hover: Color,

    pub surface: Color,
    pub surface_hover: Color,
    pub surface_active: Color,
    pub surface_elevated: Color,

    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,

    pub green: Color,
    pub yellow: Color,
    pub red: Color,

    pub border: Color,
    pub border_light: Color,
    pub border_focus: Color,

    pub fg_on_accent: Color,
}

impl ColorScheme {
    pub fn dark() -> Self {
        Self {
            // Cyan/teal accent — evokes audio waves & streaming
            accent: Color::from_rgb(0.0, 0.74, 0.85),       // #00BCD9
            accent_hover: Color::from_rgb(0.15, 0.82, 0.92), // #27D1EB
            accent_dim: Color::from_rgb(0.0, 0.55, 0.64),    // #008CA3

            // Cool-toned dark backgrounds
            bg_primary: Color::from_rgb(0.05, 0.065, 0.09),    // #0D1117
            bg_secondary: Color::from_rgb(0.086, 0.106, 0.133), // #161B22
            bg_tertiary: Color::from_rgb(0.13, 0.15, 0.176),    // #21262D
            bg_hover: Color::from_rgb(0.16, 0.18, 0.21),        // #292E36

            surface: Color::from_rgb(0.07, 0.086, 0.11),       // #12161C
            surface_hover: Color::from_rgb(0.10, 0.12, 0.15),
            surface_active: Color::from_rgb(0.14, 0.16, 0.19),
            surface_elevated: Color::from_rgb(0.086, 0.106, 0.133),

            text_primary: Color::from_rgb(0.90, 0.93, 0.95),   // #E6EDFA
            text_secondary: Color::from_rgb(0.55, 0.58, 0.62),  // #8B949E
            text_disabled: Color::from_rgb(0.30, 0.33, 0.37),   // #4D5460

            green: Color::from_rgb(0.25, 0.72, 0.31),  // #3FB84F
            yellow: Color::from_rgb(0.90, 0.70, 0.15), // #E6B326
            red: Color::from_rgb(0.97, 0.32, 0.29),    // #F85149

            border: Color::from_rgb(0.19, 0.21, 0.24),  // #30363D
            border_light: Color::from_rgb(0.13, 0.15, 0.18),
            border_focus: Color::from_rgb(0.0, 0.74, 0.85),

            fg_on_accent: Color::from_rgb(0.05, 0.065, 0.09),
        }
    }

    pub fn light() -> Self {
        Self {
            accent: Color::from_rgb(0.0, 0.59, 0.65),        // #0097A7
            accent_hover: Color::from_rgb(0.0, 0.67, 0.76),   // #00ABC1
            accent_dim: Color::from_rgb(0.0, 0.47, 0.53),     // #007887

            bg_primary: Color::from_rgb(0.96, 0.97, 0.98),    // #F6F8FA
            bg_secondary: Color::from_rgb(0.93, 0.95, 0.96),   // #EEF1F5
            bg_tertiary: Color::from_rgb(0.88, 0.91, 0.93),    // #E1E7ED
            bg_hover: Color::from_rgb(0.85, 0.87, 0.89),       // #D8DEE4

            surface: Color::WHITE,
            surface_hover: Color::from_rgb(0.96, 0.97, 0.98),
            surface_active: Color::from_rgb(0.93, 0.95, 0.96),
            surface_elevated: Color::WHITE,

            text_primary: Color::from_rgb(0.12, 0.14, 0.16),   // #1F2328
            text_secondary: Color::from_rgb(0.34, 0.38, 0.42),  // #57606A
            text_disabled: Color::from_rgb(0.55, 0.58, 0.62),   // #8C959F

            green: Color::from_rgb(0.10, 0.50, 0.22),  // #1A7F37
            yellow: Color::from_rgb(0.60, 0.40, 0.0),  // #9A6700
            red: Color::from_rgb(0.81, 0.13, 0.18),    // #CF222E

            border: Color::from_rgb(0.82, 0.84, 0.87),  // #D0D7DE
            border_light: Color::from_rgb(0.85, 0.87, 0.89),
            border_focus: Color::from_rgb(0.0, 0.59, 0.65),

            fg_on_accent: Color::WHITE,
        }
    }
}

static DARK_SCHEME: std::sync::LazyLock<ColorScheme> = std::sync::LazyLock::new(ColorScheme::dark);
static LIGHT_SCHEME: std::sync::LazyLock<ColorScheme> = std::sync::LazyLock::new(ColorScheme::light);

pub fn get_colors(mode: ThemeMode) -> ColorScheme {
    match mode {
        ThemeMode::Dark => *DARK_SCHEME,
        ThemeMode::Light => *LIGHT_SCHEME,
    }
}

pub fn pulse_theme(mode: ThemeMode) -> Theme {
    let colors = get_colors(mode);
    Theme::custom(
        "PulseStream".to_string(),
        theme::Palette {
            background: colors.bg_primary,
            text: colors.text_primary,
            primary: colors.accent,
            success: colors.green,
            danger: colors.red,
        },
    )
}

// ============== Container Styles ==============

pub struct CardStyle {
    pub mode: ThemeMode,
}

impl iced::widget::container::StyleSheet for CardStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::container::Appearance {
            text_color: Some(colors.text_primary),
            background: Some(iced::Background::Color(colors.surface_elevated)),
            border: iced::Border {
                color: colors.border_light,
                width: 1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color: if self.mode == ThemeMode::Dark {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.15)
                } else {
                    Color::from_rgba(0.0, 0.0, 0.0, 0.08)
                },
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 8.0,
            },
        }
    }
}

pub struct PanelStyle {
    pub mode: ThemeMode,
}

impl iced::widget::container::StyleSheet for PanelStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::widget::container::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::container::Appearance {
            text_color: Some(colors.text_primary),
            background: Some(iced::Background::Color(colors.surface)),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Default::default(),
        }
    }
}

// ============== Button Styles ==============

pub struct PrimaryButtonStyle {
    pub mode: ThemeMode,
}

impl iced::widget::button::StyleSheet for PrimaryButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.accent)),
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            text_color: colors.fg_on_accent,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut a = self.active(style);
        let colors = get_colors(self.mode);
        a.background = Some(iced::Background::Color(colors.accent_hover));
        a
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut a = self.active(style);
        let colors = get_colors(self.mode);
        a.background = Some(iced::Background::Color(colors.accent_dim));
        a
    }

    fn disabled(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut a = self.active(style);
        let colors = get_colors(self.mode);
        a.background = Some(iced::Background::Color(colors.bg_tertiary));
        a.text_color = colors.text_disabled;
        a
    }
}

pub struct SecondaryButtonStyle {
    pub mode: ThemeMode,
}

impl iced::widget::button::StyleSheet for SecondaryButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.surface)),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: colors.text_primary,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut a = self.active(style);
        let colors = get_colors(self.mode);
        a.background = Some(iced::Background::Color(colors.surface_hover));
        a.border.color = colors.accent;
        a
    }

    fn pressed(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut a = self.active(style);
        let colors = get_colors(self.mode);
        a.background = Some(iced::Background::Color(colors.surface_active));
        a
    }

    fn disabled(&self, style: &Self::Style) -> iced::widget::button::Appearance {
        let mut a = self.active(style);
        let colors = get_colors(self.mode);
        a.background = Some(iced::Background::Color(colors.bg_tertiary));
        a.text_color = colors.text_disabled;
        a
    }
}

pub struct DangerButtonStyle {
    pub mode: ThemeMode,
}

impl iced::widget::button::StyleSheet for DangerButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.red)),
            border: iced::Border {
                color: colors.red,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn hovered(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let hover_red = Color::from_rgb(1.0, 0.40, 0.40);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(hover_red)),
            border: iced::Border {
                color: hover_red,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn pressed(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let dark_red = Color::from_rgb(0.75, 0.20, 0.20);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(dark_red)),
            border: iced::Border {
                color: dark_red,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: Color::WHITE,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }

    fn disabled(&self, _style: &Self::Style) -> iced::widget::button::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::button::Appearance {
            background: Some(iced::Background::Color(colors.bg_tertiary)),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
            text_color: colors.text_disabled,
            shadow: Default::default(),
            shadow_offset: iced::Vector::new(0.0, 0.0),
        }
    }
}

// ============== Input Styles ==============

pub struct InputStyle {
    pub mode: ThemeMode,
    pub error: bool,
}

impl iced::widget::text_input::StyleSheet for InputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::text_input::Appearance {
            background: iced::Background::Color(colors.bg_secondary),
            border: iced::Border {
                color: if self.error { colors.red } else { colors.border },
                width: if self.error { 1.5 } else { 1.0 },
                radius: 8.0.into(),
            },
            icon_color: colors.text_secondary,
        }
    }

    fn focused(&self, _style: &Self::Style) -> iced::widget::text_input::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::text_input::Appearance {
            background: iced::Background::Color(colors.bg_secondary),
            border: iced::Border {
                color: if self.error { colors.red } else { colors.border_focus },
                width: 2.0,
                radius: 8.0.into(),
            },
            icon_color: if self.error { colors.red } else { colors.accent },
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        get_colors(self.mode).text_disabled
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        get_colors(self.mode).text_primary
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        let colors = get_colors(self.mode);
        Color::from_rgba(colors.accent.r, colors.accent.g, colors.accent.b, 0.3)
    }

    fn disabled(&self, style: &Self::Style) -> iced::widget::text_input::Appearance {
        let mut a = self.active(style);
        let colors = get_colors(self.mode);
        a.background = iced::Background::Color(colors.bg_tertiary);
        a
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        get_colors(self.mode).text_disabled
    }
}

// ============== Checkbox Styles ==============

pub struct CheckStyle {
    pub mode: ThemeMode,
}

impl iced::widget::checkbox::StyleSheet for CheckStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> iced::widget::checkbox::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::checkbox::Appearance {
            background: if is_checked {
                iced::Background::Color(colors.accent)
            } else {
                iced::Background::Color(colors.bg_tertiary)
            },
            icon_color: if is_checked {
                colors.fg_on_accent
            } else {
                Color::TRANSPARENT
            },
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 4.0.into(),
            },
            text_color: Some(colors.text_primary),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> iced::widget::checkbox::Appearance {
        let mut a = self.active(style, is_checked);
        let colors = get_colors(self.mode);
        if is_checked {
            a.background = iced::Background::Color(colors.accent_hover);
        } else {
            a.background = iced::Background::Color(colors.bg_hover);
        }
        a
    }
}

pub struct ToggleStyle {
    pub mode: ThemeMode,
}

impl iced::widget::checkbox::StyleSheet for ToggleStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> iced::widget::checkbox::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::checkbox::Appearance {
            background: if is_checked {
                iced::Background::Color(colors.accent)
            } else {
                iced::Background::Color(colors.bg_tertiary)
            },
            icon_color: if is_checked {
                colors.fg_on_accent
            } else {
                Color::WHITE
            },
            border: iced::Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 12.0.into(),
            },
            text_color: Some(colors.text_primary),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> iced::widget::checkbox::Appearance {
        let mut a = self.active(style, is_checked);
        let colors = get_colors(self.mode);
        if is_checked {
            a.background = iced::Background::Color(colors.accent_hover);
        } else {
            a.background = iced::Background::Color(colors.bg_hover);
        }
        a
    }
}

// ============== PickList Styles ==============

pub struct PickListStyle {
    pub mode: ThemeMode,
}

impl iced::widget::pick_list::StyleSheet for PickListStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &<Self as iced::widget::pick_list::StyleSheet>::Style) -> iced::widget::pick_list::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::pick_list::Appearance {
            text_color: colors.text_primary,
            placeholder_color: colors.text_disabled,
            handle_color: colors.text_secondary,
            background: iced::Background::Color(colors.bg_secondary),
            border: iced::Border {
                color: colors.border,
                width: 1.0,
                radius: 8.0.into(),
            },
        }
    }

    fn hovered(&self, _style: &<Self as iced::widget::pick_list::StyleSheet>::Style) -> iced::widget::pick_list::Appearance {
        let colors = get_colors(self.mode);
        iced::widget::pick_list::Appearance {
            text_color: colors.text_primary,
            placeholder_color: colors.text_disabled,
            handle_color: colors.accent,
            background: iced::Background::Color(colors.bg_secondary),
            border: iced::Border {
                color: colors.border_focus,
                width: 2.0,
                radius: 8.0.into(),
            },
        }
    }
}

pub struct MenuStyle {
    pub mode: ThemeMode,
}

impl iced::overlay::menu::StyleSheet for MenuStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> iced::overlay::menu::Appearance {
        let colors = get_colors(self.mode);
        let (menu_bg, highlight, border_color) = if self.mode == ThemeMode::Dark {
            (
                Color::from_rgb(0.14, 0.16, 0.19),
                Color::from_rgba(colors.accent.r, colors.accent.g, colors.accent.b, 0.18),
                Color::from_rgb(0.25, 0.28, 0.32),
            )
        } else {
            (
                Color::WHITE,
                Color::from_rgba(colors.accent.r, colors.accent.g, colors.accent.b, 0.10),
                Color::from_rgb(0.78, 0.80, 0.84),
            )
        };
        iced::overlay::menu::Appearance {
            text_color: colors.text_primary,
            background: iced::Background::Color(menu_bg),
            border: iced::Border {
                color: border_color,
                width: 1.0,
                radius: 6.0.into(),
            },
            selected_text_color: colors.text_primary,
            selected_background: iced::Background::Color(highlight),
        }
    }
}

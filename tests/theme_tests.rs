use iced::widget::button::StyleSheet as ButtonStyleSheet;
use iced::widget::container::StyleSheet as ContainerStyleSheet;
use iced::widget::text_input::StyleSheet as TextInputStyleSheet;
use iced::Color;
use pulse_stream::theme::{
    get_colors, pulse_theme, CardStyle, InputStyle, PanelStyle, PrimaryButtonStyle, ThemeMode,
};

// ==================== ColorScheme properties ====================

#[test]
fn dark_scheme_accent_is_cyan() {
    let c = get_colors(ThemeMode::Dark);
    assert!(
        c.accent.b > 0.7,
        "dark accent should have strong blue component"
    );
    assert!(c.accent.r < 0.1, "dark accent red should be near zero");
}

#[test]
fn light_scheme_accent_is_teal() {
    let c = get_colors(ThemeMode::Light);
    assert!(c.accent.b > 0.5, "light accent should have blue component");
    assert!(c.accent.g > 0.5, "light accent should have green component");
}

#[test]
fn dark_bg_is_dark() {
    let c = get_colors(ThemeMode::Dark);
    let luminance = c.bg_primary.r * 0.299 + c.bg_primary.g * 0.587 + c.bg_primary.b * 0.114;
    assert!(luminance < 0.15, "dark mode background should be dark");
}

#[test]
fn light_bg_is_light() {
    let c = get_colors(ThemeMode::Light);
    let luminance = c.bg_primary.r * 0.299 + c.bg_primary.g * 0.587 + c.bg_primary.b * 0.114;
    assert!(luminance > 0.85, "light mode background should be light");
}

#[test]
fn text_primary_readable_on_dark() {
    let c = get_colors(ThemeMode::Dark);
    let text_lum = c.text_primary.r * 0.299 + c.text_primary.g * 0.587 + c.text_primary.b * 0.114;
    let bg_lum = c.bg_primary.r * 0.299 + c.bg_primary.g * 0.587 + c.bg_primary.b * 0.114;
    let contrast = (text_lum + 0.05) / (bg_lum + 0.05);
    assert!(
        contrast > 4.5,
        "text should have sufficient contrast on dark bg (got {contrast})"
    );
}

#[test]
fn text_primary_readable_on_light() {
    let c = get_colors(ThemeMode::Light);
    let text_lum = c.text_primary.r * 0.299 + c.text_primary.g * 0.587 + c.text_primary.b * 0.114;
    let bg_lum = c.bg_primary.r * 0.299 + c.bg_primary.g * 0.587 + c.bg_primary.b * 0.114;
    let contrast = (bg_lum + 0.05) / (text_lum + 0.05);
    assert!(
        contrast > 4.5,
        "text should have sufficient contrast on light bg (got {contrast})"
    );
}

// ==================== Color value sanity ====================

#[test]
fn status_colors_are_distinct() {
    let c = get_colors(ThemeMode::Dark);
    assert!(
        c.green.g > c.green.r && c.green.g > c.green.b,
        "green should be greenish"
    );
    assert!(
        c.red.r > c.red.g && c.red.r > c.red.b,
        "red should be reddish"
    );
    assert!(
        c.yellow.r > 0.5 && c.yellow.g > 0.3,
        "yellow should be warm"
    );
}

#[test]
fn border_focus_matches_accent() {
    let dark = get_colors(ThemeMode::Dark);
    assert!((dark.border_focus.r - dark.accent.r).abs() < 0.01);
    assert!((dark.border_focus.g - dark.accent.g).abs() < 0.01);
    assert!((dark.border_focus.b - dark.accent.b).abs() < 0.01);

    let light = get_colors(ThemeMode::Light);
    assert!((light.border_focus.r - light.accent.r).abs() < 0.01);
    assert!((light.border_focus.g - light.accent.g).abs() < 0.01);
    assert!((light.border_focus.b - light.accent.b).abs() < 0.01);
}

#[test]
fn text_hierarchy_dark() {
    let c = get_colors(ThemeMode::Dark);
    let primary_lum = c.text_primary.r + c.text_primary.g + c.text_primary.b;
    let secondary_lum = c.text_secondary.r + c.text_secondary.g + c.text_secondary.b;
    let disabled_lum = c.text_disabled.r + c.text_disabled.g + c.text_disabled.b;
    assert!(
        primary_lum > secondary_lum,
        "primary text brighter than secondary"
    );
    assert!(
        secondary_lum > disabled_lum,
        "secondary text brighter than disabled"
    );
}

#[test]
fn text_hierarchy_light() {
    let c = get_colors(ThemeMode::Light);
    let primary_lum = c.text_primary.r + c.text_primary.g + c.text_primary.b;
    let secondary_lum = c.text_secondary.r + c.text_secondary.g + c.text_secondary.b;
    let disabled_lum = c.text_disabled.r + c.text_disabled.g + c.text_disabled.b;
    assert!(
        primary_lum < secondary_lum,
        "primary text darker than secondary"
    );
    assert!(
        secondary_lum < disabled_lum,
        "secondary text darker than disabled"
    );
}

// ==================== get_colors caching ====================

#[test]
fn get_colors_returns_consistent_values() {
    let a = get_colors(ThemeMode::Dark);
    let b = get_colors(ThemeMode::Dark);
    assert!((a.accent.r - b.accent.r).abs() < f32::EPSILON);
    assert!((a.accent.g - b.accent.g).abs() < f32::EPSILON);
    assert!((a.accent.b - b.accent.b).abs() < f32::EPSILON);
}

#[test]
fn get_colors_mode_produces_different_schemes() {
    let dark = get_colors(ThemeMode::Dark);
    let light = get_colors(ThemeMode::Light);
    assert!((dark.bg_primary.r - light.bg_primary.r).abs() > 0.5);
}

// ==================== Theme creation ====================

#[test]
fn pulse_theme_dark_creates_valid_theme() {
    let _theme = pulse_theme(ThemeMode::Dark);
}

#[test]
fn pulse_theme_light_creates_valid_theme() {
    let _theme = pulse_theme(ThemeMode::Light);
}

// ==================== ThemeMode ====================

#[test]
fn theme_mode_default_is_dark() {
    let mode = ThemeMode::default();
    assert_eq!(mode, ThemeMode::Dark);
}

#[test]
fn theme_mode_equality() {
    assert_eq!(ThemeMode::Dark, ThemeMode::Dark);
    assert_eq!(ThemeMode::Light, ThemeMode::Light);
    assert_ne!(ThemeMode::Dark, ThemeMode::Light);
}

#[test]
fn theme_mode_copy() {
    let a = ThemeMode::Dark;
    let b = a;
    assert_eq!(a, b);
}

// ==================== Widget style tests ====================

#[test]
fn card_style_produces_non_default_appearance() {
    let style = CardStyle {
        mode: ThemeMode::Dark,
    };
    let theme = pulse_theme(ThemeMode::Dark);
    let appearance = style.appearance(&theme);
    assert!(appearance.text_color.is_some());
    assert!(appearance.background.is_some());
    assert!(appearance.border.width > 0.0);
}

#[test]
fn panel_style_produces_non_default_appearance() {
    let style = PanelStyle {
        mode: ThemeMode::Dark,
    };
    let theme = pulse_theme(ThemeMode::Dark);
    let appearance = style.appearance(&theme);
    assert!(appearance.text_color.is_some());
    assert!(appearance.background.is_some());
    assert!(appearance.border.width > 0.0);
}

#[test]
fn primary_button_active_has_accent_bg() {
    let style = PrimaryButtonStyle {
        mode: ThemeMode::Dark,
    };
    let theme = pulse_theme(ThemeMode::Dark);
    let appearance = style.active(&theme);
    assert!(appearance.background.is_some());
    let colors = get_colors(ThemeMode::Dark);
    if let Some(iced::Background::Color(c)) = appearance.background {
        assert!((c.r - colors.accent.r).abs() < 0.01);
    }
}

#[test]
fn primary_button_hovered_differs_from_active() {
    let style = PrimaryButtonStyle {
        mode: ThemeMode::Dark,
    };
    let theme = pulse_theme(ThemeMode::Dark);
    let active = style.active(&theme);
    let hovered = style.hovered(&theme);
    let active_bg = match active.background {
        Some(iced::Background::Color(c)) => c,
        _ => Color::TRANSPARENT,
    };
    let hovered_bg = match hovered.background {
        Some(iced::Background::Color(c)) => c,
        _ => Color::TRANSPARENT,
    };
    assert!(
        (active_bg.r - hovered_bg.r).abs() > 0.01
            || (active_bg.g - hovered_bg.g).abs() > 0.01
            || (active_bg.b - hovered_bg.b).abs() > 0.01,
        "hover should differ from active"
    );
}

#[test]
fn primary_button_disabled_uses_muted_colors() {
    let style = PrimaryButtonStyle {
        mode: ThemeMode::Dark,
    };
    let theme = pulse_theme(ThemeMode::Dark);
    let disabled = style.disabled(&theme);
    let colors = get_colors(ThemeMode::Dark);
    assert_eq!(disabled.text_color.r, colors.text_disabled.r);
}

#[test]
fn input_style_normal_border() {
    let style = InputStyle {
        mode: ThemeMode::Dark,
        error: false,
    };
    let theme = pulse_theme(ThemeMode::Dark);
    let appearance = style.active(&theme);
    let colors = get_colors(ThemeMode::Dark);
    assert!((appearance.border.color.r - colors.border.r).abs() < 0.01);
}

#[test]
fn input_style_error_changes_border() {
    let normal = InputStyle {
        mode: ThemeMode::Dark,
        error: false,
    };
    let error = InputStyle {
        mode: ThemeMode::Dark,
        error: true,
    };
    let theme = pulse_theme(ThemeMode::Dark);
    let normal_app = normal.active(&theme);
    let error_app = error.active(&theme);
    assert!(
        (normal_app.border.color.r - error_app.border.color.r).abs() > 0.1,
        "error border should differ from normal"
    );
    assert!(error_app.border.width > normal_app.border.width);
}

#[test]
fn input_style_focused_error_uses_red() {
    let style = InputStyle {
        mode: ThemeMode::Dark,
        error: true,
    };
    let theme = pulse_theme(ThemeMode::Dark);
    let focused = style.focused(&theme);
    let colors = get_colors(ThemeMode::Dark);
    assert!((focused.border.color.r - colors.red.r).abs() < 0.01);
}

#[test]
fn dark_vs_light_button_styles_distinct() {
    let dark = PrimaryButtonStyle {
        mode: ThemeMode::Dark,
    };
    let light = PrimaryButtonStyle {
        mode: ThemeMode::Light,
    };
    let dark_theme = pulse_theme(ThemeMode::Dark);
    let light_theme = pulse_theme(ThemeMode::Light);
    let dark_app = dark.active(&dark_theme);
    let light_app = light.active(&light_theme);
    let dark_text = dark_app.text_color;
    let light_text = light_app.text_color;
    assert!(
        (dark_text.r - light_text.r).abs() > 0.3,
        "dark/light text colors should differ"
    );
}

#[test]
fn dark_vs_light_input_styles_distinct() {
    let dark = InputStyle {
        mode: ThemeMode::Dark,
        error: false,
    };
    let light = InputStyle {
        mode: ThemeMode::Light,
        error: false,
    };
    let dark_theme = pulse_theme(ThemeMode::Dark);
    let light_theme = pulse_theme(ThemeMode::Light);
    let dark_app = dark.active(&dark_theme);
    let light_app = light.active(&light_theme);
    let dark_bg = match dark_app.background {
        iced::Background::Color(c) => c,
        _ => Color::TRANSPARENT,
    };
    let light_bg = match light_app.background {
        iced::Background::Color(c) => c,
        _ => Color::TRANSPARENT,
    };
    assert!(
        (dark_bg.r - light_bg.r).abs() > 0.3,
        "dark/light input bg should differ"
    );
}

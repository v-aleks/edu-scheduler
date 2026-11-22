//! Палитра, шрифты и стили виджетов.

use iced::font::Weight;
use iced::widget::{button, container};
use iced::{Border, Color, Font, Shadow, Theme, Vector};

pub const BACKGROUND: Color = Color::from_rgb8(0xF7, 0xF8, 0xFA);
pub const SURFACE: Color = Color::from_rgb8(0xFF, 0xFF, 0xFF);
pub const SURFACE_MUTED: Color = Color::from_rgb8(0xF3, 0xF4, 0xF6);
pub const BORDER: Color = Color::from_rgb8(0xE5, 0xE7, 0xEB);
pub const BORDER_STRONG: Color = Color::from_rgb8(0xD1, 0xD5, 0xDB);
pub const TEXT: Color = Color::from_rgb8(0x11, 0x18, 0x27);
pub const MUTED: Color = Color::from_rgb8(0x6B, 0x72, 0x80);
pub const ACCENT: Color = Color::from_rgb8(0x25, 0x63, 0xEB);
pub const ACCENT_HOVER: Color = Color::from_rgb8(0x1D, 0x4E, 0xD8);
pub const ACCENT_SOFT: Color = Color::from_rgb8(0xDB, 0xEA, 0xFE);
pub const DANGER: Color = Color::from_rgb8(0xDC, 0x26, 0x26);
pub const DANGER_SOFT: Color = Color::from_rgb8(0xFE, 0xE2, 0xE2);
pub const BREAK_BG: Color = Color::from_rgb8(0xF3, 0xF4, 0xF6);
pub const BREAK_BORDER: Color = Color::from_rgb8(0xD1, 0xD5, 0xDB);
pub const WEEKEND_BG: Color = Color::from_rgb8(0xFA, 0xFA, 0xFC);
pub const TODAY_BG: Color = Color::from_rgb8(0xEF, 0xF6, 0xFF);
pub const WHITE: Color = Color::from_rgb8(0xFF, 0xFF, 0xFF);

pub const FONT_NAME: &str = "Noto Sans";

pub fn font() -> Font {
    Font::with_name(FONT_NAME)
}

pub fn semibold() -> Font {
    Font {
        weight: Weight::Semibold,
        ..font()
    }
}

pub fn bold() -> Font {
    Font {
        weight: Weight::Bold,
        ..font()
    }
}

pub fn teacher_color(hex: &str) -> Color {
    let hex = hex.trim().trim_start_matches('#');

    if hex.len() != 6 {
        return MUTED;
    }

    let parse = |range: std::ops::Range<usize>| u8::from_str_radix(&hex[range], 16).unwrap_or(0x6B);

    Color::from_rgb8(parse(0..2), parse(2..4), parse(4..6))
}

pub fn card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 14.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba8(0x11, 0x18, 0x27, 0.04),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..container::Style::default()
    }
}

pub fn modal(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba8(0x11, 0x18, 0x27, 0.22),
            offset: Vector::new(0.0, 12.0),
            blur_radius: 32.0,
        },
        ..container::Style::default()
    }
}

pub fn banner(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(DANGER_SOFT.into()),
        border: Border {
            color: Color::from_rgb8(0xFE, 0xCA, 0xCA),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn muted_box(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(SURFACE_MUTED.into()),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => ACCENT_HOVER,
        button::Status::Pressed => Color::from_rgb8(0x1E, 0x40, 0xAF),
        button::Status::Disabled => Color::from_rgb8(0x93, 0xC5, 0xFD),
        button::Status::Active => ACCENT,
    };

    button::Style {
        background: Some(background.into()),
        text_color: WHITE,
        border: Border {
            color: background,
            width: 1.0,
            radius: 9.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn secondary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgb8(0xF3, 0xF4, 0xF6),
        button::Status::Disabled => Color::from_rgb8(0xFA, 0xFA, 0xFA),
        button::Status::Active => SURFACE,
    };

    button::Style {
        background: Some(background.into()),
        text_color: TEXT,
        border: Border {
            color: BORDER_STRONG,
            width: 1.0,
            radius: 9.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn ghost_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => Some(SURFACE_MUTED.into()),
        _ => None,
    };

    button::Style {
        background,
        text_color: MUTED,
        border: Border::default(),
        ..button::Style::default()
    }
}

pub fn danger_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgb8(0xB9, 0x1C, 0x1C),
        button::Status::Pressed => Color::from_rgb8(0x99, 0x1B, 0x1B),
        button::Status::Disabled => Color::from_rgb8(0xFE, 0xCA, 0xCA),
        button::Status::Active => DANGER,
    };

    button::Style {
        background: Some(background.into()),
        text_color: WHITE,
        border: Border {
            color: background,
            width: 1.0,
            radius: 9.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn active_toggle(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => Color::from_rgb8(0x1D, 0x4E, 0xD8),
        _ => ACCENT,
    };

    button::Style {
        background: Some(background.into()),
        text_color: WHITE,
        border: Border {
            color: background,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn inactive_toggle(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => Some(SURFACE_MUTED.into()),
        _ => None,
    };

    button::Style {
        background,
        text_color: MUTED,
        border: Border {
            color: Color::TRANSPARENT,
            width: 1.0,
            radius: 8.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn chip_lesson(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => ACCENT_SOFT,
        _ => Color::from_rgba8(0x25, 0x63, 0xEB, 0.10),
    };

    button::Style {
        background: Some(background.into()),
        text_color: TEXT,
        border: Border {
            color: Color::from_rgba8(0x25, 0x63, 0xEB, 0.18),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn chip_break(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => BORDER,
        _ => BREAK_BG,
    };

    button::Style {
        background: Some(background.into()),
        text_color: MUTED,
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: 6.0.into(),
        },
        ..button::Style::default()
    }
}

pub fn color_swatch(
    color: Color,
    selected: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let border_color = if selected { TEXT } else { Color::TRANSPARENT };
        let _ = status;

        button::Style {
            background: Some(color.into()),
            text_color: WHITE,
            border: Border {
                color: border_color,
                width: if selected { 2.5 } else { 1.0 },
                radius: 999.0.into(),
            },
            ..button::Style::default()
        }
    }
}

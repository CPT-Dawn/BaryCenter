// ── Cosmic Dawn v2 Theme ─────────────────────────────────────────────────────
//
// Deep void background (#0D0B14), electric cyan (#00FFFF) borders,
// soft snow text (#E0E6F0), dawn red accent (#FF4500).
// ─────────────────────────────────────────────────────────────────────────────

use crate::config::{parse_hex_color, AppConfig};
use iced::widget::{container, scrollable, text_input};
use iced::{Border, Color, Shadow, Theme, Vector};

/// Resolved colors from the hex config — computed once at startup.
#[derive(Debug, Clone)]
pub struct CosmicDawn {
    pub border: Color,
    pub background: Color,
    pub text: Color,
    pub accent: Color,
    pub border_width: f32,
    pub border_radius: f32,
}

impl CosmicDawn {
    pub fn from_config(cfg: &AppConfig) -> Self {
        Self {
            border: parse_hex_color(&cfg.border_color),
            background: parse_hex_color(&cfg.background_color),
            text: parse_hex_color(&cfg.text_color),
            accent: parse_hex_color(&cfg.accent_color),
            border_width: cfg.border_width,
            border_radius: cfg.border_radius,
        }
    }
}

// ── Container Styles ─────────────────────────────────────────────────────────

pub fn window_container(theme: &CosmicDawn) -> container::Style {
    let t = theme.clone();
    container::Style {
        background: Some(iced::Background::Color(t.background)),
        border: Border {
            color: t.border,
            width: t.border_width,
            radius: t.border_radius.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
            offset: Vector::new(0.0, 6.0),
            blur_radius: 28.0,
        },
        text_color: Some(t.text),
    }
}

pub fn result_row(_theme: &CosmicDawn) -> container::Style {
    container::Style {
        background: None,
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 10.0.into(),
        },
        shadow: Shadow::default(),
        text_color: None,
    }
}

pub fn result_row_selected(theme: &CosmicDawn) -> container::Style {
    let t = theme.clone();
    container::Style {
        background: Some(iced::Background::Color(Color::from_rgba(
            t.accent.r, t.accent.g, t.accent.b, 0.14,
        ))),
        border: Border {
            color: Color::from_rgba(t.accent.r, t.accent.g, t.accent.b, 0.6),
            width: 1.5,
            radius: 10.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(t.accent.r, t.accent.g, t.accent.b, 0.08),
            offset: Vector::new(0.0, 0.0),
            blur_radius: 12.0,
        },
        text_color: None,
    }
}

// ── Text Input Style ─────────────────────────────────────────────────────────

pub fn search_input(theme: &CosmicDawn) -> text_input::Style {
    let t = theme.clone();
    // Subtle cyan tint on the search background.
    text_input::Style {
        background: iced::Background::Color(Color::from_rgba(
            t.border.r, t.border.g, t.border.b, 0.04,
        )),
        border: Border {
            color: t.border,
            width: 1.5,
            radius: (t.border_radius * 0.75).into(),
        },
        icon: t.text,
        placeholder: Color::from_rgba(t.text.r, t.text.g, t.text.b, 0.35),
        value: t.text,
        selection: Color::from_rgba(t.accent.r, t.accent.g, t.accent.b, 0.35),
    }
}

// ── Scrollable Style ─────────────────────────────────────────────────────────

pub fn results_scrollable(_theme: &Theme, _status: scrollable::Status) -> scrollable::Style {
    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 4.0.into(),
                },
            },
        },
        horizontal_rail: scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: Color::TRANSPARENT,
                border: Border::default(),
            },
        },
        gap: None,
    }
}

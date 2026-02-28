use iced::{
    Background, Color, Gradient,
    gradient::{ColorStop, Linear},
};

/// Creates a horizontal gradient for a color channel slider.
/// The gradient goes from `start_color` (at offset 0) to `end_color` (at offset 1).
pub fn channel_gradient(start_color: Color, end_color: Color) -> Background {
    Gradient::Linear(Linear::new(90.0).add_stops([
        ColorStop {
            color: start_color,
            offset: 0.0,
        },
        ColorStop {
            color: end_color,
            offset: 1.0,
        },
    ]))
    .into()
}

/// Represents a color channel (Red, Green, Blue, Alpha).
#[derive(Debug, Clone, Copy)]
pub enum ColorChannel {
    Red,
    Green,
    Blue,
    Alpha,
}

impl ColorChannel {
    /// A static letter representation of the color channel, e.g. "R" for Red.
    pub fn letter(&self) -> &'static str {
        match self {
            ColorChannel::Red => "R",
            ColorChannel::Green => "G",
            ColorChannel::Blue => "B",
            ColorChannel::Alpha => "A",
        }
    }
}

/// Returns the gradient backgrounds for a color channel slider.
/// The left gradient shows colors from 0 to the current value.
/// The right gradient shows colors from the current value to 255.
pub fn channel_slider_backgrounds(
    channel: ColorChannel,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
) -> (Background, Background) {
    use Color;

    let alpha = a as f32 / 255.0;

    let (min_color, current_color, max_color) = match channel {
        ColorChannel::Red => (
            Color::from_rgba8(0, g, b, alpha),
            Color::from_rgba8(r, g, b, alpha),
            Color::from_rgba8(255, g, b, alpha),
        ),
        ColorChannel::Green => (
            Color::from_rgba8(r, 0, b, alpha),
            Color::from_rgba8(r, g, b, alpha),
            Color::from_rgba8(r, 255, b, alpha),
        ),
        ColorChannel::Blue => (
            Color::from_rgba8(r, g, 0, alpha),
            Color::from_rgba8(r, g, b, alpha),
            Color::from_rgba8(r, g, 255, alpha),
        ),
        ColorChannel::Alpha => (
            Color::from_rgba8(r, g, b, 0.0),
            Color::from_rgba8(r, g, b, alpha),
            Color::from_rgba8(r, g, b, 1.0),
        ),
    };

    (
        channel_gradient(min_color, current_color),
        channel_gradient(current_color, max_color),
    )
}

pub mod button {
    use iced::widget::button;
    use iced::{Theme, border};

    /// Subtle button style for secondary actions.
    pub fn subtle(theme: &Theme, status: button::Status) -> button::Style {
        let palette = theme.extended_palette();
        match status {
            button::Status::Active => button::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: palette.background.base.text,
                border: border::rounded(4),
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: Some(palette.background.strong.color.into()),
                text_color: palette.background.base.text,
                border: border::rounded(4),
                ..Default::default()
            },
            button::Status::Pressed => button::Style {
                background: Some(palette.background.stronger.color.into()),
                text_color: palette.background.stronger.text,
                border: border::rounded(4),
                ..Default::default()
            },
            button::Status::Disabled => button::Style {
                background: Some(palette.background.weak.color.scale_alpha(0.5).into()),
                text_color: palette.background.base.text.scale_alpha(0.5),
                border: border::rounded(4),
                ..Default::default()
            },
        }
    }

    /// Ghost button style: transparent at rest with subtle hover/press background.
    pub fn ghost_subtle(theme: &Theme, status: button::Status) -> button::Style {
        let palette = theme.extended_palette();

        match status {
            button::Status::Active => button::Style {
                background: None,
                text_color: palette.background.base.text,
                border: border::rounded(4),
                ..Default::default()
            },
            button::Status::Hovered => button::Style {
                background: Some(palette.background.weak.color.into()),
                text_color: palette.background.base.text,
                border: border::rounded(4),
                ..Default::default()
            },
            button::Status::Pressed => button::Style {
                background: Some(palette.background.strong.color.into()),
                text_color: palette.background.strong.text,
                border: border::rounded(4),
                ..Default::default()
            },
            button::Status::Disabled => button::Style {
                background: None,
                text_color: palette.background.base.text.scale_alpha(0.5),
                border: border::rounded(4),
                ..Default::default()
            },
        }
    }
}

pub mod container {
    use iced::widget::container;
    use iced::{Border, Color, Shadow, Theme, Vector};

    pub fn tooltip_background(theme: &Theme) -> container::Style {
        container::Style {
            text_color: Some(theme.extended_palette().background.weak.text),
            background: Some(theme.extended_palette().background.weak.color.into()),
            border: Border::default().rounded(4),
            shadow: Shadow {
                color: Color::BLACK.scale_alpha(0.3),
                offset: Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            },
            ..Default::default()
        }
    }

    pub fn dialog_backdrop(theme: &Theme, animate: bool) -> container::Style {
        let alpha = if animate { 0.5 } else { 0.6 };

        container::Style {
            background: Some(theme.palette().text.scale_alpha(alpha).into()),
            ..Default::default()
        }
    }

    pub fn dialog_panel(theme: &Theme) -> container::Style {
        let pair = theme.extended_palette().background.base;

        container::Style {
            text_color: Some(pair.text),
            background: Some(pair.color.into()),
            border: Border::default().rounded(8),
            shadow: Shadow {
                color: Color::BLACK.scale_alpha(0.35),
                offset: Vector::new(0.0, 8.0),
                blur_radius: 24.0,
            },
            ..Default::default()
        }
    }
}

pub mod pick_list {
    use iced::Theme;
    use iced::widget::overlay::menu;
    use iced::widget::pick_list;

    pub fn default(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
        let default = pick_list::default(theme, status);
        pick_list::Style {
            border: default.border.rounded(4),
            ..default
        }
    }

    pub fn menu(theme: &Theme) -> menu::Style {
        let default = menu::default(theme);
        menu::Style {
            border: default.border.rounded(4),
            ..default
        }
    }
}

pub mod text {
    use iced::Theme;
    use iced::widget::text;

    /// Faded text style (50% alpha) using background weakest text color.
    pub fn faded(theme: &Theme) -> text::Style {
        text::Style {
            color: Some(
                theme
                    .extended_palette()
                    .background
                    .weakest
                    .text
                    .scale_alpha(0.5),
            ),
        }
    }

    /// Secondary text style (60% alpha) using background weakest text color.
    pub fn secondary(theme: &Theme) -> text::Style {
        text::Style {
            color: Some(
                theme
                    .extended_palette()
                    .background
                    .weakest
                    .text
                    .scale_alpha(0.6),
            ),
        }
    }

    /// Muted text style (70% alpha) using background weakest text color.
    pub fn muted(theme: &Theme) -> text::Style {
        text::Style {
            color: Some(
                theme
                    .extended_palette()
                    .background
                    .weakest
                    .text
                    .scale_alpha(0.7),
            ),
        }
    }

    /// Subdued text style (80% alpha) using background weakest text color.
    pub fn subdued(theme: &Theme) -> text::Style {
        text::Style {
            color: Some(
                theme
                    .extended_palette()
                    .background
                    .weakest
                    .text
                    .scale_alpha(0.8),
            ),
        }
    }

    pub fn danger(theme: &Theme) -> text::Style {
        text::Style {
            color: Some(theme.extended_palette().danger.strong.color),
        }
    }
}

pub mod text_input {
    use iced::Theme;
    use iced::widget::text_input;

    pub fn default(theme: &Theme, status: text_input::Status) -> text_input::Style {
        let default = text_input::default(theme, status);
        text_input::Style {
            border: default.border.rounded(4),
            ..default
        }
    }

    /// A text input with a danger border when the content is invalid.
    pub fn validated(theme: &Theme, status: text_input::Status, valid: bool) -> text_input::Style {
        let mut style = default(theme, status);
        if !valid {
            style.border = style
                .border
                .color(theme.extended_palette().danger.strong.color);
        }
        style
    }
}

pub mod svg {
    use iced::Theme;
    use iced::widget::svg;

    pub fn text(theme: &Theme, _status: svg::Status) -> svg::Style {
        svg::Style {
            color: Some(theme.palette().text),
        }
    }

    pub fn strong_background(theme: &Theme, _status: svg::Status) -> svg::Style {
        svg::Style {
            color: Some(theme.extended_palette().background.strong.color),
        }
    }

    pub fn success(theme: &Theme, _status: svg::Status) -> svg::Style {
        svg::Style {
            color: Some(theme.extended_palette().success.strong.color),
        }
    }

    pub fn danger(theme: &Theme, _status: svg::Status) -> svg::Style {
        svg::Style {
            color: Some(theme.extended_palette().danger.strong.color),
        }
    }
}

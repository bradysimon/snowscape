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

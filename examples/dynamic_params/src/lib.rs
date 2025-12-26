use std::fmt;

use iced::{
    Color, Element,
    widget::{column, container, row, space, text},
};

/// A view that takes in various parameters intended to show off how dynamic parameters work.
pub fn adjustable_view(
    label_text: &str,
    number: i32,
    toggle: bool,
    alignment: Alignment,
    padding: f32,
    color: Color,
) -> Element<'_, ()> {
    let align = match alignment {
        Alignment::Left => iced::Alignment::Start,
        Alignment::Center => iced::Alignment::Center,
        Alignment::Right => iced::Alignment::End,
    };

    // Convert color to hex string
    let hex_color = format!(
        "#{:02X}{:02X}{:02X}",
        (color.r * 255.0) as u8,
        (color.g * 255.0) as u8,
        (color.b * 255.0) as u8
    );

    // Color swatch
    let color_swatch =
        container(space().width(20).height(20)).style(move |_theme| container::Style {
            background: Some(color.into()),
            border: iced::border::rounded(4),
            ..Default::default()
        });

    container(
        column![
            text(label_text).size(30),
            text(number),
            text(if toggle {
                "The toggle is ON"
            } else {
                "The toggle is OFF"
            }),
            text!("Alignment: {}", alignment).size(14),
            text!("Padding: {:.0}px", padding).size(14),
            row![
                text("Color: ").size(14),
                color_swatch,
                text!(" {}", hex_color).size(14),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        ]
        .align_x(align)
        .spacing(12),
    )
    .style(container::bordered_box)
    .padding(padding as u16)
    .into()
}

/// Alignment options for the layout.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Alignment {
    Left,
    #[default]
    Center,
    Right,
}

impl fmt::Display for Alignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Alignment::Left => write!(f, "Left"),
            Alignment::Center => write!(f, "Center"),
            Alignment::Right => write!(f, "Right"),
        }
    }
}

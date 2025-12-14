pub mod badge;
pub mod config_pane;
pub mod split;

pub use badge::*;
pub use config_pane::*;

use iced::widget::{button, container, pick_list, row, space, svg, text};
use iced::{Alignment::Center, Element, Length::Fill, Theme, border, overlay::menu};
use iced_anim::Animated;

use crate::{message::Message, preview::Preview};

/// The theme picker dropdown shown in the header.
pub fn theme_picker<'a>(theme: Option<Theme>) -> Element<'a, Message> {
    pick_list(Theme::ALL, theme, |theme| {
        Message::UpdateTheme(theme.into())
    })
    .text_size(14)
    .placeholder("System theme")
    .style(|theme, status| {
        let default = pick_list::default(theme, status);
        pick_list::Style {
            border: default.border.rounded(4),
            ..default
        }
    })
    .menu_style(|theme| {
        let default = menu::default(theme);
        menu::Style {
            border: default.border.rounded(4),
            ..default
        }
    })
    .into()
}

/// The header shown above the preview area.
pub fn header<'a>(theme: &'a Option<Animated<Theme>>) -> Element<'a, Message> {
    row![
        reset_button(),
        space::horizontal(),
        theme_picker(theme.as_ref().map(|t| t.target().clone())),
    ]
    .align_y(Center)
    .padding(10)
    .into()
}

/// A button to reset the current preview.
pub fn reset_button<'a>() -> Element<'a, Message> {
    button(
        row![
            crate::icon::refresh()
                .width(16)
                .height(16)
                .style(|theme, _status| svg::Style {
                    color: Some(theme.palette().text),
                }),
            text("Reset").size(14),
        ]
        .spacing(6)
        .align_y(Center),
    )
    .on_press(Message::ResetPreview)
    .style(|theme: &Theme, status| {
        let pair = match status {
            button::Status::Hovered => theme.extended_palette().background.weaker,
            button::Status::Pressed => theme.extended_palette().background.weak,
            button::Status::Disabled => theme.extended_palette().background.weakest,
            _ => theme.extended_palette().background.base,
        };
        button::Style {
            background: Some(pair.color.into()),
            text_color: pair.text,
            border: border::rounded(4),
            ..button::text(theme, status)
        }
    })
    .into()
}

/// The main preview area showing the selected `preview`.
pub fn preview_area(preview: Option<&dyn Preview>) -> Element<'_, Message> {
    container(if let Some(preview) = preview {
        preview.view()
    } else {
        // TODO: Improve placeholder view
        text("No preview selected").into()
    })
    .padding(20)
    .center(Fill)
    .into()
}

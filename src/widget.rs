pub mod badge;
pub mod config_pane;
pub mod split;

pub use badge::*;
pub use config_pane::*;

use iced::theme;
use iced::widget::{Column, button, container, pick_list, row, space, svg, text};
use iced::{Alignment::Center, Element, Length::Fill, Theme, border};
use iced_anim::Animated;

use crate::preview::Descriptor;
use crate::{message::Message, preview::Preview};

/// The theme picker dropdown shown in the header.
pub fn theme_picker<'a>(theme: Option<Theme>) -> Element<'a, Message> {
    pick_list(Theme::ALL, theme, |theme| {
        Message::UpdateTheme(theme.into())
    })
    .text_size(14)
    .placeholder("System theme")
    .style(crate::style::pick_list::default)
    .menu_style(crate::style::pick_list::menu)
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

/// A list of available previews the user can select from to view.
pub fn preview_list(
    previews: &[Descriptor],
    selected_index: Option<usize>,
) -> Element<'_, Message> {
    if previews.is_empty() {
        text("No previews available").size(14).into()
    } else {
        previews
            .iter()
            .enumerate()
            .fold(Column::new(), |column, (index, descriptor)| {
                let is_selected = Some(index) == selected_index;
                column.push(preview_list_item(descriptor, index, is_selected))
            })
            .into()
    }
}

/// A single preview that is shown in the list of available previews.
fn preview_list_item(
    descriptor: &Descriptor,
    index: usize,
    is_selected: bool,
) -> Element<'_, Message> {
    button(text(&descriptor.metadata().label).size(14))
        .width(Fill)
        .on_press(Message::SelectPreview(index))
        .style(move |theme, status| {
            let base = button::primary(theme, status);
            if is_selected {
                button::Style {
                    background: Some(theme.extended_palette().primary.base.color.into()),
                    text_color: theme.extended_palette().primary.base.text,
                    border: border::rounded(4),
                    ..base
                }
            } else {
                let default = button::text(theme, status);
                let pair: Option<theme::palette::Pair> = match status {
                    button::Status::Hovered => Some(theme.extended_palette().background.stronger),
                    button::Status::Pressed => Some(theme.extended_palette().background.strongest),
                    _ => None,
                };
                button::Style {
                    background: pair.map(|p| p.color.into()),
                    text_color: pair.map(|p| p.text).unwrap_or(default.text_color),
                    border: border::rounded(4),
                    ..default
                }
            }
        })
        .into()
}

use iced::Alignment::Center;
use iced::Element;
use iced::Length::Fill;
use iced::Length::Shrink;
use iced::widget::{column, row, scrollable, space, text};

use crate::style;
use crate::widget::badge;
use crate::{app::Message, metadata::Metadata};

/// A pane shown in the configuration area displaying metadata about the preview.
pub fn about_pane(metadata: &Metadata) -> Element<'_, Message> {
    scrollable(
        column![
            row![
                text(&metadata.label).size(18),
                space::horizontal().width(Shrink),
                row(metadata.tags.iter().cloned().map(badge))
                    .spacing(4)
                    .wrap()
            ]
            .spacing(8)
            .align_y(Center)
            .wrap(),
            space::vertical().height(5),
            if let Some(description) = &metadata.description {
                text(description)
            } else {
                text("No description available.")
            }
            .style(style::text::muted),
        ]
        .width(Fill),
    )
    .into()
}

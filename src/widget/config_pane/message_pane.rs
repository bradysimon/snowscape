use std::time::Duration;

use iced::Alignment::Center;
use iced::Element;
use iced::Length::Fill;
use iced::widget::{column, container, row, scrollable, text, tooltip};

use crate::app::Message;
use crate::widget::mini_badge;

/// The pane containing the list of emitted messages by the preview.
pub fn message_pane(messages: &[String]) -> Element<'_, Message> {
    if messages.is_empty() {
        text("No messages emitted.").into()
    } else {
        scrollable(
            column(
                messages
                    .iter()
                    .enumerate()
                    .map(|(i, message)| message_item(message, i)),
            )
            .spacing(4)
            .width(Fill),
        )
        .anchor_bottom()
        .into()
    }
}

/// A single message item within the message pane.
fn message_item(message: &str, index: usize) -> Element<'_, Message> {
    tooltip(
        row![
            mini_badge(index + 1),
            text(message).wrapping(text::Wrapping::None)
        ]
        .spacing(4)
        .align_y(Center),
        container(message).max_width(768),
        tooltip::Position::Top,
    )
    .delay(Duration::from_secs(1))
    .style(crate::style::container::tooltip_background)
    .into()
}

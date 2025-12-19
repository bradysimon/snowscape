use iced::Alignment::Center;
use iced::Element;
use iced::Length::Fill;
use iced::widget::{column, row, scrollable, text};

use crate::app::Message;
use crate::widget::mini_badge;

/// The pane containing the list of emitted messages by the preview.
pub fn message_pane(messages: &[String]) -> Element<'_, Message> {
    if messages.is_empty() {
        text("No messages emitted.").into()
    } else {
        scrollable(
            column(messages.iter().enumerate().map(|(i, message)| {
                row![mini_badge(i + 1), text(message)]
                    .spacing(4)
                    .align_y(Center)
                    .into()
            }))
            .spacing(4)
            .width(Fill),
        )
        .anchor_bottom()
        .into()
    }
}

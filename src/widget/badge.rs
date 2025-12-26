use iced::{
    Element, Theme, border,
    widget::{container, text, text::IntoFragment},
};

use crate::app::Message;

/// A small badge that shows some `content` within it.
pub fn badge<'a>(content: impl IntoFragment<'a>) -> Element<'a, Message> {
    container(text(content).size(14))
        .padding([2, 6])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weak.color.into()),
            border: border::rounded(4),
            ..container::Style::default()
        })
        .into()
}

/// A round badge typically showing a number, e.g. the number of emitted messages.
pub fn round_badge<'a>(content: impl IntoFragment<'a>, is_primary: bool) -> Element<'a, Message> {
    container(text(content).size(10))
        .padding([2, 6])
        .style(move |theme: &Theme| {
            let pair = if is_primary {
                theme.extended_palette().primary.base
            } else {
                theme.extended_palette().background.neutral
            };
            container::Style {
                background: Some(pair.color.into()),
                text_color: Some(pair.text),
                border: border::rounded(16),
                ..container::Style::default()
            }
        })
        .into()
}

/// A very tiny badge typically shown within message history.
pub fn mini_badge<'a>(content: impl IntoFragment<'a>) -> Element<'a, Message> {
    container(text(content).size(12))
        .center_x(32)
        .style(|theme: &Theme| {
            let pair = theme.extended_palette().background.weak;
            container::Style {
                background: Some(pair.color.into()),
                text_color: Some(pair.text),
                border: border::rounded(2),
                ..container::Style::default()
            }
        })
        .into()
}

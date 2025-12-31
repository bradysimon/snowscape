pub mod theme;

use iced::{
    Element,
    Length::Fill,
    widget::{Button, Text, column, container, row, space, text::IntoFragment, themer},
};
use snowscape::stateless;

use crate::theme::{ContainerVariant, CustomTheme};

/// Previews various components used within Snowscape.
fn main() -> iced::Result {
    snowscape::run(|app| {
        app.title("Custom Themed Previews")
            .preview(stateless("Product Card", || {
                wrapper(
                    card(
                        column![
                            text("Fake Product Title").class(theme::TextVariant::Primary),
                            text("Here's a product description that goes on for a couple lines.")
                                .class(theme::TextVariant::Secondary),
                            space::vertical().height(8),
                            row![
                                text("$19.99").size(20),
                                space::horizontal(),
                                button("Buy Now").on_press(Message::None),
                            ]
                        ]
                        .spacing(4),
                    )
                    .max_width(250),
                )
            }))
    })
}

#[derive(Debug, Clone)]
enum Message {
    None,
}

// MARK: Widgets

fn text<'a>(content: impl IntoFragment<'a>) -> Text<'a, CustomTheme> {
    iced::widget::text(content)
}

fn button<'a>(
    content: impl Into<Element<'a, Message, CustomTheme>>,
) -> Button<'a, Message, CustomTheme> {
    iced::widget::button(content)
}

fn card<'a>(
    content: impl Into<Element<'a, Message, CustomTheme>>,
) -> container::Container<'a, Message, CustomTheme> {
    container(content)
        .class(ContainerVariant::Panel)
        .padding(16)
}

/// A wrapper that converts content into a themed container.
/// This is used to apply your custom theme to all preview content.
fn wrapper<'a>(content: impl Into<Element<'a, Message, CustomTheme>>) -> Element<'a, Message> {
    themer(
        Some(CustomTheme::default()),
        container(content)
            .class(ContainerVariant::Background)
            .center(Fill),
    )
    .into()
}

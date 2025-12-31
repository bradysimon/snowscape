pub mod theme;

use iced::{
    Element,
    Length::Fill,
    widget::{Button, Text, column, container, row, space, text::IntoFragment, themer},
};
use snowscape::dynamic;

use crate::theme::{ContainerVariant, CustomTheme, TextVariant};

/// Previews various components used within Snowscape.
fn main() -> iced::Result {
    snowscape::run(|app| {
        app.title("Custom Themed Previews").preview(
            dynamic::stateless(
                "Product Card",
                (
                    dynamic::text("Title", "Awesome Gadget"),
                    dynamic::text(
                        "Description",
                        "This gadget is awesome because it has many features.",
                    ),
                    dynamic::number("Price", 50),
                ),
                |(title, description, price)| wrapper(product_card(title, description, *price)),
            )
            .tags(["Product", "Card", "Price"])
            .description("A card displaying a product, the price, and a buy button."),
        )
    })
}

#[derive(Debug, Clone)]
enum Message {
    Purchase,
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

/// An example product card component that you might have in your app.
fn product_card<'a>(
    title: &'a str,
    description: &'a str,
    price: i32,
) -> Element<'a, Message, CustomTheme> {
    card(
        column![
            text(title).class(TextVariant::Primary),
            text(description).class(TextVariant::Secondary),
            space::vertical().height(8),
            row![
                text(format!("${price}")).size(20),
                space::horizontal(),
                button("Buy Now").on_press(Message::Purchase),
            ]
        ]
        .spacing(4),
    )
    .max_width(250)
    .into()
}

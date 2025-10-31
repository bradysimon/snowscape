use iced::{
    Element,
    widget::{button, column, text},
};

// Example message type for the components
#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    Decrement,
}

// Stateless preview with no parameters
#[snowscape::preview]
pub fn simple_text() -> Element<'static, Message> {
    text("Hello, Snowscape!").into()
}

// Stateless preview with parameters
#[snowscape::preview("Hello")]
#[snowscape::preview("World")]
#[snowscape::preview("Rust")]
pub fn parameterized_text(content: &str) -> Element<'_, Message> {
    text(content).size(32).into()
}

// Stateless preview returning a more complex layout
#[snowscape::preview]
pub fn button_column() -> Element<'static, Message> {
    column![
        text("Click the buttons:").size(20),
        button("Increment").on_press(Message::Increment),
        button("Decrement").on_press(Message::Decrement),
    ]
    .spacing(10)
    .padding(20)
    .into()
}

fn main() -> iced::Result {
    println!("Starting Snowscape preview...");
    println!("Found {} previews", snowscape::previews().len());

    for preview in snowscape::previews() {
        println!("  - {}", preview.label);
    }

    snowscape::run()
}

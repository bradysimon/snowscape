use iced::{
    Element,
    widget::{button, column, text},
};

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Default)]
pub struct App {
    count: i32,
}

impl App {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            text(format!("Count: {}", self.count)).size(32),
            button("Increment").on_press(Message::Increment),
            button("Decrement").on_press(Message::Decrement),
        ]
        .spacing(10)
        .padding(20)
        .into()
    }
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

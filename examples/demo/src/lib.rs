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
        counter(self.count)
    }
}

#[snowscape::preview]
fn add_button<'a>() -> Element<'a, Message> {
    button("Increment").on_press(Message::Increment).into()
}

#[snowscape::preview]
fn minus_button<'a>() -> Element<'a, Message> {
    button("Decrement").on_press(Message::Decrement).into()
}

#[snowscape::preview(0)]
#[snowscape::preview(5)]
#[snowscape::preview(10)]
fn label<'a>(count: i32) -> Element<'a, Message> {
    text(format!("Count: {}", count)).size(32).into()
}

// Stateless preview returning a more complex layout
#[snowscape::preview(0)]
pub fn counter(count: i32) -> Element<'static, Message> {
    column![label(count), add_button(), minus_button()]
        .spacing(10)
        .padding(20)
        .into()
}

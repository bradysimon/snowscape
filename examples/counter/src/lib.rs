use iced::{
    Alignment::Center,
    Element,
    widget::{button, column, text, text::IntoFragment},
};

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    Decrement,
}

#[derive(Debug, Clone, Default)]
pub struct App {
    pub count: i32,
}

impl App {
    pub fn new(count: i32) -> Self {
        Self { count }
    }

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

pub fn add_button<'a>() -> Element<'a, Message> {
    button("Increment").on_press(Message::Increment).into()
}

pub fn minus_button<'a>() -> Element<'a, Message> {
    button("Decrement").on_press(Message::Decrement).into()
}

pub fn label<'a>(content: impl IntoFragment<'a>) -> Element<'a, Message> {
    text(format!("Count: {}", content.into_fragment()))
        .size(32)
        .into()
}

// Stateless preview returning a more complex layout
pub fn counter(count: i32) -> Element<'static, Message> {
    column![label(count), add_button(), minus_button()]
        .align_x(Center)
        .spacing(10)
        .padding(20)
        .into()
}

/// A counter with adjustable labels for the increment and decrement buttons.
pub fn adjustable_counter<'a>(
    count: i32,
    inc_label: &'a str,
    dec_label: &'a str,
) -> Element<'a, Message> {
    column![
        label(count),
        button(inc_label).on_press(Message::Increment),
        button(dec_label).on_press(Message::Decrement)
    ]
    .align_x(Center)
    .spacing(10)
    .padding(20)
    .into()
}

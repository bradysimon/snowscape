use std::time::Duration;

use iced::{
    Alignment::Center,
    Element, Task,
    widget::{button, column, container, text, text::IntoFragment},
};

pub const INCREMENT_BUTTON_ID: &str = "increment-button";
pub const COUNT_TEXT_ID: &str = "count-text";

#[derive(Debug, Clone)]
pub enum Message {
    Increment,
    Decrement,
    DelayedIncrement,
}

#[derive(Debug, Clone, Default)]
pub struct App {
    pub count: i32,
}

impl App {
    pub fn new(count: i32) -> Self {
        Self { count }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => {
                self.count += 1;
                Task::none()
            }
            Message::Decrement => {
                self.count -= 1;
                Task::none()
            }
            Message::DelayedIncrement => Task::perform(
                async { tokio::time::sleep(Duration::from_millis(150)).await },
                |()| Message::Increment,
            ),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        counter(self.count)
    }
}

pub fn add_button<'a>() -> Element<'a, Message> {
    container(button("Increment").on_press(Message::Increment))
        .id(INCREMENT_BUTTON_ID)
        .into()
}

pub fn minus_button<'a>() -> Element<'a, Message> {
    button("Decrement").on_press(Message::Decrement).into()
}

pub fn delayed_button<'a>() -> Element<'a, Message> {
    button("Delayed Increment")
        .on_press(Message::DelayedIncrement)
        .into()
}

pub fn label<'a>(content: impl IntoFragment<'a>) -> Element<'a, Message> {
    container(text!("Count: {}", content.into_fragment()).size(32))
        .id(COUNT_TEXT_ID)
        .into()
}

// Stateless preview returning a more complex layout
pub fn counter<'a>(count: i32) -> Element<'a, Message> {
    column![label(count), add_button(), minus_button(), delayed_button()]
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

/// Builds the counter as a standalone [`iced::Program`] for use in
/// `snowscape::test::Emulator`-driven automation tests as well as for
/// standalone running.
pub fn program() -> impl iced::Program<State = App, Message = Message, Theme = iced::Theme> {
    iced::application(App::default, App::update, App::view).title("Counter")
}

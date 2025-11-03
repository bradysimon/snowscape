use iced::{
    Alignment::Center,
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

pub fn label<'a>(count: i32) -> Element<'a, Message> {
    text(format!("Count: {}", count)).size(32).into()
}

// Stateless preview returning a more complex layout
pub fn counter(count: i32) -> Element<'static, Message> {
    column![label(count), add_button(), minus_button()]
        .align_x(Center)
        .spacing(10)
        .padding(20)
        .into()
}

// mod previews {
//     use super::*;

//     #[snowscape::stateless]
//     fn add_button_preview<'a>() -> Element<'a, Message> {
//         add_button()
//     }

//     #[snowscape::stateless]
//     fn minus_button_preview() -> Element<'static, Message> {
//         minus_button()
//     }

//     #[snowscape::stateless(0)]
//     #[snowscape::stateless(5)]
//     #[snowscape::stateless(10)]
//     fn label_preview<'a>(n: i32) -> Element<'a, Message> {
//         label(n)
//     }

//     #[snowscape::stateful(App::update, App::view)]
//     fn stateful_counter() -> App {
//         App::default()
//     }

//     #[snowscape::stateful(App::update, App::view)]
//     fn stateful_counter_custom() -> App {
//         App { count: 100 }
//     }
// }

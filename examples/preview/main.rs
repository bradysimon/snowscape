use iced::{Element, widget::text};
use snowscape::test;

#[derive(Debug, Clone)]
pub enum Message {
    None,
}

#[derive(Debug, Clone, Default)]
pub struct App;

impl App {
    pub fn update(&mut self, _message: Message) {}

    pub fn view(&self) -> Element<Message> {
        text(test()).into()
    }
}

pub fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view).run()
}

use demo::App;

fn main() -> iced::Result {
    iced::application(App::default, App::update, App::view)
        .title("Counter")
        .run()
}

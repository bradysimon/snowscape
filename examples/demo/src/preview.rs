use demo::{App, add_button, label, minus_button};
use snowscape::preview::{stateful, stateless};

pub fn main() -> iced::Result {
    snowscape::run(|app| {
        app.title("My previews")
            .preview(stateless("Label", || label(42)))
            .preview(stateless("Increment", add_button).group("Button"))
            .preview(stateless("Decrement", minus_button).group("Button"))
            .preview(
                stateful("Counter", App::default, App::update, App::view)
                    .description("A counter that increments when the button is pressed")
                    .tags(["counter", "stateful"]),
            )
    })
}

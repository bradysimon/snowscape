use counter::{App, add_button, label, minus_button};
use snowscape::{dynamic, stateful, stateless};

pub fn main() -> iced::Result {
    snowscape::run(|app| {
        app.title("My previews")
            .preview(stateless("Label (0)", || label(0)))
            .preview(stateless("Label (42)", || label(42)))
            .preview(
                dynamic::stateless(
                    "Dynamic stateless",
                    dynamic::text("Content", "Editable"),
                    |content| label(content),
                )
                .description("A label with editable content"),
            )
            .preview(stateless("Increment", add_button).group("Button"))
            .preview(stateless("Decrement", minus_button).group("Button"))
            .preview(
                stateful("Counter", App::default, App::update, App::view)
                    .description("A counter that increments when the button is pressed")
                    .tags(["counter", "stateful"]),
            )
    })
}

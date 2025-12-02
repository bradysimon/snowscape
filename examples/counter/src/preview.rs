use counter::{App, add_button, adjustable_counter, adjustable_view, label, minus_button};
use snowscape::{dynamic, stateful, stateless};

pub fn main() -> iced::Result {
    snowscape::run(|app| {
        app.title("My previews")
            .preview(
                dynamic::stateless("Label", dynamic::text("Content", "Editable"), |content| {
                    label(content)
                })
                .description("A label with editable content"),
            )
            .preview(stateless("Increment", add_button).group("Button"))
            .preview(stateless("Decrement", minus_button).group("Button"))
            .preview(
                stateful("Counter", App::default, App::update, App::view)
                    .description("A counter that increments when the button is pressed")
                    .tags(["counter", "stateful"]),
            )
            .preview(dynamic::stateful(
                "Adjustable counter",
                (
                    dynamic::text("Increment label", "Increment"),
                    dynamic::text("Decrement label", "Decrement"),
                ),
                App::default,
                App::update,
                |state, params| {
                    let (inc_label, dec_label) = params;
                    adjustable_counter(state.count, inc_label, dec_label)
                },
            ))
            .preview(dynamic::stateless(
                "All dynamic params",
                (
                    dynamic::text("Label", "The meaning of life"),
                    dynamic::number("The magic number", 42),
                    dynamic::boolean("A toggle", true),
                ),
                |(label, number, toggle)| adjustable_view(label, *number, *toggle),
            ))
    })
}

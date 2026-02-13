use std::path::PathBuf;

use counter::{App, add_button, adjustable_counter, label, minus_button};
use snowscape::{dynamic, stateful, stateless};

/// Configures the Snowscape app with all counter previews.
///
/// This function is shared between the main application and tests.
pub fn previews(app: snowscape::App) -> snowscape::App {
    app.title("Counter Previews")
        .with_tests_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests"))
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
}

pub fn main() -> iced::Result {
    snowscape::run(previews)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_visual_tests() -> Result<(), snowscape::test::Error> {
        snowscape::test::run(
            previews,
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests"),
        )
    }
}

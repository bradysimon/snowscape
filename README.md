# Snowscape

This crate will be for previewing components build using [Iced](https://github.com/iced-rs/iced).
This is a placeholder since the crate depends on some unreleased Iced APIs.
This crate will let you create a preview application that displays your 
components in isolation, similar to Storybook for React or SwiftUI Previews.

If you're familiar with Iced, this is what the usage will look like. Stateless
widgets are just functions that return an `iced::Element` and ignore messages, 
while stateful widgets have their own state and can respond to messages.

```rs
use snowscape::{stateful, stateless};

/// A counter app that has various parts of the counter UI as
/// individual functions that can be previewed in isolation.
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
```

In the meantime, follow along with this crate's development at https://github.com/bradysimon/snowscape

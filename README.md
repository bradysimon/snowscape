# Snowscape

A preview system for [Iced](https://github.com/iced-rs/iced) UI components, inspired by Storybook and SwiftUI previews.

## Features

- Register functions that return elements for previewing
- Parameterize previews to see how they look with different inputs
- See all previewable views in your app

## Quick Start

### 1. Add to your `Cargo.toml`

```toml
[dependencies]
snowscape = "0.1.0"
iced = "0.14.0-dev"

[[example]]
name = "preview"
path = "examples/preview.rs"
```

### 2. Mark functions with `#[snowscape::stateless]` or `#[snowscape::stateful]`

```rust
use iced::{Element, widget::{button, column, text}};

#[derive(Debug, Clone)]
pub enum Message {
    ButtonClicked,
    Increment,
    Decrement,
}

// Simple stateless preview
#[snowscape::stateless]
pub fn hello_world<'a>() -> Element<'a, Message> {
    text("Hello, World!").into()
}

// Parameterized stateless preview - test with different inputs
#[snowscape::stateless("Click Me")]
#[snowscape::stateless("Press Here")]
#[snowscape::stateless("Tap This")]
pub fn custom_button(label: &str) -> Element<'_, Message> {
    button(text(label))
        .on_press(Message::ButtonClicked)
        .into()
}

// Stateful preview - interactive components
#[derive(Debug, Clone, Default)]
pub struct Counter {
    count: i32,
}

impl Counter {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
            _ => {}
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        column![
            text(format!("Count: {}", self.count)),
            button("Increment").on_press(Message::Increment),
            button("Decrement").on_press(Message::Decrement),
        ]
        .into()
    }
}

#[snowscape::stateful(Counter::update, Counter::view)]
pub fn interactive_counter() -> Counter {
    Counter::default()
}
```

### 3. Create a preview runner

```rust
// examples/preview.rs
fn main() -> iced::Result {
    snowscape::run()
}
```

### 4. Run your previews

```bash
cargo run --example preview
```

## How It Works

Snowscape uses procedural macros and compile-time registration to automatically discover and display your preview functions. When you add `#[snowscape::stateless]` or `#[snowscape::stateful]` to a function:

1. The original function remains unchanged (or becomes the boot function for stateful previews)
2. A preview registration is generated automatically
3. Stateless previews ignore messages; stateful previews handle them with your update function

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed implementation notes.

## Roadmap

- [X] Preview stateless components
- [X] See all available previewable components
- [X] Stateful preview support
- [ ] Improved preview metadata (names, descriptions, categories)
- [ ] Search/filter previews
- [ ] Custom themes
- [ ] Layout options (centered, fullscreen, grid)

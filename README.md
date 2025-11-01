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

### 2. Mark functions with `#[snowscape::preview]`

```rust
use iced::{Element, widget::{button, column, text}};

#[derive(Debug, Clone)]
pub enum Message {
    ButtonClicked,
}

// Simple preview
#[snowscape::preview]
pub fn hello_world<'a>() -> Element<'a, Message> {
    text("Hello, World!").into()
}

// Parameterized preview - test with different inputs
#[snowscape::preview("Click Me")]
#[snowscape::preview("Press Here")]
#[snowscape::preview("Tap This")]
pub fn custom_button(label: &str) -> Element<'_, Message> {
    button(text(label))
        .on_press(Message::ButtonClicked)
        .into()
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

Snowscape uses procedural macros and compile-time registration to automatically discover and display your preview functions. When you add `#[snowscape::preview]` to a function:

1. The original function remains unchanged
2. A preview registration is generated automatically
3. Messages are currently ignored due to previews being stateless

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed implementation notes.

## Roadmap

- [X] Preview stateless components
- [X] See all available previewable components
- [ ] Improved preview metadata (names, descriptions, categories)
- [ ] Search/filter previews
- [ ] Custom themes
- [ ] Layout options (centered, fullscreen, grid)
- [ ] Stateful preview support

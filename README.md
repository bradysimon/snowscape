# Snowscape 🏔️

A preview system for [Iced](https://github.com/iced-rs/iced) UI components, inspired by Storybook and SwiftUI previews.

## Features

- ✨ **Zero Boilerplate** - Automatic preview discovery with no manual registration
- 🎯 **Parameterized Previews** - Test components with different inputs
- 🔄 **Multiple Variants** - Stack preview attributes for different scenarios  
- 🎨 **Type Safe** - Works with any message type automatically
- 🚀 **Simple API** - Just add `#[snowscape::preview]` to your view functions

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
pub fn hello_world() -> Element<'static, Message> {
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

// Complex layout preview
#[snowscape::preview]
pub fn button_column() -> Element<'static, Message> {
    column![
        text("Welcome!").size(24),
        button("Click Me").on_press(Message::ButtonClicked),
        text("Some description text").size(12),
    ]
    .spacing(10)
    .padding(20)
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
3. Message types are converted transparently
4. The preview becomes available in the preview runner

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed implementation notes.

## Examples

### Stateless Preview

Perfect for simple, non-interactive components:

```rust
#[snowscape::preview]
pub fn loading_spinner() -> Element<'static, Message> {
    text("Loading...").into()
}
```

### Parameterized Preview

Test your component with different inputs:

```rust
#[snowscape::preview(16)]
#[snowscape::preview(24)]
#[snowscape::preview(32)]
pub fn sized_text(size: u16) -> Element<'_, Message> {
    text("Hello").size(size).into()
}
```

### String Parameters

```rust
#[snowscape::preview("Success")]
#[snowscape::preview("Warning")]  
#[snowscape::preview("Error")]
pub fn status_badge(status: &str) -> Element<'_, Message> {
    let color = match status {
        "Success" => iced::Color::from_rgb(0.0, 1.0, 0.0),
        "Warning" => iced::Color::from_rgb(1.0, 1.0, 0.0),
        "Error" => iced::Color::from_rgb(1.0, 0.0, 0.0),
        _ => iced::Color::WHITE,
    };
    
    container(text(status))
        .style(|_| container::Style {
            background: Some(Background::Color(color)),
            ..Default::default()
        })
        .padding(10)
        .into()
}
```

## Current Limitations

- **Single Preview Display**: Currently shows only the first registered preview
- **Message Handling**: Messages from previewed components are discarded
- **No State**: Preview components are stateless (state support planned)

## Roadmap

- [ ] Preview selector UI to switch between previews
- [ ] Stateful preview support with full update/view cycle
- [ ] Theme customization
- [ ] Hot reload support
- [ ] Preview metadata (descriptions, categories)
- [ ] Layout options (centered, fullscreen, grid)

## Requirements

- Rust 2024 edition
- Iced master branch (0.14.0-dev)

## Contributing

Contributions welcome! Please see [ARCHITECTURE.md](ARCHITECTURE.md) for implementation details.

## License

MIT OR Apache-2.0

## Acknowledgments

Inspired by:
- [Storybook](https://storybook.js.org/) - Component development for web
- SwiftUI Previews - Xcode's preview system
- [iced](https://github.com/iced-rs/iced) - The wonderful Rust GUI framework

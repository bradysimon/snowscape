# Snowscape

A preview system for [Iced](https://github.com/iced-rs/iced) UI components, inspired by Storybook and SwiftUI previews.

## Quick Start

The following assumes you have an existing Iced application and are using the
master version of Iced. There isn't a usable crates.io release of Snowscape yet
due to Iced not having a recent release.

### 1. Add to your `Cargo.toml`

Update your `Cargo.toml` to include Snowscape as a dependency. You'll also be
setting up multiple binaries for your main app and previews if you don't
already have that set up.

```toml
[package]
default-run = "main" # Ensures `cargo run` runs your main app

[dependencies]
snowscape = { git = "https://github.com/bradysimon/snowscape", branch = "main" }
iced = "0.14.0-dev"

[[bin]]
name = "main"
path = "src/main.rs"

# Preview UI components
[[bin]]
name = "preview"
path = "src/preview.rs"
```

### 2. Create `preview.rs`

This file will be the binary hosting all your previews. Feel free to use a
different name other than `preview.rs`: just make sure it matches the name in
your `Cargo.toml`.

```rust
// preview.rs
use iced::widget::text;
use snowscape::stateless;

fn label(name: &str) -> iced::Element<'_, ()> {
    text(format!("Hello, {}!", name)).into()
}

fn goodbye<'a>() -> iced::Element<'a, ()> {
    text("Goodbye").into()
}

fn main() -> iced::Result {
    snowscape::run(|app| {
        app.preview(stateless("Hello world", || label("world")))
            .preview(stateless("Goodbye", goodbye))
    })
}
```

### 4. Run your previews

This will launch an Iced app that will allow you to preview your components.

```bash
cargo run --bin preview
```

## Roadmap

- [X] Preview stateless components
- [X] Preview stateful components
- [X] Improved preview metadata (descriptions, groups, tags)
- [ ] Search/filter previews
- [ ] Custom themes
- [ ] Layout options (centered, fullscreen, grid)
- [ ] More I haven't thought of yet :)

## License

Snowscape is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

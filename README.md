# Snowscape

A preview system for [Iced](https://github.com/iced-rs/iced) UI components,
inspired by Storybook and SwiftUI previews.

Snowscape makes it easy to preview specific Iced elements in isolation while
developing your application. It provides an API for you to run an Iced app that
displays your elements, allowing you to quickly iterate on specific components
without having to navigate through your entire app.

## Quick Start

The following assumes you have an existing Iced application and are using the
master version of Iced. There isn't a usable crates.io release of Snowscape yet
due to Iced not having a recent release. You can set up the preview application
to either run as an example within your project or as a separate binary.

### 1. Add `snowscape` to your dependencies

Update your `Cargo.toml` to include Snowscape as a dependency. If you're using
the master branch of Iced, then you'll want to the main branch of Snowscape:

```toml
[dependencies]
snowscape = { git = "https://github.com/bradysimon/snowscape", branch = "main" }
```

### 2. (For separate binaries) Configure the preview binary

For separate binaries, update your `Cargo.toml` to include a new binary for the
previews, which we'll call `preview` here (and assuming your existing main app is
in `src/main.rs`). If you're using examples instead, skip this step.

```toml
[package]
default-run = "main" # Ensures `cargo run` runs your main app

[[bin]]
name = "main"
path = "src/main.rs"

# Preview UI elements
[[bin]]
name = "preview"
path = "src/preview.rs"
```


### 3. Create the preview file

This file will be the binary hosting all your previews. If you're using
examples, then create `examples/preview.rs`, and do `src/preview.rs` for 
separate binaries. Feel free to use a different name other than `preview.rs`: 
just make sure to adjust the `Cargo.toml` accordingly for separate binaries
and use `cargo run --example <name>` for examples.

```rust
// preview.rs
use iced::widget::text;
use snowscape::stateless;

// Import your own components here and add previews for them below
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

**Examples:**
```bash
cargo run --example preview
```

**Separate binary:**
```bash
cargo run --bin preview
```

## Roadmap

- [X] Preview stateless components
- [X] Preview stateful components
- [X] Improved preview metadata (descriptions, groups, tags)
- [X] Search/filter previews
- [ ] Custom themes
- [ ] Layout options (centered, fullscreen, grid)
- [ ] More I haven't thought of yet :)

## License

Snowscape is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

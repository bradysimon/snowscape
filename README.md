# Snowscape

A preview system for [Iced](https://github.com/iced-rs/iced) UI components,
inspired by Storybook and SwiftUI previews.

Snowscape makes it easy to preview specific Iced elements in isolation while
developing your application. It provides an API for you to run an Iced app that
displays your elements, allowing you to quickly iterate and test specific 
components without having to navigate through your entire app. Snowscape 
supports:

- Documenting how components work and how you can use them in a searchable,
  interactive environment
- Adjusting dynamic parameters for your preview that adjusts the preview in
  real time
- Seeing all the messages a preview emits
- Tracking view/update performance of each preview
- Recording, managing, and running tests for previews
- Capturing screenshots of your previews from the command line

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
    text!("Hello, {}!", name).into()
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

## Creating preview tests

You can create tests for previews either manually or from within Snowscape.
These are `.ice` test files that run against one of your previews, perform
some interactions, then assert that certain parts of the UI are how you expect.
You can choose a test name and window dimensions and then begin recording a
test, which will open a new window with only your preview. Interactions with
the preview are recorded into a `.ice` file, and you can add text that you
expect to see from within the main window.

These tests are then saved at a location of your choosing (configurable using
`with_tests_dir` function when building your previews) and can be re-run both
from within Snowscape or via the command line to act as automation tests.
Refer to the `counter` example for a simple example to follow.

```rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_visual_tests() -> Result<(), snowscape::test::Error> {
        snowscape::test::run(previews, format!("{}/tests", env!("CARGO_MANIFEST_DIR")))
    }
}
```

## Running the examples

You can do `cargo run -p {package_name}` to run any of the included examples.
You can run the `self` example with `cargo run -p self` to see Snowscape's 
own previews.

## Capture screenshots of previews

Snowscape can capture screenshots of your previews from the command line
without launching the GUI. This may be useful for documentation and 
automated testing. This works through the same `snowscape::run` API used for
the GUI, but instead of launching a window, it captures a snapshot of the
preview you specify via the `--screenshot` flag passed on the command line.

The `--screenshot` name passed is the name you've given to the preview.

> cargo run --example preview -- --screenshot "My Button"

### Options

- `--screenshot <name>` - Name of the preview to capture (supports partial, case-insensitive matching)
- `--output <path>` - Output path for the PNG file (default: `./screenshots/<name>.png`)
  - Note: file name will include a counter if a file with the same name already exists,
    and the renderer name (e.g., `-wgpu`) is automatically added as a suffix.
- `--theme <name>` - Iced theme to use for rendering (default: `Light`)
- `--size <WxH>` - Viewport size in pixels (default: `800x600`)
- `--help` - Show help message

## License

Snowscape is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

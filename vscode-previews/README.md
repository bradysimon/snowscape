# Snowscape Previews - VS Code Extension

A Visual Studio Code extension that provides "Run Preview" code lenses for Snowscape preview functions.

## Features

- **Code Lenses**: Adds clickable "▶ Run Preview" buttons above functions marked with `#[snowscape::stateless]` or `#[snowscape::stateful]`
- **Terminal Integration**: Launches previews in an integrated terminal
- **Configurable**: Customize the preview command through VS Code settings

## Installation

### From VSIX (Local Development)

1. Package the extension:
   ```bash
   npm install
   npm run package
   ```

2. Install the generated `.vsix` file:
   ```bash
   code --install-extension snowscape-previews-0.0.1.vsix
   ```

### For Development

1. Open this folder in VS Code
2. Press `F5` to launch the Extension Development Host
3. Open your Snowscape project in the new window

## Usage

When you have a Rust file with Snowscape preview functions, you'll see "▶ Run Preview" buttons appear above them:

```rust
#[snowscape::stateless]
fn my_button() -> Element<'_, Message> {
    button("Click me").into()
}
// ▶ Run Preview appears above this function
```

Click the button to run the preview in a terminal.

## Configuration

- `snowscape.previewCommand`: Base command to run previews (default: `cargo run --bin preview`)
  - The extension will automatically detect workspace packages and add `-p <package>` when needed
  - For example, in a workspace, `cargo run --bin preview` becomes `cargo run -p demo --bin preview`
- `snowscape.enableCodeLens`: Enable or disable code lenses (default: `true`)

## Workspace Support

The extension automatically detects:
- **Single-package projects**: Uses the command as-is
- **Cargo workspaces**: Adds `-p <package-name>` to target the correct package
- **Nested packages**: Finds the nearest `Cargo.toml` to determine the package name

This means you can use the same configuration across different project types!

## Requirements

- VS Code 1.80.0 or higher
- Rust language support (rust-analyzer recommended)
- A Snowscape project with preview functions

## Development

```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Watch for changes
npm run watch

# Run linter
npm run lint

# Package the extension
npm run package
```

## License

MIT

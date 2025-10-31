# Snowscape - Iced Component Preview System

## Summary

**Snowscape** is a Rust crate that enables easy previewing of Iced UI components in isolation. It uses procedural macros and compile-time registration to automatically discover and display preview functions without manual registration.

## Architecture Overview

### Core Components

1. **`#[snowscape::preview]` Macro** - Marks functions as previewable
2. **Inventory-based Registration** - Automatic compile-time discovery
3. **Preview Trait** - Abstraction for different preview types
4. **Message Type Conversion** - Handles generic message types

### Key Features Implemented

✅ **Sidebar Preview Selector** - Interactive UI to browse and switch between previews
✅ **Parameterized Previews** - Support for preview functions with parameters
✅ **Automatic Registration** - No manual registration required
✅ **Multiple Preview Variants** - Stack multiple `#[preview]` attributes with different parameters
✅ **Message Type Erasure** - Works with any user-defined message type

## How It Works

### 1. Message Type Conversion

**Challenge**: Preview functions return `Element<'_, UserMessage>` but the preview app needs a unified message type.

**Solution**: The proc macro wraps function calls with `.map()` to convert messages:

```rust
// User writes:
#[snowscape::preview]
pub fn my_button() -> Element<'_, Message> {
    button("Click me").on_press(Message::Clicked).into()
}

// Macro generates:
snowscape::inventory::submit! {
    snowscape::PreviewDescriptor {
        label: "my_button",
        create: || {
            Box::new(snowscape::StatelessPreview::new(|| {
                (my_button()).map(|_| snowscape::PreviewMessage::Noop)
            }))
        },
    }
}
```

The user's message is discarded (mapped to `Noop`) since we're previewing in isolation.

### 2. Automatic Registration with Inventory

**Challenge**: Discover all preview functions at runtime without manual registration.

**Solution**: Use the `inventory` crate:

```rust
// In snowscape/src/lib.rs:
pub struct PreviewDescriptor {
    pub label: &'static str,
    pub create: fn() -> Box<dyn Preview>,
}

inventory::collect!(PreviewDescriptor);

pub fn previews() -> Vec<&'static PreviewDescriptor> {
    inventory::iter::<PreviewDescriptor>().collect()
}
```

The macro emits `inventory::submit!` calls that register each preview at compile time.

### 3. Parameterized Previews

**Challenge**: Support preview functions with parameters.

**Solution**: The macro accepts attribute arguments and generates unique registrations:

```rust
// User writes:
#[snowscape::preview("Hello")]
#[snowscape::preview("World")]
pub fn text_preview(text: &str) -> Element<'_, Message> {
    text(text).into()
}

// Generates two registrations:
// 1. text_preview("Hello") -> label: "text_preview(\"Hello\")"
// 2. text_preview("World") -> label: "text_preview(\"World\")"
```

### 4. Preview Trait Design

```rust
pub trait Preview: Send {
    fn update(&mut self, message: PreviewMessage) -> Task<PreviewMessage>;
    fn view(&self) -> Element<'_, PreviewMessage>;
    fn label(&self) -> &str;
}
```

This allows for both stateless and stateful previews:

- **StatelessPreview**: For simple view functions
- **StatefulPreview**: For components needing state (future enhancement)

### 5. Sidebar Preview Selector UI

**Challenge**: Allow users to browse and switch between multiple previews at runtime.

**Solution**: Implement a sidebar navigation system:

```rust
struct PreviewApp {
    descriptors: Vec<&'static PreviewDescriptor>,
    selected_index: usize,
    current_preview: Box<dyn Preview>,
}

enum PreviewMessage {
    SelectPreview(usize),  // Switch to a different preview
    PreviewComponent,      // Messages from the current preview
    Noop,
}
```

**UI Layout**:
- **Left Sidebar** (250px): Scrollable list of preview buttons
  - Shows all preview labels
  - Highlights the selected preview
  - Click to switch previews
- **Right Content Area**: Displays the selected preview
  - Header showing current preview name
  - Centered preview content with padding

**Preview Switching**:
When a user clicks a preview in the sidebar:
1. `SelectPreview(index)` message is sent
2. Update handler creates a new preview instance
3. View is refreshed to show the new preview

This provides a seamless browsing experience similar to Storybook.

## Usage Examples

### Basic Stateless Preview

```rust
use iced::{Element, widget::text};

#[derive(Debug, Clone)]
pub enum Message {
    // Your message types
}

#[snowscape::preview]
pub fn hello_world() -> Element<'static, Message> {
    text("Hello, World!").into()
}
```

### Parameterized Previews

```rust
#[snowscape::preview("Small")]
#[snowscape::preview("Medium")]
#[snowscape::preview("Large")]
pub fn sized_text(size: &str) -> Element<'_, Message> {
    let size_value = match size {
        "Small" => 16,
        "Medium" => 24,
        "Large" => 32,
        _ => 20,
    };
    text(size).size(size_value).into()
}
```

### Running Previews

```rust
// In examples/demo/src/main.rs or a dedicated preview binary:
fn main() -> iced::Result {
    snowscape::run()
}
```

## Architecture Decisions & Considerations

### 1. Inventory vs. Manual Registration

**Chosen**: Inventory (automatic)

**Pros**:
- Zero boilerplate for users
- Can't forget to register a preview
- Works across crate boundaries

**Cons**:
- Slightly slower compile times
- All previews always loaded
- Less control over which previews run

### 2. Message Handling Strategy

**Chosen**: Map to `Noop` (discard messages)

**Rationale**:
- Simplifies implementation
- Previews are meant to be stateless demos
- Full interactivity requires the real app context

**Future Enhancement**: Support stateful previews with real message handling using type-erased message routing.

### 3. Preview Discovery Order

**Current**: Order depends on link order (non-deterministic)

**Future Enhancement**: Add explicit ordering or alphabetical sorting.

### 4. Multiple Preview Selection

**Current**: Runs only the first preview

**Future Enhancement**: Build a selector UI to choose between previews.

## Challenges & Solutions

### Challenge 1: Lifetime Management

**Problem**: Preview closures must be `'static` but reference function calls.

**Solution**: The `create` function returns `Box<dyn Preview>`, allowing each preview to own its data.

### Challenge 2: Generic Message Types

**Problem**: Different components use different message types.

**Solution**: Use `.map(|_| PreviewMessage::Noop)` to erase the original message type.

### Challenge 3: Iced's New API

**Problem**: No `Sandbox` trait in master branch.

**Solution**: Use `iced::application()` with proper `boot`, `update`, and `view` functions.

### Challenge 4: Proc Macro Complexity

**Problem**: Parsing attribute arguments and generating correct code.

**Solution**: Keep it simple - parse attrs as strings and use `quote!` to generate clean code.

## Future Enhancements

### 1. Stateful Previews
Support components that need state:

```rust
#[snowscape::preview]
pub fn counter() -> impl snowscape::Preview {
    snowscape::stateful(
        Counter::default(),
        Counter::update,
        Counter::view
    )
}
```

### 2. Hot Reload
Watch for file changes and reload previews without recompiling.

### 3. Theme Support
Allow previews to specify themes:

```rust
#[snowscape::preview(theme = "dark")]
pub fn dark_ui() -> Element<'_, Message> { ... }
```

### 4. Layout Options
Support different preview layouts (centered, full-width, grid, etc.).

### 5. Search and Filter
Add search functionality to filter previews by name or tags.

### 6. Preview Metadata
Add descriptions, categories, and tags:

```rust
#[snowscape::preview(
    description = "A red button with hover effect",
    category = "Buttons"
)]
```

## Technical Notes

### Why `inventory`?
The `inventory` crate provides distributed plugin registration. It's perfect for:
- Collecting items across multiple modules
- Zero-cost at runtime (collection happens at link time)
- No need for a central registry

### Why Box<dyn Preview>?
- Different preview types (stateless vs. stateful)
- Allows future extension without breaking changes
- Minimal overhead for the preview use case

### Why 'static lifetimes?
- Preview descriptors must live for the entire program
- Simplifies the registration system
- Matches iced's application model

## Comparison with Alternative Approaches

### Approach 1: Feature Flag + Main Wrapper
```rust
#[snowscape::main]
fn main() { ... }
```
**Pros**: Single source file  
**Cons**: Requires conditional compilation, messy with feature flags

### Approach 2: Separate Binary (Chosen)
```rust
// examples/demo/src/main.rs
fn main() { snowscape::run() }
```
**Pros**: Clean separation, easy to run  
**Cons**: Extra binary target

## Conclusion

Snowscape provides an elegant solution for previewing Iced components by:
1. Using proc macros for zero-boilerplate registration
2. Leveraging `inventory` for automatic discovery
3. Handling message type erasure transparently
4. Supporting parameterized preview variants
5. Providing an interactive sidebar UI for browsing previews

The architecture is extensible for future enhancements like stateful previews, theme support, and hot reload functionality.

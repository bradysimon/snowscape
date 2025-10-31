// Re-export the proc macro
pub use snowscape_macros::preview;

// Re-export inventory for use in generated code
#[doc(hidden)]
pub use inventory;

use iced::{Element, Task};
use std::fmt;

/// A descriptor for a preview component that can be registered.
pub struct PreviewDescriptor {
    pub label: &'static str,
    pub create: fn() -> Box<dyn Preview>,
}

inventory::collect!(PreviewDescriptor);

/// Trait for preview components that can be displayed in the preview window.
pub trait Preview: Send {
    /// Update the preview state with a message.
    fn update(&mut self, message: PreviewMessage) -> Task<PreviewMessage>;

    /// Render the preview.
    fn view(&self) -> Element<'_, PreviewMessage>;

    /// Get the label for this preview.
    fn label(&self) -> &str;
}

impl fmt::Debug for dyn Preview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Preview")
            .field("label", &self.label())
            .finish()
    }
}

/// Message type for the preview system.
#[derive(Debug, Clone)]
pub enum PreviewMessage {
    /// No-op message.
    Noop,
}

/// A stateless preview that renders a view function.
pub struct StatelessPreview<F>
where
    F: Fn() -> Element<'static, PreviewMessage> + Send + 'static,
{
    view_fn: F,
    label: String,
}

impl<F> StatelessPreview<F>
where
    F: Fn() -> Element<'static, PreviewMessage> + Send + 'static,
{
    pub fn new(view_fn: F) -> Self {
        Self {
            view_fn,
            label: "Stateless Preview".to_string(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl<F> Preview for StatelessPreview<F>
where
    F: Fn() -> Element<'static, PreviewMessage> + Send + 'static,
{
    fn update(&mut self, _message: PreviewMessage) -> Task<PreviewMessage> {
        Task::none()
    }

    fn view(&self) -> Element<'_, PreviewMessage> {
        (self.view_fn)()
    }

    fn label(&self) -> &str {
        &self.label
    }
}

/// A stateful preview with full update/view cycle.
pub struct StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + 'static,
{
    state: State,
    update_fn: fn(&mut State, Msg) -> Task<Msg>,
    view_fn: fn(&State) -> Element<'_, Msg>,
    label: String,
}

impl<State, Msg> StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
{
    pub fn new(
        state: State,
        update_fn: fn(&mut State, Msg) -> Task<Msg>,
        view_fn: fn(&State) -> Element<'_, Msg>,
    ) -> Self {
        Self {
            state,
            update_fn,
            view_fn,
            label: "Stateful Preview".to_string(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl<State, Msg> Preview for StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
{
    fn update(&mut self, _message: PreviewMessage) -> Task<PreviewMessage> {
        // For now, stateful previews don't handle messages from the UI
        // This would require more complex message routing
        Task::none()
    }

    fn view(&self) -> Element<'_, PreviewMessage> {
        (self.view_fn)(&self.state).map(|_msg| PreviewMessage::Noop)
    }

    fn label(&self) -> &str {
        &self.label
    }
}

/// Helper function to create a stateful preview.
pub fn stateful<State, Msg>(
    state: State,
    update_fn: fn(&mut State, Msg) -> Task<Msg>,
    view_fn: fn(&State) -> Element<'_, Msg>,
) -> StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
{
    StatefulPreview::new(state, update_fn, view_fn)
}

/// Get all registered previews.
pub fn previews() -> Vec<&'static PreviewDescriptor> {
    inventory::iter::<PreviewDescriptor>().collect()
}

/// Run the preview application.
pub fn run() -> iced::Result {
    let preview_list = previews();

    if preview_list.is_empty() {
        eprintln!("No previews found. Add #[snowscape::preview] to your functions.");
        return Ok(());
    }

    // For now, just run the first preview
    // TODO: Build a selector UI
    let descriptor = preview_list[0];

    PreviewApp::run(descriptor)
}

/// The preview application wrapper.
struct PreviewApp {
    preview: Box<dyn Preview>,
}

impl PreviewApp {
    fn run(descriptor: &'static PreviewDescriptor) -> iced::Result {
        iced::application(
            || Self {
                preview: (descriptor.create)(),
            },
            Self::update,
            Self::view,
        )
        .run()
    }

    fn update(&mut self, message: PreviewMessage) -> Task<PreviewMessage> {
        self.preview.update(message)
    }

    fn view(&self) -> Element<'_, PreviewMessage> {
        self.preview.view()
    }
}

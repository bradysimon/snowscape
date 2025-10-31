mod app;
mod message;
mod widget;

use iced::{Element, Task};
use std::fmt;

use app::App;
pub use message::Message;
pub use snowscape_macros::preview;

// Re-export inventory for use in generated code
#[doc(hidden)]
pub use inventory;

/// A descriptor for a preview component that can be registered.
pub struct PreviewDescriptor {
    pub label: &'static str,
    pub create: fn() -> Box<dyn Preview>,
}

inventory::collect!(PreviewDescriptor);

/// Trait for preview components that can be displayed in the preview window.
pub trait Preview: Send {
    /// Update the preview state with a message.
    fn update(&mut self, message: Message) -> Task<Message>;

    /// Render the preview.
    fn view(&self) -> Element<'_, Message>;

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

/// A stateless preview that renders a view function.
pub struct StatelessPreview<F>
where
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    view_fn: F,
    label: String,
}

impl<F> StatelessPreview<F>
where
    F: Fn() -> Element<'static, Message> + Send + 'static,
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
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
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
    fn update(&mut self, _message: Message) -> Task<Message> {
        // For now, stateful previews don't handle messages from the UI
        // This would require more complex message routing
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        (self.view_fn)(&self.state).map(|_msg| Message::Noop)
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

    App::run(preview_list)
}

mod descriptor;
mod history;
mod stateful;
mod stateless;

pub use descriptor::Descriptor;
pub use history::History;
use iced::{Element, Task};
pub use stateful::Stateful;
pub use stateless::Stateless;

use crate::{Message, message::AnyMessage};

/// Trait for preview components that can be displayed in the preview window.
pub trait Preview: Send {
    /// Update the preview state with a message.
    fn update(&mut self, message: Message) -> Task<Message>;

    /// Render the preview.
    fn view(&self) -> Element<'_, Message>;

    /// Returns the history of the messages the preview has emitted.
    /// `None` indicates the preview doesn't support message tracking.
    fn history(&self) -> Option<&'_ [String]> {
        None
    }
}

pub fn stateless<F, Message>(label: impl Into<String>, view_fn: F) -> Stateless<F, Message>
where
    Message: AnyMessage,
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    let metadata = crate::Metadata::new(label);
    Stateless::new(view_fn, metadata)
}

pub fn stateful<Boot, State, Message, IntoTask>(
    label: impl Into<String>,
    boot: Boot,
    update_fn: fn(&mut State, Message) -> IntoTask,
    view_fn: fn(&State) -> Element<'_, Message>,
) -> Stateful<Boot, State, Message, IntoTask>
where
    Boot: Fn() -> State + Send,
    State: Send + 'static,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    let metadata = crate::Metadata::new(label);
    Stateful::new(boot, update_fn, view_fn, metadata)
}

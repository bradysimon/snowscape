mod descriptor;
mod stateful;
mod stateless;

use std::{any::Any, fmt::Debug};

pub use descriptor::Descriptor;
use iced::{Element, Task};
pub use stateful::Stateful;
pub use stateless::Stateless;

use crate::Message;

/// Trait for preview components that can be displayed in the preview window.
pub trait Preview: Send {
    /// Update the preview state with a message.
    fn update(&mut self, message: Message) -> Task<Message>;

    /// Render the preview.
    fn view(&self) -> Element<'_, Message>;

    /// Returns the history of the messages the preview has emitted.
    /// `None` indicates the preview doesn't support message tracking.
    fn history(&self) -> Option<Vec<String>> {
        None
    }
}

pub fn stateless<F, Message>(label: impl Into<String>, view_fn: F) -> Stateless<F, Message>
where
    Message: Send + Sync + Any + Clone + Debug + 'static,
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    let metadata = crate::Metadata::new(label);
    Stateless::new(view_fn, metadata)
}

pub fn stateful<Boot, State, Msg, IntoTask>(
    label: impl Into<String>,
    boot: Boot,
    update_fn: fn(&mut State, Msg) -> IntoTask,
    view_fn: fn(&State) -> Element<'_, Msg>,
) -> Stateful<Boot, State, Msg, IntoTask>
where
    Boot: Fn() -> State + Send,
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + Debug + 'static,
    IntoTask: Into<Task<Msg>>,
{
    let metadata = crate::Metadata::new(label);
    Stateful::new(boot, update_fn, view_fn, metadata)
}

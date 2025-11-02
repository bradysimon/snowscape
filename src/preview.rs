mod descriptor;
mod stateful;
mod stateless;

pub use descriptor::Descriptor;
use iced::{Element, Task};
pub use stateful::StatefulPreview;
pub use stateless::StatelessPreview;

use crate::Message;

/// Trait for preview components that can be displayed in the preview window.
pub trait Preview: Send {
    /// Update the preview state with a message.
    fn update(&mut self, message: Message) -> Task<Message>;

    /// Render the preview.
    fn view(&self) -> Element<'_, Message>;
}

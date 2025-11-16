mod descriptor;
mod history;
mod stateful;
mod stateless;
mod timeline;

use crate::Message;
use iced::{Element, Task};

pub use descriptor::Descriptor;
pub use history::History;
pub use stateful::{Stateful, stateful};
pub use stateless::{Stateless, stateless};
pub use timeline::Timeline;

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

    /// The index and range of the message timeline if the preview supports time travel.
    fn timeline(&self) -> Option<Timeline> {
        None
    }
}

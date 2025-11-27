mod descriptor;
pub mod dynamic;
mod history;
mod stateful;
mod stateless;
mod timeline;

use crate::{Message, preview::dynamic::Param};
use iced::{Element, Task};

pub use descriptor::Descriptor;
pub use dynamic::{Dynamic, dynamic};
pub use history::History;
pub use stateful::{Stateful, stateful};
pub use stateless::{Stateless, stateless};
pub use timeline::Timeline;

/// Trait for preview components that can be displayed in the preview window.
///
/// This must be a trait because the generic parameters (i.e. message types) for previews
/// can be different per preview, so we need a way to store them in a type-erased manner.
pub trait Preview: Send {
    /// Metadata associated with the preview.
    fn metadata(&self) -> &crate::Metadata;

    /// Update the preview state with a message.
    fn update(&mut self, message: Message) -> Task<Message>;

    /// Render the preview.
    fn view(&self) -> Element<'_, Message>;

    /// The total number of messages the preview has emitted.
    fn message_count(&self) -> usize;

    /// Returns the visible history of the messages the preview has emitted.
    /// This may be a subset of all messages if the preview supports time travel.
    fn visible_messages(&self) -> &'_ [String];

    /// The index and range of the message timeline if the preview supports time travel.
    fn timeline(&self) -> Option<Timeline> {
        None
    }

    /// The parameters for the dynamic preview if applicable.
    fn params(&self) -> Option<Vec<Param>> {
        None
    }
}

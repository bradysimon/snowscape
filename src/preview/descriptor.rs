use std::fmt::Debug;

use crate::{
    Metadata, Preview,
    message::AnyMessage,
    preview::{Dynamic, Stateful, Stateless},
};

/// A descriptor for a preview component that can be registered.
pub struct Descriptor {
    pub preview: Box<dyn Preview>,
}

impl Descriptor {
    /// Create a new [`Descriptor`] with the given label and preview.
    pub fn new(preview: Box<dyn Preview>) -> Self {
        Self { preview }
    }

    /// Get the metadata associated with the preview.
    pub fn metadata(&self) -> &Metadata {
        self.preview.metadata()
    }
}

impl Debug for Descriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Descriptor")
            .field("metadata", self.metadata())
            .finish()
    }
}

impl<F, Message> From<Stateless<F, Message>> for Descriptor
where
    F: Fn() -> iced::Element<'static, Message> + Send + 'static,
    Message: AnyMessage,
{
    fn from(stateless: Stateless<F, Message>) -> Self {
        Self {
            preview: Box::new(stateless),
        }
    }
}

impl<Boot, State, Message, IntoTask> From<Stateful<Boot, State, Message, IntoTask>> for Descriptor
where
    Boot: Fn() -> State + Send + 'static,
    State: Send + 'static,
    Message: AnyMessage,
    IntoTask: Into<iced::Task<Message>> + 'static,
{
    fn from(stateful: Stateful<Boot, State, Message, IntoTask>) -> Self {
        Self {
            preview: Box::new(stateful),
        }
    }
}

impl<P: Preview + 'static> From<Dynamic<P>> for Descriptor {
    fn from(dynamic: Dynamic<P>) -> Self {
        Self {
            preview: Box::new(dynamic),
        }
    }
}

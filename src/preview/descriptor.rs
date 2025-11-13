use std::fmt::Debug;

use crate::{
    Metadata, Preview,
    message::AnyMessage,
    preview::{Stateful, Stateless},
};

/// A descriptor for a preview component that can be registered.
pub struct Descriptor {
    /// Metadata associated with the preview.
    pub metadata: Metadata,

    pub preview: Box<dyn Preview>,
}

impl Descriptor {
    /// Create a new [`Descriptor`] with the given label and preview.
    pub fn new(label: impl Into<String>, preview: Box<dyn Preview>) -> Self {
        Self {
            metadata: Metadata::new(label),
            preview,
        }
    }

    /// Add a description to the preview.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.metadata = self.metadata.description(description);
        self
    }

    /// Add a group to the preview.
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.metadata = self.metadata.group(group);
        self
    }

    /// Add tags to the preview.
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.metadata = self
            .metadata
            .tags(tags.into_iter().map(Into::into).collect());
        self
    }
}

impl Debug for Descriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Descriptor")
            .field("metadata", &self.metadata)
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
            metadata: stateless.metadata.clone(),
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
            metadata: stateful.metadata.clone(),
            preview: Box::new(stateful),
        }
    }
}

use crate::{Metadata, Preview};

/// A descriptor for a preview component that can be registered.
pub struct Descriptor {
    /// Metadata associated with the preview.
    pub metadata: Metadata,
    /// The function to create the preview instance.
    pub create: fn() -> Box<dyn Preview>,
}

impl Descriptor {
    /// Create a new preview descriptor with just a label.
    pub fn new(label: &'static str, create: fn() -> Box<dyn Preview>) -> Self {
        Self {
            metadata: Metadata::new(label),
            create,
        }
    }

    /// Add a description to the preview.
    pub fn with_description(mut self, description: &'static str) -> Self {
        self.metadata = self.metadata.description(description);
        self
    }

    /// Add a group to the preview.
    pub fn with_group(mut self, group: &'static str) -> Self {
        self.metadata = self.metadata.group(group);
        self
    }

    /// Add tags to the preview.
    pub fn with_tags(mut self, tags: &'static [&'static str]) -> Self {
        self.metadata = self.metadata.tags(tags);
        self
    }
}

inventory::collect!(Descriptor);

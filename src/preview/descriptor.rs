use std::fmt::Debug;

use crate::{Metadata, Preview};

/// A descriptor for a preview component that can be registered.
pub struct Descriptor {
    pub preview: Box<dyn Preview>,
}

impl Descriptor {
    /// Create a new [`Descriptor`] with the given label and preview.
    pub fn new(preview: impl Preview + 'static) -> Self {
        Self {
            preview: Box::new(preview),
        }
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

impl<P> From<P> for Descriptor
where
    P: Preview + 'static,
{
    fn from(preview: P) -> Self {
        Descriptor::new(preview)
    }
}

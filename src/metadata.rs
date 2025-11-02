/// Metadata associated with a preview.
#[derive(Debug, Clone, Copy, Default)]
pub struct Metadata {
    /// A label displaying the name of the preview.
    pub label: &'static str,
    /// An optional description of the preview.
    pub description: Option<&'static str>,
    /// A way to categorize related previews together in the UI.
    pub group: Option<&'static str>,
    /// Tags associated with the preview for filtering.
    pub tags: &'static [&'static str],
}

impl Metadata {
    /// Creates a [`Metadata`] instance with the given `label` and default values for other fields.
    pub const fn new(label: &'static str) -> Self {
        Self {
            label,
            description: None,
            group: None,
            tags: &[],
        }
    }

    /// Sets the description for the metadata.
    pub const fn description(mut self, description: &'static str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the group for the metadata.
    pub const fn group(mut self, group: &'static str) -> Self {
        self.group = Some(group);
        self
    }

    /// Sets the tags for the metadata.
    pub const fn tags(mut self, tags: &'static [&'static str]) -> Self {
        self.tags = tags;
        self
    }
}

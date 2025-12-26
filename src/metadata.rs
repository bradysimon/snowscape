/// Metadata associated with a preview.
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    /// A label displaying the name of the preview.
    pub label: String,
    /// An optional description of the preview.
    pub description: Option<String>,
    /// A way to categorize related previews together in the UI.
    pub group: Option<String>,
    /// Tags associated with the preview for filtering.
    pub tags: Vec<String>,
}

impl Metadata {
    /// Creates a [`Metadata`] instance with the given `label` and default values for other fields.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            group: None,
            tags: Vec::new(),
        }
    }

    /// Sets the description for the metadata.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the group for the metadata.
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Sets the tags for the metadata.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Checks if the metadata matches the given search `query`.
    /// Assumes the `query` is already in lowercase.
    pub(crate) fn matches(&self, query: &str) -> bool {
        if self.label.to_lowercase().contains(query) {
            return true;
        }

        if let Some(description) = &self.description
            && description.to_lowercase().contains(query)
        {
            return true;
        }

        if let Some(group) = &self.group
            && group.to_lowercase().contains(query)
        {
            return true;
        }

        for tag in &self.tags {
            if tag.to_lowercase().contains(query) {
                return true;
            }
        }

        false
    }
}

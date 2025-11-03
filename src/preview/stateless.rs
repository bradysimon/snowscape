use crate::{Metadata, Preview};
use iced::{Element, Task};

/// A stateless preview that renders a view function.
pub struct Stateless<F, Message>
where
    F: Fn() -> Element<'static, Message>,
{
    view_fn: F,
    pub(crate) metadata: Metadata,
}

impl<F, Message> Stateless<F, Message>
where
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    pub const fn new(view_fn: F, metadata: Metadata) -> Self {
        Self { view_fn, metadata }
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

impl<F, Message> Preview for Stateless<F, Message>
where
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    fn update(&mut self, _message: crate::Message) -> Task<crate::Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, crate::Message> {
        (self.view_fn)().map(|_| crate::Message::Noop)
    }
}

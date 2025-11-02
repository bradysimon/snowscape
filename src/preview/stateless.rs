use crate::{Message, Metadata, Preview, preview::Descriptor};
use iced::{Element, Task};

/// A stateless preview that renders a view function.
pub struct StatelessPreview<F>
where
    F: Fn() -> Element<'static, Message> + Send + Sync + 'static,
{
    view_fn: F,
}

impl<F> StatelessPreview<F>
where
    F: Fn() -> Element<'static, Message> + Send + Sync + 'static,
{
    pub const fn new(view_fn: F) -> Self {
        Self { view_fn }
    }
}

impl<F> Preview for StatelessPreview<F>
where
    F: Fn() -> Element<'static, Message> + Send + Sync + 'static,
{
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        (self.view_fn)()
    }
}

/// Builder for stateless previews with metadata support.
/// Use this with the `snowscape::preview!` macro for full autocomplete support.
pub struct StatelessBuilder<Message>
where
    Message: 'static,
{
    metadata: Metadata,
    view_fn: fn() -> Element<'static, Message>,
}

impl<Message> StatelessBuilder<Message>
where
    Message: 'static,
{
    /// Add a description to the preview.
    pub const fn description(mut self, description: &'static str) -> Self {
        self.metadata = self.metadata.description(description);
        self
    }

    /// Add a group to the preview.
    pub const fn group(mut self, group: &'static str) -> Self {
        self.metadata = self.metadata.group(group);
        self
    }

    /// Add tags to the preview.
    pub const fn tags(mut self, tags: &'static [&'static str]) -> Self {
        self.metadata = self.metadata.tags(tags);
        self
    }
}

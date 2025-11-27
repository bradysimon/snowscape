use crate::{Metadata, Preview, message::AnyMessage, preview::History};
use iced::{Element, Task};

/// A stateless preview that renders a view function.
pub struct Stateless<F, Message>
where
    Message: AnyMessage,
    F: Fn() -> Element<'static, Message>,
{
    view_fn: F,
    /// The history of messages emitted by the preview.
    history: History<Message>,
    pub(crate) metadata: Metadata,
}

impl<F, Message> Stateless<F, Message>
where
    Message: AnyMessage,
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    pub fn new(view_fn: F, metadata: Metadata) -> Self {
        Self {
            view_fn,
            history: History::new(),
            metadata,
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

impl<F, Message> Preview for Stateless<F, Message>
where
    Message: AnyMessage,
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    fn metadata(&self) -> &crate::Metadata {
        &self.metadata
    }

    fn update(&mut self, message: crate::Message) -> Task<crate::Message> {
        match message {
            crate::Message::Component(boxed) => {
                if let Some(message) = boxed.as_any().downcast_ref::<Message>() {
                    self.history.push(message.clone());
                }
            }
            crate::app::Message::ResetPreview => {
                self.history = History::new();
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, crate::Message> {
        (self.view_fn)().map(|message| crate::Message::Component(Box::new(message)))
    }

    fn message_count(&self) -> usize {
        self.history.len()
    }

    fn visible_messages(&self) -> &'_ [String] {
        self.history.traces()
    }
}

pub fn stateless<F, Message>(label: impl Into<String>, view_fn: F) -> Stateless<F, Message>
where
    Message: AnyMessage,
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    let metadata = crate::Metadata::new(label);
    Stateless::new(view_fn, metadata)
}

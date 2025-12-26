use crate::{
    Metadata, Preview,
    message::AnyMessage,
    preview::{History, Performance},
};
use iced::{Element, Task};

/// A stateless preview that renders a view function.
///
/// The preview can optionally store owned data that the view function can borrow from.
/// Use [`stateless`] for simple previews without external data, or [`stateless_with`]
/// when you need to provide data for the view function to borrow.
pub struct Stateless<Data, F, Message>
where
    Message: AnyMessage,
    Data: Send + 'static,
    F: Fn(&Data) -> Element<'_, Message>,
{
    /// The owned data that the view function can borrow from.
    data: Data,
    /// The view function that renders the preview.
    view_fn: F,
    /// The history of messages emitted by the preview.
    history: History<Message>,
    /// Performance metrics for tracking view function execution times.
    performance: Performance,
    /// Metadata about the this preview.
    pub(crate) metadata: Metadata,
}

impl<Data, F, Message> Stateless<Data, F, Message>
where
    Message: AnyMessage,
    Data: Send + 'static,
    F: Fn(&Data) -> Element<'_, Message> + Send + 'static,
{
    pub fn new(data: Data, view_fn: F, metadata: Metadata) -> Self {
        Self {
            data,
            view_fn,
            history: History::new(),
            performance: Performance::new(),
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

impl<Data, F, Message> Preview for Stateless<Data, F, Message>
where
    Message: AnyMessage,
    Data: Send + 'static,
    F: Fn(&Data) -> Element<'_, Message> + Send + 'static,
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
                self.performance.reset();
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, crate::Message> {
        self.performance
            .record_view(|| (self.view_fn)(&self.data).map(crate::Message::component))
    }

    fn message_count(&self) -> usize {
        self.history.len()
    }

    fn visible_messages(&self) -> &'_ [String] {
        self.history.traces()
    }

    fn performance(&self) -> Option<&Performance> {
        Some(&self.performance)
    }
}

/// Create a new stateless preview with a simple view function.
///
/// This is the simplest way to create a preview when you don't need to provide
/// external data for the view to borrow.
///
/// # Example
///
/// ```
/// use snowscape::preview::stateless;
/// use iced::{Element, widget::text};
///
/// fn previews() -> iced::Result {
///     snowscape::run(|app| {
///         app.preview(stateless("My Preview", || -> Element<'static, ()> {
///             text("Hello, world!").into()
///         }))
///     })
/// }
/// ```
pub fn stateless<F, Message>(
    label: impl Into<String>,
    view_fn: F,
) -> Stateless<(), impl Fn(&()) -> Element<'_, Message> + Send + 'static, Message>
where
    Message: AnyMessage,
    F: Fn() -> Element<'static, Message> + Send + 'static,
{
    let metadata = crate::Metadata::new(label);
    Stateless::new((), move |_| view_fn(), metadata)
}

/// Create a new stateless preview with owned data that the view function can borrow from.
///
/// This is useful when you need to create data inline (like a list of items) that your view
/// function can borrow from that data, rather than requiring `'static` data.
///
/// # Example
///
/// ```
/// use snowscape::preview::stateless_with;
///
/// fn previews() -> iced::Result {
///     snowscape::run(|app| {
///         app.preview(stateless_with(
///             "Label",
///             vec![String::from("Hello"), String::from("World")],
///             |items| list(items)
///         ))
///     })
/// }
///
/// fn list(items: &[String]) -> iced::Element<'_, ()> {
///     use iced::widget::{column, text};
///     column(items.iter().map(|label| text(label).into())).into()
/// }
/// ```
pub fn stateless_with<Data, F, Message>(
    label: impl Into<String>,
    data: Data,
    view_fn: F,
) -> Stateless<Data, F, Message>
where
    Message: AnyMessage,
    Data: Send + 'static,
    F: Fn(&Data) -> Element<'_, Message> + Send + 'static,
{
    let metadata = crate::Metadata::new(label);
    Stateless::new(data, view_fn, metadata)
}

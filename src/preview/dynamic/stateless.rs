use iced::{Element, Task};

use crate::{
    dynamic::{ExtractParams, Param},
    message::AnyMessage,
    metadata::Metadata,
    preview::{History, Performance, Preview},
};

/// A dynamic stateless preview that renders an element based on adjustable parameters.
pub struct Stateless<Data, Params, F, Message>
where
    Data: Send + 'static,
    Params: ExtractParams,
    F: for<'a> Fn(&'a Data, &'a Params::Values) -> Element<'a, Message> + Send,
    Message: AnyMessage,
{
    /// Metadata about the preview.
    metadata: Metadata,
    /// The history of messages emitted by the preview.
    history: History<Message>,
    /// Performance metrics for tracking view function execution times.
    performance: Performance,
    /// The owned data that the view function can borrow from.
    data: Data,
    /// The dynamic parameters the user can adjust.
    params: Params,
    /// The default parameters for resetting.
    default_params: Params,
    /// A cached list of params generated from `params` for displaying in the UI.
    cached_params: Vec<Param>,
    /// The cached extracted parameter values.
    cached_values: Params::Values,
    /// The view function that generates the preview from parameters.
    view_fn: F,
}

impl<Data, Params, F, Message> Stateless<Data, Params, F, Message>
where
    Data: Send + 'static,
    Params: ExtractParams,
    F: for<'a> Fn(&'a Data, &'a Params::Values) -> Element<'a, Message> + Send,
    Message: AnyMessage,
{
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

impl<Data, Params, F, Message> Preview for Stateless<Data, Params, F, Message>
where
    Data: Send + 'static,
    Params: ExtractParams,
    F: for<'a> Fn(&'a Data, &'a Params::Values) -> Element<'a, Message> + Send,
    Message: AnyMessage,
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
            crate::Message::ChangeParam(index, param) => {
                self.params.update_index(index, param);
                self.cached_params = self.params.to_params();
                self.cached_values = self.params.extract();
            }
            crate::Message::ResetParams => {
                self.params = self.default_params.clone();
                self.cached_params = self.params.to_params();
                self.cached_values = self.params.extract();
            }
            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, crate::Message> {
        self.performance.record_view(|| {
            (self.view_fn)(&self.data, &self.cached_values).map(crate::Message::component)
        })
    }

    fn message_count(&self) -> usize {
        self.history.len()
    }

    fn visible_messages(&self) -> &'_ [String] {
        self.history.traces()
    }

    fn timeline(&self) -> Option<crate::preview::Timeline> {
        None
    }

    fn params(&self) -> &[Param] {
        &self.cached_params
    }

    fn performance(&self) -> Option<&Performance> {
        Some(&self.performance)
    }
}

/// Create a new dynamic stateless preview with the given label, parameters, and view function.
///
/// This is a convenience wrapper around [`stateless_with`] that doesn't require external data.
pub fn stateless<Params, F, Message>(
    label: impl Into<String>,
    params: Params,
    view_fn: F,
) -> Stateless<
    (),
    Params,
    impl for<'a> Fn(&'a (), &'a Params::Values) -> Element<'a, Message> + Send,
    Message,
>
where
    Params: ExtractParams,
    F: for<'a> Fn(&'a Params::Values) -> Element<'a, Message> + Send + 'static,
    Message: AnyMessage,
{
    stateless_with(label, (), params, move |_data, params| view_fn(params))
}

/// Create a new dynamic stateless preview with static data, parameters, and a view function.
///
/// This allows you to pass owned data that the view function can borrow from,
/// along with dynamic parameters that can be adjusted at runtime.
///
/// # Example
///
/// ```
/// use snowscape::dynamic;
/// use iced::{Element, widget::text};
///
/// struct Config {
///     prefix: String,
/// }
///
/// fn config_view<'a>(config: &'a Config, name: &'a str) -> Element<'a, ()> {
///     text!("{}, {}!", config.prefix, name).into()
/// }
///
/// fn previews() -> iced::Result {
///     snowscape::run(|app| {
///         app.preview(dynamic::stateless_with(
///             "Greeting",
///             Config { prefix: "Hello".to_string() },
///             dynamic::text("Name", "World"),
///             |config, name| config_view(config, name),
///         ))
///     })
/// }
/// ```
pub fn stateless_with<Data, Params, F, Message>(
    label: impl Into<String>,
    data: Data,
    params: Params,
    view_fn: F,
) -> Stateless<Data, Params, F, Message>
where
    Data: Send + 'static,
    Params: ExtractParams,
    F: for<'a> Fn(&'a Data, &'a Params::Values) -> Element<'a, Message> + Send,
    Message: AnyMessage,
{
    let metadata = crate::Metadata::new(label);
    let cached_params = params.to_params();
    let cached_values = params.extract();
    Stateless {
        metadata,
        data,
        params: params.clone(),
        default_params: params,
        history: History::new(),
        performance: Performance::new(),
        cached_params,
        cached_values,
        view_fn,
    }
}

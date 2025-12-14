use iced::{Element, Task};

use crate::{
    dynamic::{ExtractParams, Param},
    message::AnyMessage,
    metadata::Metadata,
    preview::{History, Preview},
};

/// A dynamic stateless preview that renders an element based on adjustable parameters.
pub struct Stateless<Params, F, Message>
where
    Params: ExtractParams,
    F: Fn(&Params::Values) -> Element<'_, Message> + Send,
    Message: AnyMessage,
{
    /// Metadata about the preview.
    metadata: Metadata,
    /// The history of messages emitted by the preview.
    history: History<Message>,
    /// The dynamic parameters the user can adjust.
    params: Params,
    /// A cached list of params generated from `params` for displaying in the UI.
    cached_params: Vec<Param>,
    /// The cached extracted parameter values.
    cached_values: Params::Values,
    /// The view function that generates the preview from parameters.
    view_fn: F,
}

impl<Params, F, Message> Stateless<Params, F, Message>
where
    Params: ExtractParams,
    F: Fn(&Params::Values) -> Element<'_, Message> + Send,
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

impl<Params, F, Message> Preview for Stateless<Params, F, Message>
where
    Params: ExtractParams,
    F: Fn(&Params::Values) -> Element<'_, Message> + Send,
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
            }
            crate::Message::ChangeParam(index, param) => {
                self.params.update_index(index, param);
                self.cached_params = self.params.to_params();
                self.cached_values = self.params.extract();
            }
            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, crate::Message> {
        (self.view_fn)(&self.cached_values).map(crate::Message::component)
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
}

/// Create a new dynamic stateless preview with the given label, parameters, and view function.
pub fn stateless<Params, F, Message>(
    label: impl Into<String>,
    params: Params,
    view_fn: F,
) -> Stateless<Params, F, Message>
where
    Params: ExtractParams,
    F: Fn(&Params::Values) -> Element<'_, Message> + Send,
    Message: AnyMessage,
{
    let metadata = crate::Metadata::new(label);
    let cached_params = params.to_params();
    let cached_values = params.extract();
    Stateless {
        metadata,
        params,
        history: History::new(),
        cached_params,
        cached_values,
        view_fn,
    }
}

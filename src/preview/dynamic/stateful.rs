use iced::{Element, Task};

use crate::{
    dynamic::{ExtractParams, Param},
    message::AnyMessage,
    metadata::Metadata,
    preview::{History, Preview, Timeline},
};

/// A dynamic stateful preview with full update/view cycle and adjustable parameters.
pub struct Stateful<Boot, Params, State, Message, IntoTask>
where
    Boot: Fn() -> State,
    Params: ExtractParams,
    State: Send,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    /// Metadata about the preview.
    metadata: Metadata,
    /// The dynamic parameters the user can adjust.
    params: Params,
    /// A cached list of params generated from `params` for displaying in the UI.
    cached_params: Vec<Param>,
    /// The cached extracted parameter values.
    cached_values: Params::Values,
    /// The boot function that initializes state from parameters.
    boot: Boot,
    /// The current state of the preview.
    state: State,
    /// The history of messages emitted by the preview.
    history: History<Message>,
    /// The update function that processes messages.
    update_fn: fn(&mut State, Message) -> IntoTask,
    /// The view function that renders the preview.
    view_fn: for<'a> fn(&'a State, &'a Params::Values) -> Element<'a, Message>,
}

impl<Boot, Params, State, Message, IntoTask> Stateful<Boot, Params, State, Message, IntoTask>
where
    Boot: Fn() -> State + Send,
    Params: ExtractParams,
    State: Send,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    pub fn new(
        params: Params,
        boot: Boot,
        update_fn: fn(&mut State, Message) -> IntoTask,
        view_fn: for<'a> fn(&'a State, &'a Params::Values) -> Element<'a, Message>,
        metadata: Metadata,
    ) -> Self {
        let cached_params = params.to_params();
        let cached_values = params.extract();
        let state = boot();
        Self {
            metadata,
            params,
            cached_params,
            cached_values,
            boot,
            state,
            history: History::new(),
            update_fn,
            view_fn,
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

impl<Boot, Params, State, Message, IntoTask> Preview
    for Stateful<Boot, Params, State, Message, IntoTask>
where
    Boot: Fn() -> State + Send,
    Params: ExtractParams,
    State: Send,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    fn metadata(&self) -> &crate::Metadata {
        &self.metadata
    }

    fn update(&mut self, message: crate::Message) -> Task<crate::Message> {
        match message {
            crate::Message::Component(boxed) => {
                // Ignore incoming messages if we're in the past.
                if !self.history.is_live() {
                    return Task::none();
                }

                let Some(message) = boxed.as_any().downcast_ref::<Message>() else {
                    return Task::none();
                };

                self.history.push(message.clone());
                let message = message.clone();
                let result = (self.update_fn)(&mut self.state, message);
                let task: Task<Message> = result.into();

                // Map the task's messages back to the preview's crate::Message type
                task.map(|message| crate::Message::Component(Box::new(message)))
            }
            crate::Message::ResetPreview => {
                // Reset state with current parameter values
                self.state = (self.boot)();
                self.history.reset();
                Task::none()
            }
            crate::Message::TimeTravel(index) => {
                self.history.change_position(index as usize);
                self.state = (self.boot)();
                self.history
                    .messages
                    .iter()
                    .take(self.history.position)
                    .for_each(|message| _ = (self.update_fn)(&mut self.state, message.clone()));
                Task::none()
            }
            crate::Message::JumpToPresent => {
                if self.history.is_live() {
                    return Task::none();
                }

                let position = self.history.position;
                self.history.go_live();
                self.history
                    .messages
                    .iter()
                    .skip(position.saturating_sub(0))
                    .for_each(|message| _ = (self.update_fn)(&mut self.state, message.clone()));
                Task::none()
            }
            crate::Message::ChangeParam(index, param) => {
                // Update parameters and reset state with new values
                self.params.update_index(index, param);
                self.cached_params = self.params.to_params();
                self.cached_values = self.params.extract();
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, crate::Message> {
        (self.view_fn)(&self.state, &self.cached_values).map(crate::Message::component)
    }

    fn message_count(&self) -> usize {
        self.history.len()
    }

    fn visible_messages(&self) -> &'_ [String] {
        self.history.visible_traces()
    }

    fn timeline(&self) -> Option<Timeline> {
        Some(self.history.timeline())
    }

    fn params(&self) -> &[Param] {
        &self.cached_params
    }
}

/// Create a new dynamic stateful preview, allowing users to adjust parameters
/// that affect the view at runtime.
pub fn stateful<Boot, Params, State, Message, IntoTask>(
    label: impl Into<String>,
    params: Params,
    boot: Boot,
    update_fn: fn(&mut State, Message) -> IntoTask,
    view_fn: for<'a> fn(&'a State, &'a Params::Values) -> Element<'a, Message>,
) -> Stateful<Boot, Params, State, Message, IntoTask>
where
    Params: ExtractParams,
    Boot: Fn() -> State + Send,
    State: Send,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    let metadata = crate::Metadata::new(label);
    Stateful::new(params, boot, update_fn, view_fn, metadata)
}

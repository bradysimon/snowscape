use crate::{Metadata, Preview, message::AnyMessage, preview::History};
use iced::{Element, Task};

/// A stateful preview with full update/view cycle.
pub struct Stateful<Boot, State, Message, IntoTask>
where
    Boot: Fn() -> State,
    State: Send + 'static,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    boot: Boot,
    state: State,
    /// The history of messages emitted by the preview.
    history: History<Message>,
    update_fn: fn(&mut State, Message) -> IntoTask,
    view_fn: fn(&State) -> Element<'_, Message>,
    pub(crate) metadata: Metadata,
}

impl<Boot, State, Message, IntoTask> Stateful<Boot, State, Message, IntoTask>
where
    Boot: Fn() -> State + Send,
    State: Send + 'static,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    pub fn new(
        boot: Boot,
        update_fn: fn(&mut State, Message) -> IntoTask,
        view_fn: fn(&State) -> Element<'_, Message>,
        metadata: Metadata,
    ) -> Self {
        let state = boot();
        Self {
            boot,
            state,
            history: History::new(),
            update_fn,
            view_fn,
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

impl<Boot, State, Message, IntoTask> Preview for Stateful<Boot, State, Message, IntoTask>
where
    Boot: Fn() -> State + Send,
    State: Send + 'static,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    fn update(&mut self, message: crate::Message) -> Task<crate::Message> {
        // Try to downcast the message to the component's message type
        if let crate::Message::Component(boxed) = message {
            if let Some(message) = boxed.as_any().downcast_ref::<Message>() {
                self.history.push(message.clone());
                let message = message.clone();
                let result = (self.update_fn)(&mut self.state, message);
                let task: Task<Message> = result.into();

                // Map the task's messages back to the preview's crate::Message type
                return task.map(|message| crate::Message::Component(Box::new(message)));
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, crate::Message> {
        (self.view_fn)(&self.state).map(|message| crate::Message::Component(Box::new(message)))
    }

    fn history(&self) -> Option<&'_ [String]> {
        Some(self.history.traces())
    }
}

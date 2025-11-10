use crate::{Message, Metadata, Preview, message::AnyMessage};
use iced::{Element, Task};

/// A stateful preview with full update/view cycle.
pub struct Stateful<Boot, State, Msg, IntoTask>
where
    Boot: Fn() -> State,
    State: Send + 'static,
    Msg: AnyMessage,
    IntoTask: Into<Task<Msg>>,
{
    boot: Boot,
    state: State,
    /// The history of messages emitted by the preview.
    history: Vec<Msg>,
    update_fn: fn(&mut State, Msg) -> IntoTask,
    view_fn: fn(&State) -> Element<'_, Msg>,
    pub(crate) metadata: Metadata,
}

impl<Boot, State, Msg, IntoTask> Stateful<Boot, State, Msg, IntoTask>
where
    Boot: Fn() -> State + Send,
    State: Send + 'static,
    Msg: AnyMessage,
    IntoTask: Into<Task<Msg>>,
{
    pub fn new(
        boot: Boot,
        update_fn: fn(&mut State, Msg) -> IntoTask,
        view_fn: fn(&State) -> Element<'_, Msg>,
        metadata: Metadata,
    ) -> Self {
        let state = boot();
        Self {
            boot,
            state,
            history: Vec::new(),
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

impl<Boot, State, Msg, IntoTask> Preview for Stateful<Boot, State, Msg, IntoTask>
where
    Boot: Fn() -> State + Send,
    State: Send + 'static,
    Msg: AnyMessage,
    IntoTask: Into<Task<Msg>>,
{
    fn update(&mut self, message: Message) -> Task<Message> {
        // Try to downcast the message to the component's message type
        if let Message::Component(boxed_msg) = message {
            if let Some(component_msg) = boxed_msg.as_any().downcast_ref::<Msg>() {
                self.history.push(component_msg.clone());
                let component_msg = component_msg.clone();
                let result = (self.update_fn)(&mut self.state, component_msg);
                let task: Task<Msg> = result.into();

                // Map the task's messages back to the preview's Message type
                return task.map(|msg| Message::Component(Box::new(msg)));
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        (self.view_fn)(&self.state).map(|msg| Message::Component(Box::new(msg)))
    }

    fn history(&self) -> Option<Vec<String>> {
        Some(
            self.history
                .iter()
                .map(|message| format!("{message:?}"))
                .collect(),
        )
    }
}

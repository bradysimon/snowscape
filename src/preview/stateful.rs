use crate::{
    Metadata, Preview,
    message::AnyMessage,
    preview::{History, Performance, Timeline},
};
use iced::{Element, Task};

/// A stateful preview with full update/view cycle.
pub struct Stateful<Boot, State, Message, IntoTask>
where
    Boot: Fn() -> State,
    State: Send,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    boot: Boot,
    state: State,
    /// The history of messages emitted by the preview.
    history: History<Message>,
    /// Performance metrics for tracking view/update function execution times.
    performance: Performance,
    update_fn: fn(&mut State, Message) -> IntoTask,
    view_fn: fn(&State) -> Element<'_, Message>,
    pub(crate) metadata: Metadata,
}

impl<Boot, State, Message, IntoTask> Stateful<Boot, State, Message, IntoTask>
where
    Boot: Fn() -> State + Send,
    State: Send,
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
            performance: Performance::default(),
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
                // Track performance only when live (not during time travel replay)
                let result = self
                    .performance
                    .record_update(|| (self.update_fn)(&mut self.state, message));
                let task: Task<Message> = result.into();

                // Map the task's messages back to the preview's crate::Message type
                task.map(|message| crate::Message::Component(Box::new(message)))
            }
            crate::Message::ResetPreview => {
                self.state = (self.boot)();
                self.history.reset();
                self.performance.reset();
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
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, crate::Message> {
        self.performance
            .record_view(|| (self.view_fn)(&self.state).map(crate::Message::component))
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

    fn performance(&self) -> Option<&Performance> {
        Some(&self.performance)
    }
}

pub fn stateful<Boot, State, Message, IntoTask>(
    label: impl Into<String>,
    boot: Boot,
    update_fn: fn(&mut State, Message) -> IntoTask,
    view_fn: fn(&State) -> Element<'_, Message>,
) -> Stateful<Boot, State, Message, IntoTask>
where
    Boot: Fn() -> State + Send,
    State: Send,
    Message: AnyMessage,
    IntoTask: Into<Task<Message>>,
{
    let metadata = crate::Metadata::new(label);
    Stateful::new(boot, update_fn, view_fn, metadata)
}

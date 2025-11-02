use crate::{Message, Preview};
use iced::{Element, Task};

/// A stateful preview with full update/view cycle.
pub struct StatefulPreview<State, Msg, IntoTask>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + 'static,
    IntoTask: Into<Task<Msg>>,
{
    state: State,
    update_fn: fn(&mut State, Msg) -> IntoTask,
    view_fn: fn(&State) -> Element<'_, Msg>,
}

impl<State, Msg, IntoTask> StatefulPreview<State, Msg, IntoTask>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
    IntoTask: Into<Task<Msg>>,
{
    pub fn new(
        state: State,
        update_fn: fn(&mut State, Msg) -> IntoTask,
        view_fn: fn(&State) -> Element<'_, Msg>,
    ) -> Self {
        Self {
            state,
            update_fn,
            view_fn,
        }
    }
}

impl<State, Msg, UpdateRet> Preview for StatefulPreview<State, Msg, UpdateRet>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
    UpdateRet: Into<Task<Msg>>,
{
    fn update(&mut self, message: Message) -> Task<Message> {
        // Try to downcast the message to the component's message type
        if let Message::Component(boxed_msg) = message {
            if let Some(component_msg) = boxed_msg.as_any().downcast_ref::<Msg>() {
                // Call the update function with the component's message
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
        // Render the view and map messages to wrap them in Message::Component
        (self.view_fn)(&self.state).map(|msg| Message::Component(Box::new(msg)))
    }
}

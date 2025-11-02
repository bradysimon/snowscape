use crate::{Message, Preview};
use iced::{Element, Task};

/// A stateful preview with full update/view cycle.
pub struct StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + 'static,
{
    state: State,
    update_fn: fn(&mut State, Msg) -> Task<Msg>,
    view_fn: fn(&State) -> Element<'_, Msg>,
}

impl<State, Msg> StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
{
    pub fn new(
        state: State,
        update_fn: fn(&mut State, Msg) -> Task<Msg>,
        view_fn: fn(&State) -> Element<'_, Msg>,
    ) -> Self {
        Self {
            state,
            update_fn,
            view_fn,
        }
    }
}

impl<State, Msg> Preview for StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
{
    fn update(&mut self, message: Message) -> Task<Message> {
        // Try to downcast the message to the component's message type
        if let Message::Component(boxed_msg) = message {
            if let Some(component_msg) = boxed_msg.as_any().downcast_ref::<Msg>() {
                // Call the update function with the component's message
                let component_msg = component_msg.clone();
                let task = (self.update_fn)(&mut self.state, component_msg);

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

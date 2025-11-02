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
    fn update(&mut self, _message: Message) -> Task<Message> {
        // For now, stateful previews don't handle messages from the UI
        // This would require more complex message routing
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        (self.view_fn)(&self.state).map(|_msg| Message::Noop)
    }
}

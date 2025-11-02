use crate::{Message, Preview};
use iced::{Element, Task};

/// A stateless preview that renders a view function.
pub struct StatelessPreview<F>
where
    F: Fn() -> Element<'static, Message> + Send + Sync + 'static,
{
    view_fn: F,
}

impl<F> StatelessPreview<F>
where
    F: Fn() -> Element<'static, Message> + Send + Sync + 'static,
{
    pub const fn new(view_fn: F) -> Self {
        Self { view_fn }
    }
}

impl<F> Preview for StatelessPreview<F>
where
    F: Fn() -> Element<'static, Message> + Send + Sync + 'static,
{
    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        (self.view_fn)()
    }
}

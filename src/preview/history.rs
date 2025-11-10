use crate::message::AnyMessage;

/// A history of messages emitted by a preview.
#[derive(Debug, Clone, Default)]
pub struct History<Message>
where
    Message: AnyMessage,
{
    /// The messages emitted by the preview.
    pub messages: Vec<Message>,
    /// Debug representations of the emitted `messages`.
    /// Stored as a separate Vec to avoid constant allocations.
    pub debug: Vec<String>,
}

impl<Message> History<Message>
where
    Message: AnyMessage,
{
    /// Creates a new, empty [`History`].
    pub const fn new() -> Self {
        Self {
            messages: Vec::new(),
            debug: Vec::new(),
        }
    }

    /// Pushes a new `message` to the history.
    pub fn push(&mut self, message: Message) {
        self.debug.push(format!("{message:?}"));
        self.messages.push(message);
    }

    /// Returns a reference to the debug representations of the messages in the history.
    pub fn debug(&self) -> &[String] {
        &self.debug
    }
}

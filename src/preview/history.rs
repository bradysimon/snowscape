use crate::message::AnyMessage;

/// A history of messages emitted by a preview.
#[derive(Debug, Clone, Default)]
pub struct History<Message>
where
    Message: AnyMessage,
{
    /// The messages emitted by the preview.
    pub messages: Vec<Message>,
    /// Message traces of the emitted `messages`.
    /// Stored as a separate `Vec` to avoid constant string allocations.
    pub traces: Vec<String>,
}

impl<Message> History<Message>
where
    Message: AnyMessage,
{
    /// Creates a new, empty [`History`].
    pub const fn new() -> Self {
        Self {
            messages: Vec::new(),
            traces: Vec::new(),
        }
    }

    /// Pushes a new `message` to the history.
    pub fn push(&mut self, message: Message) {
        self.traces.push(format!("{message:?}"));
        self.messages.push(message);
    }

    /// Returns a reference to the message traces in the history.
    pub fn traces(&self) -> &[String] {
        &self.traces
    }

    /// Returns the number of messages in the history.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Returns whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

use crate::{message::AnyMessage, preview::Timeline};

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
    /// The index of the current message in the timeline.
    pub position: usize,
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
            position: 0,
        }
    }

    /// Pushes a new `message` to the history.
    pub fn push(&mut self, message: Message) {
        // If the timeline is live, update the position to stay live.
        if self.is_live() {
            self.position += 1;
        }

        self.traces.push(format!("{message:?}"));
        self.messages.push(message);
    }

    /// Resets the history, clearing all messages and traces
    /// and setting the position back to zero.
    pub fn reset(&mut self) {
        self.messages.clear();
        self.traces.clear();
        self.position = 0;
    }

    /// Returns a reference to the message traces in the history.
    pub fn traces(&self) -> &[String] {
        &self.traces
    }

    /// Returns a reference to the visible message traces in the history, i.e.
    /// those up to the current position.
    pub fn visible_traces(&self) -> &[String] {
        &self.traces[..self.position]
    }

    /// Returns the number of messages in the history.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Returns whether the history is empty.
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Whether the timeline is live, i.e. the user is at the end of the timeline
    /// and seeing the most recent state.
    pub fn is_live(&self) -> bool {
        self.position == self.messages.len()
    }

    /// Jumps the timeline forward to the latest state.
    pub fn go_live(&mut self) {
        self.position = self.messages.len();
    }

    /// Changes the current position in the timeline to the given `position`
    /// if it is within the valid range.
    pub fn change_position(&mut self, position: usize) {
        if position > self.messages.len() {
            return;
        }
        self.position = position;
    }

    /// Returns the current timeline of the history.
    pub fn timeline(&self) -> Timeline {
        Timeline::new(self.position as u32, self.messages.len() as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pushing messages while live keeps the timeline live.
    #[test]
    fn push_stays_live() {
        let mut history = History::new();
        assert!(history.is_live());

        history.push(1);
        assert_eq!(history.position, 1);
        assert!(history.is_live());

        history.push(2);
        assert_eq!(history.position, 2);
        assert!(history.is_live());
    }

    #[test]
    fn change_position() {
        let mut history = History::new();
        history.push(1);
        history.push(2);
        history.push(3);

        history.change_position(1);
        assert_eq!(history.position, 1);

        history.change_position(5); // Out of bounds
        assert_eq!(history.position, 1); // Position should not change
    }
}

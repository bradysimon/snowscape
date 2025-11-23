use std::ops::RangeInclusive;

/// A timeline of previous messages for stateful previews,
/// including the current position and valid range of messages.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Timeline {
    /// The index of the current message in the timeline.
    position: u32,
    /// The number of messages in the timeline.
    count: u32,
}

impl Timeline {
    /// Creates a new [`Timeline`] with the given `position` and `range`,
    /// clamping the `position` to be within the `range`.
    pub fn new(position: u32, count: u32) -> Self {
        Self {
            position: position.min(count),
            count,
        }
    }

    /// Returns the current position in the timeline.
    pub fn position(&self) -> u32 {
        self.position
    }

    /// Returns the range of valid message indices in the timeline,
    /// which can be useful for slider widgets.
    pub fn range(&self) -> RangeInclusive<u32> {
        0..=self.count
    }

    /// Whether the timeline is empty, i.e. has no messages.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Whether the timeline is live, i.e. the user is at the end of the timeline
    /// and seeing the most recent state.
    pub fn is_live(&self) -> bool {
        self.position == self.count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_position() {
        let timeline = Timeline::new(10, 5);
        assert_eq!(timeline.position(), 5);

        let timeline = Timeline::new(2, 5);
        assert_eq!(timeline.position(), 2);
    }

    /// The default [`Timeline`] should be considered live.
    #[test]
    fn default_is_live() {
        let timeline = Timeline::default();
        assert!(timeline.is_live());
    }

    #[test]
    fn is_live() {
        let mut timeline = Timeline::new(5, 5);
        assert!(timeline.is_live());

        timeline.position = 3;
        assert!(!timeline.is_live());
    }
}

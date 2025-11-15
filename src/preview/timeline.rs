use std::ops::RangeInclusive;

/// A timeline of previous messages for stateful previews,
/// including the current position and valid range of messages.
#[derive(Debug, Clone)]
pub struct Timeline {
    /// The index of the current message in the timeline.
    pub position: u32,
    /// The range of valid message indices in the timeline,
    /// where 0 is the initial state.
    pub range: RangeInclusive<u32>,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            position: 0,
            range: 0..=0,
        }
    }
}

impl Timeline {
    /// Updates the timeline to reflect the given number of messages.
    /// This will only update the current `position` if the timeline is live,
    /// i.e. the user isn't scrubbing through previous states.
    pub fn update(&mut self, count: usize) {
        if self.is_live() {
            self.position = count as u32;
        }
        self.range = 0..=count as u32;
    }

    /// Changes the current position in the timeline to the given `position`
    /// if it is within the valid range.
    pub fn change_position(&mut self, position: u32) {
        if self.range.contains(&position) {
            self.position = position;
        }
    }

    /// Whether the timeline is live, i.e. the user is at the end of the timeline
    /// and seeing the most recent state.
    pub fn is_live(&self) -> bool {
        self.position == *self.range.end()
    }

    /// Jumps the timeline forward to the latest state.
    pub fn go_live(&mut self) {
        self.position = *self.range.end();
    }
}

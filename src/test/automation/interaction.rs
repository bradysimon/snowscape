//! Mouse, keyboard, and composite interactions for the [`Emulator`].

use std::time::Duration;

use iced_test::Instruction;
use iced_test::core::{mouse, widget};
use iced_test::instruction::{
    Interaction, Key, Keyboard, Mouse as MouseInteraction, Target as InstrTarget,
};

use super::{Emulator, Result};

impl<P: iced_test::program::Program + 'static> Emulator<P> {
    // MARK: - Mouse interactions

    /// Clicks the widget targeted by `selector` with the left mouse button.
    pub fn click(&mut self, selector: impl IntoSelector) -> Result {
        self.click_button(mouse::Button::Left, selector)
    }

    /// Right-clicks the widget targeted by `selector`.
    pub fn right_click(&mut self, selector: impl IntoSelector) -> Result {
        self.click_button(mouse::Button::Right, selector)
    }

    /// Middle-clicks the widget targeted by `selector`.
    pub fn middle_click(&mut self, selector: impl IntoSelector) -> Result {
        self.click_button(mouse::Button::Middle, selector)
    }

    fn click_button(&mut self, button: mouse::Button, selector: impl IntoSelector) -> Result {
        let target = selector.into_instr_target();
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Click {
                button,
                target: Some(target),
            },
        )))
    }

    /// Moves the mouse cursor to the given target.
    pub fn move_cursor(&mut self, selector: impl IntoSelector) -> Result {
        let target = selector.into_instr_target();
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Move(target),
        )))
    }

    /// Hovers the cursor over the widget targeted by `selector`.
    ///
    /// This is currently equivalent to [`move_cursor`](Self::move_cursor); it
    /// exists as a Playwright-style alias for readability.
    pub fn hover(&mut self, selector: impl IntoSelector) -> Result {
        self.move_cursor(selector)
    }

    /// Presses (without releasing) the given mouse button at the current
    /// cursor position.
    pub fn press_button(&mut self, button: mouse::Button) -> Result {
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Press {
                button,
                target: None,
            },
        )))
    }

    /// Releases the given mouse button at the current cursor position.
    pub fn release_button(&mut self, button: mouse::Button) -> Result {
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Release {
                button,
                target: None,
            },
        )))
    }

    // MARK: - Keyboard interactions

    /// Types the given text by simulating individual key presses.
    pub fn type_text(&mut self, text: impl Into<String>) -> Result {
        self.run_instruction(Instruction::Interact(Interaction::Keyboard(
            Keyboard::Typewrite(text.into()),
        )))
    }

    /// Presses and releases a named key.
    pub fn tap_key(&mut self, key: Key) -> Result {
        self.run_instruction(Instruction::Interact(Interaction::Keyboard(
            Keyboard::Type(key),
        )))
    }

    /// Presses (without releasing) a named key.
    pub fn press_key(&mut self, key: Key) -> Result {
        self.run_instruction(Instruction::Interact(Interaction::Keyboard(
            Keyboard::Press(key),
        )))
    }

    /// Releases a previously pressed named key.
    pub fn release_key(&mut self, key: Key) -> Result {
        self.run_instruction(Instruction::Interact(Interaction::Keyboard(
            Keyboard::Release(key),
        )))
    }

    // MARK: - Composite interactions

    /// Double-clicks the widget targeted by `selector`.
    pub fn double_click(&mut self, selector: impl IntoSelector) -> Result {
        let target = selector.into_instr_target();
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Click {
                button: mouse::Button::Left,
                target: Some(target.clone()),
            },
        )))?;
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Click {
                button: mouse::Button::Left,
                target: Some(target),
            },
        )))
    }

    /// Drags from one target to another by pressing at `from`, moving to
    /// `to`, and releasing.
    pub fn drag(&mut self, from: impl IntoSelector, to: impl IntoSelector) -> Result {
        let from_target = from.into_instr_target();
        let to_target = to.into_instr_target();

        // Move + press at source
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Move(from_target),
        )))?;
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Press {
                button: mouse::Button::Left,
                target: None,
            },
        )))?;
        // Move to destination
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Move(to_target),
        )))?;
        // Release
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Release {
                button: mouse::Button::Left,
                target: None,
            },
        )))
    }

    /// Scrolls the widget targeted by `selector` by the given `delta`.
    ///
    /// Positive `delta_y` scrolls down, negative scrolls up.
    /// The selector must match a scrollable widget with a [`widget::Id`].
    pub fn scroll(&mut self, id: impl Into<widget::Id>, delta_x: f32, delta_y: f32) -> Result {
        use iced_test::core::widget::operation::scrollable;

        let operation = scrollable::scroll_by::<()>(
            id.into(),
            scrollable::AbsoluteOffset {
                x: delta_x,
                y: delta_y,
            },
        );
        let _ = self.run_operation(operation);
        // Scrolling mutates layout state; invalidate the cache so the next
        // query observes the new translation.
        self.cache = iced_test::runtime::user_interface::Cache::default();
        Ok(())
    }

    /// Clicks a text input, clears its existing content, and types `text`.
    ///
    /// This is a common pattern for filling form fields. The current
    /// content is read from the focused [`TextInput`](super::WidgetKind::TextInput)
    /// in the widget tree and erased with `Backspace` key taps before the
    /// new text is typed.
    pub fn fill(&mut self, selector: impl IntoSelector, text: impl Into<String>) -> Result {
        let target = selector.into_instr_target();
        // Click to focus the input.
        self.run_instruction(Instruction::Interact(Interaction::Mouse(
            MouseInteraction::Click {
                button: mouse::Button::Left,
                target: Some(target),
            },
        )))?;
        // Read the focused text input's current contents and clear them.
        let existing_chars = self
            .focused_text_input_content()
            .map(|content| content.chars().count())
            .unwrap_or(0);
        for _ in 0..existing_chars {
            self.run_instruction(Instruction::Interact(Interaction::Keyboard(
                Keyboard::Type(Key::Backspace),
            )))?;
        }
        self.run_instruction(Instruction::Interact(Interaction::Keyboard(
            Keyboard::Typewrite(text.into()),
        )))
    }

    fn focused_text_input_content(&mut self) -> Option<String> {
        fn walk(node: &super::WidgetNode) -> Option<String> {
            if node.kind == super::WidgetKind::TextInput && node.focused {
                return node.text.clone();
            }
            node.children.iter().find_map(walk)
        }
        let tree = self.widget_tree();
        walk(&tree)
    }

    // MARK: - Time and waiting

    /// Sleeps for the given duration and then drains any actions emitted by
    /// background tasks or subscriptions during the sleep.
    pub fn wait(&mut self, duration: Duration) -> Result {
        std::thread::sleep(duration);
        self.drain_pending()
    }

    /// Polls `selector` until it matches or the default timeout elapses.
    pub fn wait_for<S>(&mut self, selector: S) -> Result<S::Output>
    where
        S: iced_test::Selector + Clone + Send,
        S::Output: Clone + Send,
    {
        self.wait_for_with_timeout(selector, self.default_timeout)
    }

    /// Polls `selector` until it matches or `timeout` elapses.
    pub fn wait_for_with_timeout<S>(&mut self, selector: S, timeout: Duration) -> Result<S::Output>
    where
        S: iced_test::Selector + Clone + Send,
        S::Output: Clone + Send,
    {
        let description = selector.clone().description();
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(16);
        loop {
            // Drain any pending actions from background tasks before checking.
            self.drain_pending()?;
            if let Some(output) = self.try_find(selector.clone()) {
                return Ok(output);
            }
            if start.elapsed() >= timeout {
                return Err(super::Error::Timeout {
                    description,
                    elapsed: start.elapsed(),
                });
            }
            std::thread::sleep(poll_interval);
        }
    }

    /// Polls until the given text is visible, using the default timeout.
    pub fn wait_for_text(&mut self, text: impl Into<String>) -> Result {
        self.wait_for_text_with_timeout(text, self.default_timeout)
    }

    /// Polls until the given text is visible or `timeout` elapses.
    pub fn wait_for_text_with_timeout(
        &mut self,
        text: impl Into<String>,
        timeout: Duration,
    ) -> Result {
        let text = text.into();
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(16);
        loop {
            self.drain_pending()?;
            if self.contains_text(&text) {
                return Ok(());
            }
            if start.elapsed() >= timeout {
                return Err(super::Error::Timeout {
                    description: format!("text {text:?} to appear"),
                    elapsed: start.elapsed(),
                });
            }
            std::thread::sleep(poll_interval);
        }
    }

    /// Polls until the selector **stops** matching, using the default timeout.
    pub fn wait_until_gone<S>(&mut self, selector: S) -> Result
    where
        S: iced_test::Selector + Clone + Send,
        S::Output: Clone + Send,
    {
        self.wait_until_gone_with_timeout(selector, self.default_timeout)
    }

    /// Polls until the selector **stops** matching or `timeout` elapses.
    pub fn wait_until_gone_with_timeout<S>(&mut self, selector: S, timeout: Duration) -> Result
    where
        S: iced_test::Selector + Clone + Send,
        S::Output: Clone + Send,
    {
        let description = selector.clone().description();
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(16);
        loop {
            self.drain_pending()?;
            if self.try_find(selector.clone()).is_none() {
                return Ok(());
            }
            if start.elapsed() >= timeout {
                return Err(super::Error::Timeout {
                    description: format!("{description} to disappear"),
                    elapsed: start.elapsed(),
                });
            }
            std::thread::sleep(poll_interval);
        }
    }
}

/// Conversion into an [`InstrTarget`] for click/move/hover-style methods.
///
/// Implemented for `&str` / `String` (text content), [`iced_core::Point`]
/// (absolute coordinates), and [`iced_core::widget::Id`] (widget id).
pub trait IntoSelector {
    /// Converts `self` into a [`Target`](InstrTarget) used by [`Instruction`].
    fn into_instr_target(self) -> InstrTarget;
}

impl IntoSelector for &str {
    fn into_instr_target(self) -> InstrTarget {
        InstrTarget::Text(self.to_owned())
    }
}

impl IntoSelector for String {
    fn into_instr_target(self) -> InstrTarget {
        InstrTarget::Text(self)
    }
}

impl IntoSelector for iced_test::core::Point {
    fn into_instr_target(self) -> InstrTarget {
        InstrTarget::Point(self)
    }
}

impl IntoSelector for widget::Id {
    fn into_instr_target(self) -> InstrTarget {
        // `widget::Id` does not expose its inner string, so we format it.
        // Iced's instruction parser also accepts the `#id` form.
        let id = format!("{self:?}");
        // `widget::Id::Debug` formats as e.g. `Id("foo")`. Strip the wrapper
        // so the instruction's target is the bare id.
        let trimmed = id
            .strip_prefix("Id(\"")
            .and_then(|s| s.strip_suffix("\")"))
            .unwrap_or(&id)
            .to_owned();
        InstrTarget::Id(trimmed)
    }
}

/// A bare id string, useful when you have the id as a `&str`/`String`.
///
/// Pass via `Id::new("my-id")` to disambiguate from text content selection.
pub struct Id(pub String);

impl Id {
    /// Creates a new id selector from any string-like value.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl IntoSelector for Id {
    fn into_instr_target(self) -> InstrTarget {
        InstrTarget::Id(self.0)
    }
}

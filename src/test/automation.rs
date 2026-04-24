//! Automation testing for `iced` applications.
//!
//! This module provides a Playwright-style [`Emulator`] that wraps any
//! [`iced::Program`] and lets you drive it from plain `#[test]` functions:
//!
//! ```ignore
//! # use std::time::Duration;
//! # fn program() -> impl iced_test::program::Program { unimplemented!() }
//! #[test]
//! fn my_test() -> snowscape::test::automation::Result {
//!     let mut emulator = snowscape::test::Emulator::new(program())?;
//!     emulator.click("Increment")?;
//!     emulator.wait(Duration::from_millis(100))?;
//!     assert!(emulator.exists("Count: 1"));
//!     Ok(())
//! }
//! ```
//!
//! The [`Emulator`] runs the program with its real executor, so subscriptions
//! and tasks behave exactly as they do in production. Use [`Emulator::wait`]
//! to sleep and drain background events, and [`Emulator::wait_for`] /
//! [`Emulator::wait_for_text`] to poll for a state change.

use std::time::Duration;

use iced_test::core::renderer as core_renderer;
use iced_test::core::{Settings, Size, mouse, widget};
use iced_test::emulator::{Action, Event, Mode};
use iced_test::futures::futures::channel::mpsc;
use iced_test::futures::futures::executor;
use iced_test::instruction::{
    Interaction, Key, Keyboard, Mouse as MouseInteraction, Target as InstrTarget,
};
use iced_test::program::Program;
use iced_test::runtime::UserInterface;
use iced_test::runtime::user_interface::Cache;
use iced_test::{Instruction, Selector};

/// Convenience re-exports for building [`Selector`]s in automation tests.
pub mod select {
    pub use iced_test::Selector;
    pub use iced_test::selector::{Bounded, Candidate, Target, Text, id, is_focused};
}

/// The default timeout used by [`Emulator::wait_for`] and friends when no
/// explicit timeout is provided.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Errors that can arise from automation tests.
#[derive(Debug)]
pub enum Error {
    /// An [`Instruction`] failed to execute (e.g. a selector did not match).
    InstructionFailed(Instruction),
    /// A [`Selector`] did not match anything.
    SelectorNotFound(String),
    /// The matched [`Target`] is not a textual widget.
    NotTextual(String),
    /// A [`wait_for`](Emulator::wait_for) call exceeded its timeout.
    Timeout {
        /// What we were waiting for.
        description: String,
        /// How long we waited.
        elapsed: Duration,
    },
    /// The headless runtime disconnected unexpectedly.
    Disconnected,
    /// Failed to construct the headless renderer.
    Renderer(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InstructionFailed(instr) => {
                write!(f, "instruction failed: {instr}")
            }
            Error::SelectorNotFound(desc) => {
                write!(f, "selector not found: {desc}")
            }
            Error::NotTextual(desc) => {
                write!(f, "selected target is not textual: {desc}")
            }
            Error::Timeout {
                description,
                elapsed,
            } => write!(f, "timed out after {elapsed:?} waiting for: {description}"),
            Error::Disconnected => write!(f, "emulator runtime disconnected"),
            Error::Renderer(msg) => write!(f, "failed to create headless renderer: {msg}"),
        }
    }
}

impl std::error::Error for Error {}

/// Convenience alias for automation test return types.
///
/// Defaults to `Result<(), Error>`, which is the common signature for
/// `#[test]` functions that use the [`Emulator`].
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Builder for an [`Emulator`].
pub struct Builder<P: Program> {
    program: P,
    mode: Mode,
    size: Size,
    preset: Option<String>,
    default_timeout: Duration,
}

impl<P: Program + 'static> Builder<P> {
    /// Sets the headless [`Mode`] used to wait for tasks between instructions.
    #[must_use]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the viewport size of the emulator.
    #[must_use]
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }

    /// Selects a [`Preset`](iced::Preset) by name to boot the program with.
    #[must_use]
    pub fn preset(mut self, name: impl Into<String>) -> Self {
        self.preset = Some(name.into());
        self
    }

    /// Sets the default timeout used by [`Emulator::wait_for`] and friends.
    #[must_use]
    pub fn default_timeout(mut self, timeout: Duration) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Boots the [`Emulator`].
    pub fn build(self) -> Result<Emulator<P>> {
        Emulator::boot(
            self.program,
            self.mode,
            self.size,
            self.preset,
            self.default_timeout,
        )
    }
}

/// A Playwright-style headless test driver for any [`iced::Program`].
///
/// Built on top of [`iced_test::Emulator`], the [`Emulator`] runs the program
/// with its real executor (so subscriptions and tasks fire normally) and
/// exposes methods for clicking, typing, waiting on conditions, and
/// querying the rendered widget tree.
pub struct Emulator<P: Program> {
    /// The inner test emulator that runs the program and produces events.
    inner: iced_test::Emulator<P>,
    /// The program being tested passed in by the user.
    program: P,
    /// The channel receiving events from the inner emulator.
    receiver: mpsc::Receiver<Event<P>>,
    /// The headless renderer used for selector queries.
    renderer: P::Renderer,
    /// The configured viewport size of the emulator.
    size: Size,
    /// The current position of the mouse cursor, tracked across interactions.
    cursor: mouse::Cursor,
    default_timeout: Duration,
}

impl<P: Program + 'static> Emulator<P> {
    /// Boots an [`Emulator`] for the given program with default settings:
    /// [`Mode::Zen`], 1024x768 viewport, no preset, and [`DEFAULT_TIMEOUT`].
    pub fn new(program: P) -> Result<Self> {
        Self::builder(program).build()
    }

    /// Begins building an [`Emulator`] with custom settings.
    pub fn builder(program: P) -> Builder<P> {
        Builder {
            program,
            mode: Mode::default(),
            size: Size::new(1024.0, 768.0),
            preset: None,
            default_timeout: DEFAULT_TIMEOUT,
        }
    }

    fn boot(
        program: P,
        mode: Mode,
        size: Size,
        preset_name: Option<String>,
        default_timeout: Duration,
    ) -> Result<Self> {
        let (sender, receiver) = mpsc::channel(64);

        let preset = preset_name
            .as_deref()
            .map(|name| {
                program
                    .presets()
                    .iter()
                    .find(|candidate| candidate.name() == name)
                    .ok_or_else(|| {
                        Error::SelectorNotFound(format!(
                            "preset lookup failed: preset {name:?} not found"
                        ))
                    })
            })
            .transpose()?;

        let inner = iced_test::Emulator::with_preset(sender, &program, mode, size, preset);

        let settings = program.settings();
        let renderer = build_renderer::<P::Renderer>(&settings)?;

        let mut emulator = Self {
            inner,
            program,
            receiver,
            renderer,
            size,
            cursor: mouse::Cursor::Unavailable,
            default_timeout,
        };

        // Drain the initial `Ready` event produced during boot.
        emulator.pump_until_ready()?;

        Ok(emulator)
    }

    /// Returns the configured default timeout for `wait_for*` methods.
    pub fn default_timeout(&self) -> Duration {
        self.default_timeout
    }

    /// Replaces the default timeout used by `wait_for*` methods.
    pub fn set_default_timeout(&mut self, timeout: Duration) {
        self.default_timeout = timeout;
    }

    /// Returns the viewport size of the emulator.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Returns the current theme of the program, if any.
    pub fn theme(&self) -> Option<P::Theme> {
        self.inner.theme(&self.program)
    }

    /// Consumes the [`Emulator`] and returns the final program state.
    pub fn into_state(self) -> P::State {
        let (state, _window) = self.inner.into_state();
        state
    }

    fn pump_until_ready(&mut self) -> Result {
        use iced_test::futures::futures::StreamExt;

        executor::block_on(async {
            loop {
                match self.receiver.next().await {
                    Some(Event::Action(action)) => self.dispatch(action),
                    Some(Event::Ready) => return Ok(()),
                    Some(Event::Failed(instruction)) => {
                        return Err(Error::InstructionFailed(instruction));
                    }
                    None => return Err(Error::Disconnected),
                }
            }
        })
    }

    fn dispatch(&mut self, action: Action<P>) {
        self.inner.perform(&self.program, action);
    }

    /// Drains any actions queued by background tasks/subscriptions without
    /// blocking. This is used internally after `wait` to process anything
    /// that fired while we slept.
    fn drain_pending(&mut self) -> Result {
        loop {
            match self.receiver.try_recv() {
                Ok(Event::Action(action)) => self.dispatch(action),
                Ok(Event::Ready) => {}
                Ok(Event::Failed(instruction)) => {
                    return Err(Error::InstructionFailed(instruction));
                }
                Err(_) => return Ok(()),
            }
        }
    }

    fn run_instruction(&mut self, instruction: Instruction) -> Result {
        // Track cursor position for any move events embedded in the instruction.
        if let Instruction::Interact(Interaction::Mouse(MouseInteraction::Move(
            InstrTarget::Point(point),
        ))) = &instruction
        {
            self.cursor = mouse::Cursor::Available(*point);
        }
        self.inner.run(&self.program, &instruction);
        self.pump_until_ready()
    }

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
        S: Selector + Clone + Send,
        S::Output: Clone + Send,
    {
        self.wait_for_with_timeout(selector, self.default_timeout)
    }

    /// Polls `selector` until it matches or `timeout` elapses.
    pub fn wait_for_with_timeout<S>(&mut self, selector: S, timeout: Duration) -> Result<S::Output>
    where
        S: Selector + Clone + Send,
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
                return Err(Error::Timeout {
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
                return Err(Error::Timeout {
                    description: format!("text {text:?} to appear"),
                    elapsed: start.elapsed(),
                });
            }
            std::thread::sleep(poll_interval);
        }
    }

    // MARK: - Queries

    /// Returns the output of the selector if it matches the current view.
    pub fn find<S>(&mut self, selector: S) -> Result<S::Output>
    where
        S: Selector + Send,
        S::Output: Clone + Send,
    {
        let description = selector.description();
        self.try_find(selector)
            .ok_or(Error::SelectorNotFound(description))
    }

    /// Returns whether the selector matches anything in the current view.
    pub fn exists<S>(&mut self, selector: S) -> bool
    where
        S: Selector + Send,
        S::Output: Clone + Send,
    {
        self.try_find(selector).is_some()
    }

    /// Returns whether the given text appears anywhere in the current view.
    pub fn contains_text(&mut self, text: impl AsRef<str>) -> bool {
        self.exists(text.as_ref().to_owned())
    }

    /// Returns the textual content of the widget targeted by `selector`.
    ///
    /// The selector must match a text or text-input widget (for example by
    /// using [`select::id`]).
    pub fn get_text<S>(&mut self, selector: S) -> Result<String>
    where
        S: Selector<Output = select::Target> + Send,
    {
        let description = selector.description();
        let target = self.find(selector)?;
        match target {
            select::Target::Text { content, .. } | select::Target::TextInput { content, .. } => {
                Ok(content)
            }
            _ => Err(Error::NotTextual(description)),
        }
    }

    fn try_find<S>(&mut self, selector: S) -> Option<S::Output>
    where
        S: Selector + Send,
        S::Output: Clone + Send,
    {
        use widget::Operation;

        let element = self.inner.view(&self.program);
        let mut ui = UserInterface::build(element, self.size, Cache::default(), &mut self.renderer);

        let mut operation = selector.find();
        ui.operate(
            &self.renderer,
            &mut widget::operation::black_box(&mut operation),
        );
        let _ = ui.into_cache();

        match operation.finish() {
            widget::operation::Outcome::Some(output) => output,
            _ => None,
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

fn build_renderer<R>(settings: &Settings) -> Result<R>
where
    R: core_renderer::Headless,
{
    let backend = std::env::var("ICED_TEST_BACKEND").ok();
    // The iced renderer's `new` is async, but we only need a one-shot
    // block_on here. The iced executor (tokio) lives on its own threads
    // inside the `iced_test::Emulator`, so there is no nested-runtime risk.
    executor::block_on(R::new(
        core_renderer::Settings {
            default_font: settings.default_font,
            default_text_size: settings.default_text_size,
        },
        backend.as_deref(),
    ))
    .ok_or_else(|| Error::Renderer("renderer initialization returned None".to_owned()))
}

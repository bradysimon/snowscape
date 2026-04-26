//! A headless test driver for an [`iced::Program`].
use std::path::PathBuf;
use std::time::Duration;

use super::{Builder, Error, Result};

use iced_test::Instruction;
use iced_test::core::renderer as core_renderer;
use iced_test::core::{Settings, Size, mouse, widget};
use iced_test::emulator::{Action, Event, Mode};
use iced_test::futures::futures::channel::mpsc;
use iced_test::futures::futures::executor;
use iced_test::instruction::{Interaction, Mouse as MouseInteraction, Target as InstrTarget};
use iced_test::program::Program;
use iced_test::runtime::UserInterface;
use iced_test::runtime::user_interface::Cache;

/// A Playwright-style headless test driver for any [`iced::Program`].
///
/// Built on top of [`iced_test::Emulator`], the [`Emulator`] runs the program
/// with its real executor (so subscriptions and tasks fire normally) and
/// exposes methods for clicking, typing, waiting on conditions, and
/// querying the rendered widget tree.
pub struct Emulator<P: Program> {
    /// The inner test emulator that runs the program and produces events.
    pub(super) inner: iced_test::Emulator<P>,
    /// The program being tested passed in by the user.
    pub(super) program: P,
    /// The channel receiving events from the inner emulator.
    pub(super) receiver: mpsc::Receiver<Event<P>>,
    /// The headless renderer used for selector queries.
    pub(super) renderer: P::Renderer,
    /// The configured viewport size of the emulator.
    pub(super) size: Size,
    /// The current position of the mouse cursor, tracked across interactions.
    pub(super) cursor: mouse::Cursor,
    pub(super) default_timeout: Duration,
    pub(super) screenshot_dir: Option<PathBuf>,
    /// Cached layout from the last [`UserInterface`] build. Reused across
    /// query operations and invalidated whenever an action mutates state.
    pub(super) cache: Cache,
}

impl<P: Program + 'static> Emulator<P> {
    /// Boots an [`Emulator`] for the given program with default settings:
    /// [`Mode::Zen`], 1024x768 viewport, no preset, and [`DEFAULT_TIMEOUT`].
    pub fn new(program: P) -> Result<Self> {
        Self::builder(program).build()
    }

    /// Begins building an [`Emulator`] with custom settings.
    pub fn builder(program: P) -> Builder<P> {
        Builder::new(program)
    }

    pub(super) fn boot(
        program: P,
        mode: Mode,
        size: Size,
        preset_name: Option<String>,
        default_timeout: Duration,
        screenshot_dir: Option<PathBuf>,
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
            screenshot_dir,
            cache: Cache::default(),
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

    pub(super) fn pump_until_ready(&mut self) -> Result {
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
        // The action may have mutated program state, invalidating the
        // cached layout we keep for query operations.
        self.cache = Cache::default();
    }

    /// Drains any actions queued by background tasks/subscriptions without
    /// blocking. This is used internally after `wait` to process anything
    /// that fired while we slept.
    pub(super) fn drain_pending(&mut self) -> Result {
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

    pub(super) fn run_instruction(&mut self, instruction: Instruction) -> Result {
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

    /// Runs a [`widget::Operation`] against a freshly built (or cached)
    /// [`UserInterface`] and returns the operation's [`Outcome`].
    ///
    /// The cache produced by `UserInterface::into_cache` is preserved on
    /// the emulator so subsequent operations skip re-layout when state
    /// hasn't changed. State-mutating actions clear the cache via
    /// [`dispatch`](Self::dispatch).
    pub(super) fn run_operation<T, O>(&mut self, mut operation: O) -> widget::operation::Outcome<T>
    where
        O: widget::Operation<T>,
    {
        let element = self.inner.view(&self.program);
        let cache = std::mem::take(&mut self.cache);
        let mut ui = UserInterface::build(element, self.size, cache, &mut self.renderer);
        ui.operate(
            &self.renderer,
            &mut widget::operation::black_box(&mut operation),
        );
        self.cache = ui.into_cache();
        operation.finish()
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

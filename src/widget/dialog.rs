//! A composable dialog wrapper that can show modal content over a base element.
//!
//! The dialog lifecycle is externally controlled with [`State`] and lifecycle
//! events are emitted as [`Message`] so parent state can keep content mounted
//! until close animations are fully complete.
use iced::{
    Alignment, Animation, Color, Element, Event,
    Length::{self, Fill},
    Rectangle, Size, Theme, Vector,
    advanced::{
        Layout, Shell,
        layout::{Limits, Node},
        mouse, overlay,
        renderer::{self, Quad},
        widget::{Operation, Tree, tree},
    },
    alignment::Horizontal::Right,
    animation, keyboard,
    time::{Duration, Instant},
    touch,
    widget::{button, column, container, mouse_area, row, space, text},
    window,
};

/// The default width used by a dialog panel.
pub const DEFAULT_WIDTH: Length = Length::Fixed(400.0);

/// Dialog lifecycle message emitted by the dialog widget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Message {
    /// The dialog has finished opening.
    Opened,
    /// The user requested the dialog to close.
    Close,
    /// The dialog has finished closing.
    /// Primarily used for animated dialogs to signal the end of the closing animation.
    Closed,
}

/// Actions that the dialog informs the main app about.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// The dialog has been closed and the parent app should clean up any necessary state.
    Closed,
}

/// The visual status of the dialog.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Status {
    /// The dialog is fully closed.
    #[default]
    Closed,
    /// The dialog is in the process of opening and animating in.
    Opening,
    /// The dialog is open and visible.
    Open,
    /// The dialog is in the process of closing and animating out.
    Closing,
}

/// External dialog state managed by the parent app.
#[derive(Debug, Clone, Copy)]
pub struct State {
    /// The current visual status of the dialog.
    /// Animated dialogs will use [`Status::Opening`] and [`Status::Closing`] states.
    status: Status,
    /// Whether the dialog should animate when opening or closing.
    is_animated: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            status: Status::Closed,
            is_animated: true,
        }
    }
}

impl State {
    /// Sets whether the dialog should animate when opening or closing.
    pub fn animated(mut self, animated: bool) -> Self {
        self.is_animated = animated;
        self
    }

    /// Sets the dialog state to open.
    pub fn open(&mut self) {
        self.status = if self.is_animated {
            Status::Opening
        } else {
            Status::Open
        };
    }

    /// Closes the dialog, either immediately closing if not animated or starting the closing animation.
    pub fn close(&mut self) {
        if self.is_animated {
            self.status = Status::Closing;
        } else {
            self.status = Status::Closed;
        }
    }

    /// Applies a dialog lifecycle message to this state.
    #[must_use]
    pub fn update(&mut self, message: Message) -> Option<Action> {
        match message {
            Message::Opened => {
                self.status = Status::Open;
                None
            }
            Message::Close => {
                if self.is_animated {
                    if self.status != Status::Closing {
                        self.status = Status::Closing;
                    }
                    None
                } else {
                    self.status = Status::Closed;
                    Some(Action::Closed)
                }
            }
            Message::Closed => {
                self.status = Status::Closed;
                Some(Action::Closed)
            }
        }
    }

    /// Returns the current status.
    pub fn status(&self) -> Status {
        self.status
    }

    /// Returns whether this dialog state is configured to animate.
    pub fn is_animated(&self) -> bool {
        self.is_animated
    }

    /// Returns true when the dialog target state is open.
    pub fn is_open(&self) -> bool {
        matches!(self.status, Status::Opening | Status::Open)
    }

    /// Returns true while dialog content should be rendered.
    pub fn is_visible(&self) -> bool {
        self.status != Status::Closed
    }
}

/// Creates a new [`Dialog`] wrapping the given `base` element.
///
/// The external [`State`] controls whether the dialog target is open or closing.
pub fn dialog<'a, Message, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    state: &'a State,
    config: Option<Config<'a, Message, Renderer>>,
) -> Dialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    Dialog::new(base, state, config)
}

/// Describes what to show inside a [`Dialog`] panel.
///
/// Bundles the body content, optional title, and footer action buttons into a
/// single value that can be passed as `Option<Config>` to [`dialog()`].
pub struct Config<'a, Message, Renderer = iced::Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    /// Optional title shown in the dialog header.
    title: Option<String>,
    /// Optional close button label shown next to the close icon.
    close_label: Option<String>,
    /// The width of the dialog panel.
    width: Length,
    /// Body content shown inside the dialog panel.
    content: Element<'a, Message, Theme, Renderer>,
    /// Footer action widgets rendered at the bottom of the panel.
    actions: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Renderer> Config<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    /// Creates a new [`Config`] with the given body content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            title: None,
            close_label: None,
            width: DEFAULT_WIDTH,
            content: content.into(),
            actions: Vec::new(),
        }
    }

    /// Sets an optional dialog title shown in the header.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets an optional close button label shown next to the close icon.
    #[must_use]
    pub fn close_label(mut self, label: impl Into<String>) -> Self {
        self.close_label = Some(label.into());
        self
    }

    /// Sets the width of the dialog panel.
    ///
    /// Some designs like Material recommend widths between 280-560px.
    /// See: [`DEFAULT_WIDTH`]
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Adds an action element to the dialog footer.
    #[must_use]
    pub fn push_action(mut self, action: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.actions.push(action.into());
        self
    }

    /// Adds multiple action elements to the dialog footer.
    #[must_use]
    pub fn actions(
        mut self,
        actions: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        self.actions.extend(actions);
        self
    }
}

/// A composable dialog wrapper that can show modal content over a base element.
///
/// Use [`dialog()`] to create a `Dialog`, passing `Some(config)` to show a
/// dialog or `None` to hide it. The dialog will animate open/closed
/// automatically when the config transitions between `Some` and `None`.
pub struct Dialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    /// The underlying content that remains visible behind the dialog.
    base: Element<'a, Message, Theme, Renderer>,
    /// Body content shown inside the dialog panel.
    ///
    /// Always populated — when `config` is `None`, a transparent placeholder is
    /// used so the widget tree structure stays stable for close animations.
    content: Element<'a, Message, Theme, Renderer>,
    /// Whether the dialog should be shown.
    open: bool,
    /// Callback used to map dialog lifecycle events into app messages.
    on_update: Option<Box<dyn Fn(self::Message) -> Message + 'a>>,
    /// Optional title shown in the dialog header.
    title: Option<String>,
    /// Optional close button label shown next to the close icon.
    close_label: Option<String>,
    /// The width of the dialog panel.
    width: Length,
    /// Whether clicking the backdrop triggers the close message.
    backdrop_close: bool,
    /// Whether pressing `Esc` triggers the close message.
    esc_close: bool,
    /// Whether open/close transitions are animated.
    animate: bool,
    /// Optional footer action widgets rendered at the bottom of the panel.
    actions: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Renderer> Dialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    /// Creates a new [`Dialog`] with the given base element and optional config.
    ///
    /// When `config` is `Some`, the dialog is open. When `None`, it is closed
    /// (but still present in the widget tree so close animations can run).
    pub fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        state: &State,
        config: Option<Config<'a, Message, Renderer>>,
    ) -> Self {
        let (content, title, close_label, width, actions, open) = match config {
            Some(config) => (
                config.content,
                config.title,
                config.close_label,
                config.width,
                config.actions,
                state.is_open(),
            ),
            None => (space().into(), None, None, DEFAULT_WIDTH, Vec::new(), false),
        };

        Self {
            base: base.into(),
            content,
            open,
            on_update: None,
            title,
            close_label,
            width,
            backdrop_close: true,
            esc_close: true,
            animate: state.is_animated(),
            actions,
        }
    }

    /// Sets the app message mapper for dialog lifecycle events.
    #[must_use]
    pub fn on_update(mut self, mapper: impl Fn(self::Message) -> Message + 'a) -> Self {
        self.on_update = Some(Box::new(mapper));
        self
    }

    /// Sets whether clicking the backdrop will close the dialog.
    #[must_use]
    pub fn backdrop_close(mut self, enabled: bool) -> Self {
        self.backdrop_close = enabled;
        self
    }

    /// Sets whether pressing `Esc` will close the dialog.
    #[must_use]
    pub fn esc_close(mut self, enabled: bool) -> Self {
        self.esc_close = enabled;
        self
    }

    /// Builds the finalized overlay widgets used by the runtime dialog.
    ///
    /// This function takes the builder-time dialog configuration and constructs:
    /// - a full-size backdrop interaction target (for optional backdrop close), and
    /// - a styled dialog panel containing header, content, and footer actions.
    ///
    /// The returned tuple is `(backdrop_target, panel)` and is moved into
    /// [`ResolvedDialog`] during `From<Dialog> for Element`.
    fn build_overlay_parts(
        content: Element<'a, Message, Theme, Renderer>,
        title_text: Option<String>,
        close_label: Option<String>,
        width: Length,
        actions: Vec<Element<'a, Message, Theme, Renderer>>,
        close_message: Option<Message>,
        backdrop_close: bool,
    ) -> (
        Element<'a, Message, Theme, Renderer>,
        Element<'a, Message, Theme, Renderer>,
    )
    where
        Renderer: iced::advanced::svg::Renderer + iced::advanced::text::Renderer,
    {
        let close_content: Element<'a, Message, Theme, Renderer> = if let Some(label) = close_label
        {
            row![
                crate::icon::xmark()
                    .width(14)
                    .height(14)
                    .style(crate::style::svg::text),
                text(label).size(14),
            ]
            .spacing(6)
            .align_y(Alignment::Center)
            .into()
        } else {
            crate::icon::xmark()
                .width(16)
                .height(16)
                .style(crate::style::svg::text)
                .into()
        };

        let close_button: Element<'a, Message, Theme, Renderer> =
            if let Some(message) = close_message.clone() {
                button(close_content)
                    .padding(6)
                    .style(crate::style::button::ghost_subtle)
                    .on_press(message)
                    .into()
            } else {
                button(close_content)
                    .padding(6)
                    .style(crate::style::button::ghost_subtle)
                    .into()
            };

        let title: Element<'a, Message, Theme, Renderer> = if let Some(title) = title_text {
            text(title).size(18).into()
        } else {
            space().into()
        };

        let header = row![title, space::horizontal(), close_button].align_y(Alignment::Center);

        let body = column![
            column![header, content].spacing(8),
            (!actions.is_empty()).then(|| row(actions).spacing(8).align_y(Alignment::Center))
        ]
        .spacing(8)
        .align_x(Right);

        let panel: Element<'a, Message, Theme, Renderer> = container(body)
            .padding(16)
            .width(width)
            .style(crate::style::container::dialog_panel)
            .into();

        let backdrop_target = if backdrop_close {
            if let Some(message) = close_message {
                mouse_area(space().width(Fill).height(Fill))
                    .on_press(message)
                    .into()
            } else {
                space().width(Fill).height(Fill).into()
            }
        } else {
            space().width(Fill).height(Fill).into()
        };

        (backdrop_target, panel)
    }
}

impl<'a, Message, Renderer> From<Dialog<'a, Message, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer
        + iced::advanced::svg::Renderer
        + iced::advanced::text::Renderer
        + 'a,
{
    fn from(dialog: Dialog<'a, Message, Renderer>) -> Self {
        let Dialog {
            base,
            content,
            open,
            on_update,
            title,
            close_label,
            width,
            backdrop_close,
            esc_close,
            animate,
            actions,
            ..
        } = dialog;

        let close_intent_message = on_update.as_ref().map(|map| map(self::Message::Close));
        let on_opened = on_update.as_ref().map(|map| map(self::Message::Opened));
        let on_closed = on_update.as_ref().map(|map| map(self::Message::Closed));

        let (backdrop_target, panel) = Dialog::build_overlay_parts(
            content,
            title.clone(),
            close_label,
            width,
            actions,
            close_intent_message.clone(),
            backdrop_close,
        );

        Element::new(ResolvedDialog {
            base,
            backdrop_target,
            panel,
            open,
            on_opened,
            on_close_intent: close_intent_message,
            on_closed,
            esc_close,
            animate,
        })
    }
}

/// Internal, fully-resolved runtime form of [`Dialog`].
///
/// `Dialog` is the public builder/config type that collects normal `Element`
/// content (base, body, title, actions). During `From<Dialog> for Element`,
/// those pieces are composed into concrete runtime children (`backdrop_target`
/// and `panel`) and moved into `ResolvedDialog`.
///
/// This split keeps the public API ergonomic while ensuring the actual widget
/// implementation always has a valid, fully-constructed set of children.
struct ResolvedDialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    base: Element<'a, Message, Theme, Renderer>,
    backdrop_target: Element<'a, Message, Theme, Renderer>,
    panel: Element<'a, Message, Theme, Renderer>,
    open: bool,
    on_opened: Option<Message>,
    on_close_intent: Option<Message>,
    on_closed: Option<Message>,
    esc_close: bool,
    animate: bool,
}

struct WidgetState {
    visibility: Animation<bool>,
    now: Instant,
    was_open: bool,
    opened_emitted: bool,
    closed_emitted: bool,
}

impl WidgetState {
    fn new() -> Self {
        Self {
            visibility: Animation::new(false)
                .duration(Duration::from_millis(500))
                .easing(animation::Easing::EaseInOutBack),
            now: Instant::now(),
            was_open: false,
            opened_emitted: false,
            closed_emitted: false,
        }
    }
}

impl<'a, Message, Renderer> ResolvedDialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    /// Returns the current progress of the dialog animation, from `0.0` to `1.0`.
    fn progress(&self, state: &WidgetState) -> f32 {
        if self.animate {
            state.visibility.interpolate(0.0, 1.0, state.now)
        } else if self.open {
            1.0
        } else {
            0.0
        }
    }

    /// Returns whether the dialog is currently showing or animating.
    fn is_showing(&self, state: &WidgetState) -> bool {
        let progress = self.progress(state);
        progress > 0.0 || self.open || (self.animate && state.visibility.is_animating(state.now))
    }

    fn is_transitioning(&self, state: &WidgetState) -> bool {
        self.animate && state.visibility.is_animating(state.now)
    }

    fn did_close(&self, state: &WidgetState) -> bool {
        self.animate
            && !self.open
            && state.was_open
            && !state.visibility.is_animating(state.now)
            && self.progress(state) <= f32::EPSILON
    }

    fn did_open(&self, state: &WidgetState) -> bool {
        self.open
            && (!self.animate
                || (!state.visibility.is_animating(state.now)
                    && self.progress(state) >= 1.0 - f32::EPSILON))
    }
}

impl<Message, Renderer> iced::advanced::Widget<Message, Theme, Renderer>
    for ResolvedDialog<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<WidgetState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(WidgetState::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![
            Tree::new(&self.base),
            Tree::new(&self.backdrop_target),
            Tree::new(&self.panel),
        ]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[&self.base, &self.backdrop_target, &self.panel]);
    }

    fn size(&self) -> Size<iced::Length> {
        self.base.as_widget().size()
    }

    fn size_hint(&self) -> Size<iced::Length> {
        self.base.as_widget().size_hint()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let base = self
            .base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        let size = base.size();

        let overlay_limits = Limits::new(Size::ZERO, size);
        let backdrop = self.backdrop_target.as_widget_mut().layout(
            &mut tree.children[1],
            renderer,
            &overlay_limits.width(Fill).height(Fill),
        );

        let mut panel =
            self.panel
                .as_widget_mut()
                .layout(&mut tree.children[2], renderer, &overlay_limits);
        panel.align_mut(Alignment::Center, Alignment::Center, size);

        Node::with_children(size, vec![base, backdrop, panel])
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<WidgetState>();

        if self.open {
            state.was_open = true;
            state.closed_emitted = false;
        } else {
            state.opened_emitted = false;
        }

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = *now;
        }

        let was_animating = state.visibility.is_animating(state.now);
        if self.animate {
            state.visibility.go_mut(self.open, state.now);
        }

        if self.animate {
            let is_animating = state.visibility.is_animating(state.now);

            if matches!(event, Event::Window(window::Event::RedrawRequested(_))) {
                if is_animating {
                    shell.request_redraw();
                }
            } else if !was_animating && is_animating {
                // Kick off the animation on state changes (e.g. open/close click).
                shell.request_redraw();
            }
        }

        if self.did_open(state) && !state.opened_emitted {
            if let Some(message) = &self.on_opened {
                shell.publish(message.clone());
            }
            state.opened_emitted = true;
        }

        if self.did_close(state) && !state.closed_emitted {
            if let Some(message) = &self.on_closed {
                shell.publish(message.clone());
            }
            state.closed_emitted = true;
            state.was_open = false;
            state.opened_emitted = false;
        }

        let mut children = layout.children();
        let Some(base_layout) = children.next() else {
            return;
        };
        let Some(backdrop_layout) = children.next() else {
            return;
        };
        let Some(panel_layout) = children.next() else {
            return;
        };

        if self.is_showing(state) {
            let is_transitioning = self.is_transitioning(state);
            let panel_press = matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. })
            ) && cursor.is_over(panel_layout.bounds());

            if !is_transitioning {
                if !panel_press {
                    self.backdrop_target.as_widget_mut().update(
                        &mut tree.children[1],
                        event,
                        backdrop_layout,
                        cursor,
                        renderer,
                        shell,
                        viewport,
                    );
                }

                self.panel.as_widget_mut().update(
                    &mut tree.children[2],
                    event,
                    panel_layout,
                    cursor,
                    renderer,
                    shell,
                    viewport,
                );

                if self.esc_close
                    && let Some(message) = &self.on_close_intent
                    && let Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) = event
                    && matches!(
                        key.as_ref(),
                        keyboard::Key::Named(keyboard::key::Named::Escape)
                    )
                    && !shell.is_event_captured()
                {
                    shell.publish(message.clone());
                    shell.capture_event();
                }
            }

            let in_bounds = match event {
                Event::Mouse(mouse::Event::ButtonPressed(_)) => {
                    cursor.is_over(backdrop_layout.bounds())
                }
                Event::Touch(touch::Event::FingerPressed { position, .. }) => {
                    backdrop_layout.bounds().contains(*position)
                }
                _ => false,
            };

            if in_bounds && !shell.is_event_captured() {
                shell.capture_event();
            }

            return;
        }

        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            base_layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<WidgetState>();

        let mut children = layout.children();
        let Some(base_layout) = children.next() else {
            return;
        };

        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            base_layout,
            cursor,
            viewport,
        );

        let progress = self.progress(state);
        if progress <= 0.0 {
            return;
        }

        let Some(_backdrop_layout) = children.next() else {
            return;
        };
        let Some(panel_layout) = children.next() else {
            return;
        };

        renderer.start_layer(layout.bounds());

        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                ..Quad::default()
            },
            Color::BLACK.scale_alpha(0.5 * progress),
        );

        let y_offset = (1.0 - progress) * 18.0;
        renderer.with_translation(Vector::new(0.0, y_offset), |renderer| {
            self.panel.as_widget().draw(
                &tree.children[2],
                renderer,
                theme,
                style,
                panel_layout,
                cursor,
                viewport,
            );
        });
        renderer.end_layer();
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<WidgetState>();

        let mut children = layout.children();
        let Some(base_layout) = children.next() else {
            return mouse::Interaction::None;
        };
        let Some(backdrop_layout) = children.next() else {
            return mouse::Interaction::None;
        };
        let Some(panel_layout) = children.next() else {
            return mouse::Interaction::None;
        };

        if self.is_showing(state) {
            if self.is_transitioning(state) {
                return mouse::Interaction::None;
            }

            let panel_interaction = self.panel.as_widget().mouse_interaction(
                &tree.children[2],
                panel_layout,
                cursor,
                viewport,
                renderer,
            );

            if panel_interaction != mouse::Interaction::None {
                panel_interaction
            } else {
                self.backdrop_target.as_widget().mouse_interaction(
                    &tree.children[1],
                    backdrop_layout,
                    cursor,
                    viewport,
                    renderer,
                )
            }
        } else {
            self.base.as_widget().mouse_interaction(
                &tree.children[0],
                base_layout,
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let mut children = layout.children();

        if let Some(base_layout) = children.next() {
            self.base.as_widget_mut().operate(
                &mut tree.children[0],
                base_layout,
                renderer,
                operation,
            );
        }

        if let Some(backdrop_layout) = children.next() {
            self.backdrop_target.as_widget_mut().operate(
                &mut tree.children[1],
                backdrop_layout,
                renderer,
                operation,
            );
        }

        if let Some(panel_layout) = children.next() {
            self.panel.as_widget_mut().operate(
                &mut tree.children[2],
                panel_layout,
                renderer,
                operation,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let mut children = layout.children();
        let base_layout = children.next()?;
        let _backdrop_layout = children.next()?;
        let panel_layout = children.next()?;

        let show_panel_overlay = {
            let state = tree.state.downcast_ref::<WidgetState>();
            self.is_showing(state) && !self.is_transitioning(state)
        };

        let (base_and_backdrop, panel_tree) = tree.children.split_at_mut(2);
        let base_tree = &mut base_and_backdrop[0];
        let panel_tree = &mut panel_tree[0];

        let mut overlays = Vec::new();

        if let Some(base_overlay) = self.base.as_widget_mut().overlay(
            base_tree,
            base_layout,
            renderer,
            viewport,
            translation,
        ) {
            overlays.push(base_overlay);
        }

        if show_panel_overlay
            && let Some(panel_overlay) = self.panel.as_widget_mut().overlay(
                panel_tree,
                panel_layout,
                renderer,
                viewport,
                translation,
            )
        {
            overlays.push(panel_overlay);
        }

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Opening a non-animated dialog should immediately set its status to open.
    #[test]
    fn open_no_animation() {
        let mut state = State::default();
        state.open();
        assert_eq!(state.status, Status::Open);
    }

    /// Opening an animated dialog should set its status to opening.
    #[test]
    fn open_animated() {
        let mut state = State::default().animated(true);
        state.open();
        assert_eq!(state.status, Status::Opening);
    }

    /// Requesting the dialog to close should transition it to the closed state.
    #[test]
    fn update_close_no_animation() {
        let mut state = State {
            status: Status::Open,
            is_animated: false,
        };
        let action = state.update(Message::Close);
        assert_eq!(state.status, Status::Closed);
        assert_eq!(action, None);
    }

    /// Closing an animated dialog should lead to [`Status::Closing`] state first,
    /// then to [`Status::Closed`] once the closing animation is signaled
    /// with [`Message::Closed`].
    #[test]
    fn update_close_animated() {
        let mut state = State {
            status: Status::Open,
            is_animated: true,
        };
        let action = state.update(Message::Close);
        assert_eq!(state.status, Status::Closing);
        assert_eq!(action, None);

        let action = state.update(Message::Closed);
        assert_eq!(state.status, Status::Closed);
        assert_eq!(action, Some(Action::Closed));
    }
}

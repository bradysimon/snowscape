//! A composable dialog wrapper that can show modal content over a base element.
//!
//! This combines a `Widget` impl with normal Elements used within the dialog to
//! avoid making the widget re-implement core things like buttons. The `Dialog`
//! struct is the user-facing API, which then gets converted to a `ResolvedDialog`
//! in the `From<Dialog> for Element` implementation.
use iced::{
    Alignment, Animation, Color, Element, Event,
    Length::Fill,
    Rectangle, Size, Theme, Vector,
    advanced::{
        Layout, Shell,
        layout::{Limits, Node},
        mouse, overlay,
        renderer::{self, Quad},
        widget::{Operation, Tree, tree},
    },
    animation, keyboard,
    time::{Duration, Instant},
    touch,
    widget::{button, column, container, mouse_area, row, space, text},
    window,
};

/// Creates a new [`Dialog`] wrapping the given `base` and `content` widgets.
pub fn dialog<'a, Message, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Dialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    Dialog::new(base, content)
}

/// A composable dialog wrapper that can show modal content over a base element.
pub struct Dialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    /// The underlying content that remains visible behind the dialog.
    base: Element<'a, Message, Theme, Renderer>,
    /// Optional body content shown inside the dialog panel.
    ///
    /// This is consumed during conversion to `Element` when runtime parts are built.
    content: Option<Element<'a, Message, Theme, Renderer>>,
    /// Whether the dialog should be shown.
    open: bool,
    /// Message published when the dialog requests dismissal.
    on_close: Option<Message>,
    /// Optional title shown in the dialog header.
    title: Option<String>,
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
    /// Creates a new [`Dialog`] with the given base and content widgets.
    pub fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            base: base.into(),
            content: Some(content.into()),
            open: true,
            on_close: None,
            title: None,
            backdrop_close: true,
            esc_close: true,
            animate: true,
            actions: Vec::new(),
        }
    }

    /// Sets whether the dialog is open.
    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Sets the message published when the dialog should close.
    #[must_use]
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets an optional dialog title shown in the header.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
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

    /// Enables or disables dialog animations.
    #[must_use]
    pub fn animate(mut self, enabled: bool) -> Self {
        self.animate = enabled;
        self
    }

    /// Replaces content with an optional value.
    #[must_use]
    pub fn content_maybe<T>(mut self, content: Option<T>) -> Self
    where
        T: Into<Element<'a, Message, Theme, Renderer>>,
    {
        self.content = content.map(Into::into);
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
        let close_button: Element<'a, Message, Theme, Renderer> =
            if let Some(message) = close_message.clone() {
                button(
                    row![
                        crate::icon::xmark()
                            .width(14)
                            .height(14)
                            .style(crate::style::svg::text),
                        text("Close").size(14),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding(6)
                .style(crate::style::button::ghost_subtle)
                .on_press(message)
                .into()
            } else {
                button(
                    row![
                        crate::icon::xmark()
                            .width(14)
                            .height(14)
                            .style(crate::style::svg::text),
                        text("Close").size(14),
                    ]
                    .spacing(6)
                    .align_y(Alignment::Center),
                )
                .padding(6)
                .style(crate::style::button::ghost_subtle)
                .into()
            };

        let title: Element<'a, Message, Theme, Renderer> = if let Some(title) = title_text {
            text(title).size(18).into()
        } else {
            space::horizontal().into()
        };

        let header = row![title, space::horizontal(), close_button].align_y(Alignment::Center);

        let body = column![header, content].spacing(12);
        let body = if actions.is_empty() {
            body
        } else {
            body.push(row(actions).spacing(8).align_y(Alignment::Center))
        };

        let panel: Element<'a, Message, Theme, Renderer> = container(body)
            .padding(16)
            .max_width(500)
            .width(Fill)
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
            on_close,
            title,
            backdrop_close,
            esc_close,
            animate,
            actions,
            ..
        } = dialog;

        let Some(content) = content else {
            return base;
        };

        let (backdrop_target, panel) = Dialog::build_overlay_parts(
            content,
            title.clone(),
            actions,
            on_close.clone(),
            backdrop_close,
        );

        Element::new(ResolvedDialog {
            base,
            backdrop_target,
            panel,
            open,
            on_close,
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
    on_close: Option<Message>,
    esc_close: bool,
    animate: bool,
}

struct State {
    visibility: Animation<bool>,
    now: Instant,
}

impl State {
    fn new() -> Self {
        Self {
            visibility: Animation::new(false)
                .duration(Duration::from_millis(500))
                .easing(animation::Easing::EaseInOutBack),
            now: Instant::now(),
        }
    }
}

impl<'a, Message, Renderer> ResolvedDialog<'a, Message, Renderer>
where
    Message: Clone + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    /// Returns the current progress of the dialog animation, from `0.0` to `1.0`.
    fn progress(&self, state: &State) -> f32 {
        if self.animate {
            state.visibility.interpolate(0.0, 1.0, state.now)
        } else if self.open {
            1.0
        } else {
            0.0
        }
    }

    /// Returns whether the dialog is currently showing or animating.
    fn is_showing(&self, state: &State) -> bool {
        let progress = self.progress(state);
        progress > 0.0 || self.open || (self.animate && state.visibility.is_animating(state.now))
    }
}

impl<Message, Renderer> iced::advanced::Widget<Message, Theme, Renderer>
    for ResolvedDialog<'_, Message, Renderer>
where
    Message: Clone,
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
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
        let state = tree.state.downcast_mut::<State>();

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
            let panel_press = matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerPressed { .. })
            ) && cursor.is_over(panel_layout.bounds());

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
                && let Some(message) = &self.on_close
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

            if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_)))
                && !shell.is_event_captured()
            {
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
        let state = tree.state.downcast_ref::<State>();

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
        let state = tree.state.downcast_ref::<State>();

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
        let _base_layout = children.next()?;
        let _backdrop_layout = children.next()?;
        let panel_layout = children.next()?;

        self.panel.as_widget_mut().overlay(
            &mut tree.children[2],
            panel_layout,
            renderer,
            viewport,
            translation,
        )
    }
}

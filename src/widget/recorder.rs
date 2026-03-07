//! A recorder widget that captures user interactions.
//!
//! This widget wraps content and emits [`Interaction`] messages when the user
//! interacts with the content (mouse clicks, key presses, etc).
//!
//! Primarily follows https://github.com/iced-rs/iced/blob/master/tester/src/recorder.rs

use iced::{
    Color, Element, Event, Length, Point, Rectangle, Size, Vector,
    advanced::{
        Layout, Shell, layout, mouse, overlay, renderer,
        widget::{self, Operation, Tree, operation, tree},
    },
    theme,
};
use iced_test::{
    Selector,
    instruction::{Interaction, Mouse, Target},
    selector,
};

/// A widget that records user interactions.
pub struct Recorder<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    on_record: Option<Box<dyn Fn(Interaction) -> Message + 'a>>,
    has_overlay: bool,
}

impl<'a, Message, Theme, Renderer> Recorder<'a, Message, Theme, Renderer> {
    /// Creates a new [`Recorder`] wrapping the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            on_record: None,
            has_overlay: false,
        }
    }

    /// Sets the callback to invoke when an interaction is recorded.
    pub fn on_record(mut self, on_record: impl Fn(Interaction) -> Message + 'a) -> Self {
        self.on_record = Some(Box::new(on_record));
        self
    }
}

/// Creates a new [`Recorder`] wrapping the given content.
pub fn recorder<'a, Message, Theme, Renderer>(
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Recorder<'a, Message, Theme, Renderer> {
    Recorder::new(content)
}

struct State {
    last_hovered: Option<Rectangle>,
    last_hovered_overlay: Option<Rectangle>,
}

impl<Message, Theme, Renderer> widget::Widget<Message, Theme, Renderer>
    for Recorder<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: theme::Base,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            last_hovered: None,
            last_hovered_overlay: None,
        })
    }

    fn children(&self) -> Vec<widget::Tree> {
        vec![widget::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if shell.is_event_captured() {
            return;
        }

        // Only record from widget if there's no overlay
        if !self.has_overlay
            && let Some(on_record) = &self.on_record
        {
            let state = tree.state.downcast_mut::<State>();

            record(
                event,
                cursor,
                shell,
                layout.bounds(),
                &mut state.last_hovered,
                on_record,
                |operation| {
                    self.content.as_widget_mut().operate(
                        &mut tree.children[0],
                        layout,
                        renderer,
                        operation,
                    );
                },
            );
        }

        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        );
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );

        // Draw highlight overlay for hovered elements
        let state = tree.state.downcast_ref::<State>();
        if let Some(last_hovered) = &state.last_hovered {
            renderer.with_layer(*viewport, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: *last_hovered,
                        ..renderer::Quad::default()
                    },
                    highlight(theme).scale_alpha(0.7),
                );
            });
        }
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.has_overlay = false;

        self.content
            .as_widget_mut()
            .overlay(
                &mut tree.children[0],
                layout,
                renderer,
                &layout.bounds(),
                translation,
            )
            .map(|raw| {
                self.has_overlay = true;

                let state = tree.state.downcast_mut::<State>();

                overlay::Element::new(Box::new(Overlay {
                    raw,
                    bounds: layout.bounds(),
                    last_hovered: &mut state.last_hovered_overlay,
                    on_record: self.on_record.as_deref(),
                }))
            })
    }
}

impl<'a, Message, Theme, Renderer> From<Recorder<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: theme::Base + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(recorder: Recorder<'a, Message, Theme, Renderer>) -> Self {
        Self::new(recorder)
    }
}

/// Overlay wrapper that also records interactions.
struct Overlay<'a, Message, Theme, Renderer> {
    raw: overlay::Element<'a, Message, Theme, Renderer>,
    bounds: Rectangle,
    last_hovered: &'a mut Option<Rectangle>,
    on_record: Option<&'a dyn Fn(Interaction) -> Message>,
}

impl<Message, Theme, Renderer> iced::advanced::Overlay<Message, Theme, Renderer>
    for Overlay<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: theme::Base,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        self.raw.as_overlay_mut().layout(renderer, bounds)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.raw
            .as_overlay()
            .draw(renderer, theme, style, layout, cursor);

        // Draw highlight overlay for hovered elements
        if let Some(last_hovered) = &self.last_hovered {
            renderer.with_layer(self.bounds, |renderer| {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: *last_hovered,
                        ..renderer::Quad::default()
                    },
                    highlight(theme).scale_alpha(0.7),
                );
            });
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.raw
            .as_overlay_mut()
            .operate(layout, renderer, operation);
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        if shell.is_event_captured() {
            return;
        }

        if let Some(on_record) = &self.on_record {
            record(
                event,
                cursor,
                shell,
                self.bounds,
                self.last_hovered,
                on_record,
                |operation| {
                    self.raw
                        .as_overlay_mut()
                        .operate(layout, renderer, operation);
                },
            );
        }

        self.raw
            .as_overlay_mut()
            .update(event, layout, cursor, renderer, shell);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.raw
            .as_overlay()
            .mouse_interaction(layout, cursor, renderer)
    }

    fn overlay<'b>(
        &'b mut self,
        layout: Layout<'b>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.raw
            .as_overlay_mut()
            .overlay(layout, renderer)
            .map(|raw| {
                overlay::Element::new(Box::new(Overlay {
                    raw,
                    bounds: self.bounds,
                    last_hovered: self.last_hovered,
                    on_record: self.on_record,
                }))
            })
    }

    fn index(&self) -> f32 {
        self.raw.as_overlay().index()
    }
}

/// Records an interaction from an event, if applicable.
fn record<Message>(
    event: &Event,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    bounds: Rectangle,
    last_hovered: &mut Option<Rectangle>,
    on_record: impl Fn(Interaction) -> Message,
    operate: impl FnMut(&mut dyn widget::Operation),
) {
    // Only process mouse events if cursor is over bounds
    if let Event::Mouse(_) = event
        && !cursor.is_over(bounds)
    {
        return;
    }

    // Convert cursor position to be relative to content bounds
    let interaction = if let Event::Mouse(mouse::Event::CursorMoved { position }) = event {
        Interaction::from_event(&Event::Mouse(mouse::Event::CursorMoved {
            position: *position - (bounds.position() - Point::ORIGIN),
        }))
    } else {
        Interaction::from_event(event)
    };

    let Some(mut interaction) = interaction else {
        return;
    };

    // Try to find text under cursor for smarter targeting
    let Interaction::Mouse(mouse) = &mut interaction else {
        // Non-mouse interaction (e.g., keyboard), publish as-is
        shell.publish(on_record(interaction));
        return;
    };

    // Get the target from the mouse interaction, or try to determine it from cursor position
    let target = match mouse {
        Mouse::Move(target)
        | Mouse::Press {
            target: Some(target),
            ..
        }
        | Mouse::Release {
            target: Some(target),
            ..
        }
        | Mouse::Click {
            target: Some(target),
            ..
        } => target,
        Mouse::Press {
            target: target @ None,
            ..
        }
        | Mouse::Release {
            target: target @ None,
            ..
        }
        | Mouse::Click {
            target: target @ None,
            ..
        } => {
            // For press/release/click without a target, try to find one from cursor position
            if let Some(position) = cursor.position() {
                let relative_position = position - (bounds.position() - Point::ORIGIN);
                *target = Some(Target::Point(relative_position));
                target.as_mut().unwrap()
            } else {
                // No cursor position, can't determine target
                shell.publish(on_record(interaction));
                return;
            }
        }
    };

    let Target::Point(position) = *target else {
        shell.publish(on_record(interaction));
        return;
    };

    // Prefer widget ids, then unique text, for more robust targeting.
    if let Some((better_target, visible_bounds)) =
        find_target(position + (bounds.position() - Point::ORIGIN), operate)
    {
        *target = better_target;
        *last_hovered = visible_bounds;
    } else {
        *last_hovered = None;
    }

    shell.publish(on_record(interaction));
}

/// Finds a stable target at a given position using widget operations.
fn find_target(
    position: Point,
    mut operate: impl FnMut(&mut dyn widget::Operation),
) -> Option<(Target, Option<Rectangle>)> {
    let mut by_position = position.find_all();
    operate(&mut operation::black_box(&mut by_position));

    let operation::Outcome::Some(targets) = by_position.finish() else {
        return None;
    };

    if let Some((id, visible_bounds)) = targets.iter().rev().find_map(|target| {
        target_id(target).map(|id| (Target::Id(id), target_visible_bounds(target)))
    }) {
        return Some((id, visible_bounds));
    }

    let (content, visible_bounds) = targets.into_iter().rev().find_map(|target| {
        if let selector::Target::Text {
            content,
            visible_bounds,
            ..
        }
        | selector::Target::TextInput {
            content,
            visible_bounds,
            ..
        } = target
        {
            Some((content, visible_bounds))
        } else {
            None
        }
    })?;

    // Check if this text is unique
    let mut by_text = content.clone().find_all();
    operate(&mut operation::black_box(&mut by_text));

    let operation::Outcome::Some(texts) = by_text.finish() else {
        return None;
    };

    // Only use text target if it's unique
    if texts.len() > 1 {
        return None;
    }

    Some((Target::Text(content), visible_bounds))
}

fn target_id(target: &selector::Target) -> Option<String> {
    match target {
        selector::Target::Container { id, .. }
        | selector::Target::Focusable { id, .. }
        | selector::Target::Scrollable { id, .. }
        | selector::Target::TextInput { id, .. }
        | selector::Target::Text { id, .. }
        | selector::Target::Custom { id, .. } => id.as_ref().and_then(custom_widget_id),
    }
}

fn custom_widget_id(id: &iced::widget::Id) -> Option<String> {
    let debug = format!("{id:?}");
    let prefix = "Id(Custom(\"";
    let suffix = "\"))";

    debug
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(suffix))
        .map(str::to_owned)
}

fn target_visible_bounds(target: &selector::Target) -> Option<Rectangle> {
    match target {
        selector::Target::Container { visible_bounds, .. }
        | selector::Target::Focusable { visible_bounds, .. }
        | selector::Target::Scrollable { visible_bounds, .. }
        | selector::Target::TextInput { visible_bounds, .. }
        | selector::Target::Text { visible_bounds, .. }
        | selector::Target::Custom { visible_bounds, .. } => *visible_bounds,
    }
}

/// Returns the highlight color for hovered elements.
fn highlight(theme: &impl theme::Base) -> Color {
    theme
        .palette()
        .map(|palette| palette.primary)
        .unwrap_or(Color::from_rgb(0.0, 0.0, 1.0))
}

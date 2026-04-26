//! Widget tree representation and collection for automation tests.
//!
//! This module provides [`WidgetNode`] and [`WidgetKind`] for capturing
//! a snapshot of the rendered widget hierarchy, as well as the internal
//! [`TreeCollector`] operation that populates the tree from iced's
//! [`widget::Operation`] trait.
//!
//! I hope all of this can be replaced with something built into iced in the future,
//! but for now this will do. For now, we have to infer layout types like `Column` and
//! `Row` from the geometry of child widgets since iced doesn't report them directly.

use std::cmp::Ordering;
use std::fmt;

use iced_test::core::{Rectangle, widget};

// MARK: - Widget kind

/// The kind of widget encountered during a tree traversal.
#[derive(Debug, Clone, PartialEq)]
pub enum WidgetKind {
    /// A column layout (children arranged vertically).
    Column,
    /// A row layout (children arranged horizontally).
    Row,
    /// A stack layout (children overlapping).
    Stack,
    /// A generic container widget.
    Container,
    /// A focusable widget (e.g. `Button`).
    Focusable,
    /// A scrollable container.
    Scrollable,
    /// A text input field.
    TextInput,
    /// A static text label.
    Text,
    /// A custom or unrecognised widget.
    Custom,
}

impl fmt::Display for WidgetKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WidgetKind::Column => write!(f, "Column"),
            WidgetKind::Row => write!(f, "Row"),
            WidgetKind::Stack => write!(f, "Stack"),
            WidgetKind::Container => write!(f, "Container"),
            WidgetKind::Focusable => write!(f, "Focusable"),
            WidgetKind::Scrollable => write!(f, "Scrollable"),
            WidgetKind::TextInput => write!(f, "TextInput"),
            WidgetKind::Text => write!(f, "Text"),
            WidgetKind::Custom => write!(f, "Custom"),
        }
    }
}

// MARK: - Widget node

/// A node in a widget tree, representing a single widget.
#[derive(Debug, Clone)]
pub struct WidgetNode {
    /// The kind of widget.
    pub kind: WidgetKind,
    /// The widget's id, if one was assigned.
    pub id: Option<String>,
    /// Layout bounds of the widget.
    pub bounds: Rectangle,
    /// Text content for [`WidgetKind::Text`] and [`WidgetKind::TextInput`].
    pub text: Option<String>,
    /// Whether the widget is focused (only meaningful for [`WidgetKind::Focusable`]
    /// and [`WidgetKind::TextInput`]).
    pub focused: bool,
    /// Whether the widget has `visible_bounds` within the viewport.
    pub visible: bool,
    /// Child nodes.
    pub children: Vec<WidgetNode>,
}

impl WidgetNode {
    /// Returns `true` if this node or any descendant matches `predicate`.
    pub fn contains(&self, predicate: impl Fn(&WidgetNode) -> bool) -> bool {
        fn go(node: &WidgetNode, predicate: &impl Fn(&WidgetNode) -> bool) -> bool {
            predicate(node) || node.children.iter().any(|c| go(c, predicate))
        }
        go(self, &predicate)
    }

    /// Counts nodes (including this one) that satisfy `predicate`.
    pub fn count(&self, predicate: impl Fn(&WidgetNode) -> bool) -> usize {
        fn go(node: &WidgetNode, predicate: &impl Fn(&WidgetNode) -> bool) -> usize {
            let mut total = if predicate(node) { 1 } else { 0 };
            for child in &node.children {
                total += go(child, predicate);
            }
            total
        }
        go(self, &predicate)
    }

    fn fmt_tree(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefix: &str,
        is_last: bool,
        is_root: bool,
    ) -> fmt::Result {
        // Print the connector and node label.
        if is_root {
            write!(f, "{}", self.kind)?;
        } else {
            let connector = if is_last { "└── " } else { "├── " };
            write!(f, "{prefix}{connector}{}", self.kind)?;
        }
        if let Some(id) = &self.id {
            write!(f, " #{id}")?;
        }
        if let Some(text) = &self.text {
            // Truncate long text for readability (counted in chars, not bytes).
            let char_count = text.chars().count();
            if char_count > 40 {
                let truncated: String = text.chars().take(37).collect();
                write!(f, " \"{truncated}...\"")?;
            } else {
                write!(f, " \"{text}\"")?;
            }
        }
        if self.focused {
            write!(f, " [focused]")?;
        }
        if !self.visible {
            write!(f, " [hidden]")?;
        }
        writeln!(f)?;

        // Recurse into children with updated prefix.
        let child_prefix = if is_root {
            String::new()
        } else if is_last {
            format!("{prefix}    ")
        } else {
            format!("{prefix}│   ")
        };
        let count = self.children.len();
        for (i, child) in self.children.iter().enumerate() {
            child.fmt_tree(f, &child_prefix, i == count - 1, false)?;
        }
        Ok(())
    }
}

impl fmt::Display for WidgetNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt_tree(f, "", true, true)
    }
}

// MARK: - Tree collector

/// An [`Operation`](widget::Operation) that collects the full widget tree
/// into a [`WidgetNode`] hierarchy.
pub(super) struct TreeCollector {
    /// Stack of parent nodes. The last entry is the current parent being
    /// populated by emit calls (`container`, `text`, etc.).
    stack: Vec<WidgetNode>,
    viewport: Rectangle,
}

impl TreeCollector {
    pub fn new(viewport: Rectangle) -> Self {
        // The root is a synthetic container that holds top-level children.
        let root = WidgetNode {
            kind: WidgetKind::Container,
            id: None,
            bounds: viewport,
            text: None,
            focused: false,
            visible: true,
            children: Vec::new(),
        };
        Self {
            stack: vec![root],
            viewport,
        }
    }

    fn push_node(&mut self, node: WidgetNode) {
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(node);
        }
    }

    fn id_string(id: Option<&widget::Id>) -> Option<String> {
        id.map(|id| {
            let debug = format!("{id:?}");
            debug
                .strip_prefix("Id(\"")
                .and_then(|s| s.strip_suffix("\")"))
                .unwrap_or(&debug)
                .to_owned()
        })
    }

    fn is_visible(&self, bounds: Rectangle) -> bool {
        self.viewport.intersection(&bounds).is_some()
    }

    pub fn into_root(mut self) -> WidgetNode {
        // Flatten any unfinished frames back onto the root.
        while self.stack.len() > 1 {
            let child = self.stack.pop().unwrap();
            if let Some(parent) = self.stack.last_mut() {
                parent.children.push(child);
            }
        }
        self.stack.pop().unwrap_or_else(|| WidgetNode {
            kind: WidgetKind::Container,
            id: None,
            bounds: self.viewport,
            text: None,
            focused: false,
            visible: true,
            children: Vec::new(),
        })
    }
}

impl widget::Operation for TreeCollector {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn widget::Operation)) {
        // Iced calls one of the emit methods (e.g. `container`) for the
        // current widget *before* `traverse`, so that node is the last child
        // of the current parent. Move it onto the stack so descendants
        // accumulate inside it. If for some reason no node was emitted
        // (e.g. a transparent wrapper), push a synthetic Container so the
        // hierarchy depth is preserved.
        let synthetic = match self.stack.last_mut().and_then(|p| p.children.pop()) {
            Some(last_child) => {
                self.stack.push(last_child);
                false
            }
            None => {
                self.stack.push(WidgetNode {
                    kind: WidgetKind::Container,
                    id: None,
                    bounds: self.viewport,
                    text: None,
                    focused: false,
                    visible: true,
                    children: Vec::new(),
                });
                true
            }
        };

        operate(self);

        let frame = self.stack.pop().expect("traverse pushed a frame");
        if let Some(parent) = self.stack.last_mut() {
            // Drop empty synthetic frames so the tree stays clean.
            if !(synthetic && frame.children.is_empty()) {
                parent.children.push(frame);
            }
        }
    }

    fn container(&mut self, id: Option<&widget::Id>, bounds: Rectangle) {
        self.push_node(WidgetNode {
            kind: WidgetKind::Container,
            id: Self::id_string(id),
            bounds,
            text: None,
            focused: false,
            visible: self.is_visible(bounds),
            children: Vec::new(),
        });
    }

    fn focusable(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        state: &mut dyn widget::operation::Focusable,
    ) {
        self.push_node(WidgetNode {
            kind: WidgetKind::Focusable,
            id: Self::id_string(id),
            bounds,
            text: None,
            focused: state.is_focused(),
            visible: self.is_visible(bounds),
            children: Vec::new(),
        });
    }

    fn scrollable(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: iced_test::core::Vector,
        _state: &mut dyn widget::operation::Scrollable,
    ) {
        self.push_node(WidgetNode {
            kind: WidgetKind::Scrollable,
            id: Self::id_string(id),
            bounds,
            text: None,
            focused: false,
            visible: self.is_visible(bounds),
            children: Vec::new(),
        });
    }

    fn text_input(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        state: &mut dyn widget::operation::TextInput,
    ) {
        self.push_node(WidgetNode {
            kind: WidgetKind::TextInput,
            id: Self::id_string(id),
            bounds,
            text: Some(state.text().to_owned()),
            focused: false,
            visible: self.is_visible(bounds),
            children: Vec::new(),
        });
    }

    fn text(&mut self, id: Option<&widget::Id>, bounds: Rectangle, content: &str) {
        self.push_node(WidgetNode {
            kind: WidgetKind::Text,
            id: Self::id_string(id),
            bounds,
            text: Some(content.to_owned()),
            focused: false,
            visible: self.is_visible(bounds),
            children: Vec::new(),
        });
    }

    fn custom(
        &mut self,
        id: Option<&widget::Id>,
        bounds: Rectangle,
        _state: &mut dyn std::any::Any,
    ) {
        self.push_node(WidgetNode {
            kind: WidgetKind::Custom,
            id: Self::id_string(id),
            bounds,
            text: None,
            focused: false,
            visible: self.is_visible(bounds),
            children: Vec::new(),
        });
    }

    fn finish(&self) -> widget::operation::Outcome<()> {
        widget::operation::Outcome::None
    }
}

// MARK: - Post-processing

/// Merges redundant [`WidgetKind::Focusable`] nodes into an adjacent
/// [`WidgetKind::TextInput`] sibling that shares the same bounds or id.
///
/// Iced's `TextInput` widget reports itself via *both* `text_input()` and
/// `focusable()`, producing two sibling nodes for one logical widget. We
/// absorb the focus state into the `TextInput` and drop the duplicate.
pub(super) fn deduplicate_focusables(node: &mut WidgetNode) {
    for child in &mut node.children {
        deduplicate_focusables(child);
    }

    // Collect indices of Focusable nodes that can be merged into a TextInput.
    let mut remove = Vec::new();
    for i in 0..node.children.len() {
        if node.children[i].kind != WidgetKind::Focusable {
            continue;
        }
        // Extract match criteria before taking a mutable borrow.
        let focusable_id = node.children[i].id.clone();
        let focusable_bounds = node.children[i].bounds;
        let focusable_focused = node.children[i].focused;
        // Look for a TextInput sibling with matching id or bounds. We require
        // an explicit id match when both sides have ids; otherwise we fall
        // back to bounds equality so unidentified focusables are not merged
        // with arbitrary unidentified text inputs.
        let partner = node.children.iter_mut().enumerate().find(|(j, sibling)| {
            if *j == i || sibling.kind != WidgetKind::TextInput {
                return false;
            }
            match (&focusable_id, &sibling.id) {
                (Some(a), Some(b)) => a == b,
                _ => sibling.bounds == focusable_bounds,
            }
        });
        if let Some((_j, text_input)) = partner {
            // Merge focus state into the TextInput.
            if focusable_focused {
                text_input.focused = true;
            }
            remove.push(i);
        }
    }

    // Remove absorbed Focusable nodes in reverse order to keep indices valid.
    for i in remove.into_iter().rev() {
        node.children.remove(i);
    }
}

/// Recursively reclassifies [`WidgetKind::Container`] nodes as
/// [`Column`](WidgetKind::Column), [`Row`](WidgetKind::Row), or
/// [`Stack`](WidgetKind::Stack) based on how their children are laid out.
///
/// The iced [`Operation`](widget::Operation) trait reports `Column`, `Row`,
/// `Button`, `Stack`, and `Tooltip` identically via `container()`, so we
/// infer the likely type from the spatial arrangement of child bounds.
///
/// Ideally, iced would have a way to report this directly so we don't have to guess.
pub(super) fn reclassify_containers(node: &mut WidgetNode) {
    for child in &mut node.children {
        reclassify_containers(child);
    }

    if node.kind == WidgetKind::Container && node.children.len() >= 2 {
        node.kind = infer_layout_kind(&node.children);
    }
}

/// Infers whether a multi-child container is a `Column`, `Row`, or `Stack`
/// based on the bounding boxes of its children.
///
/// For wrapped rows, children flow left-to-right (or right-to-left) and
/// wrap onto new lines, so we group them by visual row and check that
/// each row is arranged horizontally and rows are stacked vertically.
fn infer_layout_kind(children: &[WidgetNode]) -> WidgetKind {
    debug_assert!(children.len() >= 2);

    // Small tolerance for floating-point layout rounding.
    const TOLERANCE: f32 = 1.0;

    let bounds: Vec<Rectangle> = children.iter().map(|c| c.bounds).collect();

    // Check for a pure column: children are non-overlapping vertically
    // (in any order).
    let mut ys: Vec<(f32, f32)> = bounds.iter().map(|b| (b.y, b.y + b.height)).collect();
    ys.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    if ys.windows(2).all(|pair| pair[1].0 >= pair[0].1 - TOLERANCE) {
        return WidgetKind::Column;
    }

    // Check for a pure row: children are non-overlapping horizontally
    // (in any order, supporting both LTR and RTL).
    let mut xs: Vec<(f32, f32)> = bounds.iter().map(|b| (b.x, b.x + b.width)).collect();
    xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

    if xs.windows(2).all(|pair| pair[1].0 >= pair[0].1 - TOLERANCE) {
        return WidgetKind::Row;
    }

    // Check for a wrapped row: group children into visual rows by
    // overlapping vertical extent, then verify each row is horizontal
    // and the rows themselves are stacked vertically.
    if let Some(kind) = detect_wrapped_row(&bounds, TOLERANCE) {
        return kind;
    }

    // Children overlap in both dimensions - likely a Stack.
    WidgetKind::Stack
}

/// Tries to detect a wrapped row/column pattern.
///
/// Groups children into visual "lines" that share a vertical band,
/// checks that items within each line are arranged horizontally and
/// that the lines themselves stack vertically.
fn detect_wrapped_row(bounds: &[Rectangle], tolerance: f32) -> Option<WidgetKind> {
    // Sort children by top edge, then left edge.
    let mut sorted: Vec<&Rectangle> = bounds.iter().collect();
    sorted.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap_or(Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(Ordering::Equal))
    });

    // Group into rows: a child starts a new row when its top edge is
    // at or past the bottom of the current row's tallest member.
    let mut rows: Vec<Vec<&Rectangle>> = Vec::new();
    let mut row_bottom = f32::NEG_INFINITY;

    for &b in &sorted {
        if b.y >= row_bottom - tolerance {
            rows.push(vec![b]);
            row_bottom = b.y + b.height;
        } else {
            rows.last_mut().unwrap().push(b);
            row_bottom = row_bottom.max(b.y + b.height);
        }
    }

    // Need at least 2 rows for it to be a wrapped pattern, and at
    // least one row must have multiple items.
    if rows.len() < 2 || rows.iter().all(|r| r.len() < 2) {
        return None;
    }

    // Verify each row's items are non-overlapping horizontally.
    for row in &rows {
        if row.len() >= 2 {
            let mut xs: Vec<(f32, f32)> = row.iter().map(|b| (b.x, b.x + b.width)).collect();
            xs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
            if !xs.windows(2).all(|pair| pair[1].0 >= pair[0].1 - tolerance) {
                return None;
            }
        }
    }

    // Verify the rows themselves are stacked vertically.
    let row_extents: Vec<(f32, f32)> = rows
        .iter()
        .map(|row| {
            let top = row.iter().map(|b| b.y).fold(f32::INFINITY, f32::min);
            let bottom = row
                .iter()
                .map(|b| b.y + b.height)
                .fold(f32::NEG_INFINITY, f32::max);
            (top, bottom)
        })
        .collect();

    if row_extents
        .windows(2)
        .all(|pair| pair[1].0 >= pair[0].1 - tolerance)
    {
        Some(WidgetKind::Row)
    } else {
        None
    }
}

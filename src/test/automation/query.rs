//! Queries and assertions for the [`Emulator`].

use iced_test::Selector;
use iced_test::core::{Rectangle, widget};
use iced_test::runtime::UserInterface;
use iced_test::runtime::user_interface::Cache;

use super::widget_tree::{
    TreeCollector, WidgetNode, deduplicate_focusables, reclassify_containers,
};
use super::{Emulator, Error, Result, select};

impl<P: iced_test::program::Program + 'static> Emulator<P> {
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

    pub(super) fn try_find<S>(&mut self, selector: S) -> Option<S::Output>
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

    fn try_find_all<S>(&mut self, selector: S) -> Vec<S::Output>
    where
        S: Selector + Send,
        S::Output: Clone + Send,
    {
        use widget::Operation;

        let element = self.inner.view(&self.program);
        let mut ui = UserInterface::build(element, self.size, Cache::default(), &mut self.renderer);

        let mut operation = selector.find_all();
        ui.operate(
            &self.renderer,
            &mut widget::operation::black_box(&mut operation),
        );
        let _ = ui.into_cache();

        match operation.finish() {
            widget::operation::Outcome::Some(outputs) => outputs,
            _ => Vec::new(),
        }
    }

    // MARK: - Advanced queries

    /// Returns all matches for the given selector in the current view.
    pub fn find_all<S>(&mut self, selector: S) -> Vec<S::Output>
    where
        S: Selector + Send,
        S::Output: Clone + Send,
    {
        self.try_find_all(selector)
    }

    /// Counts how many widgets match the given selector.
    pub fn count<S>(&mut self, selector: S) -> usize
    where
        S: Selector + Send,
        S::Output: Clone + Send,
    {
        self.try_find_all(selector).len()
    }

    /// Returns whether the widget matched by `selector` has visible bounds
    /// within the current viewport.
    pub fn is_visible<S>(&mut self, selector: S) -> bool
    where
        S: Selector<Output = select::Target> + Send,
    {
        self.try_find(selector)
            .is_some_and(|target| target.visible_bounds().is_some())
    }

    /// Captures a snapshot of the entire widget tree for debugging.
    ///
    /// The returned [`WidgetNode`] can be printed with `Display` to produce
    /// a compact, indented representation.
    pub fn widget_tree(&mut self) -> WidgetNode {
        let element = self.inner.view(&self.program);
        let mut ui = UserInterface::build(element, self.size, Cache::default(), &mut self.renderer);

        let mut op = TreeCollector::new(Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.size.width,
            height: self.size.height,
        });
        ui.operate(&self.renderer, &mut widget::operation::black_box(&mut op));
        let _ = ui.into_cache();
        let mut root = op.into_root();
        deduplicate_focusables(&mut root);
        reclassify_containers(&mut root);
        root
    }

    // MARK: - Assertions

    /// Asserts that the selector matches at least one widget.
    ///
    /// Returns `Err(Error::AssertionFailed)` with a descriptive message if
    /// no match is found.
    pub fn assert_exists<S>(&mut self, selector: S) -> Result
    where
        S: Selector + Send,
        S::Output: Clone + Send,
    {
        let desc = selector.description();
        if self.try_find(selector).is_some() {
            Ok(())
        } else {
            Err(Error::AssertionFailed(format!(
                "expected to find `{desc}`, but no match was found\n\nWidget tree:\n{}",
                self.widget_tree(),
            )))
        }
    }

    /// Asserts that the selector does **not** match any widget.
    pub fn assert_not_exists<S>(&mut self, selector: S) -> Result
    where
        S: Selector + Send,
        S::Output: Clone + Send,
    {
        let desc = selector.description();
        if self.try_find(selector).is_none() {
            Ok(())
        } else {
            Err(Error::AssertionFailed(format!(
                "expected `{desc}` to not exist, but a match was found"
            )))
        }
    }

    /// Asserts that the text content of the widget matched by `selector`
    /// equals `expected`.
    pub fn assert_text<S>(&mut self, selector: S, expected: &str) -> Result
    where
        S: Selector<Output = select::Target> + Send,
    {
        let actual = self.get_text(selector)?;
        if actual == expected {
            Ok(())
        } else {
            Err(Error::AssertionFailed(format!(
                "expected text {expected:?}, got {actual:?}"
            )))
        }
    }

    /// Asserts that exactly `expected` widgets match the given selector.
    pub fn assert_count<S>(&mut self, selector: S, expected: usize) -> Result
    where
        S: Selector + Send,
        S::Output: Clone + Send,
    {
        let actual = self.count(selector);
        if actual == expected {
            Ok(())
        } else {
            Err(Error::AssertionFailed(format!(
                "expected {expected} matches, found {actual}"
            )))
        }
    }
}

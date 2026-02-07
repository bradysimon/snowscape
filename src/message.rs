use std::{any::Any, fmt::Debug};

use iced::{Theme, theme, window};

use crate::{
    config_tab::ConfigTab,
    dynamic::{self},
    test::Interaction,
};

/// Supertrait for messages that can be used in the preview system.
/// - `Any`: Previews support any type of message via downcasting
/// - `Clone`: Messages can be stored in history
/// - `Debug`: Messages can be displayed in the UI
pub trait AnyMessage: Any + Clone + Debug + Send + Sync + 'static {}
impl<T> AnyMessage for T where T: Any + Clone + Debug + Send + Sync + 'static {}

/// Helper trait for cloneable, type-erased messages
pub trait AnyClone: Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn AnyClone>;
    fn as_any(&self) -> &dyn Any;
}

impl<T> AnyClone for T
where
    T: Any + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn AnyClone> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clone for Box<dyn AnyClone> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Message type for the preview system.
pub enum Message {
    /// No-op message.
    Noop,
    /// Focuses the search input.
    FocusInput,
    /// Select a different preview by index.
    SelectPreview(usize),
    /// Resets a stateful preview to its initial state.
    ResetPreview,
    /// Change the search query.
    ChangeSearch(String),
    /// Change a dynamic parameter's value at some index.
    ChangeParam(usize, dynamic::Value),
    /// Resets all dynamic parameters for the current preview to their default values.
    ResetParams,
    /// Time travel to a previous state in a stateful preview's timeline by index.
    TimeTravel(u32),
    /// Jump to the latest state in a stateful preview's timeline.
    JumpToPresent,
    /// Resize the sidebar to the given pixel size.
    ResizeSidebar(f32),
    /// Resize the configuration pane underneath the preview to the given pixel size.
    ResizeConfigPane(f32),
    /// Change the currently selected configuration tab below the preview.
    ChangeConfigTab(ConfigTab),
    /// Updates the current theme.
    UpdateTheme(iced_anim::Event<Theme>),
    /// The theme mode of the system has changed.
    ChangeThemeMode(theme::Mode),
    /// Message from a stateful component (type-erased).
    Component(Box<dyn AnyClone>),

    // Test-related messages
    /// Change the test window width configuration.
    ChangeTestWidth(String),
    /// Change the test window height configuration.
    ChangeTestHeight(String),
    /// Toggle whether to capture a snapshot at the end of the test.
    ToggleTestSnapshot(bool),
    /// Start recording a test for the currently selected preview.
    StartTestRecording,
    /// A test window was opened with the given ID.
    TestWindowOpened(window::Id),
    /// Record an interaction during test recording.
    RecordInteraction(Interaction),
    /// Stop recording and save the test.
    StopTestRecording,
    /// A window was closed.
    WindowClosed(window::Id),
    /// A screenshot was captured for the test.
    TestScreenshotCaptured(iced::window::Screenshot),
    /// Change the text in the expectation input field.
    ChangeExpectText(String),
    /// Add a text expectation to the current recording.
    AddTextExpectation,
    /// Capture a snapshot at the current point in the recording.
    CaptureSnapshot,
    /// Removes the current test session.
    /// Should be done after ending a test recording and saving state.
    RemoveTestSession,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Noop => write!(f, "Noop"),
            Self::FocusInput => write!(f, "FocusInput"),
            Self::SelectPreview(arg0) => f.debug_tuple("SelectPreview").field(arg0).finish(),
            Self::ResetPreview => write!(f, "ResetPreview"),
            Self::ChangeSearch(text) => f.debug_tuple("ChangeSearch").field(text).finish(),
            Self::ChangeParam(arg0, arg1) => f
                .debug_tuple("ChangeParam")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::ResetParams => write!(f, "ResetParams"),
            Self::TimeTravel(arg0) => f.debug_tuple("TimeTravel").field(arg0).finish(),
            Self::JumpToPresent => write!(f, "JumpToPresent"),
            Self::ResizeSidebar(arg0) => f.debug_tuple("ResizePreviewPane").field(arg0).finish(),
            Self::ResizeConfigPane(arg0) => f.debug_tuple("ResizeConfigPane").field(arg0).finish(),
            Self::ChangeConfigTab(arg0) => f.debug_tuple("ChangeConfigTab").field(arg0).finish(),
            Self::UpdateTheme(event) => write!(f, "UpdateTheme({event:?})"),
            Self::ChangeThemeMode(arg0) => f.debug_tuple("ChangeThemeMode").field(arg0).finish(),
            Self::Component(_) => write!(f, "Component(..)"),
            Self::ChangeTestWidth(w) => f.debug_tuple("ChangeTestWidth").field(w).finish(),
            Self::ChangeTestHeight(h) => f.debug_tuple("ChangeTestHeight").field(h).finish(),
            Self::ToggleTestSnapshot(b) => f.debug_tuple("ToggleTestSnapshot").field(b).finish(),
            Self::StartTestRecording => write!(f, "StartTestRecording"),
            Self::TestWindowOpened(id) => f.debug_tuple("TestWindowOpened").field(id).finish(),
            Self::RecordInteraction(i) => f.debug_tuple("RecordInteraction").field(i).finish(),
            Self::StopTestRecording => write!(f, "StopTestRecording"),
            Self::WindowClosed(id) => f.debug_tuple("WindowClosed").field(id).finish(),
            Self::TestScreenshotCaptured(_) => write!(f, "TestScreenshotCaptured(..)"),
            Self::ChangeExpectText(t) => f.debug_tuple("ChangeExpectText").field(t).finish(),
            Self::AddTextExpectation => write!(f, "AddTextExpectation"),
            Self::CaptureSnapshot => write!(f, "CaptureSnapshot"),
            Self::RemoveTestSession => write!(f, "RemoveTestSession"),
        }
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        match self {
            Message::Noop => Message::Noop,
            Message::FocusInput => Message::FocusInput,
            Message::SelectPreview(i) => Message::SelectPreview(*i),
            Message::ResetPreview => Message::ResetPreview,
            Message::ChangeSearch(s) => Message::ChangeSearch(s.clone()),
            Message::ChangeParam(i, v) => Message::ChangeParam(*i, v.clone()),
            Message::ResetParams => Message::ResetParams,
            Message::TimeTravel(t) => Message::TimeTravel(*t),
            Message::JumpToPresent => Message::JumpToPresent,
            Message::ResizeSidebar(f) => Message::ResizeSidebar(*f),
            Message::ResizeConfigPane(f) => Message::ResizeConfigPane(*f),
            Message::ChangeConfigTab(tab) => Message::ChangeConfigTab(*tab),
            Message::UpdateTheme(ev) => Message::UpdateTheme(ev.clone()),
            Message::ChangeThemeMode(mode) => Message::ChangeThemeMode(*mode),
            Message::Component(inner) => {
                // Avoid infinite clone recursion when the payload itself is a `Message`.
                // Instead of calling `inner.clone()` (which invokes `clone_box`, which
                // calls `T::clone`, which re-enters here), we downcast and clone directly.
                // Note: we must deref twice to get the trait object, not the Box.
                if let Some(msg) = (**inner).as_any().downcast_ref::<Message>() {
                    // Clone the inner Message using this same impl (safe recursion).
                    let cloned = msg.clone();
                    Message::Component(Box::new(cloned))
                } else {
                    // Payload is not a Message; safe to use clone_box.
                    Message::Component(inner.clone_box())
                }
            }
            Message::ChangeTestWidth(w) => Message::ChangeTestWidth(w.clone()),
            Message::ChangeTestHeight(h) => Message::ChangeTestHeight(h.clone()),
            Message::ToggleTestSnapshot(b) => Message::ToggleTestSnapshot(*b),
            Message::StartTestRecording => Message::StartTestRecording,
            Message::TestWindowOpened(id) => Message::TestWindowOpened(*id),
            Message::RecordInteraction(i) => Message::RecordInteraction(i.clone()),
            Message::StopTestRecording => Message::StopTestRecording,
            Message::WindowClosed(id) => Message::WindowClosed(*id),
            Message::TestScreenshotCaptured(s) => Message::TestScreenshotCaptured(s.clone()),
            Message::ChangeExpectText(t) => Message::ChangeExpectText(t.clone()),
            Message::AddTextExpectation => Message::AddTextExpectation,
            Message::CaptureSnapshot => Message::CaptureSnapshot,
            Message::RemoveTestSession => Message::RemoveTestSession,
        }
    }
}

impl Message {
    /// Creates a new boxed [`Message::Component`] from any cloneable message.
    pub fn component(message: impl AnyClone) -> Self {
        Self::Component(Box::new(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cloning a Component containing a simple Message should not overflow.
    #[test]
    fn clone_simple_component() {
        let simple = Message::component(Message::Noop);
        let cloned = simple.clone();
        let Message::Component(boxed) = cloned else {
            panic!("Expected Component message");
        };
        let inner = (*boxed).as_any().downcast_ref::<Message>().unwrap();
        assert!(matches!(inner, Message::Noop));
    }

    /// Cloning nested Component messages should not cause stack overflow.
    #[test]
    fn clone_nested_component_does_not_overflow() {
        let nested = Message::component(Message::component(Message::Noop));
        _ = nested.clone();
    }

    /// Cloning nested Components should preserve the message structure.
    #[test]
    fn clone_nested_component_preserves_structure() {
        let nested = Message::component(Message::component(Message::Noop));
        let cloned = nested.clone();

        // First level: Component
        let Message::Component(boxed) = cloned else {
            panic!("Expected Component message at first level");
        };

        // Second level: Component
        let inner = (*boxed).as_any().downcast_ref::<Message>().unwrap().clone();
        let Message::Component(boxed) = inner else {
            panic!("Expected Component message at second level");
        };

        // Third level: Noop
        let inner = (*boxed).as_any().downcast_ref::<Message>().unwrap().clone();
        assert!(matches!(inner, Message::Noop));
    }

    /// Deeply nested Component messages should not cause stack overflow.
    #[test]
    fn clone_deeply_nested_component_does_not_overflow() {
        let deep = Message::component(Message::component(Message::component(Message::Noop)));
        _ = deep.clone();
    }
}

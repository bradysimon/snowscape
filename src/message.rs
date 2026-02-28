use std::{any::Any, fmt::Debug};

use iced::{Theme, theme, window};

use crate::{config_tab::ConfigTab, dynamic, test};

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
    /// A window was closed.
    WindowClosed(window::Id),
    /// Test-related messages.
    Test(test::Message),
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
            Self::ResizeSidebar(arg0) => f.debug_tuple("ResizeSidebar").field(arg0).finish(),
            Self::ResizeConfigPane(arg0) => f.debug_tuple("ResizeConfigPane").field(arg0).finish(),
            Self::ChangeConfigTab(arg0) => f.debug_tuple("ChangeConfigTab").field(arg0).finish(),
            Self::UpdateTheme(event) => write!(f, "UpdateTheme({event:?})"),
            Self::ChangeThemeMode(arg0) => f.debug_tuple("ChangeThemeMode").field(arg0).finish(),
            Self::Component(_) => write!(f, "Component(..)"),
            Self::WindowClosed(id) => f.debug_tuple("WindowClosed").field(id).finish(),
            Self::Test(msg) => f.debug_tuple("Test").field(msg).finish(),
        }
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        /// Recursively clones a component payload, ensuring that nested `Message::Component`
        /// instances are properly cloned without causing stack overflow.
        fn clone_component_payload(payload: &dyn AnyClone) -> Box<dyn AnyClone> {
            if let Some(message) = payload.as_any().downcast_ref::<Message>() {
                Box::new(clone_message(message))
            } else {
                payload.clone_box()
            }
        }

        fn clone_message(message: &Message) -> Message {
            match message {
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
                Message::Component(inner) => Message::Component(clone_component_payload(&**inner)),
                Message::WindowClosed(id) => Message::WindowClosed(*id),
                Message::Test(msg) => Message::Test(msg.clone()),
            }
        }

        clone_message(self)
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

    #[derive(Clone, Debug)]
    enum DialogMessage {
        Open,
        Close,
    }

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

    /// Cloning nested Components with a foreign leaf message should preserve shape.
    /// Helps avoid stack overflows when cloning nested messages in the app preview.
    #[test]
    fn clone_nested_component_with_foreign_leaf_preserves_structure() {
        let nested = Message::component(Message::component(DialogMessage::Open));
        let cloned = nested.clone();

        let Message::Component(level1) = cloned else {
            panic!("Expected first-level Component");
        };

        let inner_message = (*level1)
            .as_any()
            .downcast_ref::<Message>()
            .unwrap()
            .clone();
        let Message::Component(level2) = inner_message else {
            panic!("Expected second-level Component");
        };

        let leaf = (*level2).as_any().downcast_ref::<DialogMessage>().unwrap();
        assert!(matches!(leaf, DialogMessage::Open));
    }

    /// Shouldn't stack overflow when cloning nested components with a foreign leaf message.
    #[test]
    fn clone_foreign_leaf_close_variant_does_not_overflow() {
        let nested = Message::component(Message::component(DialogMessage::Close));
        _ = nested.clone();
    }
}

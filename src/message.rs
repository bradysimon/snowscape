use std::{any::Any, fmt::Debug};

use iced::{Theme, theme};

use crate::config_tab::ConfigTab;

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
#[derive(Clone)]
pub enum Message {
    /// No-op message.
    Noop,
    /// Select a different preview by index.
    SelectPreview(usize),
    /// Change the search query.
    ChangeSearch(String),
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
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Noop => write!(f, "Noop"),
            Self::SelectPreview(arg0) => f.debug_tuple("SelectPreview").field(arg0).finish(),
            Self::ChangeSearch(text) => f.debug_tuple("ChangeSearch").field(text).finish(),
            Self::ResizeSidebar(arg0) => f.debug_tuple("ResizePreviewPane").field(arg0).finish(),
            Self::ResizeConfigPane(arg0) => f.debug_tuple("ResizeConfigPane").field(arg0).finish(),
            Self::ChangeConfigTab(arg0) => f.debug_tuple("ChangeConfigTab").field(arg0).finish(),
            Self::UpdateTheme(event) => write!(f, "UpdateTheme({event:?})"),
            Self::ChangeThemeMode(arg0) => f.debug_tuple("ChangeThemeMode").field(arg0).finish(),
            Self::Component(_) => write!(f, "Component(..)"),
        }
    }
}

use iced::{Theme, theme};

/// Helper trait for cloneable, type-erased messages
pub trait AnyClone: std::any::Any + Send + Sync {
    fn clone_box(&self) -> Box<dyn AnyClone>;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T> AnyClone for T
where
    T: std::any::Any + Clone + Send + Sync,
{
    fn clone_box(&self) -> Box<dyn AnyClone> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
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
    /// Message from the active preview component.
    PreviewComponent,
    /// Change the search query.
    ChangeSearch(String),
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
            Self::PreviewComponent => write!(f, "PreviewComponent"),
            Self::ChangeSearch(text) => f.debug_tuple("ChangeSearch").field(text).finish(),
            Self::UpdateTheme(event) => write!(f, "UpdateTheme({event:?})"),
            Self::ChangeThemeMode(arg0) => f.debug_tuple("ChangeThemeMode").field(arg0).finish(),
            Self::Component(_) => write!(f, "Component(..)"),
        }
    }
}

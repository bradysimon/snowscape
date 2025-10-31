use iced::{Theme, theme};

/// Message type for the preview system.
#[derive(Debug, Clone)]
pub enum Message {
    /// No-op message.
    Noop,
    /// Select a different preview by index.
    SelectPreview(usize),
    /// Message from the active preview component.
    PreviewComponent,
    /// Updates the current theme.
    UpdateTheme(iced_anim::Event<Theme>),
    /// The theme mode of the system has changed.
    ChangeThemeMode(theme::Mode),
}

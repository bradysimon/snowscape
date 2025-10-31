use iced::{Element, Theme, overlay::menu, widget::pick_list};

use crate::message::PreviewMessage;

/// A picker allowing the user to select a theme, emitting a `Message` when the
/// selection changes.
pub fn theme_picker<'a>(theme: Option<Theme>) -> Element<'a, PreviewMessage> {
    pick_list(Theme::ALL, theme, |theme| {
        PreviewMessage::UpdateTheme(theme.into())
    })
    .text_size(14)
    .placeholder("System theme")
    .style(|theme, status| {
        let default = pick_list::default(theme, status);
        pick_list::Style {
            border: default.border.rounded(4),
            ..default
        }
    })
    .menu_style(|theme| {
        let default = menu::default(theme);
        menu::Style {
            border: default.border.rounded(4),
            ..default
        }
    })
    .into()
}

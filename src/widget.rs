pub mod split;

use iced::{
    Alignment::Center,
    Element,
    Length::{Fill, Shrink},
    Theme, border,
    overlay::menu,
    widget::{button, column, container, pick_list, row, space, text, text::IntoFragment},
};
use iced_anim::Animated;

use crate::{
    config_tab::ConfigTab,
    message::Message,
    metadata::Metadata,
    preview::{Descriptor, Preview},
};

/// The theme picker dropdown shown in the header.
pub fn theme_picker<'a>(theme: Option<Theme>) -> Element<'a, Message> {
    pick_list(Theme::ALL, theme, |theme| {
        Message::UpdateTheme(theme.into())
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

/// The header shown above the preview area.
pub fn header<'a>(
    descriptor: Option<&'a Descriptor>,
    theme: &'a Option<Animated<Theme>>,
) -> Element<'a, Message> {
    row![
        descriptor.map(|descriptor| container(text(&descriptor.metadata.label)).width(Fill)),
        space::horizontal(),
        theme_picker(theme.as_ref().map(|t| t.target().clone())),
    ]
    .align_y(Center)
    .padding(10)
    .into()
}

/// The main preview area showing the selected `preview`.
pub fn preview_area(preview: Option<&dyn Preview>) -> Element<'_, Message> {
    container(if let Some(preview) = preview {
        preview.view()
    } else {
        // TODO: Improve placeholder view
        text("No preview selected").into()
    })
    .padding(20)
    .center(Fill)
    .into()
}

/// The configuration pane shown underneath the preview area.
pub fn config_pane(descriptor: &Descriptor, tab: ConfigTab) -> Element<'_, Message> {
    let content = match tab {
        ConfigTab::About => about_config_pane(&descriptor.metadata),
        ConfigTab::Parameters => parameter_config_pane(),
        ConfigTab::Messages => message_config_pane(),
        ConfigTab::Performance => performance_config_pane(),
    };
    container(column![config_tabs(tab), container(content).padding([2, 8])].spacing(4))
        .padding(4)
        .width(Fill)
        .height(Fill)
        .style(|theme: &Theme| {
            container::background(theme.extended_palette().background.weakest.color)
        })
        .into()
}

/// The configuration tabs shown in the configuration pane.
fn config_tabs<'a>(tab: ConfigTab) -> Element<'a, Message> {
    row(ConfigTab::ALL.iter().map(|&variant| {
        let is_selected = variant == tab;
        config_tab(variant, is_selected)
    }))
    .into()
}

/// A tab button used within [`config_tabs`].
fn config_tab<'a>(tab: ConfigTab, selected: bool) -> Element<'a, Message> {
    let label = match tab {
        ConfigTab::About => "About",
        ConfigTab::Parameters => "Parameters",
        ConfigTab::Messages => "Messages",
        ConfigTab::Performance => "Performance",
    };

    button(
        column![
            container(text(label).size(14)).padding([2, 4]),
            container(space::horizontal())
                .width(Fill)
                .height(2)
                .style(move |theme: &Theme| if selected {
                    container::Style {
                        border: border::rounded(1),
                        ..container::background(theme.palette().primary)
                    }
                } else {
                    container::Style::default()
                })
        ]
        .width(Shrink),
    )
    .padding([4, 6])
    .on_press(Message::ChangeConfigTab(tab))
    .style(move |theme: &Theme, status| {
        if selected {
            button::Style {
                text_color: theme.palette().text,
                ..button::text(theme, status)
            }
        } else {
            let alpha = if status == button::Status::Hovered {
                1.0
            } else {
                0.6
            };
            button::Style {
                text_color: theme.palette().text.scale_alpha(alpha),
                ..button::text(theme, status)
            }
        }
    })
    .into()
}

/// A pane shown in the configuration area displaying metadata about the preview.
fn about_config_pane(metadata: &Metadata) -> Element<'_, Message> {
    column![
        row![
            text(&metadata.label).size(18),
            space::horizontal().width(Shrink),
            row(metadata.tags.iter().cloned().map(badge))
                .spacing(4)
                .wrap()
        ]
        .spacing(8)
        .align_y(Center)
        .wrap(),
        space::vertical().height(5),
        if let Some(description) = &metadata.description {
            text(description)
        } else {
            text("No description available.")
        }
        .style(|theme: &Theme| text::Style {
            color: Some(
                theme
                    .extended_palette()
                    .background
                    .weakest
                    .text
                    .scale_alpha(0.75)
            )
        }),
    ]
    .width(Fill)
    .into()
}

/// A small badge that shows some `content` within it.
fn badge<'a>(contenxt: impl IntoFragment<'a>) -> Element<'a, Message> {
    container(text(contenxt).size(14))
        .padding([2, 6])
        .style(|theme: &Theme| container::Style {
            background: Some(theme.extended_palette().background.weak.color.into()),
            border: border::rounded(4),
            ..container::Style::default()
        })
        .into()
}

fn parameter_config_pane<'a>() -> Element<'a, Message> {
    text("Parameters").into()
}

fn message_config_pane<'a>() -> Element<'a, Message> {
    text("Messages").into()
}

fn performance_config_pane<'a>() -> Element<'a, Message> {
    text("Performance").into()
}

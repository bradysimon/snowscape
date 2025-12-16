pub mod about_pane;
pub mod message_pane;
pub mod parameter_pane;
pub mod performance_pane;

use iced::{
    Alignment::Center,
    Element,
    Length::{self, Fill, Shrink},
    Theme, border, padding,
    widget::{button, column, container, responsive, row, slider, space, text},
};

use crate::{
    app::Message,
    config_tab::ConfigTab,
    preview::{Descriptor, Timeline},
    widget::{mini_badge, round_badge},
};

/// The configuration pane shown underneath the preview area.
pub fn config_pane(descriptor: &Descriptor, tab: ConfigTab) -> Element<'_, Message> {
    responsive(move |size| {
        // The main content of the config pane
        let content = match tab {
            ConfigTab::About => about_pane::about_pane(descriptor.metadata()),
            ConfigTab::Parameters => parameter_pane::parameter_pane(descriptor.preview.params()),
            ConfigTab::Messages => message_pane::message_pane(descriptor.preview.as_ref()),
            ConfigTab::Performance => performance_pane::performance_pane(),
        };

        let is_horizontal_layout = size.width >= 675.0;

        // Trailing element shown on the right of the config tabs
        let trailing = match tab {
            ConfigTab::About | ConfigTab::Parameters | ConfigTab::Performance => None,
            ConfigTab::Messages => descriptor
                .preview
                .timeline()
                .map(|timeline| timeline_slider(timeline, !is_horizontal_layout)),
        };

        // The header containing the config tabs and any trailing elements
        let header: Element<'_, Message> = if is_horizontal_layout {
            row![
                config_tabs(
                    tab,
                    descriptor.preview.params().len(),
                    descriptor.preview.message_count()
                ),
                space::horizontal(),
                trailing,
            ]
            .align_y(Center)
            .into()
        } else {
            // Display the config tabs and trailing element vertically on smaller widths
            column![
                config_tabs(
                    tab,
                    descriptor.preview.params().len(),
                    descriptor.preview.message_count()
                ),
                trailing,
            ]
            .into()
        };

        container(column![header, container(content).padding([2, 8]).height(Fill)].spacing(4))
            .padding(4)
            .width(Fill)
            .height(Fill)
            .style(|theme: &Theme| {
                container::background(theme.extended_palette().background.weakest.color)
            })
            .into()
    })
    .into()
}

/// The timeline slider used for time travel in stateful previews.
fn timeline_slider<'a>(timeline: Timeline, fill: bool) -> Element<'a, Message> {
    // Use `1` as a value if the timeline is empty to ensure the slider
    // still shows the slider at the end of the range when empty.
    let (value, range) = if timeline.is_empty() {
        (1, 0..=1)
    } else {
        (timeline.position(), timeline.range())
    };

    row![
        container(mini_badge(format!("{}", timeline.position()))).padding(padding::left(if fill {
            8.0
        } else {
            0.0
        })),
        slider(range, value, Message::TimeTravel).width(if fill {
            Fill
        } else {
            Length::Fixed(200.0)
        }),
        live_button(timeline.is_live()),
    ]
    .align_y(Center)
    .spacing(4)
    .into()
}

/// The "Live" button used to jump to the latest state in the timeline in the [`timeline_slider`].
fn live_button<'a>(is_live: bool) -> Element<'a, Message> {
    const SIZE: u32 = 6;
    button(
        row![
            container(space::horizontal())
                .width(SIZE)
                .height(SIZE)
                .style(move |theme: &Theme| container::Style {
                    background: if is_live {
                        Some(theme.extended_palette().danger.base.color.into())
                    } else {
                        Some(theme.extended_palette().background.neutral.color.into())
                    },
                    border: border::rounded(SIZE / 2),
                    ..Default::default()
                }),
            text("Live").size(14),
        ]
        .align_y(Center)
        .spacing(6),
    )
    .on_press(Message::JumpToPresent)
    .style(button::text)
    .into()
}

/// The configuration tabs shown in the configuration pane.
pub fn config_tabs<'a>(
    selected_tab: ConfigTab,
    params: usize,
    messages: usize,
) -> Element<'a, Message> {
    row(ConfigTab::ALL.iter().map(|&variant| {
        let is_selected = variant == selected_tab;
        config_tab(variant, is_selected, params, messages)
    }))
    .into()
}

/// A tab button used within [`config_tabs`].
fn config_tab<'a>(
    tab: ConfigTab,
    selected: bool,
    params: usize,
    messages: usize,
) -> Element<'a, Message> {
    let badge_info = match tab {
        ConfigTab::Messages if messages > 0 => Some((messages, true)),
        ConfigTab::Parameters if params > 0 => Some((params, false)),
        _ => None,
    };

    button(
        column![
            container(
                row![
                    text(tab.name()).size(14),
                    badge_info.map(|(count, primary)| round_badge(count, primary)),
                ]
                .spacing(4)
                .align_y(Center)
            )
            .padding([2, 4]),
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

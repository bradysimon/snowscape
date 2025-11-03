pub use crate::message::Message;
use crate::{Preview, preview::Descriptor, widget::theme_picker};
use iced::{
    Alignment::Center,
    Border, Element, Subscription, Task, Theme, system,
    theme::{self, Base},
    widget::{rule, space, text_input},
};
use iced_anim::{Animated, Animation, Easing};
use std::time::Duration;

/// The preview app that shows registered previews.
#[derive(Default)]
pub struct App {
    /// A custom title for the application window.
    pub(crate) title: Option<String>,
    /// The current search query that filters previews.
    search: String,
    /// The list of registered previewable elements.
    descriptors: Vec<Descriptor>,
    /// The index of the selected `descriptor` in the list.
    selected_index: Option<usize>,
    /// The theme used by the application.
    theme: Option<Animated<Theme>>,
    /// The initial theme mode used by the application.
    theme_mode: theme::Mode,
}

impl App {
    /// Adds a custom title to the application.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Adds a preview to the application.
    pub fn preview(mut self, preview: impl Into<Descriptor>) -> Self {
        self.descriptors.push(preview.into());
        self
    }

    /// Find a preview by name from the `SNOWSCAPE_PREVIEW` environment variable.
    ///
    /// This first looks for exact matches, and if none are found, it looks for partial matches.
    /// Partial matches may happen when there are multiple stateless previews on the same function.
    fn find_preview_by_env(descriptors: &[Descriptor]) -> usize {
        if let Ok(preview_name) = std::env::var("SNOWSCAPE_PREVIEW") {
            let mut partial_match: Option<usize> = None;
            for (i, descriptor) in descriptors.iter().enumerate() {
                if descriptor.metadata.label == preview_name {
                    return i;
                } else if descriptor.metadata.label.starts_with(&preview_name)
                    && partial_match.is_none()
                {
                    // Try checking for partial starting matches if no exact match is found.
                    partial_match = Some(i);
                }
            }

            // Use partial match if found
            if let Some(index) = partial_match {
                return index;
            }
        }
        0
    }

    /// Gets a task that retrieves the theme mode.
    pub fn initial_theme() -> Task<Message> {
        system::theme().map(Message::ChangeThemeMode)
    }

    /// The theme that the application is using.
    pub(crate) fn theme(&self) -> Option<Theme> {
        self.theme.as_ref().map(|t| t.value().clone())
    }

    /// The currently selected preview.
    fn current_preview(&self) -> Option<&dyn Preview> {
        self.selected_index
            .and_then(|index| self.descriptors.get(index))
            .map(|descriptor| descriptor.preview.as_ref())
    }

    pub(crate) fn setup<F>(configure: F) -> (Self, Task<Message>)
    where
        F: Fn(App) -> App,
    {
        let app = configure(App::default());
        // Check for environment variable to auto-select a specific preview
        let selected_index = App::find_preview_by_env(&app.descriptors);

        (
            Self {
                selected_index: Some(selected_index),
                ..app
            },
            App::initial_theme(),
        )
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectPreview(index) => {
                if index < self.descriptors.len() {
                    self.selected_index = Some(index);
                }
                Task::none()
            }
            Message::ChangeSearch(text) => {
                self.search = text;
                Task::none()
            }
            Message::Component(msg) => {
                // Forward component messages to the current preview
                if let Some(descriptor) = self
                    .selected_index
                    .and_then(|index| self.descriptors.get_mut(index))
                {
                    descriptor.preview.update(Message::Component(msg))
                } else {
                    Task::none()
                }
            }
            Message::Noop => Task::none(),
            Message::UpdateTheme(event) => {
                let theme = self.theme.get_or_insert_with(|| {
                    Animated::new(
                        Theme::default(self.theme_mode),
                        Easing::EASE.with_duration(Duration::from_millis(300)),
                    )
                });
                theme.update(event);
                Task::none()
            }
            Message::ChangeThemeMode(mode) => {
                self.theme_mode = mode;
                Task::none()
            }
        }
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        system::theme_changes().map(Message::ChangeThemeMode)
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        use iced::widget::{button, column, container, row, scrollable, text};
        use iced::{Alignment, Length};

        // Build sidebar with preview list
        let mut sidebar = column![
            text("Previews").size(18),
            text_input("Search previews", &self.search)
                .on_input(Message::ChangeSearch)
                .style(|theme, status| {
                    let default = text_input::default(theme, status);
                    let pair = theme.extended_palette().background.stronger;
                    text_input::Style {
                        border: match status {
                            text_input::Status::Active => {
                                default.border.rounded(4).color(pair.color)
                            }
                            _ => default.border.rounded(4),
                        },
                        value: pair.text,
                        background: pair.color.into(),
                        placeholder: pair.text.scale_alpha(0.6),
                        ..default
                    }
                }),
        ]
        .spacing(10)
        .padding(10);

        let mut sidebar_items = column![];

        // TODO: Filter descriptors based on search query
        for (index, descriptor) in self.descriptors.iter().enumerate() {
            let is_selected = Some(index) == self.selected_index;

            let btn = button(text(&descriptor.metadata.label).size(14))
                .width(Length::Fill)
                .on_press(Message::SelectPreview(index))
                .style(move |theme, status| {
                    let base = button::primary(theme, status);
                    if is_selected {
                        button::Style {
                            background: Some(theme.extended_palette().primary.base.color.into()),
                            text_color: theme.extended_palette().primary.base.text,
                            border: Border::default().rounded(4),
                            ..base
                        }
                    } else {
                        let default = button::text(theme, status);
                        let pair: Option<theme::palette::Pair> = match status {
                            button::Status::Hovered => {
                                Some(theme.extended_palette().background.stronger)
                            }
                            button::Status::Pressed => {
                                Some(theme.extended_palette().background.strongest)
                            }
                            _ => None,
                        };
                        button::Style {
                            background: pair.map(|p| p.color.into()),
                            text_color: pair.map(|p| p.text).unwrap_or(default.text_color),
                            border: Border::default().rounded(4),
                            ..default
                        }
                    }
                });

            sidebar_items = sidebar_items.push(btn);
        }

        sidebar = sidebar.push(sidebar_items);
        let sidebar = container(scrollable(sidebar))
            .width(250)
            .height(Length::Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.extended_palette().background.weak.color.into()),
                border: iced::Border {
                    color: theme.extended_palette().background.strong.color,
                    width: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            });

        // Build preview area
        let preview_content = container(
            column![
                row![
                    if let Some(index) = self.selected_index {
                        Some(
                            container(text(&self.descriptors[index].metadata.label))
                                .width(Length::Fill),
                        )
                    } else {
                        None
                    },
                    space::horizontal(),
                    theme_picker(self.theme.as_ref().map(|t| t.target().clone())),
                ]
                .align_y(Center)
                .padding(10),
                rule::horizontal(1).style(rule::weak),
                container(if let Some(preview) = &self.current_preview() {
                    preview.view()
                } else {
                    // TODO: Improve placeholder view
                    text("No preview selected").into()
                })
                .padding(20)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
            ]
            .spacing(0),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        // Combine sidebar and preview
        let page = row![sidebar, preview_content]
            .spacing(0)
            .align_y(Alignment::Start);

        if let Some(theme) = self.theme.as_ref() {
            Animation::new(theme, page)
                .on_update(Message::UpdateTheme)
                .into()
        } else {
            page.into()
        }
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("search", &self.search)
            .field("selected_index", &self.selected_index)
            .field("theme", &self.theme)
            .field("theme_mode", &self.theme_mode)
            .finish()
    }
}

pub use crate::message::Message;
use crate::{
    Preview,
    config_tab::ConfigTab,
    preview::Descriptor,
    widget::{
        config_pane, header, preview_area,
        split::{Strategy, horizontal_split, vertical_split},
    },
};
use iced::{
    Element,
    Length::Fill,
    Subscription, Task, Theme, border, keyboard, system,
    theme::{self, Base},
    widget::{button, column, container, operation, rule, scrollable, text, text_input},
};
use iced_anim::{Animated, Animation, Easing};
use std::time::Duration;

pub const SEARCH_INPUT_ID: &str = "search_input";

/// The preview app that shows registered previews.
pub struct App {
    /// A custom title for the application window.
    pub(crate) title: Option<String>,
    /// The current search query that filters previews.
    search: String,
    /// The width of the sidebar.
    sidebar_width: f32,
    /// The currently selected configuration tab.
    config_tab: ConfigTab,
    /// The height of the configuration pane underneath the preview.
    config_pane_height: f32,
    /// The list of registered previewable elements.
    descriptors: Vec<Descriptor>,
    /// The index of the selected `descriptor` in the list.
    selected_index: Option<usize>,
    /// The theme used by the application.
    theme: Option<Animated<Theme>>,
    /// The initial theme mode used by the application.
    theme_mode: theme::Mode,
}

impl Default for App {
    fn default() -> Self {
        Self {
            title: None,
            search: String::new(),
            sidebar_width: 250.0,
            config_tab: ConfigTab::default(),
            config_pane_height: 200.0,
            descriptors: Vec::new(),
            selected_index: None,
            theme: None,
            theme_mode: Default::default(),
        }
    }
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

    /// Sets up the application with the given configuration function.
    pub(crate) fn setup<F>(configure: F) -> (Self, Task<Message>)
    where
        F: Fn(App) -> App,
    {
        let mut app = configure(App::default());
        if !app.descriptors.is_empty() {
            app.selected_index = Some(0);
        }

        (app, App::initial_theme())
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectPreview(index) => {
                if index < self.descriptors.len() {
                    self.selected_index = Some(index);
                }
                Task::none()
            }
            Message::ResetPreview => {
                let Some(descriptor) = self
                    .selected_index
                    .and_then(|i| self.descriptors.get_mut(i))
                else {
                    return Task::none();
                };

                descriptor.preview.update(Message::ResetPreview)
            }
            Message::FocusInput => operation::focus(SEARCH_INPUT_ID),
            Message::ChangeSearch(text) => {
                self.search = text;
                Task::none()
            }
            Message::ResizeSidebar(size) => {
                self.sidebar_width = size;
                Task::none()
            }
            Message::ResizeConfigPane(size) => {
                self.config_pane_height = size;
                Task::none()
            }
            Message::ChangeConfigTab(tab) => {
                self.config_tab = tab;
                Task::none()
            }
            Message::TimeTravel(index) => {
                let Some(descriptor) = self
                    .selected_index
                    .and_then(|i| self.descriptors.get_mut(i))
                else {
                    return Task::none();
                };

                descriptor.preview.update(Message::TimeTravel(index))
            }
            Message::JumpToPresent => {
                let Some(descriptor) = self
                    .selected_index
                    .and_then(|i| self.descriptors.get_mut(i))
                else {
                    return Task::none();
                };

                descriptor.preview.update(Message::JumpToPresent)
            }
            Message::Component(message) => {
                // Forward component messages to the current preview
                if let Some(descriptor) = self
                    .selected_index
                    .and_then(|index| self.descriptors.get_mut(index))
                {
                    descriptor.preview.update(Message::Component(message))
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
        Subscription::batch([
            system::theme_changes().map(Message::ChangeThemeMode),
            keyboard::on_key_press(|key, modifiers| match key.as_ref() {
                keyboard::Key::Character("/") => Some(Message::FocusInput),
                keyboard::Key::Character("r") if modifiers.command() => Some(Message::ResetPreview),
                _ => None,
            }),
        ])
    }

    pub(crate) fn view(&self) -> Element<'_, Message> {
        let visible_previews: Vec<_> = self.visible_previews().collect();
        // Build sidebar with preview list
        let mut sidebar = column![
            text(format!("Previews ({})", visible_previews.len())).size(18),
            text_input("Search previews ('/' to focus)", &self.search)
                .id(SEARCH_INPUT_ID)
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

        let mut sidebar_items = vec![];

        for (index, descriptor) in visible_previews {
            let is_selected = Some(index) == self.selected_index;

            let btn = button(text(&descriptor.metadata().label).size(14))
                .width(Fill)
                .on_press(Message::SelectPreview(index))
                .style(move |theme, status| {
                    let base = button::primary(theme, status);
                    if is_selected {
                        button::Style {
                            background: Some(theme.extended_palette().primary.base.color.into()),
                            text_color: theme.extended_palette().primary.base.text,
                            border: border::rounded(4),
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
                            border: border::rounded(4),
                            ..default
                        }
                    }
                });

            sidebar_items.push(btn);
        }

        if sidebar_items.is_empty() {
            sidebar = sidebar.push(text("No previews found").size(14));
        } else {
            sidebar = sidebar.push(
                sidebar_items
                    .into_iter()
                    .fold(column![], |col, btn| col.push(btn)),
            );
        }

        let sidebar = container(scrollable(sidebar))
            .width(Fill)
            .height(Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.extended_palette().background.weak.color.into()),
                ..Default::default()
            });

        // Build preview area
        let preview_content = container(
            column![
                header(&self.theme),
                rule::horizontal(1).style(rule::weak),
                horizontal_split(
                    preview_area(self.current_preview()),
                    self.selected_index
                        .and_then(|index| self.descriptors.get(index))
                        .map(|descriptor| { config_pane(descriptor, self.config_tab) }),
                    self.config_pane_height,
                    Message::ResizeConfigPane,
                )
                .strategy(Strategy::End)
            ]
            .spacing(0),
        )
        .width(Fill)
        .height(Fill);

        // Combine sidebar and preview
        let page = vertical_split(
            sidebar,
            preview_content,
            self.sidebar_width,
            Message::ResizeSidebar,
        )
        .strategy(Strategy::Start);

        if let Some(theme) = self.theme.as_ref() {
            Animation::new(theme, page)
                .on_update(Message::UpdateTheme)
                .into()
        } else {
            page.into()
        }
    }

    /// Returns an iterator over the previews that match the current search query.
    fn visible_previews(&self) -> impl Iterator<Item = (usize, &Descriptor)> {
        let query = self.search.trim().to_lowercase();
        self.descriptors
            .iter()
            .enumerate()
            .filter(move |(_, descriptor)| descriptor.metadata().matches(&query))
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

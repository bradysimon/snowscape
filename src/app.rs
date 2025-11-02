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
pub struct App {
    /// The current search query that filters previews.
    search: String,
    /// The list of available preview descriptors.
    descriptors: Vec<&'static Descriptor>,
    /// The index of the selected `descriptor` in the list.
    selected_index: usize,
    /// The preview the user has selected.
    current_preview: Box<dyn Preview>,
    /// The theme used by the application.
    theme: Option<Animated<Theme>>,
    /// The initial theme mode used by the application.
    theme_mode: theme::Mode,
}

impl App {
    pub fn run(descriptors: Vec<&'static Descriptor>) -> iced::Result {
        iced::application(
            move || {
                (
                    Self {
                        search: String::new(),
                        current_preview: (descriptors[0].create)(),
                        descriptors: descriptors.clone(),
                        selected_index: 0,
                        theme: None,
                        theme_mode: Default::default(),
                    },
                    App::initial_theme(),
                )
            },
            Self::update,
            Self::view,
        )
        .title("Snowscape")
        .theme(App::theme)
        .subscription(App::subscription)
        .run()
    }

    /// Gets a task that retrieves the theme mode.
    pub fn initial_theme() -> Task<Message> {
        system::theme().map(Message::ChangeThemeMode)
    }

    /// The theme that the application is using.
    pub fn theme(&self) -> Option<Theme> {
        self.theme.as_ref().map(|t| t.value().clone())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectPreview(index) => {
                if index < self.descriptors.len() && index != self.selected_index {
                    self.selected_index = index;
                    self.current_preview = (self.descriptors[index].create)();
                }
                Task::none()
            }
            Message::PreviewComponent => {
                // Forward to the current preview
                self.current_preview.update(Message::PreviewComponent)
            }
            Message::ChangeSearch(text) => {
                self.search = text;
                Task::none()
            }
            Message::Component(msg) => {
                // Forward component messages to the current preview
                self.current_preview.update(Message::Component(msg))
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

    fn subscription(&self) -> Subscription<Message> {
        system::theme_changes().map(Message::ChangeThemeMode)
    }

    fn view(&self) -> Element<'_, Message> {
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
            let is_selected = index == self.selected_index;

            let btn = button(text(descriptor.metadata.label).size(14))
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
                    container(text(self.descriptors[self.selected_index].metadata.label).size(16))
                        .width(Length::Fill),
                    space::horizontal(),
                    theme_picker(self.theme.as_ref().map(|t| t.target().clone())),
                ]
                .align_y(Center)
                .padding(10),
                rule::horizontal(1).style(rule::weak),
                container(self.current_preview.view())
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

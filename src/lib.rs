mod message;
mod widget;

use iced_anim::{Animated, Animation, Easing};
pub use message::PreviewMessage;
pub use snowscape_macros::preview;

// Re-export inventory for use in generated code
#[doc(hidden)]
pub use inventory;

use iced::{
    Alignment::Center,
    Border, Element, Subscription, Task, Theme, system,
    theme::{self, Base},
    widget::{rule, space},
};
use std::{fmt, time::Duration};

use crate::widget::theme_picker;

/// A descriptor for a preview component that can be registered.
pub struct PreviewDescriptor {
    pub label: &'static str,
    pub create: fn() -> Box<dyn Preview>,
}

inventory::collect!(PreviewDescriptor);

/// Trait for preview components that can be displayed in the preview window.
pub trait Preview: Send {
    /// Update the preview state with a message.
    fn update(&mut self, message: PreviewMessage) -> Task<PreviewMessage>;

    /// Render the preview.
    fn view(&self) -> Element<'_, PreviewMessage>;

    /// Get the label for this preview.
    fn label(&self) -> &str;
}

impl fmt::Debug for dyn Preview {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Preview")
            .field("label", &self.label())
            .finish()
    }
}

/// A stateless preview that renders a view function.
pub struct StatelessPreview<F>
where
    F: Fn() -> Element<'static, PreviewMessage> + Send + 'static,
{
    view_fn: F,
    label: String,
}

impl<F> StatelessPreview<F>
where
    F: Fn() -> Element<'static, PreviewMessage> + Send + 'static,
{
    pub fn new(view_fn: F) -> Self {
        Self {
            view_fn,
            label: "Stateless Preview".to_string(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl<F> Preview for StatelessPreview<F>
where
    F: Fn() -> Element<'static, PreviewMessage> + Send + 'static,
{
    fn update(&mut self, _message: PreviewMessage) -> Task<PreviewMessage> {
        Task::none()
    }

    fn view(&self) -> Element<'_, PreviewMessage> {
        (self.view_fn)()
    }

    fn label(&self) -> &str {
        &self.label
    }
}

/// A stateful preview with full update/view cycle.
pub struct StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + 'static,
{
    state: State,
    update_fn: fn(&mut State, Msg) -> Task<Msg>,
    view_fn: fn(&State) -> Element<'_, Msg>,
    label: String,
}

impl<State, Msg> StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
{
    pub fn new(
        state: State,
        update_fn: fn(&mut State, Msg) -> Task<Msg>,
        view_fn: fn(&State) -> Element<'_, Msg>,
    ) -> Self {
        Self {
            state,
            update_fn,
            view_fn,
            label: "Stateful Preview".to_string(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl<State, Msg> Preview for StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
{
    fn update(&mut self, _message: PreviewMessage) -> Task<PreviewMessage> {
        // For now, stateful previews don't handle messages from the UI
        // This would require more complex message routing
        Task::none()
    }

    fn view(&self) -> Element<'_, PreviewMessage> {
        (self.view_fn)(&self.state).map(|_msg| PreviewMessage::Noop)
    }

    fn label(&self) -> &str {
        &self.label
    }
}

/// Helper function to create a stateful preview.
pub fn stateful<State, Msg>(
    state: State,
    update_fn: fn(&mut State, Msg) -> Task<Msg>,
    view_fn: fn(&State) -> Element<'_, Msg>,
) -> StatefulPreview<State, Msg>
where
    State: Send + 'static,
    Msg: Send + Sync + std::any::Any + Clone + 'static,
{
    StatefulPreview::new(state, update_fn, view_fn)
}

/// Get all registered previews.
pub fn previews() -> Vec<&'static PreviewDescriptor> {
    inventory::iter::<PreviewDescriptor>().collect()
}

/// Run the preview application.
pub fn run() -> iced::Result {
    let preview_list = previews();

    if preview_list.is_empty() {
        eprintln!("No previews found. Add #[snowscape::preview] to your functions.");
        return Ok(());
    }

    PreviewApp::run(preview_list)
}

/// The preview application wrapper with sidebar selector.
struct PreviewApp {
    /// The list of available preview descriptors.
    descriptors: Vec<&'static PreviewDescriptor>,
    selected_index: usize,
    /// The preview the user has selected.
    current_preview: Box<dyn Preview>,
    /// The theme used by the application.
    theme: Option<Animated<Theme>>,
    /// The initial theme mode used by the application.
    theme_mode: theme::Mode,
}

impl PreviewApp {
    fn run(descriptors: Vec<&'static PreviewDescriptor>) -> iced::Result {
        iced::application(
            move || {
                (
                    Self {
                        current_preview: (descriptors[0].create)(),
                        descriptors: descriptors.clone(),
                        selected_index: 0,
                        theme: None,
                        theme_mode: Default::default(),
                    },
                    PreviewApp::initial_theme(),
                )
            },
            Self::update,
            Self::view,
        )
        .title("Snowscape")
        .theme(PreviewApp::theme)
        .subscription(PreviewApp::subscription)
        .run()
    }

    /// Gets a task that retrieves the theme mode.
    pub fn initial_theme() -> Task<PreviewMessage> {
        system::theme().map(PreviewMessage::ChangeThemeMode)
    }

    /// The theme that the application is using.
    pub fn theme(&self) -> Option<Theme> {
        self.theme.as_ref().map(|t| t.value().clone())
    }

    fn update(&mut self, message: PreviewMessage) -> Task<PreviewMessage> {
        match message {
            PreviewMessage::SelectPreview(index) => {
                if index < self.descriptors.len() && index != self.selected_index {
                    self.selected_index = index;
                    self.current_preview = (self.descriptors[index].create)();
                }
                Task::none()
            }
            PreviewMessage::PreviewComponent => {
                // Forward to the current preview
                self.current_preview
                    .update(PreviewMessage::PreviewComponent)
            }
            PreviewMessage::Noop => Task::none(),
            PreviewMessage::UpdateTheme(event) => {
                println!("Updating theme... {event:?}");
                let theme = self.theme.get_or_insert_with(|| {
                    Animated::new(
                        Theme::default(self.theme_mode),
                        Easing::EASE.with_duration(Duration::from_millis(300)),
                    )
                });
                theme.update(event);
                Task::none()
            }
            PreviewMessage::ChangeThemeMode(mode) => {
                self.theme_mode = mode;
                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<PreviewMessage> {
        system::theme_changes().map(PreviewMessage::ChangeThemeMode)
    }

    fn view(&self) -> Element<'_, PreviewMessage> {
        use iced::widget::{button, column, container, row, scrollable, text};
        use iced::{Alignment, Length};

        // Build sidebar with preview list
        let mut sidebar = column![
            text("Previews").size(18),
            container(row![]).height(1).style(|theme: &Theme| {
                container::Style {
                    border: iced::Border {
                        color: theme.extended_palette().background.strong.color,
                        width: 1.0,
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
        ]
        .spacing(10)
        .padding(10);

        let mut sidebar_items = column![];

        for (index, descriptor) in self.descriptors.iter().enumerate() {
            let is_selected = index == self.selected_index;

            let btn = button(text(descriptor.label).size(14))
                .width(Length::Fill)
                .on_press(PreviewMessage::SelectPreview(index))
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
                    container(text(self.descriptors[self.selected_index].label).size(16))
                        .width(Length::Fill),
                    space::horizontal(),
                    theme_picker(self.theme())
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
                .on_update(PreviewMessage::UpdateTheme)
                .into()
        } else {
            page.into()
        }
    }
}

pub use crate::message::Message;
use crate::{
    Preview,
    config_tab::ConfigTab,
    preview::Descriptor,
    test,
    widget::{
        config_pane, header, preview_area, preview_list, recorder, search_input,
        split::{Strategy, horizontal_split, vertical_split},
    },
};
use iced::{
    Element,
    Length::Fill,
    Subscription, Task, Theme, keyboard, system,
    theme::{self, Base},
    widget::{button, column, container, opaque, operation, rule, scrollable, space, stack, text},
    window,
};
use iced_anim::{Animated, Animation, Easing};
use std::{path::PathBuf, sync::Arc, time::Duration};

pub const SEARCH_INPUT_ID: &str = "search_input";

/// A function to configure your app's previews.
/// Send + Sync bound required to allow running tests off the main thread.
pub(crate) type ConfigureFn = Arc<dyn Fn(App) -> App + Send + Sync>;

#[derive(Debug, Clone)]
struct DeleteTestDialog {
    path: PathBuf,
    name: String,
}

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
    /// The ID of the main window.
    main_window: Option<window::Id>,
    /// App configuration callback used for test runs.
    configure: Option<ConfigureFn>,
    /// Test-related state.
    test: test::State,
    /// Pending delete confirmation dialog state.
    delete_test_dialog: Option<DeleteTestDialog>,
    /// State for the global dialog widget.
    dialog: crate::widget::dialog::State,
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
            main_window: None,
            configure: None,
            test: test::State::default(),
            delete_test_dialog: None,
            dialog: crate::widget::dialog::State::default(),
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
    pub fn preview(mut self, preview: impl Preview + 'static) -> Self {
        self.descriptors.push(preview.into());
        self
    }

    /// Sets the path to the tests directory (useful for previews/testing).
    pub fn with_tests_dir(mut self, path: impl AsRef<std::path::Path>) -> Self {
        self.test = test::State {
            config: test::Config::default().with_tests_dir(path.as_ref()),
            ..test::State::default()
        };
        self
    }

    #[cfg(feature = "internal")]
    pub fn with_test_state(mut self, test_state: test::State) -> Self {
        self.test = test_state;
        self
    }

    /// Gets a task that retrieves the theme mode.
    pub fn initial_theme() -> Task<Message> {
        system::theme().map(Message::ChangeThemeMode)
    }

    /// The theme that the application is using.
    pub(crate) fn theme(&self, _window: window::Id) -> Option<Theme> {
        self.theme.as_ref().map(|t| t.value().clone())
    }

    /// The currently selected preview.
    fn current_preview(&self) -> Option<&dyn Preview> {
        self.selected_index
            .and_then(|index| self.descriptors.get(index))
            .map(|descriptor| descriptor.preview.as_ref())
    }

    /// Returns true if a test recording is currently active.
    pub fn is_recording(&self) -> bool {
        self.test.is_recording()
    }

    /// Returns the test state.
    pub fn test_state(&self) -> &test::State {
        &self.test
    }

    /// Returns the registered preview descriptors.
    pub fn descriptors(&self) -> &[Descriptor] {
        &self.descriptors
    }

    /// Returns mutable access to the registered preview descriptors.
    pub fn descriptors_mut(&mut self) -> &mut [Descriptor] {
        &mut self.descriptors
    }

    /// Sets up the application with the given configuration function.
    pub(crate) fn setup(configure: ConfigureFn) -> (Self, Task<Message>) {
        let mut app = (configure)(App::default());
        app.configure = Some(configure.clone());
        if !app.descriptors.is_empty() {
            app.selected_index = Some(0);
        }

        let refresh_task = app
            .descriptors
            .first()
            .map(|descriptor| {
                app.test
                    .update(
                        test::Message::RefreshList(descriptor.metadata().label.clone()),
                        None,
                    )
                    .map(Message::Test)
            })
            .unwrap_or_else(Task::none);

        // Open the main window
        let (main_id, open_main) = window::open(window::Settings {
            exit_on_close_request: false,
            ..Default::default()
        });
        app.main_window = Some(main_id);

        (
            app,
            Task::batch([open_main.discard(), App::initial_theme(), refresh_task]),
        )
    }

    pub(crate) fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SelectPreview(index) => {
                if index < self.descriptors.len() {
                    self.selected_index = Some(index);
                    let preview_name = self.descriptors[index].metadata().label.clone();
                    return self
                        .test
                        .update(test::Message::RefreshList(preview_name), None)
                        .map(Message::Test);
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
            Message::ChangeParam(index, param) => {
                let Some(descriptor) = self
                    .selected_index
                    .and_then(|i| self.descriptors.get_mut(i))
                else {
                    return Task::none();
                };

                descriptor
                    .preview
                    .update(Message::ChangeParam(index, param))
            }
            Message::ResetParams => {
                let Some(descriptor) = self
                    .selected_index
                    .and_then(|i| self.descriptors.get_mut(i))
                else {
                    return Task::none();
                };

                descriptor.preview.update(Message::ResetParams)
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
            Message::WindowClosed(id) => {
                // If the test window was the one closed, stop the recording
                if self.test.window_id == Some(id) {
                    self.test
                        .update(test::Message::StopRecording, None)
                        .map(Message::Test)
                } else if self.main_window == Some(id) {
                    // Main window is closing, so shut down the application
                    window::close(id).chain(iced::exit())
                } else {
                    window::close(id)
                }
            }
            Message::Test(msg) => {
                // Reset preview state when starting a recording to ensure consistent test runs.
                let reset_task = if matches!(msg, test::Message::StartRecording) {
                    self.selected_index
                        .and_then(|index| self.descriptors.get_mut(index))
                        .map(|descriptor| descriptor.preview.update(Message::ResetPreview))
                        .unwrap_or_else(Task::none)
                } else {
                    Task::none()
                };

                // Build context for test update if we have a selected preview
                let ctx = self.selected_index.and_then(|index| {
                    self.descriptors
                        .get(index)
                        .map(|d| test::state::UpdateContext {
                            preview_name: &d.metadata().label,
                            preview_index: index,
                            configure: self.configure.clone(),
                        })
                });
                Task::batch([reset_task, self.test.update(msg, ctx).map(Message::Test)])
            }
            Message::OpenDeleteTestDialog(path) => {
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("this test")
                    .to_owned();

                self.delete_test_dialog = Some(DeleteTestDialog { path, name });
                self.dialog.open();
                Task::none()
            }
            Message::Dialog(message) => {
                let action = self.dialog.update(message);
                if let Some(crate::widget::dialog::Action::Closed) = action {
                    self.delete_test_dialog = None;
                }

                Task::none()
            }
            Message::ConfirmDeleteTest => {
                let Some(dialog) = &self.delete_test_dialog else {
                    return Task::none();
                };

                self.dialog.close();

                // Build context for test update if we have a selected preview
                let ctx = self.selected_index.and_then(|index| {
                    self.descriptors
                        .get(index)
                        .map(|d| test::state::UpdateContext {
                            preview_name: &d.metadata().label,
                            preview_index: index,
                            configure: self.configure.clone(),
                        })
                });

                self.test
                    .update(test::Message::Delete(dialog.path.clone()), ctx)
                    .map(Message::Test)
            }
        }
    }

    pub(crate) fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            system::theme_changes().map(Message::ChangeThemeMode),
            keyboard::listen().filter_map(|event| match event {
                keyboard::Event::KeyPressed { key, modifiers, .. } => match key.as_ref() {
                    keyboard::Key::Character("/") => Some(Message::FocusInput),
                    keyboard::Key::Character("r") if modifiers.command() => {
                        Some(Message::ResetPreview)
                    }
                    _ => None,
                },
                _ => None,
            }),
            window::close_requests().map(Message::WindowClosed),
        ])
    }

    pub(crate) fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        // Check if this is the test window
        if self.test.session.is_some() && Some(window_id) != self.main_window {
            return self.view_test_window();
        }

        // Main window view
        self.view_main_window()
    }

    /// Renders the main application window.
    fn view_main_window(&self) -> Element<'_, Message> {
        // Build sidebar with preview list
        let sidebar = column![
            text("Previews").size(18),
            search_input(&self.search),
            preview_list(self.visible_previews(), self.selected_index),
        ]
        .spacing(10)
        .padding(10);

        let sidebar = container(scrollable(sidebar))
            .width(Fill)
            .height(Fill)
            .style(|theme: &Theme| container::Style {
                background: Some(theme.palette().background.weaker.color.into()),
                ..Default::default()
            });

        // Prevent clicks in the main window if there's an active test recording.
        let preview_body = if self.test.is_recording() {
            stack![
                preview_area(self.current_preview()),
                opaque(space().width(Fill).height(Fill)),
            ]
            .into()
        } else {
            preview_area(self.current_preview())
        };

        let preview_content = container(
            column![
                header(&self.theme),
                rule::horizontal(1).style(rule::weak),
                horizontal_split(
                    preview_body,
                    self.selected_index
                        .and_then(|index| self.descriptors.get(index))
                        .map(|descriptor| { config_pane(descriptor, self.config_tab, self) }),
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

        let delete_config = if self.dialog.is_visible() {
            self.delete_test_dialog.as_ref().map(|dialog| {
                crate::widget::dialog::Config::new(
                    column![
                        text("Are you sure you want to delete this test?").size(16),
                        text(&dialog.name).size(14).style(crate::style::text::muted),
                    ]
                    .spacing(8),
                )
                .title("Delete Test")
                .push_action(
                    button("Delete")
                        .on_press(Message::ConfirmDeleteTest)
                        .style(crate::style::button::danger),
                )
                .push_action(
                    button("Cancel")
                        .on_press(Message::Dialog(crate::widget::dialog::Message::Close))
                        .style(crate::style::button::ghost_subtle),
                )
            })
        } else {
            None
        };

        let page: Element<'_, Message> = crate::widget::dialog(page, &self.dialog, delete_config)
            .on_update(Message::Dialog)
            .into();

        if let Some(theme) = self.theme.as_ref() {
            Animation::new(theme, page)
                .on_update(Message::UpdateTheme)
                .into()
        } else {
            page
        }
    }

    /// Renders the isolated test window with just the preview.
    fn view_test_window(&self) -> Element<'_, Message> {
        let Some(session) = &self.test.session else {
            return text("No test session").into();
        };

        let Some(descriptor) = self.descriptors.get(session.preview_index) else {
            return text("Preview not found").into();
        };

        // Render the preview wrapped with the recorder to capture interactions
        let preview_content = container(descriptor.preview.view()).center(Fill);

        if session.is_recording {
            // Wrap with recorder to capture user interactions
            recorder(preview_content)
                .on_record(|i| Message::Test(test::Message::RecordInteraction(i)))
                .into()
        } else {
            preview_content.into()
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

    /// Returns the title for a given window.
    pub(crate) fn window_title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.main_window {
            self.title
                .clone()
                .unwrap_or_else(|| "Snowscape Previews".to_owned())
        } else if let Some(session) = &self.test.session {
            format!("Test: {}", session.preview_name)
        } else {
            "Test Window".to_owned()
        }
    }

    /// An `internal` feature view method exclusively used for previewing Snowscape.
    /// This renders the main window view without requiring a window ID.
    #[cfg(feature = "internal")]
    pub fn internal_view(&self) -> Element<'_, Message> {
        self.view_main_window()
    }

    /// An `internal` feature update method exclusively used for previewing Snowscape.
    #[cfg(feature = "internal")]
    pub fn internal_update(&mut self, message: Message) -> Task<Message> {
        self.update(message)
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

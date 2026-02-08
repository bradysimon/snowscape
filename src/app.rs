pub use crate::message::Message;
use crate::{
    Preview,
    config_tab::ConfigTab,
    preview::Descriptor,
    test::{Config, Session},
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
    widget::{column, container, operation, rule, scrollable, text},
    window,
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
    /// The ID of the main window.
    main_window: Option<window::Id>,
    /// The ID of the test window when recording.
    test_window: Option<window::Id>,
    /// Test configuration for the test window.
    test_config: Config,
    /// The width input for the test window (as string for text input).
    test_width_input: String,
    /// The height input for the test window (as string for text input).
    test_height_input: String,
    /// The active test recording session, if any.
    test_session: Option<Session>,
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
            test_window: None,
            test_config: Config::default(),
            test_width_input: "800".to_string(),
            test_height_input: "600".to_string(),
            test_session: None,
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
        self.test_session.as_ref().is_some_and(|s| s.is_recording)
    }

    /// Returns the test configuration.
    pub fn test_config(&self) -> &Config {
        &self.test_config
    }

    /// Returns the test width input string.
    pub fn test_width_input(&self) -> &str {
        &self.test_width_input
    }

    /// Returns the test height input string.
    pub fn test_height_input(&self) -> &str {
        &self.test_height_input
    }

    /// Returns the registered preview descriptors.
    pub fn descriptors(&self) -> &[Descriptor] {
        &self.descriptors
    }

    /// Returns mutable access to the registered preview descriptors.
    pub fn descriptors_mut(&mut self) -> &mut [Descriptor] {
        &mut self.descriptors
    }

    /// Returns the current test session, if any.
    pub fn test_session(&self) -> Option<&Session> {
        self.test_session.as_ref()
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

        // Open the main window
        let (main_id, open_main) = window::open(window::Settings {
            exit_on_close_request: false,
            ..Default::default()
        });
        app.main_window = Some(main_id);

        (
            app,
            Task::batch([open_main.discard(), App::initial_theme()]),
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
            // Test-related messages
            Message::ChangeTestWidth(width) => {
                self.test_width_input = width.clone();
                if let Ok(w) = width.parse::<f32>() {
                    self.test_config.window_size.width = w;
                }
                Task::none()
            }
            Message::ChangeTestHeight(height) => {
                self.test_height_input = height.clone();
                if let Ok(h) = height.parse::<f32>() {
                    self.test_config.window_size.height = h;
                }
                Task::none()
            }
            Message::ToggleTestSnapshot(enabled) => {
                self.test_config.capture_snapshot = enabled;
                Task::none()
            }
            Message::StartTestRecording => {
                let Some(index) = self.selected_index else {
                    return Task::none();
                };
                let Some(descriptor) = self.descriptors.get(index) else {
                    return Task::none();
                };

                // Create the test session
                let preview_name = descriptor.metadata().label.clone();
                let session = Session::new(self.test_config.clone(), index, preview_name.clone());
                self.test_session = Some(session);

                // Open the test window
                // Note: Using Default position to avoid objc2 NSScreen enumeration crash on some macOS versions
                let (id, open_task) = window::open(window::Settings {
                    size: self.test_config.window_size,
                    exit_on_close_request: false,
                    ..Default::default()
                });
                self.test_window = Some(id);

                open_task.map(Message::TestWindowOpened)
            }
            Message::TestWindowOpened(id) => {
                // Window is now open, store the ID for later closing
                self.test_window = Some(id);
                Task::none()
            }
            Message::RecordInteraction(interaction) => {
                if let Some(session) = &mut self.test_session {
                    session.record(interaction);
                }
                Task::none()
            }
            Message::StopTestRecording => {
                let Some(session) = &self.test_session else {
                    return Task::none();
                };

                // Save the test file
                if let Err(e) = session.save() {
                    eprintln!("Failed to save test: {}", e);
                }

                // Close the test window
                if let Some(test_window_id) = self.test_window.take() {
                    if session.config.capture_snapshot {
                        window::screenshot(test_window_id)
                            .map(Message::TestScreenshotCaptured)
                            .chain(window::close(test_window_id))
                            .chain(Task::done(Message::RemoveTestSession))
                    } else {
                        window::close(test_window_id).chain(Task::done(Message::RemoveTestSession))
                    }
                } else {
                    Task::none()
                }
            }
            Message::WindowClosed(id) => {
                // If the test window was the one closed, stop the recording
                // Don't close immediately - let StopTestRecording handle closing after screenshot
                if self.test_window == Some(id) {
                    Task::done(Message::StopTestRecording)
                } else if self.main_window == Some(id) {
                    // Main window is closing, so shut down the application
                    window::close(id).chain(iced::exit())
                } else {
                    window::close(id)
                }
            }
            Message::TestScreenshotCaptured(screenshot) => {
                // Save screenshot to disk as PNG
                if let Some(session) = &mut self.test_session {
                    let snapshot_name = session.next_snapshot_name();
                    let snapshot_path = session.config.tests_dir.join(&snapshot_name);
                    if let Err(e) = std::fs::create_dir_all(&session.config.tests_dir) {
                        eprintln!("Failed to create tests directory: {}", e);
                    } else {
                        // Create an image buffer from the screenshot's RGBA data
                        let width = screenshot.size.width;
                        let height = screenshot.size.height;
                        let rgba_data: &[u8] = screenshot.as_ref();

                        match image::RgbaImage::from_raw(width, height, rgba_data.to_vec()) {
                            Some(img) => {
                                if let Err(e) = img.save(&snapshot_path) {
                                    eprintln!("Failed to save snapshot as PNG: {}", e);
                                }
                            }
                            None => {
                                eprintln!("Failed to create image from screenshot data");
                            }
                        }
                    }
                }
                Task::none()
            }
            Message::ChangeExpectText(text) => {
                if let Some(session) = &mut self.test_session {
                    session.expect_text_input = text;
                }
                Task::none()
            }
            Message::AddTextExpectation => {
                if let Some(session) = &mut self.test_session {
                    let text = std::mem::take(&mut session.expect_text_input);
                    session.add_text_expectation(text);
                }
                Task::none()
            }
            Message::CaptureSnapshot => {
                // Request a screenshot from the test window
                if let Some(test_window_id) = self.test_window {
                    window::screenshot(test_window_id).map(Message::TestScreenshotCaptured)
                } else {
                    Task::none()
                }
            }
            Message::RemoveTestSession => {
                self.test_session = None;
                Task::none()
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
        if self.test_session.is_some() && Some(window_id) != self.main_window {
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
                background: Some(theme.extended_palette().background.weaker.color.into()),
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

        if let Some(theme) = self.theme.as_ref() {
            Animation::new(theme, page)
                .on_update(Message::UpdateTheme)
                .into()
        } else {
            page.into()
        }
    }

    /// Renders the isolated test window with just the preview.
    fn view_test_window(&self) -> Element<'_, Message> {
        let Some(session) = &self.test_session else {
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
                .on_record(Message::RecordInteraction)
                .into()
        } else {
            preview_content.into()
        }
    }

    /// Returns an iterator over the previews that match the current search query.
    fn visible_previews(&self) -> impl Iterator<Item = &Descriptor> {
        let query = self.search.trim().to_lowercase();
        self.descriptors
            .iter()
            .filter(move |descriptor| descriptor.metadata().matches(&query))
    }

    /// Returns the title for a given window.
    pub(crate) fn window_title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.main_window {
            self.title
                .clone()
                .unwrap_or_else(|| "Snowscape Previews".to_owned())
        } else if let Some(session) = &self.test_session {
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

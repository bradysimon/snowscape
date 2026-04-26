use std::time::Duration;

use iced::Alignment::Center;
use iced::Length::Fill;
use iced::widget::{button, column, container, pick_list, row, space, text};
use iced::{Color, Element};
use snowscape::preview::{Performance, Preview};
use snowscape::preview::{dynamic, performance::Indicator, stateful, stateless, stateless_with};
use snowscape::test::discovery::TestInfo;
use snowscape::{App, ConfigTab, Metadata, test, widget};

/// Configures the Snowscape app with all self-previews.
pub fn previews(app: App) -> App {
    app.title("Snowscape Previews")
        .with_tests_dir(format!("{}/tests", env!("CARGO_MANIFEST_DIR")))
        .preview(config_tabs())
        .preview(preview_list())
        .preview(about_pane())
        .preview(parameter_pane())
        .preview(message_pane())
        .preview(performance_pane())
        .preview(test_pane())
        .preview(dialog_preview())
        .preview(dialog_preview_no_animation())
        .preview(app_preview())
}

/// Builds the Snowscape app as a standalone [`iced::Program`] for use in
/// `snowscape::test::Emulator`-driven automation tests.
pub fn program()
-> impl iced::Program<State = App, Message = snowscape::Message, Theme = iced::Theme> {
    iced::application(
        || {
            let mut app = previews(App::default());
            // Select the first preview so the UI has something to show.
            if !app.descriptors().is_empty() {
                let _ = app.internal_update(snowscape::Message::SelectPreview(0));
            }
            app
        },
        App::internal_update,
        App::internal_view,
    )
    .title("Snowscape Previews")
}

fn config_tabs() -> impl Preview {
    dynamic::stateless(
        "Config Tabs",
        (
            dynamic::select("Selected Tab", &ConfigTab::ALL, ConfigTab::default()),
            dynamic::number("Parameter Count", 0),
            dynamic::number("Message Count", 0),
            dynamic::number("Test Count", 0),
            dynamic::select(
                "Performance Indicator",
                &Indicator::ALL,
                Indicator::default(),
            ),
        ),
        |(tab, params, messages, tests, indicator)| {
            widget::config_tabs(
                *tab,
                usize::try_from(*params).unwrap_or(0),
                usize::try_from(*messages).unwrap_or(0),
                usize::try_from(*tests).unwrap_or(0),
                *indicator,
            )
        },
    )
    .description(
        "Tabs that appear in the configuration pane, which is shown underneath \
        the selected preview. This configuration pane lets the user change, interact, \
        or debug various parts of the preview.",
    )
}

fn preview_list() -> impl Preview {
    stateless_with(
        "Preview List",
        vec![
            stateless("Item 1", || -> Element<'static, ()> { space().into() }).into(),
            stateless("Item 2", || -> Element<'static, ()> { space().into() }).into(),
            stateless("Item 3", || -> Element<'static, ()> { space().into() }).into(),
        ],
        |items| {
            container(widget::preview_list(items.iter().enumerate(), Some(1)))
                .max_width(200)
                .into()
        },
    )
    .description(
        "Shows the user a list of previews that appears in the app's sidebar so the \
        user can choose which preview to view. The selected preview is highlighted.",
    )
}

fn about_pane() -> impl Preview {
    stateless_with(
        "About Pane",
        Metadata {
            label: String::from("A label about a component"),
            description: Some(String::from(
                "This is a longer description about the component being previewed.",
            )),
            group: Some(String::from("Group Name")),
            tags: vec![String::from("tag1"), String::from("tag2")],
        },
        widget::config_pane::about_pane::about_pane,
    )
    .description(
        "Shows metadata information that has been associated with the preview. These \
        can be added onto the `stateful` and `stateless` preview functions to give context \
        about how a component works or what its purpose is.",
    )
}

/// Previews the parameter pane widget with its own editable state.
fn parameter_pane() -> impl Preview {
    struct App {
        params: Vec<dynamic::Param>,
    }

    impl App {
        fn new() -> Self {
            Self {
                params: vec![
                    dynamic::Param::new("Boolean param", true),
                    dynamic::Param::new("Text param", String::from("Hello")),
                    dynamic::Param::new("Number param", 42),
                    dynamic::Param::new("Slider param", dynamic::Value::Slider(50.0, 0.0..=100.0)),
                    dynamic::Param::new(
                        "Color param",
                        dynamic::Value::Color(Color::from_rgba8(0, 178, 255, 1.0)),
                    ),
                ],
            }
        }

        fn view(&self) -> Element<'_, snowscape::Message> {
            widget::config_pane::parameter_pane::parameter_pane(&self.params)
        }

        fn update(&mut self, message: snowscape::Message) {
            if let snowscape::Message::ChangeParam(index, value) = message
                && let Some(param) = self.params.get_mut(index)
            {
                param.value = value;
            }
        }
    }

    stateful("Parameter Pane", App::new, App::update, App::view).description(
        "A pane allowing the user to dynamically change parameters that appear within Snowscape. \
        This lets users experiment with different settings and immediately see the effects on the \
        previewed component without having to recompile or restart the application.",
    )
}

fn message_pane() -> impl Preview {
    stateless_with(
        "Message Pane",
        [
            String::from("Initialized preview."),
            String::from("Parameter 'X' changed to 42."),
            String::from("Parameter 'Color' changed to #00B2FF."),
            String::from("Preview rendered successfully."),
        ],
        |messages| widget::config_pane::message_pane::message_pane(messages),
    )
    .description(
        "Displays a log of messages that have been emitted by the open preview. \
        This is useful for debugging and understanding the flow of data and events \
        within the previewed component.",
    )
}

/// Previews the performance pane widget with sample performance data.
fn performance_pane() -> impl Preview {
    stateless_with(
        "Performance Pane",
        Performance::new(
            vec![
                Duration::from_micros(10),
                Duration::from_micros(5),
                Duration::from_micros(8),
                Duration::from_micros(15),
                Duration::from_micros(3),
                Duration::from_micros(12),
                Duration::from_micros(10),
                Duration::from_micros(10),
                Duration::from_micros(3),
                Duration::from_micros(12),
                Duration::from_micros(10),
            ],
            vec![
                Duration::from_micros(50),
                Duration::from_micros(100),
                Duration::from_micros(200),
                Duration::from_micros(400),
                Duration::from_micros(300),
                Duration::from_micros(125),
                Duration::from_micros(125),
                Duration::from_micros(3_000),
                Duration::from_micros(4_500),
                Duration::from_micros(8_000),
                Duration::from_micros(16_000),
            ],
        ),
        |performance| widget::config_pane::performance_pane::performance_pane(Some(performance)),
    )
    .description(
        "Shows performance metrics for the previewed component, including view/update times \
        and jank indicators. This helps users identify performance bottlenecks and optimize \
        their components for smoother interactions.",
    )
}

/// Previews the test configuration pane with sample test data.
fn test_pane() -> impl Preview {
    stateful(
        "Test Pane",
        || {
            // Create sample test state with discovered tests
            let test_state = test::State {
                discovered_tests: vec![
                    TestInfo {
                        name: "basic-increment".to_string(),
                        path: std::path::PathBuf::from("tests/counter/basic-increment.ice"),
                        preview: "counter".to_string(),
                        has_snapshot: true,
                    },
                    TestInfo {
                        name: "increment-and-decrement".to_string(),
                        path: std::path::PathBuf::from("tests/counter/increment-and-decrement.ice"),
                        preview: "counter".to_string(),
                        has_snapshot: true,
                    },
                    TestInfo {
                        name: "edge-cases".to_string(),
                        path: std::path::PathBuf::from("tests/counter/edge-cases.ice"),
                        preview: "counter".to_string(),
                        has_snapshot: false,
                    },
                ],
                last_run_results: Some(vec![
                    test::Outcome::passed("basic-increment"),
                    test::Outcome::failed("increment-and-decrement", "Snapshot mismatch at step 3"),
                ]),
                ..test::State::default()
            };

            snowscape::App::default()
                .title("Test Pane Preview")
                .with_test_state(test_state)
        },
        snowscape::App::internal_update,
        |app: &snowscape::App| widget::config_pane::test_pane::test_pane(app),
    )
    .description(
        "The test configuration pane allows users to create, run, and manage visual tests \
        for previews. Tests are organized in folders by preview name and support snapshots.",
    )
}

/// Builds a dialog preview configured with optional animation.
/// This is done this way instead of a preview with a param for now
/// since we want the ability to have tests for this component.
fn dialog_preview_builder(animated: bool) -> impl Preview {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    enum Flavor {
        #[default]
        Vanilla,
        Chocolate,
        Strawberry,
        CookiesAndCream,
        Butterscotch,
        Pecan,
    }

    impl Flavor {
        const ALL: [Flavor; 6] = [
            Flavor::Vanilla,
            Flavor::Chocolate,
            Flavor::Strawberry,
            Flavor::CookiesAndCream,
            Flavor::Butterscotch,
            Flavor::Pecan,
        ];
    }

    impl std::fmt::Display for Flavor {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let label = match self {
                Flavor::Vanilla => "Vanilla",
                Flavor::Chocolate => "Chocolate",
                Flavor::Strawberry => "Strawberry",
                Flavor::CookiesAndCream => "Cookies and Cream",
                Flavor::Butterscotch => "Butterscotch",
                Flavor::Pecan => "Pecan",
            };
            write!(f, "{label}")
        }
    }

    #[derive(Debug, Clone)]
    enum Message {
        Open,
        Increment,
        Decrement,
        SelectFlavor(Flavor),
        Dialog(widget::dialog::Message),
    }

    struct DialogDemo {
        counter: i32,
        flavor: Flavor,
        dialog_state: widget::dialog::State,
        closed_count: u32,
    }

    impl DialogDemo {
        fn update(&mut self, message: Message) {
            match message {
                Message::Open => self.dialog_state.open(),
                Message::Increment => self.counter += 1,
                Message::Decrement => self.counter -= 1,
                Message::SelectFlavor(flavor) => self.flavor = flavor,
                Message::Dialog(dialog_message) => {
                    let action = self.dialog_state.update(dialog_message);
                    if let Some(widget::dialog::Action::Closed) = action {
                        self.closed_count += 1;
                    }
                }
            }
        }

        fn view(&self) -> Element<'_, Message> {
            let base = container(
                column![
                    button("Open Dialog").on_press(Message::Open),
                    text(format!("Dialog status: {:?}", self.dialog_state.status())),
                    text(format!("Closed count: {}", self.closed_count)),
                ]
                .spacing(10)
                .padding(20),
            )
            .width(Fill)
            .height(Fill);

            let config = self.dialog_state.is_visible().then(|| {
                widget::dialog::Config::new(
                    container(
                        column![
                            row![
                                button("-").on_press(Message::Decrement),
                                text!("{}", self.counter).width(40).line_height(1.0).center(),
                                button("+").on_press(Message::Increment),
                                space::horizontal(),
                                pick_list(Some(self.flavor), Flavor::ALL, Flavor::to_string)
                                    .on_select(Message::SelectFlavor)
                                    .placeholder("Select flavor")
                            ]
                            .spacing(4)
                            .align_y(Center),
                            text("A dialog that appears over the main content with a backdrop behind it."),
                            text("Click the backdrop, press Esc, or use the top-right close button."),
                        ]
                        .spacing(8),
                    )
                    .width(Fill),
                )
                .title("Dialog Title")
                .close_label("Close")
                .width(500)
                .push_action(
                    button("Confirm")
                        .on_press(Message::Dialog(widget::dialog::Message::Close)),
                )
            });

            widget::dialog(base, &self.dialog_state, config)
                .on_update(Message::Dialog)
                .backdrop_close(true)
                .esc_close(true)
                .into()
        }
    }

    stateful(
        if animated {
            "Dialog"
        } else {
            "Dialog Without Animation"
        },
        move || DialogDemo {
            counter: 0,
            flavor: Flavor::default(),
            dialog_state: widget::dialog::State::default().animated(animated),
            closed_count: 0,
        },
        DialogDemo::update,
        DialogDemo::view,
    )
    .description("A stateful preview to validate opening and closing the dialog widget.")
}

/// Previews the dialog widget with animations enabled.
fn dialog_preview() -> impl Preview {
    dialog_preview_builder(true)
}

// TODO: Might be cool to make params configurable for tests, or utilize presets somehow.
/// Previews the dialog widget with animations disabled for deterministic tests.
fn dialog_preview_no_animation() -> impl Preview {
    dialog_preview_builder(false)
}

/// Previews the entire Snowscape application itself as a nested preview.
fn app_preview() -> impl Preview {
    stateful(
        "App",
        || {
            snowscape::App::default()
                .title("Nested App")
                .preview(config_tabs())
                .preview(preview_list())
                .preview(about_pane())
                .preview(parameter_pane())
                .preview(message_pane())
                .preview(performance_pane())
                .preview(test_pane())
                .preview(dialog_preview())
                .preview(dialog_preview_no_animation())
        },
        snowscape::App::internal_update,
        snowscape::App::internal_view,
    )
    .description("A nested preview of the entire Snowscape application itself!")
}

use std::time::Duration;

use iced::widget::{container, space};
use iced::{Color, Element};
use snowscape::preview::Performance;
use snowscape::preview::{
    Descriptor, dynamic, performance::Indicator, stateful, stateless, stateless_with,
};
use snowscape::{App, ConfigTab, Metadata, widget};

/// Previews various components used within Snowscape.
fn main() -> iced::Result {
    snowscape::run(|app| {
        app.title("Snowscape Previews")
            .preview(config_tabs())
            .preview(preview_list())
            .preview(about_pane())
            .preview(parameter_pane())
            .preview(message_pane())
            .preview(performance_pane())
            .preview(app_preview())
    })
}

fn config_tabs() -> impl Into<Descriptor> {
    dynamic::stateless(
        "Config Tabs",
        (
            dynamic::select("Selected Tab", &ConfigTab::ALL, ConfigTab::default()),
            dynamic::number("Parameter Count", 0),
            dynamic::number("Message Count", 0),
            dynamic::select(
                "Performance Indicator",
                &Indicator::ALL,
                Indicator::default(),
            ),
        ),
        |(tab, params, messages, indicator)| {
            widget::config_tabs(
                *tab,
                usize::try_from(*params).unwrap_or(0),
                usize::try_from(*messages).unwrap_or(0),
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

fn preview_list() -> impl Into<Descriptor> {
    stateless_with(
        "Preview List",
        vec![
            stateless("Item 1", || -> Element<'static, ()> { space().into() }).into(),
            stateless("Item 2", || -> Element<'static, ()> { space().into() }).into(),
            stateless("Item 3", || -> Element<'static, ()> { space().into() }).into(),
        ],
        |items| {
            container(widget::preview_list(items, Some(1)))
                .max_width(200)
                .into()
        },
    )
    .description(
        "Shows the user a list of previews that appears in the app's sidebar so the \
        user can choose which preview to view. The selected preview is highlighted.",
    )
}

fn about_pane() -> impl Into<Descriptor> {
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
        |metadata| widget::config_pane::about_pane::about_pane(&metadata),
    )
    .description(
        "Shows metadata information that has been associated with the preview. These \
        can be added onto the `stateful` and `stateless` preview functions to give context \
        about how a component works or what its purpose is.",
    )
}

/// Previews the parameter pane widget with its own editable state.
fn parameter_pane() -> impl Into<Descriptor> {
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
            match message {
                snowscape::Message::ChangeParam(index, value) => {
                    if let Some(param) = self.params.get_mut(index) {
                        param.value = value;
                    }
                }
                _ => {}
            }
        }
    }

    stateful("Parameter Pane", App::new, App::update, App::view).description(
        "A pane allowing the user to dynamically change parameters that appear within Snowscape. \
        This lets users experiment with different settings and immediately see the effects on the \
        previewed component without having to recompile or restart the application.",
    )
}

fn message_pane() -> impl Into<Descriptor> {
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
fn performance_pane() -> impl Into<Descriptor> {
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
        "Shows performance metrics for the previewed component, including frame times \
        and jank indicators. This helps users identify performance bottlenecks and optimize \
        their components for smoother interactions.",
    )
}

/// Previews the entire Snowscape application itself as a nested preview.
fn app_preview() -> impl Into<Descriptor> {
    stateful(
        "App",
        || {
            App::default()
                .title("Nested App")
                .preview(config_tabs())
                .preview(preview_list())
                .preview(about_pane())
                .preview(parameter_pane())
                .preview(message_pane())
                .preview(performance_pane())
        },
        App::internal_update,
        App::internal_view,
    )
    .description("A nested preview of the entire Snowscape application itself!")
}

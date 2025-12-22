use iced::widget::{container, space};
use iced::{Color, Element};
use snowscape::preview::{Descriptor, dynamic, stateful, stateless, stateless_with};
use snowscape::{App, ConfigTab, Metadata, widget};

/// Previews various components used within Snowscape.
fn main() -> iced::Result {
    snowscape::run(|app| {
        app.title("Snowscape Previews")
            .preview(dynamic::stateless(
                "Config Tabs",
                (
                    dynamic::select("Selected Tab", &ConfigTab::ALL, ConfigTab::default()),
                    dynamic::number("Parameter Count", 0),
                    dynamic::number("Message Count", 0),
                ),
                |(tab, params, messages)| {
                    widget::config_tabs(
                        *tab,
                        usize::try_from(*params).unwrap_or(0),
                        usize::try_from(*messages).unwrap_or(0),
                    )
                },
            ))
            .preview(stateless_with(
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
            ))
            .preview(stateless_with(
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
            ))
            .preview(parameter_pane())
            .preview(stateless_with(
                "Message Pane",
                [
                    String::from("Initialized preview."),
                    String::from("Parameter 'X' changed to 42."),
                    String::from("Parameter 'Color' changed to #00B2FF."),
                    String::from("Preview rendered successfully."),
                ],
                |messages| widget::config_pane::message_pane::message_pane(messages),
            ))
            .preview(stateful(
                "App",
                || App::default().title("Nested App").preview(parameter_pane()),
                App::internal_update,
                App::internal_view,
            ))
    })
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
        "A pane allowing the user to dynamically change parameters that appear within Snowscape",
    )
}

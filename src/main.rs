use iced::Element;
use iced::widget::{container, space};
use snowscape::preview::{dynamic, stateless, stateless_with};
use snowscape::{ConfigTab, Metadata, widget};

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
            .preview(stateless_with(
                "Parameter Pane",
                [
                    dynamic::Param::new("Boolean param", true),
                    dynamic::Param::new("Text param", String::from("Hello")),
                    dynamic::Param::new("Number param", 42),
                    dynamic::Param::new(
                        "Select param",
                        dynamic::Value::Select(
                            0,
                            vec![String::from("Option 1"), String::from("Option 2")],
                        ),
                    ),
                ],
                |params| widget::config_pane::parameter_pane::parameter_pane(params),
            ))
    })
}

use snowscape::preview::dynamic;
use snowscape::{ConfigTab, widget};

/// Previews various components used within Snowscape.
fn main() -> iced::Result {
    snowscape::run(|app| {
        app.title("Snowscape Previews").preview(dynamic::stateless(
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
    })
}

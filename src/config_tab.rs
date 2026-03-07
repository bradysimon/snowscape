use std::fmt::Display;

/// Configuration tabs shown beneath a preview.
#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ConfigTab {
    /// Displays metadata information about the current preview.
    #[default]
    About,
    /// Lists dynamic parameters that the user can configure.
    Parameters,
    /// Displays messages the current preview has emitted.
    Messages,
    /// Shows performance metrics for the current preview.
    Performance,
    /// Allows the user to record and run visual tests.
    Tests,
}

impl ConfigTab {
    /// All possible configuration tabs.
    pub const ALL: [ConfigTab; 5] = [
        ConfigTab::About,
        ConfigTab::Parameters,
        ConfigTab::Messages,
        ConfigTab::Performance,
        ConfigTab::Tests,
    ];

    /// A display name for this tab.
    pub fn name(&self) -> &'static str {
        match self {
            ConfigTab::About => "About",
            ConfigTab::Parameters => "Parameters",
            ConfigTab::Messages => "Messages",
            ConfigTab::Performance => "Performance",
            ConfigTab::Tests => "Tests",
        }
    }
}

impl Display for ConfigTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

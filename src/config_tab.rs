use std::fmt::Display;

#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ConfigTab {
    /// Displays metadata information about the current preview.
    #[default]
    About,
    Parameters,
    Messages,
    Performance,
}

impl ConfigTab {
    /// All possible configuration tabs.
    pub const ALL: [ConfigTab; 4] = [
        ConfigTab::About,
        ConfigTab::Parameters,
        ConfigTab::Messages,
        ConfigTab::Performance,
    ];

    /// A display name for this tab.
    pub fn name(&self) -> &'static str {
        match self {
            ConfigTab::About => "About",
            ConfigTab::Parameters => "Parameters",
            ConfigTab::Messages => "Messages",
            ConfigTab::Performance => "Performance",
        }
    }
}

impl Display for ConfigTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

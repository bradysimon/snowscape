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
}

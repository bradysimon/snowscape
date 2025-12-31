use iced::{Color, border};

pub struct CustomTheme {
    background: Color,
    panel_bg: Color,
    neon_pink: Color,
    neon_cyan: Color,
    neon_purple: Color,
    accent: Color,
    text_primary: Color,
    text_secondary: Color,
    text_inverted: Color,
    border: Color,
    shadow: Color,
    highlight: Color,
    radius: f32,
}

impl Default for CustomTheme {
    fn default() -> Self {
        Self {
            // Deep navy spacey background typical of 80's synthwave art
            background: Color::from_rgba8(11, 15, 46, 1.0),
            // Slightly lighter panels so neon elements pop
            panel_bg: Color::from_rgba8(18, 22, 66, 1.0),
            // Classic neon magenta / hot pink
            neon_pink: Color::from_rgba8(255, 20, 147, 1.0),
            // Neon cyan / electric blue
            neon_cyan: Color::from_rgba8(0, 242, 255, 1.0),
            // Vibrant purple for accents and glows
            neon_purple: Color::from_rgba8(170, 0, 255, 1.0),
            // Primary accent — leans cyan for contrast with pink/purple
            accent: Color::from_rgba8(0, 200, 255, 1.0),
            // High-contrast light text for readability on dark backgrounds
            text_primary: Color::from_rgba8(235, 235, 255, 0.95),
            // Muted secondary text
            text_secondary: Color::from_rgba8(160, 170, 200, 0.85),
            // Inverted text (for brighter backgrounds)
            text_inverted: Color::from_rgba8(20, 20, 30, 0.95),
            // Soft neon-ish border for UI elements
            border: Color::from_rgba8(255, 0, 255, 0.18),
            // Soft shadow for depth on panels
            shadow: Color::from_rgba8(0, 0, 0, 0.6),
            // Bright highlight for focus states (warm neon yellow)
            highlight: Color::from_rgba8(255, 210, 64, 0.95),
            // Default corner radius for rounded elements
            radius: 6.0,
        }
    }
}

impl iced::theme::Base for CustomTheme {
    fn base(&self) -> iced::theme::Style {
        iced::theme::Style {
            background_color: self.background,
            text_color: self.text_primary,
        }
    }

    fn default(_preference: iced::theme::Mode) -> Self {
        Default::default()
    }

    fn mode(&self) -> iced::theme::Mode {
        iced::theme::Mode::Dark
    }

    fn name(&self) -> &str {
        "Retro 80's Synthwave"
    }

    fn palette(&self) -> Option<iced::theme::Palette> {
        Some(iced::theme::Palette {
            background: self.background,
            text: self.text_primary,
            primary: self.accent,
            success: self.neon_cyan,
            warning: self.neon_pink,
            danger: self.neon_purple,
        })
    }
}

// MARK: Catalog impls

#[derive(Debug, Clone, Copy)]
pub enum TextVariant {
    Default,
    Primary,
    Secondary,
}

impl iced::widget::text::Catalog for CustomTheme {
    type Class<'a> = TextVariant;

    fn default<'a>() -> Self::Class<'a> {
        TextVariant::Default
    }

    fn style(&self, class: &Self::Class<'_>) -> iced::widget::text::Style {
        match class {
            TextVariant::Default => iced::widget::text::Style { color: None },
            TextVariant::Primary => iced::widget::text::Style {
                color: Some(self.text_primary),
            },
            TextVariant::Secondary => iced::widget::text::Style {
                color: Some(self.text_secondary),
            },
        }
    }
}

impl iced::widget::button::Catalog for CustomTheme {
    type Class<'a> = ();

    fn default<'a>() -> Self::Class<'a> {
        ()
    }

    fn style(
        &self,
        _class: &Self::Class<'_>,
        status: iced::widget::button::Status,
    ) -> iced::widget::button::Style {
        match status {
            iced::widget::button::Status::Disabled => iced::widget::button::Style {
                background: Some(self.panel_bg.into()),
                text_color: self.text_secondary,
                border: border::rounded(self.radius),
                ..iced::widget::button::Style::default()
            },
            iced::widget::button::Status::Active => iced::widget::button::Style {
                background: Some(self.accent.into()),
                text_color: self.text_inverted,
                border: border::rounded(self.radius),
                ..iced::widget::button::Style::default()
            },
            iced::widget::button::Status::Hovered => iced::widget::button::Style {
                background: Some(self.highlight.into()),
                text_color: self.text_inverted,
                border: border::rounded(self.radius),
                ..iced::widget::button::Style::default()
            },
            iced::widget::button::Status::Pressed => iced::widget::button::Style {
                background: Some(self.neon_purple.into()),
                text_color: self.text_primary,
                border: border::rounded(self.radius),
                ..iced::widget::button::Style::default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ContainerVariant {
    Default,
    Background,
    Panel,
}

impl iced::widget::container::Catalog for CustomTheme {
    type Class<'a> = ContainerVariant;

    fn default<'a>() -> Self::Class<'a> {
        ContainerVariant::Default
    }

    fn style(&self, class: &Self::Class<'_>) -> iced::widget::container::Style {
        match class {
            ContainerVariant::Default => iced::widget::container::Style::default(),
            ContainerVariant::Background => iced::widget::container::Style {
                background: Some(self.background.into()),
                ..iced::widget::container::Style::default()
            },
            ContainerVariant::Panel => iced::widget::container::Style {
                background: Some(self.panel_bg.into()),
                border: border::rounded(self.radius).color(self.border).width(1),
                shadow: iced::Shadow {
                    color: self.shadow,
                    offset: iced::Vector::new(0.0, 4.0),
                    blur_radius: 10.0,
                },
                text_color: Some(self.text_primary),
                ..iced::widget::container::Style::default()
            },
        }
    }
}

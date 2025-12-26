use dynamic_params::{Alignment, adjustable_view};
use iced::Color;
use snowscape::dynamic;

pub fn main() -> iced::Result {
    snowscape::run(|app| {
        app.title("Dynamic Parameters")
            .preview(
                dynamic::stateless(
                    "All dynamic params",
                    (
                        dynamic::text("Label", "The meaning of life"),
                        dynamic::number("The magic number", 42),
                        dynamic::boolean("A toggle", true),
                        dynamic::select(
                            "Alignment",
                            &[Alignment::Left, Alignment::Center, Alignment::Right],
                            Alignment::Center,
                        ),
                        dynamic::slider("Padding", 0.0..=64.0, 16.0),
                        dynamic::color("Background", Color::from_rgb(0.0, 0.78, 1.0)),
                    ),
                    |(label, number, toggle, alignment, padding, color)| {
                        adjustable_view(label, *number, *toggle, *alignment, *padding, *color)
                    },
                )
                .description("Demonstrates all dynamic parameter types: text, number, boolean, select, slider, and color"),
            )
    })
}

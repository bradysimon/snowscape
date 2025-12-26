use std::time::Duration;

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Theme,
    widget::{column, container, row, scrollable, space, text},
};

use crate::{app::Message, preview::Performance};

/// A pane shown in the configuration area displaying performance metrics.
pub fn performance_pane(performance: Option<&Performance>) -> Element<'_, Message> {
    let Some(performance) = performance else {
        return text("Performance metrics are not available for this preview.").into();
    };

    let view_stats = performance.view_stats();
    let update_stats = performance.update_stats();

    let has_view_data = view_stats.count > 0;
    let has_update_data = update_stats.count > 0;

    if !has_view_data && !has_update_data {
        return text("No performance data recorded yet. Interact with the preview to see metrics.")
            .into();
    }

    let view_section: Element<'_, Message> = if has_view_data {
        stats_grid(view_stats)
    } else {
        text("No view data recorded.").into()
    };

    let update_section: Element<'_, Message> = if has_update_data {
        stats_grid(update_stats)
    } else {
        text("No update data recorded (stateless preview or no interactions).")
            .style(|theme: &Theme| text::Style {
                color: Some(
                    theme
                        .extended_palette()
                        .background
                        .weakest
                        .text
                        .scale_alpha(0.6),
                ),
            })
            .into()
    };

    scrollable(
        column![
            // View function stats
            section_header("View Function"),
            view_section,
            space::vertical().height(16),
            // Update function stats (only for stateful previews)
            section_header("Update Function"),
            update_section,
        ]
        .spacing(8)
        .width(Fill),
    )
    .into()
}

/// A section header for performance stats.
fn section_header<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(14))
        .padding([4, 0])
        .style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().primary.base.color),
            ..Default::default()
        })
        .into()
}

/// A grid displaying timing statistics.
fn stats_grid(stats: crate::preview::Stats) -> Element<'static, Message> {
    column![
        stat_row("Calls", format!("{}", stats.count)),
        stat_row("Last", format_duration(stats.last)),
        stat_row("Average", format_duration(stats.avg)),
        stat_row("Min", format_duration(stats.min)),
        stat_row("Max", format_duration(stats.max)),
    ]
    .spacing(4)
    .into()
}

/// A single row in the stats grid showing a label and value.
fn stat_row(label: &'static str, value: String) -> Element<'static, Message> {
    row![
        container(text(label).size(13))
            .width(80)
            .style(|theme: &Theme| container::Style {
                text_color: Some(
                    theme
                        .extended_palette()
                        .background
                        .weakest
                        .text
                        .scale_alpha(0.7),
                ),
                ..Default::default()
            }),
        text(value).size(13),
    ]
    .align_y(Center)
    .spacing(8)
    .into()
}

/// Format a duration for display, showing appropriate units.
fn format_duration(duration: Option<Duration>) -> String {
    match duration {
        None => "—".to_string(),
        Some(d) => {
            let micros = d.as_micros();
            if micros < 1_000 {
                format!("{}µs", micros)
            } else if micros < 1_000_000 {
                format!("{:.2}ms", micros as f64 / 1_000.0)
            } else {
                format!("{:.2}s", micros as f64 / 1_000_000.0)
            }
        }
    }
}

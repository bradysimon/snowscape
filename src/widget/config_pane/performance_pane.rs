use std::time::Duration;

use iced::{
    Alignment::Center,
    Element,
    Length::Fill,
    Theme, border,
    widget::{column, container, row, scrollable, space, text},
};

use crate::app::Message;
use crate::preview::performance::{Indicator, Performance, Stats};

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
    .spacing(2)
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
fn stats_grid(stats: Stats) -> Element<'static, Message> {
    column![
        // Core stats
        stat_row("Calls", format!("{}", stats.count)),
        stat_row("Last", format_duration(stats.last)),
        stat_row("Average", format_duration(stats.avg)),
        space::vertical().height(4),
        // Percentiles
        subsection_header("Percentiles"),
        stat_row("p50", format_duration(stats.p50)),
        stat_row("p90", format_duration(stats.p90)),
        stat_row("p99", format_duration(stats.p99)),
        space::vertical().height(4),
        // Range
        subsection_header("Range"),
        stat_row("Min", format_duration(stats.min)),
        stat_row("Max", format_duration(stats.max)),
        space::vertical().height(4),
        // Jank
        jank_indicator(stats.jank_count, stats.count),
    ]
    .spacing(2)
    .into()
}

/// A subsection header within the stats grid.
fn subsection_header(label: &'static str) -> Element<'static, Message> {
    text(label)
        .size(11)
        .style(|theme: &Theme| text::Style {
            color: Some(
                theme
                    .extended_palette()
                    .background
                    .weakest
                    .text
                    .scale_alpha(0.5),
            ),
        })
        .into()
}

/// Jank indicator showing how many frames exceeded the budget.
fn jank_indicator(jank_count: usize, total_count: usize) -> Element<'static, Message> {
    if total_count == 0 {
        return space::vertical().height(0).into();
    }

    let jank_percentage = if total_count > 0 {
        (jank_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };

    let (status_text, status_color) = if jank_count == 0 {
        ("No jank detected", iced::Color::from_rgb(0.2, 0.8, 0.3))
    } else if jank_percentage < 1.0 {
        ("Occasional jank", iced::Color::from_rgb(0.7, 0.8, 0.2))
    } else if jank_percentage < 5.0 {
        ("Some jank", iced::Color::from_rgb(0.9, 0.6, 0.1))
    } else {
        ("Frequent jank", iced::Color::from_rgb(0.9, 0.3, 0.2))
    };

    row![
        container(space::horizontal())
            .width(8)
            .height(8)
            .style(move |_theme: &Theme| container::Style {
                background: Some(status_color.into()),
                border: border::rounded(4),
                ..Default::default()
            }),
        text(if jank_count > 0 {
            format!(
                "{} ({} frames, {:.1}%)",
                status_text, jank_count, jank_percentage
            )
        } else {
            status_text.to_string()
        })
        .size(12),
    ]
    .align_y(Center)
    .spacing(6)
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

/// A colored status dot indicator for showing performance status.
pub fn status_dot(status: Indicator) -> Element<'static, Message> {
    container(space::horizontal())
        .width(8)
        .height(8)
        .style(move |theme: &Theme| container::Style {
            background: match status {
                Indicator::Healthy => Some(theme.extended_palette().success.strong.color.into()),
                Indicator::Degraded => Some(theme.palette().warning.into()),
                Indicator::Severe => Some(theme.palette().danger.into()),
                Indicator::Unknown => {
                    Some(theme.extended_palette().background.neutral.color.into())
                }
            },
            border: border::rounded(4),
            ..Default::default()
        })
        .into()
}

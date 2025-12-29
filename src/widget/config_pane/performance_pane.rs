use std::time::Duration;

use iced::{
    Alignment::Center,
    Element,
    Length::{self, Fill, FillPortion},
    Theme, border,
    widget::{Container, column, container, responsive, right, row, scrollable, space, text},
};

use crate::app::Message;
use crate::preview::performance::{Indicator, Performance, Stats};
use crate::style;

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

    scrollable(responsive(move |size| {
        let view_section: Element<'_, Message> = if has_view_data {
            stats_grid(view_stats)
        } else {
            text("No view data recorded.").into()
        };

        let update_section: Element<'_, Message> = if has_update_data {
            stats_grid(update_stats)
        } else {
            text("No update data recorded (stateless preview or no interactions).")
                .style(style::text::secondary)
                .into()
        };

        if size.width >= 576.0 {
            row![
                section("View", view_section).width(FillPortion(1)),
                container(space::vertical())
                    .width(1)
                    .height(Fill)
                    .style(container::rounded_box),
                section("Update", update_section).width(FillPortion(1)),
            ]
            .spacing(8)
            .width(Fill)
            .into()
        } else {
            column![
                section("View", view_section),
                container(space::horizontal())
                    .height(1)
                    .width(Fill)
                    .style(container::rounded_box),
                section("Update", update_section),
            ]
            .spacing(8)
            .width(Fill)
            .into()
        }
    }))
    .spacing(2)
    .into()
}

/// A section header for performance stats.
fn section<'a>(label: &'a str, content: Element<'a, Message>) -> Container<'a, Message> {
    container(column![text(label), content]).padding(4)
}

/// A grid displaying timing statistics.
fn stats_grid<'a>(stats: Stats) -> Element<'a, Message> {
    column![
        // Stats around total calls and last call
        row![
            stat_row("Calls", format!("{}", stats.count)),
            stat_row("Last", format_duration(stats.last)),
            right(jank_indicator(
                stats.indicator(),
                stats.jank_count,
                stats.count
            )),
        ]
        .align_y(Center)
        .spacing(8),
        space::vertical().height(4),
        // Visual range display
        subsection_header("Timing Range"),
        timing_range_bar(stats),
        space::vertical().height(4),
        // Percentiles
        subsection_header("Percentiles"),
        percentile_bars(stats),
    ]
    .spacing(2)
    .into()
}

/// A subsection header within the stats grid.
fn subsection_header<'a>(label: &'static str) -> Element<'a, Message> {
    text(label).size(11).style(style::text::faded).into()
}

/// A horizontal bar visualization showing min, average, and max timing.
fn timing_range_bar<'a>(stats: Stats) -> Element<'a, Message> {
    let (Some(min), Some(max), Some(avg)) = (stats.min, stats.max, stats.avg) else {
        return text("—").size(12).into();
    };

    // If min == max, just show a single value
    if min == max {
        return row![
            text(format_duration(Some(min))).size(12),
            text(" (no variance)").size(11).style(style::text::faded),
        ]
        .align_y(Center)
        .into();
    }

    // Calculate position of average within the range (0.0 to 1.0)
    let range = max.as_nanos() - min.as_nanos();
    let avg_position = if range > 0 {
        ((avg.as_nanos() - min.as_nanos()) as f64 / range as f64).clamp(0.0, 1.0)
    } else {
        0.5
    };

    // Convert to fill portions (use 1000 as scale for precision)
    let left_portion = (avg_position * 1000.0) as u16;
    let right_portion = 1000 - left_portion;

    let min_label = format_duration(Some(min));
    let avg_label = format_duration(Some(avg));
    let max_label = format_duration(Some(max));

    column![
        // The visual bar
        container(
            row![
                // Left portion (min to avg)
                container(space::horizontal())
                    .width(Length::FillPortion(left_portion.max(1)))
                    .height(6)
                    .style(|theme: &Theme| container::Style {
                        background: Some(
                            theme
                                .extended_palette()
                                .primary
                                .weak
                                .color
                                .scale_alpha(0.5)
                                .into()
                        ),
                        border: border::rounded(border::left(2)),
                        ..Default::default()
                    }),
                // Average marker
                container(space::horizontal())
                    .width(3)
                    .height(12)
                    .style(|theme: &Theme| container::Style {
                        background: Some(theme.extended_palette().primary.base.color.into()),
                        border: border::rounded(1),
                        ..Default::default()
                    }),
                // Right portion (avg to max)
                container(space::horizontal())
                    .width(Length::FillPortion(right_portion.max(1)))
                    .height(6)
                    .style(|theme: &Theme| container::Style {
                        background: Some(
                            theme
                                .extended_palette()
                                .primary
                                .weak
                                .color
                                .scale_alpha(0.5)
                                .into()
                        ),
                        border: border::rounded(border::right(2)),
                        ..Default::default()
                    }),
            ]
            .align_y(Center)
            .width(Fill),
        )
        .width(Fill)
        .padding([0, 1]),
        // Labels row
        row![
            text(min_label).size(12).style(style::text::secondary),
            space::horizontal(),
            text(format!("avg: {}", avg_label))
                .size(12)
                .style(style::text::secondary),
            space::horizontal(),
            text(max_label).size(12).style(style::text::secondary),
        ]
        .width(Fill),
    ]
    .spacing(2)
    .into()
}

/// Visual percentile bars showing p50, p90, and p99 on the same scale.
fn percentile_bars<'a>(stats: Stats) -> Element<'a, Message> {
    let (Some(p50), Some(p90), Some(p99)) = (stats.p50, stats.p90, stats.p99) else {
        return text("—").size(12).into();
    };

    // Use p99 as the max scale (100%)
    let max_nanos = p99.as_nanos().max(1) as f64;

    let p50_pct = (p50.as_nanos() as f64 / max_nanos).clamp(0.0, 1.0);
    let p90_pct = (p90.as_nanos() as f64 / max_nanos).clamp(0.0, 1.0);
    // p99 is always 100% since it's the max

    column![
        percentile_bar_row("p50", p50, p50_pct),
        percentile_bar_row("p90", p90, p90_pct),
        percentile_bar_row("p99", p99, 1.0),
    ]
    .spacing(3)
    .into()
}

/// A single percentile bar row with label, bar, and value.
fn percentile_bar_row<'a>(
    label: &'static str,
    duration: Duration,
    fill_pct: f64,
) -> Element<'a, Message> {
    let fill_portion = (fill_pct * 1000.0) as u16;

    row![
        // Label
        text(label).size(11).style(style::text::muted),
        // Bar track (background) with fill inside
        container(row![
                container(space::horizontal())
                    .width(Length::FillPortion(fill_portion.max(1)))
                    .height(Fill)
                    .style(|theme: &Theme| container::Style {
                        background: Some(theme.extended_palette().primary.weak.color.into()),
                        border: border::rounded(2),
                        ..Default::default()
                    }),
                (fill_pct < 1.0)
                    .then(|| space::horizontal()
                        .width(Length::FillPortion(1000 - fill_portion.max(1)))),
            ])
        .width(Fill)
        .height(8)
        .style(|theme: &Theme| container::Style {
            background: Some(
                theme
                    .extended_palette()
                    .background
                    .weak
                    .color
                    .scale_alpha(0.3)
                    .into(),
            ),
            border: border::rounded(2),
            ..Default::default()
        }),
        // Value (fixed width, right-aligned text)
        container(
            text(format_duration(Some(duration)))
                .size(11)
                .style(style::text::subdued)
        )
        .width(40)
        .align_x(iced::alignment::Horizontal::Right),
    ]
    .align_y(Center)
    .spacing(6)
    .width(Fill)
    .into()
}

/// Jank indicator showing how many frames exceeded the budget.
fn jank_indicator<'a>(
    indicator: Indicator,
    jank_count: usize,
    total_count: usize,
) -> Element<'a, Message> {
    if total_count == 0 {
        return space::vertical().height(0).into();
    }

    let jank_percentage = if total_count > 0 {
        (jank_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };

    row![
        indicator_dot(indicator),
        text(format!(
            "{} jank frames, {:.1}%",
            jank_count, jank_percentage
        ))
        .size(12),
    ]
    .align_y(Center)
    .spacing(6)
    .into()
}

/// A single row in the stats grid showing a label and value.
fn stat_row<'a>(label: &'static str, value: String) -> Element<'a, Message> {
    row![
        text(label).size(12).style(style::text::secondary),
        text(value).size(14),
    ]
    .width(60)
    .align_y(Center)
    .spacing(4)
    .into()
}

/// Format a duration for display, showing appropriate units.
fn format_duration(duration: Option<Duration>) -> String {
    match duration {
        None => "—".to_string(),
        Some(d) => {
            let nanos = d.as_nanos();
            if nanos < 1_000 {
                format!("{}ns", nanos)
            } else if nanos < 1_000_000 {
                format!("{}µs", d.as_micros())
            } else if nanos < 1_000_000_000 {
                format!("{:.2}ms", nanos as f64 / 1_000_000.0)
            } else {
                format!("{:.2}s", nanos as f64 / 1_000_000_000.0)
            }
        }
    }
}

/// A colored dot indicating the current overall performance status.
pub fn indicator_dot<'a>(status: Indicator) -> Element<'a, Message> {
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

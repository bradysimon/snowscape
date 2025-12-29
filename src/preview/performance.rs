use std::{
    cell::RefCell,
    fmt::Display,
    time::{Duration, Instant},
};

/// Maximum number of timing entries to store per metric type.
const MAX_ENTRIES: usize = 1_000_000;

/// Performance metrics for tracking view and update function execution times.
#[derive(Debug, Default)]
pub struct Performance {
    /// Recorded durations for view function calls.
    view_times: RefCell<Vec<Duration>>,
    /// Recorded durations for update function calls.
    update_times: Vec<Duration>,
}

impl Performance {
    /// Create a new empty `Performance` tracker.
    pub fn new(view: Vec<Duration>, update: Vec<Duration>) -> Self {
        Self {
            view_times: RefCell::new(view),
            update_times: update,
        }
    }

    /// Record a view function execution, timing the provided closure.
    ///
    /// Returns the result of the closure.
    pub fn record_view<T>(&self, f: impl FnOnce() -> T) -> T {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();

        let mut times = self.view_times.borrow_mut();
        if times.len() < MAX_ENTRIES {
            times.push(elapsed);
        }

        result
    }

    /// Record an update function execution, timing the provided closure.
    ///
    /// Returns the result of the closure.
    pub fn record_update<T>(&mut self, f: impl FnOnce() -> T) -> T {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();

        if self.update_times.len() < MAX_ENTRIES {
            self.update_times.push(elapsed);
        }

        result
    }

    /// Reset all performance metrics.
    pub fn reset(&mut self) {
        self.view_times.borrow_mut().clear();
        self.update_times.clear();
    }

    /// Get the number of recorded view function calls.
    pub fn view_count(&self) -> usize {
        self.view_times.borrow().len()
    }

    /// Get the number of recorded update function calls.
    pub fn update_count(&self) -> usize {
        self.update_times.len()
    }

    /// Get the last recorded view duration.
    pub fn last_view_time(&self) -> Option<Duration> {
        self.view_times.borrow().last().copied()
    }

    /// Get the last recorded update duration.
    pub fn last_update_time(&self) -> Option<Duration> {
        self.update_times.last().copied()
    }

    /// Get the average view duration.
    pub fn avg_view_time(&self) -> Option<Duration> {
        let times = self.view_times.borrow();
        if times.is_empty() {
            None
        } else {
            let total: Duration = times.iter().sum();
            Some(total / times.len() as u32)
        }
    }

    /// Get the average update duration.
    pub fn avg_update_time(&self) -> Option<Duration> {
        if self.update_times.is_empty() {
            None
        } else {
            let total: Duration = self.update_times.iter().sum();
            Some(total / self.update_times.len() as u32)
        }
    }

    /// Get the minimum view duration.
    pub fn min_view_time(&self) -> Option<Duration> {
        self.view_times.borrow().iter().min().copied()
    }

    /// Get the minimum update duration.
    pub fn min_update_time(&self) -> Option<Duration> {
        self.update_times.iter().min().copied()
    }

    /// Get the maximum view duration.
    pub fn max_view_time(&self) -> Option<Duration> {
        self.view_times.borrow().iter().max().copied()
    }

    /// Get the maximum update duration.
    pub fn max_update_time(&self) -> Option<Duration> {
        self.update_times.iter().max().copied()
    }

    /// Get view timing statistics as a [`Stats`] struct.
    pub fn view_stats(&self) -> Stats {
        let times = self.view_times.borrow();
        let (p50, p90, p99) = compute_percentiles(&times);
        let jank_count = times.iter().filter(|&&d| d > FRAME_BUDGET_120FPS).count();
        Stats {
            count: times.len(),
            last: times.last().copied(),
            avg: if times.is_empty() {
                None
            } else {
                let total: Duration = times.iter().sum();
                Some(total / times.len() as u32)
            },
            min: times.iter().min().copied(),
            max: times.iter().max().copied(),
            p50,
            p90,
            p99,
            jank_count,
        }
    }

    /// Get update timing statistics as a [`Stats`] struct.
    pub fn update_stats(&self) -> Stats {
        let (p50, p90, p99) = compute_percentiles(&self.update_times);
        let jank_count = self
            .update_times
            .iter()
            .filter(|&&d| d > FRAME_BUDGET_120FPS)
            .count();
        Stats {
            count: self.update_count(),
            last: self.last_update_time(),
            avg: self.avg_update_time(),
            min: self.min_update_time(),
            max: self.max_update_time(),
            p50,
            p90,
            p99,
            jank_count,
        }
    }

    /// Get the overall performance status combining view and update stats.
    pub fn overall_status(&self) -> Indicator {
        let view_status = self.view_stats().indicator();
        let update_status = self.update_stats().indicator();
        view_status.combine(update_status)
    }
}

/// Compute percentiles (p50, p90, p99) from a slice of durations.
fn compute_percentiles(
    times: &[Duration],
) -> (Option<Duration>, Option<Duration>, Option<Duration>) {
    if times.is_empty() {
        return (None, None, None);
    }

    let mut sorted: Vec<Duration> = times.to_vec();
    sorted.sort();

    let p50 = percentile(&sorted, 50);
    let p90 = percentile(&sorted, 90);
    let p99 = percentile(&sorted, 99);

    (Some(p50), Some(p90), Some(p99))
}

/// Get the value at a given percentile from a sorted slice.
fn percentile(sorted: &[Duration], p: usize) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let index = (p * sorted.len() / 100).min(sorted.len() - 1);
    sorted[index]
}

/// Computed statistics for a set of timing measurements.
#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    /// Number of recorded measurements.
    pub count: usize,
    /// The most recent measurement.
    pub last: Option<Duration>,
    /// Average of all measurements.
    pub avg: Option<Duration>,
    /// Minimum measurement.
    pub min: Option<Duration>,
    /// Maximum measurement.
    pub max: Option<Duration>,
    /// 50th percentile (median).
    pub p50: Option<Duration>,
    /// 90th percentile.
    pub p90: Option<Duration>,
    /// 99th percentile.
    pub p99: Option<Duration>,
    /// Number of measurements exceeding 16.67ms (60 FPS budget).
    pub jank_count: usize,
}

impl Stats {
    /// Compute the performance indicator for these stats.
    ///
    /// Uses p90 as the primary indicator since it represents
    /// what most users will experience in practice.
    pub fn indicator(&self) -> Indicator {
        let Some(p90) = self.p90 else {
            return Indicator::Unknown;
        };

        // Use p90 as primary indicator with jank percentage as secondary
        let jank_percentage = if self.count > 0 {
            (self.jank_count as f64 / self.count as f64) * 100.0
        } else {
            0.0
        };

        // Good: p90 under 4ms (plenty of headroom) and <1% jank
        if p90.as_micros() <= 4_000 && jank_percentage < 1.0 {
            Indicator::Healthy
        }
        // Warning: p90 under 8ms (some headroom) or 1-5% jank
        else if p90.as_micros() <= 8_000 && jank_percentage < 5.0 {
            Indicator::Degraded
        }
        // Error: p90 over 8ms or >5% jank
        else {
            Indicator::Severe
        }
    }
}

/// Performance status indicator for quick visual feedback.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Indicator {
    /// Performance status is unknown (no data).
    #[default]
    Unknown,
    /// Performance is good (p90 ≤ 4ms, jank < 1%).
    Healthy,
    /// Performance may need attention (p90 ≤ 8ms, jank < 5%).
    Degraded,
    /// Performance issues detected (p90 > 8ms or jank ≥ 5%).
    Severe,
}

impl Display for Indicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Indicator::Unknown => write!(f, "Unknown"),
            Indicator::Healthy => write!(f, "Healthy"),
            Indicator::Degraded => write!(f, "Degraded"),
            Indicator::Severe => write!(f, "Severe"),
        }
    }
}

impl Indicator {
    /// All possible performance statuses.
    pub const ALL: [Indicator; 4] = [
        Indicator::Healthy,
        Indicator::Degraded,
        Indicator::Severe,
        Indicator::Unknown,
    ];

    /// Combine two statuses, returning the worse of the two.
    pub fn combine(self, other: Self) -> Self {
        match (self, other) {
            (Indicator::Severe, _) | (_, Indicator::Severe) => Indicator::Severe,
            (Indicator::Degraded, _) | (_, Indicator::Degraded) => Indicator::Degraded,
            (Indicator::Healthy, _) | (_, Indicator::Healthy) => Indicator::Healthy,
            (Indicator::Unknown, Indicator::Unknown) => Indicator::Unknown,
        }
    }
}

/// Frame budget threshold for 120 FPS (8.33ms).
pub const FRAME_BUDGET_120FPS: Duration = Duration::from_micros(8_333);

use std::{
    cell::RefCell,
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
    pub fn new() -> Self {
        Self {
            view_times: RefCell::new(Vec::new()),
            update_times: Vec::new(),
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
        Stats {
            count: self.view_count(),
            last: self.last_view_time(),
            avg: self.avg_view_time(),
            min: self.min_view_time(),
            max: self.max_view_time(),
        }
    }

    /// Get update timing statistics as a [`Stats`] struct.
    pub fn update_stats(&self) -> Stats {
        Stats {
            count: self.update_count(),
            last: self.last_update_time(),
            avg: self.avg_update_time(),
            min: self.min_update_time(),
            max: self.max_update_time(),
        }
    }
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
}

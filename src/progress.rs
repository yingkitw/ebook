//! Progress reporting utilities for long-running operations

use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

/// A simple progress reporter for tracking operation progress
#[derive(Clone)]
pub struct Progress {
    current: Arc<AtomicUsize>,
    total: usize,
    name: String,
}

impl Progress {
    /// Create a new progress indicator
    pub fn new(name: String, total: usize) -> Self {
        Self {
            current: Arc::new(AtomicUsize::new(0)),
            total,
            name,
        }
    }

    /// Increment the progress counter
    pub fn increment(&self, amount: usize) {
        self.current.fetch_add(amount, Ordering::Relaxed);
    }

    /// Set the current progress value
    pub fn set(&self, value: usize) {
        self.current.store(value, Ordering::Relaxed);
    }

    /// Get the current progress value
    pub fn current(&self) -> usize {
        self.current.load(Ordering::Relaxed)
    }

    /// Get the total value
    pub fn total(&self) -> usize {
        self.total
    }

    /// Get the progress as a percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        let current = self.current();
        (current as f64 / self.total as f64 * 100.0).min(100.0)
    }

    /// Get the operation name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Print the current progress to stderr
    pub fn print(&self) {
        eprint!("\r{}: {:.0}% ({}/{})",
            self.name,
            self.percentage(),
            self.current(),
            self.total
        );
    }

    /// Print the current progress with a message
    pub fn print_with_message(&self, message: &str) {
        eprint!("\r{}: {:.0}% - {}",
            self.name,
            self.percentage(),
            message
        );
    }

    /// Finish the progress display
    pub fn finish(&self) {
        eprintln!("\r{}: Complete! (100%)", self.name);
    }

    /// Finish the progress display with a message
    pub fn finish_with_message(&self, message: &str) {
        eprintln!("\r{}: {}", self.name, message);
    }
}

/// A callback type for progress updates
pub type ProgressCallback = Box<dyn Fn(usize, usize) + Send + Sync>;

/// A progress handler that can be passed to operations
pub struct ProgressHandler {
    callback: Option<ProgressCallback>,
}

impl ProgressHandler {
    /// Create a new progress handler without callback
    pub fn new() -> Self {
        Self { callback: None }
    }

    /// Create a new progress handler with a callback
    pub fn with_callback(callback: ProgressCallback) -> Self {
        Self { callback: Some(callback) }
    }

    /// Report progress
    pub fn report(&self, current: usize, total: usize) {
        if let Some(ref callback) = self.callback {
            callback(current, total);
        }
    }

    /// Check if this handler has a callback
    pub fn has_callback(&self) -> bool {
        self.callback.is_some()
    }
}

impl Default for ProgressHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a simple console progress callback
pub fn console_progress_callback(name: String) -> ProgressCallback {
    Box::new(move |current: usize, total: usize| {
        let percentage = if total > 0 {
            (current as f64 / total as f64 * 100.0).min(100.0)
        } else {
            100.0
        };
        eprint!("\r{name}: {percentage:.0}% ({current}/{total})");
    })
}

/// Create a silent progress callback (does nothing)
pub fn silent_progress_callback() -> ProgressCallback {
    Box::new(|_current: usize, _total: usize| {})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_creation() {
        let progress = Progress::new("Test".to_string(), 100);
        assert_eq!(progress.current(), 0);
        assert_eq!(progress.total(), 100);
        assert_eq!(progress.name(), "Test");
    }

    #[test]
    fn test_progress_increment() {
        let progress = Progress::new("Test".to_string(), 100);
        progress.increment(10);
        assert_eq!(progress.current(), 10);
        progress.increment(20);
        assert_eq!(progress.current(), 30);
    }

    #[test]
    fn test_progress_set() {
        let progress = Progress::new("Test".to_string(), 100);
        progress.set(50);
        assert_eq!(progress.current(), 50);
    }

    #[test]
    fn test_progress_percentage() {
        let progress = Progress::new("Test".to_string(), 100);
        assert_eq!(progress.percentage(), 0.0);
        progress.set(50);
        assert_eq!(progress.percentage(), 50.0);
        progress.set(100);
        assert_eq!(progress.percentage(), 100.0);
        progress.set(150); // Cap at 100%
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_progress_zero_total() {
        let progress = Progress::new("Test".to_string(), 0);
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_progress_handler() {
        let handler = ProgressHandler::new();
        assert!(!handler.has_callback());
        handler.report(10, 100); // Should not panic

        let callback = silent_progress_callback();
        let handler_with_cb = ProgressHandler::with_callback(callback);
        assert!(handler_with_cb.has_callback());
        handler_with_cb.report(50, 100); // Should not panic
    }
}

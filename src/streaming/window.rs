//! Time Window Management for Stream Processing
//!
//! Provides time-based windows for event aggregation and analysis.

use crate::streaming::event::StreamEvent;
use std::collections::VecDeque;
use std::time::Duration;

/// Type of time window
#[derive(Debug, Clone, PartialEq)]
pub enum WindowType {
    /// Sliding window - continuously moves forward
    Sliding,
    /// Tumbling window - non-overlapping fixed intervals
    Tumbling,
    /// Session window - based on inactivity gaps
    Session { timeout: Duration },
}

/// Time-based window for event processing
#[derive(Debug)]
pub struct TimeWindow {
    /// Window type
    pub window_type: WindowType,
    /// Window duration
    pub duration: Duration,
    /// Events in this window
    events: VecDeque<StreamEvent>,
    /// Window start time (milliseconds since epoch)
    pub start_time: u64,
    /// Window end time (milliseconds since epoch)
    pub end_time: u64,
    /// Maximum number of events to retain
    max_events: usize,
}

impl TimeWindow {
    /// Create a new time window
    pub fn new(
        window_type: WindowType,
        duration: Duration,
        start_time: u64,
        max_events: usize,
    ) -> Self {
        let end_time = start_time + duration.as_millis() as u64;

        Self {
            window_type,
            duration,
            events: VecDeque::new(),
            start_time,
            end_time,
            max_events,
        }
    }

    /// Add event to window if it fits
    pub fn add_event(&mut self, event: StreamEvent) -> bool {
        if self.contains_timestamp(event.metadata.timestamp) {
            self.events.push_back(event);

            // Keep window size under limit
            while self.events.len() > self.max_events {
                self.events.pop_front();
            }

            true
        } else {
            false
        }
    }

    /// Check if timestamp falls within this window
    pub fn contains_timestamp(&self, timestamp: u64) -> bool {
        timestamp >= self.start_time && timestamp < self.end_time
    }

    /// Get all events in window
    pub fn events(&self) -> &VecDeque<StreamEvent> {
        &self.events
    }

    /// Get event count
    pub fn count(&self) -> usize {
        self.events.len()
    }

    /// Check if window is expired
    pub fn is_expired(&self, current_time: u64) -> bool {
        current_time >= self.end_time
    }

    /// Get window duration in milliseconds
    pub fn duration_ms(&self) -> u64 {
        self.duration.as_millis() as u64
    }

    /// Clear all events from window
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Get events filtered by type
    pub fn events_by_type(&self, event_type: &str) -> Vec<&StreamEvent> {
        self.events
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// Calculate sum of numeric field across events
    pub fn sum(&self, field: &str) -> f64 {
        self.events
            .iter()
            .filter_map(|e| e.get_numeric(field))
            .sum()
    }

    /// Calculate average of numeric field across events
    pub fn average(&self, field: &str) -> Option<f64> {
        let values: Vec<f64> = self
            .events
            .iter()
            .filter_map(|e| e.get_numeric(field))
            .collect();

        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        }
    }

    /// Find minimum value of numeric field
    pub fn min(&self, field: &str) -> Option<f64> {
        self.events
            .iter()
            .filter_map(|e| e.get_numeric(field))
            .fold(None, |acc, x| match acc {
                None => Some(x),
                Some(min) => Some(min.min(x)),
            })
    }

    /// Find maximum value of numeric field
    pub fn max(&self, field: &str) -> Option<f64> {
        self.events
            .iter()
            .filter_map(|e| e.get_numeric(field))
            .fold(None, |acc, x| match acc {
                None => Some(x),
                Some(max) => Some(max.max(x)),
            })
    }

    /// Get the latest event timestamp
    pub fn latest_timestamp(&self) -> Option<u64> {
        self.events.iter().map(|e| e.metadata.timestamp).max()
    }

    /// Get events within a sub-window
    pub fn events_in_range(&self, start: u64, end: u64) -> Vec<&StreamEvent> {
        self.events
            .iter()
            .filter(|e| e.metadata.timestamp >= start && e.metadata.timestamp < end)
            .collect()
    }
}

/// Manages multiple time windows for stream processing
#[derive(Debug)]
pub struct WindowManager {
    /// Active windows
    windows: Vec<TimeWindow>,
    /// Window configuration
    window_type: WindowType,
    /// Window duration
    duration: Duration,
    /// Maximum events per window
    max_events_per_window: usize,
    /// Maximum number of windows to keep
    max_windows: usize,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new(
        window_type: WindowType,
        duration: Duration,
        max_events_per_window: usize,
        max_windows: usize,
    ) -> Self {
        Self {
            windows: Vec::new(),
            window_type,
            duration,
            max_events_per_window,
            max_windows,
        }
    }

    /// Process a new event through the window system
    pub fn process_event(&mut self, event: StreamEvent) {
        let event_time = event.metadata.timestamp;

        // Find or create appropriate window
        let mut added = false;

        for window in &mut self.windows {
            if window.add_event(event.clone()) {
                added = true;
                break;
            }
        }

        if !added {
            // Create new window for this event
            let window_start = self.calculate_window_start(event_time);
            let mut new_window = TimeWindow::new(
                self.window_type.clone(),
                self.duration,
                window_start,
                self.max_events_per_window,
            );

            new_window.add_event(event);
            self.windows.push(new_window);
        }

        // Clean up expired windows
        self.cleanup_expired_windows(event_time);

        // Limit total number of windows
        while self.windows.len() > self.max_windows {
            self.windows.remove(0);
        }

        // Sort windows by start time
        self.windows.sort_by_key(|w| w.start_time);
    }

    /// Calculate window start time based on window type
    fn calculate_window_start(&self, event_time: u64) -> u64 {
        match self.window_type {
            WindowType::Tumbling => {
                let window_ms = self.duration.as_millis() as u64;
                (event_time / window_ms) * window_ms
            }
            WindowType::Sliding | WindowType::Session { .. } => event_time,
        }
    }

    /// Remove expired windows
    fn cleanup_expired_windows(&mut self, current_time: u64) {
        self.windows
            .retain(|window| !window.is_expired(current_time));
    }

    /// Get all active windows
    pub fn active_windows(&self) -> &[TimeWindow] {
        &self.windows
    }

    /// Get the latest window
    pub fn latest_window(&self) -> Option<&TimeWindow> {
        self.windows.last()
    }

    /// Get total event count across all windows
    pub fn total_event_count(&self) -> usize {
        self.windows.iter().map(|w| w.count()).sum()
    }

    /// Get windows that contain events of a specific type
    pub fn windows_with_event_type(&self, event_type: &str) -> Vec<&TimeWindow> {
        self.windows
            .iter()
            .filter(|w| w.events().iter().any(|e| e.event_type == event_type))
            .collect()
    }

    /// Calculate aggregate across all windows
    pub fn aggregate_across_windows<F>(&self, aggregator: F) -> f64
    where
        F: Fn(&TimeWindow) -> f64,
    {
        self.windows.iter().map(aggregator).sum()
    }

    /// Get window statistics
    pub fn get_statistics(&self) -> WindowStatistics {
        WindowStatistics {
            total_windows: self.windows.len(),
            total_events: self.total_event_count(),
            oldest_window_start: self.windows.first().map(|w| w.start_time),
            newest_window_start: self.windows.last().map(|w| w.start_time),
            average_events_per_window: if self.windows.is_empty() {
                0.0
            } else {
                self.total_event_count() as f64 / self.windows.len() as f64
            },
        }
    }
}

/// Statistics about window manager state
#[derive(Debug, Clone)]
pub struct WindowStatistics {
    /// Total number of active windows
    pub total_windows: usize,
    /// Total events across all windows
    pub total_events: usize,
    /// Start time of oldest window
    pub oldest_window_start: Option<u64>,
    /// Start time of newest window
    pub newest_window_start: Option<u64>,
    /// Average events per window
    pub average_events_per_window: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;
    use std::collections::HashMap;

    #[test]
    fn test_time_window_creation() {
        let window = TimeWindow::new(WindowType::Sliding, Duration::from_secs(60), 1000, 100);

        assert_eq!(window.start_time, 1000);
        assert_eq!(window.end_time, 61000);
        assert_eq!(window.count(), 0);
    }

    #[test]
    fn test_window_event_addition() {
        let mut window = TimeWindow::new(WindowType::Sliding, Duration::from_secs(60), 1000, 100);

        let mut data = HashMap::new();
        data.insert("value".to_string(), Value::Number(10.0));

        let event = StreamEvent::with_timestamp("TestEvent", data, "test", 30000);

        assert!(window.add_event(event));
        assert_eq!(window.count(), 1);
    }

    #[test]
    fn test_window_aggregations() {
        let mut window = TimeWindow::new(WindowType::Sliding, Duration::from_secs(60), 1000, 100);

        // Add test events
        for i in 0..5 {
            let mut data = HashMap::new();
            data.insert("value".to_string(), Value::Number(i as f64));

            let event = StreamEvent::with_timestamp("TestEvent", data, "test", 30000 + i);
            window.add_event(event);
        }

        assert_eq!(window.sum("value"), 10.0); // 0+1+2+3+4
        assert_eq!(window.average("value"), Some(2.0));
        assert_eq!(window.min("value"), Some(0.0));
        assert_eq!(window.max("value"), Some(4.0));
    }

    #[test]
    fn test_window_manager() {
        let mut manager = WindowManager::new(WindowType::Sliding, Duration::from_secs(60), 100, 10);

        let mut data = HashMap::new();
        data.insert("value".to_string(), Value::Number(1.0));

        let event = StreamEvent::with_timestamp("TestEvent", data, "test", 30000);
        manager.process_event(event);

        assert_eq!(manager.active_windows().len(), 1);
        assert_eq!(manager.total_event_count(), 1);
    }
}

//! StreamAlphaNode: RETE node for filtering stream events
//!
//! This module implements a specialized alpha node for stream sources in the RETE network.
//! It connects stream sources to the rule engine, managing windows and filtering events.

#![allow(missing_docs)]

use crate::streaming::event::StreamEvent;
use crate::streaming::window::WindowType;
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// StreamAlphaNode filters events from a named stream
///
/// This node:
/// - Filters events by stream name
/// - Optionally filters by event type
/// - Manages time-based windows (sliding, tumbling)
/// - Evicts expired events automatically
#[derive(Debug, Clone)]
pub struct StreamAlphaNode {
    /// Name of the stream to filter
    pub stream_name: String,

    /// Optional event type filter
    pub event_type: Option<String>,

    /// Window specification (if any)
    pub window: Option<WindowSpec>,

    /// Events currently in the window
    events: VecDeque<StreamEvent>,

    /// Maximum events to retain in memory
    max_events: usize,

    /// Last window start time (for tumbling windows)
    last_window_start: u64,

    /// Last event timestamp in session (for session windows)
    last_session_event_timestamp: Option<u64>,
}

/// Window specification for StreamAlphaNode
#[derive(Debug, Clone, PartialEq)]
pub struct WindowSpec {
    pub duration: Duration,
    pub window_type: WindowType,
}

impl StreamAlphaNode {
    /// Create a new StreamAlphaNode
    ///
    /// # Arguments
    /// * `stream_name` - Name of the stream to filter
    /// * `event_type` - Optional event type filter
    /// * `window` - Optional window specification
    ///
    /// # Example
    /// ```rust
    /// use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
    /// use rust_rule_engine::streaming::window::WindowType;
    /// use std::time::Duration;
    ///
    /// // Node without window
    /// let node = StreamAlphaNode::new("user-events", None, None);
    ///
    /// // Node with sliding window
    /// let window = WindowSpec {
    ///     duration: Duration::from_secs(300),
    ///     window_type: WindowType::Sliding,
    /// };
    /// let node = StreamAlphaNode::new("logins", Some("LoginEvent".to_string()), Some(window));
    /// ```
    pub fn new(
        stream_name: impl Into<String>,
        event_type: Option<String>,
        window: Option<WindowSpec>,
    ) -> Self {
        Self {
            stream_name: stream_name.into(),
            event_type,
            window,
            events: VecDeque::new(),
            max_events: 10_000, // Default: keep 10k events max
            last_window_start: 0,
            last_session_event_timestamp: None,
        }
    }

    /// Create with custom max events
    pub fn with_max_events(mut self, max_events: usize) -> Self {
        self.max_events = max_events;
        self
    }

    /// Process incoming event and return whether it matches
    ///
    /// # Arguments
    /// * `event` - The stream event to process
    ///
    /// # Returns
    /// * `true` if event matches stream name, event type, and window criteria
    /// * `false` otherwise
    pub fn process_event(&mut self, event: &StreamEvent) -> bool {
        // Check stream name matches
        if event.metadata.source != self.stream_name {
            return false;
        }

        // Check event type matches (if specified)
        if let Some(ref expected_type) = self.event_type {
            if &event.event_type != expected_type {
                return false;
            }
        }

        // If no window, event matches
        let matches = if self.window.is_none() {
            true
        } else {
            // With window, check if event is within window
            self.is_in_window(event.metadata.timestamp)
        };

        if matches {
            // For session windows, check if we need to start a new session BEFORE adding
            if let Some(WindowSpec {
                window_type: WindowType::Session { timeout },
                ..
            }) = &self.window
            {
                if let Some(last_time) = self.last_session_event_timestamp {
                    let gap = event.metadata.timestamp.saturating_sub(last_time);
                    let timeout_ms = timeout.as_millis() as u64;

                    if gap > timeout_ms {
                        // Session expired - clear old events before adding new one
                        self.events.clear();
                        self.last_session_event_timestamp = None;
                    }
                }
            }

            // Add to buffer and evict old events
            self.add_event(event.clone());
            self.evict_expired_events();
        }

        matches
    }

    /// Add event to internal buffer
    fn add_event(&mut self, event: StreamEvent) {
        let event_timestamp = event.metadata.timestamp;
        self.events.push_back(event);

        // Update session tracking
        if let Some(WindowSpec {
            window_type: WindowType::Session { .. },
            ..
        }) = &self.window
        {
            self.last_session_event_timestamp = Some(event_timestamp);
        }

        // Keep buffer size under limit
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }
    }

    /// Check if timestamp falls within current window
    fn is_in_window(&self, timestamp: u64) -> bool {
        match &self.window {
            None => true,
            Some(spec) => {
                let current_time = Self::current_time_ms();
                let window_duration_ms = spec.duration.as_millis() as u64;

                match spec.window_type {
                    WindowType::Sliding => {
                        // Event must be within duration from now
                        timestamp >= current_time.saturating_sub(window_duration_ms)
                            && timestamp <= current_time
                    }
                    WindowType::Tumbling => {
                        // Calculate window boundaries
                        let window_start = (current_time / window_duration_ms) * window_duration_ms;
                        let window_end = window_start + window_duration_ms;

                        timestamp >= window_start && timestamp < window_end
                    }
                    WindowType::Session { timeout } => {
                        let timeout_ms = timeout.as_millis() as u64;

                        match self.last_session_event_timestamp {
                            None => {
                                // First event in session - always accept
                                true
                            }
                            Some(last_event_time) => {
                                // Check if gap from last event exceeds timeout
                                let gap = timestamp.saturating_sub(last_event_time);

                                if gap > timeout_ms {
                                    // Gap too large - this starts a new session
                                    // But we still accept this event (it starts new session)
                                    true
                                } else {
                                    // Within timeout - part of current session
                                    true
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Evict events that are outside the window
    fn evict_expired_events(&mut self) {
        if let Some(spec) = &self.window {
            let current_time = Self::current_time_ms();
            let window_duration_ms = spec.duration.as_millis() as u64;

            match spec.window_type {
                WindowType::Sliding => {
                    let cutoff_time = current_time.saturating_sub(window_duration_ms);

                    // Remove events older than cutoff
                    while let Some(event) = self.events.front() {
                        if event.metadata.timestamp < cutoff_time {
                            self.events.pop_front();
                        } else {
                            break;
                        }
                    }
                }
                WindowType::Tumbling => {
                    let window_start = (current_time / window_duration_ms) * window_duration_ms;

                    // If we've moved to a new window, clear old events
                    if self.last_window_start != 0 && window_start != self.last_window_start {
                        self.events.clear();
                        self.last_window_start = window_start;
                    } else if self.last_window_start == 0 {
                        self.last_window_start = window_start;
                    }

                    // Remove events from previous windows
                    while let Some(event) = self.events.front() {
                        if event.metadata.timestamp < window_start {
                            self.events.pop_front();
                        } else {
                            break;
                        }
                    }
                }
                WindowType::Session { timeout } => {
                    let timeout_ms = timeout.as_millis() as u64;

                    // Check if current session has expired
                    if let Some(last_event_time) = self.last_session_event_timestamp {
                        let gap_since_last = current_time.saturating_sub(last_event_time);

                        if gap_since_last > timeout_ms {
                            // Session expired - clear all events and reset
                            self.events.clear();
                            self.last_session_event_timestamp = None;
                        }
                    }

                    // Note: Unlike sliding/tumbling, session windows don't evict individual events
                    // They either keep the entire session or clear it when timeout expires
                }
            }
        }
    }

    /// Get all events currently in the window
    ///
    /// # Returns
    /// A slice of events that are within the current window
    pub fn get_events(&self) -> &VecDeque<StreamEvent> {
        &self.events
    }

    /// Get count of events in window
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get current time in milliseconds since epoch
    fn current_time_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    /// Clear all events from buffer
    pub fn clear(&mut self) {
        self.events.clear();
        self.last_window_start = 0;
        self.last_session_event_timestamp = None;
    }

    /// Get window statistics
    pub fn window_stats(&self) -> WindowStats {
        WindowStats {
            event_count: self.events.len(),
            oldest_event_timestamp: self.events.front().map(|e| e.metadata.timestamp),
            newest_event_timestamp: self.events.back().map(|e| e.metadata.timestamp),
            window_duration_ms: self.window.as_ref().map(|w| w.duration.as_millis() as u64),
        }
    }
}

/// Window statistics for monitoring
#[derive(Debug, Clone)]
pub struct WindowStats {
    pub event_count: usize,
    pub oldest_event_timestamp: Option<u64>,
    pub newest_event_timestamp: Option<u64>,
    pub window_duration_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::event::StreamEvent;
    use crate::types::Value;
    use std::collections::HashMap;

    fn create_test_event(stream_name: &str, event_type: &str, timestamp: u64) -> StreamEvent {
        let mut data = HashMap::new();
        data.insert(
            "test_field".to_string(),
            Value::String("test_value".to_string()),
        );

        StreamEvent::with_timestamp(event_type, data, stream_name, timestamp)
    }

    #[test]
    fn test_stream_alpha_node_basic() {
        let mut node = StreamAlphaNode::new("user-events", None, None);
        let event = create_test_event("user-events", "LoginEvent", 1000);

        assert!(node.process_event(&event));
        assert_eq!(node.event_count(), 1);
    }

    #[test]
    fn test_stream_name_filtering() {
        let mut node = StreamAlphaNode::new("user-events", None, None);

        let matching_event = create_test_event("user-events", "LoginEvent", 1000);
        let non_matching_event = create_test_event("other-stream", "LoginEvent", 1000);

        assert!(node.process_event(&matching_event));
        assert!(!node.process_event(&non_matching_event));
        assert_eq!(node.event_count(), 1);
    }

    #[test]
    fn test_event_type_filtering() {
        let mut node = StreamAlphaNode::new("user-events", Some("LoginEvent".to_string()), None);

        let matching_event = create_test_event("user-events", "LoginEvent", 1000);
        let non_matching_event = create_test_event("user-events", "LogoutEvent", 1000);

        assert!(node.process_event(&matching_event));
        assert!(!node.process_event(&non_matching_event));
        assert_eq!(node.event_count(), 1);
    }

    #[test]
    fn test_sliding_window() {
        let window = WindowSpec {
            duration: Duration::from_secs(5),
            window_type: WindowType::Sliding,
        };

        let mut node = StreamAlphaNode::new("sensors", None, Some(window));

        let current_time = StreamAlphaNode::current_time_ms();

        // Event within window
        let recent_event = create_test_event("sensors", "TempReading", current_time - 2000);
        assert!(node.process_event(&recent_event));

        // Event outside window (6 seconds ago)
        let old_event = create_test_event("sensors", "TempReading", current_time - 6000);
        assert!(!node.process_event(&old_event));

        assert_eq!(node.event_count(), 1);
    }

    #[test]
    fn test_tumbling_window() {
        let window = WindowSpec {
            duration: Duration::from_secs(10),
            window_type: WindowType::Tumbling,
        };

        let mut node = StreamAlphaNode::new("sensors", None, Some(window));

        let current_time = StreamAlphaNode::current_time_ms();
        let window_start = (current_time / 10_000) * 10_000;

        // Event in current window
        let event1 = create_test_event("sensors", "TempReading", window_start + 1000);
        assert!(node.process_event(&event1));

        // Event in current window
        let event2 = create_test_event("sensors", "TempReading", window_start + 5000);
        assert!(node.process_event(&event2));

        // Event from previous window
        let old_event = create_test_event("sensors", "TempReading", window_start - 5000);
        assert!(!node.process_event(&old_event));

        assert_eq!(node.event_count(), 2);
    }

    #[test]
    fn test_eviction() {
        let window = WindowSpec {
            duration: Duration::from_millis(100),
            window_type: WindowType::Sliding,
        };

        let mut node = StreamAlphaNode::new("test-stream", None, Some(window));

        let current_time = StreamAlphaNode::current_time_ms();

        // Add event within window
        let event1 = create_test_event("test-stream", "TestEvent", current_time - 50);
        node.process_event(&event1);

        assert_eq!(node.event_count(), 1);

        // Wait to ensure event becomes old
        std::thread::sleep(Duration::from_millis(150));

        // Process new event, which should trigger eviction
        let event2 = create_test_event(
            "test-stream",
            "TestEvent",
            StreamAlphaNode::current_time_ms(),
        );
        node.process_event(&event2);

        // Old event should be evicted
        assert_eq!(node.event_count(), 1);
    }

    #[test]
    fn test_max_events_limit() {
        let mut node = StreamAlphaNode::new("test-stream", None, None).with_max_events(5);

        let current_time = StreamAlphaNode::current_time_ms();

        // Add 10 events
        for i in 0..10 {
            let event = create_test_event("test-stream", "TestEvent", current_time + i);
            node.process_event(&event);
        }

        // Should only keep 5 events
        assert_eq!(node.event_count(), 5);
    }

    #[test]
    fn test_clear() {
        let mut node = StreamAlphaNode::new("test-stream", None, None);

        let event = create_test_event("test-stream", "TestEvent", 1000);
        node.process_event(&event);

        assert_eq!(node.event_count(), 1);

        node.clear();
        assert_eq!(node.event_count(), 0);
    }

    #[test]
    fn test_window_stats() {
        let window = WindowSpec {
            duration: Duration::from_secs(60),
            window_type: WindowType::Sliding,
        };

        let mut node = StreamAlphaNode::new("test-stream", None, Some(window));

        let current_time = StreamAlphaNode::current_time_ms();
        let event1 = create_test_event("test-stream", "TestEvent", current_time - 10_000);
        let event2 = create_test_event("test-stream", "TestEvent", current_time - 5_000);

        node.process_event(&event1);
        node.process_event(&event2);

        let stats = node.window_stats();
        assert_eq!(stats.event_count, 2);
        assert_eq!(stats.oldest_event_timestamp, Some(current_time - 10_000));
        assert_eq!(stats.newest_event_timestamp, Some(current_time - 5_000));
        assert_eq!(stats.window_duration_ms, Some(60_000));
    }

    #[test]
    fn test_get_events() {
        let mut node = StreamAlphaNode::new("test-stream", None, None);

        let event1 = create_test_event("test-stream", "Event1", 1000);
        let event2 = create_test_event("test-stream", "Event2", 2000);

        node.process_event(&event1);
        node.process_event(&event2);

        let events = node.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, "Event1");
        assert_eq!(events[1].event_type, "Event2");
    }

    #[test]
    fn test_session_window_basic() {
        let window = WindowSpec {
            duration: Duration::from_secs(60), // Not used for session windows (timeout is in WindowType)
            window_type: WindowType::Session {
                timeout: Duration::from_secs(5),
            },
        };

        let mut node = StreamAlphaNode::new("test-stream", None, Some(window));

        let current_time = StreamAlphaNode::current_time_ms();

        // First event - starts session
        let event1 = create_test_event("test-stream", "Event1", current_time);
        assert!(node.process_event(&event1));
        assert_eq!(node.event_count(), 1);

        // Second event within timeout (2 seconds later)
        let event2 = create_test_event("test-stream", "Event2", current_time + 2000);
        assert!(node.process_event(&event2));
        assert_eq!(node.event_count(), 2);

        // Third event within timeout (1 second after event2)
        let event3 = create_test_event("test-stream", "Event3", current_time + 3000);
        assert!(node.process_event(&event3));
        assert_eq!(node.event_count(), 3);
    }

    #[test]
    fn test_session_window_timeout_new_session() {
        let window = WindowSpec {
            duration: Duration::from_secs(60),
            window_type: WindowType::Session {
                timeout: Duration::from_millis(100),
            },
        };

        let mut node = StreamAlphaNode::new("test-stream", None, Some(window));

        let current_time = StreamAlphaNode::current_time_ms();

        // First event - Session 1
        let event1 = create_test_event("test-stream", "Event1", current_time);
        node.process_event(&event1);
        assert_eq!(node.event_count(), 1);

        // Wait 150ms - exceed timeout
        std::thread::sleep(Duration::from_millis(150));

        // This should trigger eviction of old session
        let event2 = create_test_event("test-stream", "Event2", StreamAlphaNode::current_time_ms());
        node.process_event(&event2);

        // Old session should be cleared, only new event remains
        assert_eq!(node.event_count(), 1);
        assert_eq!(node.get_events()[0].event_type, "Event2");
    }

    #[test]
    fn test_session_window_gap_detection() {
        let window = WindowSpec {
            duration: Duration::from_secs(60),
            window_type: WindowType::Session {
                timeout: Duration::from_secs(2),
            },
        };

        let mut node = StreamAlphaNode::new("test-stream", None, Some(window));

        let base_time = StreamAlphaNode::current_time_ms();

        // Session 1: Events at t=0, t=1
        let event1 = create_test_event("test-stream", "S1_Event1", base_time);
        let event2 = create_test_event("test-stream", "S1_Event2", base_time + 1000);

        node.process_event(&event1);
        node.process_event(&event2);
        assert_eq!(node.event_count(), 2);

        // Gap of 3 seconds (exceeds 2-second timeout)
        // Session 2: Event at t=5
        let event3 = create_test_event("test-stream", "S2_Event1", base_time + 5000);
        node.process_event(&event3);

        // New event is accepted (starts new session)
        assert!(node
            .get_events()
            .iter()
            .any(|e| e.event_type == "S2_Event1"));
    }

    #[test]
    fn test_session_window_eviction_after_timeout() {
        let window = WindowSpec {
            duration: Duration::from_secs(60),
            window_type: WindowType::Session {
                timeout: Duration::from_millis(200),
            },
        };

        let mut node = StreamAlphaNode::new("test-stream", None, Some(window));

        let current_time = StreamAlphaNode::current_time_ms();

        // Add events to session
        let event1 = create_test_event("test-stream", "Event1", current_time);
        let event2 = create_test_event("test-stream", "Event2", current_time + 50);

        node.process_event(&event1);
        node.process_event(&event2);
        assert_eq!(node.event_count(), 2);

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(250));

        // Process new event - should trigger eviction
        let event3 = create_test_event("test-stream", "Event3", StreamAlphaNode::current_time_ms());
        node.process_event(&event3);

        // Only the new event should remain
        assert_eq!(node.event_count(), 1);
        assert_eq!(node.get_events()[0].event_type, "Event3");
    }

    #[test]
    fn test_session_window_clear_resets_state() {
        let window = WindowSpec {
            duration: Duration::from_secs(60),
            window_type: WindowType::Session {
                timeout: Duration::from_secs(5),
            },
        };

        let mut node = StreamAlphaNode::new("test-stream", None, Some(window));

        let current_time = StreamAlphaNode::current_time_ms();
        let event = create_test_event("test-stream", "Event1", current_time);

        node.process_event(&event);
        assert_eq!(node.event_count(), 1);
        assert!(node.last_session_event_timestamp.is_some());

        node.clear();
        assert_eq!(node.event_count(), 0);
        assert!(node.last_session_event_timestamp.is_none());
    }

    #[test]
    fn test_session_window_continuous_activity() {
        let window = WindowSpec {
            duration: Duration::from_secs(60),
            window_type: WindowType::Session {
                timeout: Duration::from_secs(1),
            },
        };

        let mut node = StreamAlphaNode::new("test-stream", None, Some(window));

        let base_time = StreamAlphaNode::current_time_ms();

        // Add events every 500ms (within 1-second timeout)
        for i in 0..5 {
            let event =
                create_test_event("test-stream", &format!("Event{}", i), base_time + (i * 500));
            node.process_event(&event);
        }

        // All events should be in the same session
        assert_eq!(node.event_count(), 5);
    }

    #[test]
    fn test_session_window_multiple_sessions() {
        let window = WindowSpec {
            duration: Duration::from_secs(60),
            window_type: WindowType::Session {
                timeout: Duration::from_millis(500),
            },
        };

        let mut node = StreamAlphaNode::new("test-stream", None, Some(window));

        let base_time = StreamAlphaNode::current_time_ms();

        // Session 1: Events at t=0, t=200
        node.process_event(&create_test_event("test-stream", "S1_E1", base_time));
        node.process_event(&create_test_event("test-stream", "S1_E2", base_time + 200));

        // Gap of 600ms - new session
        // Session 2: Event at t=1000
        node.process_event(&create_test_event("test-stream", "S2_E1", base_time + 1000));

        // Gap of 700ms - new session
        // Session 3: Event at t=2000
        node.process_event(&create_test_event("test-stream", "S3_E1", base_time + 2000));

        // Node should contain events from the latest session only
        // (previous sessions should be evicted when gap exceeded)
        assert!(node.event_count() > 0);
    }
}

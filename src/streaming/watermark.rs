//! Watermark and Late Data Handling
//!
//! This module provides watermark generation and late data handling for
//! stream processing with out-of-order events.

use super::event::StreamEvent;
use std::collections::VecDeque;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Watermark representing event-time progress
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Watermark {
    /// Timestamp in milliseconds since UNIX epoch
    pub timestamp: u64,
}

impl Watermark {
    /// Create a new watermark with the given timestamp
    pub fn new(timestamp: u64) -> Self {
        Self { timestamp }
    }

    /// Create a watermark from system time
    pub fn from_system_time(time: SystemTime) -> Self {
        let timestamp = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_millis() as u64;
        Self { timestamp }
    }

    /// Get current time watermark
    pub fn now() -> Self {
        Self::from_system_time(SystemTime::now())
    }

    /// Check if this watermark is before another
    pub fn is_before(&self, other: &Watermark) -> bool {
        self.timestamp < other.timestamp
    }

    /// Check if an event is late according to this watermark
    pub fn is_late(&self, event_time: u64) -> bool {
        event_time < self.timestamp
    }
}

/// Strategy for generating watermarks
#[derive(Debug, Clone)]
pub enum WatermarkStrategy {
    /// Periodic watermarks based on processing time
    Periodic {
        /// Interval between watermark generations
        interval: Duration,
    },

    /// Bounded out-of-orderness: watermark = max_timestamp - max_delay
    BoundedOutOfOrder {
        /// Maximum delay for out-of-order events
        max_delay: Duration,
    },

    /// Monotonic ascending watermarks (no out-of-order tolerance)
    MonotonicAscending,

    /// Custom watermark generation function
    Custom,
}

/// Watermark generator that tracks event-time progress
#[allow(dead_code)]
pub struct WatermarkGenerator {
    /// Current watermark
    current_watermark: Watermark,

    /// Strategy for generating watermarks
    strategy: WatermarkStrategy,

    /// Maximum observed event timestamp
    max_timestamp: u64,

    /// Last watermark emission time (processing time)
    last_emission: SystemTime,

    /// Pending events waiting for watermark advancement
    _pending_events: VecDeque<StreamEvent>,
}

impl WatermarkGenerator {
    /// Create a new watermark generator with the given strategy
    pub fn new(strategy: WatermarkStrategy) -> Self {
        Self {
            current_watermark: Watermark::new(0),
            strategy,
            max_timestamp: 0,
            last_emission: SystemTime::now(),
            _pending_events: VecDeque::new(),
        }
    }

    /// Process an event and update watermark if needed
    pub fn process_event(&mut self, event: &StreamEvent) -> Option<Watermark> {
        let event_time = event.metadata.timestamp;

        // Track maximum timestamp
        if event_time > self.max_timestamp {
            self.max_timestamp = event_time;
        }

        // Generate watermark based on strategy
        self.maybe_generate_watermark()
    }

    /// Generate watermark based on strategy
    fn maybe_generate_watermark(&mut self) -> Option<Watermark> {
        let new_watermark = match &self.strategy {
            WatermarkStrategy::Periodic { interval } => {
                let now = SystemTime::now();
                let elapsed = now.duration_since(self.last_emission).ok()?;

                if elapsed >= *interval {
                    self.last_emission = now;
                    Some(Watermark::new(self.max_timestamp))
                } else {
                    None
                }
            }

            WatermarkStrategy::BoundedOutOfOrder { max_delay } => {
                let delay_ms = max_delay.as_millis() as u64;
                let new_ts = self.max_timestamp.saturating_sub(delay_ms);

                if new_ts > self.current_watermark.timestamp {
                    Some(Watermark::new(new_ts))
                } else {
                    None
                }
            }

            WatermarkStrategy::MonotonicAscending => {
                if self.max_timestamp > self.current_watermark.timestamp {
                    Some(Watermark::new(self.max_timestamp))
                } else {
                    None
                }
            }

            WatermarkStrategy::Custom => {
                // Custom logic can be implemented by subclassing
                None
            }
        };

        if let Some(wm) = new_watermark {
            if wm > self.current_watermark {
                self.current_watermark = wm;
                return Some(wm);
            }
        }

        None
    }

    /// Get the current watermark
    pub fn current_watermark(&self) -> Watermark {
        self.current_watermark
    }

    /// Check if an event is late
    pub fn is_late(&self, event: &StreamEvent) -> bool {
        self.current_watermark.is_late(event.metadata.timestamp)
    }
}

/// Strategy for handling late events
#[derive(Debug, Clone)]
pub enum LateDataStrategy {
    /// Drop late events completely
    Drop,

    /// Allow late events up to a certain lateness threshold
    AllowedLateness {
        /// Maximum allowed lateness
        max_lateness: Duration,
    },

    /// Route late events to a side output for special processing
    SideOutput,

    /// Recompute affected windows when late data arrives
    RecomputeWindows,
}

/// Handler for late data events
pub struct LateDataHandler {
    /// Strategy for handling late data
    strategy: LateDataStrategy,

    /// Side output for late events
    side_output: Vec<StreamEvent>,

    /// Statistics about late events
    late_count: usize,
    dropped_count: usize,
    allowed_count: usize,
}

impl LateDataHandler {
    /// Create a new late data handler with the given strategy
    pub fn new(strategy: LateDataStrategy) -> Self {
        Self {
            strategy,
            side_output: Vec::new(),
            late_count: 0,
            dropped_count: 0,
            allowed_count: 0,
        }
    }

    /// Handle a late event according to the strategy
    pub fn handle_late_event(
        &mut self,
        event: StreamEvent,
        watermark: &Watermark,
    ) -> LateEventDecision {
        self.late_count += 1;

        let lateness = watermark.timestamp.saturating_sub(event.metadata.timestamp);

        match &self.strategy {
            LateDataStrategy::Drop => {
                self.dropped_count += 1;
                LateEventDecision::Drop
            }

            LateDataStrategy::AllowedLateness { max_lateness } => {
                let max_lateness_ms = max_lateness.as_millis() as u64;

                if lateness <= max_lateness_ms {
                    self.allowed_count += 1;
                    LateEventDecision::Process(event)
                } else {
                    self.dropped_count += 1;
                    LateEventDecision::Drop
                }
            }

            LateDataStrategy::SideOutput => {
                self.side_output.push(event.clone());
                LateEventDecision::SideOutput(event)
            }

            LateDataStrategy::RecomputeWindows => {
                self.allowed_count += 1;
                LateEventDecision::Recompute(event)
            }
        }
    }

    /// Get the side output events
    pub fn side_output(&self) -> &[StreamEvent] {
        &self.side_output
    }

    /// Clear the side output
    pub fn clear_side_output(&mut self) {
        self.side_output.clear();
    }

    /// Get statistics about late events
    pub fn stats(&self) -> LateDataStats {
        LateDataStats {
            total_late: self.late_count,
            dropped: self.dropped_count,
            allowed: self.allowed_count,
            side_output: self.side_output.len(),
        }
    }
}

/// Decision for how to handle a late event
#[derive(Debug, Clone)]
pub enum LateEventDecision {
    /// Drop the event
    Drop,

    /// Process the event normally
    Process(StreamEvent),

    /// Route to side output
    SideOutput(StreamEvent),

    /// Recompute affected windows
    Recompute(StreamEvent),
}

/// Statistics about late data handling
#[derive(Debug, Clone, Copy)]
pub struct LateDataStats {
    /// Total number of late events
    pub total_late: usize,

    /// Number of dropped late events
    pub dropped: usize,

    /// Number of allowed late events
    pub allowed: usize,

    /// Number of events in side output
    pub side_output: usize,
}

/// Watermark-aware stream that tracks event-time progress
pub struct WatermarkedStream {
    /// Events in the stream
    events: Vec<StreamEvent>,

    /// Watermark generator
    watermark_gen: WatermarkGenerator,

    /// Late data handler
    late_handler: LateDataHandler,

    /// Watermark history for debugging
    watermark_history: Vec<Watermark>,
}

impl WatermarkedStream {
    /// Create a new watermarked stream
    pub fn new(watermark_strategy: WatermarkStrategy, late_strategy: LateDataStrategy) -> Self {
        Self {
            events: Vec::new(),
            watermark_gen: WatermarkGenerator::new(watermark_strategy),
            late_handler: LateDataHandler::new(late_strategy),
            watermark_history: Vec::new(),
        }
    }

    /// Add an event to the stream, checking for lateness
    pub fn add_event(&mut self, event: StreamEvent) -> Result<(), String> {
        // Check if event is late
        if self.watermark_gen.is_late(&event) {
            // Handle late event
            match self
                .late_handler
                .handle_late_event(event, &self.watermark_gen.current_watermark())
            {
                LateEventDecision::Drop => {
                    // Event dropped, do nothing
                }
                LateEventDecision::Process(e) => {
                    self.events.push(e);
                }
                LateEventDecision::SideOutput(_) => {
                    // Event stored in side output
                }
                LateEventDecision::Recompute(e) => {
                    self.events.push(e);
                }
            }
        } else {
            // Event is on-time
            self.events.push(event.clone());

            // Update watermark
            if let Some(new_watermark) = self.watermark_gen.process_event(&event) {
                self.watermark_history.push(new_watermark);
            }
        }

        Ok(())
    }

    /// Get all events
    pub fn events(&self) -> &[StreamEvent] {
        &self.events
    }

    /// Get current watermark
    pub fn current_watermark(&self) -> Watermark {
        self.watermark_gen.current_watermark()
    }

    /// Get late data statistics
    pub fn late_stats(&self) -> LateDataStats {
        self.late_handler.stats()
    }

    /// Get side output events
    pub fn side_output(&self) -> &[StreamEvent] {
        self.late_handler.side_output()
    }

    /// Get watermark history
    pub fn watermark_history(&self) -> &[Watermark] {
        &self.watermark_history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Value;
    use std::collections::HashMap;

    fn create_event(timestamp: u64, value: i64) -> StreamEvent {
        let mut data = HashMap::new();
        data.insert("value".to_string(), Value::Integer(value));
        let event = StreamEvent::new("TestEvent", data, "test");

        // Manually set timestamp
        StreamEvent {
            metadata: super::super::event::EventMetadata {
                timestamp,
                ..event.metadata
            },
            ..event
        }
    }

    #[test]
    fn test_watermark_ordering() {
        let wm1 = Watermark::new(1000);
        let wm2 = Watermark::new(2000);

        assert!(wm1.is_before(&wm2));
        assert!(!wm2.is_before(&wm1));
        assert!(wm1 < wm2);
    }

    #[test]
    fn test_monotonic_watermark() {
        let mut gen = WatermarkGenerator::new(WatermarkStrategy::MonotonicAscending);

        let e1 = create_event(1000, 1);
        let e2 = create_event(2000, 2);
        let e3 = create_event(1500, 3); // Out of order

        gen.process_event(&e1);
        assert_eq!(gen.current_watermark().timestamp, 1000);

        gen.process_event(&e2);
        assert_eq!(gen.current_watermark().timestamp, 2000);

        gen.process_event(&e3);
        // Watermark stays at 2000 (monotonic)
        assert_eq!(gen.current_watermark().timestamp, 2000);
    }

    #[test]
    fn test_bounded_out_of_order() {
        let strategy = WatermarkStrategy::BoundedOutOfOrder {
            max_delay: Duration::from_millis(500),
        };
        let mut gen = WatermarkGenerator::new(strategy);

        let e1 = create_event(2000, 1);
        gen.process_event(&e1);

        // Watermark should be max_timestamp - max_delay = 2000 - 500 = 1500
        assert_eq!(gen.current_watermark().timestamp, 1500);
    }

    #[test]
    fn test_late_data_drop() {
        let mut handler = LateDataHandler::new(LateDataStrategy::Drop);
        let watermark = Watermark::new(2000);

        let late_event = create_event(1000, 1); // 1000ms late

        match handler.handle_late_event(late_event, &watermark) {
            LateEventDecision::Drop => {
                let stats = handler.stats();
                assert_eq!(stats.total_late, 1);
                assert_eq!(stats.dropped, 1);
            }
            _ => panic!("Expected Drop decision"),
        }
    }

    #[test]
    fn test_late_data_allowed_lateness() {
        let strategy = LateDataStrategy::AllowedLateness {
            max_lateness: Duration::from_millis(500),
        };
        let mut handler = LateDataHandler::new(strategy);
        let watermark = Watermark::new(2000);

        // Event within allowed lateness
        let late_event1 = create_event(1600, 1); // 400ms late
        match handler.handle_late_event(late_event1, &watermark) {
            LateEventDecision::Process(_) => {
                assert_eq!(handler.stats().allowed, 1);
            }
            _ => panic!("Expected Process decision"),
        }

        // Event beyond allowed lateness
        let late_event2 = create_event(1400, 2); // 600ms late
        match handler.handle_late_event(late_event2, &watermark) {
            LateEventDecision::Drop => {
                assert_eq!(handler.stats().dropped, 1);
            }
            _ => panic!("Expected Drop decision"),
        }
    }

    #[test]
    fn test_watermarked_stream() {
        let strategy = WatermarkStrategy::BoundedOutOfOrder {
            max_delay: Duration::from_millis(500),
        };
        let late_strategy = LateDataStrategy::Drop;

        let mut stream = WatermarkedStream::new(strategy, late_strategy);

        // Add events in order
        stream.add_event(create_event(1000, 1)).unwrap();
        stream.add_event(create_event(2000, 2)).unwrap();

        // Watermark should be 2000 - 500 = 1500
        assert_eq!(stream.current_watermark().timestamp, 1500);

        // Add late event (before watermark)
        stream.add_event(create_event(1200, 3)).unwrap();

        // Late event should be dropped
        let stats = stream.late_stats();
        assert_eq!(stats.total_late, 1);
        assert_eq!(stats.dropped, 1);

        // Should have 2 on-time events
        assert_eq!(stream.events().len(), 2);
    }
}

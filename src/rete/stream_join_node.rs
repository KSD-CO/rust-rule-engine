use crate::streaming::event::StreamEvent;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

/// Join types supported by the stream join node
#[derive(Debug, Clone, PartialEq)]
pub enum JoinType {
    /// Inner join - only emit when events match in both streams
    Inner,
    /// Left outer join - emit left events even if no right match
    LeftOuter,
    /// Right outer join - emit right events even if no left match
    RightOuter,
    /// Full outer join - emit all events from both streams
    FullOuter,
}

/// Strategy for buffering and matching stream events
#[derive(Debug, Clone, PartialEq)]
pub enum JoinStrategy {
    /// Time-based window join (most common for streaming)
    TimeWindow { duration: Duration },
    /// Count-based window join
    CountWindow { count: usize },
    /// Session-based join with gap timeout
    SessionWindow { gap: Duration },
}

/// Represents a matched pair of events from two streams
#[derive(Debug, Clone)]
pub struct JoinedEvent {
    pub left: Option<StreamEvent>,
    pub right: Option<StreamEvent>,
    pub join_timestamp: i64,
}

/// Stream join node for RETE network
/// Performs windowed joins between two streams based on join conditions
pub struct StreamJoinNode {
    /// Name of the left input stream
    pub left_stream: String,
    /// Name of the right input stream
    pub right_stream: String,
    /// Join type (inner, left outer, right outer, full outer)
    pub join_type: JoinType,
    /// Join strategy (time window, count window, session window)
    pub join_strategy: JoinStrategy,
    /// Join key extractor for left stream
    pub left_key_extractor: Box<dyn Fn(&StreamEvent) -> Option<String> + Send + Sync>,
    /// Join key extractor for right stream
    pub right_key_extractor: Box<dyn Fn(&StreamEvent) -> Option<String> + Send + Sync>,
    /// Additional join condition predicate
    pub join_condition: Box<dyn Fn(&StreamEvent, &StreamEvent) -> bool + Send + Sync>,
    /// Buffer for left stream events, partitioned by join key
    left_buffer: HashMap<String, VecDeque<StreamEvent>>,
    /// Buffer for right stream events, partitioned by join key
    right_buffer: HashMap<String, VecDeque<StreamEvent>>,
    /// Tracking for which left events have been matched (for outer joins)
    left_matched: HashMap<String, bool>,
    /// Tracking for which right events have been matched (for outer joins)
    right_matched: HashMap<String, bool>,
    /// Current watermark timestamp
    watermark: i64,
}

impl StreamJoinNode {
    /// Create a new stream join node
    pub fn new(
        left_stream: String,
        right_stream: String,
        join_type: JoinType,
        join_strategy: JoinStrategy,
        left_key_extractor: Box<dyn Fn(&StreamEvent) -> Option<String> + Send + Sync>,
        right_key_extractor: Box<dyn Fn(&StreamEvent) -> Option<String> + Send + Sync>,
        join_condition: Box<dyn Fn(&StreamEvent, &StreamEvent) -> bool + Send + Sync>,
    ) -> Self {
        Self {
            left_stream,
            right_stream,
            join_type,
            join_strategy,
            left_key_extractor,
            right_key_extractor,
            join_condition,
            left_buffer: HashMap::new(),
            right_buffer: HashMap::new(),
            left_matched: HashMap::new(),
            right_matched: HashMap::new(),
            watermark: 0,
        }
    }

    /// Process a left stream event and produce joined events
    pub fn process_left(&mut self, event: StreamEvent) -> Vec<JoinedEvent> {
        let mut results = Vec::new();

        // Extract join key
        let key = match (self.left_key_extractor)(&event) {
            Some(k) => k,
            None => return results, // No key, skip
        };

        let event_id = Self::generate_event_id(&event);

        // Add to buffer
        self.left_buffer
            .entry(key.clone())
            .or_insert_with(VecDeque::new)
            .push_back(event.clone());

        // Try to join with right stream events
        if let Some(right_events) = self.right_buffer.get(&key) {
            for right_event in right_events {
                if self.is_within_window(&event, right_event)
                    && (self.join_condition)(&event, right_event)
                {
                    results.push(JoinedEvent {
                        left: Some(event.clone()),
                        right: Some(right_event.clone()),
                        join_timestamp: (event.metadata.timestamp as i64)
                            .max(right_event.metadata.timestamp as i64),
                    });

                    // Mark as matched for outer join tracking
                    self.left_matched.insert(event_id.clone(), true);
                    self.right_matched
                        .insert(Self::generate_event_id(right_event), true);
                }
            }
        }

        // For outer joins, emit unmatched left events
        if (self.join_type == JoinType::LeftOuter || self.join_type == JoinType::FullOuter)
            && !self.left_matched.contains_key(&event_id)
        {
            results.push(JoinedEvent {
                left: Some(event.clone()),
                right: None,
                join_timestamp: event.metadata.timestamp as i64,
            });
        }

        results
    }

    /// Process a right stream event and produce joined events
    pub fn process_right(&mut self, event: StreamEvent) -> Vec<JoinedEvent> {
        let mut results = Vec::new();

        // Extract join key
        let key = match (self.right_key_extractor)(&event) {
            Some(k) => k,
            None => return results, // No key, skip
        };

        let event_id = Self::generate_event_id(&event);

        // Add to buffer
        self.right_buffer
            .entry(key.clone())
            .or_insert_with(VecDeque::new)
            .push_back(event.clone());

        // Try to join with left stream events
        if let Some(left_events) = self.left_buffer.get(&key) {
            for left_event in left_events {
                if self.is_within_window(left_event, &event)
                    && (self.join_condition)(left_event, &event)
                {
                    results.push(JoinedEvent {
                        left: Some(left_event.clone()),
                        right: Some(event.clone()),
                        join_timestamp: (left_event.metadata.timestamp as i64)
                            .max(event.metadata.timestamp as i64),
                    });

                    // Mark as matched for outer join tracking
                    self.left_matched
                        .insert(Self::generate_event_id(left_event), true);
                    self.right_matched.insert(event_id.clone(), true);
                }
            }
        }

        // For outer joins, emit unmatched right events
        if (self.join_type == JoinType::RightOuter || self.join_type == JoinType::FullOuter)
            && !self.right_matched.contains_key(&event_id)
        {
            results.push(JoinedEvent {
                left: None,
                right: Some(event.clone()),
                join_timestamp: event.metadata.timestamp as i64,
            });
        }

        results
    }

    /// Update watermark and evict old events
    pub fn update_watermark(&mut self, new_watermark: i64) -> Vec<JoinedEvent> {
        let mut results = Vec::new();
        self.watermark = new_watermark;

        // Evict expired events from buffers
        self.evict_expired_events();

        // For outer joins, emit any remaining unmatched events that are now beyond the window
        if self.join_type == JoinType::LeftOuter || self.join_type == JoinType::FullOuter {
            results.extend(self.emit_unmatched_left());
        }
        if self.join_type == JoinType::RightOuter || self.join_type == JoinType::FullOuter {
            results.extend(self.emit_unmatched_right());
        }

        results
    }

    /// Check if two events are within the join window
    fn is_within_window(&self, left: &StreamEvent, right: &StreamEvent) -> bool {
        match &self.join_strategy {
            JoinStrategy::TimeWindow { duration } => {
                let time_diff =
                    ((left.metadata.timestamp as i64) - (right.metadata.timestamp as i64)).abs();
                time_diff <= duration.as_millis() as i64
            }
            JoinStrategy::CountWindow { .. } => {
                // For count windows, we handle this differently in buffer management
                true
            }
            JoinStrategy::SessionWindow { gap } => {
                let time_diff =
                    ((left.metadata.timestamp as i64) - (right.metadata.timestamp as i64)).abs();
                time_diff <= gap.as_millis() as i64
            }
        }
    }

    /// Evict events that are outside the join window
    fn evict_expired_events(&mut self) {
        let watermark = self.watermark;
        let window_size = self.get_window_duration();

        // Evict from left buffer
        for queue in self.left_buffer.values_mut() {
            while let Some(event) = queue.front() {
                if watermark - event.metadata.timestamp as i64 > window_size {
                    if let Some(evicted) = queue.pop_front() {
                        let id = Self::generate_event_id(&evicted);
                        self.left_matched.remove(&id);
                    }
                } else {
                    break;
                }
            }
        }

        // Evict from right buffer
        for queue in self.right_buffer.values_mut() {
            while let Some(event) = queue.front() {
                if watermark - event.metadata.timestamp as i64 > window_size {
                    if let Some(evicted) = queue.pop_front() {
                        let id = Self::generate_event_id(&evicted);
                        self.right_matched.remove(&id);
                    }
                } else {
                    break;
                }
            }
        }

        // Clean up empty queues
        self.left_buffer.retain(|_, queue| !queue.is_empty());
        self.right_buffer.retain(|_, queue| !queue.is_empty());
    }

    /// Emit unmatched left events (for left/full outer joins)
    fn emit_unmatched_left(&mut self) -> Vec<JoinedEvent> {
        let mut results = Vec::new();
        let watermark = self.watermark;
        let window_size = self.get_window_duration();

        for queue in self.left_buffer.values() {
            for event in queue {
                let id = Self::generate_event_id(event);
                if !self.left_matched.contains_key(&id)
                    && watermark - event.metadata.timestamp as i64 > window_size
                {
                    results.push(JoinedEvent {
                        left: Some(event.clone()),
                        right: None,
                        join_timestamp: event.metadata.timestamp as i64,
                    });
                }
            }
        }

        results
    }

    /// Emit unmatched right events (for right/full outer joins)
    fn emit_unmatched_right(&mut self) -> Vec<JoinedEvent> {
        let mut results = Vec::new();
        let watermark = self.watermark;
        let window_size = self.get_window_duration();

        for queue in self.right_buffer.values() {
            for event in queue {
                let id = Self::generate_event_id(event);
                if !self.right_matched.contains_key(&id)
                    && watermark - event.metadata.timestamp as i64 > window_size
                {
                    results.push(JoinedEvent {
                        left: None,
                        right: Some(event.clone()),
                        join_timestamp: event.metadata.timestamp as i64,
                    });
                }
            }
        }

        results
    }

    /// Get window duration in milliseconds
    fn get_window_duration(&self) -> i64 {
        match &self.join_strategy {
            JoinStrategy::TimeWindow { duration } => duration.as_millis() as i64,
            JoinStrategy::SessionWindow { gap } => gap.as_millis() as i64,
            JoinStrategy::CountWindow { .. } => i64::MAX, // Count windows don't time out
        }
    }

    /// Generate a unique ID for an event
    fn generate_event_id(event: &StreamEvent) -> String {
        format!("{}_{}", event.id, event.metadata.timestamp as i64)
    }

    /// Get buffer statistics (for monitoring and debugging)
    pub fn get_stats(&self) -> JoinNodeStats {
        let left_count: usize = self.left_buffer.values().map(|q| q.len()).sum();
        let right_count: usize = self.right_buffer.values().map(|q| q.len()).sum();

        JoinNodeStats {
            left_buffer_size: left_count,
            right_buffer_size: right_count,
            left_partitions: self.left_buffer.len(),
            right_partitions: self.right_buffer.len(),
            watermark: self.watermark,
        }
    }
}

/// Statistics for join node monitoring
#[derive(Debug, Clone)]
pub struct JoinNodeStats {
    pub left_buffer_size: usize,
    pub right_buffer_size: usize,
    pub left_partitions: usize,
    pub right_partitions: usize,
    pub watermark: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(stream_id: &str, timestamp: i64, key: &str) -> StreamEvent {
        use crate::streaming::event::EventMetadata;
        use crate::types::Value;

        StreamEvent {
            id: format!("test_{}", timestamp),
            event_type: "test".to_string(),
            data: vec![(key.to_string(), Value::String(key.to_string()))]
                .into_iter()
                .collect(),
            metadata: EventMetadata {
                timestamp: timestamp as u64,
                source: stream_id.to_string(),
                sequence: 0,
                tags: std::collections::HashMap::new(),
            },
        }
    }

    #[test]
    fn test_inner_join_basic() {
        let mut join_node = StreamJoinNode::new(
            "left".to_string(),
            "right".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(10),
            },
            Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        let left_event = create_test_event("left", 1000, "user1");
        let right_event = create_test_event("right", 1005, "user1");

        let results1 = join_node.process_left(left_event);
        assert_eq!(results1.len(), 0); // No right events yet

        let results2 = join_node.process_right(right_event);
        assert_eq!(results2.len(), 1); // Should join
        assert!(results2[0].left.is_some());
        assert!(results2[0].right.is_some());
    }

    #[test]
    fn test_time_window_filtering() {
        let mut join_node = StreamJoinNode::new(
            "left".to_string(),
            "right".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(5),
            },
            Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        let left_event = create_test_event("left", 1000, "user1");
        let right_event_close = create_test_event("right", 1003, "user1");
        let right_event_far = create_test_event("right", 8000, "user1");

        join_node.process_left(left_event);

        let results1 = join_node.process_right(right_event_close);
        assert_eq!(results1.len(), 1); // Within window

        let results2 = join_node.process_right(right_event_far);
        assert_eq!(results2.len(), 0); // Outside window
    }

    #[test]
    fn test_left_outer_join() {
        let mut join_node = StreamJoinNode::new(
            "left".to_string(),
            "right".to_string(),
            JoinType::LeftOuter,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(10),
            },
            Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        let left_event = create_test_event("left", 1000, "user1");

        let results = join_node.process_left(left_event);
        assert_eq!(results.len(), 1); // Emits unmatched left
        assert!(results[0].left.is_some());
        assert!(results[0].right.is_none());
    }

    #[test]
    fn test_partition_by_key() {
        let mut join_node = StreamJoinNode::new(
            "left".to_string(),
            "right".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(10),
            },
            Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        let left1 = create_test_event("left", 1000, "user1");
        let left2 = create_test_event("left", 1000, "user2");
        let right1 = create_test_event("right", 1005, "user1");

        join_node.process_left(left1);
        join_node.process_left(left2);

        let results = join_node.process_right(right1);
        assert_eq!(results.len(), 1); // Only joins with user1
        assert_eq!(
            results[0]
                .left
                .as_ref()
                .unwrap()
                .data
                .get("key")
                .unwrap()
                .as_string()
                .unwrap(),
            "user1"
        );
    }
}

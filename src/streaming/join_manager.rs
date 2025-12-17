use crate::rete::stream_join_node::{JoinedEvent, StreamJoinNode};
use crate::streaming::event::StreamEvent;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Manages multiple stream joins and coordinates event routing
pub struct StreamJoinManager {
    /// All registered join nodes, indexed by join ID
    joins: HashMap<String, Arc<Mutex<StreamJoinNode>>>,
    /// Maps stream names to the join nodes that consume them
    stream_to_joins: HashMap<String, Vec<String>>,
    /// Result handlers for each join
    result_handlers: HashMap<String, Box<dyn Fn(JoinedEvent) + Send + Sync>>,
}

impl StreamJoinManager {
    /// Create a new stream join manager
    pub fn new() -> Self {
        Self {
            joins: HashMap::new(),
            stream_to_joins: HashMap::new(),
            result_handlers: HashMap::new(),
        }
    }

    /// Register a new stream join
    pub fn register_join(
        &mut self,
        join_id: String,
        join_node: StreamJoinNode,
        result_handler: Box<dyn Fn(JoinedEvent) + Send + Sync>,
    ) {
        let left_stream = join_node.left_stream.clone();
        let right_stream = join_node.right_stream.clone();

        // Index the join by streams
        self.stream_to_joins
            .entry(left_stream)
            .or_insert_with(Vec::new)
            .push(join_id.clone());

        self.stream_to_joins
            .entry(right_stream)
            .or_insert_with(Vec::new)
            .push(join_id.clone());

        // Store join node and handler
        self.joins
            .insert(join_id.clone(), Arc::new(Mutex::new(join_node)));
        self.result_handlers.insert(join_id, result_handler);
    }

    /// Remove a stream join
    pub fn unregister_join(&mut self, join_id: &str) {
        if let Some(join) = self.joins.get(join_id) {
            let join_lock = join.lock().unwrap();
            let left_stream = join_lock.left_stream.clone();
            let right_stream = join_lock.right_stream.clone();

            // Remove from stream indices
            if let Some(joins) = self.stream_to_joins.get_mut(&left_stream) {
                joins.retain(|id| id != join_id);
            }
            if let Some(joins) = self.stream_to_joins.get_mut(&right_stream) {
                joins.retain(|id| id != join_id);
            }
        }

        self.joins.remove(join_id);
        self.result_handlers.remove(join_id);
    }

    /// Process an incoming stream event
    /// Routes the event to all relevant join nodes
    pub fn process_event(&self, event: StreamEvent) {
        let stream_id = event.metadata.source.clone();

        // Find all joins that consume this stream
        if let Some(join_ids) = self.stream_to_joins.get(&stream_id) {
            for join_id in join_ids {
                if let Some(join) = self.joins.get(join_id) {
                    let mut join_lock = join.lock().unwrap();

                    // Determine if this is a left or right stream for this join
                    let results = if join_lock.left_stream == stream_id {
                        join_lock.process_left(event.clone())
                    } else {
                        join_lock.process_right(event.clone())
                    };

                    // Process results
                    if let Some(handler) = self.result_handlers.get(join_id) {
                        for joined in results {
                            handler(joined);
                        }
                    }
                }
            }
        }
    }

    /// Update watermark for a specific stream
    /// This triggers eviction of old events and emission of outer join results
    pub fn update_watermark(&self, stream_id: &str, watermark: i64) {
        if let Some(join_ids) = self.stream_to_joins.get(stream_id) {
            for join_id in join_ids {
                if let Some(join) = self.joins.get(join_id) {
                    let mut join_lock = join.lock().unwrap();
                    let results = join_lock.update_watermark(watermark);

                    // Process results from watermark update (outer join emissions)
                    if let Some(handler) = self.result_handlers.get(join_id) {
                        for joined in results {
                            handler(joined);
                        }
                    }
                }
            }
        }
    }

    /// Get statistics for all joins
    pub fn get_all_stats(&self) -> HashMap<String, crate::rete::stream_join_node::JoinNodeStats> {
        let mut stats = HashMap::new();
        for (join_id, join) in &self.joins {
            let join_lock = join.lock().unwrap();
            stats.insert(join_id.clone(), join_lock.get_stats());
        }
        stats
    }

    /// Get statistics for a specific join
    pub fn get_join_stats(
        &self,
        join_id: &str,
    ) -> Option<crate::rete::stream_join_node::JoinNodeStats> {
        self.joins.get(join_id).map(|join| {
            let join_lock = join.lock().unwrap();
            join_lock.get_stats()
        })
    }

    /// Clear all joins (for testing or reset)
    pub fn clear(&mut self) {
        self.joins.clear();
        self.stream_to_joins.clear();
        self.result_handlers.clear();
    }
}

impl Default for StreamJoinManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rete::stream_join_node::{JoinStrategy, JoinType};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    fn create_test_event(stream_id: &str, timestamp: i64, user_id: &str) -> StreamEvent {
        use crate::streaming::event::EventMetadata;
        use crate::types::Value;

        StreamEvent {
            id: format!("test_{}_{}", stream_id, timestamp),
            event_type: "test".to_string(),
            data: vec![("user_id".to_string(), Value::String(user_id.to_string()))]
                .into_iter()
                .collect(),
            metadata: EventMetadata {
                timestamp: timestamp as u64,
                source: stream_id.to_string(),
                sequence: 0,
                tags: HashMap::new(),
            },
        }
    }

    #[test]
    fn test_register_and_route_events() {
        let mut manager = StreamJoinManager::new();
        let result_count = Arc::new(AtomicUsize::new(0));
        let result_count_clone = result_count.clone();

        let join_node = StreamJoinNode::new(
            "left".to_string(),
            "right".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(10),
            },
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        manager.register_join(
            "join1".to_string(),
            join_node,
            Box::new(move |_| {
                result_count_clone.fetch_add(1, Ordering::SeqCst);
            }),
        );

        // Send events
        let left_event = create_test_event("left", 1000, "user1");
        let right_event = create_test_event("right", 1005, "user1");

        manager.process_event(left_event);
        manager.process_event(right_event);

        // Should have one join result
        assert_eq!(result_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_multiple_joins_same_stream() {
        let mut manager = StreamJoinManager::new();
        let result_count1 = Arc::new(AtomicUsize::new(0));
        let result_count2 = Arc::new(AtomicUsize::new(0));
        let rc1 = result_count1.clone();
        let rc2 = result_count2.clone();

        // Join 1: left + right
        let join1 = StreamJoinNode::new(
            "left".to_string(),
            "right".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(10),
            },
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        // Join 2: left + other
        let join2 = StreamJoinNode::new(
            "left".to_string(),
            "other".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(10),
            },
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        manager.register_join(
            "join1".to_string(),
            join1,
            Box::new(move |_| {
                rc1.fetch_add(1, Ordering::SeqCst);
            }),
        );

        manager.register_join(
            "join2".to_string(),
            join2,
            Box::new(move |_| {
                rc2.fetch_add(1, Ordering::SeqCst);
            }),
        );

        // Send left event (should be routed to both joins)
        let left_event = create_test_event("left", 1000, "user1");
        manager.process_event(left_event);

        // Send right event (should only go to join1)
        let right_event = create_test_event("right", 1005, "user1");
        manager.process_event(right_event);

        // Send other event (should only go to join2)
        let other_event = create_test_event("other", 1005, "user1");
        manager.process_event(other_event);

        // Each join should have one result
        assert_eq!(result_count1.load(Ordering::SeqCst), 1);
        assert_eq!(result_count2.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_unregister_join() {
        let mut manager = StreamJoinManager::new();
        let result_count = Arc::new(AtomicUsize::new(0));
        let rc = result_count.clone();

        let join_node = StreamJoinNode::new(
            "left".to_string(),
            "right".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(10),
            },
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        manager.register_join(
            "join1".to_string(),
            join_node,
            Box::new(move |_| {
                rc.fetch_add(1, Ordering::SeqCst);
            }),
        );

        // Unregister the join
        manager.unregister_join("join1");

        // Send events - should not produce results
        let left_event = create_test_event("left", 1000, "user1");
        let right_event = create_test_event("right", 1005, "user1");

        manager.process_event(left_event);
        manager.process_event(right_event);

        assert_eq!(result_count.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_watermark_update() {
        let mut manager = StreamJoinManager::new();
        let result_count = Arc::new(AtomicUsize::new(0));
        let rc = result_count.clone();

        // Use left outer join to test watermark emissions
        let join_node = StreamJoinNode::new(
            "left".to_string(),
            "right".to_string(),
            JoinType::LeftOuter,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(5),
            },
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        manager.register_join(
            "join1".to_string(),
            join_node,
            Box::new(move |_| {
                rc.fetch_add(1, Ordering::SeqCst);
            }),
        );

        // Send left event
        let left_event = create_test_event("left", 1000, "user1");
        manager.process_event(left_event);

        // At this point, left outer join should have emitted unmatched left event
        assert_eq!(result_count.load(Ordering::SeqCst), 1);

        // Update watermark - might emit more for outer joins
        manager.update_watermark("left", 10000);

        // Should still be 1 (event already emitted)
        assert_eq!(result_count.load(Ordering::SeqCst), 1);
    }
}

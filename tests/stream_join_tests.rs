#![cfg(feature = "streaming")]

use rust_rule_engine::rete::stream_join_node::{JoinStrategy, JoinType, StreamJoinNode};
use rust_rule_engine::streaming::event::StreamEvent;
use rust_rule_engine::streaming::join_manager::StreamJoinManager;
use rust_rule_engine::streaming::join_optimizer::{JoinOptimizer, StreamStats};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn create_event(stream_id: &str, timestamp: i64, data: Vec<(&str, &str)>) -> StreamEvent {
    use rust_rule_engine::streaming::event::EventMetadata;
    use rust_rule_engine::types::Value;

    StreamEvent {
        id: format!("test_{}_{}", stream_id, timestamp),
        event_type: "test".to_string(),
        data: data
            .into_iter()
            .map(|(k, v)| (k.to_string(), Value::String(v.to_string())))
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
fn test_basic_inner_join() {
    let mut join = StreamJoinNode::new(
        "clicks".to_string(),
        "purchases".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(10),
        },
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|_, _| true),
    );

    let click = create_event("clicks", 1000, vec![("user_id", "user1")]);
    let purchase = create_event("purchases", 1005, vec![("user_id", "user1")]);

    let results1 = join.process_left(click);
    assert_eq!(results1.len(), 0, "No right events yet");

    let results2 = join.process_right(purchase);
    assert_eq!(results2.len(), 1, "Should have one join result");
    assert!(results2[0].left.is_some());
    assert!(results2[0].right.is_some());
}

#[test]
fn test_join_with_time_window_constraint() {
    let mut join = StreamJoinNode::new(
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

    let left = create_event("left", 1000, vec![("key", "k1")]);
    let right_close = create_event("right", 1003, vec![("key", "k1")]);
    let right_far = create_event("right", 8000, vec![("key", "k1")]);

    join.process_left(left);

    let results1 = join.process_right(right_close);
    assert_eq!(results1.len(), 1, "Within window");

    let results2 = join.process_right(right_far);
    assert_eq!(results2.len(), 0, "Outside window");
}

#[test]
fn test_left_outer_join() {
    let mut join = StreamJoinNode::new(
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

    let left = create_event("left", 1000, vec![("key", "k1")]);

    let results = join.process_left(left);
    assert_eq!(results.len(), 1, "Left outer should emit unmatched");
    assert!(results[0].left.is_some());
    assert!(results[0].right.is_none());
}

#[test]
fn test_right_outer_join() {
    let mut join = StreamJoinNode::new(
        "left".to_string(),
        "right".to_string(),
        JoinType::RightOuter,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(10),
        },
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|_, _| true),
    );

    let right = create_event("right", 1000, vec![("key", "k1")]);

    let results = join.process_right(right);
    assert_eq!(results.len(), 1, "Right outer should emit unmatched");
    assert!(results[0].left.is_none());
    assert!(results[0].right.is_some());
}

#[test]
fn test_full_outer_join() {
    let mut join = StreamJoinNode::new(
        "left".to_string(),
        "right".to_string(),
        JoinType::FullOuter,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(10),
        },
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|_, _| true),
    );

    let left = create_event("left", 1000, vec![("key", "k1")]);
    let right = create_event("right", 2000, vec![("key", "k2")]);

    let results1 = join.process_left(left);
    assert_eq!(results1.len(), 1);
    assert!(results1[0].left.is_some());
    assert!(results1[0].right.is_none());

    let results2 = join.process_right(right);
    assert_eq!(results2.len(), 1);
    assert!(results2[0].left.is_none());
    assert!(results2[0].right.is_some());
}

#[test]
fn test_join_with_custom_condition() {
    let mut join = StreamJoinNode::new(
        "left".to_string(),
        "right".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(10),
        },
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|left, right| {
            // Custom condition: right amount must be > left amount
            let left_amount = left
                .data
                .get("amount")
                .and_then(|v| v.as_string())
                .and_then(|s| s.parse::<i32>().ok());
            let right_amount = right
                .data
                .get("amount")
                .and_then(|v| v.as_string())
                .and_then(|s| s.parse::<i32>().ok());
            match (left_amount, right_amount) {
                (Some(l), Some(r)) => r > l,
                _ => false,
            }
        }),
    );

    let left = create_event("left", 1000, vec![("key", "k1"), ("amount", "100")]);
    let right_high = create_event("right", 1005, vec![("key", "k1"), ("amount", "200")]);
    let right_low = create_event("right", 1010, vec![("key", "k1"), ("amount", "50")]);

    join.process_left(left);

    let results1 = join.process_right(right_high);
    assert_eq!(results1.len(), 1, "Should match: 200 > 100");

    let results2 = join.process_right(right_low);
    assert_eq!(results2.len(), 0, "Should not match: 50 < 100");
}

#[test]
fn test_partition_by_key() {
    let mut join = StreamJoinNode::new(
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

    let left1 = create_event("left", 1000, vec![("user_id", "user1")]);
    let left2 = create_event("left", 1000, vec![("user_id", "user2")]);
    let right1 = create_event("right", 1005, vec![("user_id", "user1")]);
    let right2 = create_event("right", 1005, vec![("user_id", "user2")]);

    join.process_left(left1);
    join.process_left(left2);

    let results1 = join.process_right(right1);
    assert_eq!(results1.len(), 1);
    assert_eq!(
        results1[0]
            .left
            .as_ref()
            .unwrap()
            .data
            .get("user_id")
            .unwrap()
            .as_string()
            .unwrap(),
        "user1"
    );

    let results2 = join.process_right(right2);
    assert_eq!(results2.len(), 1);
    assert_eq!(
        results2[0]
            .left
            .as_ref()
            .unwrap()
            .data
            .get("user_id")
            .unwrap()
            .as_string()
            .unwrap(),
        "user2"
    );
}

#[test]
fn test_watermark_eviction() {
    let mut join = StreamJoinNode::new(
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

    let old_event = create_event("left", 1000, vec![("key", "k1")]);
    join.process_left(old_event);

    // Advance watermark past the window
    join.update_watermark(20000);

    // Try to join with a new event - old event should be evicted
    let right = create_event("right", 20005, vec![("key", "k1")]);
    let results = join.process_right(right);
    assert_eq!(results.len(), 0, "Old event should be evicted");
}

#[test]
fn test_join_manager_routing() {
    let mut manager = StreamJoinManager::new();
    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    let join = StreamJoinNode::new(
        "stream1".to_string(),
        "stream2".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(10),
        },
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|_, _| true),
    );

    manager.register_join(
        "join1".to_string(),
        join,
        Box::new(move |joined| {
            results_clone.lock().unwrap().push(joined);
        }),
    );

    let event1 = create_event("stream1", 1000, vec![("key", "k1")]);
    let event2 = create_event("stream2", 1005, vec![("key", "k1")]);

    manager.process_event(event1);
    manager.process_event(event2);

    let results_lock = results.lock().unwrap();
    assert_eq!(results_lock.len(), 1);
}

#[test]
fn test_join_manager_multiple_joins() {
    let mut manager = StreamJoinManager::new();
    let results1 = Arc::new(Mutex::new(Vec::new()));
    let results2 = Arc::new(Mutex::new(Vec::new()));
    let r1 = results1.clone();
    let r2 = results2.clone();

    let join1 = StreamJoinNode::new(
        "common".to_string(),
        "stream1".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(10),
        },
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|_, _| true),
    );

    let join2 = StreamJoinNode::new(
        "common".to_string(),
        "stream2".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(10),
        },
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("key").and_then(|v| v.as_string())),
        Box::new(|_, _| true),
    );

    manager.register_join(
        "join1".to_string(),
        join1,
        Box::new(move |j| r1.lock().unwrap().push(j)),
    );
    manager.register_join(
        "join2".to_string(),
        join2,
        Box::new(move |j| r2.lock().unwrap().push(j)),
    );

    let common = create_event("common", 1000, vec![("key", "k1")]);
    let event1 = create_event("stream1", 1005, vec![("key", "k1")]);
    let event2 = create_event("stream2", 1010, vec![("key", "k1")]);

    manager.process_event(common);
    manager.process_event(event1);
    manager.process_event(event2);

    assert_eq!(results1.lock().unwrap().len(), 1);
    assert_eq!(results2.lock().unwrap().len(), 1);
}

#[test]
fn test_join_optimizer_build_smaller() {
    let mut optimizer = JoinOptimizer::new();

    optimizer.register_stream_stats(StreamStats {
        stream_name: "small".to_string(),
        estimated_event_rate: 10.0,
        estimated_cardinality: 100,
        average_event_size: 100,
    });

    optimizer.register_stream_stats(StreamStats {
        stream_name: "large".to_string(),
        estimated_event_rate: 100.0,
        estimated_cardinality: 1000,
        average_event_size: 100,
    });

    let plan = optimizer.optimize_join(
        "small",
        "large",
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(60),
        },
    );

    assert!(!plan.optimizations.is_empty());
    assert!(plan.estimated_cost < 1.0);
}

#[test]
fn test_join_optimizer_memory_estimation() {
    let mut optimizer = JoinOptimizer::new();

    optimizer.register_stream_stats(StreamStats {
        stream_name: "stream1".to_string(),
        estimated_event_rate: 100.0,
        estimated_cardinality: 100,
        average_event_size: 1000,
    });

    optimizer.register_stream_stats(StreamStats {
        stream_name: "stream2".to_string(),
        estimated_event_rate: 100.0,
        estimated_cardinality: 100,
        average_event_size: 1000,
    });

    let memory = optimizer.estimate_memory_usage("stream1", "stream2", Duration::from_secs(10));

    assert!(memory > 2_000_000 && memory < 4_000_000);
}

#[test]
fn test_click_to_purchase_join() {
    // Real-world scenario: joining clicks with purchases
    let mut join = StreamJoinNode::new(
        "clicks".to_string(),
        "purchases".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(600), // 10 minute window
        },
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|click, purchase| {
            // Purchase must happen after click
            purchase.metadata.timestamp as i64 > click.metadata.timestamp as i64
        }),
    );

    let click = create_event(
        "clicks",
        1000,
        vec![("user_id", "user123"), ("product_id", "prod456")],
    );

    let purchase_before = create_event(
        "purchases",
        500,
        vec![("user_id", "user123"), ("product_id", "prod456")],
    );

    let purchase_after = create_event(
        "purchases",
        1500,
        vec![("user_id", "user123"), ("product_id", "prod456")],
    );

    join.process_left(click);

    let results1 = join.process_right(purchase_before);
    assert_eq!(results1.len(), 0, "Purchase before click should not match");

    let results2 = join.process_right(purchase_after);
    assert_eq!(results2.len(), 1, "Purchase after click should match");
}

#[test]
fn test_session_window_join() {
    let mut join = StreamJoinNode::new(
        "left".to_string(),
        "right".to_string(),
        JoinType::Inner,
        JoinStrategy::SessionWindow {
            gap: Duration::from_secs(30),
        },
        Box::new(|e| e.data.get("session_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("session_id").and_then(|v| v.as_string())),
        Box::new(|_, _| true),
    );

    let left1 = create_event("left", 1000, vec![("session_id", "s1")]);
    let right1 = create_event("right", 1020, vec![("session_id", "s1")]);
    let right2 = create_event("right", 2000, vec![("session_id", "s1")]);

    join.process_left(left1);

    let results1 = join.process_right(right1);
    assert_eq!(results1.len(), 1, "Within session gap");

    let results2 = join.process_right(right2);
    assert_eq!(results2.len(), 0, "Outside session gap");
}

#[test]
fn test_join_statistics() {
    let mut join = StreamJoinNode::new(
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

    for i in 0..5 {
        let left = create_event("left", 1000 + i, vec![("key", &format!("k{}", i % 2))]);
        join.process_left(left);
    }

    for i in 0..3 {
        let right = create_event("right", 1000 + i, vec![("key", &format!("k{}", i % 2))]);
        join.process_right(right);
    }

    let stats = join.get_stats();
    assert_eq!(stats.left_buffer_size, 5);
    assert_eq!(stats.right_buffer_size, 3);
    assert_eq!(stats.left_partitions, 2); // k0 and k1
}

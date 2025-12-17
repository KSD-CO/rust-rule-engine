//! StreamAlphaNode Demo
//!
//! This example demonstrates the StreamAlphaNode functionality for filtering
//! and managing stream events with time-based windows.

use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
use rust_rule_engine::streaming::event::StreamEvent;
use rust_rule_engine::streaming::window::WindowType;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    println!("=== StreamAlphaNode Demo ===\n");

    // Demo 1: Basic stream filtering
    basic_filtering_demo();

    // Demo 2: Sliding window
    sliding_window_demo();

    // Demo 3: Tumbling window
    tumbling_window_demo();

    // Demo 4: Event type filtering
    event_type_filtering_demo();
}

fn basic_filtering_demo() {
    println!("📊 Demo 1: Basic Stream Filtering");
    println!("----------------------------------");

    let mut node = StreamAlphaNode::new("user-events", None, None);

    // Create test events
    let mut data1 = HashMap::new();
    data1.insert("user_id".to_string(), Value::String("user123".to_string()));
    data1.insert("action".to_string(), Value::String("login".to_string()));

    let event1 = StreamEvent::new("LoginEvent", data1, "user-events");

    let mut data2 = HashMap::new();
    data2.insert("user_id".to_string(), Value::String("user456".to_string()));

    let event2 = StreamEvent::new("LogoutEvent", data2, "other-stream");

    // Process events
    println!("Event 1 (user-events): {}", node.process_event(&event1));
    println!("Event 2 (other-stream): {}", node.process_event(&event2));
    println!("Events in node: {}\n", node.event_count());
}

fn sliding_window_demo() {
    println!("⏱️  Demo 2: Sliding Window (5 seconds)");
    println!("--------------------------------------");

    let window = WindowSpec {
        duration: Duration::from_secs(5),
        window_type: WindowType::Sliding,
    };

    let mut node = StreamAlphaNode::new("sensors", None, Some(window));

    // Create events at different times
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    for i in 0..3 {
        let timestamp = current_time - (i * 1000); // 1 second apart

        let mut data = HashMap::new();
        data.insert("temperature".to_string(), Value::Number(20.0 + i as f64));
        data.insert(
            "sensor_id".to_string(),
            Value::String(format!("sensor-{}", i)),
        );

        let event = StreamEvent::with_timestamp("TempReading", data, "sensors", timestamp);

        println!(
            "Event {} ({}ms ago): {}",
            i,
            i * 1000,
            node.process_event(&event)
        );
    }

    let stats = node.window_stats();
    println!("\nWindow Stats:");
    println!("  Events: {}", stats.event_count);
    println!("  Duration: {:?}ms", stats.window_duration_ms);
    println!();
}

fn tumbling_window_demo() {
    println!("📦 Demo 3: Tumbling Window (10 seconds)");
    println!("---------------------------------------");

    let window = WindowSpec {
        duration: Duration::from_secs(10),
        window_type: WindowType::Tumbling,
    };

    let mut node = StreamAlphaNode::new("metrics", None, Some(window));

    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Calculate current window boundaries
    let window_duration_ms = 10_000u64;
    let window_start = (current_time / window_duration_ms) * window_duration_ms;

    println!(
        "Current window: {} - {}",
        window_start,
        window_start + window_duration_ms
    );

    // Events in current window
    for i in 0..3 {
        let timestamp = window_start + (i * 2000); // 2 seconds apart within window

        let mut data = HashMap::new();
        data.insert("value".to_string(), Value::Number(i as f64));

        let event = StreamEvent::with_timestamp("MetricEvent", data, "metrics", timestamp);

        println!("Event in current window: {}", node.process_event(&event));
    }

    // Event from previous window
    let old_event = {
        let mut data = HashMap::new();
        data.insert("value".to_string(), Value::Number(999.0));

        StreamEvent::with_timestamp(
            "MetricEvent",
            data,
            "metrics",
            window_start - 5000, // 5 seconds before window
        )
    };

    println!("Event from old window: {}", node.process_event(&old_event));
    println!("Events in window: {}\n", node.event_count());
}

fn event_type_filtering_demo() {
    println!("🔍 Demo 4: Event Type Filtering");
    println!("--------------------------------");

    let mut node = StreamAlphaNode::new("all-events", Some("LoginEvent".to_string()), None);

    // Create different event types
    let mut login_data = HashMap::new();
    login_data.insert("user_id".to_string(), Value::String("user123".to_string()));
    let login_event = StreamEvent::new("LoginEvent", login_data, "all-events");

    let mut logout_data = HashMap::new();
    logout_data.insert("user_id".to_string(), Value::String("user123".to_string()));
    let logout_event = StreamEvent::new("LogoutEvent", logout_data, "all-events");

    let mut purchase_data = HashMap::new();
    purchase_data.insert("amount".to_string(), Value::Number(99.99));
    let purchase_event = StreamEvent::new("PurchaseEvent", purchase_data, "all-events");

    println!("LoginEvent: {}", node.process_event(&login_event));
    println!("LogoutEvent: {}", node.process_event(&logout_event));
    println!("PurchaseEvent: {}", node.process_event(&purchase_event));
    println!("Events in node (only LoginEvent): {}\n", node.event_count());
}

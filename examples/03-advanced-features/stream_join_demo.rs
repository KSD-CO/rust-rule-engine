/// Stream Join Demo - E-Commerce Click-to-Purchase Attribution
///
/// This example demonstrates stream joins by implementing a real-world scenario:
/// joining user click events with purchase events to measure conversion rates
/// and attribution within a time window.
///
/// Key Concepts:
/// - Inner Join: Match clicks with purchases
/// - Left Outer Join: Track clicks that didn't convert
/// - Time Window: 10-minute attribution window
/// - Custom Conditions: Purchase must happen after click
/// - Join Optimization: Partitioning by user_id
///
/// Run with: cargo run --features streaming --example stream_join_demo

#[cfg(feature = "streaming")]
use rust_rule_engine::rete::stream_join_node::{JoinStrategy, JoinType, StreamJoinNode};
#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::event::StreamEvent;
#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::join_manager::StreamJoinManager;
#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::join_optimizer::{JoinOptimizer, StreamStats};
#[cfg(feature = "streaming")]
use std::collections::HashMap;
#[cfg(feature = "streaming")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "streaming")]
use std::time::Duration;

#[cfg(feature = "streaming")]
fn create_click_event(timestamp: i64, user_id: &str, product_id: &str) -> StreamEvent {
    use rust_rule_engine::streaming::event::EventMetadata;
    use rust_rule_engine::types::Value;

    StreamEvent {
        id: format!("click_{}_{}", user_id, timestamp),
        event_type: "ClickEvent".to_string(),
        data: vec![
            ("user_id".to_string(), Value::String(user_id.to_string())),
            (
                "product_id".to_string(),
                Value::String(product_id.to_string()),
            ),
        ]
        .into_iter()
        .collect(),
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "clicks".to_string(),
            sequence: 0,
            tags: std::collections::HashMap::new(),
        },
    }
}

#[cfg(feature = "streaming")]
fn create_purchase_event(
    timestamp: i64,
    user_id: &str,
    product_id: &str,
    amount: &str,
) -> StreamEvent {
    use rust_rule_engine::streaming::event::EventMetadata;
    use rust_rule_engine::types::Value;

    StreamEvent {
        id: format!("purchase_{}_{}", user_id, timestamp),
        event_type: "PurchaseEvent".to_string(),
        data: vec![
            ("user_id".to_string(), Value::String(user_id.to_string())),
            (
                "product_id".to_string(),
                Value::String(product_id.to_string()),
            ),
            ("amount".to_string(), Value::String(amount.to_string())),
        ]
        .into_iter()
        .collect(),
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "purchases".to_string(),
            sequence: 0,
            tags: std::collections::HashMap::new(),
        },
    }
}

#[cfg(feature = "streaming")]
fn demo_basic_inner_join() {
    println!("\n=== Demo 1: Basic Inner Join ===");
    println!("Join clicks with purchases within 10-minute window\n");

    let mut join = StreamJoinNode::new(
        "clicks".to_string(),
        "purchases".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(600), // 10 minutes
        },
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|click, purchase| {
            // Purchase must happen after click
            purchase.metadata.timestamp as i64 > click.metadata.timestamp as i64
        }),
    );

    // User clicks on product at t=0
    let click1 = create_click_event(0, "user123", "laptop");
    println!("Click: user123 clicked on laptop at t=0");

    let results = join.process_left(click1);
    println!("Results: {} (no purchase yet)", results.len());

    // User purchases 5 minutes later
    let purchase1 = create_purchase_event(300, "user123", "laptop", "999.99");
    println!("\nPurchase: user123 bought laptop at t=300 ($999.99)");

    let results = join.process_right(purchase1);
    println!("Results: {} ✓ Conversion tracked!", results.len());

    if let Some(joined) = results.first() {
        if let (Some(click), Some(purchase)) = (&joined.left, &joined.right) {
            let time_to_purchase = purchase.timestamp - click.timestamp;
            println!("  Time to purchase: {} seconds", time_to_purchase);
            println!(
                "  Product: {}",
                purchase
                    .data
                    .get("product_id")
                    .unwrap()
                    .as_string()
                    .unwrap()
            );
            println!(
                "  Amount: ${}",
                purchase.data.get("amount").unwrap().as_string().unwrap()
            );
        }
    }
}

#[cfg(feature = "streaming")]
fn demo_left_outer_join() {
    println!("\n=== Demo 2: Left Outer Join - Track Non-Converting Clicks ===");
    println!("Identify clicks that didn't result in purchases\n");

    let conversion_stats = Arc::new(Mutex::new(HashMap::new()));
    let stats_clone = conversion_stats.clone();

    let mut manager = StreamJoinManager::new();

    let join = StreamJoinNode::new(
        "clicks".to_string(),
        "purchases".to_string(),
        JoinType::LeftOuter,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(600),
        },
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|click, purchase| {
            purchase.metadata.timestamp as i64 > click.metadata.timestamp as i64
        }),
    );

    manager.register_join(
        "conversion_tracking".to_string(),
        join,
        Box::new(move |joined| {
            let mut stats = stats_clone.lock().unwrap();
            if joined.right.is_some() {
                *stats.entry("converted".to_string()).or_insert(0) += 1;
            } else {
                *stats.entry("not_converted".to_string()).or_insert(0) += 1;
            }
        }),
    );

    // Send events
    let events = vec![
        ("click", create_click_event(0, "user1", "laptop")),
        ("click", create_click_event(10, "user2", "phone")),
        ("click", create_click_event(20, "user3", "tablet")),
        (
            "purchase",
            create_purchase_event(50, "user1", "laptop", "999"),
        ),
        (
            "purchase",
            create_purchase_event(100, "user3", "tablet", "499"),
        ),
        // user2 never purchased
    ];

    for (event_type, event) in events {
        println!("{}: {:?}", event_type, event.data);
        manager.process_event(event);
    }

    let stats = conversion_stats.lock().unwrap();
    let converted = stats.get("converted").unwrap_or(&0);
    let not_converted = stats.get("not_converted").unwrap_or(&0);
    let total = converted + not_converted;
    let conversion_rate = if total > 0 {
        (*converted as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    println!("\n📊 Conversion Statistics:");
    println!("  Converted: {}", converted);
    println!("  Not Converted: {}", not_converted);
    println!("  Conversion Rate: {:.1}%", conversion_rate);
}

#[cfg(feature = "streaming")]
fn demo_multi_product_join() {
    println!("\n=== Demo 3: Multi-Product Attribution ===");
    println!("Track which products users clicked before purchasing\n");

    let mut join = StreamJoinNode::new(
        "clicks".to_string(),
        "purchases".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(600),
        },
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|click, purchase| {
            purchase.metadata.timestamp as i64 > click.metadata.timestamp as i64
        }),
    );

    println!("User journey for user456:");

    // User clicks on multiple products
    let click1 = create_click_event(0, "user456", "laptop");
    println!("  t=0: Clicked on laptop");
    join.process_left(click1);

    let click2 = create_click_event(60, "user456", "mouse");
    println!("  t=60: Clicked on mouse");
    join.process_left(click2);

    let click3 = create_click_event(120, "user456", "keyboard");
    println!("  t=120: Clicked on keyboard");
    join.process_left(click3);

    // User purchases laptop
    let purchase = create_purchase_event(180, "user456", "laptop", "999.99");
    println!("  t=180: Purchased laptop");

    let results = join.process_right(purchase);

    println!("\n🎯 Attribution Results:");
    println!("  Found {} clicks before purchase:", results.len());
    for (i, joined) in results.iter().enumerate() {
        if let Some(click) = &joined.left {
            println!(
                "    {}. Click on {} (before purchase)",
                i + 1,
                click.data.get("product_id").unwrap().as_string().unwrap()
            );
        }
    }
}

#[cfg(feature = "streaming")]
fn demo_join_optimization() {
    println!("\n=== Demo 4: Join Optimization ===");
    println!("Optimize join strategy based on stream characteristics\n");

    let mut optimizer = JoinOptimizer::new();

    // Register statistics for clicks stream (high volume)
    optimizer.register_stream_stats(StreamStats {
        stream_name: "clicks".to_string(),
        estimated_event_rate: 1000.0, // 1000 clicks/sec
        estimated_cardinality: 5000,  // 5000 unique users
        average_event_size: 200,      // 200 bytes per event
    });

    // Register statistics for purchases stream (low volume)
    optimizer.register_stream_stats(StreamStats {
        stream_name: "purchases".to_string(),
        estimated_event_rate: 50.0, // 50 purchases/sec
        estimated_cardinality: 500, // 500 unique users
        average_event_size: 300,    // 300 bytes per event
    });

    let plan = optimizer.optimize_join(
        "clicks",
        "purchases",
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(600),
        },
    );

    println!("📈 Optimization Analysis:");
    println!("  Estimated Cost: {:.2}", plan.estimated_cost);
    println!("  Optimizations Applied: {}", plan.optimizations.len());
    println!("  Strategy: {}", plan.explanation);

    // Estimate memory usage
    let memory = optimizer.estimate_memory_usage("clicks", "purchases", Duration::from_secs(600));
    println!("\n💾 Memory Estimation:");
    println!("  Estimated memory: {} MB", memory / 1_000_000);

    // Recommend window size
    let window = optimizer.recommend_window_size("clicks", "purchases", 100_000_000);
    println!("\n⏱️  Window Recommendation:");
    println!("  For 100MB memory limit: {} seconds", window.as_secs());
}

#[cfg(feature = "streaming")]
fn demo_time_window_behavior() {
    println!("\n=== Demo 5: Time Window Behavior ===");
    println!("Observe how events outside the window are not joined\n");

    let mut join = StreamJoinNode::new(
        "clicks".to_string(),
        "purchases".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(300), // 5 minute window
        },
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("user_id").and_then(|v| v.as_string())),
        Box::new(|click, purchase| {
            purchase.metadata.timestamp as i64 > click.metadata.timestamp as i64
        }),
    );

    let click = create_click_event(0, "user789", "phone");
    println!("Click: user789 clicked on phone at t=0");
    join.process_left(click);

    // Purchase within window (4 minutes)
    let purchase1 = create_purchase_event(240, "user789", "phone", "799");
    println!("\nPurchase 1: user789 bought phone at t=240 (4 minutes)");
    let results1 = join.process_right(purchase1);
    println!("  Result: {} ✓ Within 5-minute window", results1.len());

    // Purchase outside window (7 minutes)
    let purchase2 = create_purchase_event(420, "user789", "phone", "799");
    println!("\nPurchase 2: user789 bought phone at t=420 (7 minutes)");
    let results2 = join.process_right(purchase2);
    println!("  Result: {} ✗ Outside 5-minute window", results2.len());

    // Update watermark and check eviction
    println!("\n⏰ Advancing watermark to t=500...");
    join.update_watermark(500);

    let stats = join.get_stats();
    println!("📊 Buffer statistics after watermark:");
    println!("  Left buffer size: {}", stats.left_buffer_size);
    println!("  Right buffer size: {}", stats.right_buffer_size);
    println!("  Current watermark: {}", stats.watermark);
}

#[cfg(feature = "streaming")]
fn main() {
    println!("🚀 Stream Join Demo - E-Commerce Attribution");
    println!("==============================================");

    demo_basic_inner_join();
    demo_left_outer_join();
    demo_multi_product_join();
    demo_join_optimization();
    demo_time_window_behavior();

    println!("\n✅ All demos completed!");
    println!("\n💡 Key Takeaways:");
    println!("  • Inner joins match events from both streams");
    println!("  • Outer joins track unmatched events");
    println!("  • Time windows control how far back to look");
    println!("  • Custom conditions enable complex logic");
    println!("  • Optimization reduces cost and memory");
    println!("  • Watermarks trigger cleanup of old events");
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("This example requires the 'streaming' feature to be enabled.");
    println!("Run with: cargo run --features streaming --example stream_join_demo");
}

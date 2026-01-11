//! Session Window Demo
//!
//! Demonstrates session-based windowing for stream processing.
//! Session windows group events based on inactivity gaps.
//!
//! Run with: cargo run --example session_window_demo --features streaming

#[cfg(feature = "streaming")]
use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::event::StreamEvent;
#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::window::WindowType;
#[cfg(feature = "streaming")]
use rust_rule_engine::types::Value;
#[cfg(feature = "streaming")]
use std::collections::HashMap;
#[cfg(feature = "streaming")]
use std::time::Duration;

#[cfg(feature = "streaming")]
fn main() {
    println!("=== Session Window Demo ===\n");

    // Create a session window with 2-second timeout
    // Events separated by > 2 seconds belong to different sessions
    let window = WindowSpec {
        duration: Duration::from_secs(60), // Not used for session windows
        window_type: WindowType::Session {
            timeout: Duration::from_secs(2),
        },
    };

    let mut node = StreamAlphaNode::new(
        "user-activity",
        Some("UserAction".to_string()),
        Some(window),
    );

    println!("📊 Session Window Configuration:");
    println!("   Stream: user-activity");
    println!("   Event Type: UserAction");
    println!("   Session Timeout: 2 seconds");
    println!("   (Events with >2s gap start a new session)\n");

    // Simulate user activity sessions
    let base_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    println!("🎬 Simulating User Activity Events:\n");

    // Session 1: User browses products
    println!("📱 Session 1 - Product Browsing:");
    simulate_event(&mut node, "ViewProduct", "Laptop", base_time, 0);
    simulate_event(&mut node, "ViewProduct", "Mouse", base_time, 500);
    simulate_event(&mut node, "ViewProduct", "Keyboard", base_time, 1200);
    simulate_event(&mut node, "AddToCart", "Laptop", base_time, 1800);

    println!("   Session 1 Events: {}", node.event_count());
    println!("   All events within 2s - same session ✅\n");

    // Gap of 3 seconds - new session starts
    println!("⏱️  [3 second gap - SESSION TIMEOUT]\n");

    // Session 2: User returns and checks out
    println!("📱 Session 2 - Checkout:");
    simulate_event(&mut node, "ViewCart", "", base_time, 5000);
    simulate_event(&mut node, "Checkout", "Order123", base_time, 5500);
    simulate_event(&mut node, "Payment", "Success", base_time, 6000);

    println!("   Session 2 Events: {}", node.event_count());
    println!("   Previous session cleared, new session started ✅\n");

    // Gap of 4 seconds - another new session
    println!("⏱️  [4 second gap - SESSION TIMEOUT]\n");

    // Session 3: User leaves feedback
    println!("📱 Session 3 - Feedback:");
    simulate_event(&mut node, "ViewOrder", "Order123", base_time, 10000);
    simulate_event(&mut node, "WriteFeedback", "5-stars", base_time, 10800);

    println!("   Session 3 Events: {}", node.event_count());
    println!("   Another new session ✅\n");

    println!("=== Session Window Benefits ===");
    println!("✅ Groups related user actions automatically");
    println!("✅ Adapts to varying activity patterns");
    println!("✅ No fixed time boundaries - more natural");
    println!("✅ Perfect for:");
    println!("   - User session analytics");
    println!("   - Shopping cart abandonment detection");
    println!("   - Fraud detection (unusual session patterns)");
    println!("   - IoT sensor grouping (burst detection)");

    // Real-world example: Cart abandonment
    println!("\n=== Real-World Use Case: Cart Abandonment ===");
    demonstrate_cart_abandonment();
}

#[cfg(feature = "streaming")]
fn simulate_event(
    node: &mut StreamAlphaNode,
    action: &str,
    details: &str,
    base_time: u64,
    offset_ms: u64,
) {
    let mut data = HashMap::new();
    data.insert("action".to_string(), Value::String(action.to_string()));
    if !details.is_empty() {
        data.insert("details".to_string(), Value::String(details.to_string()));
    }

    let event =
        StreamEvent::with_timestamp("UserAction", data, "user-activity", base_time + offset_ms);

    let accepted = node.process_event(&event);
    println!(
        "   t+{:>5}ms: {} {} - {}",
        offset_ms,
        if accepted { "✓" } else { "✗" },
        action,
        if !details.is_empty() { details } else { "" }
    );
}

#[cfg(feature = "streaming")]
fn demonstrate_cart_abandonment() {
    let window = WindowSpec {
        duration: Duration::from_secs(60),
        window_type: WindowType::Session {
            timeout: Duration::from_secs(5), // 5-second session timeout
        },
    };

    let mut cart_node = StreamAlphaNode::new("cart-events", None, Some(window));

    let base_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    println!("\nUser adds items to cart:");
    let mut data = HashMap::new();
    data.insert("action".to_string(), Value::String("AddToCart".to_string()));

    for i in 0..3 {
        let event = StreamEvent::with_timestamp(
            "CartEvent",
            data.clone(),
            "cart-events",
            base_time + (i * 1000),
        );
        cart_node.process_event(&event);
        println!("  ✓ Added item {} to cart", i + 1);
    }

    println!("\n⏱️  [User gets distracted - 6 second gap]\n");

    // After 6 seconds (> 5 second timeout), session expires
    let abandonment_check = StreamEvent::with_timestamp(
        "CheckSession",
        HashMap::new(),
        "cart-events",
        base_time + 7000,
    );

    cart_node.process_event(&abandonment_check);

    println!("🚨 Cart Abandonment Detected!");
    println!("   Previous session had 3 items");
    println!("   Session expired without checkout");
    println!("   → Trigger: Send reminder email 📧");
    println!("   → Trigger: Push notification 📱");
}

#[cfg(not(feature = "streaming"))]
fn main() {
    eprintln!("This example requires the 'streaming' feature.");
    eprintln!("Run with: cargo run --example session_window_demo --features streaming");
}

//! State Management Demo - Stateful Stream Processing
//!
//! Demonstrates state management capabilities including:
//! - Stateful counters and aggregations
//! - Checkpointing for fault tolerance
//! - State recovery after failures
//! - TTL-based state expiration
//!
//! Run with: cargo run --example state_management_demo --features streaming

use rust_rule_engine::streaming::*;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    println!("💾 State Management Demo\n");
    println!("{}", "=".repeat(70));

    // Demo 1: Basic state operations
    demo_basic_state_operations();

    // Demo 2: Stateful event counter
    demo_stateful_counter();

    // Demo 3: Session tracking with state
    demo_session_tracking();

    // Demo 4: Checkpointing and recovery
    demo_checkpoint_recovery();

    // Demo 5: TTL-based state expiration
    demo_state_ttl();

    // Demo 6: Real-world: User activity tracking
    demo_user_activity_tracking();

    println!("\n\n");
    println!("{}", "=".repeat(70));
    println!("✅ All state management demos completed!");
}

fn demo_basic_state_operations() {
    println!("\n📌 Demo 1: Basic State Operations");
    println!("{}", "-".repeat(70));

    let mut store = StateStore::new(StateBackend::Memory);

    // Put values
    store.put("counter", Value::Integer(0)).unwrap();
    store.put("total_amount", Value::Number(0.0)).unwrap();
    store.put("user_name", Value::String("Alice".to_string())).unwrap();

    println!("   ✓ Stored 3 values in state");

    // Get values
    let counter = store.get("counter").unwrap();
    println!("   Counter: {:?}", counter);

    // Update values
    store.update("counter", Value::Integer(42)).unwrap();
    let counter = store.get("counter").unwrap();
    println!("   Updated counter: {:?}", counter);

    // Check existence
    println!("   Contains 'counter': {}", store.contains("counter"));
    println!("   Contains 'missing': {}", store.contains("missing"));

    // List keys
    let keys = store.keys();
    println!("   All keys: {:?}", keys);

    // Statistics
    let stats = store.statistics();
    println!("   Active entries: {}", stats.active_entries);
}

fn demo_stateful_counter() {
    println!("\n📌 Demo 2: Stateful Event Counter");
    println!("{}", "-".repeat(70));

    let store = StateStore::new(StateBackend::Memory);

    // Create a counter operator
    let mut counter_op = StatefulOperator::new(store, |state, event| {
        let key = format!("count_{}", event.event_type);
        let current = state.get(&key)?.unwrap_or(Value::Integer(0));

        if let Value::Integer(count) = current {
            let new_count = count + 1;
            state.put(&key, Value::Integer(new_count))?;
            Ok(Some(Value::Integer(new_count)))
        } else {
            Ok(None)
        }
    });

    // Generate events
    let events = vec![
        ("Login", 5),
        ("Purchase", 3),
        ("Logout", 2),
        ("Login", 3),
        ("Purchase", 1),
    ];

    for (event_type, count) in events {
        for _ in 0..count {
            let mut data = HashMap::new();
            data.insert("type".to_string(), Value::String(event_type.to_string()));
            let event = StreamEvent::new(event_type, data, "app");
            counter_op.process(&event).unwrap();
        }
    }

    // Display counts
    println!("   Event counts:");
    for key in counter_op.state().keys() {
        if let Some(Value::Integer(count)) = counter_op.state().get(&key).unwrap() {
            println!("   - {}: {} events", key.replace("count_", ""), count);
        }
    }
}

fn demo_session_tracking() {
    println!("\n📌 Demo 3: Session Tracking with State");
    println!("{}", "-".repeat(70));

    let store = StateStore::new(StateBackend::Memory);

    // Track user sessions
    let mut session_op = StatefulOperator::new(store, |state, event| {
        if let Some(user_id) = event.get_string("user_id") {
            let session_key = format!("session_{}", user_id);
            
            // Get or create session data
            let mut session_data = if let Some(Value::Object(map)) = state.get(&session_key)? {
                map
            } else {
                HashMap::new()
            };

            // Update session metrics
            let page_views = *session_data
                .get("page_views")
                .and_then(|v| match v {
                    Value::Integer(n) => Some(n),
                    _ => None,
                })
                .unwrap_or(&0) + 1;

            let total_time = *session_data
                .get("total_time_ms")
                .and_then(|v| match v {
                    Value::Number(n) => Some(n),
                    _ => None,
                })
                .unwrap_or(&0.0) + event.get_numeric("duration_ms").unwrap_or(0.0);

            session_data.insert("page_views".to_string(), Value::Integer(page_views));
            session_data.insert("total_time_ms".to_string(), Value::Number(total_time));
            session_data.insert("user_id".to_string(), Value::String(user_id.to_string()));
            session_data.insert("last_page".to_string(), 
                Value::String(event.get_string("page").unwrap_or("unknown").to_string()));

            state.put(&session_key, Value::Object(session_data.clone()))?;

            Ok(Some(Value::Object(session_data)))
        } else {
            Ok(None)
        }
    });

    // Simulate user sessions
    let users = vec!["alice", "bob", "alice", "charlie", "bob", "alice"];
    let pages = vec!["home", "products", "cart", "home", "products", "checkout"];
    let durations = vec![1000.0, 2500.0, 1500.0, 800.0, 3000.0, 4000.0];

    for i in 0..users.len() {
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), Value::String(users[i].to_string()));
        data.insert("page".to_string(), Value::String(pages[i].to_string()));
        data.insert("duration_ms".to_string(), Value::Number(durations[i]));

        let event = StreamEvent::new("PageView", data, "web_app");
        session_op.process(&event).unwrap();
    }

    // Display session summaries
    println!("   User session summaries:");
    for key in session_op.state().keys() {
        if let Some(Value::Object(session)) = session_op.state().get(&key).unwrap() {
            let user_id = session.get("user_id").and_then(|v| match v {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            }).unwrap_or("unknown");

            let page_views = session.get("page_views").and_then(|v| match v {
                Value::Integer(n) => Some(*n),
                _ => None,
            }).unwrap_or(0);

            let total_time = session.get("total_time_ms").and_then(|v| match v {
                Value::Number(n) => Some(*n),
                _ => None,
            }).unwrap_or(0.0);

            let last_page = session.get("last_page").and_then(|v| match v {
                Value::String(s) => Some(s.as_str()),
                _ => None,
            }).unwrap_or("unknown");

            println!("   - {}: {} pages, {:.1}s total, last: {}", 
                     user_id, page_views, total_time / 1000.0, last_page);
        }
    }
}

fn demo_checkpoint_recovery() {
    println!("\n📌 Demo 4: Checkpointing and Recovery");
    println!("{}", "-".repeat(70));

    // Create a file-based state store
    let temp_dir = std::env::temp_dir().join("rust_rule_engine_demo");
    std::fs::create_dir_all(&temp_dir).unwrap();

    let mut store = StateStore::new(StateBackend::File { 
        path: temp_dir.clone() 
    });

    // Add some state
    store.put("request_count", Value::Integer(1000)).unwrap();
    store.put("error_count", Value::Integer(50)).unwrap();
    store.put("success_rate", Value::Number(95.0)).unwrap();

    println!("   ✓ Created initial state with 3 entries");

    // Create checkpoint
    let checkpoint_id = store.checkpoint("before_crash").unwrap();
    println!("   ✓ Created checkpoint: {}", checkpoint_id);

    // List checkpoints
    let checkpoints = store.list_checkpoints();
    println!("   Total checkpoints: {}", checkpoints.len());
    for cp in &checkpoints {
        println!("   - {}: {} entries, {} bytes", 
                 cp.name, cp.entry_count, cp.size_bytes);
    }

    // Simulate crash: clear state
    store.clear().unwrap();
    println!("   ⚠️  Simulated crash: state cleared");
    println!("   State entries after crash: {}", store.len());

    // Restore from checkpoint
    store.restore(&checkpoint_id).unwrap();
    println!("   ✓ Restored from checkpoint");

    // Verify restored state
    let request_count = store.get("request_count").unwrap();
    println!("   Restored request_count: {:?}", request_count);
    println!("   Total entries after restore: {}", store.len());

    // Cleanup
    std::fs::remove_dir_all(&temp_dir).unwrap();
}

fn demo_state_ttl() {
    println!("\n📌 Demo 5: TTL-based State Expiration");
    println!("{}", "-".repeat(70));

    let mut config = StateConfig::default();
    config.enable_ttl = true;
    config.default_ttl = Duration::from_millis(500);

    let mut store = StateStore::with_config(config);

    // Add entries with default TTL
    store.put("temp_token", Value::String("abc123".to_string())).unwrap();
    store.put("temp_session", Value::String("xyz789".to_string())).unwrap();
    
    // Add entry with custom TTL
    store.put_with_ttl("quick_expire", 
                       Value::String("expires fast".to_string()), 
                       Duration::from_millis(200)).unwrap();

    println!("   ✓ Added 3 entries with TTL");
    println!("   Active entries: {}", store.len());

    // Wait for quick_expire to expire
    std::thread::sleep(Duration::from_millis(250));
    
    println!("   After 250ms:");
    println!("   - quick_expire exists: {}", store.contains("quick_expire"));
    println!("   - temp_token exists: {}", store.contains("temp_token"));
    println!("   Active entries: {}", store.len());

    // Cleanup expired
    let expired = store.cleanup_expired();
    println!("   ✓ Cleaned up {} expired entries", expired);

    // Wait for all to expire
    std::thread::sleep(Duration::from_millis(300));
    
    println!("   After 550ms total:");
    println!("   - temp_token exists: {}", store.contains("temp_token"));
    println!("   Active entries: {}", store.len());
}

fn demo_user_activity_tracking() {
    println!("\n📌 Demo 6: Real-World - User Activity Tracking");
    println!("{}", "-".repeat(70));

    let store = StateStore::new(StateBackend::Memory);

    // Track user activity with state
    let mut activity_tracker = StatefulOperator::new(store, |state, event| {
        if let Some(user_id) = event.get_string("user_id") {
            let key = format!("user_{}", user_id);
            
            // Get or initialize user profile
            let mut profile = if let Some(Value::Object(map)) = state.get(&key)? {
                map
            } else {
                let mut new_profile = HashMap::new();
                new_profile.insert("first_seen".to_string(), 
                    Value::Number(event.metadata.timestamp as f64));
                new_profile
            };

            // Update activity metrics
            let login_count = *profile
                .get("login_count")
                .and_then(|v| match v { Value::Integer(n) => Some(n), _ => None })
                .unwrap_or(&0);

            let transaction_count = *profile
                .get("transaction_count")
                .and_then(|v| match v { Value::Integer(n) => Some(n), _ => None })
                .unwrap_or(&0);

            let total_spent = *profile
                .get("total_spent")
                .and_then(|v| match v { Value::Number(n) => Some(n), _ => None })
                .unwrap_or(&0.0);

            // Update based on event type
            match event.event_type.as_str() {
                "Login" => {
                    profile.insert("login_count".to_string(), Value::Integer(login_count + 1));
                    profile.insert("last_login".to_string(), 
                        Value::Number(event.metadata.timestamp as f64));
                }
                "Purchase" => {
                    let amount = event.get_numeric("amount").unwrap_or(0.0);
                    profile.insert("transaction_count".to_string(), 
                        Value::Integer(transaction_count + 1));
                    profile.insert("total_spent".to_string(), 
                        Value::Number(total_spent + amount));
                    profile.insert("last_purchase".to_string(), 
                        Value::Number(event.metadata.timestamp as f64));
                }
                _ => {}
            }

            profile.insert("last_activity".to_string(), 
                Value::Number(event.metadata.timestamp as f64));

            state.put(&key, Value::Object(profile.clone()))?;
            Ok(Some(Value::Object(profile)))
        } else {
            Ok(None)
        }
    });

    // Simulate user activities
    let activities = vec![
        ("alice", "Login", 0.0),
        ("alice", "Purchase", 99.99),
        ("bob", "Login", 0.0),
        ("alice", "Purchase", 49.99),
        ("charlie", "Login", 0.0),
        ("bob", "Purchase", 199.99),
        ("alice", "Purchase", 29.99),
        ("bob", "Login", 0.0),
    ];

    for (user, event_type, amount) in activities {
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), Value::String(user.to_string()));
        if amount > 0.0 {
            data.insert("amount".to_string(), Value::Number(amount));
        }

        let event = StreamEvent::new(event_type, data, "app");
        activity_tracker.process(&event).unwrap();
    }

    // Generate user analytics
    println!("   User activity analytics:");
    
    let mut users: Vec<_> = activity_tracker.state().keys()
        .into_iter()
        .filter(|k| k.starts_with("user_"))
        .collect();
    users.sort();

    for key in users {
        if let Some(Value::Object(profile)) = activity_tracker.state().get(&key).unwrap() {
            let user_id = key.replace("user_", "");
            let logins = profile.get("login_count")
                .and_then(|v| match v { Value::Integer(n) => Some(*n), _ => None })
                .unwrap_or(0);
            
            let purchases = profile.get("transaction_count")
                .and_then(|v| match v { Value::Integer(n) => Some(*n), _ => None })
                .unwrap_or(0);
            
            let total = profile.get("total_spent")
                .and_then(|v| match v { Value::Number(n) => Some(*n), _ => None })
                .unwrap_or(0.0);

            println!("   - {}: {} logins, {} purchases, ${:.2} spent", 
                     user_id, logins, purchases, total);
        }
    }

    // Create checkpoint for this state
    let checkpoint_id = activity_tracker.checkpoint("user_analytics").unwrap();
    println!("\n   ✓ Checkpoint created: {}", checkpoint_id);

    // Show statistics
    let stats = activity_tracker.state().statistics();
    println!("   State statistics:");
    println!("   - Active entries: {}", stats.active_entries);
    println!("   - Checkpoints: {}", stats.checkpoint_count);
}

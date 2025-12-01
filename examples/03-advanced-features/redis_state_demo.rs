//! Redis State Backend Demo
//!
//! This example demonstrates using Redis as a distributed state backend
//! for scalable stream processing across multiple instances.
//!
//! **Prerequisites**: Redis server running on localhost:6379
//! Start Redis with: `docker run -d -p 6379:6379 redis:latest`

use rust_rule_engine::streaming::*;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔴 Redis State Backend Demo\n");
    println!("{}", "=".repeat(70));
    println!();
    
    // Check if Redis is available
    if !is_redis_available() {
        println!("❌ Redis is not running!");
        println!();
        println!("Please start Redis server:");
        println!("  docker run -d -p 6379:6379 redis:latest");
        println!();
        println!("Or install Redis locally:");
        println!("  Ubuntu/Debian: sudo apt-get install redis-server");
        println!("  macOS: brew install redis");
        println!();
        return Ok(());
    }

    println!("✓ Redis server detected");
    println!();

    demo_1_basic_redis_operations()?;
    demo_2_distributed_counter()?;
    demo_3_session_state_with_ttl()?;
    demo_4_multi_instance_coordination()?;

    println!();
    println!("{}", "=".repeat(70));
    println!("✅ All Redis demos completed!");
    println!();
    println!("💡 Key Benefits:");
    println!("   • Distributed state across multiple instances");
    println!("   • Automatic persistence and replication");
    println!("   • Built-in TTL support");
    println!("   • High throughput (100k+ ops/sec)");
    println!("   • Redis Cluster for horizontal scaling");
    Ok(())
}

/// Check if Redis is available
fn is_redis_available() -> bool {
    #[cfg(feature = "streaming-redis")]
    {
        if let Ok(client) = redis::Client::open("redis://127.0.0.1:6379") {
            if let Ok(mut conn) = client.get_connection() {
                use redis::Commands;
                return conn.set::<_, _, ()>("__test__", "ping").is_ok();
            }
        }
        false
    }
    
    #[cfg(not(feature = "streaming-redis"))]
    {
        println!("❌ Redis support not enabled!");
        println!("   Rebuild with: cargo build --features streaming-redis");
        false
    }
}

/// Demo 1: Basic Redis Operations
fn demo_1_basic_redis_operations() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "streaming-redis")]
    {
        println!("📌 Demo 1: Basic Redis Operations");
        println!("{}", "-".repeat(70));

        let backend = StateBackend::Redis {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: "demo1".to_string(),
        };

        let mut store = StateStore::new(backend);

        // Put values
        println!("   Writing to Redis...");
        store.put("user_count", Value::Integer(42))?;
        store.put("app_name", Value::String("RustRuleEngine".to_string()))?;
        store.put("active", Value::Boolean(true))?;

        // Read back
        println!("   Reading from Redis...");
        if let Some(Value::Integer(count)) = store.get("user_count")? {
            println!("   ✓ user_count: {}", count);
        }

        if let Some(Value::String(name)) = store.get("app_name")? {
            println!("   ✓ app_name: {}", name);
        }

        // Update
        store.update("user_count", Value::Integer(100))?;
        if let Some(Value::Integer(count)) = store.get("user_count")? {
            println!("   ✓ Updated user_count: {}", count);
        }

        // List keys
        let keys = store.keys();
        println!("   ✓ Keys in Redis: {:?}", keys);

        // Cleanup
        let keys_to_delete = keys.clone();
        for key in &keys_to_delete {
            store.delete(&key)?;
        }
        println!("   ✓ Cleaned up {} keys", keys_to_delete.len());
        println!();
    }

    #[cfg(not(feature = "streaming-redis"))]
    {
        println!("⚠️  Demo 1 skipped (Redis support not enabled)\n");
    }

    Ok(())
}

/// Demo 2: Distributed Counter
fn demo_2_distributed_counter() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "streaming-redis")]
    {
        println!("📌 Demo 2: Distributed Event Counter");
        println!("{}", "-".repeat(70));
        println!("   Simulating sequential event processing with shared Redis state...");

        let backend = StateBackend::Redis {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: "counter".to_string(),
        };

        // Process events sequentially to demonstrate Redis state sharing
        // (For true atomic operations, use Redis INCR command - can be added later)
        let mut total_processed = 0;
        
        for instance_id in 1..=3 {
            let store = StateStore::new(backend.clone());
            let mut operator = StatefulOperator::new(store, move |state, event| {
                let key = format!("events_{}", event.event_type);
                
                // Get current count from Redis
                let current = state.get(&key)?
                    .and_then(|v| match v {
                        Value::Integer(n) => Some(n),
                        _ => None,
                    })
                    .unwrap_or(0);
                
                let new_count = current + 1;
                state.put(&key, Value::Integer(new_count))?;
                
                Ok(Some(Value::Integer(new_count)))
            });

            // Each instance processes 10 events
            for i in 0..10 {
                let mut data = HashMap::new();
                data.insert("instance".to_string(), Value::Integer(instance_id));
                data.insert("seq".to_string(), Value::Integer(i));
                
                let event = StreamEvent::new("Purchase", data, &format!("instance_{}", instance_id));
                operator.process(&event).ok();
                total_processed += 1;
            }
            
            println!("   ✓ Instance {} completed (10 events)", instance_id);
        }

        // Check final count
        let store = StateStore::new(backend.clone());
        if let Some(Value::Integer(total)) = store.get("events_Purchase")? {
            println!("   ✓ Total events in Redis: {}", total);
            println!("   ✓ State persisted across {} instances", 3);
        }

        // Cleanup
        let mut cleanup_store = StateStore::new(backend);
        cleanup_store.delete("events_Purchase")?;
        println!();
    }

    #[cfg(not(feature = "streaming-redis"))]
    {
        println!("⚠️  Demo 2 skipped (Redis support not enabled)\n");
    }

    Ok(())
}

/// Demo 3: Session State with TTL
fn demo_3_session_state_with_ttl() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "streaming-redis")]
    {
        println!("📌 Demo 3: Session State with TTL");
        println!("{}", "-".repeat(70));

        let backend = StateBackend::Redis {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: "session".to_string(),
        };

        let mut store = StateStore::new(backend.clone());

        // Create sessions with 2-second TTL
        println!("   Creating sessions with 2s TTL...");
        for user_id in 1..=5 {
            let key = format!("user_{}", user_id);
            let mut session_data = HashMap::new();
            session_data.insert("user_id".to_string(), Value::Integer(user_id));
            session_data.insert("login_time".to_string(), Value::Integer(chrono::Utc::now().timestamp()));
            
            store.put_with_ttl(&key, Value::Object(session_data), Duration::from_secs(2))?;
            println!("   ✓ Created session for user_{}", user_id);
        }

        let keys = store.keys();
        println!("   Active sessions: {}", keys.len());

        println!("   Waiting for TTL expiration (3 seconds)...");
        thread::sleep(Duration::from_secs(3));

        // Check after TTL
        let remaining = store.keys();
        println!("   Active sessions after TTL: {} (Redis auto-expired)", remaining.len());

        // Cleanup any remaining
        for key in remaining {
            store.delete(&key)?;
        }
        println!();
    }

    #[cfg(not(feature = "streaming-redis"))]
    {
        println!("⚠️  Demo 3 skipped (Redis support not enabled)\n");
    }

    Ok(())
}

/// Demo 4: Multi-Instance Coordination
fn demo_4_multi_instance_coordination() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "streaming-redis")]
    {
        println!("📌 Demo 4: Multi-Instance Stream Processing Coordination");
        println!("{}", "-".repeat(70));
        println!("   Simulating distributed stream processing with shared state...");

        let backend = StateBackend::Redis {
            url: "redis://127.0.0.1:6379".to_string(),
            key_prefix: "distributed".to_string(),
        };

        // Track metrics across instances
        let metrics = vec![
            ("total_events", 0),
            ("total_revenue", 0),
            ("peak_throughput", 0),
        ];

        // Initialize metrics in Redis
        let mut store = StateStore::new(backend.clone());
        for (key, value) in &metrics {
            store.put(*key, Value::Integer(*value))?;
        }

        println!("   Starting 4 parallel stream processors...");

        let mut handles = vec![];
        for instance_id in 1..=4 {
            let backend_clone = backend.clone();
            
            let handle = thread::spawn(move || {
                let store = StateStore::new(backend_clone);
                let mut operator = StatefulOperator::new(store, |state, event| {
                    // Update shared metrics
                    let total = state.get("total_events")?
                        .and_then(|v| match v { Value::Integer(n) => Some(n), _ => None })
                        .unwrap_or(0);
                    state.put("total_events", Value::Integer(total + 1))?;
                    
                    // Update revenue if it's a purchase
                    if event.event_type == "Purchase" {
                        if let Some(Value::Number(amount)) = event.data.get("amount") {
                            let revenue = state.get("total_revenue")?
                                .and_then(|v| match v { Value::Integer(n) => Some(n), _ => None })
                                .unwrap_or(0);
                            state.put("total_revenue", Value::Integer(revenue + *amount as i64))?;
                        }
                    }
                    
                    Ok(Some(Value::Integer(total + 1)))
                });

                // Process events
                for i in 0..25 {
                    let mut data = HashMap::new();
                    data.insert("instance".to_string(), Value::Integer(instance_id));
                    data.insert("amount".to_string(), Value::Number(50.0 + i as f64));
                    
                    let event = StreamEvent::new("Purchase", data, &format!("instance_{}", instance_id));
                    operator.process(&event).ok();
                    
                    thread::sleep(Duration::from_millis(5));
                }
            });
            
            handles.push(handle);
        }

        // Wait for completion
        for handle in handles {
            handle.join().unwrap();
        }

        println!("   All processors completed!");
        println!();
        println!("   Final metrics (aggregated across all instances):");
        
        let store = StateStore::new(backend.clone());
        if let Some(Value::Integer(events)) = store.get("total_events")? {
            println!("   • Total events processed: {}", events);
        }
        if let Some(Value::Integer(revenue)) = store.get("total_revenue")? {
            println!("   • Total revenue: ${}", revenue);
        }

        // Cleanup
        let mut cleanup_store = StateStore::new(backend);
        for key in cleanup_store.keys() {
            cleanup_store.delete(&key)?;
        }
        println!();
    }

    #[cfg(not(feature = "streaming-redis"))]
    {
        println!("⚠️  Demo 4 skipped (Redis support not enabled)\n");
    }

    Ok(())
}

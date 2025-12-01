//! Stream Operators Demo - Fluent API for Stream Processing
//!
//! This example demonstrates the powerful fluent API for building
//! complex stream processing pipelines.
//!
//! Run with: cargo run --example stream_operators_demo --features streaming

use rust_rule_engine::streaming::*;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    println!("🌊 Stream Operators Demo - Fluent API\n");
    println!("{}", "=".repeat(60));

    // Create sample events (e-commerce transactions)
    let events = create_sample_events();
    println!("\n📊 Created {} sample transaction events", events.len());

    // Demo 1: Basic filtering and mapping
    demo_filter_and_map(&events);

    // Demo 2: Key-by and aggregation
    demo_key_by_aggregation(&events);

    // Demo 3: Windowing
    demo_windowing(&events);

    // Demo 4: Complex pipeline
    demo_complex_pipeline(&events);

    // Demo 5: Group by with aggregation
    demo_group_by(&events);

    // Demo 6: Reduce operations
    demo_reduce(&events);

    // Demo 7: Custom aggregator
    demo_custom_aggregator(&events);

    println!("\n{}", "\n".repeat(2));
    println!("{}", "=".repeat(60));
    println!("✅ All stream operator demos completed successfully!");
}

fn create_sample_events() -> Vec<StreamEvent> {
    let mut events = Vec::new();

    // Simulate transactions from different users and categories
    let users = vec!["alice", "bob", "charlie", "alice", "bob"];
    let categories = vec!["Electronics", "Books", "Clothing", "Electronics", "Books"];
    let amounts = vec![299.99, 29.99, 49.99, 499.99, 19.99];

    for i in 0..20 {
        let mut data = HashMap::new();
        
        let user_idx = i % users.len();
        let amount = amounts[user_idx] * (1.0 + (i as f64 * 0.1));
        
        data.insert("user_id".to_string(), Value::String(users[user_idx].to_string()));
        data.insert("category".to_string(), Value::String(categories[user_idx].to_string()));
        data.insert("amount".to_string(), Value::Number(amount));
        data.insert("quantity".to_string(), Value::Integer((i % 5 + 1) as i64));
        data.insert("is_premium".to_string(), Value::Boolean(i % 3 == 0));

        events.push(StreamEvent::new(
            "Transaction",
            data,
            "e-commerce-system"
        ));
    }

    events
}

fn demo_filter_and_map(events: &[StreamEvent]) {
    println!("\n📌 Demo 1: Filter & Map Operations");
    println!("{}", "-".repeat(60));

    let stream = DataStream::from_events(events.to_vec());

    // Filter high-value transactions and add discount
    let result = stream
        .filter(|e| {
            e.get_numeric("amount").unwrap_or(0.0) > 100.0
        })
        .map(|mut e| {
            // Add 10% discount tag for high-value transactions
            if let Some(amount) = e.get_numeric("amount") {
                let discounted = amount * 0.9;
                e.data.insert("discounted_amount".to_string(), Value::Number(discounted));
                e.add_tag("discount_applied", "10%");
            }
            e
        })
        .collect();

    println!("   High-value transactions (>$100): {}", result.len());
    
    if let Some(first) = result.first() {
        let original = first.get_numeric("amount").unwrap_or(0.0);
        let discounted = first.get_numeric("discounted_amount").unwrap_or(0.0);
        println!("   Sample: ${:.2} → ${:.2} (after discount)", original, discounted);
    }
}

fn demo_key_by_aggregation(events: &[StreamEvent]) {
    println!("\n📌 Demo 2: Key-By and Aggregation");
    println!("{}", "-".repeat(60));

    let stream = DataStream::from_events(events.to_vec());

    // Group by user and calculate total spending
    let user_totals = stream
        .key_by(|e| e.get_string("user_id").unwrap_or("unknown").to_string())
        .aggregate(Sum::new("amount"));

    println!("   Total spending per user:");
    for (user, total) in user_totals {
        if let Some(amount) = total.as_number() {
            println!("   - {}: ${:.2}", user, amount);
        }
    }
}

fn demo_windowing(events: &[StreamEvent]) {
    println!("\n📌 Demo 3: Window Operations");
    println!("{}", "-".repeat(60));

    let stream = DataStream::from_events(events.to_vec());

    // Apply tumbling windows and count events per window
    let windowed = stream.window(WindowConfig::tumbling(Duration::from_secs(60)));
    
    let window_counts = windowed.counts();
    
    println!("   Number of windows created: {}", window_counts.len());
    println!("   Events per window: {:?}", window_counts);

    // Aggregate within windows
    let stream2 = DataStream::from_events(events.to_vec());
    let window_sums = stream2
        .window(WindowConfig::sliding(Duration::from_secs(60)))
        .aggregate(Sum::new("amount"));

    println!("   Total amount per window:");
    for (idx, sum) in window_sums.iter().enumerate() {
        if let Some(amount) = sum.as_number() {
            println!("   - Window {}: ${:.2}", idx + 1, amount);
        }
    }
}

fn demo_complex_pipeline(events: &[StreamEvent]) {
    println!("\n📌 Demo 4: Complex Pipeline");
    println!("{}", "-".repeat(60));

    let stream = DataStream::from_events(events.to_vec());

    // Complex pipeline: filter → map → key_by → window → aggregate
    let result = stream
        .filter(|e| {
            // Only premium transactions
            e.get_boolean("is_premium").unwrap_or(false)
        })
        .map(|mut e| {
            // Add VIP discount
            if let Some(amount) = e.get_numeric("amount") {
                e.data.insert("vip_price".to_string(), Value::Number(amount * 0.85));
            }
            e
        })
        .key_by(|e| e.get_string("category").unwrap_or("Other").to_string())
        .window(WindowConfig::tumbling(Duration::from_secs(120)))
        .aggregate(Average::new("vip_price"));

    println!("   Premium transactions by category:");
    for (category, windows) in result {
        if let Some(first_window) = windows.first() {
            if let Some(avg) = first_window.as_number() {
                println!("   - {}: avg ${:.2}", category, avg);
            }
        }
    }
}

fn demo_group_by(events: &[StreamEvent]) {
    println!("\n📌 Demo 5: Group By Operations");
    println!("{}", "-".repeat(60));

    let stream = DataStream::from_events(events.to_vec());

    // Group by category and get counts first
    let grouped_for_counts = stream.clone().group_by(|e| {
        e.get_string("category").unwrap_or("Unknown").to_string()
    });
    let counts = grouped_for_counts.count();

    // Group again for averages
    let stream2 = DataStream::from_events(events.to_vec());
    let grouped_for_avg = stream2.group_by(|e| {
        e.get_string("category").unwrap_or("Unknown").to_string()
    });
    let averages = grouped_for_avg.aggregate(Average::new("amount"));

    println!("   Category statistics:");
    for (category, count) in counts {
        if let Some(avg_result) = averages.get(&category) {
            if let Some(avg) = avg_result.as_number() {
                println!("   - {}: {} transactions, avg ${:.2}", category, count, avg);
            }
        }
    }
}

fn demo_reduce(events: &[StreamEvent]) {
    println!("\n📌 Demo 6: Reduce Operations");
    println!("{}", "-".repeat(60));

    let stream = DataStream::from_events(events.to_vec());

    // Reduce to find total and max transaction
    let total_event = stream
        .reduce(|mut acc, e| {
            let acc_amount = acc.get_numeric("amount").unwrap_or(0.0);
            let e_amount = e.get_numeric("amount").unwrap_or(0.0);
            
            // Accumulate total
            acc.data.insert("amount".to_string(), Value::Number(acc_amount + e_amount));
            
            // Track max
            let acc_max = acc.get_numeric("max_amount").unwrap_or(0.0);
            let new_max = acc_max.max(e_amount);
            acc.data.insert("max_amount".to_string(), Value::Number(new_max));
            
            acc
        });

    if let Some(result) = total_event {
        println!("   Total amount: ${:.2}", result.get_numeric("amount").unwrap_or(0.0));
        println!("   Max transaction: ${:.2}", result.get_numeric("max_amount").unwrap_or(0.0));
    }
}

fn demo_custom_aggregator(events: &[StreamEvent]) {
    println!("\n📌 Demo 7: Custom Aggregator");
    println!("{}", "-".repeat(60));

    let stream = DataStream::from_events(events.to_vec());

    // Create a custom aggregator that calculates multiple statistics
    let custom_agg = CustomAggregator::new(|events: &[StreamEvent]| {
        let amounts: Vec<f64> = events
            .iter()
            .filter_map(|e| e.get_numeric("amount"))
            .collect();

        if amounts.is_empty() {
            return AggregateResult::None;
        }

        let total: f64 = amounts.iter().sum();
        let avg = total / amounts.len() as f64;
        let min = amounts.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = amounts.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let mut result = HashMap::new();
        result.insert("total".to_string(), Value::Number(total));
        result.insert("average".to_string(), Value::Number(avg));
        result.insert("min".to_string(), Value::Number(min));
        result.insert("max".to_string(), Value::Number(max));
        result.insert("count".to_string(), Value::Number(amounts.len() as f64));

        AggregateResult::Map(result)
    });

    let result = stream.aggregate(custom_agg);

    if let Some(stats) = result.as_map() {
        println!("   Transaction statistics:");
        println!("   - Count: {}", stats.get("count").and_then(|v| match v {
            Value::Number(n) => Some(*n as usize),
            _ => None,
        }).unwrap_or(0));
        println!("   - Total: ${:.2}", stats.get("total").and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        }).unwrap_or(0.0));
        println!("   - Average: ${:.2}", stats.get("average").and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        }).unwrap_or(0.0));
        println!("   - Min: ${:.2}", stats.get("min").and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        }).unwrap_or(0.0));
        println!("   - Max: ${:.2}", stats.get("max").and_then(|v| match v {
            Value::Number(n) => Some(*n),
            _ => None,
        }).unwrap_or(0.0));
    }
}

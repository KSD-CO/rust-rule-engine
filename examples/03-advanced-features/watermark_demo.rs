//! Watermark and Late Data Handling Demo
//!
//! This example demonstrates watermark-based stream processing and handling of
//! out-of-order events in distributed streaming scenarios.

use rust_rule_engine::streaming::*;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⏱️  Watermark & Late Data Handling Demo\n");
    println!("{}", "=".repeat(70));
    println!();

    demo_1_bounded_out_of_order()?;
    demo_2_late_data_strategies()?;
    demo_3_watermarked_stream()?;
    demo_4_real_world_iot_sensors()?;

    println!();
    println!("{}", "=".repeat(70));
    println!("✅ All watermark demos completed!");
    Ok(())
}

/// Demo 1: Bounded Out-of-Orderness
fn demo_1_bounded_out_of_order() -> Result<(), Box<dyn std::error::Error>> {
    println!("📌 Demo 1: Bounded Out-of-Orderness Watermarks");
    println!("{}", "-".repeat(70));
    
    // Allow events to be 500ms out of order
    let strategy = WatermarkStrategy::BoundedOutOfOrder {
        max_delay: Duration::from_millis(500),
    };
    
    let mut generator = WatermarkGenerator::new(strategy);
    
    // Simulate events arriving out of order
    let events = vec![
        (1000, "Event 1"),
        (1200, "Event 2"),
        (1100, "Event 3 - out of order!"),
        (1500, "Event 4"),
        (1300, "Event 5 - out of order!"),
        (2000, "Event 6"),
    ];
    
    println!("   Processing events with 500ms out-of-order tolerance:");
    for (timestamp, name) in events {
        let event = create_event(timestamp, name);
        
        if let Some(watermark) = generator.process_event(&event) {
            println!("   - Event at {}ms: {} → Watermark: {}ms", 
                     timestamp, name, watermark.timestamp);
        } else {
            println!("   - Event at {}ms: {} (no watermark update)", 
                     timestamp, name);
        }
    }
    
    let final_wm = generator.current_watermark();
    println!("   ✓ Final watermark: {}ms", final_wm.timestamp);
    println!();
    Ok(())
}

/// Demo 2: Late Data Strategies
fn demo_2_late_data_strategies() -> Result<(), Box<dyn std::error::Error>> {
    println!("📌 Demo 2: Late Data Handling Strategies");
    println!("{}", "-".repeat(70));
    
    // Strategy 1: Drop late events
    println!("   Strategy 1: Drop late events");
    let mut drop_handler = LateDataHandler::new(LateDataStrategy::Drop);
    let watermark = Watermark::new(2000);
    
    let late_event = create_event(1500, "Late event");
    match drop_handler.handle_late_event(late_event, &watermark) {
        LateEventDecision::Drop => {
            println!("   - Event at 1500ms is 500ms late → Dropped");
        }
        _ => {}
    }
    
    let stats = drop_handler.stats();
    println!("   - Stats: {} late, {} dropped\n", stats.total_late, stats.dropped);
    
    // Strategy 2: Allowed lateness
    println!("   Strategy 2: Allow late events within threshold");
    let mut allowed_handler = LateDataHandler::new(LateDataStrategy::AllowedLateness {
        max_lateness: Duration::from_millis(300),
    });
    
    let late_event1 = create_event(1800, "Slightly late");
    match allowed_handler.handle_late_event(late_event1, &watermark) {
        LateEventDecision::Process(_e) => {
            println!("   - Event at 1800ms is 200ms late → Accepted (within 300ms threshold)");
        }
        _ => {}
    }
    
    let late_event2 = create_event(1600, "Too late");
    match allowed_handler.handle_late_event(late_event2, &watermark) {
        LateEventDecision::Drop => {
            println!("   - Event at 1600ms is 400ms late → Dropped (exceeds 300ms threshold)");
        }
        _ => {}
    }
    
    let stats = allowed_handler.stats();
    println!("   - Stats: {} late, {} allowed, {} dropped\n", 
             stats.total_late, stats.allowed, stats.dropped);
    
    // Strategy 3: Side output
    println!("   Strategy 3: Route late events to side output");
    let mut side_handler = LateDataHandler::new(LateDataStrategy::SideOutput);
    
    for ts in vec![1500, 1700, 1900] {
        let late_event = create_event(ts, &format!("Late event {}", ts));
        side_handler.handle_late_event(late_event, &watermark);
    }
    
    println!("   - Collected {} late events in side output", side_handler.side_output().len());
    for event in side_handler.side_output() {
        println!("     • {}: {}", event.metadata.timestamp, event.event_type);
    }
    
    println!();
    Ok(())
}

/// Demo 3: Watermarked Stream Processing
fn demo_3_watermarked_stream() -> Result<(), Box<dyn std::error::Error>> {
    println!("📌 Demo 3: Watermarked Stream Processing");
    println!("{}", "-".repeat(70));
    
    // Create watermarked stream with 300ms out-of-order tolerance
    let watermark_strategy = WatermarkStrategy::BoundedOutOfOrder {
        max_delay: Duration::from_millis(300),
    };
    let late_strategy = LateDataStrategy::AllowedLateness {
        max_lateness: Duration::from_millis(200),
    };
    
    let mut stream = WatermarkedStream::new(watermark_strategy, late_strategy);
    
    // Simulate mixed in-order and out-of-order events
    let events = vec![
        (1000, 100.0),
        (1200, 150.0),
        (1100, 120.0),  // 100ms late - should be accepted
        (1500, 200.0),
        (1000, 90.0),   // 500ms late - should be dropped
        (1600, 180.0),
        (1400, 160.0),  // 200ms late - should be accepted
        (2000, 250.0),
    ];
    
    println!("   Processing {} events:", events.len());
    for (ts, value) in events {
        let event = create_event(ts, &format!("value_{}", value));
        stream.add_event(event)?;
    }
    
    println!("   ✓ On-time events: {}", stream.events().len());
    println!("   ✓ Current watermark: {}ms", stream.current_watermark().timestamp);
    
    let stats = stream.late_stats();
    println!("   Late data statistics:");
    println!("   - Total late events: {}", stats.total_late);
    println!("   - Allowed (processed): {}", stats.allowed);
    println!("   - Dropped: {}", stats.dropped);
    
    println!("   Watermark progression:");
    for (i, wm) in stream.watermark_history().iter().enumerate() {
        println!("   - Step {}: {}ms", i + 1, wm.timestamp);
    }
    
    println!();
    Ok(())
}

/// Demo 4: Real-World IoT Sensor Scenario
fn demo_4_real_world_iot_sensors() -> Result<(), Box<dyn std::error::Error>> {
    println!("📌 Demo 4: Real-World - IoT Sensor Data with Network Delays");
    println!("{}", "-".repeat(70));
    
    // IoT sensors can have network delays and out-of-order delivery
    // Allow 1 second out-of-order tolerance for network delays
    let watermark_strategy = WatermarkStrategy::BoundedOutOfOrder {
        max_delay: Duration::from_secs(1),
    };
    
    // Accept events up to 500ms late (reasonable for network jitter)
    let late_strategy = LateDataStrategy::AllowedLateness {
        max_lateness: Duration::from_millis(500),
    };
    
    let mut sensor_stream = WatermarkedStream::new(watermark_strategy, late_strategy);
    
    // Simulate sensor readings with realistic network delays
    println!("   Simulating sensor data from 3 devices:");
    
    let sensor_data = vec![
        (1000, "sensor_001", 23.5, "Temperature"),
        (1100, "sensor_002", 24.1, "Temperature"),
        (1200, "sensor_003", 22.8, "Temperature"),
        (1050, "sensor_001", 23.6, "Temperature"),  // Delayed packet
        (1300, "sensor_001", 23.7, "Temperature"),
        (1150, "sensor_002", 24.2, "Temperature"),  // Delayed packet
        (1400, "sensor_002", 24.0, "Temperature"),
        (900, "sensor_003", 22.5, "Temperature"),   // Very late - should be dropped
        (1500, "sensor_003", 23.0, "Temperature"),
        (1600, "sensor_001", 23.9, "Temperature"),
    ];
    
    for (ts, sensor_id, value, measurement_type) in &sensor_data {
        let mut data = HashMap::new();
        data.insert("sensor_id".to_string(), Value::String(sensor_id.to_string()));
        data.insert("value".to_string(), Value::Number(*value));
        data.insert("measurement".to_string(), Value::String(measurement_type.to_string()));
        
        let mut event = StreamEvent::new("SensorReading", data, *sensor_id);
        event.metadata.timestamp = *ts;
        
        let is_late = sensor_stream.current_watermark().is_late(event.metadata.timestamp);
        if is_late {
            println!("   ⚠️  Late data from {}: {}ms (reading: {}°C)", 
                     sensor_id, ts, value);
        } else {
            println!("   ✓ On-time from {}: {}ms (reading: {}°C)", 
                     sensor_id, ts, value);
        }
        
        sensor_stream.add_event(event)?;
    }
    
    println!();
    println!("   Processing summary:");
    println!("   - Total readings received: {}", sensor_data.len());
    println!("   - Valid readings processed: {}", sensor_stream.events().len());
    
    let stats = sensor_stream.late_stats();
    println!("   - Late arrivals detected: {}", stats.total_late);
    println!("   - Accepted despite lateness: {}", stats.allowed);
    println!("   - Dropped (too late): {}", stats.dropped);
    println!("   - Final watermark: {}ms", sensor_stream.current_watermark().timestamp);
    
    // Calculate statistics on accepted readings
    let mut temp_sum = 0.0;
    let mut count = 0;
    
    for event in sensor_stream.events() {
        if let Some(Value::Number(temp)) = event.data.get("value") {
            temp_sum += temp;
            count += 1;
        }
    }
    
    if count > 0 {
        println!();
        println!("   Temperature statistics (valid readings only):");
        println!("   - Average: {:.2}°C", temp_sum / count as f64);
        println!("   - Samples: {}", count);
    }
    
    println!();
    Ok(())
}

// Helper function to create test events
fn create_event(timestamp: u64, name: &str) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("name".to_string(), Value::String(name.to_string()));
    data.insert("timestamp".to_string(), Value::Integer(timestamp as i64));
    
    let mut event = StreamEvent::new(name, data, "test");
    event.metadata.timestamp = timestamp;
    event
}

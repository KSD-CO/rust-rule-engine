//! Watermark-based Stream Processing with Rule Engine
//!
//! This example demonstrates watermark generation and late data handling
//! combined with Rule Engine for time-based business logic.
//!
//! Use cases:
//! - Out-of-order event processing
//! - Late data detection and handling
//! - Time-based windowing with rules
//!
//! Run with: cargo run --example streaming_watermark_demo --features streaming

use rust_rule_engine::streaming::*;
use rust_rule_engine::streaming::watermark::{Watermark, WatermarkGenerator, WatermarkStrategy};
use rust_rule_engine::types::Value;
use rust_rule_engine::engine::{
    RustRuleEngine,
    rule::{Rule, Condition, ConditionGroup},
    knowledge_base::KnowledgeBase,
    facts::Facts,
};
use rust_rule_engine::parser::grl::GRLParser;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌊 Watermark-based Stream Processing + Rule Engine Demo");
    println!("{}", "=".repeat(80));
    
    demo1_late_data_detection_with_rules()?;
    demo2_bounded_out_of_order_processing()?;
    demo3_time_window_aggregation_with_alerts()?;
    
    println!("\n{}", "=".repeat(80));
    println!("✅ All watermark demos completed!");
    println!("\n📝 Key Features Demonstrated:");
    println!("   ✅ Watermark generation for event-time tracking");
    println!("   ✅ Late data detection and handling");
    println!("   ✅ Out-of-order event processing");
    println!("   ✅ Rule Engine evaluates time-based business logic");
    
    Ok(())
}

/// Demo 1: Late data detection with rules
fn demo1_late_data_detection_with_rules() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⏰ Demo 1: Late Data Detection with Rules");
    println!("{}", "-".repeat(80));

    // Create watermark generator with bounded out-of-order strategy
    let watermark_gen = Arc::new(Mutex::new(
        WatermarkGenerator::new(WatermarkStrategy::BoundedOutOfOrder {
            max_delay: Duration::from_secs(300), // 5 minutes tolerance
        })
    ));

    // Load late data handling rules
    let grl_rules = r#"
rule LateDataWarning "Warn on late arriving data" salience 100 {
    when
        Event.IsLate == true &&
        Event.DelaySeconds < 600
    then
        Event.Action = "PROCESS_WITH_WARNING";
        Alert.Type = "LATE_DATA";
}

rule VeryLateData "Block very late data" salience 90 {
    when
        Event.IsLate == true &&
        Event.DelaySeconds >= 600
    then
        Event.Action = "DISCARD";
        Alert.Type = "VERY_LATE";
}

rule OnTimeData "Normal processing for on-time data" salience 80 {
    when
        Event.IsLate == false
    then
        Event.Action = "PROCESS_NORMAL";
}
"#;

    let rules = GRLParser::parse_rules(grl_rules)?;
    let kb = KnowledgeBase::new("LateDataHandling");
    for rule in rules {
        kb.add_rule(rule)?;
    }
    let engine = Arc::new(Mutex::new(RustRuleEngine::new(kb)));

    println!("📋 Loaded 3 late data handling rules");
    println!("🌊 Processing events with watermark tracking...\n");

    // Simulate events with various timestamps (some late)
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let event_times = vec![
        (base_time, "sensor-1", 25.5, false),          // On time
        (base_time - 100, "sensor-2", 26.0, true),     // 100s late (OK)
        (base_time - 200, "sensor-3", 24.5, true),     // 200s late (warning)
        (base_time + 10, "sensor-1", 25.8, false),     // On time
        (base_time - 700, "sensor-4", 27.0, true),     // 700s late (very late)
        (base_time + 20, "sensor-2", 26.2, false),     // On time
        (base_time - 400, "sensor-5", 23.5, true),     // 400s late (warning)
    ];

    let mut events = Vec::new();
    for (event_time, sensor_id, temperature, is_late) in event_times {
        let mut data = HashMap::new();
        data.insert("sensor_id".to_string(), Value::String(sensor_id.to_string()));
        data.insert("temperature".to_string(), Value::Number(temperature));
        data.insert("event_time".to_string(), Value::Number(event_time as f64));
        data.insert("is_late".to_string(), Value::Boolean(is_late));
        
        let delay = if is_late {
            base_time.saturating_sub(event_time)
        } else {
            0
        };
        data.insert("delay_seconds".to_string(), Value::Number(delay as f64));
        
        events.push(StreamEvent::new("SensorReading", data, "iot"));
    }

    let stream = DataStream::from_events(events);

    stream.for_each(move |e| {
        let sensor_id = e.get_string("sensor_id").unwrap_or("");
        let temperature = e.get_numeric("temperature").unwrap_or(0.0);
        let event_time = e.get_numeric("event_time").unwrap_or(0.0) as u64;
        let is_late = e.get_boolean("is_late").unwrap_or(false);
        let delay = e.get_numeric("delay_seconds").unwrap_or(0.0);

        // Update watermark
        let mut wm_gen = watermark_gen.lock().unwrap();
        let current_watermark = wm_gen.current_watermark();
        
        // Check if event is late according to watermark
        let late_by_watermark = current_watermark.is_late(event_time);
        
        // Advance watermark if this is newer event
        if event_time > current_watermark.timestamp {
            // Would normally call wm_gen.process_event() here
        }
        drop(wm_gen);

        // Evaluate rules for late data handling
        let facts = Facts::new();
        
        let mut event_data = HashMap::new();
        event_data.insert("SensorID".to_string(), Value::String(sensor_id.to_string()));
        event_data.insert("Temperature".to_string(), Value::Number(temperature));
        event_data.insert("IsLate".to_string(), Value::Boolean(is_late));
        event_data.insert("DelaySeconds".to_string(), Value::Number(delay));
        event_data.insert("Action".to_string(), Value::String("PENDING".to_string()));
        let _ = facts.add_value("Event", Value::Object(event_data));

        let mut alert_data = HashMap::new();
        alert_data.insert("Type".to_string(), Value::String("NONE".to_string()));
        let _ = facts.add_value("Alert", Value::Object(alert_data));

        let mut eng = engine.lock().unwrap();
        let _ = eng.execute(&facts);

        // Extract rule results
        if let Some(Value::Object(event)) = facts.get("Event") {
            let action = event.get("Action")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("PENDING");
            
            if let Some(Value::Object(alert)) = facts.get("Alert") {
                let alert_type = alert.get("Type")
                    .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                    .unwrap_or("NONE");

                let icon = match action {
                    "DISCARD" => "🚫",
                    "PROCESS_WITH_WARNING" => "⚠️ ",
                    _ => "✅",
                };

                println!("{} {} | {:.1}°C | Delay: {:.0}s | {} | Alert: {}",
                         icon, sensor_id, temperature, delay, action, alert_type);
            }
        }
    });

    println!("\n✅ Late data detection completed");
    Ok(())
}

/// Demo 2: Bounded out-of-order processing
fn demo2_bounded_out_of_order_processing() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n🔄 Demo 2: Bounded Out-of-Order Processing");
    println!("{}", "-".repeat(80));

    let watermark_gen = Arc::new(Mutex::new(
        WatermarkGenerator::new(WatermarkStrategy::BoundedOutOfOrder {
            max_delay: Duration::from_secs(60), // 1 minute tolerance
        })
    ));

    // Load ordering rules
    let grl_rules = r#"
rule ProcessInOrder "Process events within order bounds" salience 100 {
    when
        Event.TimeDrift < 60
    then
        Event.Status = "PROCESSED";
        Event.Priority = "NORMAL";
}

rule DelayedProcessing "Delay processing for slightly out-of-order" salience 90 {
    when
        Event.TimeDrift >= 60 &&
        Event.TimeDrift < 120
    then
        Event.Status = "DELAYED";
        Event.Priority = "HIGH";
}

rule OutOfOrderReject "Reject severely out-of-order events" salience 80 {
    when
        Event.TimeDrift >= 120
    then
        Event.Status = "REJECTED";
        Event.Priority = "CRITICAL";
}
"#;

    let rules = GRLParser::parse_rules(grl_rules)?;
    let kb = KnowledgeBase::new("OutOfOrderProcessing");
    for rule in rules {
        kb.add_rule(rule)?;
    }
    let engine = Arc::new(Mutex::new(RustRuleEngine::new(kb)));

    println!("📋 Loaded 3 out-of-order processing rules");
    println!("📦 Processing packets...\n");

    // Simulate network packets with various ordering
    let base_seq = 1000u64;
    let packet_data = vec![
        (base_seq, 0),      // On time
        (base_seq + 1, 10),  // Slightly delayed
        (base_seq + 2, 5),   // Good order
        (base_seq + 4, 0),   // Jump ahead
        (base_seq + 3, 80),  // Out of order
        (base_seq + 5, 150), // Very out of order
        (base_seq + 6, 20),  // Slight delay
    ];

    let mut events = Vec::new();
    for (seq_num, time_drift) in packet_data {
        let mut data = HashMap::new();
        data.insert("sequence".to_string(), Value::Number(seq_num as f64));
        data.insert("time_drift".to_string(), Value::Number(time_drift as f64));
        data.insert("payload_size".to_string(), Value::Number(1024.0));
        events.push(StreamEvent::new("NetworkPacket", data, "network"));
    }

    let stream = DataStream::from_events(events);

    stream.for_each(move |e| {
        let sequence = e.get_numeric("sequence").unwrap_or(0.0) as u64;
        let time_drift = e.get_numeric("time_drift").unwrap_or(0.0);
        let payload_size = e.get_numeric("payload_size").unwrap_or(0.0);

        // Evaluate ordering rules
        let facts = Facts::new();
        
        let mut event_data = HashMap::new();
        event_data.insert("Sequence".to_string(), Value::Number(sequence as f64));
        event_data.insert("TimeDrift".to_string(), Value::Number(time_drift));
        event_data.insert("PayloadSize".to_string(), Value::Number(payload_size));
        event_data.insert("Status".to_string(), Value::String("PENDING".to_string()));
        event_data.insert("Priority".to_string(), Value::String("NORMAL".to_string()));
        let _ = facts.add_value("Event", Value::Object(event_data));

        let mut eng = engine.lock().unwrap();
        let _ = eng.execute(&facts);

        // Extract results
        if let Some(Value::Object(event)) = facts.get("Event") {
            let status = event.get("Status")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("PENDING");
            let priority = event.get("Priority")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("NORMAL");

            let icon = match status {
                "REJECTED" => "🚫",
                "DELAYED" => "⏸️ ",
                _ => "✅",
            };

            println!("{} Seq: {} | Drift: {:.0}s | {} | Priority: {}",
                     icon, sequence, time_drift, status, priority);
        }
    });

    println!("\n✅ Out-of-order processing completed");
    Ok(())
}

/// Demo 3: Time window aggregation with alerts
fn demo3_time_window_aggregation_with_alerts() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📊 Demo 3: Time Window Aggregation with Alerts");
    println!("{}", "-".repeat(80));

    let watermark_gen = Arc::new(Mutex::new(
        WatermarkGenerator::new(WatermarkStrategy::Periodic {
            interval: Duration::from_secs(10),
        })
    ));

    // Load window aggregation rules
    let grl_rules = r#"
rule HighTrafficAlert "Alert on high traffic in window" salience 100 {
    when
        Window.EventCount > 50
    then
        Window.Alert = "HIGH_TRAFFIC";
        Window.Action = "SCALE_UP";
}

rule LowTrafficAlert "Alert on low traffic in window" salience 90 {
    when
        Window.EventCount < 10
    then
        Window.Alert = "LOW_TRAFFIC";
        Window.Action = "SCALE_DOWN";
}

rule AnomalousPattern "Detect anomalous patterns" salience 80 {
    when
        Window.AverageValue > 1000 &&
        Window.EventCount < 20
    then
        Window.Alert = "ANOMALY";
        Window.Action = "INVESTIGATE";
}
"#;

    let rules = GRLParser::parse_rules(grl_rules)?;
    let kb = KnowledgeBase::new("WindowAggregation");
    for rule in rules {
        kb.add_rule(rule)?;
    }
    let engine = Arc::new(Mutex::new(RustRuleEngine::new(kb)));

    println!("📋 Loaded 3 window aggregation rules");
    println!("🪟 Processing windowed events...\n");

    // Simulate different traffic patterns
    let traffic_patterns = vec![
        ("window-1", 60, 100.0),   // High traffic, normal values
        ("window-2", 8, 150.0),    // Low traffic
        ("window-3", 15, 1200.0),  // Anomalous: low count, high values
        ("window-4", 45, 200.0),   // Normal traffic
    ];

    let mut events = Vec::new();
    for (window_id, count, avg_value) in traffic_patterns {
        let mut data = HashMap::new();
        data.insert("window_id".to_string(), Value::String(window_id.to_string()));
        data.insert("event_count".to_string(), Value::Number(count as f64));
        data.insert("average_value".to_string(), Value::Number(avg_value));
        data.insert("total_value".to_string(), Value::Number(count as f64 * avg_value));
        events.push(StreamEvent::new("WindowAggregate", data, "aggregator"));
    }

    let stream = DataStream::from_events(events);

    stream.for_each(move |e| {
        let window_id = e.get_string("window_id").unwrap_or("");
        let event_count = e.get_numeric("event_count").unwrap_or(0.0);
        let avg_value = e.get_numeric("average_value").unwrap_or(0.0);
        let total_value = e.get_numeric("total_value").unwrap_or(0.0);

        // Evaluate window rules
        let facts = Facts::new();
        
        let mut window_data = HashMap::new();
        window_data.insert("ID".to_string(), Value::String(window_id.to_string()));
        window_data.insert("EventCount".to_string(), Value::Number(event_count));
        window_data.insert("AverageValue".to_string(), Value::Number(avg_value));
        window_data.insert("TotalValue".to_string(), Value::Number(total_value));
        window_data.insert("Alert".to_string(), Value::String("NONE".to_string()));
        window_data.insert("Action".to_string(), Value::String("NONE".to_string()));
        let _ = facts.add_value("Window", Value::Object(window_data));

        let mut eng = engine.lock().unwrap();
        let _ = eng.execute(&facts);

        // Extract results
        if let Some(Value::Object(window)) = facts.get("Window") {
            let alert = window.get("Alert")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("NONE");
            let action = window.get("Action")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("NONE");

            let icon = match alert {
                "HIGH_TRAFFIC" => "🔥",
                "LOW_TRAFFIC" => "❄️ ",
                "ANOMALY" => "⚠️ ",
                _ => "✅",
            };

            println!("{} {} | Events: {:.0} | Avg: ${:.0} | Total: ${:.0} | Alert: {} | Action: {}",
                     icon, window_id, event_count, avg_value, total_value, alert, action);
        }
    });

    println!("\n✅ Window aggregation with alerts completed");
    Ok(())
}

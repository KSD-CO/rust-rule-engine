//! Real-Time IoT Sensor Monitoring with Stream Operators
//!
//! This example demonstrates using stream operators for real-time
//! IoT sensor data processing and anomaly detection.
//!
//! Run with: cargo run --example iot_monitoring_demo --features streaming

use rust_rule_engine::streaming::*;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    println!("🏭 Real-Time IoT Sensor Monitoring Demo");
    println!("{}", "=".repeat(70));

    // Simulate sensor data from multiple devices
    let sensor_data = generate_sensor_data();
    println!("\n📡 Generated {} sensor readings from 5 devices", sensor_data.len());

    // Use Case 1: Detect high temperature alerts
    detect_high_temperature(&sensor_data);

    // Use Case 2: Monitor average metrics per device
    monitor_device_metrics(&sensor_data);

    // Use Case 3: Detect anomalies using windowed analysis
    detect_anomalies_windowed(&sensor_data);

    // Use Case 4: Calculate moving averages
    calculate_moving_averages(&sensor_data);

    // Use Case 5: Multi-condition alerts
    multi_condition_alerts(&sensor_data);

    println!("\n\n");
    println!("{}", "=".repeat(70));
    println!("✅ IoT monitoring demo completed!");
}

fn generate_sensor_data() -> Vec<StreamEvent> {
    let mut events = Vec::new();
    let device_ids = vec!["sensor-001", "sensor-002", "sensor-003", "sensor-004", "sensor-005"];
    let locations = vec!["Factory-A", "Factory-A", "Factory-B", "Factory-B", "Warehouse"];
    
    let base_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;

    // Generate 100 readings over simulated 5 minutes
    for i in 0..100 {
        let device_idx = i % device_ids.len();
        let mut data = HashMap::new();

        // Base temperature with some variation
        let base_temp = 25.0 + (device_idx as f64 * 5.0);
        let variation = ((i as f64 * 0.5).sin() * 10.0);
        let temperature = base_temp + variation;
        
        // Inject some anomalies (high temp)
        let temp = if i % 23 == 0 {
            temperature + 30.0 // Anomaly!
        } else {
            temperature
        };

        data.insert("device_id".to_string(), Value::String(device_ids[device_idx].to_string()));
        data.insert("location".to_string(), Value::String(locations[device_idx].to_string()));
        data.insert("temperature".to_string(), Value::Number(temp));
        data.insert("humidity".to_string(), Value::Number(60.0 + (i as f64 % 20.0)));
        data.insert("pressure".to_string(), Value::Number(1013.0 + (i as f64 % 10.0)));
        data.insert("vibration".to_string(), Value::Number(0.5 + (i as f64 % 5.0) * 0.1));

        // Timestamp increases by 3 seconds per reading
        let timestamp = base_time + (i as u64 * 3000);

        events.push(StreamEvent::with_timestamp(
            "SensorReading",
            data,
            format!("iot-gateway-{}", device_idx % 2),
            timestamp,
        ));
    }

    events
}

fn detect_high_temperature(events: &[StreamEvent]) {
    println!("\n🔥 Use Case 1: High Temperature Detection");
    println!("{}", "-".repeat(70));

    let stream = DataStream::from_events(events.to_vec());

    let alerts = stream
        .filter(|e| {
            // Alert if temperature exceeds 50°C
            e.get_numeric("temperature").unwrap_or(0.0) > 50.0
        })
        .map(|mut e| {
            // Add alert metadata
            e.add_tag("alert_type", "HIGH_TEMPERATURE");
            e.add_tag("severity", "CRITICAL");
            e
        })
        .collect();

    println!("   🚨 High temperature alerts: {}", alerts.len());
    
    for alert in alerts.iter().take(5) {
        let device = alert.get_string("device_id").unwrap_or("unknown");
        let temp = alert.get_numeric("temperature").unwrap_or(0.0);
        let location = alert.get_string("location").unwrap_or("unknown");
        println!("   - Device: {} at {} → {:.1}°C", device, location, temp);
    }

    if alerts.len() > 5 {
        println!("   ... and {} more alerts", alerts.len() - 5);
    }
}

fn monitor_device_metrics(events: &[StreamEvent]) {
    println!("\n📊 Use Case 2: Device Metrics Monitoring");
    println!("{}", "-".repeat(70));

    let stream = DataStream::from_events(events.to_vec());

    // Calculate average metrics per device
    let device_stats = stream
        .key_by(|e| e.get_string("device_id").unwrap_or("unknown").to_string())
        .aggregate(CustomAggregator::new(|events: &[StreamEvent]| {
            let temps: Vec<f64> = events
                .iter()
                .filter_map(|e| e.get_numeric("temperature"))
                .collect();
            
            let humidities: Vec<f64> = events
                .iter()
                .filter_map(|e| e.get_numeric("humidity"))
                .collect();

            if temps.is_empty() {
                return AggregateResult::None;
            }

            let avg_temp = temps.iter().sum::<f64>() / temps.len() as f64;
            let avg_humidity = humidities.iter().sum::<f64>() / humidities.len() as f64;
            let max_temp = temps.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            let mut result = HashMap::new();
            result.insert("avg_temp".to_string(), Value::Number(avg_temp));
            result.insert("avg_humidity".to_string(), Value::Number(avg_humidity));
            result.insert("max_temp".to_string(), Value::Number(max_temp));
            result.insert("readings".to_string(), Value::Number(temps.len() as f64));

            AggregateResult::Map(result)
        }));

    println!("   Device health metrics:");
    for (device, stats) in device_stats {
        if let Some(map) = stats.as_map() {
            let avg_temp = map.get("avg_temp").and_then(|v| match v {
                Value::Number(n) => Some(*n),
                _ => None,
            }).unwrap_or(0.0);
            
            let max_temp = map.get("max_temp").and_then(|v| match v {
                Value::Number(n) => Some(*n),
                _ => None,
            }).unwrap_or(0.0);
            
            let readings = map.get("readings").and_then(|v| match v {
                Value::Number(n) => Some(*n as usize),
                _ => None,
            }).unwrap_or(0);

            println!("   - {}: {} readings, avg {:.1}°C, max {:.1}°C", 
                     device, readings, avg_temp, max_temp);
        }
    }
}

fn detect_anomalies_windowed(events: &[StreamEvent]) {
    println!("\n🔍 Use Case 3: Windowed Anomaly Detection");
    println!("{}", "-".repeat(70));

    let stream = DataStream::from_events(events.to_vec());

    // Use 30-second windows to detect sudden spikes
    let window_results = stream
        .window(WindowConfig::tumbling(Duration::from_secs(30)))
        .aggregate(CustomAggregator::new(|events: &[StreamEvent]| {
            let temps: Vec<f64> = events
                .iter()
                .filter_map(|e| e.get_numeric("temperature"))
                .collect();

            if temps.is_empty() {
                return AggregateResult::None;
            }

            let avg = temps.iter().sum::<f64>() / temps.len() as f64;
            let max = temps.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
            let anomaly = max > avg + 20.0; // 20°C above average is anomaly

            let mut result = HashMap::new();
            result.insert("avg_temp".to_string(), Value::Number(avg));
            result.insert("max_temp".to_string(), Value::Number(max));
            result.insert("has_anomaly".to_string(), Value::Boolean(anomaly));
            result.insert("count".to_string(), Value::Number(temps.len() as f64));

            AggregateResult::Map(result)
        }));

    let anomaly_windows = window_results.iter().filter(|r| {
        r.as_map()
            .and_then(|m| m.get("has_anomaly"))
            .and_then(|v| match v {
                Value::Boolean(b) => Some(*b),
                _ => None,
            })
            .unwrap_or(false)
    }).count();

    println!("   Total windows analyzed: {}", window_results.len());
    println!("   Windows with anomalies: {}", anomaly_windows);

    for (idx, result) in window_results.iter().enumerate() {
        if let Some(map) = result.as_map() {
            if let Some(Value::Boolean(true)) = map.get("has_anomaly") {
                let avg = map.get("avg_temp").and_then(|v| match v {
                    Value::Number(n) => Some(*n),
                    _ => None,
                }).unwrap_or(0.0);
                
                let max = map.get("max_temp").and_then(|v| match v {
                    Value::Number(n) => Some(*n),
                    _ => None,
                }).unwrap_or(0.0);

                println!("   ⚠️  Window {}: avg {:.1}°C, spike {:.1}°C (+{:.1}°C)", 
                         idx + 1, avg, max, max - avg);
            }
        }
    }
}

fn calculate_moving_averages(events: &[StreamEvent]) {
    println!("\n📈 Use Case 4: Moving Average Calculation");
    println!("{}", "-".repeat(70));

    let stream = DataStream::from_events(events.to_vec());

    // Group by device and calculate metrics
    let device_trends = stream
        .key_by(|e| e.get_string("device_id").unwrap_or("unknown").to_string())
        .window(WindowConfig::sliding(Duration::from_secs(60)))
        .aggregate(Average::new("temperature"));

    println!("   Temperature trends (60s windows):");
    for (device, windows) in device_trends.iter().take(3) {
        print!("   - {}: ", device);
        
        let averages: Vec<String> = windows
            .iter()
            .filter_map(|r| r.as_number())
            .take(3)
            .map(|v| format!("{:.1}°C", v))
            .collect();
        
        println!("{}", averages.join(" → "));
    }
}

fn multi_condition_alerts(events: &[StreamEvent]) {
    println!("\n⚡ Use Case 5: Multi-Condition Alerts");
    println!("{}", "-".repeat(70));

    let stream = DataStream::from_events(events.to_vec());

    // Complex condition: high temp AND high vibration
    let critical_alerts = stream
        .filter(|e| {
            let temp = e.get_numeric("temperature").unwrap_or(0.0);
            let vibration = e.get_numeric("vibration").unwrap_or(0.0);
            let humidity = e.get_numeric("humidity").unwrap_or(0.0);
            
            // Critical if: temp > 45°C AND (vibration > 0.8 OR humidity > 75%)
            temp > 45.0 && (vibration > 0.8 || humidity > 75.0)
        })
        .map(|mut e| {
            e.add_tag("alert_level", "CRITICAL");
            e.add_tag("requires_action", "IMMEDIATE");
            e
        })
        .group_by(|e| e.get_string("location").unwrap_or("unknown").to_string());

    let location_alerts = critical_alerts.count();

    println!("   Critical multi-condition alerts by location:");
    for (location, count) in location_alerts {
        if count > 0 {
            println!("   🚨 {}: {} critical alerts", location, count);
        }
    }

    // Calculate severity distribution
    let stream2 = DataStream::from_events(events.to_vec());
    let severity_analysis = stream2
        .map(|mut e| {
            // Classify severity based on temperature
            let temp = e.get_numeric("temperature").unwrap_or(0.0);
            let severity = if temp > 60.0 {
                "CRITICAL"
            } else if temp > 50.0 {
                "HIGH"
            } else if temp > 40.0 {
                "MEDIUM"
            } else {
                "NORMAL"
            };
            e.data.insert("severity".to_string(), Value::String(severity.to_string()));
            e
        })
        .group_by(|e| e.get_string("severity").unwrap_or("UNKNOWN").to_string())
        .count();

    println!("\n   Overall severity distribution:");
    let mut severities: Vec<_> = severity_analysis.iter().collect();
    severities.sort_by(|a, b| b.1.cmp(a.1));
    
    for (severity, count) in severities {
        let bar = "█".repeat(count / 2);
        println!("   {:10} | {} {}", severity, bar, count);
    }
}

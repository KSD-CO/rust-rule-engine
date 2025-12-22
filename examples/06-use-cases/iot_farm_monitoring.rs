/// IoT Farm Monitoring System - Stream Processing Case Study
///
/// This example demonstrates a complete IoT system for smart farm monitoring
/// that processes real-time sensor data from Kafka streams.
///
/// # System Overview
///
/// The farm has multiple types of sensors:
/// - **Soil Moisture Sensors**: Monitor soil humidity levels
/// - **Temperature Sensors**: Track ambient and soil temperature
/// - **Irrigation Controllers**: Control water flow to different zones
/// - **Weather Stations**: External weather data
///
/// # Use Cases
///
/// 1. **Automatic Irrigation**: Trigger watering based on soil moisture + temperature
/// 2. **Frost Alert**: Detect dangerous temperature drops that could harm crops
/// 3. **Water Efficiency**: Correlate irrigation with actual moisture increase
/// 4. **Anomaly Detection**: Identify sensor malfunctions or unusual patterns
///
/// # Architecture
///
/// ```text
/// Kafka Topics          Stream Joins                  Actions
/// ┌──────────────┐     ┌─────────────┐             ┌──────────────┐
/// │ soil-sensors │────▶│ Moisture +  │────────────▶│ Auto Irrigate│
/// └──────────────┘     │ Temperature │             └──────────────┘
///                      └─────────────┘
/// ┌──────────────┐     ┌─────────────┐             ┌──────────────┐
/// │ temp-sensors │────▶│ Temperature │────────────▶│ Frost Alert  │
/// └──────────────┘     │ Trend       │             └──────────────┘
///                      └─────────────┘
/// ┌──────────────┐     ┌─────────────┐             ┌──────────────┐
/// │ irrigation   │────▶│ Irrigation +│────────────▶│ Efficiency   │
/// │ -events      │     │ Moisture    │             │ Report       │
/// └──────────────┘     └─────────────┘             └──────────────┘
/// ```
///
/// Run with: cargo run --features streaming --example iot_farm_monitoring

#[cfg(feature = "streaming")]
use rust_rule_engine::rete::stream_join_node::{JoinStrategy, JoinType, StreamJoinNode};
#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::event::{EventMetadata, StreamEvent};
#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::join_manager::StreamJoinManager;
#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::join_optimizer::{JoinOptimizer, StreamStats};
#[cfg(feature = "streaming")]
use rust_rule_engine::types::Value;
#[cfg(feature = "streaming")]
use std::collections::HashMap;
#[cfg(feature = "streaming")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "streaming")]
use std::time::Duration;

#[cfg(feature = "streaming")]
#[derive(Debug, Clone)]
struct SoilSensorReading {
    zone_id: String,
    moisture_level: f64, // 0.0 - 100.0%
    timestamp: i64,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Clone)]
struct TemperatureReading {
    zone_id: String,
    temperature: f64, // Celsius
    sensor_type: String, // "soil" or "air"
    timestamp: i64,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Clone)]
struct IrrigationEvent {
    zone_id: String,
    action: String, // "start" or "stop"
    water_volume_ml: i32,
    timestamp: i64,
}

#[cfg(feature = "streaming")]
#[derive(Debug, Clone)]
struct WeatherEvent {
    location: String,
    condition: String, // "sunny", "cloudy", "rainy", "frost"
    temperature: f64,
    timestamp: i64,
}

#[cfg(feature = "streaming")]
fn create_soil_event(zone_id: &str, moisture: f64, timestamp: i64) -> StreamEvent {
    StreamEvent {
        id: format!("soil_{}_{}", zone_id, timestamp),
        event_type: "SoilMoistureReading".to_string(),
        data: vec![
            ("zone_id".to_string(), Value::String(zone_id.to_string())),
            ("moisture_level".to_string(), Value::Number(moisture)),
            ("sensor_id".to_string(), Value::String(format!("SOIL-{}", zone_id))),
        ]
        .into_iter()
        .collect(),
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "soil-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

#[cfg(feature = "streaming")]
fn create_temperature_event(zone_id: &str, temp: f64, sensor_type: &str, timestamp: i64) -> StreamEvent {
    StreamEvent {
        id: format!("temp_{}_{}_{}", zone_id, sensor_type, timestamp),
        event_type: "TemperatureReading".to_string(),
        data: vec![
            ("zone_id".to_string(), Value::String(zone_id.to_string())),
            ("temperature".to_string(), Value::Number(temp)),
            ("sensor_type".to_string(), Value::String(sensor_type.to_string())),
        ]
        .into_iter()
        .collect(),
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "temp-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

#[cfg(feature = "streaming")]
fn create_irrigation_event(zone_id: &str, action: &str, volume: i32, timestamp: i64) -> StreamEvent {
    StreamEvent {
        id: format!("irrigation_{}_{}", zone_id, timestamp),
        event_type: "IrrigationEvent".to_string(),
        data: vec![
            ("zone_id".to_string(), Value::String(zone_id.to_string())),
            ("action".to_string(), Value::String(action.to_string())),
            ("water_volume_ml".to_string(), Value::Integer(volume as i64)),
        ]
        .into_iter()
        .collect(),
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "irrigation-events".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

#[cfg(feature = "streaming")]
fn create_weather_event(condition: &str, temp: f64, timestamp: i64) -> StreamEvent {
    StreamEvent {
        id: format!("weather_{}", timestamp),
        event_type: "WeatherEvent".to_string(),
        data: vec![
            ("location".to_string(), Value::String("farm".to_string())),
            ("condition".to_string(), Value::String(condition.to_string())),
            ("temperature".to_string(), Value::Number(temp)),
        ]
        .into_iter()
        .collect(),
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "weather-station".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

#[cfg(feature = "streaming")]
fn demo_automatic_irrigation() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Use Case 1: Automatic Irrigation Control               ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("Rule: When soil moisture < 30% AND temperature > 25°C, irrigate\n");

    let irrigation_decisions = Arc::new(Mutex::new(Vec::new()));
    let decisions_clone = irrigation_decisions.clone();

    // Join soil moisture with temperature to make irrigation decisions
    let join = StreamJoinNode::new(
        "soil-sensors".to_string(),
        "temp-sensors".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(300), // 5 minute window
        },
        Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
        Box::new(|soil, temp| {
            // Only join recent readings (within 2 minutes)
            let time_diff = (temp.metadata.timestamp as i64 - soil.metadata.timestamp as i64).abs();
            time_diff < 120
        }),
    );

    let mut manager = StreamJoinManager::new();
    manager.register_join(
        "irrigation_control".to_string(),
        join,
        Box::new(move |joined| {
            if let (Some(soil), Some(temp)) = (&joined.left, &joined.right) {
                let zone_id = soil.data.get("zone_id").and_then(|v| v.as_string()).unwrap();
                let moisture = soil.data.get("moisture_level").and_then(|v| {
                    if let Value::Number(n) = v { Some(*n) } else { None }
                }).unwrap_or(0.0);
                let temperature = temp.data.get("temperature").and_then(|v| {
                    if let Value::Number(n) = v { Some(*n) } else { None }
                }).unwrap_or(0.0);

                // Rule: Low moisture + high temperature = irrigate
                if moisture < 30.0 && temperature > 25.0 {
                    let decision = format!(
                        "🚰 IRRIGATE Zone {} - Moisture: {:.1}% (low), Temp: {:.1}°C (high)",
                        zone_id, moisture, temperature
                    );
                    println!("{}", decision);
                    decisions_clone.lock().unwrap().push(decision);
                } else {
                    println!(
                        "✓  Zone {} OK - Moisture: {:.1}%, Temp: {:.1}°C",
                        zone_id, moisture, temperature
                    );
                }
            }
        }),
    );

    // Simulate sensor readings
    println!("📊 Processing sensor readings...\n");

    // Zone A: Low moisture, high temp - needs irrigation
    manager.process_event(create_soil_event("A", 25.0, 1000));
    manager.process_event(create_temperature_event("A", 28.5, "soil", 1030));

    // Zone B: Good moisture, moderate temp - no action needed
    manager.process_event(create_soil_event("B", 55.0, 1050));
    manager.process_event(create_temperature_event("B", 23.0, "soil", 1070));

    // Zone C: Low moisture, low temp - monitor only
    manager.process_event(create_soil_event("C", 28.0, 1100));
    manager.process_event(create_temperature_event("C", 18.0, "soil", 1120));

    // Zone D: Critical - very low moisture, very high temp
    manager.process_event(create_soil_event("D", 15.0, 1150));
    manager.process_event(create_temperature_event("D", 32.0, "soil", 1170));

    let decisions = irrigation_decisions.lock().unwrap();
    println!("\n📈 Summary: {} zones require irrigation", decisions.len());
}

#[cfg(feature = "streaming")]
fn demo_frost_alert() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Use Case 2: Frost Alert System                         ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("Rule: Temperature drops below 2°C = FROST ALERT\n");

    let alerts = Arc::new(Mutex::new(Vec::new()));
    let alerts_clone = alerts.clone();

    // Monitor temperature trends for frost detection
    let join = StreamJoinNode::new(
        "temp-sensors".to_string(),
        "weather-station".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(600), // 10 minute window
        },
        Box::new(|_e| Some("farm".to_string())), // All sensors belong to same farm
        Box::new(|e| e.data.get("location").and_then(|v| v.as_string())),
        Box::new(|sensor, weather| {
            // Correlate local sensor with weather station
            sensor.metadata.timestamp as i64 - weather.metadata.timestamp as i64 < 300
        }),
    );

    let mut manager = StreamJoinManager::new();
    manager.register_join(
        "frost_detection".to_string(),
        join,
        Box::new(move |joined| {
            if let (Some(sensor), Some(weather)) = (&joined.left, &joined.right) {
                let zone_id = sensor.data.get("zone_id").and_then(|v| v.as_string()).unwrap_or_default();
                let sensor_temp = sensor.data.get("temperature").and_then(|v| {
                    if let Value::Number(n) = v { Some(*n) } else { None }
                }).unwrap_or(0.0);
                let weather_temp = weather.data.get("temperature").and_then(|v| {
                    if let Value::Number(n) = v { Some(*n) } else { None }
                }).unwrap_or(0.0);
                let condition = weather.data.get("condition").and_then(|v| v.as_string()).unwrap_or_default();

                // Rule: Frost alert when temperature drops below 2°C
                if sensor_temp < 2.0 || weather_temp < 2.0 || condition == "frost" {
                    let alert = format!(
                        "❄️  FROST ALERT! Zone {} - Sensor: {:.1}°C, Weather: {:.1}°C, Condition: {}",
                        zone_id, sensor_temp, weather_temp, condition
                    );
                    println!("{}", alert);
                    alerts_clone.lock().unwrap().push(alert);
                } else {
                    println!(
                        "✓  Zone {} Safe - Sensor: {:.1}°C, Weather: {:.1}°C",
                        zone_id, sensor_temp, weather_temp
                    );
                }
            }
        }),
    );

    println!("🌡️  Monitoring temperature trends...\n");

    // Normal conditions
    manager.process_event(create_temperature_event("A", 15.0, "air", 2000));
    manager.process_event(create_weather_event("sunny", 16.0, 2010));

    // Temperature dropping
    manager.process_event(create_temperature_event("B", 5.0, "air", 2100));
    manager.process_event(create_weather_event("cloudy", 6.0, 2110));

    // Frost warning!
    manager.process_event(create_temperature_event("C", 1.5, "air", 2200));
    manager.process_event(create_weather_event("frost", 0.5, 2210));

    // Critical frost
    manager.process_event(create_temperature_event("D", -2.0, "air", 2300));
    manager.process_event(create_weather_event("frost", -1.5, 2310));

    let alert_count = alerts.lock().unwrap().len();
    println!("\n⚠️  Total frost alerts: {}", alert_count);
}

#[cfg(feature = "streaming")]
fn demo_irrigation_efficiency() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Use Case 3: Irrigation Efficiency Analysis             ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("Rule: Measure moisture increase after irrigation\n");

    let efficiency_reports = Arc::new(Mutex::new(Vec::new()));
    let reports_clone = efficiency_reports.clone();

    // Join irrigation events with subsequent moisture readings
    let join = StreamJoinNode::new(
        "irrigation-events".to_string(),
        "soil-sensors".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(1800), // 30 minute window
        },
        Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
        Box::new(|irrigation, moisture| {
            // Moisture reading must be AFTER irrigation
            let action = irrigation.data.get("action").and_then(|v| v.as_string()).unwrap_or_default();
            let time_diff = moisture.metadata.timestamp as i64 - irrigation.metadata.timestamp as i64;
            action == "stop" && time_diff > 0 && time_diff < 1800 // Within 30 min after irrigation
        }),
    );

    let mut manager = StreamJoinManager::new();
    manager.register_join(
        "efficiency_tracking".to_string(),
        join,
        Box::new(move |joined| {
            if let (Some(irrigation), Some(moisture)) = (&joined.left, &joined.right) {
                let zone_id = irrigation.data.get("zone_id").and_then(|v| v.as_string()).unwrap();
                let water_volume = irrigation.data.get("water_volume_ml").and_then(|v| {
                    if let Value::Integer(i) = v { Some(*i) } else { None }
                }).unwrap_or(0);
                let moisture_after = moisture.data.get("moisture_level").and_then(|v| {
                    if let Value::Number(n) = v { Some(*n) } else { None }
                }).unwrap_or(0.0);

                let time_diff = (moisture.metadata.timestamp - irrigation.metadata.timestamp) as i64;

                let report = format!(
                    "💧 Zone {}: {}ml water → {:.1}% moisture after {}s (efficiency: {:.2}%/L)",
                    zone_id,
                    water_volume,
                    moisture_after,
                    time_diff,
                    (moisture_after / (water_volume as f64 / 1000.0))
                );
                println!("{}", report);
                reports_clone.lock().unwrap().push((zone_id, water_volume, moisture_after));
            }
        }),
    );

    println!("💦 Tracking irrigation efficiency...\n");

    // Zone A: Efficient irrigation
    manager.process_event(create_irrigation_event("A", "start", 0, 3000));
    manager.process_event(create_irrigation_event("A", "stop", 5000, 3300));
    manager.process_event(create_soil_event("A", 65.0, 3600)); // Good increase

    // Zone B: Less efficient
    manager.process_event(create_irrigation_event("B", "start", 0, 3100));
    manager.process_event(create_irrigation_event("B", "stop", 8000, 3400));
    manager.process_event(create_soil_event("B", 55.0, 3700)); // Moderate increase

    // Zone C: Very efficient
    manager.process_event(create_irrigation_event("C", "start", 0, 3200));
    manager.process_event(create_irrigation_event("C", "stop", 3000, 3500));
    manager.process_event(create_soil_event("C", 70.0, 3800)); // Excellent increase

    let reports = efficiency_reports.lock().unwrap();
    println!("\n📊 Efficiency Summary:");
    let avg_efficiency: f64 = reports.iter()
        .map(|(_, water, moisture)| moisture / (*water as f64 / 1000.0))
        .sum::<f64>() / reports.len() as f64;
    println!("   Average efficiency: {:.2}% moisture per liter", avg_efficiency);
}

#[cfg(feature = "streaming")]
fn demo_anomaly_detection() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Use Case 4: Sensor Anomaly Detection                   ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("Rule: Detect sensor malfunctions or unusual readings\n");

    let anomalies = Arc::new(Mutex::new(Vec::new()));
    let anomalies_clone = anomalies.clone();

    // Left outer join to detect missing sensor correlations
    let join = StreamJoinNode::new(
        "soil-sensors".to_string(),
        "temp-sensors".to_string(),
        JoinType::LeftOuter,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(120), // 2 minute window
        },
        Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
        Box::new(|soil, temp| {
            let time_diff = (temp.metadata.timestamp as i64 - soil.metadata.timestamp as i64).abs();
            time_diff < 120
        }),
    );

    let mut manager = StreamJoinManager::new();
    manager.register_join(
        "anomaly_detection".to_string(),
        join,
        Box::new(move |joined| {
            if let Some(soil) = &joined.left {
                let zone_id = soil.data.get("zone_id").and_then(|v| v.as_string()).unwrap();
                let moisture = soil.data.get("moisture_level").and_then(|v| {
                    if let Value::Number(n) = v { Some(*n) } else { None }
                }).unwrap_or(0.0);

                if joined.right.is_none() {
                    // No temperature reading for this zone - sensor malfunction?
                    let anomaly = format!(
                        "⚠️  ANOMALY: Zone {} has soil reading ({:.1}%) but NO temperature reading!",
                        zone_id, moisture
                    );
                    println!("{}", anomaly);
                    anomalies_clone.lock().unwrap().push(anomaly);
                } else if let Some(temp) = &joined.right {
                    let temperature = temp.data.get("temperature").and_then(|v| {
                        if let Value::Number(n) = v { Some(*n) } else { None }
                    }).unwrap_or(0.0);

                    // Check for impossible readings
                    if moisture > 100.0 || moisture < 0.0 {
                        let anomaly = format!(
                            "🔴 SENSOR ERROR: Zone {} moisture reading out of range: {:.1}%",
                            zone_id, moisture
                        );
                        println!("{}", anomaly);
                        anomalies_clone.lock().unwrap().push(anomaly);
                    } else if temperature > 60.0 || temperature < -20.0 {
                        let anomaly = format!(
                            "🔴 SENSOR ERROR: Zone {} temperature reading out of range: {:.1}°C",
                            zone_id, temperature
                        );
                        println!("{}", anomaly);
                        anomalies_clone.lock().unwrap().push(anomaly);
                    } else {
                        println!("✓  Zone {} sensors working normally", zone_id);
                    }
                }
            }
        }),
    );

    println!("🔍 Monitoring sensor health...\n");

    // Normal operation
    manager.process_event(create_soil_event("A", 45.0, 4000));
    manager.process_event(create_temperature_event("A", 22.0, "soil", 4030));

    // Missing temperature sensor
    manager.process_event(create_soil_event("B", 50.0, 4100));
    // No temperature reading for Zone B!

    // Impossible moisture reading
    manager.process_event(create_soil_event("C", 150.0, 4200)); // > 100%!
    manager.process_event(create_temperature_event("C", 23.0, "soil", 4230));

    // Impossible temperature reading
    manager.process_event(create_soil_event("D", 48.0, 4300));
    manager.process_event(create_temperature_event("D", 75.0, "soil", 4330)); // Too high!

    let anomaly_count = anomalies.lock().unwrap().len();
    println!("\n🚨 Total anomalies detected: {}", anomaly_count);
}

#[cfg(feature = "streaming")]
fn demo_system_optimization() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  System Optimization Analysis                            ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let mut optimizer = JoinOptimizer::new();

    // Register stream statistics based on typical farm sensor data
    optimizer.register_stream_stats(StreamStats {
        stream_name: "soil-sensors".to_string(),
        estimated_event_rate: 0.5,  // 1 reading per 2 seconds per zone
        estimated_cardinality: 20,   // 20 farm zones
        average_event_size: 150,     // bytes
    });

    optimizer.register_stream_stats(StreamStats {
        stream_name: "temp-sensors".to_string(),
        estimated_event_rate: 1.0,   // 1 reading per second per zone
        estimated_cardinality: 20,
        average_event_size: 120,
    });

    optimizer.register_stream_stats(StreamStats {
        stream_name: "irrigation-events".to_string(),
        estimated_event_rate: 0.05,  // Sporadic events
        estimated_cardinality: 20,
        average_event_size: 100,
    });

    optimizer.register_stream_stats(StreamStats {
        stream_name: "weather-station".to_string(),
        estimated_event_rate: 0.017, // Every ~60 seconds
        estimated_cardinality: 1,     // Single weather station
        average_event_size: 200,
    });

    // Optimize irrigation control join
    let irrigation_plan = optimizer.optimize_join(
        "soil-sensors",
        "temp-sensors",
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(300),
        },
    );

    println!("🔧 Irrigation Control Join Optimization:");
    println!("   Estimated Cost: {:.2}", irrigation_plan.estimated_cost);
    println!("   Strategy: {}", irrigation_plan.explanation);
    println!("   Optimizations: {} applied", irrigation_plan.optimizations.len());

    // Memory estimation
    let memory = optimizer.estimate_memory_usage(
        "soil-sensors",
        "temp-sensors",
        Duration::from_secs(300),
    );
    println!("\n💾 Memory Usage:");
    println!("   5-minute window: {} KB", memory / 1024);

    // Window recommendation
    let recommended_window = optimizer.recommend_window_size(
        "soil-sensors",
        "temp-sensors",
        1_000_000, // 1MB memory limit
    );
    println!("\n⏱️  Recommended Configuration:");
    println!("   Window size: {} seconds", recommended_window.as_secs());
    println!("   For 1MB memory budget");
}

#[cfg(feature = "streaming")]
fn print_kafka_integration_guide() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║  Kafka Integration Guide                                 ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("📦 To integrate with real Kafka streams, add to Cargo.toml:");
    println!("   rdkafka = \"0.36\"");
    println!("   tokio = {{ version = \"1\", features = [\"full\"] }}");
    println!();
    println!("🔌 Example Kafka consumer setup:");
    println!();
    println!("```rust");
    println!("use rdkafka::consumer::{{Consumer, StreamConsumer}};");
    println!("use rdkafka::config::ClientConfig;");
    println!("use rdkafka::Message;");
    println!();
    println!("async fn consume_sensor_data(manager: Arc<Mutex<StreamJoinManager>>) {{");
    println!("    let consumer: StreamConsumer = ClientConfig::new()");
    println!("        .set(\"bootstrap.servers\", \"localhost:9092\")");
    println!("        .set(\"group.id\", \"farm-monitoring\")");
    println!("        .create()");
    println!("        .expect(\"Consumer creation failed\");");
    println!();
    println!("    consumer.subscribe(&[\"soil-sensors\", \"temp-sensors\",");
    println!("                         \"irrigation-events\", \"weather-station\"])");
    println!("        .expect(\"Subscription failed\");");
    println!();
    println!("    loop {{");
    println!("        match consumer.recv().await {{");
    println!("            Ok(message) => {{");
    println!("                let payload = message.payload().unwrap();");
    println!("                let event: StreamEvent = serde_json::from_slice(payload).unwrap();");
    println!("                manager.lock().unwrap().process_event(event);");
    println!("            }}");
    println!("            Err(e) => eprintln!(\"Kafka error: {{}}\", e),");
    println!("        }}");
    println!("    }}");
    println!("}}");
    println!("```");
    println!();
    println!("📝 Example GRL rule with Kafka (future syntax):");
    println!();
    println!("```grl");
    println!("rule \"SmartIrrigation\" {{");
    println!("    when");
    println!("        soil: SoilReading from kafka(\"soil-sensors\", \"localhost:9092\")");
    println!("            over window(5 min, sliding) &&");
    println!("        temp: TempReading from kafka(\"temp-sensors\", \"localhost:9092\")");
    println!("            over window(5 min, sliding) &&");
    println!("        soil.zone_id == temp.zone_id &&");
    println!("        soil.moisture_level < 30 &&");
    println!("        temp.temperature > 25");
    println!("    then");
    println!("        emit to kafka(\"irrigation-commands\") {{");
    println!("            zone_id: soil.zone_id,");
    println!("            action: \"start_irrigation\",");
    println!("            priority: \"high\"");
    println!("        }};");
    println!("}}");
    println!("```");
}

#[cfg(feature = "streaming")]
fn main() {
    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                                                                ║");
    println!("║     🌾 IoT Farm Monitoring System - Stream Processing 🌾      ║");
    println!("║                                                                ║");
    println!("║  Real-time sensor data processing with Stream Joins           ║");
    println!("║  Demonstrates: Kafka integration, CEP, Multi-stream joins     ║");
    println!("║                                                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝");

    // Run all use cases
    demo_automatic_irrigation();
    demo_frost_alert();
    demo_irrigation_efficiency();
    demo_anomaly_detection();
    demo_system_optimization();
    print_kafka_integration_guide();

    println!("\n");
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                   Demo Completed Successfully!                 ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("📊 Summary of Demonstrated Features:");
    println!("   ✅ Stream-to-stream joins (soil + temperature)");
    println!("   ✅ Time-based windows (5-30 minute windows)");
    println!("   ✅ Inner and outer joins (correlation + anomaly detection)");
    println!("   ✅ Complex join conditions (time-based, value-based)");
    println!("   ✅ Real-time decision making (irrigation control)");
    println!("   ✅ Anomaly detection (sensor health monitoring)");
    println!("   ✅ Efficiency tracking (irrigation effectiveness)");
    println!("   ✅ System optimization (memory, window sizing)");
    println!();
    println!("🚀 Next Steps:");
    println!("   1. Connect to real Kafka cluster");
    println!("   2. Deploy to farm gateway (Raspberry Pi, edge device)");
    println!("   3. Add persistence layer for historical analysis");
    println!("   4. Implement alert notifications (SMS, email, dashboard)");
    println!("   5. Add machine learning predictions for proactive irrigation");
    println!();
}

#[cfg(not(feature = "streaming"))]
fn main() {
    println!("This example requires the 'streaming' feature to be enabled.");
    println!("Run with: cargo run --features streaming --example iot_farm_monitoring");
}

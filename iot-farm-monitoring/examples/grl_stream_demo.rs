/// GRL + Stream Integration Demo
///
/// This example demonstrates the COMPLETE integration of:
/// - Stream Processing (joins, windows, watermarks)
/// - GRL Rules (business logic from .grl files)
///
/// Architecture:
/// ```
/// Sensor Events → Stream Joins → TypedFacts → GRL Rules → Actions
/// ```

use iot_farm_monitoring::grl_stream_adapter::{GrlStreamProcessor, StreamToReteAdapter};
use rust_rule_engine::rete::{JoinStrategy, JoinType, StreamJoinNode};
use rust_rule_engine::streaming::StreamEvent;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║         🌊 GRL + STREAM INTEGRATION - Complete Demo 🌊              ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    demo_greenhouse_climate_control();
    demo_fish_pond_monitoring();
    demo_farm_health_assessment();

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║              ✅ ALL GRL + STREAM DEMOS COMPLETED ✅                  ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");
}

/// Demo 1: Greenhouse Climate Control with Stream Joins + GRL Rules
fn demo_greenhouse_climate_control() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🥬 DEMO 1: Greenhouse Climate Control (Stream Join + GRL)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Create processor
    let mut processor = GrlStreamProcessor::new();

    // Load GRL rules
    match processor.load_rules("grl_rules/vegetable_monitoring.grl") {
        Ok(count) => println!("✅ Loaded {} GRL rules\n", count),
        Err(e) => {
            println!("❌ Failed to load rules: {}", e);
            return;
        }
    }

    // Create stream join: temperature ⨝ humidity
    let join = StreamJoinNode::new(
        "air-temperature".to_string(),
        "humidity-sensors".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(300), // 5 minutes
        },
        Box::new(|left| {
            left.data
                .get("zone_id")
                .and_then(|v| v.as_string())
        }),
        Box::new(|right| {
            right.data
                .get("zone_id")
                .and_then(|v| v.as_string())
        }),
        Box::new(|_left, _right| true), // Accept all matches with same zone_id
    );

    let adapter = StreamToReteAdapter::new("Greenhouse".to_string());
    processor.register_join("greenhouse-climate".to_string(), join, adapter);

    // Scenario: High temperature + low humidity
    println!("📊 Scenario: Temperature 32°C, Humidity 55%");
    println!("   Expected: Cooling + Misting activated (GRL Rule 1)\n");

    // Send temperature event
    let temp_event = create_temperature_event("GH-1", 32.0, 1000);
    processor.process_event("air-temperature", temp_event);

    // Send humidity event (this will trigger the join)
    let humidity_event = create_humidity_event("GH-1", 55.0, 1050);
    let fired = processor.process_event("humidity-sensors", humidity_event);

    println!("🔥 Rules fired: {} rules", fired.len());
    if !fired.is_empty() {
        println!("   Fired: {:?}", fired);
    }
    println!("📊 Stats: {:?}\n", processor.stats());

    println!("✅ Demo 1 completed\n");
}

/// Demo 2: Fish Pond Monitoring with Stream Joins + GRL Rules
fn demo_fish_pond_monitoring() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🐟 DEMO 2: Fish Pond Monitoring (Stream Join + GRL)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut processor = GrlStreamProcessor::new();

    match processor.load_rules("grl_rules/aquaculture_monitoring.grl") {
        Ok(count) => println!("✅ Loaded {} GRL rules\n", count),
        Err(e) => {
            println!("❌ Failed to load rules: {}", e);
            return;
        }
    }

    // Create stream join: DO ⨝ temperature
    let join = StreamJoinNode::new(
        "dissolved-oxygen".to_string(),
        "water-temperature".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(600), // 10 minutes
        },
        Box::new(|left| {
            left.data
                .get("pond_id")
                .and_then(|v| v.as_string())
        }),
        Box::new(|right| {
            right.data
                .get("pond_id")
                .and_then(|v| v.as_string())
        }),
        Box::new(|_left, _right| true), // Accept all matches with same pond_id
    );

    let adapter = StreamToReteAdapter::new("FishPond".to_string());
    processor.register_join("fish-pond-monitoring".to_string(), join, adapter);

    // Scenario: Critical low DO + high temperature
    println!("📊 Scenario: DO 3.5 mg/L, Temperature 30°C");
    println!("   Expected: CRITICAL alert + Emergency aeration (GRL Rule 1)\n");

    let do_event = create_do_event("POND-1", 3.5, 2000);
    processor.process_event("dissolved-oxygen", do_event);

    let temp_event = create_water_temp_event("POND-1", 30.0, 2050);
    let fired = processor.process_event("water-temperature", temp_event);

    println!("🔥 Rules fired: {} rules", fired.len());
    if !fired.is_empty() {
        println!("   Fired: {:?}", fired);
    }
    println!("📊 Stats: {:?}\n", processor.stats());

    println!("✅ Demo 2 completed\n");
}

/// Demo 3: Farm Health Assessment (single-stream GRL)
fn demo_farm_health_assessment() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("♻️  DEMO 3: Farm Health Assessment (GRL Rules)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mut processor = GrlStreamProcessor::new();

    match processor.load_rules("grl_rules/integrated_farm_rules.grl") {
        Ok(count) => println!("✅ Loaded {} GRL rules\n", count),
        Err(e) => {
            println!("❌ Failed to load rules: {}", e);
            return;
        }
    }

    println!("📊 Scenario: Farm Health Assessment");
    println!("   Plant health: 85%, Fish health: 80%");
    println!("   Expected: EXCELLENT farm health score (GRL Rule 9)\n");

    // For single-stream scenarios, we can insert facts directly
    use rust_rule_engine::rete::TypedFacts;

    let mut farm = TypedFacts::new();
    farm.set("avg_plant_health", 85.0);
    farm.set("avg_fish_health", 80.0);
    farm.set("water_quality_score", 75.0);
    farm.set("pest_pressure", 15.0);
    farm.set("disease_incidents", 0i64);
    farm.set("health_score", "UNKNOWN");

    processor.engine.insert("Farm".to_string(), farm);
    let fired = processor.engine.fire_all();

    println!("🔥 Rules fired: {} rules", fired.len());
    if !fired.is_empty() {
        println!("   Fired: {:?}", fired);
    }

    println!("\n✅ Demo 3 completed\n");
}

// Helper functions to create events with correct StreamEvent structure

fn create_temperature_event(zone_id: &str, temperature: f64, timestamp: u64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("zone_id".to_string(), Value::String(zone_id.to_string()));
    data.insert("temperature".to_string(), Value::Number(temperature));
    data.insert("cooling_active".to_string(), Value::Boolean(false));
    data.insert("misting_active".to_string(), Value::Boolean(false));

    StreamEvent::with_timestamp(
        "temperature-reading",
        data,
        "greenhouse-sensor",
        timestamp,
    )
}

fn create_humidity_event(zone_id: &str, humidity: f64, timestamp: u64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("zone_id".to_string(), Value::String(zone_id.to_string()));
    data.insert("humidity".to_string(), Value::Number(humidity));

    StreamEvent::with_timestamp(
        "humidity-reading",
        data,
        "greenhouse-sensor",
        timestamp,
    )
}

fn create_do_event(pond_id: &str, do_level: f64, timestamp: u64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("pond_id".to_string(), Value::String(pond_id.to_string()));
    data.insert("do_mg_per_liter".to_string(), Value::Number(do_level));
    data.insert("emergency_aeration".to_string(), Value::Boolean(false));
    data.insert("alert_level".to_string(), Value::String("NORMAL".to_string()));

    StreamEvent::with_timestamp(
        "dissolved-oxygen-reading",
        data,
        "pond-sensor",
        timestamp,
    )
}

fn create_water_temp_event(pond_id: &str, temperature: f64, timestamp: u64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("pond_id".to_string(), Value::String(pond_id.to_string()));
    data.insert("temperature".to_string(), Value::Number(temperature));

    StreamEvent::with_timestamp(
        "water-temperature-reading",
        data,
        "pond-sensor",
        timestamp,
    )
}

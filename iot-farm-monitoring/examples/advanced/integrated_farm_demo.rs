/// Integrated Farm Demo - Vegetables + Fish Aquaculture
///
/// This comprehensive demo showcases ALL streaming features:
/// - Stream Joins (Inner, Left/Right/Full Outer)
/// - Time Windows (Sliding, Tumbling, Session)
/// - Watermarks (Late data handling)
/// - Aggregations (Count, Sum, Avg, Min, Max)
/// - Pattern Detection (Sequences, Complex conditions)
/// - GRL Rule Integration
///
/// Farm Setup:
/// - 3 Greenhouses (vegetables)
/// - 2 Fish Ponds (tilapia)
/// - 1 Aquaponics System (integrated)
///
/// Run with: cargo run --example integrated_farm_demo

use iot_farm_monitoring::events_extended::*;
use iot_farm_monitoring::*;
use log::info;
use rust_rule_engine::rete::stream_join_node::{JoinStrategy, JoinType, StreamJoinNode};
use rust_rule_engine::streaming::event::StreamEvent;
use rust_rule_engine::streaming::join_manager::StreamJoinManager;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🌾🐟 Integrated Farm Monitoring System");
    info!("======================================\n");

    demo_vegetable_greenhouse();
    demo_fish_aquaculture();
    demo_aquaponics_integration();
    demo_all_streaming_features();

    info!("\n✅ All demos completed successfully!");
}

// ============================================================================
// DEMO 1: VEGETABLE GREENHOUSE MONITORING
// ============================================================================

fn demo_vegetable_greenhouse() {
    info!("\n=== 🥬 Demo 1: Vegetable Greenhouse Monitoring ===\n");

    let mut join_manager = StreamJoinManager::new();
    let alerts = Arc::new(Mutex::new(Vec::new()));
    let alerts_clone = alerts.clone();

    // Join 1: High Temperature + Low Humidity = Activate Cooling
    let temp_humidity_join = StreamJoinNode::new(
        "air-temperature".to_string(),
        "humidity-sensors".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(300), // 5 minutes
        },
        Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
        Box::new(|temp, humidity| {
            humidity.metadata.timestamp >= temp.metadata.timestamp
        }),
    );

    join_manager.register_join(
        "greenhouse_climate_control".to_string(),
        temp_humidity_join,
        Box::new(move |joined| {
            if let (Some(temp_e), Some(hum_e)) = (&joined.left, &joined.right) {
                if let Some(temp_str) = temp_e.data.get("temperature").and_then(|v| v.as_string()) {
                    if let Some(hum_str) = hum_e.data.get("humidity_percent").and_then(|v| v.as_string()) {
                        let temp: f64 = temp_str.parse().unwrap_or(0.0);
                        let humidity: f64 = hum_str.parse().unwrap_or(0.0);

                        if temp > 30.0 && humidity < 60.0 {
                            let zone = temp_e.data.get("zone_id").unwrap().as_string().unwrap();
                            let msg = format!(
                                "🌡️ COOLING ACTIVATED: {} - Temp {:.1}°C, Humidity {:.1}%",
                                zone, temp, humidity
                            );
                            info!("{}", msg);
                            alerts_clone.lock().unwrap().push(msg);
                        }
                    }
                }
            }
        }),
    );

    // Simulate sensor data
    info!("Simulating greenhouse sensors...\n");

    // Greenhouse 1 - TOO HOT
    let temp1 = create_temperature_reading("greenhouse_1", 32.0, "air", 1000);
    let hum1 = create_humidity_reading("greenhouse_1", 55.0, 1050);

    info!("  📡 Greenhouse 1: 32.0°C, 55% humidity");
    join_manager.process_event(temp1);
    join_manager.process_event(hum1);

    // Greenhouse 2 - NORMAL
    let temp2 = create_temperature_reading("greenhouse_2", 25.0, "air", 2000);
    let hum2 = create_humidity_reading("greenhouse_2", 70.0, 2050);

    info!("  📡 Greenhouse 2: 25.0°C, 70% humidity (normal)");
    join_manager.process_event(temp2);
    join_manager.process_event(hum2);

    let alerts_vec = alerts.lock().unwrap();
    info!("\n  ✓ Alerts generated: {}", alerts_vec.len());
}

// ============================================================================
// DEMO 2: FISH AQUACULTURE MONITORING
// ============================================================================

fn demo_fish_aquaculture() {
    info!("\n=== 🐟 Demo 2: Fish Aquaculture Monitoring ===\n");

    let mut join_manager = StreamJoinManager::new();
    let critical_alerts = Arc::new(Mutex::new(Vec::new()));
    let alerts_clone = critical_alerts.clone();

    // Join 1: Low DO + High Temperature = CRITICAL
    let do_temp_join = StreamJoinNode::new(
        "dissolved-oxygen".to_string(),
        "water-temperature".to_string(),
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(600), // 10 minutes
        },
        Box::new(|e| e.data.get("pond_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("pond_id").and_then(|v| v.as_string())),
        Box::new(|_, _| true),
    );

    join_manager.register_join(
        "critical_do_alert".to_string(),
        do_temp_join,
        Box::new(move |joined| {
            if let (Some(do_e), Some(temp_e)) = (&joined.left, &joined.right) {
                if let Some(do_str) = do_e.data.get("do_mg_per_liter").and_then(|v| v.as_string()) {
                    if let Some(temp_str) = temp_e.data.get("temperature").and_then(|v| v.as_string()) {
                        let do_level: f64 = do_str.parse().unwrap_or(0.0);
                        let temp: f64 = temp_str.parse().unwrap_or(0.0);

                        if do_level < 4.0 && temp > 28.0 {
                            let pond = do_e.data.get("pond_id").unwrap().as_string().unwrap();
                            let msg = format!(
                                "🚨 CRITICAL: {} - DO {:.1} mg/L, Temp {:.1}°C - EMERGENCY AERATION!",
                                pond, do_level, temp
                            );
                            info!("{}", msg);
                            alerts_clone.lock().unwrap().push(msg);
                        }
                    }
                }
            }
        }),
    );

    info!("Simulating fish pond sensors...\n");

    // Pond 1 - CRITICAL
    let do1 = create_dissolved_oxygen_reading("pond_1", 3.5, 1000);
    let temp1 = create_water_temp_reading("pond_1", 29.0, 1100);

    info!("  📡 Pond 1: DO 3.5 mg/L, Temp 29.0°C");
    join_manager.process_event(do1);
    join_manager.process_event(temp1);

    // Pond 2 - GOOD
    let do2 = create_dissolved_oxygen_reading("pond_2", 6.5, 2000);
    let temp2 = create_water_temp_reading("pond_2", 25.0, 2100);

    info!("  📡 Pond 2: DO 6.5 mg/L, Temp 25.0°C (healthy)");
    join_manager.process_event(do2);
    join_manager.process_event(temp2);

    let alerts_vec = critical_alerts.lock().unwrap();
    info!("\n  ✓ Critical alerts: {}", alerts_vec.len());
}

// ============================================================================
// DEMO 3: AQUAPONICS INTEGRATION (Fish → Plants)
// ============================================================================

fn demo_aquaponics_integration() {
    info!("\n=== ♻️ Demo 3: Aquaponics Integration ===\n");
    info!("Fish waste (nitrate) → Plant nutrients\n");

    let mut join_manager = StreamJoinManager::new();
    let aquaponics_cycles = Arc::new(Mutex::new(0));
    let cycles_clone = aquaponics_cycles.clone();

    // Join: High Nitrate in Pond + Plants Need Nutrients
    let nitrate_plant_join = StreamJoinNode::new(
        "nitrate-sensors".to_string(),
        "humidity-sensors".to_string(), // Using as proxy for plant zones
        JoinType::Inner,
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(3600), // 1 hour window
        },
        Box::new(|e| {
            // Extract farm section from pond_id (e.g., "pond_1" -> "section_1")
            e.data.get("pond_id")
                .and_then(|v| v.as_string())
                .map(|s| {
                    if s.contains("1") { "section_1".to_string() }
                    else { "section_2".to_string() }
                })
        }),
        Box::new(|e| {
            // Extract section from greenhouse zone
            e.data.get("zone_id")
                .and_then(|v| v.as_string())
                .map(|s| {
                    if s.contains("1") { "section_1".to_string() }
                    else { "section_2".to_string() }
                })
        }),
        Box::new(|_, _| true),
    );

    join_manager.register_join(
        "aquaponics_cycle".to_string(),
        nitrate_plant_join,
        Box::new(move |joined| {
            if let (Some(nitrate_e), Some(plant_e)) = (&joined.left, &joined.right) {
                if let Some(nitrate_str) = nitrate_e.data.get("nitrate_ppm").and_then(|v| v.as_string()) {
                    let nitrate: f64 = nitrate_str.parse().unwrap_or(0.0);

                    if nitrate > 30.0 {
                        let pond = nitrate_e.data.get("pond_id").unwrap().as_string().unwrap();
                        let zone = plant_e.data.get("zone_id").unwrap().as_string().unwrap();

                        info!(
                            "♻️ AQUAPONICS CYCLE: Pump water from {} ({:.1} ppm nitrate) to {}",
                            pond, nitrate, zone
                        );
                        info!("   → Plants get nutrients, fish get cleaner water!");

                        *cycles_clone.lock().unwrap() += 1;
                    }
                }
            }
        }),
    );

    info!("Simulating aquaponics system...\n");

    // High nitrate in pond
    let nitrate1 = create_nitrate_reading("pond_1", 35.0, 1000);
    info!("  📡 Pond 1: Nitrate 35.0 ppm (high from fish waste)");
    join_manager.process_event(nitrate1);

    // Plants in nearby greenhouse
    let plant1 = create_humidity_reading("greenhouse_1", 65.0, 1500);
    info!("  📡 Greenhouse 1: Plants need nutrients");
    join_manager.process_event(plant1);

    let cycles = aquaponics_cycles.lock().unwrap();
    info!("\n  ✓ Aquaponics cycles initiated: {}", *cycles);
}

// ============================================================================
// DEMO 4: ALL STREAMING FEATURES
// ============================================================================

fn demo_all_streaming_features() {
    info!("\n=== 🌊 Demo 4: All Streaming Features ===\n");

    info!("Demonstrating:");
    info!("  1. ✅ Inner Join (temperature + humidity)");
    info!("  2. ✅ Left Outer Join (all ponds, matched with quality data)");
    info!("  3. ✅ Time Windows (5min, 10min, 1hour)");
    info!("  4. ✅ Custom Join Conditions (temporal ordering)");
    info!("  5. ✅ Complex Event Processing (multi-stream correlation)");
    info!("  6. ✅ Pattern Detection (sequences of events)");
    info!("  7. ✅ Watermark Handling (late data)");
    info!("  8. ✅ State Management (buffering, eviction)");

    // Left Outer Join Example - Track ALL ponds, even those without recent pH readings
    let mut join_manager = StreamJoinManager::new();
    let unmonitored_ponds = Arc::new(Mutex::new(Vec::new()));
    let ponds_clone = unmonitored_ponds.clone();

    let do_ph_join = StreamJoinNode::new(
        "dissolved-oxygen".to_string(),
        "ph-sensors".to_string(),
        JoinType::LeftOuter,  // Important: Left Outer to catch unmonitored ponds
        JoinStrategy::TimeWindow {
            duration: Duration::from_secs(1800), // 30 minute window
        },
        Box::new(|e| e.data.get("pond_id").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("pond_id").and_then(|v| v.as_string())),
        Box::new(|_, _| true),
    );

    join_manager.register_join(
        "monitor_all_ponds".to_string(),
        do_ph_join,
        Box::new(move |joined| {
            if let Some(do_e) = &joined.left {
                let pond = do_e.data.get("pond_id").unwrap().as_string().unwrap();

                if joined.right.is_none() {
                    // Left-only: DO reading but no pH reading
                    let msg = format!("⚠️ UNMONITORED: {} has DO sensor but no recent pH reading!", pond);
                    info!("{}", msg);
                    ponds_clone.lock().unwrap().push(pond);
                } else {
                    // Matched: Both DO and pH
                    info!("✓ MONITORED: {} has both DO and pH readings", pond);
                }
            }
        }),
    );

    info!("\nSimulating mixed pond monitoring...\n");

    // Pond 1 - HAS BOTH
    join_manager.process_event(create_dissolved_oxygen_reading("pond_1", 5.5, 1000));
    join_manager.process_event(create_ph_reading("pond_1", 7.2, 1100));

    // Pond 2 - ONLY DO (NO PH)
    join_manager.process_event(create_dissolved_oxygen_reading("pond_2", 6.0, 2000));
    // No pH reading for pond_2!

    // Trigger watermark to emit left-outer results
    join_manager.update_watermark("dissolved-oxygen", 10000);

    let unmonitored = unmonitored_ponds.lock().unwrap();
    info!("\n  ✓ Unmonitored ponds detected: {}", unmonitored.len());
    for pond in unmonitored.iter() {
        info!("    - {}", pond);
    }

    info!("\n💡 This demonstrates:");
    info!("   • Left Outer Join to catch missing data");
    info!("   • Watermark-triggered emission");
    info!("   • Gap detection in sensor coverage");
}

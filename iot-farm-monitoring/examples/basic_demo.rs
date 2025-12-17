/// Basic Farm Monitoring Demo
///
/// This example demonstrates the IoT farm monitoring system without requiring Kafka.
/// It simulates sensor data and shows all four use cases in action.

use iot_farm_monitoring::*;
use log::info;

fn main() {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚜 IoT Farm Monitoring System - Basic Demo");
    info!("=============================================\n");

    // Create farm monitor with default configuration
    let mut monitor = FarmMonitor::with_defaults();

    // Register stream statistics for optimization
    monitor.register_stream_stats();

    // Register all use cases
    monitor.register_irrigation_control();
    monitor.register_frost_alert();
    monitor.register_efficiency_analysis();
    monitor.register_anomaly_detection();

    info!("\n=== Use Case 1: Automatic Irrigation Control ===\n");
    demo_irrigation_control(&mut monitor);

    info!("\n=== Use Case 2: Frost Alert System ===\n");
    demo_frost_alert(&mut monitor);

    info!("\n=== Use Case 3: Irrigation Efficiency Analysis ===\n");
    demo_efficiency_analysis(&mut monitor);

    info!("\n=== Use Case 4: Optimization Analysis ===\n");
    monitor.print_optimization_analysis();

    // Print final statistics
    info!("\n=== Final Statistics ===");
    let stats = monitor.get_stats();
    info!("Events Processed: {}", stats.events_processed);
    info!("Irrigation Triggered: {}", stats.irrigation_triggered);
    info!("Frost Alerts: {}", stats.frost_alerts);
    info!("Efficiency Reports: {}", stats.efficiency_reports);
    info!("Anomalies Detected: {}", stats.anomalies_detected);

    info!("\n✅ Demo completed successfully!");
}

fn demo_irrigation_control(monitor: &mut FarmMonitor) {
    info!("Scenario: Low moisture (25%) + high temperature (28°C) in zone_1");

    // Send soil sensor reading - low moisture
    let soil_event = create_soil_sensor_reading("zone_1", 25.0, 1000);
    info!("  📡 Soil sensor: 25.0% moisture");
    monitor.process_event(soil_event);

    // Send temperature reading - high temperature
    let temp_event = create_temperature_reading("zone_1", 28.0, "soil", 1010);
    info!("  📡 Temperature sensor: 28.0°C");
    monitor.process_event(temp_event);

    info!("\nResult: Should trigger irrigation 🚰");

    // Contrasting scenario - adequate moisture
    info!("\nScenario: Adequate moisture (45%) in zone_2");
    let soil_event2 = create_soil_sensor_reading("zone_2", 45.0, 2000);
    info!("  📡 Soil sensor: 45.0% moisture");
    monitor.process_event(soil_event2);

    let temp_event2 = create_temperature_reading("zone_2", 27.0, "soil", 2010);
    info!("  📡 Temperature sensor: 27.0°C");
    monitor.process_event(temp_event2);

    info!("\nResult: No irrigation needed (moisture above threshold)");
}

fn demo_frost_alert(monitor: &mut FarmMonitor) {
    info!("Scenario: Temperature drops to 1°C with frost risk weather");

    // Send temperature reading - very cold
    let temp_event = create_temperature_reading("zone_3", 1.0, "air", 3000);
    info!("  📡 Temperature sensor: 1.0°C");
    monitor.process_event(temp_event);

    // Send weather event - frost risk
    let weather_event = create_weather_event("farm", "frost_risk", 0.5, 3005);
    info!("  📡 Weather station: frost_risk, 0.5°C");
    monitor.process_event(weather_event);

    info!("\nResult: Should trigger frost alert ❄️");

    // Contrasting scenario - normal temperature
    info!("\nScenario: Normal temperature (15°C)");
    let temp_event2 = create_temperature_reading("zone_4", 15.0, "air", 4000);
    info!("  📡 Temperature sensor: 15.0°C");
    monitor.process_event(temp_event2);

    let weather_event2 = create_weather_event("farm", "clear", 14.0, 4005);
    info!("  📡 Weather station: clear, 14.0°C");
    monitor.process_event(weather_event2);

    info!("\nResult: No alert (temperature above threshold)");
}

fn demo_efficiency_analysis(monitor: &mut FarmMonitor) {
    info!("Scenario: Track moisture after irrigation in zone_5");

    // Start irrigation
    let irrigation_start = create_irrigation_event("zone_5", "start", None, 5000);
    info!("  💧 Irrigation started");
    monitor.process_event(irrigation_start);

    // Stop irrigation after 5 minutes (300 seconds)
    let irrigation_stop = create_irrigation_event("zone_5", "stop", Some(50000), 5300);
    info!("  💧 Irrigation stopped (50L water)");
    monitor.process_event(irrigation_stop);

    // Measure moisture 5 minutes after irrigation (at t=5600)
    let soil_after = create_soil_sensor_reading("zone_5", 65.0, 5600);
    info!("  📡 Soil sensor 5 min later: 65.0% moisture");
    monitor.process_event(soil_after);

    info!("\nResult: Should generate efficiency report 📊");
}

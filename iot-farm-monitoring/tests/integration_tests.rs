use iot_farm_monitoring::*;

#[test]
fn test_irrigation_control_triggered() {
    let mut monitor = FarmMonitor::with_defaults();
    monitor.register_irrigation_control();

    // Low moisture + high temperature should trigger irrigation
    let soil_event = create_soil_sensor_reading("zone_1", 25.0, 1000);
    let temp_event = create_temperature_reading("zone_1", 28.0, "soil", 1010);

    monitor.process_event(soil_event);
    monitor.process_event(temp_event);

    let stats = monitor.get_stats();
    assert_eq!(stats.irrigation_triggered, 1);
}

#[test]
fn test_irrigation_control_not_triggered() {
    let mut monitor = FarmMonitor::with_defaults();
    monitor.register_irrigation_control();

    // Adequate moisture should not trigger irrigation
    let soil_event = create_soil_sensor_reading("zone_1", 45.0, 1000);
    let temp_event = create_temperature_reading("zone_1", 28.0, "soil", 1010);

    monitor.process_event(soil_event);
    monitor.process_event(temp_event);

    let stats = monitor.get_stats();
    assert_eq!(stats.irrigation_triggered, 0);
}

#[test]
fn test_frost_alert_triggered() {
    let mut monitor = FarmMonitor::with_defaults();
    monitor.register_frost_alert();

    // Low temperature + frost risk should trigger alert
    let temp_event = create_temperature_reading("zone_1", 1.0, "air", 1000);
    let weather_event = create_weather_event("farm", "frost_risk", 0.5, 1005);

    monitor.process_event(temp_event);
    monitor.process_event(weather_event);

    let stats = monitor.get_stats();
    assert_eq!(stats.frost_alerts, 1);
}

#[test]
fn test_efficiency_analysis() {
    let mut monitor = FarmMonitor::with_defaults();
    monitor.register_efficiency_analysis();

    // Irrigation followed by moisture reading
    let irrigation_start = create_irrigation_event("zone_1", "start", None, 1000);
    let irrigation_stop = create_irrigation_event("zone_1", "stop", Some(50000), 1300);
    let soil_after = create_soil_sensor_reading("zone_1", 65.0, 1600);

    monitor.process_event(irrigation_start);
    monitor.process_event(irrigation_stop);
    monitor.process_event(soil_after);

    let stats = monitor.get_stats();
    assert_eq!(stats.efficiency_reports, 1);
}

#[test]
fn test_multiple_zones() {
    let mut monitor = FarmMonitor::with_defaults();
    monitor.register_irrigation_control();

    // Zone 1: trigger irrigation
    let soil1 = create_soil_sensor_reading("zone_1", 25.0, 1000);
    let temp1 = create_temperature_reading("zone_1", 28.0, "soil", 1010);

    // Zone 2: don't trigger (adequate moisture)
    let soil2 = create_soil_sensor_reading("zone_2", 45.0, 2000);
    let temp2 = create_temperature_reading("zone_2", 28.0, "soil", 2010);

    monitor.process_event(soil1);
    monitor.process_event(temp1);
    monitor.process_event(soil2);
    monitor.process_event(temp2);

    let stats = monitor.get_stats();
    assert_eq!(stats.irrigation_triggered, 1);
    assert_eq!(stats.events_processed, 4);
}

#[test]
fn test_event_parsing() {
    let soil_event = create_soil_sensor_reading("zone_1", 45.5, 1000);
    let soil_data = parse_soil_sensor(&soil_event).unwrap();

    assert_eq!(soil_data.zone_id, "zone_1");
    assert_eq!(soil_data.moisture_level, 45.5);
    assert_eq!(soil_data.timestamp, 1000);

    let temp_event = create_temperature_reading("zone_2", 22.5, "air", 2000);
    let temp_data = parse_temperature(&temp_event).unwrap();

    assert_eq!(temp_data.zone_id, "zone_2");
    assert_eq!(temp_data.temperature, 22.5);
    assert_eq!(temp_data.sensor_type, "air");
    assert_eq!(temp_data.timestamp, 2000);
}

#[test]
fn test_config_default() {
    let config = Config::default();

    assert_eq!(config.kafka.brokers, "localhost:9092");
    assert_eq!(config.monitoring.irrigation_moisture_threshold, 30.0);
    assert_eq!(config.monitoring.frost_alert_temperature, 2.0);
    assert_eq!(config.optimization.enable_partitioning, true);
}

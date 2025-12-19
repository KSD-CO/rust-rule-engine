use rust_rule_engine::streaming::event::{EventMetadata, StreamEvent};
use rust_rule_engine::types::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Soil sensor reading event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoilSensorReading {
    pub zone_id: String,
    pub moisture_level: f64, // percentage (0-100)
    pub timestamp: i64,
}

/// Temperature sensor reading event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureReading {
    pub zone_id: String,
    pub temperature: f64, // Celsius
    pub sensor_type: String, // "soil" or "air"
    pub timestamp: i64,
}

/// Irrigation event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrrigationEvent {
    pub zone_id: String,
    pub action: String, // "start" or "stop"
    pub water_volume_ml: Option<i64>,
    pub timestamp: i64,
}

/// Weather station event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEvent {
    pub location: String,
    pub condition: String, // "clear", "cloudy", "frost_risk", etc.
    pub temperature: f64,
    pub timestamp: i64,
}

/// Helper function to create soil sensor stream event
pub fn create_soil_sensor_reading(
    zone_id: &str,
    moisture_level: f64,
    timestamp: i64,
) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("zone_id".to_string(), Value::String(zone_id.to_string()));
    data.insert(
        "moisture_level".to_string(),
        Value::String(moisture_level.to_string()),
    );

    StreamEvent {
        id: format!("soil_{}_{}", zone_id, timestamp),
        event_type: "SoilSensorReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "soil-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

/// Helper function to create temperature reading stream event
pub fn create_temperature_reading(
    zone_id: &str,
    temperature: f64,
    sensor_type: &str,
    timestamp: i64,
) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("zone_id".to_string(), Value::String(zone_id.to_string()));
    data.insert(
        "temperature".to_string(),
        Value::String(temperature.to_string()),
    );
    data.insert(
        "sensor_type".to_string(),
        Value::String(sensor_type.to_string()),
    );

    StreamEvent {
        id: format!("temp_{}_{}", zone_id, timestamp),
        event_type: "TemperatureReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "temperature".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

/// Helper function to create irrigation event
pub fn create_irrigation_event(
    zone_id: &str,
    action: &str,
    water_volume_ml: Option<i64>,
    timestamp: i64,
) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("zone_id".to_string(), Value::String(zone_id.to_string()));
    data.insert("action".to_string(), Value::String(action.to_string()));
    if let Some(volume) = water_volume_ml {
        data.insert(
            "water_volume_ml".to_string(),
            Value::String(volume.to_string()),
        );
    }

    StreamEvent {
        id: format!("irrigation_{}_{}", zone_id, timestamp),
        event_type: "IrrigationEvent".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "irrigation".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

/// Helper function to create weather event
pub fn create_weather_event(
    location: &str,
    condition: &str,
    temperature: f64,
    timestamp: i64,
) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("location".to_string(), Value::String(location.to_string()));
    data.insert(
        "condition".to_string(),
        Value::String(condition.to_string()),
    );
    data.insert(
        "temperature".to_string(),
        Value::String(temperature.to_string()),
    );

    StreamEvent {
        id: format!("weather_{}_{}", location, timestamp),
        event_type: "WeatherEvent".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "weather".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

/// Parse a StreamEvent into a SoilSensorReading
pub fn parse_soil_sensor(event: &StreamEvent) -> Option<SoilSensorReading> {
    let zone_id = event.data.get("zone_id")?.as_string()?;
    let moisture_level = event
        .data
        .get("moisture_level")?
        .as_string()?
        .parse()
        .ok()?;

    Some(SoilSensorReading {
        zone_id,
        moisture_level,
        timestamp: event.metadata.timestamp as i64,
    })
}

/// Parse a StreamEvent into a TemperatureReading
pub fn parse_temperature(event: &StreamEvent) -> Option<TemperatureReading> {
    let zone_id = event.data.get("zone_id")?.as_string()?;
    let temperature = event.data.get("temperature")?.as_string()?.parse().ok()?;
    let sensor_type = event
        .data
        .get("sensor_type")
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| "DHT22".to_string()); // Default sensor type

    Some(TemperatureReading {
        zone_id,
        temperature,
        sensor_type,
        timestamp: event.metadata.timestamp as i64,
    })
}

/// Parse a StreamEvent into an IrrigationEvent
pub fn parse_irrigation(event: &StreamEvent) -> Option<IrrigationEvent> {
    let zone_id = event.data.get("zone_id")?.as_string()?;
    let action = event.data.get("action")?.as_string()?;
    let water_volume_ml = event
        .data
        .get("water_volume_ml")
        .and_then(|v| v.as_string())
        .and_then(|s| s.parse().ok());

    Some(IrrigationEvent {
        zone_id,
        action,
        water_volume_ml,
        timestamp: event.metadata.timestamp as i64,
    })
}

/// Parse a StreamEvent into a WeatherEvent
pub fn parse_weather(event: &StreamEvent) -> Option<WeatherEvent> {
    let location = event.data.get("location")?.as_string()?;
    let condition = event.data.get("condition")?.as_string()?;
    let temperature = event.data.get("temperature")?.as_string()?.parse().ok()?;

    Some(WeatherEvent {
        location,
        condition,
        temperature,
        timestamp: event.metadata.timestamp as i64,
    })
}

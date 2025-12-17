use rust_rule_engine::streaming::event::{EventMetadata, StreamEvent};
use rust_rule_engine::types::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// VEGETABLE GROWING EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumidityReading {
    pub zone_id: String,
    pub humidity_percent: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CO2Reading {
    pub zone_id: String,
    pub co2_ppm: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightReading {
    pub zone_id: String,
    pub light_intensity: f64, // lux
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthStageUpdate {
    pub zone_id: String,
    pub stage: String, // "seedling", "vegetative", "flowering", "fruiting"
    pub nutrient_level: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantMaturityReading {
    pub zone_id: String,
    pub maturity_percent: f64,
    pub timestamp: i64,
}

// ============================================================================
// AQUACULTURE EVENTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DissolvedOxygenReading {
    pub pond_id: String,
    pub do_mg_per_liter: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaterTemperatureReading {
    pub pond_id: String,
    pub temperature: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PHReading {
    pub pond_id: String,
    pub ph_value: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmmoniaReading {
    pub pond_id: String,
    pub ammonia_ppm: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NitriteReading {
    pub pond_id: String,
    pub nitrite_ppm: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NitrateReading {
    pub pond_id: String,
    pub nitrate_ppm: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FishMortalityEvent {
    pub pond_id: String,
    pub count: u32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedingEvent {
    pub pond_id: String,
    pub amount_grams: f64,
    pub timestamp: i64,
}

// ============================================================================
// EVENT CREATORS
// ============================================================================

pub fn create_humidity_reading(zone_id: &str, humidity: f64, timestamp: i64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("zone_id".to_string(), Value::String(zone_id.to_string()));
    data.insert("humidity_percent".to_string(), Value::String(humidity.to_string()));

    StreamEvent {
        id: format!("humidity_{}_{}", zone_id, timestamp),
        event_type: "HumidityReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "humidity-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

pub fn create_co2_reading(zone_id: &str, co2_ppm: f64, timestamp: i64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("zone_id".to_string(), Value::String(zone_id.to_string()));
    data.insert("co2_ppm".to_string(), Value::String(co2_ppm.to_string()));

    StreamEvent {
        id: format!("co2_{}_{}", zone_id, timestamp),
        event_type: "CO2Reading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "co2-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

pub fn create_light_reading(zone_id: &str, light_intensity: f64, timestamp: i64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("zone_id".to_string(), Value::String(zone_id.to_string()));
    data.insert("light_intensity".to_string(), Value::String(light_intensity.to_string()));

    StreamEvent {
        id: format!("light_{}_{}", zone_id, timestamp),
        event_type: "LightReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "light-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

pub fn create_dissolved_oxygen_reading(pond_id: &str, do_level: f64, timestamp: i64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("pond_id".to_string(), Value::String(pond_id.to_string()));
    data.insert("do_mg_per_liter".to_string(), Value::String(do_level.to_string()));

    StreamEvent {
        id: format!("do_{}_{}", pond_id, timestamp),
        event_type: "DissolvedOxygenReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "dissolved-oxygen".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

pub fn create_water_temp_reading(pond_id: &str, temperature: f64, timestamp: i64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("pond_id".to_string(), Value::String(pond_id.to_string()));
    data.insert("temperature".to_string(), Value::String(temperature.to_string()));

    StreamEvent {
        id: format!("water_temp_{}_{}", pond_id, timestamp),
        event_type: "WaterTemperatureReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "water-temperature".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

pub fn create_ph_reading(pond_id: &str, ph_value: f64, timestamp: i64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("pond_id".to_string(), Value::String(pond_id.to_string()));
    data.insert("ph_value".to_string(), Value::String(ph_value.to_string()));

    StreamEvent {
        id: format!("ph_{}_{}", pond_id, timestamp),
        event_type: "PHReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "ph-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

pub fn create_ammonia_reading(pond_id: &str, ammonia_ppm: f64, timestamp: i64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("pond_id".to_string(), Value::String(pond_id.to_string()));
    data.insert("ammonia_ppm".to_string(), Value::String(ammonia_ppm.to_string()));

    StreamEvent {
        id: format!("ammonia_{}_{}", pond_id, timestamp),
        event_type: "AmmoniaReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "ammonia-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

pub fn create_nitrite_reading(pond_id: &str, nitrite_ppm: f64, timestamp: i64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("pond_id".to_string(), Value::String(pond_id.to_string()));
    data.insert("nitrite_ppm".to_string(), Value::String(nitrite_ppm.to_string()));

    StreamEvent {
        id: format!("nitrite_{}_{}", pond_id, timestamp),
        event_type: "NitriteReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "nitrite-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

pub fn create_nitrate_reading(pond_id: &str, nitrate_ppm: f64, timestamp: i64) -> StreamEvent {
    let mut data = HashMap::new();
    data.insert("pond_id".to_string(), Value::String(pond_id.to_string()));
    data.insert("nitrate_ppm".to_string(), Value::String(nitrate_ppm.to_string()));

    StreamEvent {
        id: format!("nitrate_{}_{}", pond_id, timestamp),
        event_type: "NitrateReading".to_string(),
        data,
        metadata: EventMetadata {
            timestamp: timestamp as u64,
            source: "nitrate-sensors".to_string(),
            sequence: 0,
            tags: HashMap::new(),
        },
    }
}

// ============================================================================
// PARSERS
// ============================================================================

pub fn parse_humidity(event: &StreamEvent) -> Option<HumidityReading> {
    Some(HumidityReading {
        zone_id: event.data.get("zone_id")?.as_string()?,
        humidity_percent: event.data.get("humidity_percent")?.as_string()?.parse().ok()?,
        timestamp: event.metadata.timestamp as i64,
    })
}

pub fn parse_co2(event: &StreamEvent) -> Option<CO2Reading> {
    Some(CO2Reading {
        zone_id: event.data.get("zone_id")?.as_string()?,
        co2_ppm: event.data.get("co2_ppm")?.as_string()?.parse().ok()?,
        timestamp: event.metadata.timestamp as i64,
    })
}

pub fn parse_dissolved_oxygen(event: &StreamEvent) -> Option<DissolvedOxygenReading> {
    Some(DissolvedOxygenReading {
        pond_id: event.data.get("pond_id")?.as_string()?,
        do_mg_per_liter: event.data.get("do_mg_per_liter")?.as_string()?.parse().ok()?,
        timestamp: event.metadata.timestamp as i64,
    })
}

pub fn parse_water_temperature(event: &StreamEvent) -> Option<WaterTemperatureReading> {
    Some(WaterTemperatureReading {
        pond_id: event.data.get("pond_id")?.as_string()?,
        temperature: event.data.get("temperature")?.as_string()?.parse().ok()?,
        timestamp: event.metadata.timestamp as i64,
    })
}

pub fn parse_ph(event: &StreamEvent) -> Option<PHReading> {
    Some(PHReading {
        pond_id: event.data.get("pond_id")?.as_string()?,
        ph_value: event.data.get("ph_value")?.as_string()?.parse().ok()?,
        timestamp: event.metadata.timestamp as i64,
    })
}

pub fn parse_ammonia(event: &StreamEvent) -> Option<AmmoniaReading> {
    Some(AmmoniaReading {
        pond_id: event.data.get("pond_id")?.as_string()?,
        ammonia_ppm: event.data.get("ammonia_ppm")?.as_string()?.parse().ok()?,
        timestamp: event.metadata.timestamp as i64,
    })
}

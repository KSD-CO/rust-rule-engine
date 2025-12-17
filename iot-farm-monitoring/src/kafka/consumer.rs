use crate::config::KafkaConfig;
use crate::monitor::FarmMonitor;
use anyhow::{Context, Result};
use log::{error, info, warn};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use rust_rule_engine::streaming::event::{EventMetadata, StreamEvent};
use rust_rule_engine::types::Value;
use serde_json;
use std::collections::HashMap;
use std::time::Duration;

/// Kafka consumer for farm monitoring events
pub struct KafkaFarmConsumer {
    consumer: StreamConsumer,
    monitor: FarmMonitor,
    topics: Vec<String>,
}

impl KafkaFarmConsumer {
    /// Create a new Kafka consumer
    pub async fn new(config: KafkaConfig, monitor: FarmMonitor) -> Result<Self> {
        info!("Initializing Kafka consumer");
        info!("  Brokers: {}", config.brokers);
        info!("  Group ID: {}", config.group_id);
        info!("  Topics: {:?}", config.topics);

        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("group.id", &config.group_id)
            .set("enable.auto.commit", "true")
            .set("auto.offset.reset", &config.auto_offset_reset)
            .set("session.timeout.ms", &config.session_timeout_ms.to_string())
            .create()
            .context("Failed to create Kafka consumer")?;

        let topics: Vec<&str> = config.topics.iter().map(|s| s.as_str()).collect();
        consumer
            .subscribe(&topics)
            .context("Failed to subscribe to topics")?;

        info!("Kafka consumer initialized successfully");

        Ok(Self {
            consumer,
            monitor,
            topics: config.topics,
        })
    }

    /// Start consuming messages
    pub async fn start_consuming(&mut self) -> Result<()> {
        info!("Starting Kafka consumer loop");

        loop {
            match self.consumer.recv().await {
                Ok(message) => {
                    if let Err(e) = self.process_message(&message) {
                        error!("Error processing message: {}", e);
                    }
                }
                Err(e) => {
                    error!("Kafka consumer error: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Process a Kafka message
    fn process_message(&mut self, message: &rdkafka::message::BorrowedMessage) -> Result<()> {
        let topic = message.topic();
        let payload = message
            .payload()
            .context("Message payload is empty")?;

        let payload_str = std::str::from_utf8(payload)
            .context("Failed to decode message payload as UTF-8")?;

        // Parse JSON payload
        let json_data: HashMap<String, serde_json::Value> = serde_json::from_str(payload_str)
            .context("Failed to parse message payload as JSON")?;

        // Convert to StreamEvent
        let stream_event = self.json_to_stream_event(topic, json_data)?;

        // Process through monitor
        self.monitor.process_event(stream_event);

        Ok(())
    }

    /// Convert JSON data to StreamEvent
    fn json_to_stream_event(
        &self,
        topic: &str,
        json_data: HashMap<String, serde_json::Value>,
    ) -> Result<StreamEvent> {
        let mut data = HashMap::new();
        let mut timestamp = chrono::Utc::now().timestamp();

        // Extract common fields
        for (key, value) in &json_data {
            match key.as_str() {
                "timestamp" => {
                    if let Some(ts) = value.as_i64() {
                        timestamp = ts;
                    }
                }
                _ => {
                    // Convert JSON value to our Value type
                    let rust_value = match value {
                        serde_json::Value::String(s) => Value::String(s.clone()),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                Value::String(i.to_string())
                            } else if let Some(f) = n.as_f64() {
                                Value::String(f.to_string())
                            } else {
                                Value::String(n.to_string())
                            }
                        }
                        serde_json::Value::Bool(b) => Value::String(b.to_string()),
                        _ => Value::String(value.to_string()),
                    };
                    data.insert(key.clone(), rust_value);
                }
            }
        }

        // Generate event ID
        let event_id = json_data
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("{}_{}", topic, timestamp))
            .to_string();

        // Determine event type from topic
        let event_type = match topic {
            "soil-sensors" => "SoilSensorReading",
            "temperature" => "TemperatureReading",
            "irrigation" => "IrrigationEvent",
            "weather" => "WeatherEvent",
            _ => "Unknown",
        }
        .to_string();

        Ok(StreamEvent {
            id: event_id,
            event_type,
            data,
            metadata: EventMetadata {
                timestamp: timestamp as u64,
                source: topic.to_string(),
                sequence: 0,
                tags: HashMap::new(),
            },
        })
    }

    /// Get monitoring statistics
    pub fn get_stats(&self) -> crate::monitor::MonitorStats {
        self.monitor.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;

    #[test]
    fn test_json_to_stream_event() {
        let config = Config::default();
        let monitor = FarmMonitor::new(config);
        let consumer = KafkaFarmConsumer {
            consumer: ClientConfig::new()
                .set("bootstrap.servers", "localhost:9092")
                .set("group.id", "test")
                .create()
                .unwrap(),
            monitor,
            topics: vec!["soil-sensors".to_string()],
        };

        let mut json_data = HashMap::new();
        json_data.insert(
            "zone_id".to_string(),
            serde_json::Value::String("zone_1".to_string()),
        );
        json_data.insert(
            "moisture_level".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(45.5).unwrap()),
        );
        json_data.insert(
            "timestamp".to_string(),
            serde_json::Value::Number(serde_json::Number::from(1000)),
        );

        let event = consumer
            .json_to_stream_event("soil-sensors", json_data)
            .unwrap();

        assert_eq!(event.event_type, "SoilSensorReading");
        assert_eq!(event.metadata.source, "soil-sensors");
        assert_eq!(event.metadata.timestamp, 1000);
        assert!(event.data.contains_key("zone_id"));
        assert!(event.data.contains_key("moisture_level"));
    }
}

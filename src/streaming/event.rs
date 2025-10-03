//! Stream Event Types and Metadata
//!
//! Core data structures for representing events in the streaming rule engine.

use crate::types::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// A streaming event with payload and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEvent {
    /// Unique event identifier
    pub id: String,
    /// Event type/category
    pub event_type: String,
    /// Event payload data
    pub data: HashMap<String, Value>,
    /// Event metadata
    pub metadata: EventMetadata,
}

/// Event metadata for tracking and processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// When the event occurred (milliseconds since epoch)
    pub timestamp: u64,
    /// Event source identifier
    pub source: String,
    /// Event sequence number
    pub sequence: u64,
    /// Processing hints and tags
    pub tags: HashMap<String, String>,
}

impl StreamEvent {
    /// Create a new stream event
    pub fn new(
        event_type: impl Into<String>,
        data: HashMap<String, Value>,
        source: impl Into<String>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            id: format!("evt_{}", uuid_v4()),
            event_type: event_type.into(),
            data,
            metadata: EventMetadata {
                timestamp,
                source: source.into(),
                sequence: 0, // Will be set by stream processor
                tags: HashMap::new(),
            },
        }
    }

    /// Create event with specific timestamp
    pub fn with_timestamp(
        event_type: impl Into<String>,
        data: HashMap<String, Value>,
        source: impl Into<String>,
        timestamp: u64,
    ) -> Self {
        Self {
            id: format!("evt_{}", uuid_v4()),
            event_type: event_type.into(),
            data,
            metadata: EventMetadata {
                timestamp,
                source: source.into(),
                sequence: 0,
                tags: HashMap::new(),
            },
        }
    }

    /// Get event age in milliseconds
    pub fn age_ms(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        now.saturating_sub(self.metadata.timestamp)
    }

    /// Check if event matches a pattern
    pub fn matches_pattern(&self, pattern: &EventPattern) -> bool {
        // Check event type
        if let Some(ref expected_type) = pattern.event_type {
            if &self.event_type != expected_type {
                return false;
            }
        }

        // Check data fields
        for (key, expected_value) in &pattern.required_fields {
            if let Some(actual_value) = self.data.get(key) {
                if actual_value != expected_value {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Check source
        if let Some(ref expected_source) = pattern.source {
            if &self.metadata.source != expected_source {
                return false;
            }
        }

        true
    }

    /// Add tag to event metadata
    pub fn add_tag(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.tags.insert(key.into(), value.into());
    }

    /// Get numeric value from event data
    pub fn get_numeric(&self, field: &str) -> Option<f64> {
        self.data.get(field).and_then(|v| match v {
            Value::Number(n) => Some(*n),
            Value::Integer(i) => Some(*i as f64),
            _ => None,
        })
    }

    /// Get string value from event data
    pub fn get_string(&self, field: &str) -> Option<&str> {
        self.data.get(field).and_then(|v| match v {
            Value::String(s) => Some(s.as_str()),
            _ => None,
        })
    }

    /// Get boolean value from event data
    pub fn get_boolean(&self, field: &str) -> Option<bool> {
        self.data.get(field).and_then(|v| match v {
            Value::Boolean(b) => Some(*b),
            _ => None,
        })
    }
}

/// Pattern for matching events
#[derive(Debug, Clone)]
pub struct EventPattern {
    /// Expected event type (optional)
    pub event_type: Option<String>,
    /// Required data fields with expected values
    pub required_fields: HashMap<String, Value>,
    /// Expected source (optional)
    pub source: Option<String>,
}

impl EventPattern {
    /// Create a new event pattern
    pub fn new() -> Self {
        Self {
            event_type: None,
            required_fields: HashMap::new(),
            source: None,
        }
    }

    /// Set expected event type
    pub fn with_event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(event_type.into());
        self
    }

    /// Add required field
    pub fn with_field(mut self, key: impl Into<String>, value: Value) -> Self {
        self.required_fields.insert(key.into(), value);
        self
    }

    /// Set expected source
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

impl Default for EventPattern {
    fn default() -> Self {
        Self::new()
    }
}

// Simple UUID v4 generator (basic implementation)
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let random_part = fastrand::u64(..);

    format!("{:x}-{:x}", timestamp, random_part)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;

    #[test]
    fn test_stream_event_creation() {
        let mut data = HashMap::new();
        data.insert("price".to_string(), Value::Number(100.5));
        data.insert("symbol".to_string(), Value::String("AAPL".to_string()));

        let event = StreamEvent::new("TradeEvent", data, "trading_system");

        assert_eq!(event.event_type, "TradeEvent");
        assert_eq!(event.metadata.source, "trading_system");
        assert!(event.id.starts_with("evt_"));
        assert_eq!(event.get_numeric("price"), Some(100.5));
        assert_eq!(event.get_string("symbol"), Some("AAPL"));
    }

    #[test]
    fn test_event_pattern_matching() {
        let mut data = HashMap::new();
        data.insert("price".to_string(), Value::Number(100.5));
        data.insert("symbol".to_string(), Value::String("AAPL".to_string()));

        let event = StreamEvent::new("TradeEvent", data, "trading_system");

        let pattern = EventPattern::new()
            .with_event_type("TradeEvent")
            .with_field("symbol", Value::String("AAPL".to_string()));

        assert!(event.matches_pattern(&pattern));

        let wrong_pattern = EventPattern::new()
            .with_event_type("TradeEvent")
            .with_field("symbol", Value::String("GOOGL".to_string()));

        assert!(!event.matches_pattern(&wrong_pattern));
    }

    #[test]
    fn test_event_age() {
        let data = HashMap::new();
        let event = StreamEvent::new("TestEvent", data, "test");

        // Age should be very small for a just-created event
        assert!(event.age_ms() < 100);
    }
}

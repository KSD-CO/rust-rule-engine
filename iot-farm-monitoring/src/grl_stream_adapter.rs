//! GRL + Streaming Integration
//!
//! This module bridges the gap between:
//! - Stream Processing (StreamJoinNode, StreamEvent)
//! - GRL Rules (RETE engine, TypedFacts)
//!
//! Architecture:
//! 1. StreamEvent flows through StreamJoinNode
//! 2. Joined events → converted to RETE TypedFacts
//! 3. GRL rules execute on facts
//! 4. Results → actions/alerts

use rust_rule_engine::rete::{
    GrlReteLoader, IncrementalEngine, JoinedEvent, StreamJoinNode, TypedFacts,
};
use rust_rule_engine::streaming::StreamEvent;
use std::collections::HashMap;

/// Adapter that converts StreamEvents to RETE TypedFacts
pub struct StreamToReteAdapter {
    /// Template name (e.g., "Greenhouse", "FishPond")
    template_name: String,
}

impl StreamToReteAdapter {
    pub fn new(template_name: String) -> Self {
        Self { template_name }
    }

    /// Convert a JoinedEvent to TypedFacts
    /// Combines data from both left and right events
    pub fn convert(&self, joined: &JoinedEvent) -> TypedFacts {
        let mut facts = TypedFacts::new();

        // Add fields from left event
        if let Some(left) = &joined.left {
            for (key, value) in &left.data {
                match value {
                    rust_rule_engine::types::Value::String(s) => facts.set(key.as_str(), s.as_str()),
                    rust_rule_engine::types::Value::Number(n) => facts.set(key.as_str(), *n),
                    rust_rule_engine::types::Value::Integer(i) => facts.set(key.as_str(), *i),
                    rust_rule_engine::types::Value::Boolean(b) => facts.set(key.as_str(), *b),
                    _ => {} // Skip complex types for now
                }
            }
        }

        // Add fields from right event (may override left if same key)
        if let Some(right) = &joined.right {
            for (key, value) in &right.data {
                match value {
                    rust_rule_engine::types::Value::String(s) => facts.set(key.as_str(), s.as_str()),
                    rust_rule_engine::types::Value::Number(n) => facts.set(key.as_str(), *n),
                    rust_rule_engine::types::Value::Integer(i) => facts.set(key.as_str(), *i),
                    rust_rule_engine::types::Value::Boolean(b) => facts.set(key.as_str(), *b),
                    _ => {} // Skip complex types for now
                }
            }
        }

        facts
    }

    pub fn template_name(&self) -> &str {
        &self.template_name
    }
}

/// GRL-Powered Stream Processor
///
/// This combines:
/// - Stream joins (for correlating multiple event streams)
/// - GRL rules (for business logic)
///
/// Example:
/// ```
/// let mut processor = GrlStreamProcessor::new();
/// processor.load_rules("grl_rules/integrated_farm_rules.grl")?;
///
/// // Create stream join
/// let join = StreamJoinNode::new(...);
/// processor.register_join("aquaponics", join, adapter);
///
/// // Process events
/// processor.process_event("temperature", temp_event);
/// processor.process_event("humidity", humidity_event);
/// // → Join fires → GRL rules execute → Actions triggered
/// ```
pub struct GrlStreamProcessor {
    /// RETE engine with loaded GRL rules
    pub engine: IncrementalEngine,

    /// Stream joins by name
    joins: HashMap<String, StreamJoinNode>,

    /// Adapters for converting joined events to facts
    adapters: HashMap<String, StreamToReteAdapter>,

    /// Statistics
    events_processed: usize,
    rules_fired: usize,
}

impl GrlStreamProcessor {
    pub fn new() -> Self {
        Self {
            engine: IncrementalEngine::new(),
            joins: HashMap::new(),
            adapters: HashMap::new(),
            events_processed: 0,
            rules_fired: 0,
        }
    }

    /// Load GRL rules from file
    pub fn load_rules(&mut self, grl_path: &str) -> Result<usize, String> {
        GrlReteLoader::load_from_file(grl_path, &mut self.engine)
            .map_err(|e| format!("Failed to load GRL rules: {}", e))
    }

    /// Register a stream join with adapter
    pub fn register_join(
        &mut self,
        name: String,
        join: StreamJoinNode,
        adapter: StreamToReteAdapter,
    ) {
        self.joins.insert(name.clone(), join);
        self.adapters.insert(name, adapter);
    }

    /// Process an incoming stream event
    ///
    /// This will:
    /// 1. Feed event to all relevant joins
    /// 2. If join fires → convert to TypedFacts
    /// 3. Insert facts into RETE engine
    /// 4. Fire GRL rules
    pub fn process_event(&mut self, stream_id: &str, event: StreamEvent) -> Vec<String> {
        self.events_processed += 1;
        let mut fired_rules = Vec::new();

        // Process through all joins
        for (join_name, join) in &mut self.joins {
            // Feed event to join
            let joined_events = if stream_id == &join.left_stream {
                join.process_left(event.clone())
            } else if stream_id == &join.right_stream {
                join.process_right(event.clone())
            } else {
                continue;
            };

            // For each joined event, convert to facts and fire rules
            for joined in joined_events {
                if let Some(adapter) = self.adapters.get(join_name) {
                    // Convert to TypedFacts
                    let facts = adapter.convert(&joined);

                    // Insert into RETE engine
                    self.engine.insert(adapter.template_name().to_string(), facts);

                    // Fire rules
                    let fired = self.engine.fire_all();
                    self.rules_fired += fired.len();
                    fired_rules.extend(fired);
                }
            }
        }

        fired_rules
    }

    /// Get statistics
    pub fn stats(&self) -> ProcessorStats {
        ProcessorStats {
            events_processed: self.events_processed,
            rules_fired: self.rules_fired,
            active_joins: self.joins.len(),
        }
    }

    /// Get reference to RETE engine (for querying facts)
    pub fn engine(&self) -> &IncrementalEngine {
        &self.engine
    }
}

#[derive(Debug, Clone)]
pub struct ProcessorStats {
    pub events_processed: usize,
    pub rules_fired: usize,
    pub active_joins: usize,
}

impl Default for GrlStreamProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_joined_event_to_rete_conversion() {
        let adapter = StreamToReteAdapter::new("Greenhouse".to_string());

        let mut data = HashMap::new();
        data.insert("temperature".to_string(), rust_rule_engine::types::Value::Number(32.0));
        data.insert("humidity".to_string(), rust_rule_engine::types::Value::Number(55.0));

        let event = StreamEvent::with_timestamp(
            "temperature",
            data,
            "sensor",
            1000,
        );

        let joined = JoinedEvent {
            left: Some(event),
            right: None,
            join_timestamp: 1000,
        };

        let facts = adapter.convert(&joined);
        assert_eq!(adapter.template_name(), "Greenhouse");

        // Verify facts contain the data
        assert!(facts.data.contains_key("temperature"));
        assert!(facts.data.contains_key("humidity"));
    }
}

//! Stream Beta Node - Multi-Stream Join Processing
//!
//! Handles joins between multiple stream patterns in RETE network.
//! Correlates events from different streams based on join conditions.
//!
//! Supports:
//! - Two-stream joins: moisture && temp
//! - Three+ stream joins: moisture && temp && weather (nested beta nodes)
//!
//! Example:
//! ```grl
//! moisture: MoistureSensor from stream("moisture-sensors") over window(5 min, sliding) &&
//! temp: TemperatureSensor from stream("temperature-sensors") over window(5 min, sliding) &&
//! moisture.zone_id == temp.zone_id
//! ```

use crate::rete::stream_alpha_node::StreamAlphaNode;
use crate::streaming::event::StreamEvent;
use crate::types::Value;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

/// Input to a beta node - can be either an alpha node or another beta node
/// This enables nested beta nodes for 3+ stream joins
#[derive(Debug, Clone)]
pub enum BetaInput {
    /// Direct alpha node (single stream)
    Alpha(Arc<Mutex<StreamAlphaNode>>),
    /// Nested beta node (already joined streams)
    Beta(Arc<Mutex<StreamBetaNode>>),
}

/// Result of multi-stream join (supports 2+ streams)
#[derive(Debug, Clone)]
pub struct MultiStreamJoinResult {
    /// All events that participated in the join (ordered)
    pub events: Vec<StreamEvent>,
    /// Timestamp when join was completed
    pub join_timestamp: SystemTime,
}

impl MultiStreamJoinResult {
    /// Create from two events (basic 2-stream join)
    pub fn from_two_events(left: StreamEvent, right: StreamEvent, timestamp: SystemTime) -> Self {
        Self {
            events: vec![left, right],
            join_timestamp: timestamp,
        }
    }

    /// Create from existing result + new event (nested join)
    pub fn from_result_and_event(result: MultiStreamJoinResult, event: StreamEvent, timestamp: SystemTime) -> Self {
        let mut events = result.events;
        events.push(event);
        Self {
            events,
            join_timestamp: timestamp,
        }
    }

    /// Get event by index
    pub fn get_event(&self, index: usize) -> Option<&StreamEvent> {
        self.events.get(index)
    }

    /// Get first event (for backward compatibility)
    pub fn left_event(&self) -> &StreamEvent {
        &self.events[0]
    }

    /// Get second event (for backward compatibility)
    pub fn right_event(&self) -> &StreamEvent {
        &self.events[1]
    }
}

/// Join condition between two streams
#[derive(Debug, Clone)]
pub struct JoinCondition {
    /// Field from left stream (e.g., "zone_id")
    pub left_field: String,
    /// Field from right stream (e.g., "zone_id")
    pub right_field: String,
    /// Join operator (currently only "==" supported)
    pub operator: JoinOperator,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JoinOperator {
    Equal,
    // Future: NotEqual, GreaterThan, etc.
}

/// Join strategy
#[derive(Debug, Clone)]
pub enum JoinStrategy {
    /// Join within time window
    TimeWindow { duration: Duration },
    /// Join on exact timestamp match
    ExactTimestamp,
}

/// Beta node for joining two streams (or stream + joined result)
/// Supports nested joins for 3+ stream correlation
#[derive(Debug)]
pub struct StreamBetaNode {
    /// Name for debugging
    pub name: String,
    /// Left input (alpha node or nested beta node)
    pub left_input: BetaInput,
    /// Right input (alpha node or nested beta node)
    pub right_input: BetaInput,
    /// Join conditions (e.g., zone_id == zone_id)
    pub join_conditions: Vec<JoinCondition>,
    /// Join strategy
    pub strategy: JoinStrategy,
    /// Buffered results from left input (wrapped in MultiStreamJoinResult)
    left_buffer: Vec<(SystemTime, MultiStreamJoinResult)>,
    /// Buffered results from right input (wrapped in MultiStreamJoinResult)
    right_buffer: Vec<(SystemTime, MultiStreamJoinResult)>,
}

/// Result of a successful join
#[derive(Debug, Clone)]
pub struct JoinedStreamEvent {
    pub left_event: StreamEvent,
    pub right_event: StreamEvent,
    pub join_timestamp: SystemTime,
}

impl StreamBetaNode {
    /// Create a new beta node for stream join
    pub fn new(
        name: String,
        left_input: BetaInput,
        right_input: BetaInput,
        join_conditions: Vec<JoinCondition>,
        strategy: JoinStrategy,
    ) -> Self {
        Self {
            name,
            left_input,
            right_input,
            join_conditions,
            strategy,
            left_buffer: Vec::new(),
            right_buffer: Vec::new(),
        }
    }

    /// Create beta node from two alpha nodes (simple 2-stream join)
    pub fn from_alpha_nodes(
        name: String,
        left_alpha: Arc<Mutex<StreamAlphaNode>>,
        right_alpha: Arc<Mutex<StreamAlphaNode>>,
        join_conditions: Vec<JoinCondition>,
        strategy: JoinStrategy,
    ) -> Self {
        Self::new(
            name,
            BetaInput::Alpha(left_alpha),
            BetaInput::Alpha(right_alpha),
            join_conditions,
            strategy,
        )
    }

    /// Create beta node for nested join (beta + alpha)
    pub fn from_beta_and_alpha(
        name: String,
        left_beta: Arc<Mutex<StreamBetaNode>>,
        right_alpha: Arc<Mutex<StreamAlphaNode>>,
        join_conditions: Vec<JoinCondition>,
        strategy: JoinStrategy,
    ) -> Self {
        Self::new(
            name,
            BetaInput::Beta(left_beta),
            BetaInput::Alpha(right_alpha),
            join_conditions,
            strategy,
        )
    }

    /// Process event from left input (wrap in MultiStreamJoinResult)
    pub fn process_left_event(&mut self, event: StreamEvent) -> Vec<MultiStreamJoinResult> {
        let now = SystemTime::now();
        let wrapped = MultiStreamJoinResult {
            events: vec![event],
            join_timestamp: now,
        };
        self.process_left_result(wrapped)
    }

    /// Process event from right input (wrap in MultiStreamJoinResult)
    pub fn process_right_event(&mut self, event: StreamEvent) -> Vec<MultiStreamJoinResult> {
        let now = SystemTime::now();
        let wrapped = MultiStreamJoinResult {
            events: vec![event],
            join_timestamp: now,
        };
        self.process_right_result(wrapped)
    }

    /// Process join result from left input (for nested beta nodes)
    pub fn process_left_result(&mut self, result: MultiStreamJoinResult) -> Vec<MultiStreamJoinResult> {
        let now = SystemTime::now();

        // Add to left buffer
        self.left_buffer.push((now, result.clone()));

        // Clean old results based on strategy
        self.cleanup_buffers(now);

        // Try to find matching results in right buffer
        self.find_matches(&result, &self.right_buffer, true)
    }

    /// Process join result from right input (for nested beta nodes)
    pub fn process_right_result(&mut self, result: MultiStreamJoinResult) -> Vec<MultiStreamJoinResult> {
        let now = SystemTime::now();

        // Add to right buffer
        self.right_buffer.push((now, result.clone()));

        // Clean old results based on strategy
        self.cleanup_buffers(now);

        // Try to find matching results in left buffer
        self.find_matches(&result, &self.left_buffer, false)
    }

    /// Find matching results for join
    fn find_matches(
        &self,
        new_result: &MultiStreamJoinResult,
        other_buffer: &[(SystemTime, MultiStreamJoinResult)],
        is_left: bool,
    ) -> Vec<MultiStreamJoinResult> {
        let mut matches = Vec::new();

        for (timestamp, buffered_result) in other_buffer {
            // Check if results satisfy all join conditions
            if self.check_join_conditions_multi(new_result, buffered_result, is_left) {
                // Combine the two results
                let joined = if is_left {
                    // new_result (left) + buffered_result (right)
                    self.combine_results(new_result.clone(), buffered_result.clone(), *timestamp)
                } else {
                    // buffered_result (left) + new_result (right)
                    self.combine_results(buffered_result.clone(), new_result.clone(), *timestamp)
                };
                matches.push(joined);
            }
        }

        matches
    }

    /// Combine two MultiStreamJoinResults into one
    fn combine_results(
        &self,
        left: MultiStreamJoinResult,
        right: MultiStreamJoinResult,
        timestamp: SystemTime,
    ) -> MultiStreamJoinResult {
        let mut all_events = left.events;
        all_events.extend(right.events);
        MultiStreamJoinResult {
            events: all_events,
            join_timestamp: timestamp,
        }
    }

    /// Check if two MultiStreamJoinResults satisfy all join conditions
    /// For nested joins, compares the LAST event in left result with FIRST event in right result
    fn check_join_conditions_multi(
        &self,
        left_result: &MultiStreamJoinResult,
        right_result: &MultiStreamJoinResult,
        is_left: bool,
    ) -> bool {
        // Get the events to compare
        // For left: use last event (most recently joined)
        // For right: use first event (typically a single new event)
        let left_event = left_result.events.last().unwrap();
        let right_event = right_result.events.first().unwrap();

        for condition in &self.join_conditions {
            let (left_field, right_field) = if is_left {
                (&condition.left_field, &condition.right_field)
            } else {
                (&condition.right_field, &condition.left_field)
            };

            let left_value = Self::extract_field_value(left_event, left_field);
            let right_value = Self::extract_field_value(right_event, right_field);

            match condition.operator {
                JoinOperator::Equal => {
                    if left_value != right_value {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Extract field value from event
    fn extract_field_value(event: &StreamEvent, field: &str) -> Option<String> {
        event.data.get(field).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            Value::Integer(i) => Some(i.to_string()),
            Value::Number(n) => Some(n.to_string()),
            _ => None,
        })
    }

    /// Clean up old events from buffers based on strategy
    fn cleanup_buffers(&mut self, now: SystemTime) {
        match &self.strategy {
            JoinStrategy::TimeWindow { duration } => {
                let cutoff = now
                    .checked_sub(*duration)
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                self.left_buffer.retain(|(ts, _)| *ts >= cutoff);
                self.right_buffer.retain(|(ts, _)| *ts >= cutoff);
            }
            JoinStrategy::ExactTimestamp => {
                // For exact timestamp, we can be more aggressive
                // Keep only recent events (e.g., last 100)
                const MAX_BUFFER_SIZE: usize = 100;
                if self.left_buffer.len() > MAX_BUFFER_SIZE {
                    self.left_buffer
                        .drain(0..self.left_buffer.len() - MAX_BUFFER_SIZE);
                }
                if self.right_buffer.len() > MAX_BUFFER_SIZE {
                    self.right_buffer
                        .drain(0..self.right_buffer.len() - MAX_BUFFER_SIZE);
                }
            }
        }
    }

    /// Get statistics about buffer sizes
    pub fn get_stats(&self) -> BetaNodeStats {
        BetaNodeStats {
            left_buffer_size: self.left_buffer.len(),
            right_buffer_size: self.right_buffer.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BetaNodeStats {
    pub left_buffer_size: usize,
    pub right_buffer_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rete::stream_alpha_node::WindowSpec;
    use crate::streaming::window::WindowType;
    use std::collections::HashMap;

    #[test]
    fn test_stream_beta_node_join() {
        // Create alpha nodes
        let left_alpha = Arc::new(Mutex::new(StreamAlphaNode::new(
            "moisture-sensors",
            Some("MoistureSensor".to_string()),
            Some(WindowSpec {
                duration: Duration::from_secs(300),
                window_type: WindowType::Sliding,
            }),
        )));

        let right_alpha = Arc::new(Mutex::new(StreamAlphaNode::new(
            "temperature-sensors",
            Some("TemperatureSensor".to_string()),
            Some(WindowSpec {
                duration: Duration::from_secs(300),
                window_type: WindowType::Sliding,
            }),
        )));

        // Create beta node with join condition using from_alpha_nodes
        let mut beta = StreamBetaNode::from_alpha_nodes(
            "irrigation_join".to_string(),
            left_alpha,
            right_alpha,
            vec![JoinCondition {
                left_field: "zone_id".to_string(),
                right_field: "zone_id".to_string(),
                operator: JoinOperator::Equal,
            }],
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(300),
            },
        );

        // Create test events
        let mut moisture_data = HashMap::new();
        moisture_data.insert("zone_id".to_string(), Value::String("zone_1".to_string()));
        moisture_data.insert("moisture_level".to_string(), Value::Number(25.5));

        use crate::streaming::event::EventMetadata;

        let moisture_event = StreamEvent {
            id: "m1".to_string(),
            event_type: "MoistureSensor".to_string(),
            data: moisture_data,
            metadata: EventMetadata {
                timestamp: 1000,
                source: "sensor-1".to_string(),
                sequence: 1,
                tags: HashMap::new(),
            },
        };

        let mut temp_data = HashMap::new();
        temp_data.insert("zone_id".to_string(), Value::String("zone_1".to_string()));
        temp_data.insert("temperature".to_string(), Value::Number(35.0));

        let temp_event = StreamEvent {
            id: "t1".to_string(),
            event_type: "TemperatureSensor".to_string(),
            data: temp_data,
            metadata: EventMetadata {
                timestamp: 1100,
                source: "sensor-2".to_string(),
                sequence: 2,
                tags: HashMap::new(),
            },
        };

        // Process events - now returns MultiStreamJoinResult
        let left_matches = beta.process_left_event(moisture_event);
        assert_eq!(left_matches.len(), 0); // No match yet

        let right_matches = beta.process_right_event(temp_event);
        assert_eq!(right_matches.len(), 1); // Should match!

        // Verify the joined result
        let joined = &right_matches[0];
        assert_eq!(joined.events.len(), 2); // Two events joined
        assert_eq!(
            joined.events[0].data.get("zone_id").unwrap(),
            &Value::String("zone_1".to_string())
        );
        assert_eq!(
            joined.events[1].data.get("zone_id").unwrap(),
            &Value::String("zone_1".to_string())
        );
    }

    #[test]
    fn test_nested_beta_three_stream_join() {
        use crate::streaming::event::EventMetadata;

        // Create alpha nodes for 3 streams
        let moisture_alpha = Arc::new(Mutex::new(StreamAlphaNode::new(
            "moisture-sensors",
            Some("MoistureSensor".to_string()),
            Some(WindowSpec {
                duration: Duration::from_secs(300),
                window_type: WindowType::Sliding,
            }),
        )));

        let temp_alpha = Arc::new(Mutex::new(StreamAlphaNode::new(
            "temperature-sensors",
            Some("TemperatureSensor".to_string()),
            Some(WindowSpec {
                duration: Duration::from_secs(300),
                window_type: WindowType::Sliding,
            }),
        )));

        let weather_alpha = Arc::new(Mutex::new(StreamAlphaNode::new(
            "weather-events",
            Some("WeatherEvent".to_string()),
            Some(WindowSpec {
                duration: Duration::from_secs(300),
                window_type: WindowType::Sliding,
            }),
        )));

        // Create Beta1: moisture + temp
        let beta1 = Arc::new(Mutex::new(StreamBetaNode::from_alpha_nodes(
            "moisture_temp_join".to_string(),
            moisture_alpha,
            temp_alpha,
            vec![JoinCondition {
                left_field: "zone_id".to_string(),
                right_field: "zone_id".to_string(),
                operator: JoinOperator::Equal,
            }],
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(300),
            },
        )));

        // Create Beta2: (moisture+temp) + weather
        let mut beta2 = StreamBetaNode::from_beta_and_alpha(
            "full_join".to_string(),
            beta1.clone(),
            weather_alpha,
            vec![JoinCondition {
                left_field: "zone_id".to_string(), // from temp (last in beta1)
                right_field: "zone_id".to_string(), // from weather
                operator: JoinOperator::Equal,
            }],
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(300),
            },
        );

        // Create test events
        let mut moisture_data = HashMap::new();
        moisture_data.insert("zone_id".to_string(), Value::String("zone_1".to_string()));
        moisture_data.insert("moisture_level".to_string(), Value::Number(20.0));

        let moisture_event = StreamEvent {
            id: "m1".to_string(),
            event_type: "MoistureSensor".to_string(),
            data: moisture_data,
            metadata: EventMetadata {
                timestamp: 1000,
                source: "sensor-1".to_string(),
                sequence: 1,
                tags: HashMap::new(),
            },
        };

        let mut temp_data = HashMap::new();
        temp_data.insert("zone_id".to_string(), Value::String("zone_1".to_string()));
        temp_data.insert("temperature".to_string(), Value::Number(35.0));

        let temp_event = StreamEvent {
            id: "t1".to_string(),
            event_type: "TemperatureSensor".to_string(),
            data: temp_data,
            metadata: EventMetadata {
                timestamp: 1100,
                source: "sensor-2".to_string(),
                sequence: 2,
                tags: HashMap::new(),
            },
        };

        let mut weather_data = HashMap::new();
        weather_data.insert("zone_id".to_string(), Value::String("zone_1".to_string()));
        weather_data.insert("condition".to_string(), Value::String("sunny".to_string()));

        let weather_event = StreamEvent {
            id: "w1".to_string(),
            event_type: "WeatherEvent".to_string(),
            data: weather_data,
            metadata: EventMetadata {
                timestamp: 1200,
                source: "weather-1".to_string(),
                sequence: 3,
                tags: HashMap::new(),
            },
        };

        // Step 1: Join moisture + temp in beta1
        let beta1_result = {
            let mut b1 = beta1.lock().unwrap();
            b1.process_left_event(moisture_event);
            b1.process_right_event(temp_event)
        };

        assert_eq!(beta1_result.len(), 1); // moisture + temp matched
        assert_eq!(beta1_result[0].events.len(), 2); // Two events

        // Step 2: Pass beta1 result to beta2, then add weather
        let beta2_left_result = beta2.process_left_result(beta1_result[0].clone());
        assert_eq!(beta2_left_result.len(), 0); // No match yet (no weather)

        let beta2_final_result = beta2.process_right_event(weather_event);
        assert_eq!(beta2_final_result.len(), 1); // All 3 streams matched!

        // Verify final result contains all 3 events
        let final_joined = &beta2_final_result[0];
        assert_eq!(final_joined.events.len(), 3); // moisture + temp + weather
        assert_eq!(final_joined.events[0].event_type, "MoistureSensor");
        assert_eq!(final_joined.events[1].event_type, "TemperatureSensor");
        assert_eq!(final_joined.events[2].event_type, "WeatherEvent");

        // All should have same zone_id
        for event in &final_joined.events {
            assert_eq!(
                event.data.get("zone_id").unwrap(),
                &Value::String("zone_1".to_string())
            );
        }

        println!("✅ 3-Stream Join Success!");
        println!("   Events: {} + {} + {}",
            final_joined.events[0].event_type,
            final_joined.events[1].event_type,
            final_joined.events[2].event_type
        );
    }
}

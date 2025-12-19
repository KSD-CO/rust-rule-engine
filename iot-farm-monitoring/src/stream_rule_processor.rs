// Stream Rule Processor - Multi-Stream Join Support
// Processes stream events using GRL stream pattern rules with Beta Nodes
//
// This version supports:
// - Single-stream rules (via Alpha nodes)
// - Multi-stream joins (via Beta nodes)
// - Nested joins (3+ streams)

use log::info;
use rust_rule_engine::engine::rule::{ConditionExpression, ConditionGroup};
use rust_rule_engine::types::Operator;
use rust_rule_engine::parser::GRLParser;
use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
use rust_rule_engine::rete::stream_beta_node::{
    JoinCondition, JoinOperator, JoinStrategy, MultiStreamJoinResult, StreamBetaNode,
};
use rust_rule_engine::streaming::event::StreamEvent;
use rust_rule_engine::types::ActionType;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crate::monitor::MonitorStats;
use crate::working_memory::WorkingMemory;

/// Processes stream events using GRL-defined stream patterns with Beta nodes
/// 
/// Architecture: Alpha Nodes → Working Memory → Beta Nodes → Rule Evaluation → Actions
pub struct StreamRuleProcessor {
    /// Alpha nodes by stream name (filters)
    alpha_nodes: HashMap<String, Arc<Mutex<StreamAlphaNode>>>,
    /// Working Memory (central fact storage)
    working_memory: Arc<Mutex<WorkingMemory>>,
    /// Beta nodes for multi-stream joins
    beta_nodes: Vec<BetaNodeConfig>,
    /// Single-stream rules (no joins)
    single_stream_rules: Vec<SingleStreamRule>,
    /// Statistics for tracking rule fires
    stats: Option<Arc<Mutex<MonitorStats>>>,
}

/// Configuration for a beta node with its rule
struct BetaNodeConfig {
    rule_name: String,
    beta_node: Arc<Mutex<StreamBetaNode>>,
    left_stream: String,
    right_stream: String,
    actions: Vec<ActionType>,
    salience: i32,
    /// Filter conditions to evaluate after join (e.g., moisture_level < 25.0)
    filter_conditions: Vec<FilterCondition>,
}

/// A filter condition to evaluate on joined events
#[derive(Clone, Debug)]
struct FilterCondition {
    field: String,          // e.g., "moisture_level" or "temperature"
    operator: Operator,     // e.g., LessThan, GreaterThan
    value: rust_rule_engine::types::Value,
}

/// Single-stream rule (no join needed)
struct SingleStreamRule {
    name: String,
    stream_name: String,
    actions: Vec<ActionType>,
    salience: i32,
}

impl StreamRuleProcessor {
    /// Create a new processor from GRL file
    pub fn from_grl_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_grl_string(&content)
    }

    /// Create from GRL string
    pub fn from_grl_string(grl: &str) -> anyhow::Result<Self> {
        let rules = GRLParser::parse_rules(grl)?;
        info!("📋 Parsed {} GRL rules", rules.len());

        let mut alpha_nodes: HashMap<String, Arc<Mutex<StreamAlphaNode>>> = HashMap::new();
        let mut beta_nodes = Vec::new();
        let mut single_stream_rules = Vec::new();

        for rule in rules {
            // Analyze rule conditions to determine if it's single or multi-stream
            let stream_patterns = Self::extract_all_stream_patterns(&rule.conditions);

            if stream_patterns.is_empty() {
                info!("  ⏭️  Skipping rule '{}' - no stream patterns", rule.name);
                continue;
            }

            if stream_patterns.len() == 1 {
                // Single-stream rule
                let (stream_name, window) = &stream_patterns[0];
                
                // Create alpha node if not exists
                if !alpha_nodes.contains_key(stream_name) {
                    let node = StreamAlphaNode::new(stream_name, None, window.clone());
                    alpha_nodes.insert(stream_name.clone(), Arc::new(Mutex::new(node)));
                    info!("  ✓ Created alpha node for '{}'", stream_name);
                }

                single_stream_rules.push(SingleStreamRule {
                    name: rule.name.clone(),
                    stream_name: stream_name.clone(),
                    actions: rule.actions.clone(),
                    salience: rule.salience,
                });
                info!("  ✓ Registered single-stream rule '{}'", rule.name);
            } else {
                // Multi-stream join rule - need beta node
                info!(
                    "  🔗 Creating beta node for rule '{}' with {} streams",
                    rule.name,
                    stream_patterns.len()
                );

                // Create alpha nodes for all streams
                for (stream_name, window) in &stream_patterns {
                    if !alpha_nodes.contains_key(stream_name) {
                        let node = StreamAlphaNode::new(stream_name, None, window.clone());
                        alpha_nodes.insert(stream_name.clone(), Arc::new(Mutex::new(node)));
                        info!("    ✓ Created alpha node for '{}'", stream_name);
                    }
                }

                // Extract join conditions
                let join_conditions = Self::extract_join_conditions(&rule.conditions);
                info!("    ✓ Extracted {} join conditions", join_conditions.len());

                // Create beta node for 2-stream join
                if stream_patterns.len() == 2 {
                    let left_alpha = alpha_nodes.get(&stream_patterns[0].0).unwrap().clone();
                    let right_alpha = alpha_nodes.get(&stream_patterns[1].0).unwrap().clone();

                    let window_duration = stream_patterns[0]
                        .1
                        .as_ref()
                        .map(|w| w.duration)
                        .unwrap_or(Duration::from_secs(300));

                    let beta_node = StreamBetaNode::from_alpha_nodes(
                        rule.name.clone(),
                        left_alpha,
                        right_alpha,
                        join_conditions.clone(),
                        JoinStrategy::TimeWindow {
                            duration: window_duration,
                        },
                    );

                    // Extract filter conditions (e.g., moisture_level < 25.0)
                    let filter_conditions = Self::extract_filter_conditions(&rule.conditions);
                    info!("    ✓ Extracted {} filter conditions", filter_conditions.len());

                    beta_nodes.push(BetaNodeConfig {
                        rule_name: rule.name.clone(),
                        beta_node: Arc::new(Mutex::new(beta_node)),
                        left_stream: stream_patterns[0].0.clone(),
                        right_stream: stream_patterns[1].0.clone(),
                        actions: rule.actions.clone(),
                        salience: rule.salience,
                        filter_conditions,
                    });

                    info!("    ✅ Created 2-stream beta node for '{}'", rule.name);
                } else if stream_patterns.len() == 3 {
                    // 3-stream join: (stream1 + stream2) + stream3
                    let alpha1 = alpha_nodes.get(&stream_patterns[0].0).unwrap().clone();
                    let alpha2 = alpha_nodes.get(&stream_patterns[1].0).unwrap().clone();
                    let alpha3 = alpha_nodes.get(&stream_patterns[2].0).unwrap().clone();

                    let window_duration = stream_patterns[0]
                        .1
                        .as_ref()
                        .map(|w| w.duration)
                        .unwrap_or(Duration::from_secs(300));

                    // First beta: alpha1 + alpha2
                    let beta1 = Arc::new(Mutex::new(StreamBetaNode::from_alpha_nodes(
                        format!("{}_beta1", rule.name),
                        alpha1,
                        alpha2,
                        join_conditions.clone(),
                        JoinStrategy::TimeWindow {
                            duration: window_duration,
                        },
                    )));

                    // Second beta: beta1 + alpha3
                    let beta2 = StreamBetaNode::from_beta_and_alpha(
                        format!("{}_beta2", rule.name),
                        beta1,
                        alpha3,
                        join_conditions,
                        JoinStrategy::TimeWindow {
                            duration: window_duration,
                        },
                    );

                    // Extract filter conditions for 3-stream join
                    let filter_conditions = Self::extract_filter_conditions(&rule.conditions);
                    info!("    ✓ Extracted {} filter conditions", filter_conditions.len());

                    beta_nodes.push(BetaNodeConfig {
                        rule_name: rule.name.clone(),
                        beta_node: Arc::new(Mutex::new(beta2)),
                        left_stream: stream_patterns[0].0.clone(), // Actually nested
                        right_stream: stream_patterns[2].0.clone(),
                        actions: rule.actions.clone(),
                        salience: rule.salience,
                        filter_conditions,
                    });

                    info!("    ✅ Created 3-stream nested beta node for '{}'", rule.name);
                } else {
                    info!(
                        "    ⚠️  Skipping rule '{}' - {} streams not yet supported",
                        rule.name,
                        stream_patterns.len()
                    );
                }
            }
        }

        info!("✅ Created {} alpha nodes", alpha_nodes.len());
        info!("✅ Created {} beta nodes (multi-stream)", beta_nodes.len());
        info!("✅ Created {} single-stream rules", single_stream_rules.len());

        // Initialize Working Memory with references to Alpha nodes
        let mut working_memory = WorkingMemory::new();
        for (stream_name, alpha_node) in &alpha_nodes {
            working_memory.register_alpha_node(stream_name.clone(), alpha_node.clone());
        }
        info!("✅ Initialized Working Memory with {} streams", alpha_nodes.len());

        Ok(Self {
            alpha_nodes,
            working_memory: Arc::new(Mutex::new(working_memory)),
            beta_nodes,
            single_stream_rules,
            stats: None,
        })
    }

    /// Set statistics tracker
    pub fn set_stats(&mut self, stats: Arc<Mutex<MonitorStats>>) {
        self.stats = Some(stats);
    }

    /// Get Working Memory statistics
    pub fn working_memory_stats(&self) -> crate::working_memory::WorkingMemoryStats {
        self.working_memory.lock().unwrap().stats()
    }

    /// Get reference to Working Memory (for advanced usage)
    pub fn working_memory(&self) -> Arc<Mutex<WorkingMemory>> {
        self.working_memory.clone()
    }

    /// Process an event from a stream
    /// 
    /// Flow: Event → Alpha Node (filter) → Working Memory (store) → Beta Nodes (join)
    pub fn process_event(&mut self, stream_name: &str, event: &StreamEvent) -> bool {
        // debug!("🔵 process_event: stream='{}', event_type='{}'", stream_name, event.event_type);

        let Some(alpha_node) = self.alpha_nodes.get(stream_name) else {
            // debug!("  ❌ No alpha node found for stream '{}'", stream_name);
            return false;
        };

        // Process through alpha node first
        let mut alpha_lock = alpha_node.lock().unwrap();
        let in_window = alpha_lock.process_event(event);
        drop(alpha_lock);

        if !in_window {
            // debug!("  ❌ Event rejected: not in time window (timestamp: {})", event.metadata.timestamp);
            return false;
        }

        // debug!("  ✅ Event accepted by alpha node");

        let mut matched = false;

        // Check single-stream rules for this stream
        for rule in &self.single_stream_rules {
            if rule.stream_name == stream_name {
                Self::execute_actions(&rule.actions, event);
                matched = true;
            }
        }

        // Process through beta nodes (multi-stream joins)
        for beta_config in &mut self.beta_nodes {
            let mut beta_lock = beta_config.beta_node.lock().unwrap();

            // Determine if event is from left or right stream
            let results = if stream_name == beta_config.left_stream {
                // debug!("  → Processing LEFT event for rule '{}' from stream '{}'",
                //     beta_config.rule_name, stream_name);
                beta_lock.process_left_event(event.clone())
            } else if stream_name == beta_config.right_stream {
                // debug!("  → Processing RIGHT event for rule '{}' from stream '{}'",
                //     beta_config.rule_name, stream_name);
                beta_lock.process_right_event(event.clone())
            } else {
                continue;
            };

            // debug!("  ← Got {} join results from rule '{}'", results.len(), beta_config.rule_name);

            // Execute actions for each successful join
            for join_result in results {
                // Only log successful matches, not every join
                // debug!("🔗 JOIN SUCCESS: Rule '{}' matched {} events",
                //     beta_config.rule_name, join_result.events.len());

                // Evaluate filter conditions before executing actions
                if !beta_config.filter_conditions.is_empty() {
                    let passed = Self::evaluate_filter_conditions(
                        &beta_config.filter_conditions,
                        &join_result,
                    );

                    if !passed {
                        // debug!("  ❌ Filter conditions FAILED for rule '{}'",
                        //     beta_config.rule_name);
                        continue; // Skip this join result
                    }

                    // Log only successful rule fires
                    info!("✅ Rule '{}' FIRED", beta_config.rule_name);
                    
                    // Update statistics based on rule name
                    if let Some(stats) = &self.stats {
                        let mut s = stats.lock().unwrap();
                        match beta_config.rule_name.as_str() {
                            "CriticalIrrigationNeeded" => s.irrigation_triggered += 1,
                            "FrostAlert" => s.frost_alerts += 1,
                            "IrrigationEfficiency" => s.efficiency_reports += 1,
                            "DroughtStress" | "ExtremeWeatherIrrigation" => s.irrigation_triggered += 1,
                            _ => {}
                        }
                    }
                }

                Self::execute_join_actions(&beta_config.actions, &join_result);
                matched = true;
            }
        }

        matched
    }

    // === Helper Methods ===

    /// Extract all stream patterns from condition tree
    fn extract_all_stream_patterns(cg: &ConditionGroup) -> Vec<(String, Option<WindowSpec>)> {
        let mut patterns = Vec::new();
        Self::collect_stream_patterns(cg, &mut patterns);
        patterns
    }

    fn collect_stream_patterns(
        cg: &ConditionGroup,
        patterns: &mut Vec<(String, Option<WindowSpec>)>,
    ) {
        match cg {
            ConditionGroup::StreamPattern {
                stream_name,
                window,
                ..
            } => {
                let window_spec = window.as_ref().map(|w| WindowSpec {
                    duration: w.duration,
                    window_type: match &w.window_type {
                        rust_rule_engine::engine::rule::StreamWindowType::Sliding => {
                            rust_rule_engine::streaming::window::WindowType::Sliding
                        }
                        rust_rule_engine::engine::rule::StreamWindowType::Tumbling => {
                            rust_rule_engine::streaming::window::WindowType::Tumbling
                        }
                        rust_rule_engine::engine::rule::StreamWindowType::Session { timeout } => {
                            rust_rule_engine::streaming::window::WindowType::Session {
                                timeout: *timeout,
                            }
                        }
                    },
                });
                patterns.push((stream_name.clone(), window_spec));
            }
            ConditionGroup::Compound { left, right, .. } => {
                Self::collect_stream_patterns(left, patterns);
                Self::collect_stream_patterns(right, patterns);
            }
            _ => {}
        }
    }

    /// Extract join conditions (e.g., moisture.zone_id == temp.zone_id)
    fn extract_join_conditions(cg: &ConditionGroup) -> Vec<JoinCondition> {
        let mut conditions = Vec::new();
        Self::collect_join_conditions(cg, &mut conditions);

        info!("🔍 Extracted {} join conditions", conditions.len());
        
        // If no explicit join conditions found, try to infer common field (zone_id)
        if conditions.is_empty() {
            info!("⚠️  No explicit join conditions, using default: zone_id == zone_id");
            conditions.push(JoinCondition {
                left_field: "zone_id".to_string(),
                right_field: "zone_id".to_string(),
                operator: JoinOperator::Equal,
            });
        }

        conditions
    }

    fn collect_join_conditions(cg: &ConditionGroup, conditions: &mut Vec<JoinCondition>) {
        match cg {
            ConditionGroup::Single(condition) => {
                // Check if this condition is a join condition (field == field with dots in both)
                if let (ConditionExpression::Field(field), Operator::Equal) =
                    (&condition.expression, &condition.operator) {
                    // Check if value is also a field reference
                    if let rust_rule_engine::types::Value::String(val_str) = &condition.value {
                        // If both field and value contain dots, it's likely a join condition
                        if field.contains('.') && val_str.contains('.') {
                            // Strip variable prefix (e.g., "moisture.zone_id" -> "zone_id")
                            let left_clean = field.split('.').nth(1).unwrap_or(field);
                            let right_clean = val_str.split('.').nth(1).unwrap_or(val_str);

                            info!("  ✓ Found join condition: {} == {} (cleaned: {} == {})",
                                field, val_str, left_clean, right_clean);
                            conditions.push(JoinCondition {
                                left_field: left_clean.to_string(),
                                right_field: right_clean.to_string(),
                                operator: JoinOperator::Equal,
                            });
                        }
                    }
                }
            }
            ConditionGroup::Compound { left, right, .. } => {
                Self::collect_join_conditions(left, conditions);
                Self::collect_join_conditions(right, conditions);
            }
            _ => {}
        }
    }

    /// Extract filter conditions (e.g., moisture.moisture_level < 25.0)
    fn extract_filter_conditions(cg: &ConditionGroup) -> Vec<FilterCondition> {
        let mut conditions = Vec::new();
        Self::collect_filter_conditions(cg, &mut conditions);
        conditions
    }

    fn collect_filter_conditions(cg: &ConditionGroup, conditions: &mut Vec<FilterCondition>) {
        match cg {
            ConditionGroup::Single(condition) => {
                if let ConditionExpression::Field(field) = &condition.expression {
                    // Filter conditions have a dot (e.g., "moisture.moisture_level")
                    // but the value is a literal (not another field)
                    if field.contains('.') {
                        // Check if value is a literal (not a field reference for join)
                        let is_literal = match &condition.value {
                            rust_rule_engine::types::Value::String(s) => !s.contains('.'),
                            _ => true, // Numbers, bools, etc are literals
                        };

                        if is_literal {
                            // Extract the field name (e.g., "moisture.moisture_level" -> "moisture_level")
                            let field_name = field.split('.').nth(1).unwrap_or(field);

                            info!("  ✓ Found filter condition: {} {:?} {:?}",
                                field, condition.operator, condition.value);

                            conditions.push(FilterCondition {
                                field: field_name.to_string(),
                                operator: condition.operator.clone(),
                                value: condition.value.clone(),
                            });
                        }
                    }
                }
            }
            ConditionGroup::Compound { left, right, .. } => {
                Self::collect_filter_conditions(left, conditions);
                Self::collect_filter_conditions(right, conditions);
            }
            _ => {}
        }
    }

    /// Evaluate filter conditions against joined events
    fn evaluate_filter_conditions(
        filter_conditions: &[FilterCondition],
        join_result: &MultiStreamJoinResult,
    ) -> bool {
        // Merge all event data into a single map for evaluation
        let mut merged_data = HashMap::new();
        for event in &join_result.events {
            for (key, value) in &event.data {
                merged_data.insert(key.clone(), value.clone());
            }
        }

        // Check all filter conditions
        for filter in filter_conditions {
            let Some(event_value) = merged_data.get(&filter.field) else {
                // debug!("  ⚠️  Field '{}' not found in event data. Available fields: {:?}", 
                //     filter.field, merged_data.keys().collect::<Vec<_>>());
                return false;
            };

            // Convert event value (which might be a String) to comparable value
            let parsed_value = Self::parse_value(event_value);

            // Evaluate the condition
            let matches = Self::evaluate_condition(&parsed_value, &filter.operator, &filter.value);

            // Only log failed conditions for debugging
            // debug!("  🔍 Filter check: {} = {:?}, {:?} {:?} = {}",
            //     filter.field, parsed_value, filter.operator, filter.value, matches);

            if !matches {
                return false;
            }
        }

        true
    }

    /// Parse a value (handles String -> Number conversion if needed)
    fn parse_value(value: &rust_rule_engine::types::Value) -> rust_rule_engine::types::Value {
        use rust_rule_engine::types::Value;

        match value {
            Value::String(s) => {
                // Try to parse as f64 first
                if let Ok(f) = s.parse::<f64>() {
                    Value::Number(f)
                } else if let Ok(i) = s.parse::<i64>() {
                    Value::Integer(i)
                } else {
                    value.clone()
                }
            }
            _ => value.clone(),
        }
    }

    /// Evaluate a single condition (value operator expected_value)
    fn evaluate_condition(
        value: &rust_rule_engine::types::Value,
        operator: &Operator,
        expected: &rust_rule_engine::types::Value,
    ) -> bool {
        use rust_rule_engine::types::Value;

        match (value, operator, expected) {
            // Number comparisons
            (Value::Number(v), Operator::LessThan, Value::Number(e)) => v < e,
            (Value::Number(v), Operator::LessThanOrEqual, Value::Number(e)) => v <= e,
            (Value::Number(v), Operator::GreaterThan, Value::Number(e)) => v > e,
            (Value::Number(v), Operator::GreaterThanOrEqual, Value::Number(e)) => v >= e,
            (Value::Number(v), Operator::Equal, Value::Number(e)) => (v - e).abs() < f64::EPSILON,

            // Integer comparisons
            (Value::Integer(v), Operator::LessThan, Value::Integer(e)) => v < e,
            (Value::Integer(v), Operator::LessThanOrEqual, Value::Integer(e)) => v <= e,
            (Value::Integer(v), Operator::GreaterThan, Value::Integer(e)) => v > e,
            (Value::Integer(v), Operator::GreaterThanOrEqual, Value::Integer(e)) => v >= e,
            (Value::Integer(v), Operator::Equal, Value::Integer(e)) => v == e,

            // Mixed numeric comparisons
            (Value::Number(v), Operator::LessThan, Value::Integer(e)) => v < &(*e as f64),
            (Value::Integer(v), Operator::LessThan, Value::Number(e)) => (*v as f64) < *e,
            (Value::Number(v), Operator::LessThanOrEqual, Value::Integer(e)) => v <= &(*e as f64),
            (Value::Integer(v), Operator::LessThanOrEqual, Value::Number(e)) => (*v as f64) <= *e,
            (Value::Number(v), Operator::GreaterThan, Value::Integer(e)) => v > &(*e as f64),
            (Value::Integer(v), Operator::GreaterThan, Value::Number(e)) => (*v as f64) > *e,
            (Value::Number(v), Operator::GreaterThanOrEqual, Value::Integer(e)) => v >= &(*e as f64),
            (Value::Integer(v), Operator::GreaterThanOrEqual, Value::Number(e)) => (*v as f64) >= *e,

            // String equality
            (Value::String(v), Operator::Equal, Value::String(e)) => v == e,

            _ => {
                info!("  ⚠️  Unsupported comparison: {:?} {:?} {:?}", value, operator, expected);
                false
            }
        }
    }

    /// Execute actions for single-stream match
    fn execute_actions(actions: &[ActionType], event: &StreamEvent) {
        for action in actions {
            match action {
                ActionType::Log { message, .. } => {
                    let formatted = Self::format_message(message, event);
                    info!("{}", formatted);
                }
                _ => {}
            }
        }
    }

    /// Execute actions for multi-stream join match
    fn execute_join_actions(actions: &[ActionType], join_result: &MultiStreamJoinResult) {
        for action in actions {
            match action {
                ActionType::Log { message, .. } => {
                    let formatted = Self::format_join_message(message, join_result);
                    info!("{}", formatted);
                }
                _ => {}
            }
        }
    }

    /// Format message with event data
    fn format_message(template: &str, event: &StreamEvent) -> String {
        let mut result = template.to_string();

        // Replace placeholders like {zone_id}, {moisture_level}, etc.
        for (key, value) in &event.data {
            let placeholder = format!("{{{}}}", key);
            if let Some(val_str) = value.as_string() {
                result = result.replace(&placeholder, &val_str);
            } else {
                // Try to format as number
                let val_str = format!("{:?}", value);
                result = result.replace(&placeholder, &val_str);
            }
        }

        result
    }

    /// Format message with join result data
    fn format_join_message(template: &str, join_result: &MultiStreamJoinResult) -> String {
        let mut result = template.to_string();

        // Simple approach: use first event's data
        if let Some(event) = join_result.events.first() {
            result = Self::format_message(&result, event);
        }

        // For multi-stream: try to merge data from all events
        for event in &join_result.events {
            for (key, value) in &event.data {
                let placeholder = format!("{{{}}}", key);
                if let Some(val_str) = value.as_string() {
                    result = result.replace(&placeholder, &val_str);
                } else {
                    // Try to format as number
                    let val_str = format!("{:?}", value);
                    result = result.replace(&placeholder, &val_str);
                }
            }
        }

        result
    }
}

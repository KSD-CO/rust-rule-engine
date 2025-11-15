use crate::rete::alpha::AlphaNode;
/// Chuyển ConditionGroup sang ReteUlNode
pub fn build_rete_ul_from_condition_group(group: &crate::rete::auto_network::ConditionGroup) -> ReteUlNode {
    use crate::rete::auto_network::ConditionGroup;
    match group {
        ConditionGroup::Single(cond) => {
            ReteUlNode::UlAlpha(AlphaNode {
                field: cond.field.clone(),
                operator: cond.operator.clone(),
                value: cond.value.clone(),
            })
        }
        ConditionGroup::Compound { left, operator, right } => {
            match operator.as_str() {
                "AND" => ReteUlNode::UlAnd(
                    Box::new(build_rete_ul_from_condition_group(left)),
                    Box::new(build_rete_ul_from_condition_group(right)),
                ),
                "OR" => ReteUlNode::UlOr(
                    Box::new(build_rete_ul_from_condition_group(left)),
                    Box::new(build_rete_ul_from_condition_group(right)),
                ),
                _ => ReteUlNode::UlAnd(
                    Box::new(build_rete_ul_from_condition_group(left)),
                    Box::new(build_rete_ul_from_condition_group(right)),
                ),
            }
        }
        ConditionGroup::Not(inner) => {
            ReteUlNode::UlNot(Box::new(build_rete_ul_from_condition_group(inner)))
        }
        ConditionGroup::Exists(inner) => {
            ReteUlNode::UlExists(Box::new(build_rete_ul_from_condition_group(inner)))
        }
        ConditionGroup::Forall(inner) => {
            ReteUlNode::UlForall(Box::new(build_rete_ul_from_condition_group(inner)))
        }
    }
}
use std::collections::HashMap;

/// Helper: Evaluate a condition string against facts (for accumulate)
fn evaluate_condition_string(condition: &str, facts: &HashMap<String, String>) -> bool {
    let condition = condition.trim();
    let operators = ["==", "!=", ">=", "<=", ">", "<"];

    for op in &operators {
        if let Some(pos) = condition.find(op) {
            let field = condition[..pos].trim();
            let value_str = condition[pos + op.len()..]
                .trim()
                .trim_matches('"')
                .trim_matches('\'');

            if let Some(field_value) = facts.get(field) {
                return compare_string_values(field_value, op, value_str);
            } else {
                return false;
            }
        }
    }
    false
}

/// Helper: Compare string values
fn compare_string_values(field_value: &str, operator: &str, value_str: &str) -> bool {
    // Try numeric comparison first
    if let (Ok(field_num), Ok(val_num)) = (field_value.parse::<f64>(), value_str.parse::<f64>()) {
        match operator {
            "==" => (field_num - val_num).abs() < f64::EPSILON,
            "!=" => (field_num - val_num).abs() >= f64::EPSILON,
            ">" => field_num > val_num,
            "<" => field_num < val_num,
            ">=" => field_num >= val_num,
            "<=" => field_num <= val_num,
            _ => false,
        }
    } else {
        // String comparison
        match operator {
            "==" => field_value == value_str,
            "!=" => field_value != value_str,
            _ => false,
        }
    }
}

/// Đánh giá mạng node RETE với facts
pub fn evaluate_rete_ul_node(node: &ReteUlNode, facts: &HashMap<String, String>) -> bool {
    match node {
        ReteUlNode::UlAlpha(alpha) => {
            let val = if alpha.field.contains('.') {
                let parts: Vec<&str> = alpha.field.split('.').collect();
                if parts.len() == 2 {
                    let prefix = parts[0];
                    let suffix = parts[1];
                    facts.get(&format!("{}.{}", prefix, suffix)).or_else(|| facts.get(&format!("{}:{}", prefix, suffix)))
                } else {
                    facts.get(&alpha.field)
                }
            } else {
                facts.get(&alpha.field)
            };
            if let Some(val) = val {
                match alpha.operator.as_str() {
                    "==" => val == &alpha.value,
                    "!=" => val != &alpha.value,
                    ">" => val.parse::<f64>().unwrap_or(0.0) > alpha.value.parse::<f64>().unwrap_or(0.0),
                    "<" => val.parse::<f64>().unwrap_or(0.0) < alpha.value.parse::<f64>().unwrap_or(0.0),
                    ">=" => val.parse::<f64>().unwrap_or(0.0) >= alpha.value.parse::<f64>().unwrap_or(0.0),
                    "<=" => val.parse::<f64>().unwrap_or(0.0) <= alpha.value.parse::<f64>().unwrap_or(0.0),
                    _ => false,
                }
            } else {
                false
            }
        }
        ReteUlNode::UlAnd(left, right) => {
            evaluate_rete_ul_node(left, facts) && evaluate_rete_ul_node(right, facts)
        }
        ReteUlNode::UlOr(left, right) => {
            evaluate_rete_ul_node(left, facts) || evaluate_rete_ul_node(right, facts)
        }
        ReteUlNode::UlNot(inner) => {
            !evaluate_rete_ul_node(inner, facts)
        }
        ReteUlNode::UlExists(inner) => {
            let target_field = match &**inner {
                ReteUlNode::UlAlpha(alpha) => alpha.field.clone(),
                _ => "".to_string(),
            };
            if target_field.contains('.') {
                let parts: Vec<&str> = target_field.split('.').collect();
                if parts.len() == 2 {
                    let prefix = parts[0];
                    let suffix = parts[1];
                    let filtered: Vec<_> = facts.iter()
                        .filter(|(k, _)| k.starts_with(prefix) && k.ends_with(suffix))
                        .collect();
                    filtered.iter().any(|(_, value)| {
                        let mut sub_facts = HashMap::new();
                        sub_facts.insert(target_field.clone(), (*value).clone());
                        evaluate_rete_ul_node(inner, &sub_facts)
                    })
                } else {
                    facts.iter().any(|(field, value)| {
                        let mut sub_facts = HashMap::new();
                        sub_facts.insert(field.clone(), value.clone());
                        evaluate_rete_ul_node(inner, &sub_facts)
                    })
                }
            } else {
                facts.iter().any(|(field, value)| {
                    let mut sub_facts = HashMap::new();
                    sub_facts.insert(field.clone(), value.clone());
                    evaluate_rete_ul_node(inner, &sub_facts)
                })
            }
        }
        ReteUlNode::UlForall(inner) => {
            let target_field = match &**inner {
                ReteUlNode::UlAlpha(alpha) => alpha.field.clone(),
                _ => "".to_string(),
            };
            if target_field.contains('.') {
                let parts: Vec<&str> = target_field.split('.').collect();
                if parts.len() == 2 {
                    let prefix = parts[0];
                    let suffix = parts[1];
                    let filtered: Vec<_> = facts.iter()
                        .filter(|(k, _)| k.starts_with(prefix) && k.ends_with(suffix))
                        .collect();
                    if filtered.is_empty() {
                        return true; // Vacuous truth: FORALL on empty set is TRUE
                    }
                    filtered.iter().all(|(_, value)| {
                        let mut sub_facts = HashMap::new();
                        sub_facts.insert(target_field.clone(), (*value).clone());
                        evaluate_rete_ul_node(inner, &sub_facts)
                    })
                } else {
                    facts.iter().all(|(field, value)| {
                        let mut sub_facts = HashMap::new();
                        sub_facts.insert(field.clone(), value.clone());
                        evaluate_rete_ul_node(inner, &sub_facts)
                    })
                }
            } else {
                facts.iter().all(|(field, value)| {
                    let mut sub_facts = HashMap::new();
                    sub_facts.insert(field.clone(), value.clone());
                    evaluate_rete_ul_node(inner, &sub_facts)
                })
            }
        }
        ReteUlNode::UlAccumulate {
            source_pattern,
            extract_field,
            source_conditions,
            function,
            ..
        } => {
            // Evaluate accumulate: collect matching facts and run function
            use super::accumulate::*;

            let pattern_prefix = format!("{}.", source_pattern);
            let mut matching_values = Vec::new();

            // Group facts by instance
            let mut instances: std::collections::HashMap<String, std::collections::HashMap<String, String>> =
                std::collections::HashMap::new();

            for (key, value) in facts {
                if key.starts_with(&pattern_prefix) {
                    let parts: Vec<&str> = key.strip_prefix(&pattern_prefix).unwrap().split('.').collect();

                    if parts.len() >= 2 {
                        let instance_id = parts[0];
                        let field_name = parts[1..].join(".");

                        instances
                            .entry(instance_id.to_string())
                            .or_insert_with(std::collections::HashMap::new)
                            .insert(field_name, value.clone());
                    } else if parts.len() == 1 {
                        instances
                            .entry("default".to_string())
                            .or_insert_with(std::collections::HashMap::new)
                            .insert(parts[0].to_string(), value.clone());
                    }
                }
            }

            // Filter instances by source conditions
            for (_instance_id, instance_facts) in instances {
                let mut matches = true;

                for condition_str in source_conditions {
                    if !evaluate_condition_string(condition_str, &instance_facts) {
                        matches = false;
                        break;
                    }
                }

                if matches {
                    if let Some(value_str) = instance_facts.get(extract_field) {
                        // Convert string to FactValue
                        let fact_value = if let Ok(i) = value_str.parse::<i64>() {
                            super::facts::FactValue::Integer(i)
                        } else if let Ok(f) = value_str.parse::<f64>() {
                            super::facts::FactValue::Float(f)
                        } else if let Ok(b) = value_str.parse::<bool>() {
                            super::facts::FactValue::Boolean(b)
                        } else {
                            super::facts::FactValue::String(value_str.clone())
                        };
                        matching_values.push(fact_value);
                    }
                }
            }

            // Run accumulate function - result determines if condition passes
            let has_results = !matching_values.is_empty();

            match function.as_str() {
                "count" => has_results, // Count passes if there are any matches
                "sum" | "average" | "min" | "max" => {
                    // These functions need at least one value
                    has_results
                }
                _ => true, // Unknown function - allow to continue
            }
        }
        ReteUlNode::UlTerminal(_) => true // Rule match
    }
}

/// RETE-UL: Unified Logic Node
#[derive(Debug, Clone)]
pub enum ReteUlNode {
    UlAlpha(AlphaNode),
    UlAnd(Box<ReteUlNode>, Box<ReteUlNode>),
    UlOr(Box<ReteUlNode>, Box<ReteUlNode>),
    UlNot(Box<ReteUlNode>),
    UlExists(Box<ReteUlNode>),
    UlForall(Box<ReteUlNode>),
    UlAccumulate {
        result_var: String,
        source_pattern: String,
        extract_field: String,
        source_conditions: Vec<String>,
        function: String,
        function_arg: String,
    },
    UlTerminal(String), // Rule name
}

impl ReteUlNode {
    /// Evaluate with typed facts (convenience method)
    pub fn evaluate_typed(&self, facts: &super::facts::TypedFacts) -> bool {
        evaluate_rete_ul_node_typed(self, facts)
    }
}

/// RETE-UL Rule Struct
pub struct ReteUlRule {
    pub name: String,
    pub node: ReteUlNode,
    pub priority: i32,
    pub no_loop: bool,
    pub action: Box<dyn FnMut(&mut std::collections::HashMap<String, String>)>,
}

/// Drools-style RETE-UL rule firing loop
/// Fires all matching rules, updates facts, repeats until no more rules can fire
pub fn fire_rete_ul_rules(
    rules: &mut [(String, ReteUlNode, Box<dyn FnMut(&mut std::collections::HashMap<String, String>)>)],
    facts: &mut std::collections::HashMap<String, String>,
) -> Vec<String> {
    let mut fired_rules = Vec::new();
    let mut changed = true;
    while changed {
        changed = false;
        for (rule_name, node, action) in rules.iter_mut() {
            let fired_flag = format!("{}_fired", rule_name);
            if facts.get(&fired_flag) == Some(&"true".to_string()) {
                continue;
            }
            if evaluate_rete_ul_node(node, facts) {
                action(facts);
                facts.insert(fired_flag.clone(), "true".to_string());
                fired_rules.push(rule_name.clone());
                changed = true;
            }
        }
    }
    fired_rules
}

/// Drools-style RETE-UL rule firing loop with agenda and control
pub fn fire_rete_ul_rules_with_agenda(
    rules: &mut [ReteUlRule],
    facts: &mut std::collections::HashMap<String, String>,
) -> Vec<String> {
    let mut fired_rules = Vec::new();
    let mut fired_flags = std::collections::HashSet::new();
    let max_iterations = 100; // Prevent infinite loops
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            eprintln!("Warning: RETE engine reached max iterations ({})", max_iterations);
            break;
        }

        // Build agenda: rules that match and haven't been fired yet
        let mut agenda: Vec<usize> = rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| {
                // Check if rule already fired
                if fired_flags.contains(&rule.name) {
                    return false;
                }
                // Check if rule matches current facts
                evaluate_rete_ul_node(&rule.node, facts)
            })
            .map(|(i, _)| i)
            .collect();

        // If no rules to fire, we're done
        if agenda.is_empty() {
            break;
        }

        // Sort agenda by priority (descending)
        agenda.sort_by_key(|&i| -rules[i].priority);

        // Fire all rules in agenda
        for &i in &agenda {
            let rule = &mut rules[i];

            // Execute rule action
            (rule.action)(facts);

            // Mark as fired
            fired_rules.push(rule.name.clone());
            fired_flags.insert(rule.name.clone());

            let fired_flag = format!("{}_fired", rule.name);
            facts.insert(fired_flag, "true".to_string());
        }

        // If no_loop is enabled for all rules, stop after one iteration
        if rules.iter().all(|r| r.no_loop) {
            break;
        }
    }

    fired_rules
}

/// RETE-UL Engine with cached nodes (Performance optimized!)
/// This engine builds RETE nodes once and reuses them, avoiding expensive rebuilds
pub struct ReteUlEngine {
    rules: Vec<ReteUlRule>,
    facts: std::collections::HashMap<String, String>,
}

impl ReteUlEngine {
    /// Create new engine from Rule definitions (nodes are built and cached once)
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            facts: std::collections::HashMap::new(),
        }
    }

    /// Add a rule with custom action closure
    pub fn add_rule_with_action<F>(
        &mut self,
        name: String,
        node: ReteUlNode,
        priority: i32,
        no_loop: bool,
        action: F,
    ) where
        F: FnMut(&mut std::collections::HashMap<String, String>) + 'static,
    {
        self.rules.push(ReteUlRule {
            name,
            node,
            priority,
            no_loop,
            action: Box::new(action),
        });
    }

    /// Add a rule from Rule definition (auto-build node once and cache)
    pub fn add_rule_from_definition(
        &mut self,
        rule: &crate::rete::auto_network::Rule,
        priority: i32,
        no_loop: bool,
    ) {
        let node = build_rete_ul_from_condition_group(&rule.conditions);
        let rule_name = rule.name.clone();

        // Default action: just mark as fired
        let action = Box::new(move |facts: &mut std::collections::HashMap<String, String>| {
            facts.insert(format!("{}_executed", rule_name), "true".to_string());
        }) as Box<dyn FnMut(&mut std::collections::HashMap<String, String>)>;

        self.rules.push(ReteUlRule {
            name: rule.name.clone(),
            node,
            priority,
            no_loop,
            action,
        });
    }

    /// Set a fact
    pub fn set_fact(&mut self, key: String, value: String) {
        self.facts.insert(key, value);
    }

    /// Get a fact
    pub fn get_fact(&self, key: &str) -> Option<&String> {
        self.facts.get(key)
    }

    /// Remove a fact
    pub fn remove_fact(&mut self, key: &str) -> Option<String> {
        self.facts.remove(key)
    }

    /// Get all facts
    pub fn get_all_facts(&self) -> &std::collections::HashMap<String, String> {
        &self.facts
    }

    /// Clear all facts
    pub fn clear_facts(&mut self) {
        self.facts.clear();
    }

    /// Fire all rules with agenda (using cached nodes - NO rebuild!)
    pub fn fire_all(&mut self) -> Vec<String> {
        fire_rete_ul_rules_with_agenda(&mut self.rules, &mut self.facts)
    }

    /// Check if a specific rule matches current facts (without firing)
    pub fn matches(&self, rule_name: &str) -> bool {
        self.rules
            .iter()
            .find(|r| r.name == rule_name)
            .map(|r| evaluate_rete_ul_node(&r.node, &self.facts))
            .unwrap_or(false)
    }

    /// Get all matching rules (without firing)
    pub fn get_matching_rules(&self) -> Vec<&str> {
        self.rules
            .iter()
            .filter(|r| evaluate_rete_ul_node(&r.node, &self.facts))
            .map(|r| r.name.as_str())
            .collect()
    }

    /// Reset fired flags (allow rules to fire again)
    pub fn reset_fired_flags(&mut self) {
        let keys_to_remove: Vec<_> = self.facts
            .keys()
            .filter(|k| k.ends_with("_fired") || k.ends_with("_executed"))
            .cloned()
            .collect();
        for key in keys_to_remove {
            self.facts.remove(&key);
        }
    }
}

// ============================================================================
// Typed Facts Support (NEW!)
// ============================================================================

use super::facts::{FactValue, TypedFacts};

/// Evaluate RETE-UL node with typed facts (NEW!)
pub fn evaluate_rete_ul_node_typed(node: &ReteUlNode, facts: &TypedFacts) -> bool {
    match node {
        ReteUlNode::UlAlpha(alpha) => {
            alpha.matches_typed(facts)
        }
        ReteUlNode::UlAnd(left, right) => {
            evaluate_rete_ul_node_typed(left, facts) && evaluate_rete_ul_node_typed(right, facts)
        }
        ReteUlNode::UlOr(left, right) => {
            evaluate_rete_ul_node_typed(left, facts) || evaluate_rete_ul_node_typed(right, facts)
        }
        ReteUlNode::UlNot(inner) => {
            !evaluate_rete_ul_node_typed(inner, facts)
        }
        ReteUlNode::UlExists(inner) => {
            let target_field = match &**inner {
                ReteUlNode::UlAlpha(alpha) => alpha.field.clone(),
                _ => "".to_string(),
            };
            if target_field.contains('.') {
                let parts: Vec<&str> = target_field.split('.').collect();
                if parts.len() == 2 {
                    let prefix = parts[0];
                    let suffix = parts[1];
                    let filtered: Vec<_> = facts.get_all().iter()
                        .filter(|(k, _)| k.starts_with(prefix) && k.ends_with(suffix))
                        .collect();
                    filtered.iter().any(|(_, _)| {
                        evaluate_rete_ul_node_typed(inner, facts)
                    })
                } else {
                    evaluate_rete_ul_node_typed(inner, facts)
                }
            } else {
                evaluate_rete_ul_node_typed(inner, facts)
            }
        }
        ReteUlNode::UlForall(inner) => {
            let target_field = match &**inner {
                ReteUlNode::UlAlpha(alpha) => alpha.field.clone(),
                _ => "".to_string(),
            };
            if target_field.contains('.') {
                let parts: Vec<&str> = target_field.split('.').collect();
                if parts.len() == 2 {
                    let prefix = parts[0];
                    let suffix = parts[1];
                    let filtered: Vec<_> = facts.get_all().iter()
                        .filter(|(k, _)| k.starts_with(prefix) && k.ends_with(suffix))
                        .collect();
                    if filtered.is_empty() {
                        return true; // Vacuous truth
                    }
                    filtered.iter().all(|(_, _)| {
                        evaluate_rete_ul_node_typed(inner, facts)
                    })
                } else {
                    if facts.get_all().is_empty() {
                        return true; // Vacuous truth
                    }
                    evaluate_rete_ul_node_typed(inner, facts)
                }
            } else {
                if facts.get_all().is_empty() {
                    return true; // Vacuous truth
                }
                evaluate_rete_ul_node_typed(inner, facts)
            }
        }
        ReteUlNode::UlAccumulate {
            source_pattern,
            extract_field,
            source_conditions,
            function,
            ..
        } => {
            // Evaluate accumulate with typed facts
            use super::accumulate::*;

            let pattern_prefix = format!("{}.", source_pattern);
            let mut matching_values = Vec::new();

            // Group facts by instance
            let mut instances: std::collections::HashMap<String, std::collections::HashMap<String, FactValue>> =
                std::collections::HashMap::new();

            for (key, value) in facts.get_all() {
                if key.starts_with(&pattern_prefix) {
                    let parts: Vec<&str> = key.strip_prefix(&pattern_prefix).unwrap().split('.').collect();

                    if parts.len() >= 2 {
                        let instance_id = parts[0];
                        let field_name = parts[1..].join(".");

                        instances
                            .entry(instance_id.to_string())
                            .or_insert_with(std::collections::HashMap::new)
                            .insert(field_name, value.clone());
                    } else if parts.len() == 1 {
                        instances
                            .entry("default".to_string())
                            .or_insert_with(std::collections::HashMap::new)
                            .insert(parts[0].to_string(), value.clone());
                    }
                }
            }

            // Filter instances by source conditions
            for (_instance_id, instance_facts) in instances {
                let mut matches = true;

                for condition_str in source_conditions {
                    // Convert FactValues to strings for condition evaluation
                    let string_facts: HashMap<String, String> = instance_facts
                        .iter()
                        .map(|(k, v)| (k.clone(), format!("{:?}", v)))
                        .collect();

                    if !evaluate_condition_string(condition_str, &string_facts) {
                        matches = false;
                        break;
                    }
                }

                if matches {
                    if let Some(value) = instance_facts.get(extract_field) {
                        matching_values.push(value.clone());
                    }
                }
            }

            // Run accumulate function - result determines if condition passes
            let has_results = !matching_values.is_empty();

            match function.as_str() {
                "count" => has_results,
                "sum" | "average" | "min" | "max" => has_results,
                _ => true,
            }
        }
        ReteUlNode::UlTerminal(_) => true
    }
}

/// Typed RETE-UL Rule
pub struct TypedReteUlRule {
    pub name: String,
    pub node: ReteUlNode,
    pub priority: i32,
    pub no_loop: bool,
    pub action: Box<dyn FnMut(&mut TypedFacts)>,
}

/// Typed RETE-UL Engine with cached nodes (Performance + Type Safety!)
/// This is the recommended engine for new code
pub struct TypedReteUlEngine {
    rules: Vec<TypedReteUlRule>,
    facts: TypedFacts,
}

impl TypedReteUlEngine {
    /// Create new typed engine
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            facts: TypedFacts::new(),
        }
    }

    /// Add a rule with custom action
    pub fn add_rule_with_action<F>(
        &mut self,
        name: String,
        node: ReteUlNode,
        priority: i32,
        no_loop: bool,
        action: F,
    ) where
        F: FnMut(&mut TypedFacts) + 'static,
    {
        self.rules.push(TypedReteUlRule {
            name,
            node,
            priority,
            no_loop,
            action: Box::new(action),
        });
    }

    /// Add a rule from Rule definition
    pub fn add_rule_from_definition(
        &mut self,
        rule: &crate::rete::auto_network::Rule,
        priority: i32,
        no_loop: bool,
    ) {
        let node = build_rete_ul_from_condition_group(&rule.conditions);
        let rule_name = rule.name.clone();

        let action = Box::new(move |facts: &mut TypedFacts| {
            facts.set(format!("{}_executed", rule_name), true);
        }) as Box<dyn FnMut(&mut TypedFacts)>;

        self.rules.push(TypedReteUlRule {
            name: rule.name.clone(),
            node,
            priority,
            no_loop,
            action,
        });
    }

    /// Set a fact with typed value
    pub fn set_fact<K: Into<String>, V: Into<FactValue>>(&mut self, key: K, value: V) {
        self.facts.set(key, value);
    }

    /// Get a fact
    pub fn get_fact(&self, key: &str) -> Option<&FactValue> {
        self.facts.get(key)
    }

    /// Remove a fact
    pub fn remove_fact(&mut self, key: &str) -> Option<FactValue> {
        self.facts.remove(key)
    }

    /// Get all facts
    pub fn get_all_facts(&self) -> &TypedFacts {
        &self.facts
    }

    /// Clear all facts
    pub fn clear_facts(&mut self) {
        self.facts.clear();
    }

    /// Fire all rules with agenda (using cached nodes + typed evaluation!)
    pub fn fire_all(&mut self) -> Vec<String> {
        let mut fired_rules = Vec::new();
        let mut agenda: Vec<usize>;
        let mut changed = true;
        let mut fired_flags = std::collections::HashSet::new();

        while changed {
            changed = false;

            // Build agenda: rules that match and not fired
            agenda = self.rules.iter().enumerate()
                .filter(|(_, rule)| {
                    let fired_flag = format!("{}_fired", rule.name);
                    let already_fired = fired_flags.contains(&rule.name) ||
                        self.facts.get(&fired_flag).and_then(|v| v.as_boolean()) == Some(true);
                    !rule.no_loop || !already_fired
                })
                .filter(|(_, rule)| evaluate_rete_ul_node_typed(&rule.node, &self.facts))
                .map(|(i, _)| i)
                .collect();

            // Sort by priority (descending)
            agenda.sort_by_key(|&i| -self.rules[i].priority);

            for &i in &agenda {
                let rule = &mut self.rules[i];
                let fired_flag = format!("{}_fired", rule.name);
                let already_fired = fired_flags.contains(&rule.name) ||
                    self.facts.get(&fired_flag).and_then(|v| v.as_boolean()) == Some(true);

                if rule.no_loop && already_fired {
                    continue;
                }

                (rule.action)(&mut self.facts);
                fired_rules.push(rule.name.clone());
                fired_flags.insert(rule.name.clone());
                self.facts.set(fired_flag, true);
                changed = true;
            }
        }

        fired_rules
    }

    /// Check if a specific rule matches current facts
    pub fn matches(&self, rule_name: &str) -> bool {
        self.rules
            .iter()
            .find(|r| r.name == rule_name)
            .map(|r| evaluate_rete_ul_node_typed(&r.node, &self.facts))
            .unwrap_or(false)
    }

    /// Get all matching rules
    pub fn get_matching_rules(&self) -> Vec<&str> {
        self.rules
            .iter()
            .filter(|r| evaluate_rete_ul_node_typed(&r.node, &self.facts))
            .map(|r| r.name.as_str())
            .collect()
    }

    /// Reset fired flags
    pub fn reset_fired_flags(&mut self) {
        let keys_to_remove: Vec<_> = self.facts.get_all()
            .keys()
            .filter(|k| k.ends_with("_fired") || k.ends_with("_executed"))
            .cloned()
            .collect();
        for key in keys_to_remove {
            self.facts.remove(&key);
        }
    }
}

impl Default for TypedReteUlEngine {
    fn default() -> Self {
        Self::new()
    }
}


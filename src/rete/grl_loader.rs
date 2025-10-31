//! GRL to RETE Converter
//!
//! This module converts GRL (Grule Rule Language) rules into RETE-UL structures
//! for efficient pattern matching and rule execution.

use crate::engine::rule::{Condition, ConditionGroup, Rule};
use crate::parser::GRLParser;
use crate::rete::{AlphaNode, ReteUlNode, TypedReteUlRule};
use crate::rete::facts::{TypedFacts, FactValue};
use crate::rete::propagation::IncrementalEngine;
use crate::types::{Operator, Value};
use crate::errors::{Result, RuleEngineError};
use std::fs;
use std::path::Path;

/// GRL to RETE Loader
/// Converts GRL rules into RETE-UL structures
pub struct GrlReteLoader;

impl GrlReteLoader {
    /// Load rules from a GRL file into RETE engine
    pub fn load_from_file<P: AsRef<Path>>(
        path: P,
        engine: &mut IncrementalEngine,
    ) -> Result<usize> {
        let grl_text = fs::read_to_string(path.as_ref()).map_err(|e| {
            RuleEngineError::ParseError {
                message: format!("Failed to read GRL file: {}", e),
            }
        })?;

        Self::load_from_string(&grl_text, engine)
    }

    /// Load rules from GRL string into RETE engine
    pub fn load_from_string(
        grl_text: &str,
        engine: &mut IncrementalEngine,
    ) -> Result<usize> {
        // Parse GRL rules
        let rules = GRLParser::parse_rules(grl_text)?;

        let mut loaded_count = 0;

        for rule in rules {
            // Convert GRL rule to RETE rule
            let rete_rule = Self::convert_rule_to_rete(rule)?;

            // Extract dependencies (fact types used in conditions)
            let dependencies = Self::extract_dependencies(&rete_rule);

            // Add to engine
            engine.add_rule(rete_rule, dependencies);
            loaded_count += 1;
        }

        Ok(loaded_count)
    }

    /// Convert GRL Rule to TypedReteUlRule
    fn convert_rule_to_rete(rule: Rule) -> Result<TypedReteUlRule> {
        // Convert ConditionGroup to ReteUlNode
        let node = Self::convert_condition_group(&rule.conditions)?;

        // Create RETE rule
        let rete_rule = TypedReteUlRule {
            name: rule.name.clone(),
            node,
            priority: rule.salience,
            no_loop: rule.no_loop,
            action: Self::create_action_closure(rule.actions),
        };

        Ok(rete_rule)
    }

    /// Convert ConditionGroup to ReteUlNode
    fn convert_condition_group(group: &ConditionGroup) -> Result<ReteUlNode> {
        match group {
            ConditionGroup::Single(condition) => {
                Self::convert_condition(condition)
            }
            ConditionGroup::Compound { left, operator, right } => {
                let left_node = Self::convert_condition_group(left)?;
                let right_node = Self::convert_condition_group(right)?;

                match operator {
                    crate::types::LogicalOperator::And => {
                        Ok(ReteUlNode::UlAnd(Box::new(left_node), Box::new(right_node)))
                    }
                    crate::types::LogicalOperator::Or => {
                        Ok(ReteUlNode::UlOr(Box::new(left_node), Box::new(right_node)))
                    }
                    crate::types::LogicalOperator::Not => {
                        // For NOT, we only use left node
                        Ok(ReteUlNode::UlNot(Box::new(left_node)))
                    }
                }
            }
            ConditionGroup::Not(inner) => {
                let inner_node = Self::convert_condition_group(inner)?;
                Ok(ReteUlNode::UlNot(Box::new(inner_node)))
            }
            ConditionGroup::Exists(inner) => {
                let inner_node = Self::convert_condition_group(inner)?;
                Ok(ReteUlNode::UlExists(Box::new(inner_node)))
            }
            ConditionGroup::Forall(inner) => {
                let inner_node = Self::convert_condition_group(inner)?;
                Ok(ReteUlNode::UlForall(Box::new(inner_node)))
            }
        }
    }

    /// Convert single Condition to ReteUlNode (AlphaNode)
    fn convert_condition(condition: &Condition) -> Result<ReteUlNode> {
        let operator_str = Self::operator_to_string(&condition.operator);
        let value_str = Self::value_to_string(&condition.value);

        let alpha = AlphaNode {
            field: condition.field.clone(),
            operator: operator_str,
            value: value_str,
        };

        Ok(ReteUlNode::UlAlpha(alpha))
    }

    /// Convert Operator to string
    fn operator_to_string(op: &Operator) -> String {
        match op {
            Operator::Equal => "==".to_string(),
            Operator::NotEqual => "!=".to_string(),
            Operator::GreaterThan => ">".to_string(),
            Operator::GreaterThanOrEqual => ">=".to_string(),
            Operator::LessThan => "<".to_string(),
            Operator::LessThanOrEqual => "<=".to_string(),
            Operator::Contains => "contains".to_string(),
            Operator::NotContains => "!contains".to_string(),
            Operator::StartsWith => "startsWith".to_string(),
            Operator::EndsWith => "endsWith".to_string(),
            Operator::Matches => "matches".to_string(),
        }
    }

    /// Convert Value to string for AlphaNode
    fn value_to_string(value: &Value) -> String {
        match value {
            Value::Number(n) => n.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(arr) => {
                // Convert array to JSON-like string
                let items: Vec<String> = arr.iter()
                    .map(|v| Self::value_to_string(v))
                    .collect();
                format!("[{}]", items.join(","))
            }
            Value::Object(_) => {
                // For objects, we'll use a simplified representation
                "object".to_string()
            }
        }
    }

    /// Create action closure from ActionType list
    fn create_action_closure(
        actions: Vec<crate::types::ActionType>,
    ) -> Box<dyn FnMut(&mut TypedFacts)> {
        Box::new(move |facts: &mut TypedFacts| {
            // Execute actions
            for action in &actions {
                Self::execute_action(action, facts);
            }
        })
    }

    /// Execute a single action
    fn execute_action(action: &crate::types::ActionType, facts: &mut TypedFacts) {
        use crate::types::ActionType;

        match action {
            ActionType::Set { field, value } => {
                // Convert Value to FactValue
                let fact_value = Self::value_to_fact_value(value);
                facts.set(field, fact_value);
            }
            ActionType::Log { message } => {
                println!("📝 LOG: {}", message);
            }
            ActionType::Call { function, args: _ } => {
                // For function calls, we'll just log them
                println!("🔧 CALL: {}", function);
            }
            ActionType::MethodCall { object, method, args: _ } => {
                println!("📞 METHOD: {}.{}", object, method);
            }
            ActionType::Update { object } => {
                println!("🔄 UPDATE: {}", object);
            }
            ActionType::Custom { action_type, params: _ } => {
                println!("⚙️ CUSTOM: {}", action_type);
            }
            ActionType::ActivateAgendaGroup { group } => {
                println!("📋 ACTIVATE GROUP: {}", group);
            }
            ActionType::ScheduleRule { rule_name, delay_ms } => {
                println!("⏰ SCHEDULE: {} (delay: {}ms)", rule_name, delay_ms);
            }
            ActionType::CompleteWorkflow { workflow_name } => {
                println!("✔️ COMPLETE WORKFLOW: {}", workflow_name);
            }
            ActionType::SetWorkflowData { key, value: _ } => {
                println!("📊 SET WORKFLOW DATA: {}", key);
            }
        }
    }

    /// Convert Value to FactValue
    fn value_to_fact_value(value: &Value) -> FactValue {
        match value {
            Value::Number(n) => {
                // Try integer first, fall back to float
                if n.fract() == 0.0 {
                    FactValue::Integer(*n as i64)
                } else {
                    FactValue::Float(*n)
                }
            }
            Value::Integer(i) => FactValue::Integer(*i),
            Value::String(s) => FactValue::String(s.clone()),
            Value::Boolean(b) => FactValue::Boolean(*b),
            Value::Null => FactValue::Null,
            Value::Array(arr) => {
                let fact_arr: Vec<FactValue> = arr.iter()
                    .map(Self::value_to_fact_value)
                    .collect();
                FactValue::Array(fact_arr)
            }
            Value::Object(_) => {
                // For now, treat objects as strings
                FactValue::String("object".to_string())
            }
        }
    }

    /// Extract fact type dependencies from rule
    fn extract_dependencies(rule: &TypedReteUlRule) -> Vec<String> {
        let mut deps = Vec::new();
        Self::extract_deps_from_node(&rule.node, &mut deps);

        // Deduplicate
        deps.sort();
        deps.dedup();

        deps
    }

    /// Recursively extract dependencies from ReteUlNode
    fn extract_deps_from_node(node: &ReteUlNode, deps: &mut Vec<String>) {
        match node {
            ReteUlNode::UlAlpha(alpha) => {
                // Extract fact type from field (e.g., "Person.age" -> "Person")
                if let Some(dot_pos) = alpha.field.find('.') {
                    let fact_type = alpha.field[..dot_pos].to_string();
                    deps.push(fact_type);
                }
            }
            ReteUlNode::UlAnd(left, right) | ReteUlNode::UlOr(left, right) => {
                Self::extract_deps_from_node(left, deps);
                Self::extract_deps_from_node(right, deps);
            }
            ReteUlNode::UlNot(inner)
            | ReteUlNode::UlExists(inner)
            | ReteUlNode::UlForall(inner) => {
                Self::extract_deps_from_node(inner, deps);
            }
            ReteUlNode::UlTerminal(_) => {
                // Terminal nodes don't have dependencies
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_rule() {
        let grl = r#"
        rule "TestRule" salience 10 no-loop {
            when
                Person.age > 18
            then
                Person.is_adult = true;
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);

        let rete_rule = GrlReteLoader::convert_rule_to_rete(rules[0].clone()).unwrap();
        assert_eq!(rete_rule.name, "TestRule");
        assert_eq!(rete_rule.priority, 10);
        assert!(rete_rule.no_loop);
    }

    #[test]
    fn test_extract_dependencies() {
        let grl = r#"
        rule "MultiTypeRule" {
            when
                Person.age > 18 && Order.amount > 1000
            then
                Person.premium = true;
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        let rete_rule = GrlReteLoader::convert_rule_to_rete(rules[0].clone()).unwrap();
        let deps = GrlReteLoader::extract_dependencies(&rete_rule);

        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"Person".to_string()));
        assert!(deps.contains(&"Order".to_string()));
    }

    #[test]
    fn test_load_from_string() {
        let grl = r#"
        rule "Rule1" {
            when
                Person.age > 18
            then
                Person.is_adult = true;
        }

        rule "Rule2" {
            when
                Order.amount > 1000
            then
                Order.high_value = true;
        }
        "#;

        let mut engine = IncrementalEngine::new();
        let count = GrlReteLoader::load_from_string(grl, &mut engine).unwrap();

        assert_eq!(count, 2);
    }
}

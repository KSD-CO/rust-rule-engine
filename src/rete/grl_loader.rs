//! GRL to RETE Converter
//!
//! This module converts GRL (Grule Rule Language) rules into RETE-UL structures
//! for efficient pattern matching and rule execution.

#![allow(clippy::type_complexity)]
#![allow(deprecated)]

use crate::engine::rule::{Condition, ConditionGroup, Rule};
use crate::errors::{Result, RuleEngineError};
use crate::parser::GRLParser;
use crate::rete::facts::{FactValue, TypedFacts};
use crate::rete::propagation::IncrementalEngine;
use crate::rete::{AlphaNode, ReteUlNode, TypedReteUlRule};
use crate::types::{Operator, Value};
use log::info;
use std::fs;
use std::path::Path;

#[cfg(feature = "streaming")]
use crate::rete::network::{StreamWindowSpec, StreamWindowTypeRete};

/// GRL to RETE Loader
/// Converts GRL rules into RETE-UL structures
pub struct GrlReteLoader;

impl GrlReteLoader {
    /// Load rules from a GRL file into RETE engine
    pub fn load_from_file<P: AsRef<Path>>(
        path: P,
        engine: &mut IncrementalEngine,
    ) -> Result<usize> {
        let grl_text =
            fs::read_to_string(path.as_ref()).map_err(|e| RuleEngineError::ParseError {
                message: format!("Failed to read GRL file: {}", e),
            })?;

        Self::load_from_string(&grl_text, engine)
    }

    /// Load rules from GRL string into RETE engine
    pub fn load_from_string(grl_text: &str, engine: &mut IncrementalEngine) -> Result<usize> {
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
            ConditionGroup::Single(condition) => Self::convert_condition(condition),
            ConditionGroup::Compound {
                left,
                operator,
                right,
            } => {
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
            ConditionGroup::Accumulate {
                result_var,
                source_pattern,
                extract_field,
                source_conditions,
                function,
                function_arg,
            } => Ok(ReteUlNode::UlAccumulate {
                result_var: result_var.clone(),
                source_pattern: source_pattern.clone(),
                extract_field: extract_field.clone(),
                source_conditions: source_conditions.clone(),
                function: function.clone(),
                function_arg: function_arg.clone(),
            }),
            #[cfg(feature = "streaming")]
            ConditionGroup::StreamPattern {
                var_name,
                event_type,
                stream_name,
                window,
            } => {
                // Convert stream pattern to RETE UlStream node
                Ok(ReteUlNode::UlStream {
                    var_name: var_name.clone(),
                    event_type: event_type.clone(),
                    stream_name: stream_name.clone(),
                    window: window.as_ref().map(|w| StreamWindowSpec {
                        duration: w.duration,
                        window_type: match &w.window_type {
                            crate::engine::rule::StreamWindowType::Sliding => {
                                StreamWindowTypeRete::Sliding
                            }
                            crate::engine::rule::StreamWindowType::Tumbling => {
                                StreamWindowTypeRete::Tumbling
                            }
                            crate::engine::rule::StreamWindowType::Session { timeout } => {
                                StreamWindowTypeRete::Session { timeout: *timeout }
                            }
                        },
                    }),
                })
            }
        }
    }

    /// Convert single Condition to ReteUlNode (AlphaNode or UlMultiField)
    fn convert_condition(condition: &Condition) -> Result<ReteUlNode> {
        use crate::engine::rule::ConditionExpression;

        // Check if this is a multifield condition
        match &condition.expression {
            ConditionExpression::MultiField {
                field,
                operation,
                variable: _,
            } => {
                // Convert to UlMultiField node
                let operator_str = Self::operator_to_string(&condition.operator);
                let value_str = if !matches!(condition.value, Value::Boolean(_)) {
                    Some(Self::value_to_string(&condition.value))
                } else {
                    None
                };

                // Determine if this is a count operation with comparison
                let (op, cmp_val) = if operation == "count" && operator_str != "==" {
                    // Count with comparison: "count > 5"
                    (Some(operator_str), value_str)
                } else {
                    // Other operations
                    (None, value_str)
                };

                Ok(ReteUlNode::UlMultiField {
                    field: field.clone(),
                    operation: operation.clone(),
                    value: if operation == "contains" {
                        cmp_val.clone()
                    } else {
                        None
                    },
                    operator: op,
                    compare_value: if operation == "count" { cmp_val } else { None },
                })
            }
            _ => {
                // Standard alpha node for regular conditions
                let operator_str = Self::operator_to_string(&condition.operator);
                let value_str = Self::value_to_string(&condition.value);

                let alpha = AlphaNode {
                    field: condition.field.clone(),
                    operator: operator_str,
                    value: value_str,
                };

                Ok(ReteUlNode::UlAlpha(alpha))
            }
        }
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
            Operator::In => "in".to_string(),
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
                let items: Vec<String> = arr.iter().map(Self::value_to_string).collect();
                format!("[{}]", items.join(","))
            }
            Value::Object(_) => {
                // For objects, we'll use a simplified representation
                "object".to_string()
            }
            Value::Expression(expr) => {
                // For expressions, return the expression string
                expr.clone()
            }
        }
    }

    /// Create action closure from ActionType list
    fn create_action_closure(
        actions: Vec<crate::types::ActionType>,
    ) -> std::sync::Arc<dyn Fn(&mut TypedFacts, &mut super::ActionResults) + Send + Sync> {
        std::sync::Arc::new(
            move |facts: &mut TypedFacts, results: &mut super::ActionResults| {
                // Execute actions
                for action in &actions {
                    Self::execute_action(action, facts, results);
                }
            },
        )
    }

    /// Execute a single action
    fn execute_action(
        action: &crate::types::ActionType,
        facts: &mut TypedFacts,
        results: &mut super::ActionResults,
    ) {
        use crate::types::ActionType;

        match action {
            ActionType::Set { field, value } => {
                // Assignment action (from "field = value" syntax in GRL)
                // Note: Set() function syntax is NOT supported.
                // Use: Player.score = Player.score + 10;

                // Check if value is an expression that needs evaluation
                let evaluated_value = match value {
                    Value::Expression(expr) => {
                        // Evaluate expression with current facts
                        Self::evaluate_expression_for_rete(expr, facts)
                    }
                    _ => value.clone(),
                };

                // Convert evaluated value to FactValue
                let fact_value = Self::value_to_fact_value(&evaluated_value);
                facts.set(field, fact_value);
            }
            ActionType::Log { message } => {
                info!("📝 {}", message);
            }
            ActionType::MethodCall {
                object,
                method,
                args,
            } => {
                // Method calls can be treated as function calls with object as first arg
                let mut all_args = vec![object.clone()];
                all_args.extend(args.iter().map(Self::value_to_string));

                results.add(super::ActionResult::CallFunction {
                    function_name: format!("{}.{}", object, method),
                    args: all_args,
                });
                println!("� METHOD: {}.{}", object, method);
            }
            ActionType::Retract { object } => {
                // Strip quotes from object name if present
                let object_name = object.trim_matches('"');

                // Try to get the handle for this fact type from metadata
                if let Some(handle) = facts.get_fact_handle(object_name) {
                    // Retract specific fact by handle
                    results.add(super::ActionResult::Retract(handle));
                    println!("🗑️ RETRACT: {} (handle: {:?})", object_name, handle);
                } else {
                    // Fallback: retract by type (first matching fact)
                    results.add(super::ActionResult::RetractByType(object_name.to_string()));
                    println!("🗑️ RETRACT: {} (by type, no handle found)", object_name);
                }
            }
            ActionType::Custom {
                action_type,
                params,
            } => {
                // Treat custom actions as function calls
                let args: Vec<String> = params.values().map(Self::value_to_string).collect();

                results.add(super::ActionResult::CallFunction {
                    function_name: action_type.clone(),
                    args,
                });
                println!("🔧 CUSTOM CALL: {}", action_type);
            }
            ActionType::ActivateAgendaGroup { group } => {
                // Queue agenda group activation
                results.add(super::ActionResult::ActivateAgendaGroup(group.clone()));
                println!("📋 ACTIVATE GROUP: {}", group);
            }
            ActionType::ScheduleRule {
                rule_name,
                delay_ms,
            } => {
                // Queue rule scheduling
                results.add(super::ActionResult::ScheduleRule {
                    rule_name: rule_name.clone(),
                    delay_ms: *delay_ms,
                });
                println!("⏰ SCHEDULE: {} (delay: {}ms)", rule_name, delay_ms);
            }
            ActionType::CompleteWorkflow { workflow_name } => {
                // Mark workflow as completed by setting a fact
                let completion_key = format!("workflow.{}.completed", workflow_name);
                facts.set(&completion_key, FactValue::Boolean(true));

                let timestamp_key = format!("workflow.{}.completed_at", workflow_name);
                facts.set(
                    &timestamp_key,
                    FactValue::Integer(chrono::Utc::now().timestamp()),
                );

                println!("✔️ WORKFLOW COMPLETED: {}", workflow_name);
            }
            ActionType::SetWorkflowData { key, value } => {
                // Store workflow data as facts with "workflow.data." prefix
                let data_key = format!("workflow.data.{}", key);
                let fact_value = Self::value_to_fact_value(value);
                facts.set(&data_key, fact_value);

                println!("📊 WORKFLOW DATA SET: {} = {:?}", key, value);
            }
            ActionType::Append { field, value } => {
                // Append to array field
                // Get current array or create new one
                let current_value = facts.get(field);

                let mut array = match current_value {
                    Some(FactValue::Array(arr)) => arr.clone(),
                    Some(_) => {
                        // Field exists but is not an array, create new array
                        log::warn!("Field {} is not an array, creating new array", field);
                        Vec::new()
                    }
                    None => {
                        // Field doesn't exist, create new array
                        Vec::new()
                    }
                };

                // Evaluate value if it's an expression
                let evaluated_value = match value {
                    Value::Expression(expr) => Self::evaluate_expression_for_rete(expr, facts),
                    _ => value.clone(),
                };

                // Convert to FactValue and append
                let fact_value = Self::value_to_fact_value(&evaluated_value);
                array.push(fact_value);

                // Set the updated array
                facts.set(field, FactValue::Array(array));

                info!("➕ APPEND: {} += {:?}", field, evaluated_value);
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
                let fact_arr: Vec<FactValue> = arr.iter().map(Self::value_to_fact_value).collect();
                FactValue::Array(fact_arr)
            }
            Value::Object(_) => {
                // For now, treat objects as strings
                FactValue::String("object".to_string())
            }
            Value::Expression(expr) => {
                // For expressions, store as string - will be evaluated at runtime
                FactValue::String(format!("[EXPR: {}]", expr))
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
            ReteUlNode::UlMultiField { field, .. } => {
                // Extract fact type from field (e.g., "Order.items" -> "Order")
                if let Some(dot_pos) = field.find('.') {
                    let fact_type = field[..dot_pos].to_string();
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
            ReteUlNode::UlAccumulate { source_pattern, .. } => {
                // Add source pattern as a dependency
                deps.push(source_pattern.clone());
            }
            #[cfg(feature = "streaming")]
            ReteUlNode::UlStream { stream_name, .. } => {
                // Add stream name as a dependency
                deps.push(stream_name.clone());
            }
            ReteUlNode::UlTerminal(_) => {
                // Terminal nodes don't have dependencies
            }
        }
    }

    /// Evaluate expression for RETE engine (converts TypedFacts to Facts temporarily)
    fn evaluate_expression_for_rete(expr: &str, typed_facts: &TypedFacts) -> Value {
        // Convert TypedFacts to Facts for expression evaluation
        use crate::engine::facts::Facts;

        let facts = Facts::new();

        // Copy all facts from TypedFacts to Facts
        // RETE stores facts as "quantity" while GRL uses "Order.quantity"
        // We need to support both formats
        for (key, value) in typed_facts.get_all() {
            let converted_value = Self::fact_value_to_value(value);

            // Store both with and without prefix
            // E.g., "quantity" -> both "quantity" and "Order.quantity"
            facts.set(key, converted_value.clone());

            // Also try to add with "Order." prefix if not already present
            if !key.contains('.') {
                facts.set(&format!("Order.{}", key), converted_value);
            }
        }

        // Evaluate expression
        match crate::expression::evaluate_expression(expr, &facts) {
            Ok(result) => result,
            Err(_e) => {
                // Silently fallback - this can happen with chained expressions in RETE
                // due to working memory complexity
                Value::String(expr.to_string())
            }
        }
    }

    /// Convert FactValue back to Value (reverse of value_to_fact_value)
    fn fact_value_to_value(fact_value: &FactValue) -> Value {
        match fact_value {
            FactValue::String(s) => {
                // Try to parse as number first
                if let Ok(i) = s.parse::<i64>() {
                    Value::Integer(i)
                } else if let Ok(f) = s.parse::<f64>() {
                    Value::Number(f)
                } else if s == "true" {
                    Value::Boolean(true)
                } else if s == "false" {
                    Value::Boolean(false)
                } else {
                    Value::String(s.clone())
                }
            }
            FactValue::Integer(i) => Value::Integer(*i),
            FactValue::Float(f) => Value::Number(*f),
            FactValue::Boolean(b) => Value::Boolean(*b),
            FactValue::Array(arr) => {
                Value::Array(arr.iter().map(Self::fact_value_to_value).collect())
            }
            FactValue::Null => Value::Null,
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

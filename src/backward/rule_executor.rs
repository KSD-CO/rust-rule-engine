//! Rule execution for backward chaining
//!
//! This module provides proper condition evaluation and action execution for backward
//! chaining queries. It integrates with the Truth Maintenance System (TMS) to support
//! logical fact insertion and justification-based retraction.
//!
//! # Features
//!
//! - **Condition evaluation** - Evaluate rule conditions against current facts
//! - **Action execution** - Execute rule actions (Set, MethodCall, Log, Retract)
//! - **TMS integration** - Optional logical fact insertion with justifications
//! - **Function support** - Built-in functions (len, isEmpty, exists, etc.)
//! - **Type conversion** - Convert between Facts (string-based) and TypedFacts (RETE)
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────┐
//! │  RuleExecutor    │
//! │                  │
//! │  ┌────────────┐  │
//! │  │ Condition  │──┼──> ConditionEvaluator
//! │  │ Evaluator  │  │       │
//! │  └────────────┘  │       ├─> Built-in functions
//! │                  │       └─> Field comparison
//! │  ┌────────────┐  │
//! │  │   Action   │──┼──> Set, MethodCall, Log
//! │  │ Executor   │  │       │
//! │  └────────────┘  │       └─> TMS Inserter (optional)
//! └──────────────────┘
//! ```
//!
//! # Example: Basic Rule Execution
//!
//! ```rust
//! use rust_rule_engine::backward::rule_executor::RuleExecutor;
//! use rust_rule_engine::engine::rule::{Rule, Condition, ConditionGroup};
//! use rust_rule_engine::types::{Operator, ActionType, Value};
//! use rust_rule_engine::{KnowledgeBase, Facts};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let kb = KnowledgeBase::new("test");
//! let executor = RuleExecutor::new(kb);
//!
//! // Define a rule: If User.Age > 18, then User.IsAdult = true
//! let conditions = ConditionGroup::Single(
//!     Condition::new(
//!         "User.Age".to_string(),
//!         Operator::GreaterThan,
//!         Value::Number(18.0),
//!     )
//! );
//! let actions = vec![ActionType::Set {
//!     field: "User.IsAdult".to_string(),
//!     value: Value::Boolean(true),
//! }];
//! let rule = Rule::new("CheckAdult".to_string(), conditions, actions);
//!
//! // Execute rule
//! let mut facts = Facts::new();
//! facts.set("User.Age", Value::Number(25.0));
//!
//! let executed = executor.try_execute_rule(&rule, &mut facts)?;
//! assert!(executed); // Rule should execute successfully
//! assert_eq!(facts.get("User.IsAdult"), Some(Value::Boolean(true)));
//! # Ok(())
//! # }
//! ```
//!
//! # Example: TMS Integration
//!
//! ```rust
//! use rust_rule_engine::backward::rule_executor::RuleExecutor;
//! use rust_rule_engine::rete::propagation::IncrementalEngine;
//! use rust_rule_engine::{KnowledgeBase, Facts};
//! use std::sync::{Arc, Mutex};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let kb = KnowledgeBase::new("test");
//! let rete_engine = Arc::new(Mutex::new(IncrementalEngine::new()));
//!
//! // Create TMS inserter callback
//! let inserter = {
//!     let eng = rete_engine.clone();
//!     Arc::new(move |fact_type: String, data, rule_name: String, premises: Vec<String>| {
//!         if let Ok(mut e) = eng.lock() {
//!             let handles = e.resolve_premise_keys(premises);
//!             let _ = e.insert_logical(fact_type, data, rule_name, handles);
//!         }
//!     })
//! };
//!
//! let executor = RuleExecutor::new_with_inserter(kb, Some(inserter));
//! // Now rule executions will insert facts logically with justifications
//! # Ok(())
//! # }
//! ```
//!
//! # Supported Action Types
//!
//! - **Set** - Set a fact value: `field: value`
//! - **MethodCall** - Call a method on an object: `object.method(args)`
//! - **Log** - Log a message: `log("message")`
//! - **Retract** - Retract a fact: `retract(field)`
//!
//! # Built-in Functions
//!
//! The executor supports these built-in functions for condition evaluation:
//! - `len(field)` - Get string/array length
//! - `isEmpty(field)` - Check if string/array is empty
//! - `exists(field)` - Check if field exists
//! - `count(field)` - Count array elements

use crate::engine::rule::{Condition, ConditionGroup, Rule};
use crate::engine::condition_evaluator::ConditionEvaluator;
use crate::types::{ActionType, Value};
use crate::{Facts, KnowledgeBase};
use crate::errors::{Result, RuleEngineError};

/// Rule executor for backward chaining
pub struct RuleExecutor {
    evaluator: ConditionEvaluator,
    /// Optional TMS inserter callback: (fact_type, typed_data, source_rule, premise_keys)
    /// premise_keys are strings in the format: "Type.field=value" which the inserter
    /// can use to resolve to working-memory FactHandles.
    tms_inserter: Option<std::sync::Arc<dyn Fn(String, crate::rete::TypedFacts, String, Vec<String>) + Send + Sync>>,
}

impl RuleExecutor {
    /// Create a new rule executor
    ///
    /// Note: The knowledge_base parameter is kept for API compatibility but is not used.
    /// Rule evaluation is done through the ConditionEvaluator.
    pub fn new(_knowledge_base: KnowledgeBase) -> Self {
        Self::new_with_inserter(_knowledge_base, None)
    }

    /// Create a new executor with optional TMS inserter callback
    ///
    /// Note: The knowledge_base parameter is kept for API compatibility but is not used.
    /// Rule evaluation is done through the ConditionEvaluator.
    pub fn new_with_inserter(
        _knowledge_base: KnowledgeBase,
        inserter: Option<std::sync::Arc<dyn Fn(String, crate::rete::TypedFacts, String, Vec<String>) + Send + Sync>>,
    ) -> Self {
        Self {
            evaluator: ConditionEvaluator::with_builtin_functions(),
            tms_inserter: inserter,
        }
    }

    /// Check if rule conditions are satisfied and execute if they are
    ///
    /// Returns:
    /// - Ok(true) if rule executed successfully
    /// - Ok(false) if conditions not satisfied
    /// - Err if execution failed
    pub fn try_execute_rule(
        &self,
        rule: &Rule,
        facts: &mut Facts,
    ) -> Result<bool> {
        // Check if all conditions are satisfied
        if !self.evaluate_conditions(&rule.conditions, facts)? {
            return Ok(false);
        }

        // Conditions satisfied - execute actions
        self.execute_actions(rule, facts)?;

        Ok(true)
    }

    /// Evaluate condition group
    pub fn evaluate_conditions(
        &self,
        group: &ConditionGroup,
        facts: &Facts,
    ) -> Result<bool> {
        // Delegate to shared evaluator
        self.evaluator.evaluate_conditions(group, facts)
    }

    /// Evaluate a single condition
    pub fn evaluate_condition(&self, condition: &Condition, facts: &Facts) -> Result<bool> {
        // Delegate to shared evaluator
        self.evaluator.evaluate_condition(condition, facts)
    }

    /// Execute rule actions
    fn execute_actions(&self, rule: &Rule, facts: &mut Facts) -> Result<()> {
        for action in &rule.actions {
            self.execute_action(Some(rule), action, facts)?;
        }

        Ok(())
    }

    /// Execute a single action (has access to rule for TMS justifications)
    fn execute_action(&self, rule: Option<&Rule>, action: &ActionType, facts: &mut Facts) -> Result<()> {
        match action {
            ActionType::Set { field, value } => {
                // Evaluate value expression if needed
                let evaluated_value = self.evaluate_value_expression(value, facts)?;

                // If we have a TMS inserter and the field looks like "Type.field",
                // attempt to create a TypedFacts wrapper and call the inserter as a logical assertion.
                if let Some(inserter) = &self.tms_inserter {
                    if let Some(dot_pos) = field.find('.') {
                        let fact_type = field[..dot_pos].to_string();
                        let field_name = field[dot_pos + 1..].to_string();

                        // Build TypedFacts with this single field
                        let mut typed = crate::rete::TypedFacts::new();
                        // Map crate::types::Value -> rete::FactValue
                        let fv = match &evaluated_value {
                            crate::types::Value::String(s) => crate::rete::FactValue::String(s.clone()),
                            crate::types::Value::Integer(i) => crate::rete::FactValue::Integer(*i),
                            crate::types::Value::Number(n) => crate::rete::FactValue::Float(*n),
                            crate::types::Value::Boolean(b) => crate::rete::FactValue::Boolean(*b),
                            _ => crate::rete::FactValue::String(format!("{:?}", evaluated_value)),
                        };

                        typed.set(field_name, fv);

                        // Build premise keys from the rule's conditions (best-effort):
                        // format: "Type.field=value" so the RETE engine can map to handles.
                        let premises = match rule {
                            Some(r) => self.collect_premise_keys_from_rule(r, facts),
                            None => Vec::new(),
                        };

                        // Call inserter with rule name (string-based premises)
                        let source_name = rule.map(|r| r.name.clone()).unwrap_or_else(|| "<unknown>".to_string());
                        (inserter)(fact_type, typed, source_name, premises);
                        // Also apply to local Facts representation so backward search sees it
                        facts.set(field, evaluated_value);
                        return Ok(());
                    }
                }

                // Fallback: just set into Facts
                facts.set(field, evaluated_value);
                Ok(())
            }

            ActionType::MethodCall { object, method, args } => {
                // Execute method call
                if let Some(obj_value) = facts.get(object) {
                    let mut obj_value = obj_value.clone();
                    // Evaluate arguments
                    let mut arg_values = Vec::new();
                    for arg in args {
                        let val = self.evaluate_value_expression(arg, facts)?;
                        arg_values.push(val);
                    }

                    // Call method
                    let result = obj_value.call_method(method, arg_values)
                        .map_err(|e| RuleEngineError::ExecutionError(e))?;

                    // Update object
                    facts.set(object, obj_value);

                    // Store result if there's a return value
                    if result != Value::Null {
                        facts.set(&format!("{}._return", object), result);
                    }

                    Ok(())
                } else {
                    Err(RuleEngineError::ExecutionError(
                        format!("Object not found: {}", object)
                    ))
                }
            }

            ActionType::Retract { object } => {
                // Retract fact from working memory
                // In backward chaining, we just remove the fact
                facts.remove(object);
                Ok(())
            }

            ActionType::Log { message } => {
                // Just log for now
                println!("[BC Action] {}", message);
                Ok(())
            }

            ActionType::Custom { .. } => {
                // Custom actions not supported in backward chaining yet
                Ok(())
            }

            ActionType::ActivateAgendaGroup { .. } => {
                // Agenda groups not supported in backward chaining
                Ok(())
            }

            ActionType::ScheduleRule { .. } => {
                // Rule scheduling not supported in backward chaining
                Ok(())
            }

            ActionType::CompleteWorkflow { .. } => {
                // Workflows not supported in backward chaining
                Ok(())
            }

            ActionType::SetWorkflowData { .. } => {
                // Workflow data not supported in backward chaining
                Ok(())
            }
        }
    }

    /// Collect a best-effort list of premise keys from the rule's conditions.
    /// Each entry has the format: "Type.field=value" when possible. This is
    /// intentionally conservative: only field-based conditions with a dotted
    /// "Type.field" expression are collected.
    fn collect_premise_keys_from_rule(&self, rule: &Rule, facts: &Facts) -> Vec<String> {
        use crate::engine::rule::{ConditionGroup, ConditionExpression};

        let mut keys = Vec::new();

        fn collect_from_group(group: &ConditionGroup, keys: &mut Vec<String>, facts: &Facts) {
            match group {
                ConditionGroup::Single(cond) => {
                    if let ConditionExpression::Field(f) = &cond.expression {
                        if let Some(dot_pos) = f.find('.') {
                            let fact_type = &f[..dot_pos];
                            let field_name = &f[dot_pos + 1..];

                            // Try to get value from facts
                            if let Some(val) = facts.get(f).or_else(|| facts.get_nested(f)) {
                                let value_str = match val {
                                    crate::types::Value::String(s) => s.clone(),
                                    crate::types::Value::Integer(i) => i.to_string(),
                                    crate::types::Value::Number(n) => n.to_string(),
                                    crate::types::Value::Boolean(b) => b.to_string(),
                                    _ => format!("{:?}", val),
                                };

                                keys.push(format!("{}.{}={}", fact_type, field_name, value_str));
                            } else {
                                // If we don't have a value at this time, still record the key without value
                                keys.push(format!("{}.{}=", fact_type, field_name));
                            }
                        }
                    }
                }
                ConditionGroup::Compound { left, right, .. } => {
                    collect_from_group(left, keys, facts);
                    collect_from_group(right, keys, facts);
                }
                // For other complex groups, skip
                _ => {}
            }
        }

        collect_from_group(&rule.conditions, &mut keys, facts);
        keys
    }

    /// Evaluate value expression
    fn evaluate_value_expression(&self, value: &Value, facts: &Facts) -> Result<Value> {
        match value {
            Value::Expression(expr) => {
                // Try simple field lookup first
                if let Some(val) = facts.get(expr).or_else(|| facts.get_nested(expr)) {
                    return Ok(val);
                }

                // Try arithmetic expression evaluation
                if let Some(result) = self.try_evaluate_arithmetic(expr, facts) {
                    return Ok(result);
                }

                // Try to parse as literal
                if expr == "true" {
                    Ok(Value::Boolean(true))
                } else if expr == "false" {
                    Ok(Value::Boolean(false))
                } else if expr == "null" {
                    Ok(Value::Null)
                } else if let Ok(n) = expr.parse::<f64>() {
                    Ok(Value::Number(n))
                } else if let Ok(i) = expr.parse::<i64>() {
                    Ok(Value::Integer(i))
                } else {
                    // Try to parse as literal using simple parsing
                    if expr == "true" {
                        Ok(Value::Boolean(true))
                    } else if expr == "false" {
                        Ok(Value::Boolean(false))
                    } else if expr == "null" {
                        Ok(Value::Null)
                    } else if let Ok(n) = expr.parse::<f64>() {
                        Ok(Value::Number(n))
                    } else if let Ok(i) = expr.parse::<i64>() {
                        Ok(Value::Integer(i))
                    } else {
                        Ok(value.clone())
                    }
                }
            }
            _ => Ok(value.clone()),
        }
    }

    /// Try to evaluate simple arithmetic expressions
    /// Supports: +, -, *, /
    fn try_evaluate_arithmetic(&self, expr: &str, facts: &Facts) -> Option<Value> {
        // Check for division
        if let Some(div_pos) = expr.find(" / ") {
            let left = expr[..div_pos].trim();
            let right = expr[div_pos + 3..].trim();

            let left_val = self.get_numeric_value(left, facts)?;
            let right_val = self.get_numeric_value(right, facts)?;

            if right_val != 0.0 {
                return Some(Value::Number(left_val / right_val));
            }
            return None;
        }

        // Check for multiplication
        if let Some(mul_pos) = expr.find(" * ") {
            let left = expr[..mul_pos].trim();
            let right = expr[mul_pos + 3..].trim();

            let left_val = self.get_numeric_value(left, facts)?;
            let right_val = self.get_numeric_value(right, facts)?;

            return Some(Value::Number(left_val * right_val));
        }

        // Check for addition
        if let Some(add_pos) = expr.find(" + ") {
            let left = expr[..add_pos].trim();
            let right = expr[add_pos + 3..].trim();

            let left_val = self.get_numeric_value(left, facts)?;
            let right_val = self.get_numeric_value(right, facts)?;

            return Some(Value::Number(left_val + right_val));
        }

        // Check for subtraction
        if let Some(sub_pos) = expr.find(" - ") {
            let left = expr[..sub_pos].trim();
            let right = expr[sub_pos + 3..].trim();

            let left_val = self.get_numeric_value(left, facts)?;
            let right_val = self.get_numeric_value(right, facts)?;

            return Some(Value::Number(left_val - right_val));
        }

        None
    }

    /// Get numeric value from field name or literal
    fn get_numeric_value(&self, s: &str, facts: &Facts) -> Option<f64> {
        // Try parsing as number first
        if let Ok(n) = s.parse::<f64>() {
            return Some(n);
        }

        // Try getting from facts
        if let Some(val) = facts.get(s).or_else(|| facts.get_nested(s)) {
            match val {
                Value::Number(n) => Some(n),
                Value::Integer(i) => Some(i as f64),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Operator;

    #[test]
    fn test_evaluate_simple_condition() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Age", Value::Number(25.0));

        let condition = Condition::new(
            "User.Age".to_string(),
            Operator::GreaterThan,
            Value::Number(18.0),
        );

        let result = executor.evaluate_condition(&condition, &facts).unwrap();
        assert!(result);
    }

    #[test]
    fn test_evaluate_function_call_len() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Name", Value::String("John".to_string()));

        let condition = Condition::with_function(
            "len".to_string(),
            vec!["User.Name".to_string()],
            Operator::GreaterThan,
            Value::Number(3.0),
        );

        let result = executor.evaluate_condition(&condition, &facts).unwrap();
        assert!(result); // "John".len() = 4 > 3
    }

    #[test]
    fn test_execute_set_action() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();

        let action = ActionType::Set {
            field: "User.IsVIP".to_string(),
            value: Value::Boolean(true),
        };

        executor.execute_action(None, &action, &mut facts).unwrap();

        assert_eq!(facts.get("User.IsVIP"), Some(Value::Boolean(true)));
    }

    #[test]
    fn test_evaluate_compound_and_condition() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Age", Value::Number(25.0));
        facts.set("User.Country", Value::String("US".to_string()));

        let conditions = ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition::new(
                "User.Age".to_string(),
                Operator::GreaterThan,
                Value::Number(18.0),
            ))),
            operator: crate::types::LogicalOperator::And,
            right: Box::new(ConditionGroup::Single(Condition::new(
                "User.Country".to_string(),
                Operator::Equal,
                Value::String("US".to_string()),
            ))),
        };

        let result = executor.evaluate_conditions(&conditions, &facts).unwrap();
        assert!(result);
    }

    #[test]
    fn test_evaluate_compound_or_condition() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Age", Value::Number(15.0));
        facts.set("User.HasParentalConsent", Value::Boolean(true));

        let conditions = ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition::new(
                "User.Age".to_string(),
                Operator::GreaterThan,
                Value::Number(18.0),
            ))),
            operator: crate::types::LogicalOperator::Or,
            right: Box::new(ConditionGroup::Single(Condition::new(
                "User.HasParentalConsent".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            ))),
        };

        let result = executor.evaluate_conditions(&conditions, &facts).unwrap();
        assert!(result); // True because HasParentalConsent is true
    }

    #[test]
    fn test_evaluate_not_condition() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.IsBanned", Value::Boolean(false));

        let conditions = ConditionGroup::Not(Box::new(ConditionGroup::Single(
            Condition::new(
                "User.IsBanned".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )
        )));

        let result = executor.evaluate_conditions(&conditions, &facts).unwrap();
        assert!(result); // True because NOT banned
    }

    #[test]
    fn test_evaluate_function_isempty() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Description", Value::String("".to_string()));

        let condition = Condition::with_function(
            "isEmpty".to_string(),
            vec!["User.Description".to_string()],
            Operator::Equal,
            Value::Boolean(true),
        );

        let result = executor.evaluate_condition(&condition, &facts).unwrap();
        assert!(result); // Empty string
    }

    #[test]
    fn test_evaluate_test_expression_exists() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Email", Value::String("user@example.com".to_string()));

        let condition = Condition {
            field: "User.Email".to_string(),
            expression: crate::engine::rule::ConditionExpression::Test {
                name: "exists".to_string(),
                args: vec!["User.Email".to_string()],
            },
            operator: Operator::Equal,
            value: Value::Boolean(true),
        };

        let result = executor.evaluate_condition(&condition, &facts).unwrap();
        assert!(result);
    }

    #[test]
    fn test_execute_log_action() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();

        let action = ActionType::Log {
            message: "Test log message".to_string(),
        };

        // Should not panic
        executor.execute_action(None, &action, &mut facts).unwrap();
    }

    #[test]
    fn test_try_execute_rule_success() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Age", Value::Number(25.0));

        let conditions = ConditionGroup::Single(Condition::new(
            "User.Age".to_string(),
            Operator::GreaterThan,
            Value::Number(18.0),
        ));

        let actions = vec![ActionType::Set {
            field: "User.IsAdult".to_string(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new("CheckAdult".to_string(), conditions, actions);

        let executed = executor.try_execute_rule(&rule, &mut facts).unwrap();
        assert!(executed);
        assert_eq!(facts.get("User.IsAdult"), Some(Value::Boolean(true)));
    }

    #[test]
    fn test_try_execute_rule_failure() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Age", Value::Number(15.0));

        let conditions = ConditionGroup::Single(Condition::new(
            "User.Age".to_string(),
            Operator::GreaterThan,
            Value::Number(18.0),
        ));

        let actions = vec![ActionType::Set {
            field: "User.IsAdult".to_string(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new("CheckAdult".to_string(), conditions, actions);

        let executed = executor.try_execute_rule(&rule, &mut facts).unwrap();
        assert!(!executed); // Conditions not met
        assert_eq!(facts.get("User.IsAdult"), None); // Action not executed
    }

    #[test]
    fn test_evaluate_string_operators() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Email", Value::String("user@example.com".to_string()));

        // Test Contains
        let condition = Condition::new(
            "User.Email".to_string(),
            Operator::Contains,
            Value::String("@example".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test StartsWith
        let condition = Condition::new(
            "User.Email".to_string(),
            Operator::StartsWith,
            Value::String("user".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test EndsWith
        let condition = Condition::new(
            "User.Email".to_string(),
            Operator::EndsWith,
            Value::String(".com".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());
    }

    #[test]
    fn test_evaluate_numeric_operators() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("Order.Amount", Value::Number(1500.0));

        // Test GreaterThanOrEqual
        let condition = Condition::new(
            "Order.Amount".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Number(1500.0),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test LessThan
        let condition = Condition::new(
            "Order.Amount".to_string(),
            Operator::LessThan,
            Value::Number(2000.0),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test NotEqual
        let condition = Condition::new(
            "Order.Amount".to_string(),
            Operator::NotEqual,
            Value::Number(1000.0),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());
    }

    #[test]
    fn test_evaluate_missing_field() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let facts = Facts::new(); // Empty facts

        let condition = Condition::new(
            "User.Age".to_string(),
            Operator::GreaterThan,
            Value::Number(18.0),
        );

        let result = executor.evaluate_condition(&condition, &facts).unwrap();
        assert!(!result); // Missing field evaluates to false
    }

    #[test]
    fn test_execute_multiple_actions() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Points", Value::Number(150.0));

        let conditions = ConditionGroup::Single(Condition::new(
            "User.Points".to_string(),
            Operator::GreaterThan,
            Value::Number(100.0),
        ));

        let actions = vec![
            ActionType::Set {
                field: "User.IsVIP".to_string(),
                value: Value::Boolean(true),
            },
            ActionType::Log {
                message: "User promoted to VIP".to_string(),
            },
            ActionType::Set {
                field: "User.Discount".to_string(),
                value: Value::Number(0.2),
            },
        ];

        let rule = Rule::new("PromoteToVIP".to_string(), conditions, actions);

        let executed = executor.try_execute_rule(&rule, &mut facts).unwrap();
        assert!(executed);
        assert_eq!(facts.get("User.IsVIP"), Some(Value::Boolean(true)));
        assert_eq!(facts.get("User.Discount"), Some(Value::Number(0.2)));
    }

    #[test]
    fn test_evaluate_endswith_operator() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Email", Value::String("user@example.com".to_string()));
        facts.set("File.Name", Value::String("document.pdf".to_string()));
        facts.set("Domain.URL", Value::String("https://api.example.org".to_string()));

        // Test EndsWith with .com suffix
        let condition = Condition::new(
            "User.Email".to_string(),
            Operator::EndsWith,
            Value::String(".com".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test EndsWith with .pdf suffix
        let condition = Condition::new(
            "File.Name".to_string(),
            Operator::EndsWith,
            Value::String(".pdf".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test EndsWith with .org suffix
        let condition = Condition::new(
            "Domain.URL".to_string(),
            Operator::EndsWith,
            Value::String(".org".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test EndsWith that should fail
        let condition = Condition::new(
            "User.Email".to_string(),
            Operator::EndsWith,
            Value::String(".net".to_string()),
        );
        assert!(!executor.evaluate_condition(&condition, &facts).unwrap());

        // Test EndsWith with full string match
        let condition = Condition::new(
            "File.Name".to_string(),
            Operator::EndsWith,
            Value::String("document.pdf".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());
    }

    #[test]
    fn test_evaluate_endswith_edge_cases() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("Empty.String", Value::String("".to_string()));
        facts.set("Single.Char", Value::String("a".to_string()));
        facts.set("Number.Value", Value::Number(123.0));

        // Test EndsWith with empty string (should match everything)
        let condition = Condition::new(
            "Empty.String".to_string(),
            Operator::EndsWith,
            Value::String("".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test EndsWith on single character
        let condition = Condition::new(
            "Single.Char".to_string(),
            Operator::EndsWith,
            Value::String("a".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test EndsWith with non-string value (should fail gracefully)
        let condition = Condition::new(
            "Number.Value".to_string(),
            Operator::EndsWith,
            Value::String(".0".to_string()),
        );
        assert!(!executor.evaluate_condition(&condition, &facts).unwrap());

        // Test EndsWith on missing field (should fail gracefully)
        let condition = Condition::new(
            "Missing.Field".to_string(),
            Operator::EndsWith,
            Value::String("test".to_string()),
        );
        assert!(!executor.evaluate_condition(&condition, &facts).unwrap());

        // Test case sensitivity
        let mut facts2 = Facts::new();
        facts2.set("Text.Value", Value::String("HelloWorld".to_string()));

        let condition = Condition::new(
            "Text.Value".to_string(),
            Operator::EndsWith,
            Value::String("world".to_string()),
        );
        assert!(!executor.evaluate_condition(&condition, &facts2).unwrap()); // Should fail due to case mismatch

        let condition = Condition::new(
            "Text.Value".to_string(),
            Operator::EndsWith,
            Value::String("World".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts2).unwrap()); // Should pass with correct case
    }

    #[test]
    fn test_evaluate_matches_operator() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("User.Email", Value::String("user@example.com".to_string()));
        facts.set("Product.Name", Value::String("Premium Laptop Model X".to_string()));
        facts.set("Log.Message", Value::String("Error: Connection timeout".to_string()));

        // Test Matches with pattern "example"
        let condition = Condition::new(
            "User.Email".to_string(),
            Operator::Matches,
            Value::String("example".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test Matches with pattern "Premium"
        let condition = Condition::new(
            "Product.Name".to_string(),
            Operator::Matches,
            Value::String("Premium".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test Matches with pattern "Error"
        let condition = Condition::new(
            "Log.Message".to_string(),
            Operator::Matches,
            Value::String("Error".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test Matches that should fail
        let condition = Condition::new(
            "User.Email".to_string(),
            Operator::Matches,
            Value::String("notfound".to_string()),
        );
        assert!(!executor.evaluate_condition(&condition, &facts).unwrap());

        // Test Matches with partial pattern
        let condition = Condition::new(
            "Product.Name".to_string(),
            Operator::Matches,
            Value::String("Laptop".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test Matches with full string
        let condition = Condition::new(
            "Log.Message".to_string(),
            Operator::Matches,
            Value::String("Error: Connection timeout".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());
    }

    #[test]
    fn test_evaluate_matches_edge_cases() {
        let kb = KnowledgeBase::new("test");
        let executor = RuleExecutor::new(kb);

        let mut facts = Facts::new();
        facts.set("Empty.String", Value::String("".to_string()));
        facts.set("Single.Char", Value::String("x".to_string()));
        facts.set("Number.Value", Value::Number(456.0));
        facts.set("Special.Chars", Value::String("test@#$%^&*()".to_string()));

        // Test Matches with empty pattern (should match empty string)
        let condition = Condition::new(
            "Empty.String".to_string(),
            Operator::Matches,
            Value::String("".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test Matches on single character
        let condition = Condition::new(
            "Single.Char".to_string(),
            Operator::Matches,
            Value::String("x".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test Matches with non-string value (should fail gracefully)
        let condition = Condition::new(
            "Number.Value".to_string(),
            Operator::Matches,
            Value::String("456".to_string()),
        );
        assert!(!executor.evaluate_condition(&condition, &facts).unwrap());

        // Test Matches on missing field (should fail gracefully)
        let condition = Condition::new(
            "Missing.Field".to_string(),
            Operator::Matches,
            Value::String("pattern".to_string()),
        );
        assert!(!executor.evaluate_condition(&condition, &facts).unwrap());

        // Test Matches with special characters
        let condition = Condition::new(
            "Special.Chars".to_string(),
            Operator::Matches,
            Value::String("@#$".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts).unwrap());

        // Test case sensitivity
        let mut facts2 = Facts::new();
        facts2.set("Text.Value", Value::String("HelloWorld".to_string()));

        let condition = Condition::new(
            "Text.Value".to_string(),
            Operator::Matches,
            Value::String("hello".to_string()),
        );
        assert!(!executor.evaluate_condition(&condition, &facts2).unwrap()); // Should fail due to case mismatch

        let condition = Condition::new(
            "Text.Value".to_string(),
            Operator::Matches,
            Value::String("Hello".to_string()),
        );
        assert!(executor.evaluate_condition(&condition, &facts2).unwrap()); // Should pass with correct case
    }

    #[test]
    fn test_endswith_matches_in_rules() {
        // Integration test: EndsWith and Matches in actual rules
        let kb = KnowledgeBase::new("test");

        // Rule 1: If email ends with .edu, set IsStudent = true
        let condition1 = Condition::new(
            "User.Email".to_string(),
            Operator::EndsWith,
            Value::String(".edu".to_string()),
        );
        let actions1 = vec![ActionType::Set {
            field: "User.IsStudent".to_string(),
            value: Value::Boolean(true),
        }];
        let rule1 = Rule::new(
            "StudentEmailRule".to_string(),
            ConditionGroup::Single(condition1),
            actions1,
        );

        // Rule 2: If product name matches "Premium", set IsPremium = true
        let condition2 = Condition::new(
            "Product.Name".to_string(),
            Operator::Matches,
            Value::String("Premium".to_string()),
        );
        let actions2 = vec![ActionType::Set {
            field: "Product.IsPremium".to_string(),
            value: Value::Boolean(true),
        }];
        let rule2 = Rule::new(
            "PremiumProductRule".to_string(),
            ConditionGroup::Single(condition2),
            actions2,
        );

        let _ = kb.add_rule(rule1.clone());
        let _ = kb.add_rule(rule2.clone());

        let executor = RuleExecutor::new(kb);

        // Test scenario 1: Student email
        let mut facts1 = Facts::new();
        facts1.set("User.Email", Value::String("student@university.edu".to_string()));

        let executed = executor.try_execute_rule(&rule1, &mut facts1).unwrap();
        assert!(executed);
        assert_eq!(facts1.get("User.IsStudent"), Some(Value::Boolean(true)));

        // Test scenario 2: Premium product
        let mut facts2 = Facts::new();
        facts2.set("Product.Name", Value::String("Premium Laptop X1".to_string()));

        let executed = executor.try_execute_rule(&rule2, &mut facts2).unwrap();
        assert!(executed);
        assert_eq!(facts2.get("Product.IsPremium"), Some(Value::Boolean(true)));

        // Test scenario 3: Non-matching cases
        let mut facts3 = Facts::new();
        facts3.set("User.Email", Value::String("user@company.com".to_string()));

        let executed = executor.try_execute_rule(&rule1, &mut facts3).unwrap();
        assert!(!executed); // Should not execute because email doesn't end with .edu
    }
}

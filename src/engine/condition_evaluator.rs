//! Shared condition evaluation logic for both forward and backward chaining
//!
//! This module provides a unified interface for evaluating rule conditions
//! that can be used by both RustRuleEngine (forward chaining) and
//! BackwardEngine (backward chaining).

#![allow(deprecated)]

use crate::engine::rule::{Condition, ConditionExpression, ConditionGroup};
use crate::errors::{Result, RuleEngineError};
use crate::types::{Operator, Value};
use crate::Facts;
use std::collections::HashMap;

/// Type for custom function implementations
pub type CustomFunction = Box<dyn Fn(&[Value], &Facts) -> Result<Value> + Send + Sync>;

/// Shared condition evaluator that works for both forward and backward chaining
pub struct ConditionEvaluator {
    /// Custom functions registered by user (optional - for forward chaining)
    custom_functions: Option<HashMap<String, CustomFunction>>,

    /// Whether to use built-in hardcoded functions (for backward chaining)
    use_builtin_functions: bool,
}

impl ConditionEvaluator {
    /// Create new evaluator with custom functions (for forward chaining)
    pub fn with_custom_functions(custom_functions: HashMap<String, CustomFunction>) -> Self {
        Self {
            custom_functions: Some(custom_functions),
            use_builtin_functions: false,
        }
    }

    /// Create new evaluator with built-in functions (for backward chaining)
    pub fn with_builtin_functions() -> Self {
        Self {
            custom_functions: None,
            use_builtin_functions: true,
        }
    }

    /// Evaluate condition group
    pub fn evaluate_conditions(&self, group: &ConditionGroup, facts: &Facts) -> Result<bool> {
        match group {
            ConditionGroup::Single(condition) => self.evaluate_condition(condition, facts),

            ConditionGroup::Compound {
                left,
                operator,
                right,
            } => {
                let left_result = self.evaluate_conditions(left, facts)?;

                // Short-circuit evaluation
                match operator {
                    crate::types::LogicalOperator::And => {
                        if !left_result {
                            return Ok(false);
                        }
                        self.evaluate_conditions(right, facts)
                    }
                    crate::types::LogicalOperator::Or => {
                        if left_result {
                            return Ok(true);
                        }
                        self.evaluate_conditions(right, facts)
                    }
                    crate::types::LogicalOperator::Not => Err(RuleEngineError::ExecutionError(
                        "NOT operator should not appear in compound conditions".to_string(),
                    )),
                }
            }

            ConditionGroup::Not(inner) => {
                let result = self.evaluate_conditions(inner, facts)?;
                Ok(!result)
            }

            ConditionGroup::Exists(conditions) => {
                // Simplified exists for backward chaining
                self.evaluate_conditions(conditions, facts)
            }

            ConditionGroup::Forall(conditions) => {
                // Simplified forall for backward chaining
                self.evaluate_conditions(conditions, facts)
            }

            ConditionGroup::Accumulate { .. } => {
                // Accumulate needs special handling - not fully supported yet
                Ok(true)
            }

            #[cfg(feature = "streaming")]
            ConditionGroup::StreamPattern { .. } => {
                // Stream patterns need special handling in streaming engine
                // Not fully supported in backward chaining context
                Ok(true)
            }
        }
    }

    /// Evaluate a single condition
    pub fn evaluate_condition(&self, condition: &Condition, facts: &Facts) -> Result<bool> {
        match &condition.expression {
            ConditionExpression::Field(field_name) => {
                // Get field value
                if let Some(value) = facts
                    .get_nested(field_name)
                    .or_else(|| facts.get(field_name))
                {
                    Ok(condition.operator.evaluate(&value, &condition.value))
                } else {
                    // Field not found
                    // For some operators like NotEqual, this might be true
                    match condition.operator {
                        Operator::NotEqual => {
                            // null != value is true
                            Ok(true)
                        }
                        _ => Ok(false),
                    }
                }
            }

            ConditionExpression::FunctionCall { name, args } => {
                self.evaluate_function_call(name, args, condition, facts)
            }

            ConditionExpression::Test { name, args } => {
                self.evaluate_test_expression(name, args, facts)
            }

            ConditionExpression::MultiField {
                field,
                operation,
                variable,
            } => self.evaluate_multifield(field, operation, variable, condition, facts),
        }
    }

    /// Evaluate function call
    fn evaluate_function_call(
        &self,
        function_name: &str,
        args: &[String],
        condition: &Condition,
        facts: &Facts,
    ) -> Result<bool> {
        // Try custom functions first (if available)
        if let Some(custom_fns) = &self.custom_functions {
            if let Some(function) = custom_fns.get(function_name) {
                // Resolve arguments from facts
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| {
                        facts
                            .get_nested(arg)
                            .or_else(|| facts.get(arg))
                            .unwrap_or_else(|| {
                                self.parse_literal_value(arg)
                                    .unwrap_or(Value::String(arg.clone()))
                            })
                    })
                    .collect();

                // Call the function
                match function(&arg_values, facts) {
                    Ok(result_value) => {
                        return Ok(condition.operator.evaluate(&result_value, &condition.value));
                    }
                    Err(_) => return Ok(false),
                }
            }
        }

        // Fall back to built-in functions if enabled
        if self.use_builtin_functions {
            return self.evaluate_builtin_function(function_name, args, condition, facts);
        }

        // Function not found
        Ok(false)
    }

    /// Evaluate built-in functions (hardcoded for backward chaining)
    fn evaluate_builtin_function(
        &self,
        function_name: &str,
        args: &[String],
        condition: &Condition,
        facts: &Facts,
    ) -> Result<bool> {
        // Get function arguments
        let mut arg_values = Vec::new();
        for arg in args {
            if let Some(value) = facts.get(arg).or_else(|| facts.get_nested(arg)) {
                arg_values.push(value);
            } else {
                // Try to parse as literal
                if let Ok(val) = self.parse_literal_value(arg) {
                    arg_values.push(val);
                } else {
                    // Argument not available - cannot evaluate
                    return Ok(false);
                }
            }
        }

        match function_name {
            "len" | "length" | "size" => {
                if arg_values.len() == 1 {
                    let len = match &arg_values[0] {
                        Value::String(s) => s.len() as f64,
                        Value::Array(arr) => arr.len() as f64,
                        _ => return Ok(false),
                    };

                    Ok(condition
                        .operator
                        .evaluate(&Value::Number(len), &condition.value))
                } else {
                    Ok(false)
                }
            }

            "isEmpty" | "is_empty" => {
                if arg_values.len() == 1 {
                    let is_empty = match &arg_values[0] {
                        Value::String(s) => s.is_empty(),
                        Value::Array(arr) => arr.is_empty(),
                        Value::Null => true,
                        _ => false,
                    };

                    Ok(condition
                        .operator
                        .evaluate(&Value::Boolean(is_empty), &condition.value))
                } else {
                    Ok(false)
                }
            }

            "contains" => {
                if arg_values.len() == 2 {
                    let contains = match (&arg_values[0], &arg_values[1]) {
                        (Value::String(s), Value::String(substr)) => s.contains(substr.as_str()),
                        (Value::Array(arr), val) => arr.contains(val),
                        _ => false,
                    };

                    Ok(condition
                        .operator
                        .evaluate(&Value::Boolean(contains), &condition.value))
                } else {
                    Ok(false)
                }
            }

            _ => {
                // Unknown function
                Ok(false)
            }
        }
    }

    /// Evaluate test expression
    fn evaluate_test_expression(
        &self,
        function_name: &str,
        args: &[String],
        facts: &Facts,
    ) -> Result<bool> {
        // Try custom functions first
        if let Some(custom_fns) = &self.custom_functions {
            if let Some(function) = custom_fns.get(function_name) {
                let arg_values: Vec<Value> = args
                    .iter()
                    .map(|arg| {
                        facts
                            .get_nested(arg)
                            .or_else(|| facts.get(arg))
                            .unwrap_or(Value::String(arg.clone()))
                    })
                    .collect();

                match function(&arg_values, facts) {
                    Ok(result_value) => return Ok(result_value.to_bool()),
                    Err(_) => return Ok(false),
                }
            }
        }

        // Built-in test expressions
        if self.use_builtin_functions {
            return self.evaluate_builtin_test(function_name, args, facts);
        }

        Ok(false)
    }

    /// Evaluate built-in test expressions
    fn evaluate_builtin_test(
        &self,
        function_name: &str,
        args: &[String],
        facts: &Facts,
    ) -> Result<bool> {
        match function_name {
            "exists" => {
                // Check if field exists
                if args.len() == 1 {
                    Ok(facts.get(&args[0]).is_some() || facts.get_nested(&args[0]).is_some())
                } else {
                    Ok(false)
                }
            }

            "notExists" | "not_exists" => {
                // Check if field does not exist
                if args.len() == 1 {
                    Ok(facts.get(&args[0]).is_none() && facts.get_nested(&args[0]).is_none())
                } else {
                    Ok(false)
                }
            }

            _ => {
                // Unknown test function
                Ok(false)
            }
        }
    }

    /// Evaluate multi-field operation
    fn evaluate_multifield(
        &self,
        field: &str,
        operation: &str,
        _variable: &Option<String>,
        condition: &Condition,
        facts: &Facts,
    ) -> Result<bool> {
        // Get field value
        let field_value = facts.get(field).or_else(|| facts.get_nested(field));

        match operation {
            "collect" => {
                // Collect all values - just check if field exists
                Ok(field_value.is_some())
            }

            "count" => {
                // Count elements
                let count = if let Some(value) = field_value {
                    match value {
                        Value::Array(arr) => arr.len() as f64,
                        _ => 1.0,
                    }
                } else {
                    0.0
                };

                Ok(condition
                    .operator
                    .evaluate(&Value::Number(count), &condition.value))
            }

            "first" => {
                // Get first element
                if let Some(Value::Array(arr)) = field_value {
                    Ok(!arr.is_empty())
                } else {
                    Ok(false)
                }
            }

            "last" => {
                // Get last element
                if let Some(Value::Array(arr)) = field_value {
                    Ok(!arr.is_empty())
                } else {
                    Ok(false)
                }
            }

            "empty" | "isEmpty" => {
                // Check if empty
                let is_empty = if let Some(value) = field_value {
                    match value {
                        Value::Array(arr) => arr.is_empty(),
                        Value::String(s) => s.is_empty(),
                        Value::Null => true,
                        _ => false,
                    }
                } else {
                    true
                };

                Ok(is_empty)
            }

            "not_empty" | "notEmpty" => {
                // Check if not empty
                let is_not_empty = if let Some(value) = field_value {
                    match value {
                        Value::Array(arr) => !arr.is_empty(),
                        Value::String(s) => !s.is_empty(),
                        Value::Null => false,
                        _ => true,
                    }
                } else {
                    false
                };

                Ok(is_not_empty)
            }

            "contains" => {
                // Check if array contains value
                if let Some(Value::Array(arr)) = field_value {
                    Ok(arr.contains(&condition.value))
                } else {
                    Ok(false)
                }
            }

            _ => {
                // Unknown operation
                Ok(false)
            }
        }
    }

    /// Parse literal value from string
    fn parse_literal_value(&self, s: &str) -> Result<Value> {
        // Try boolean
        if s == "true" {
            return Ok(Value::Boolean(true));
        }
        if s == "false" {
            return Ok(Value::Boolean(false));
        }
        if s == "null" {
            return Ok(Value::Null);
        }

        // Try number
        if let Ok(n) = s.parse::<f64>() {
            return Ok(Value::Number(n));
        }

        // Try integer
        if let Ok(i) = s.parse::<i64>() {
            return Ok(Value::Integer(i));
        }

        // String
        Ok(Value::String(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_function_len() {
        let evaluator = ConditionEvaluator::with_builtin_functions();
        let facts = Facts::new();
        facts.set("User.Name", Value::String("John".to_string()));

        let condition = Condition::with_function(
            "len".to_string(),
            vec!["User.Name".to_string()],
            Operator::GreaterThan,
            Value::Number(3.0),
        );

        let result = evaluator.evaluate_condition(&condition, &facts).unwrap();
        assert!(result); // "John".len() = 4 > 3
    }

    #[test]
    fn test_builtin_test_exists() {
        let evaluator = ConditionEvaluator::with_builtin_functions();
        let facts = Facts::new();
        facts.set("User.Email", Value::String("test@example.com".to_string()));

        let result = evaluator
            .evaluate_builtin_test("exists", &["User.Email".to_string()], &facts)
            .unwrap();
        assert!(result);

        let result2 = evaluator
            .evaluate_builtin_test("exists", &["User.Missing".to_string()], &facts)
            .unwrap();
        assert!(!result2);
    }

    #[test]
    fn test_multifield_count() {
        let evaluator = ConditionEvaluator::with_builtin_functions();
        let facts = Facts::new();
        facts.set(
            "User.Orders",
            Value::Array(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
        );

        let condition = Condition {
            field: "User.Orders".to_string(),
            expression: ConditionExpression::MultiField {
                field: "User.Orders".to_string(),
                operation: "count".to_string(),
                variable: None,
            },
            operator: Operator::Equal,
            value: Value::Number(3.0),
        };

        let result = evaluator.evaluate_condition(&condition, &facts).unwrap();
        assert!(result);
    }
}

/// Rule execution for backward chaining
///
/// This module provides proper condition evaluation and action execution
/// for backward chaining, replacing the "fake" stub implementations.

use crate::engine::rule::{Condition, ConditionExpression, ConditionGroup, Rule};
use crate::types::{ActionType, Value};
use crate::{Facts, KnowledgeBase};
use crate::errors::{Result, RuleEngineError};

/// Rule executor for backward chaining
pub struct RuleExecutor {
    knowledge_base: KnowledgeBase,
}

impl RuleExecutor {
    /// Create a new rule executor
    pub fn new(knowledge_base: KnowledgeBase) -> Self {
        Self { knowledge_base }
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
        match group {
            ConditionGroup::Single(condition) => {
                self.evaluate_condition(condition, facts)
            }

            ConditionGroup::Compound { left, operator, right } => {
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
                    _ => {
                        Err(RuleEngineError::ExecutionError(
                            format!("Unsupported logical operator: {:?}", operator)
                        ))
                    }
                }
            }

            ConditionGroup::Not(inner) => {
                let result = self.evaluate_conditions(inner, facts)?;
                Ok(!result)
            }

            ConditionGroup::Exists(conditions) => {
                // For backward chaining, exists is simplified
                // Just check if the conditions can be satisfied
                self.evaluate_conditions(conditions, facts)
            }

            ConditionGroup::Forall(conditions) => {
                // Forall is complex - for now, simplified implementation
                // Just evaluate the nested conditions
                self.evaluate_conditions(conditions, facts)
            }

            ConditionGroup::Accumulate { .. } => {
                // Accumulate needs special handling - not fully supported in BC yet
                // Return true for now (TODO: proper implementation)
                Ok(true)
            }
        }
    }

    /// Evaluate a single condition
    pub fn evaluate_condition(&self, condition: &Condition, facts: &Facts) -> Result<bool> {
        match &condition.expression {
            ConditionExpression::Field(field_name) => {
                // Get field value
                if let Some(value) = facts.get_nested(field_name).or_else(|| facts.get(field_name)) {
                    Ok(condition.operator.evaluate(&value, &condition.value))
                } else {
                    // Field not found
                    // For some operators like NotEqual, this might be true
                    match condition.operator {
                        crate::types::Operator::NotEqual => {
                            // null != value is true
                            Ok(true)
                        }
                        _ => Ok(false),
                    }
                }
            }

            ConditionExpression::FunctionCall { name, args } => {
                // Execute function call
                self.evaluate_function_call(name, args, condition, facts)
            }

            ConditionExpression::Test { name, args } => {
                // Execute test expression
                self.evaluate_test_expression(name, args, facts)
            }

            ConditionExpression::MultiField { field, operation, variable } => {
                // Handle multi-field operations
                self.evaluate_multifield(field, operation, variable, condition, facts)
            }
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
        // For backward chaining, we need to check if the function is available
        // and execute it with the current facts

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

        // TODO: Proper function execution
        // For now, check for common functions

        match function_name {
            "len" | "length" | "size" => {
                if arg_values.len() == 1 {
                    let len = match &arg_values[0] {
                        Value::String(s) => s.len() as f64,
                        Value::Array(arr) => arr.len() as f64,
                        _ => return Ok(false),
                    };

                    Ok(condition.operator.evaluate(&Value::Number(len), &condition.value))
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

                    Ok(condition.operator.evaluate(&Value::Boolean(is_empty), &condition.value))
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

                    Ok(condition.operator.evaluate(&Value::Boolean(contains), &condition.value))
                } else {
                    Ok(false)
                }
            }

            _ => {
                // Unknown function - cannot evaluate in backward chaining
                // Return false instead of true (fix the "lie")
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
        // Test expressions must return boolean
        // For now, support common test functions

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
        variable: &Option<String>,
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

                Ok(condition.operator.evaluate(&Value::Number(count), &condition.value))
            }

            "first" => {
                // Get first element
                if let Some(Value::Array(arr)) = field_value {
                    if !arr.is_empty() {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }

            "last" => {
                // Get last element
                if let Some(Value::Array(arr)) = field_value {
                    if !arr.is_empty() {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
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

    /// Execute rule actions
    fn execute_actions(&self, rule: &Rule, facts: &mut Facts) -> Result<()> {
        for action in &rule.actions {
            self.execute_action(action, facts)?;
        }

        Ok(())
    }

    /// Execute a single action
    fn execute_action(&self, action: &ActionType, facts: &mut Facts) -> Result<()> {
        match action {
            ActionType::Set { field, value } => {
                // Evaluate value expression if needed
                let evaluated_value = self.evaluate_value_expression(value, facts)?;
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

    /// Evaluate value expression
    fn evaluate_value_expression(&self, value: &Value, facts: &Facts) -> Result<Value> {
        match value {
            Value::Expression(expr) => {
                // Parse and evaluate expression
                // For now, simple field reference or literal
                if let Some(val) = facts.get(expr).or_else(|| facts.get_nested(expr)) {
                    Ok(val)
                } else if let Ok(lit) = self.parse_literal_value(expr) {
                    Ok(lit)
                } else {
                    Ok(value.clone())
                }
            }
            _ => Ok(value.clone()),
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

        executor.execute_action(&action, &mut facts).unwrap();

        assert_eq!(facts.get("User.IsVIP"), Some(Value::Boolean(true)));
    }
}

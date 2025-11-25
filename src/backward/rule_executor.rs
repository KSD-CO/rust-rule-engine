/// Rule execution for backward chaining
///
/// This module provides proper condition evaluation and action execution
/// for backward chaining, replacing the "fake" stub implementations.

use crate::engine::rule::{Condition, ConditionGroup, Rule};
use crate::engine::condition_evaluator::ConditionEvaluator;
use crate::types::{ActionType, Value};
use crate::{Facts, KnowledgeBase};
use crate::errors::{Result, RuleEngineError};

/// Rule executor for backward chaining
pub struct RuleExecutor {
    knowledge_base: KnowledgeBase,
    evaluator: ConditionEvaluator,
}

impl RuleExecutor {
    /// Create a new rule executor
    pub fn new(knowledge_base: KnowledgeBase) -> Self {
        Self {
            knowledge_base,
            evaluator: ConditionEvaluator::with_builtin_functions(),
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

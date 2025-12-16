use crate::engine::plugin::{PluginHealth, PluginMetadata, PluginState, RulePlugin};
use crate::engine::RustRuleEngine;
use crate::errors::{Result, RuleEngineError};
use crate::types::Value;

/// Built-in plugin for mathematical operations
pub struct MathUtilsPlugin {
    metadata: PluginMetadata,
}

impl Default for MathUtilsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl MathUtilsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "math-utils".to_string(),
                version: "1.0.0".to_string(),
                description: "Mathematical operations and utilities".to_string(),
                author: "Rust Rule Engine Team".to_string(),
                state: PluginState::Loaded,
                health: PluginHealth::Healthy,
                actions: vec![
                    "Add".to_string(),
                    "Subtract".to_string(),
                    "Multiply".to_string(),
                    "Divide".to_string(),
                    "Modulo".to_string(),
                    "Power".to_string(),
                    "Abs".to_string(),
                    "Round".to_string(),
                    "Ceil".to_string(),
                    "Floor".to_string(),
                ],
                functions: vec![
                    "min".to_string(),
                    "max".to_string(),
                    "sqrt".to_string(),
                    "random".to_string(),
                    "sum".to_string(),
                    "avg".to_string(),
                ],
                dependencies: vec![],
            },
        }
    }
}

impl RulePlugin for MathUtilsPlugin {
    fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // Add - Addition
        engine.register_action_handler("Add", |params, facts| {
            let a = get_number_param(params, facts, "a", "0")?;
            let b = get_number_param(params, facts, "b", "1")?;
            let output = get_string_param(params, "output", "2")?;

            let result = a + b;
            facts.set_nested(&output, Value::Number(result))?;
            Ok(())
        });

        // Subtract - Subtraction
        engine.register_action_handler("Subtract", |params, facts| {
            let a = get_number_param(params, facts, "a", "0")?;
            let b = get_number_param(params, facts, "b", "1")?;
            let output = get_string_param(params, "output", "2")?;

            let result = a - b;
            facts.set_nested(&output, Value::Number(result))?;
            Ok(())
        });

        // Multiply - Multiplication
        engine.register_action_handler("Multiply", |params, facts| {
            let a = get_number_param(params, facts, "a", "0")?;
            let b = get_number_param(params, facts, "b", "1")?;
            let output = get_string_param(params, "output", "2")?;

            let result = a * b;
            facts.set_nested(&output, Value::Number(result))?;
            Ok(())
        });

        // Divide - Division
        engine.register_action_handler("Divide", |params, facts| {
            let a = get_number_param(params, facts, "a", "0")?;
            let b = get_number_param(params, facts, "b", "1")?;
            let output = get_string_param(params, "output", "2")?;

            if b == 0.0 {
                return Err(RuleEngineError::ActionError {
                    message: "Division by zero".to_string(),
                });
            }

            let result = a / b;
            facts.set_nested(&output, Value::Number(result))?;
            Ok(())
        });

        // Abs - Absolute value
        engine.register_action_handler("Abs", |params, facts| {
            let a = get_number_param(params, facts, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            let result = a.abs();
            facts.set_nested(&output, Value::Number(result))?;
            Ok(())
        });

        // Round - Round to nearest integer
        engine.register_action_handler("Round", |params, facts| {
            let a = get_number_param(params, facts, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            let result = a.round();
            facts.set_nested(&output, Value::Number(result))?;
            Ok(())
        });

        Ok(())
    }

    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // min - Find minimum value
        engine.register_function("min", |args, _facts| {
            if args.is_empty() {
                return Err(RuleEngineError::EvaluationError {
                    message: "min requires at least 1 argument".to_string(),
                });
            }

            let mut min_val = value_to_number(&args[0])?;
            for arg in &args[1..] {
                let val = value_to_number(arg)?;
                if val < min_val {
                    min_val = val;
                }
            }
            Ok(Value::Number(min_val))
        });

        // max - Find maximum value
        engine.register_function("max", |args, _facts| {
            if args.is_empty() {
                return Err(RuleEngineError::EvaluationError {
                    message: "max requires at least 1 argument".to_string(),
                });
            }

            let mut max_val = value_to_number(&args[0])?;
            for arg in &args[1..] {
                let val = value_to_number(arg)?;
                if val > max_val {
                    max_val = val;
                }
            }
            Ok(Value::Number(max_val))
        });

        // sqrt - Square root
        engine.register_function("sqrt", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "sqrt requires exactly 1 argument".to_string(),
                });
            }

            let val = value_to_number(&args[0])?;
            if val < 0.0 {
                return Err(RuleEngineError::EvaluationError {
                    message: "Cannot calculate square root of negative number".to_string(),
                });
            }

            Ok(Value::Number(val.sqrt()))
        });

        // sum - Sum all values
        engine.register_function("sum", |args, _facts| {
            if args.is_empty() {
                return Ok(Value::Number(0.0));
            }

            let mut total = 0.0;
            for arg in args {
                total += value_to_number(arg)?;
            }
            Ok(Value::Number(total))
        });

        // avg - Average of all values
        engine.register_function("avg", |args, _facts| {
            if args.is_empty() {
                return Err(RuleEngineError::EvaluationError {
                    message: "avg requires at least 1 argument".to_string(),
                });
            }

            let mut total = 0.0;
            for arg in args {
                total += value_to_number(arg)?;
            }
            Ok(Value::Number(total / args.len() as f64))
        });

        Ok(())
    }

    fn unload(&mut self) -> Result<()> {
        self.metadata.state = PluginState::Unloaded;
        Ok(())
    }

    fn health_check(&mut self) -> PluginHealth {
        match self.metadata.state {
            PluginState::Loaded => PluginHealth::Healthy,
            PluginState::Loading => PluginHealth::Warning("Plugin is loading".to_string()),
            PluginState::Error => PluginHealth::Error("Plugin is in error state".to_string()),
            PluginState::Unloaded => PluginHealth::Warning("Plugin is unloaded".to_string()),
        }
    }
}

// Helper functions
fn get_string_param(
    params: &std::collections::HashMap<String, Value>,
    name: &str,
    pos: &str,
) -> Result<String> {
    let value = params
        .get(name)
        .or_else(|| params.get(pos))
        .ok_or_else(|| RuleEngineError::ActionError {
            message: format!("Missing parameter: {}", name),
        })?;

    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err(RuleEngineError::ActionError {
            message: format!("Parameter {} must be string", name),
        }),
    }
}

fn get_number_param(
    params: &std::collections::HashMap<String, Value>,
    facts: &crate::Facts,
    name: &str,
    pos: &str,
) -> Result<f64> {
    let value = params
        .get(name)
        .or_else(|| params.get(pos))
        .ok_or_else(|| RuleEngineError::ActionError {
            message: format!("Missing parameter: {}", name),
        })?;

    // If it's a fact reference, resolve it
    if let Value::String(s) = value {
        if s.contains('.') {
            if let Some(fact_value) = facts.get(s) {
                return value_to_number(&fact_value);
            }
        }
    }

    value_to_number(value)
}

fn value_to_number(value: &Value) -> Result<f64> {
    match value {
        Value::Number(f) => Ok(*f),
        Value::Integer(i) => Ok(*i as f64),
        Value::String(s) => s.parse::<f64>().map_err(|_| RuleEngineError::ActionError {
            message: format!("Cannot convert '{}' to number", s),
        }),
        _ => Err(RuleEngineError::ActionError {
            message: "Value cannot be converted to number".to_string(),
        }),
    }
}

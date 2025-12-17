use rust_rule_engine::engine::plugin::{PluginHealth, PluginMetadata, PluginState, RulePlugin};
use rust_rule_engine::engine::RustRuleEngine;
use rust_rule_engine::errors::{Result, RuleEngineError};
use rust_rule_engine::types::Value;

#[allow(dead_code)]
pub struct StringUtilsPlugin {
    metadata: PluginMetadata,
}

impl StringUtilsPlugin {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "string-utils".to_string(),
                version: "1.0.0".to_string(),
                description: "String utility functions for rule engine".to_string(),
                author: "Rule Engine Team".to_string(),
                state: PluginState::Loaded,
                health: PluginHealth::Healthy,
                actions: vec![
                    "ToUpperCase".to_string(),
                    "ToLowerCase".to_string(),
                    "StringLength".to_string(),
                    "StringContains".to_string(),
                ],
                functions: vec!["concat".to_string(), "repeat".to_string()],
                dependencies: vec![],
            },
        }
    }
}

impl RulePlugin for StringUtilsPlugin {
    fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // ToUpperCase action - handles both positional and named parameters
        engine.register_action_handler("ToUpperCase", |params, facts| {
            // Try named parameters first, then positional
            let input_path = params
                .get("input")
                .or_else(|| params.get("0"))
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "ToUpperCase requires input parameter".to_string(),
                })?;

            let output_path = params
                .get("output")
                .or_else(|| params.get("1"))
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "ToUpperCase requires output parameter".to_string(),
                })?;

            // Get the input path and resolve the value
            let input_path_str = match input_path {
                Value::String(s) => s.clone(),
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: "Input path must be string".to_string(),
                    })
                }
            };

            // Resolve the input value: if `input_path_str` refers to a nested fact (e.g. "User.name")
            // prefer reading from facts; otherwise treat it as a literal string value.
            let input_value = match facts.get_nested(&input_path_str) {
                Some(v) => v,
                None => Value::String(input_path_str.clone()),
            };

            let input_str = match input_value {
                Value::String(s) => s.clone(),
                Value::Integer(i) => i.to_string(),
                Value::Number(f) => f.to_string(),
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: "Input must be convertible to string".to_string(),
                    })
                }
            };

            let output_path_str = match output_path {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };

            let result = input_str.to_uppercase();
            match facts.set_nested(&output_path_str, Value::String(result.clone())) {
                Ok(()) => Ok(()),
                Err(e) => match e {
                    RuleEngineError::FieldNotFound { .. } => {
                        // fallback to adding as top-level fact
                        facts.add_value(&output_path_str, Value::String(result))?;
                        Ok(())
                    }
                    other => Err(other),
                },
            }
        });

        // ToLowerCase action
        engine.register_action_handler("ToLowerCase", |params, facts| {
            let input_path = params
                .get("input")
                .or_else(|| params.get("0"))
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "ToLowerCase requires input parameter".to_string(),
                })?;

            let output_path = params
                .get("output")
                .or_else(|| params.get("1"))
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "ToLowerCase requires output parameter".to_string(),
                })?;

            let input_path_str = match input_path {
                Value::String(s) => s.clone(),
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: "Input path must be string".to_string(),
                    })
                }
            };

            // Resolve input similar to ToUpperCase: prefer nested fact lookup, else literal
            let input_value = match facts.get_nested(&input_path_str) {
                Some(v) => v,
                None => Value::String(input_path_str.clone()),
            };

            let input_str = match input_value {
                Value::String(s) => s.clone(),
                Value::Integer(i) => i.to_string(),
                Value::Number(f) => f.to_string(),
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: "Input must be convertible to string".to_string(),
                    })
                }
            };

            let output_path_str = match output_path {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };

            let result = input_str.to_lowercase();
            match facts.set_nested(&output_path_str, Value::String(result.clone())) {
                Ok(()) => Ok(()),
                Err(e) => match e {
                    RuleEngineError::FieldNotFound { .. } => {
                        facts.add_value(&output_path_str, Value::String(result))?;
                        Ok(())
                    }
                    other => Err(other),
                },
            }
        });

        // StringLength action
        engine.register_action_handler("StringLength", |params, facts| {
            let input_path = params
                .get("input")
                .or_else(|| params.get("0"))
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "StringLength requires input parameter".to_string(),
                })?;

            let output_path = params
                .get("output")
                .or_else(|| params.get("1"))
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "StringLength requires output parameter".to_string(),
                })?;

            let input_path_str = match input_path {
                Value::String(s) => s.clone(),
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: "Input path must be string".to_string(),
                    })
                }
            };

            // Resolve input similar to other actions: prefer nested fact lookup, else literal
            let input_value = match facts.get_nested(&input_path_str) {
                Some(v) => v,
                None => Value::String(input_path_str.clone()),
            };

            let input_str = match input_value {
                Value::String(s) => s.clone(),
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: "Input must be string".to_string(),
                    })
                }
            };

            let output_path_str = match output_path {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };

            let length = input_str.len() as i64;
            match facts.set_nested(&output_path_str, Value::Integer(length)) {
                Ok(()) => Ok(()),
                Err(e) => match e {
                    RuleEngineError::FieldNotFound { .. } => {
                        facts.add_value(&output_path_str, Value::Integer(length))?;
                        Ok(())
                    }
                    other => Err(other),
                },
            }
        });

        // StringContains action
        engine.register_action_handler("StringContains", |params, facts| {
            let input_path = params
                .get("input")
                .or_else(|| params.get("0"))
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "StringContains requires input parameter".to_string(),
                })?;

            let search_value = params
                .get("search")
                .or_else(|| params.get("1"))
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "StringContains requires search parameter".to_string(),
                })?;

            let output_path = params
                .get("output")
                .or_else(|| params.get("2"))
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "StringContains requires output parameter".to_string(),
                })?;

            let input_path_str = match input_path {
                Value::String(s) => s.clone(),
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: "Input path must be string".to_string(),
                    })
                }
            };

            // Resolve input similar to other actions: prefer nested fact lookup, else literal
            let input_value = match facts.get_nested(&input_path_str) {
                Some(v) => v,
                None => Value::String(input_path_str.clone()),
            };

            let input_str = match input_value {
                Value::String(s) => s.clone(),
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: "Input must be string".to_string(),
                    })
                }
            };

            let search_str = match search_value {
                Value::String(s) => s.clone(),
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: "Search term must be string".to_string(),
                    })
                }
            };

            let output_path_str = match output_path {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            };

            let contains = input_str.contains(&search_str);
            match facts.set_nested(&output_path_str, Value::Boolean(contains)) {
                Ok(()) => Ok(()),
                Err(e) => match e {
                    RuleEngineError::FieldNotFound { .. } => {
                        facts.add_value(&output_path_str, Value::Boolean(contains))?;
                        Ok(())
                    }
                    other => Err(other),
                },
            }
        });

        Ok(())
    }

    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // Concat function
        engine.register_function("concat", |args, _facts| {
            if args.len() < 2 {
                return Err(RuleEngineError::EvaluationError {
                    message: "concat requires at least 2 arguments".to_string(),
                });
            }

            let mut result = String::new();
            for arg in args {
                match arg {
                    Value::String(s) => result.push_str(s),
                    Value::Integer(i) => result.push_str(&i.to_string()),
                    Value::Number(f) => result.push_str(&f.to_string()),
                    Value::Boolean(b) => result.push_str(&b.to_string()),
                    _ => {
                        return Err(RuleEngineError::EvaluationError {
                            message: "concat arguments must be convertible to string".to_string(),
                        })
                    }
                }
            }

            Ok(Value::String(result))
        });

        // Repeat function
        engine.register_function("repeat", |args, _facts| {
            if args.len() != 2 {
                return Err(RuleEngineError::EvaluationError {
                    message: "repeat requires exactly 2 arguments".to_string(),
                });
            }

            let text = match &args[0] {
                Value::String(s) => s.clone(),
                _ => {
                    return Err(RuleEngineError::EvaluationError {
                        message: "First argument must be string".to_string(),
                    })
                }
            };

            let count = match &args[1] {
                Value::Integer(i) => *i as usize,
                _ => {
                    return Err(RuleEngineError::EvaluationError {
                        message: "Second argument must be integer".to_string(),
                    })
                }
            };

            if count > 1000 {
                return Err(RuleEngineError::EvaluationError {
                    message: "Repeat count too large (max 1000)".to_string(),
                });
            }

            let result = text.repeat(count);
            Ok(Value::String(result))
        });

        Ok(())
    }

    fn unload(&mut self) -> Result<()> {
        self.metadata.state = PluginState::Unloaded;
        Ok(())
    }

    fn health_check(&mut self) -> PluginHealth {
        // Simple health check - just verify state
        match self.metadata.state {
            PluginState::Loaded => PluginHealth::Healthy,
            PluginState::Loading => PluginHealth::Warning("Plugin is still loading".to_string()),
            PluginState::Error => PluginHealth::Error("Plugin is in error state".to_string()),
            PluginState::Unloaded => PluginHealth::Warning("Plugin is unloaded".to_string()),
        }
    }
}

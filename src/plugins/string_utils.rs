use crate::engine::plugin::{PluginHealth, PluginMetadata, PluginState, RulePlugin};
use crate::engine::RustRuleEngine;
use crate::errors::{Result, RuleEngineError};
use crate::types::Value;

/// Built-in plugin for string manipulation operations
pub struct StringUtilsPlugin {
    metadata: PluginMetadata,
}

impl StringUtilsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "string-utils".to_string(),
                version: "1.0.0".to_string(),
                description: "String manipulation utilities".to_string(),
                author: "Rust Rule Engine Team".to_string(),
                state: PluginState::Loaded,
                health: PluginHealth::Healthy,
                actions: vec![
                    "ToUpperCase".to_string(),
                    "ToLowerCase".to_string(),
                    "StringLength".to_string(),
                    "StringContains".to_string(),
                    "StringTrim".to_string(),
                    "StringReplace".to_string(),
                    "StringSplit".to_string(),
                    "StringJoin".to_string(),
                ],
                functions: vec![
                    "concat".to_string(),
                    "repeat".to_string(),
                    "substring".to_string(),
                    "padLeft".to_string(),
                    "padRight".to_string(),
                ],
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
        // ToUpperCase - Convert string to uppercase
        engine.register_action_handler("ToUpperCase", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let text = value_to_string(&value)?;
                facts.set_nested(&output, Value::String(text.to_uppercase()))?;
            }
            Ok(())
        });

        // ToLowerCase - Convert string to lowercase
        engine.register_action_handler("ToLowerCase", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let text = value_to_string(&value)?;
                facts.set_nested(&output, Value::String(text.to_lowercase()))?;
            }
            Ok(())
        });

        // StringLength - Get string length
        engine.register_action_handler("StringLength", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let text = value_to_string(&value)?;
                facts.set_nested(&output, Value::Integer(text.len() as i64))?;
            }
            Ok(())
        });

        // StringContains - Check if string contains substring
        engine.register_action_handler("StringContains", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let search = get_string_param(params, "search", "1")?;
            let output = get_string_param(params, "output", "2")?;

            if let Some(value) = facts.get(&input) {
                let text = value_to_string(&value)?;
                let search_text = if search.contains('.') {
                    if let Some(search_value) = facts.get(&search) {
                        value_to_string(&search_value)?
                    } else {
                        search
                    }
                } else {
                    search
                };

                let contains = text.contains(&search_text);
                facts.set_nested(&output, Value::Boolean(contains))?;
            }
            Ok(())
        });

        // StringTrim - Trim whitespace
        engine.register_action_handler("StringTrim", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let text = value_to_string(&value)?;
                facts.set_nested(&output, Value::String(text.trim().to_string()))?;
            }
            Ok(())
        });

        // StringReplace - Replace substring
        engine.register_action_handler("StringReplace", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let from = get_string_param(params, "from", "1")?;
            let to = get_string_param(params, "to", "2")?;
            let output = get_string_param(params, "output", "3")?;

            if let Some(value) = facts.get(&input) {
                let text = value_to_string(&value)?;
                let result = text.replace(&from, &to);
                facts.set_nested(&output, Value::String(result))?;
            }
            Ok(())
        });

        Ok(())
    }

    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // concat - Concatenate strings
        engine.register_function("concat", |args, _facts| {
            if args.len() < 2 {
                return Err(RuleEngineError::EvaluationError {
                    message: "concat requires at least 2 arguments".to_string(),
                });
            }

            let mut result = String::new();
            for arg in args {
                result.push_str(&value_to_string(arg)?);
            }
            Ok(Value::String(result))
        });

        // repeat - Repeat string n times
        engine.register_function("repeat", |args, _facts| {
            if args.len() != 2 {
                return Err(RuleEngineError::EvaluationError {
                    message: "repeat requires exactly 2 arguments: text, count".to_string(),
                });
            }

            let text = value_to_string(&args[0])?;
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

            Ok(Value::String(text.repeat(count)))
        });

        // substring - Get substring
        engine.register_function("substring", |args, _facts| {
            if args.len() < 2 || args.len() > 3 {
                return Err(RuleEngineError::EvaluationError {
                    message: "substring requires 2-3 arguments: text, start, [length]".to_string(),
                });
            }

            let text = value_to_string(&args[0])?;
            let start = match &args[1] {
                Value::Integer(i) => *i as usize,
                _ => {
                    return Err(RuleEngineError::EvaluationError {
                        message: "Start position must be integer".to_string(),
                    })
                }
            };

            if start >= text.len() {
                return Ok(Value::String(String::new()));
            }

            let result = if args.len() == 3 {
                let length = match &args[2] {
                    Value::Integer(i) => *i as usize,
                    _ => {
                        return Err(RuleEngineError::EvaluationError {
                            message: "Length must be integer".to_string(),
                        })
                    }
                };
                let end = std::cmp::min(start + length, text.len());
                text[start..end].to_string()
            } else {
                text[start..].to_string()
            };

            Ok(Value::String(result))
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

fn value_to_string(value: &Value) -> Result<String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Number(f) => Ok(f.to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        _ => Err(RuleEngineError::ActionError {
            message: "Value cannot be converted to string".to_string(),
        }),
    }
}

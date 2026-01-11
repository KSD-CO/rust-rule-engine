use crate::engine::plugin::{PluginHealth, PluginMetadata, PluginState, RulePlugin};
use crate::engine::RustRuleEngine;
use crate::errors::{Result, RuleEngineError};
use crate::types::Value;
use regex::Regex;
use std::sync::OnceLock;

// Cache email regex for performance
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();

fn email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("Invalid email regex pattern")
    })
}

/// Built-in plugin for data validation operations
pub struct ValidationPlugin {
    metadata: PluginMetadata,
}

impl Default for ValidationPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "validation".to_string(),
                version: "1.0.0".to_string(),
                description: "Data validation utilities".to_string(),
                author: "Rust Rule Engine Team".to_string(),
                state: PluginState::Loaded,
                health: PluginHealth::Healthy,
                actions: vec![
                    "ValidateEmail".to_string(),
                    "ValidatePhone".to_string(),
                    "ValidateUrl".to_string(),
                    "ValidateRegex".to_string(),
                    "ValidateRange".to_string(),
                    "ValidateLength".to_string(),
                    "ValidateNotEmpty".to_string(),
                    "ValidateNumeric".to_string(),
                ],
                functions: vec![
                    "isEmail".to_string(),
                    "isPhone".to_string(),
                    "isUrl".to_string(),
                    "isNumeric".to_string(),
                    "isEmpty".to_string(),
                    "inRange".to_string(),
                ],
                dependencies: vec![],
            },
        }
    }
}

impl RulePlugin for ValidationPlugin {
    fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // ValidateEmail - Validate email format
        engine.register_action_handler("ValidateEmail", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let email = value_to_string(&value)?;
                let is_valid = is_valid_email(&email);
                facts.set_nested(&output, Value::Boolean(is_valid))?;
            }
            Ok(())
        });

        // ValidatePhone - Validate phone number format
        engine.register_action_handler("ValidatePhone", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let phone = value_to_string(&value)?;
                let is_valid = is_valid_phone(&phone);
                facts.set_nested(&output, Value::Boolean(is_valid))?;
            }
            Ok(())
        });

        // ValidateUrl - Validate URL format
        engine.register_action_handler("ValidateUrl", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let url = value_to_string(&value)?;
                let is_valid = is_valid_url(&url);
                facts.set_nested(&output, Value::Boolean(is_valid))?;
            }
            Ok(())
        });

        // ValidateRegex - Validate against regex pattern
        engine.register_action_handler("ValidateRegex", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let pattern = get_string_param(params, "pattern", "1")?;
            let output = get_string_param(params, "output", "2")?;

            if let Some(value) = facts.get(&input) {
                let text = value_to_string(&value)?;
                let regex = Regex::new(&pattern).map_err(|e| RuleEngineError::ActionError {
                    message: format!("Invalid regex pattern: {}", e),
                })?;
                let is_valid = regex.is_match(&text);
                facts.set_nested(&output, Value::Boolean(is_valid))?;
            }
            Ok(())
        });

        // ValidateRange - Validate number is in range
        engine.register_action_handler("ValidateRange", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let min = get_number_param(params, facts, "min", "1")?;
            let max = get_number_param(params, facts, "max", "2")?;
            let output = get_string_param(params, "output", "3")?;

            if let Some(value) = facts.get(&input) {
                let num = value_to_number(&value)?;
                let is_valid = num >= min && num <= max;
                facts.set_nested(&output, Value::Boolean(is_valid))?;
            }
            Ok(())
        });

        // ValidateLength - Validate string length
        engine.register_action_handler("ValidateLength", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let min_len = get_number_param(params, facts, "minLength", "1")? as usize;
            let max_len = get_number_param(params, facts, "maxLength", "2")? as usize;
            let output = get_string_param(params, "output", "3")?;

            if let Some(value) = facts.get(&input) {
                let text = value_to_string(&value)?;
                let len = text.len();
                let is_valid = len >= min_len && len <= max_len;
                facts.set_nested(&output, Value::Boolean(is_valid))?;
            }
            Ok(())
        });

        // ValidateNotEmpty - Check if value is not empty
        engine.register_action_handler("ValidateNotEmpty", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let is_not_empty = match value {
                    Value::String(s) => !s.trim().is_empty(),
                    Value::Array(arr) => !arr.is_empty(),
                    Value::Object(obj) => !obj.is_empty(),
                    Value::Null => false,
                    _ => true,
                };
                facts.set_nested(&output, Value::Boolean(is_not_empty))?;
            }
            Ok(())
        });

        Ok(())
    }

    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // isEmail - Check if string is valid email
        engine.register_function("isEmail", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "isEmail requires exactly 1 argument".to_string(),
                });
            }

            let email = value_to_string(&args[0])?;
            Ok(Value::Boolean(is_valid_email(&email)))
        });

        // isPhone - Check if string is valid phone
        engine.register_function("isPhone", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "isPhone requires exactly 1 argument".to_string(),
                });
            }

            let phone = value_to_string(&args[0])?;
            Ok(Value::Boolean(is_valid_phone(&phone)))
        });

        // isUrl - Check if string is valid URL
        engine.register_function("isUrl", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "isUrl requires exactly 1 argument".to_string(),
                });
            }

            let url = value_to_string(&args[0])?;
            Ok(Value::Boolean(is_valid_url(&url)))
        });

        // isNumeric - Check if string is numeric
        engine.register_function("isNumeric", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "isNumeric requires exactly 1 argument".to_string(),
                });
            }

            let text = value_to_string(&args[0])?;
            let is_numeric = text.parse::<f64>().is_ok();
            Ok(Value::Boolean(is_numeric))
        });

        // isEmpty - Check if value is empty
        engine.register_function("isEmpty", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "isEmpty requires exactly 1 argument".to_string(),
                });
            }

            let is_empty = match &args[0] {
                Value::String(s) => s.trim().is_empty(),
                Value::Array(arr) => arr.is_empty(),
                Value::Object(obj) => obj.is_empty(),
                Value::Null => true,
                _ => false,
            };
            Ok(Value::Boolean(is_empty))
        });

        // inRange - Check if number is in range
        engine.register_function("inRange", |args, _facts| {
            if args.len() != 3 {
                return Err(RuleEngineError::EvaluationError {
                    message: "inRange requires exactly 3 arguments: value, min, max".to_string(),
                });
            }

            let value = value_to_number(&args[0])?;
            let min = value_to_number(&args[1])?;
            let max = value_to_number(&args[2])?;

            let in_range = value >= min && value <= max;
            Ok(Value::Boolean(in_range))
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

    if let Value::String(s) = value {
        if s.contains('.') {
            if let Some(fact_value) = facts.get(s) {
                return value_to_number(&fact_value);
            }
        }
    }

    value_to_number(value)
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

fn is_valid_email(email: &str) -> bool {
    email_regex().is_match(email)
}

fn is_valid_phone(phone: &str) -> bool {
    // Remove all non-digit characters
    let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    // Check if it has 10-15 digits (international phone number range)
    digits.len() >= 10 && digits.len() <= 15
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("ftp://")
}

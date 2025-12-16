#![allow(clippy::collapsible_match)]

use crate::engine::plugin::{PluginHealth, PluginMetadata, PluginState, RulePlugin};
use crate::engine::RustRuleEngine;
use crate::errors::{Result, RuleEngineError};
use crate::types::Value;
use std::collections::HashMap;

/// Built-in plugin for collection operations
pub struct CollectionUtilsPlugin {
    metadata: PluginMetadata,
}

impl Default for CollectionUtilsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectionUtilsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "collection_utils".to_string(),
                version: "1.0.0".to_string(),
                description: "Collection manipulation utilities".to_string(),
                author: "Rust Rule Engine Team".to_string(),
                state: PluginState::Loaded,
                health: PluginHealth::Healthy,
                actions: vec![
                    "ArrayLength".to_string(),
                    "ArrayPush".to_string(),
                    "ArrayPop".to_string(),
                    "ArraySort".to_string(),
                    "ArrayFilter".to_string(),
                    "ArrayMap".to_string(),
                    "ArrayFind".to_string(),
                    "ObjectKeys".to_string(),
                    "ObjectValues".to_string(),
                    "ObjectMerge".to_string(),
                ],
                functions: vec![
                    "length".to_string(),
                    "contains".to_string(),
                    "first".to_string(),
                    "last".to_string(),
                    "reverse".to_string(),
                    "join".to_string(),
                    "slice".to_string(),
                    "keys".to_string(),
                    "values".to_string(),
                ],
                dependencies: vec![],
            },
        }
    }
}

impl RulePlugin for CollectionUtilsPlugin {
    fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // ArrayLength - Get array length
        engine.register_action_handler("ArrayLength", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let length = match value {
                    Value::Array(arr) => arr.len(),
                    Value::String(s) => s.len(),
                    Value::Object(obj) => obj.len(),
                    _ => 0,
                };
                facts.set_nested(&output, Value::Integer(length as i64))?;
            }
            Ok(())
        });

        // ArrayPush - Add element to array
        engine.register_action_handler("ArrayPush", |params, facts| {
            let array_path = get_string_param(params, "array", "0")?;
            let element = get_value_param(params, facts, "element", "1")?;

            if let Some(value) = facts.get(&array_path) {
                if let Value::Array(mut arr) = value.clone() {
                    arr.push(element);
                    facts.set_nested(&array_path, Value::Array(arr))?;
                } else {
                    return Err(RuleEngineError::ActionError {
                        message: "Target must be an array".to_string(),
                    });
                }
            } else {
                // Create new array with the element
                facts.set_nested(&array_path, Value::Array(vec![element]))?;
            }
            Ok(())
        });

        // ArrayPop - Remove and return last element
        engine.register_action_handler("ArrayPop", |params, facts| {
            let array_path = get_string_param(params, "array", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&array_path) {
                if let Value::Array(mut arr) = value.clone() {
                    if let Some(popped) = arr.pop() {
                        facts.set_nested(&array_path, Value::Array(arr))?;
                        facts.set_nested(&output, popped)?;
                    } else {
                        facts.set_nested(&output, Value::Null)?;
                    }
                }
            }
            Ok(())
        });

        // ArraySort - Sort array
        engine.register_action_handler("ArraySort", |params, facts| {
            let array_path = get_string_param(params, "array", "0")?;
            let ascending = get_optional_bool_param(params, "ascending").unwrap_or(true);

            if let Some(value) = facts.get(&array_path) {
                if let Value::Array(mut arr) = value.clone() {
                    arr.sort_by(|a, b| {
                        let order = compare_values(a, b);
                        if ascending {
                            order
                        } else {
                            order.reverse()
                        }
                    });
                    facts.set_nested(&array_path, Value::Array(arr))?;
                }
            }
            Ok(())
        });

        // ArrayFilter - Filter array elements
        engine.register_action_handler("ArrayFilter", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let predicate_field = get_string_param(params, "field", "1")?;
            let predicate_value = get_value_param(params, facts, "value", "2")?;
            let output = get_string_param(params, "output", "3")?;

            if let Some(value) = facts.get(&input) {
                if let Value::Array(arr) = value {
                    let filtered: Vec<Value> = arr
                        .iter()
                        .filter(|item| filter_predicate(item, &predicate_field, &predicate_value))
                        .cloned()
                        .collect();
                    facts.set_nested(&output, Value::Array(filtered))?;
                }
            }
            Ok(())
        });

        // ArrayFind - Find first matching element
        engine.register_action_handler("ArrayFind", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let predicate_field = get_string_param(params, "field", "1")?;
            let predicate_value = get_value_param(params, facts, "value", "2")?;
            let output = get_string_param(params, "output", "3")?;

            if let Some(value) = facts.get(&input) {
                if let Value::Array(arr) = value {
                    let found = arr
                        .iter()
                        .find(|item| filter_predicate(item, &predicate_field, &predicate_value))
                        .cloned()
                        .unwrap_or(Value::Null);
                    facts.set_nested(&output, found)?;
                }
            }
            Ok(())
        });

        // ObjectKeys - Get object keys
        engine.register_action_handler("ObjectKeys", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                if let Value::Object(obj) = value {
                    let keys: Vec<Value> = obj.keys().map(|k| Value::String(k.clone())).collect();
                    facts.set_nested(&output, Value::Array(keys))?;
                }
            }
            Ok(())
        });

        // ObjectValues - Get object values
        engine.register_action_handler("ObjectValues", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                if let Value::Object(obj) = value {
                    let values: Vec<Value> = obj.values().cloned().collect();
                    facts.set_nested(&output, Value::Array(values))?;
                }
            }
            Ok(())
        });

        // ObjectMerge - Merge two objects
        engine.register_action_handler("ObjectMerge", |params, facts| {
            let source1 = get_string_param(params, "source1", "0")?;
            let source2 = get_string_param(params, "source2", "1")?;
            let output = get_string_param(params, "output", "2")?;

            let obj1 = facts
                .get(&source1)
                .and_then(|v| {
                    if let Value::Object(obj) = v {
                        Some(obj.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let obj2 = facts
                .get(&source2)
                .and_then(|v| {
                    if let Value::Object(obj) = v {
                        Some(obj.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let mut merged = obj1;
            for (key, value) in obj2 {
                merged.insert(key, value);
            }

            facts.set_nested(&output, Value::Object(merged))?;
            Ok(())
        });

        Ok(())
    }

    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // length - Get collection length
        engine.register_function("length", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "length requires exactly 1 argument".to_string(),
                });
            }

            let length = match &args[0] {
                Value::Array(arr) => arr.len(),
                Value::String(s) => s.len(),
                Value::Object(obj) => obj.len(),
                _ => 0,
            };
            Ok(Value::Integer(length as i64))
        });

        // contains - Check if collection contains value
        engine.register_function("contains", |args, _facts| {
            if args.len() != 2 {
                return Err(RuleEngineError::EvaluationError {
                    message: "contains requires exactly 2 arguments: collection, value".to_string(),
                });
            }

            let contains = match (&args[0], &args[1]) {
                (Value::Array(arr), value) => arr.contains(value),
                (Value::String(s), Value::String(search)) => s.contains(search),
                (Value::Object(obj), Value::String(key)) => obj.contains_key(key),
                _ => false,
            };
            Ok(Value::Boolean(contains))
        });

        // first - Get first element
        engine.register_function("first", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "first requires exactly 1 argument".to_string(),
                });
            }

            let first = match &args[0] {
                Value::Array(arr) => arr.first().cloned().unwrap_or(Value::Null),
                Value::String(s) => {
                    if s.is_empty() {
                        Value::Null
                    } else {
                        Value::String(s.chars().next().unwrap().to_string())
                    }
                }
                _ => Value::Null,
            };
            Ok(first)
        });

        // last - Get last element
        engine.register_function("last", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "last requires exactly 1 argument".to_string(),
                });
            }

            let last = match &args[0] {
                Value::Array(arr) => arr.last().cloned().unwrap_or(Value::Null),
                Value::String(s) => {
                    if s.is_empty() {
                        Value::Null
                    } else {
                        Value::String(s.chars().last().unwrap().to_string())
                    }
                }
                _ => Value::Null,
            };
            Ok(last)
        });

        // reverse - Reverse array or string
        engine.register_function("reverse", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "reverse requires exactly 1 argument".to_string(),
                });
            }

            let reversed = match &args[0] {
                Value::Array(arr) => {
                    let mut rev = arr.clone();
                    rev.reverse();
                    Value::Array(rev)
                }
                Value::String(s) => Value::String(s.chars().rev().collect()),
                _ => args[0].clone(),
            };
            Ok(reversed)
        });

        // join - Join array elements
        engine.register_function("join", |args, _facts| {
            if args.len() != 2 {
                return Err(RuleEngineError::EvaluationError {
                    message: "join requires exactly 2 arguments: array, separator".to_string(),
                });
            }

            match (&args[0], &args[1]) {
                (Value::Array(arr), Value::String(sep)) => {
                    let strings: Vec<String> = arr
                        .iter()
                        .map(|v| value_to_string(v).unwrap_or_default())
                        .collect();
                    Ok(Value::String(strings.join(sep)))
                }
                _ => Err(RuleEngineError::EvaluationError {
                    message: "join requires array and string separator".to_string(),
                }),
            }
        });

        // slice - Get slice of array
        engine.register_function("slice", |args, _facts| {
            if args.len() < 2 || args.len() > 3 {
                return Err(RuleEngineError::EvaluationError {
                    message: "slice requires 2-3 arguments: array, start, [end]".to_string(),
                });
            }

            match &args[0] {
                Value::Array(arr) => {
                    let start = value_to_number(&args[1])? as usize;
                    let end = if args.len() == 3 {
                        value_to_number(&args[2])? as usize
                    } else {
                        arr.len()
                    };

                    let start = start.min(arr.len());
                    let end = end.min(arr.len());

                    if start <= end {
                        Ok(Value::Array(arr[start..end].to_vec()))
                    } else {
                        Ok(Value::Array(vec![]))
                    }
                }
                _ => Err(RuleEngineError::EvaluationError {
                    message: "slice requires array as first argument".to_string(),
                }),
            }
        });

        // keys - Get object keys
        engine.register_function("keys", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "keys requires exactly 1 argument".to_string(),
                });
            }

            match &args[0] {
                Value::Object(obj) => {
                    let keys: Vec<Value> = obj.keys().map(|k| Value::String(k.clone())).collect();
                    Ok(Value::Array(keys))
                }
                _ => Ok(Value::Array(vec![])),
            }
        });

        // values - Get object values
        engine.register_function("values", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "values requires exactly 1 argument".to_string(),
                });
            }

            match &args[0] {
                Value::Object(obj) => Ok(Value::Array(obj.values().cloned().collect())),
                _ => Ok(Value::Array(vec![])),
            }
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
fn get_string_param(params: &HashMap<String, Value>, name: &str, pos: &str) -> Result<String> {
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

fn get_value_param(
    params: &HashMap<String, Value>,
    facts: &crate::Facts,
    name: &str,
    pos: &str,
) -> Result<Value> {
    let value = params
        .get(name)
        .or_else(|| params.get(pos))
        .ok_or_else(|| RuleEngineError::ActionError {
            message: format!("Missing parameter: {}", name),
        })?;

    if let Value::String(s) = value {
        if s.contains('.') {
            if let Some(fact_value) = facts.get(s) {
                return Ok(fact_value.clone());
            }
        }
    }

    Ok(value.clone())
}

fn get_optional_bool_param(params: &HashMap<String, Value>, name: &str) -> Option<bool> {
    params.get(name).and_then(|v| match v {
        Value::Boolean(b) => Some(*b),
        Value::String(s) => s.parse().ok(),
        _ => None,
    })
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

fn compare_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    use std::cmp::Ordering;

    match (a, b) {
        (Value::Integer(a), Value::Integer(b)) => a.cmp(b),
        (Value::Number(a), Value::Number(b)) => a.partial_cmp(b).unwrap_or(Ordering::Equal),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        (Value::Boolean(a), Value::Boolean(b)) => a.cmp(b),
        _ => Ordering::Equal,
    }
}

fn filter_predicate(item: &Value, field: &str, expected: &Value) -> bool {
    if field == "_value" {
        return item == expected;
    }

    if let Value::Object(obj) = item {
        if let Some(field_value) = obj.get(field) {
            return field_value == expected;
        }
    }

    false
}

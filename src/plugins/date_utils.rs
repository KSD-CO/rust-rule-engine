use crate::engine::plugin::{PluginHealth, PluginMetadata, PluginState, RulePlugin};
use crate::engine::RustRuleEngine;
use crate::errors::{Result, RuleEngineError};
use crate::types::Value;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime, TimeZone, Utc};

/// Built-in plugin for date and time operations
pub struct DateUtilsPlugin {
    metadata: PluginMetadata,
}

impl DateUtilsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "date-utils".to_string(),
                version: "1.0.0".to_string(),
                description: "Date and time manipulation utilities".to_string(),
                author: "Rust Rule Engine Team".to_string(),
                state: PluginState::Loaded,
                health: PluginHealth::Healthy,
                actions: vec![
                    "CurrentDate".to_string(),
                    "CurrentTime".to_string(),
                    "FormatDate".to_string(),
                    "ParseDate".to_string(),
                    "AddDays".to_string(),
                    "AddHours".to_string(),
                    "DateDiff".to_string(),
                    "IsWeekend".to_string(),
                ],
                functions: vec![
                    "now".to_string(),
                    "today".to_string(),
                    "dayOfWeek".to_string(),
                    "dayOfYear".to_string(),
                    "year".to_string(),
                    "month".to_string(),
                    "day".to_string(),
                ],
                dependencies: vec![],
            },
        }
    }
}

impl RulePlugin for DateUtilsPlugin {
    fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // CurrentDate - Get current date
        engine.register_action_handler("CurrentDate", |params, facts| {
            let output = get_string_param(params, "output", "0")?;
            let now = Local::now();
            let date_str = now.format("%Y-%m-%d").to_string();
            facts.set_nested(&output, Value::String(date_str))?;
            Ok(())
        });

        // CurrentTime - Get current time
        engine.register_action_handler("CurrentTime", |params, facts| {
            let output = get_string_param(params, "output", "0")?;
            let now = Local::now();
            let time_str = now.format("%H:%M:%S").to_string();
            facts.set_nested(&output, Value::String(time_str))?;
            Ok(())
        });

        // FormatDate - Format date string
        engine.register_action_handler("FormatDate", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let format = get_string_param(params, "format", "1")?;
            let output = get_string_param(params, "output", "2")?;

            if let Some(value) = facts.get(&input) {
                let date_str = value_to_string(&value)?;

                // Try to parse the date
                let dt = parse_date_string(&date_str)?;
                let formatted = dt.format(&format).to_string();
                facts.set_nested(&output, Value::String(formatted))?;
            }
            Ok(())
        });

        // AddDays - Add days to date
        engine.register_action_handler("AddDays", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let days = get_number_param(params, facts, "days", "1")?;
            let output = get_string_param(params, "output", "2")?;

            if let Some(value) = facts.get(&input) {
                let date_str = value_to_string(&value)?;
                let dt = parse_date_string(&date_str)?;
                let new_dt = dt + Duration::days(days as i64);
                let result = new_dt.format("%Y-%m-%d").to_string();
                facts.set_nested(&output, Value::String(result))?;
            }
            Ok(())
        });

        // IsWeekend - Check if date is weekend
        engine.register_action_handler("IsWeekend", |params, facts| {
            let input = get_string_param(params, "input", "0")?;
            let output = get_string_param(params, "output", "1")?;

            if let Some(value) = facts.get(&input) {
                let date_str = value_to_string(&value)?;
                let dt = parse_date_string(&date_str)?;
                let weekday = dt.weekday();
                let is_weekend = weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun;
                facts.set_nested(&output, Value::Boolean(is_weekend))?;
            }
            Ok(())
        });

        Ok(())
    }

    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // now - Get current timestamp
        engine.register_function("now", |_args, _facts| {
            let now = Utc::now();
            Ok(Value::String(now.to_rfc3339()))
        });

        // today - Get today's date
        engine.register_function("today", |_args, _facts| {
            let today = Local::now();
            Ok(Value::String(today.format("%Y-%m-%d").to_string()))
        });

        // dayOfWeek - Get day of week (1=Monday, 7=Sunday)
        engine.register_function("dayOfWeek", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "dayOfWeek requires exactly 1 argument".to_string(),
                });
            }

            let date_str = value_to_string(&args[0])?;
            let dt = parse_date_string(&date_str)?;
            let day_num = dt.weekday().number_from_monday();
            Ok(Value::Integer(day_num as i64))
        });

        // year - Extract year from date
        engine.register_function("year", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "year requires exactly 1 argument".to_string(),
                });
            }

            let date_str = value_to_string(&args[0])?;
            let dt = parse_date_string(&date_str)?;
            Ok(Value::Integer(dt.year() as i64))
        });

        // month - Extract month from date
        engine.register_function("month", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "month requires exactly 1 argument".to_string(),
                });
            }

            let date_str = value_to_string(&args[0])?;
            let dt = parse_date_string(&date_str)?;
            Ok(Value::Integer(dt.month() as i64))
        });

        // day - Extract day from date
        engine.register_function("day", |args, _facts| {
            if args.len() != 1 {
                return Err(RuleEngineError::EvaluationError {
                    message: "day requires exactly 1 argument".to_string(),
                });
            }

            let date_str = value_to_string(&args[0])?;
            let dt = parse_date_string(&date_str)?;
            Ok(Value::Integer(dt.day() as i64))
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

    value_to_number(&value)
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

fn parse_date_string(date_str: &str) -> Result<DateTime<Local>> {
    // Try various date formats
    let formats = vec![
        "%Y-%m-%d",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d",
        "%d/%m/%Y",
        "%m/%d/%Y",
    ];

    for format in formats {
        if let Ok(naive_dt) = NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(Local
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "Invalid datetime".to_string(),
                })?);
        }

        if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, format) {
            let naive_dt =
                naive_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| RuleEngineError::ActionError {
                        message: "Invalid date".to_string(),
                    })?;
            return Ok(Local
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| RuleEngineError::ActionError {
                    message: "Invalid datetime".to_string(),
                })?);
        }
    }

    Err(RuleEngineError::ActionError {
        message: format!("Cannot parse date: {}", date_str),
    })
}

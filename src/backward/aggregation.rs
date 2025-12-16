//! Aggregation support for backward chaining queries
//!
//! Provides aggregate functions like COUNT, SUM, AVG, MIN, MAX
//! for use in backward chaining queries.
//!
//! ## Example
//!
//! ```ignore
//! use rust_rule_engine::backward::*;
//!
//! let mut engine = BackwardEngine::new(kb);
//!
//! // Count all employees
//! let count = engine.query_aggregate(
//!     "count(?x) WHERE employee(?x)",
//!     &mut facts
//! )?;
//!
//! // Sum of all salaries
//! let total = engine.query_aggregate(
//!     "sum(?salary) WHERE salary(?name, ?salary)",
//!     &mut facts
//! )?;
//!
//! // Average salary
//! let avg = engine.query_aggregate(
//!     "avg(?salary) WHERE salary(?name, ?salary) AND ?salary > 50000",
//!     &mut facts
//! )?;
//! ```

use super::search::Solution;
use crate::errors::{Result, RuleEngineError};
use crate::types::Value;

/// Aggregate function types
#[derive(Debug, Clone, PartialEq)]
pub enum AggregateFunction {
    /// Count number of solutions
    Count,

    /// Sum of field values
    Sum(String),

    /// Average of field values
    Avg(String),

    /// Minimum field value
    Min(String),

    /// Maximum field value
    Max(String),

    /// First solution
    First,

    /// Last solution
    Last,
}

impl AggregateFunction {
    /// Get the field name for field-based aggregates
    pub fn field_name(&self) -> Option<&str> {
        match self {
            AggregateFunction::Sum(f)
            | AggregateFunction::Avg(f)
            | AggregateFunction::Min(f)
            | AggregateFunction::Max(f) => Some(f),
            _ => None,
        }
    }
}

/// Parsed aggregate query
#[derive(Debug, Clone)]
pub struct AggregateQuery {
    /// The aggregate function to apply
    pub function: AggregateFunction,

    /// The goal pattern to match
    pub pattern: String,

    /// Optional filter condition
    pub filter: Option<String>,
}

impl AggregateQuery {
    /// Create a new aggregate query
    pub fn new(function: AggregateFunction, pattern: String) -> Self {
        Self {
            function,
            pattern,
            filter: None,
        }
    }

    /// Add a filter condition
    pub fn with_filter(mut self, filter: String) -> Self {
        self.filter = Some(filter);
        self
    }
}

/// Parse an aggregate query string
///
/// Supported formats:
/// - `count(?x) WHERE pattern`
/// - `sum(?field) WHERE pattern`
/// - `avg(?field) WHERE pattern AND ?field > 100`
/// - `min(?field) WHERE pattern`
/// - `max(?field) WHERE pattern`
pub fn parse_aggregate_query(query: &str) -> Result<AggregateQuery> {
    let query = query.trim();

    // Split on WHERE keyword
    let parts: Vec<&str> = query.splitn(2, " WHERE ").collect();
    if parts.len() != 2 {
        return Err(RuleEngineError::ParseError {
            message: format!("Invalid aggregate query format. Expected: 'function(?var) WHERE pattern'. Got: '{}'", query),
        });
    }

    let func_part = parts[0].trim();
    let pattern_part = parts[1].trim();

    // Parse function and variable
    let (func_name, var_name) = parse_function_call(func_part)?;

    // Create aggregate function
    let function = match func_name.to_lowercase().as_str() {
        "count" => AggregateFunction::Count,
        "sum" => {
            if var_name.is_empty() {
                return Err(RuleEngineError::ParseError {
                    message: "sum() requires a variable, e.g., sum(?amount)".to_string(),
                });
            }
            AggregateFunction::Sum(var_name.to_string())
        }
        "avg" => {
            if var_name.is_empty() {
                return Err(RuleEngineError::ParseError {
                    message: "avg() requires a variable, e.g., avg(?salary)".to_string(),
                });
            }
            AggregateFunction::Avg(var_name.to_string())
        }
        "min" => {
            if var_name.is_empty() {
                return Err(RuleEngineError::ParseError {
                    message: "min() requires a variable, e.g., min(?price)".to_string(),
                });
            }
            AggregateFunction::Min(var_name.to_string())
        }
        "max" => {
            if var_name.is_empty() {
                return Err(RuleEngineError::ParseError {
                    message: "max() requires a variable, e.g., max(?score)".to_string(),
                });
            }
            AggregateFunction::Max(var_name.to_string())
        }
        "first" => AggregateFunction::First,
        "last" => AggregateFunction::Last,
        _ => {
            return Err(RuleEngineError::ParseError {
                message: format!("Unknown aggregate function: '{}'. Supported: count, sum, avg, min, max, first, last", func_name),
            });
        }
    };

    // Split pattern and filter (on AND)
    let (pattern, filter) = if pattern_part.contains(" AND ") {
        let parts: Vec<&str> = pattern_part.splitn(2, " AND ").collect();
        (
            parts[0].trim().to_string(),
            Some(parts[1].trim().to_string()),
        )
    } else {
        (pattern_part.to_string(), None)
    };

    Ok(AggregateQuery {
        function,
        pattern,
        filter,
    })
}

/// Parse a function call like "count(?x)" or "sum(?amount)"
fn parse_function_call(s: &str) -> Result<(String, String)> {
    let s = s.trim();

    // Find opening parenthesis
    let open_idx = s.find('(').ok_or_else(|| RuleEngineError::ParseError {
        message: format!("Expected '(' in function call: '{}'", s),
    })?;

    // Find closing parenthesis
    let close_idx = s.rfind(')').ok_or_else(|| RuleEngineError::ParseError {
        message: format!("Expected ')' in function call: '{}'", s),
    })?;

    if close_idx <= open_idx {
        return Err(RuleEngineError::ParseError {
            message: format!("Invalid function call syntax: '{}'", s),
        });
    }

    let func_name = s[..open_idx].trim().to_string();
    let var_name = s[open_idx + 1..close_idx].trim().to_string();

    // Remove leading ? from variable name if present
    let var_name = if let Some(stripped) = var_name.strip_prefix('?') {
        stripped.to_string()
    } else {
        var_name
    };

    Ok((func_name, var_name))
}

/// Apply aggregate function to solutions
pub fn apply_aggregate(function: &AggregateFunction, solutions: &[Solution]) -> Result<Value> {
    if solutions.is_empty() {
        // Return appropriate zero value
        return Ok(match function {
            AggregateFunction::Count => Value::Integer(0),
            AggregateFunction::Sum(_) => Value::Number(0.0),
            AggregateFunction::Avg(_) => Value::Number(0.0),
            AggregateFunction::Min(_) => Value::Null,
            AggregateFunction::Max(_) => Value::Null,
            AggregateFunction::First => Value::Null,
            AggregateFunction::Last => Value::Null,
        });
    }

    match function {
        AggregateFunction::Count => Ok(Value::Integer(solutions.len() as i64)),

        AggregateFunction::Sum(field) => {
            let sum: f64 = solutions
                .iter()
                .filter_map(|s| s.bindings.get(field))
                .filter_map(|v| value_to_float(v).ok())
                .sum();
            Ok(Value::Number(sum))
        }

        AggregateFunction::Avg(field) => {
            let values: Vec<f64> = solutions
                .iter()
                .filter_map(|s| s.bindings.get(field))
                .filter_map(|v| value_to_float(v).ok())
                .collect();

            if values.is_empty() {
                Ok(Value::Number(0.0))
            } else {
                let sum: f64 = values.iter().sum();
                Ok(Value::Number(sum / values.len() as f64))
            }
        }

        AggregateFunction::Min(field) => {
            let min = solutions
                .iter()
                .filter_map(|s| s.bindings.get(field))
                .filter_map(|v| value_to_float(v).ok())
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            Ok(min.map(Value::Number).unwrap_or(Value::Null))
        }

        AggregateFunction::Max(field) => {
            let max = solutions
                .iter()
                .filter_map(|s| s.bindings.get(field))
                .filter_map(|v| value_to_float(v).ok())
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            Ok(max.map(Value::Number).unwrap_or(Value::Null))
        }

        AggregateFunction::First => {
            Ok(solutions
                .first()
                .and_then(|s| {
                    // Return the first non-null binding
                    s.bindings.values().next().cloned()
                })
                .unwrap_or(Value::Null))
        }

        AggregateFunction::Last => {
            Ok(solutions
                .last()
                .and_then(|s| {
                    // Return the last non-null binding
                    s.bindings.values().last().cloned()
                })
                .unwrap_or(Value::Null))
        }
    }
}

/// Convert a Value to f64 for numeric aggregations
fn value_to_float(value: &Value) -> Result<f64> {
    match value {
        Value::Number(n) => Ok(*n),
        Value::Integer(i) => Ok(*i as f64),
        Value::String(s) => s
            .parse::<f64>()
            .map_err(|_| RuleEngineError::EvaluationError {
                message: format!("Cannot convert '{}' to number", s),
            }),
        _ => Err(RuleEngineError::EvaluationError {
            message: format!("Cannot aggregate non-numeric value: {:?}", value),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_parse_count_query() {
        let query = "count(?x) WHERE employee(?x)";
        let result = parse_aggregate_query(query).unwrap();

        assert_eq!(result.function, AggregateFunction::Count);
        assert_eq!(result.pattern, "employee(?x)");
        assert_eq!(result.filter, None);
    }

    #[test]
    fn test_parse_sum_query() {
        let query = "sum(?amount) WHERE purchase(?item, ?amount)";
        let result = parse_aggregate_query(query).unwrap();

        assert_eq!(
            result.function,
            AggregateFunction::Sum("amount".to_string())
        );
        assert_eq!(result.pattern, "purchase(?item, ?amount)");
    }

    #[test]
    fn test_parse_avg_with_filter() {
        let query = "avg(?salary) WHERE salary(?name, ?salary) AND ?salary > 50000";
        let result = parse_aggregate_query(query).unwrap();

        assert_eq!(
            result.function,
            AggregateFunction::Avg("salary".to_string())
        );
        assert_eq!(result.pattern, "salary(?name, ?salary)");
        assert_eq!(result.filter, Some("?salary > 50000".to_string()));
    }

    #[test]
    fn test_parse_min_query() {
        let query = "min(?price) WHERE product(?name, ?price)";
        let result = parse_aggregate_query(query).unwrap();

        assert_eq!(result.function, AggregateFunction::Min("price".to_string()));
    }

    #[test]
    fn test_parse_max_query() {
        let query = "max(?score) WHERE student(?name, ?score)";
        let result = parse_aggregate_query(query).unwrap();

        assert_eq!(result.function, AggregateFunction::Max("score".to_string()));
    }

    #[test]
    fn test_parse_invalid_query() {
        let query = "count(?x)"; // Missing WHERE
        let result = parse_aggregate_query(query);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unknown_function() {
        let query = "unknown(?x) WHERE test(?x)";
        let result = parse_aggregate_query(query);
        assert!(result.is_err());
    }

    #[test]
    fn test_apply_count() {
        let solutions = vec![
            Solution {
                path: vec![],
                bindings: HashMap::new(),
            },
            Solution {
                path: vec![],
                bindings: HashMap::new(),
            },
            Solution {
                path: vec![],
                bindings: HashMap::new(),
            },
        ];

        let result = apply_aggregate(&AggregateFunction::Count, &solutions).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_apply_sum() {
        let mut b1 = HashMap::new();
        b1.insert("amount".to_string(), Value::Number(100.0));

        let mut b2 = HashMap::new();
        b2.insert("amount".to_string(), Value::Number(200.0));

        let mut b3 = HashMap::new();
        b3.insert("amount".to_string(), Value::Number(300.0));

        let solutions = vec![
            Solution {
                path: vec![],
                bindings: b1,
            },
            Solution {
                path: vec![],
                bindings: b2,
            },
            Solution {
                path: vec![],
                bindings: b3,
            },
        ];

        let result =
            apply_aggregate(&AggregateFunction::Sum("amount".to_string()), &solutions).unwrap();
        assert_eq!(result, Value::Number(600.0));
    }

    #[test]
    fn test_apply_avg() {
        let mut b1 = HashMap::new();
        b1.insert("score".to_string(), Value::Integer(80));

        let mut b2 = HashMap::new();
        b2.insert("score".to_string(), Value::Integer(90));

        let mut b3 = HashMap::new();
        b3.insert("score".to_string(), Value::Integer(100));

        let solutions = vec![
            Solution {
                path: vec![],
                bindings: b1,
            },
            Solution {
                path: vec![],
                bindings: b2,
            },
            Solution {
                path: vec![],
                bindings: b3,
            },
        ];

        let result =
            apply_aggregate(&AggregateFunction::Avg("score".to_string()), &solutions).unwrap();
        assert_eq!(result, Value::Number(90.0));
    }

    #[test]
    fn test_apply_min() {
        let mut b1 = HashMap::new();
        b1.insert("price".to_string(), Value::Number(99.99));

        let mut b2 = HashMap::new();
        b2.insert("price".to_string(), Value::Number(49.99));

        let mut b3 = HashMap::new();
        b3.insert("price".to_string(), Value::Number(149.99));

        let solutions = vec![
            Solution {
                path: vec![],
                bindings: b1,
            },
            Solution {
                path: vec![],
                bindings: b2,
            },
            Solution {
                path: vec![],
                bindings: b3,
            },
        ];

        let result =
            apply_aggregate(&AggregateFunction::Min("price".to_string()), &solutions).unwrap();
        assert_eq!(result, Value::Number(49.99));
    }

    #[test]
    fn test_apply_max() {
        let mut b1 = HashMap::new();
        b1.insert("price".to_string(), Value::Number(99.99));

        let mut b2 = HashMap::new();
        b2.insert("price".to_string(), Value::Number(49.99));

        let mut b3 = HashMap::new();
        b3.insert("price".to_string(), Value::Number(149.99));

        let solutions = vec![
            Solution {
                path: vec![],
                bindings: b1,
            },
            Solution {
                path: vec![],
                bindings: b2,
            },
            Solution {
                path: vec![],
                bindings: b3,
            },
        ];

        let result =
            apply_aggregate(&AggregateFunction::Max("price".to_string()), &solutions).unwrap();
        assert_eq!(result, Value::Number(149.99));
    }

    #[test]
    fn test_apply_empty_solutions() {
        let solutions = vec![];

        let count = apply_aggregate(&AggregateFunction::Count, &solutions).unwrap();
        assert_eq!(count, Value::Integer(0));

        let sum =
            apply_aggregate(&AggregateFunction::Sum("amount".to_string()), &solutions).unwrap();
        assert_eq!(sum, Value::Number(0.0));

        let min =
            apply_aggregate(&AggregateFunction::Min("price".to_string()), &solutions).unwrap();
        assert_eq!(min, Value::Null);
    }
}

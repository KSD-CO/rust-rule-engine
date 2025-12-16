//! Expression Evaluator
//!
//! This module provides runtime evaluation of arithmetic expressions
//! similar to CLIPS (bind ?total (* ?quantity ?price))

use crate::engine::facts::Facts;
use crate::errors::{Result, RuleEngineError};
use crate::types::Value;

/// Evaluate an arithmetic expression with field references
/// Example: "Order.quantity * Order.price" with facts containing Order.quantity=10, Order.price=100
/// Returns: Value::Integer(1000) or Value::Number(1000.0)
pub fn evaluate_expression(expr: &str, facts: &Facts) -> Result<Value> {
    let expr = expr.trim();

    // Try to evaluate as simple arithmetic expression
    // Support: +, -, *, /, %

    // Find the operator (right to left for correct precedence)
    // Precedence: *, /, % (higher) then +, - (lower)

    // First pass: look for + or - (lowest precedence)
    if let Some(pos) = find_operator(expr, &['+', '-']) {
        let left = &expr[..pos].trim();
        let op = &expr[pos..pos + 1];
        let right = &expr[pos + 1..].trim();

        let left_val = evaluate_expression(left, facts)?;
        let right_val = evaluate_expression(right, facts)?;

        return apply_operator(&left_val, op, &right_val);
    }

    // Second pass: look for *, /, % (higher precedence)
    if let Some(pos) = find_operator(expr, &['*', '/', '%']) {
        let left = &expr[..pos].trim();
        let op = &expr[pos..pos + 1];
        let right = &expr[pos + 1..].trim();

        let left_val = evaluate_expression(left, facts)?;
        let right_val = evaluate_expression(right, facts)?;

        return apply_operator(&left_val, op, &right_val);
    }

    // No operator found - must be a single value
    // Could be: field reference (Order.quantity), number (100), or variable

    // Try to parse as number first
    if let Ok(int_val) = expr.parse::<i64>() {
        return Ok(Value::Integer(int_val));
    }

    if let Ok(float_val) = expr.parse::<f64>() {
        return Ok(Value::Number(float_val));
    }

    // Must be a field reference - get from facts
    if let Some(value) = facts.get(expr) {
        return Ok(value.clone());
    }

    // Field not found - return error
    Err(RuleEngineError::EvaluationError {
        message: format!("Field '{}' not found in facts", expr),
    })
}

/// Find position of operator, skipping parentheses
/// Returns rightmost occurrence for left-to-right evaluation
fn find_operator(expr: &str, operators: &[char]) -> Option<usize> {
    let mut paren_depth = 0;
    let mut last_pos = None;

    for (i, ch) in expr.chars().enumerate() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth -= 1,
            _ if paren_depth == 0 && operators.contains(&ch) => {
                last_pos = Some(i);
            }
            _ => {}
        }
    }

    last_pos
}

/// Apply arithmetic operator to two values
fn apply_operator(left: &Value, op: &str, right: &Value) -> Result<Value> {
    // Convert to numbers
    let left_num = value_to_number(left)?;
    let right_num = value_to_number(right)?;

    let result = match op {
        "+" => left_num + right_num,
        "-" => left_num - right_num,
        "*" => left_num * right_num,
        "/" => {
            if right_num == 0.0 {
                return Err(RuleEngineError::EvaluationError {
                    message: "Division by zero".to_string(),
                });
            }
            left_num / right_num
        }
        "%" => left_num % right_num,
        _ => {
            return Err(RuleEngineError::EvaluationError {
                message: format!("Unknown operator: {}", op),
            });
        }
    };

    // Return integer if both operands were integers and result is whole number
    if is_integer_value(left) && is_integer_value(right) && result.fract() == 0.0 {
        Ok(Value::Integer(result as i64))
    } else {
        Ok(Value::Number(result))
    }
}

/// Convert Value to f64 for arithmetic
fn value_to_number(value: &Value) -> Result<f64> {
    match value {
        Value::Integer(i) => Ok(*i as f64),
        Value::Number(n) => Ok(*n),
        Value::String(s) => s
            .parse::<f64>()
            .map_err(|_| RuleEngineError::EvaluationError {
                message: format!("Cannot convert '{}' to number", s),
            }),
        _ => Err(RuleEngineError::EvaluationError {
            message: format!("Cannot convert {:?} to number", value),
        }),
    }
}

/// Check if Value represents an integer
fn is_integer_value(value: &Value) -> bool {
    matches!(value, Value::Integer(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic() {
        let facts = Facts::new();

        assert_eq!(
            evaluate_expression("10 + 20", &facts).unwrap(),
            Value::Integer(30)
        );

        assert_eq!(
            evaluate_expression("100 - 25", &facts).unwrap(),
            Value::Integer(75)
        );

        assert_eq!(
            evaluate_expression("5 * 6", &facts).unwrap(),
            Value::Integer(30)
        );

        assert_eq!(
            evaluate_expression("100 / 4", &facts).unwrap(),
            Value::Integer(25)
        );
    }

    #[test]
    fn test_field_references() {
        let facts = Facts::new();
        facts.set("Order.quantity", Value::Integer(10));
        facts.set("Order.price", Value::Integer(100));

        assert_eq!(
            evaluate_expression("Order.quantity * Order.price", &facts).unwrap(),
            Value::Integer(1000)
        );
    }

    #[test]
    fn test_mixed_operations() {
        let facts = Facts::new();
        facts.set("a", Value::Integer(10));
        facts.set("b", Value::Integer(5));
        facts.set("c", Value::Integer(2));

        // 10 + 5 * 2 = 10 + 10 = 20
        assert_eq!(
            evaluate_expression("a + b * c", &facts).unwrap(),
            Value::Integer(20)
        );
    }
}

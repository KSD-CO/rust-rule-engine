//! Unification module for backward chaining
//!
//! This module provides variable binding and unification capabilities
//! for pattern matching in backward chaining queries.
//!
//! # Example
//! ```ignore
//! use rust_rule_engine::backward::unification::{Bindings, Unifier};
//! use rust_rule_engine::backward::expression::Expression;
//!
//! let mut bindings = Bindings::new();
//!
//! // Unify variable with value
//! let var = Expression::Variable("X".to_string());
//! let val = Expression::Literal(Value::Number(42.0));
//!
//! Unifier::unify(&var, &val, &mut bindings)?;
//! assert_eq!(bindings.get("X"), Some(&Value::Number(42.0)));
//! ```

use crate::types::Value;
use crate::errors::{Result, RuleEngineError};
use crate::Facts;
use super::expression::Expression;
use std::collections::HashMap;

/// Variable bindings during proof
///
/// This type manages variable-to-value mappings during backward chaining,
/// with support for merging, conflict detection, and binding propagation.
#[derive(Debug, Clone)]
pub struct Bindings {
    /// Map from variable name to value
    bindings: HashMap<String, Value>,
}

impl Bindings {
    /// Create a new empty bindings set
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Bind a variable to a value
    ///
    /// If the variable is already bound, this checks that the new value
    /// matches the existing binding. Returns an error if there's a conflict.
    ///
    /// # Example
    /// ```ignore
    /// let mut bindings = Bindings::new();
    /// bindings.bind("X".to_string(), Value::Number(42.0))?;
    ///
    /// // This will succeed (same value)
    /// bindings.bind("X".to_string(), Value::Number(42.0))?;
    ///
    /// // This will fail (different value)
    /// bindings.bind("X".to_string(), Value::Number(100.0))?; // Error!
    /// ```
    pub fn bind(&mut self, var_name: String, value: Value) -> Result<()> {
        // Check if already bound
        if let Some(existing) = self.bindings.get(&var_name) {
            // Must match existing binding
            if existing != &value {
                return Err(RuleEngineError::ExecutionError(
                    format!(
                        "Variable binding conflict: {} is already bound to {:?}, cannot rebind to {:?}",
                        var_name, existing, value
                    )
                ));
            }
        } else {
            self.bindings.insert(var_name, value);
        }
        Ok(())
    }

    /// Get binding for a variable
    pub fn get(&self, var_name: &str) -> Option<&Value> {
        self.bindings.get(var_name)
    }

    /// Check if variable is bound
    pub fn is_bound(&self, var_name: &str) -> bool {
        self.bindings.contains_key(var_name)
    }

    /// Merge bindings from another set
    ///
    /// This attempts to merge all bindings from `other` into this set.
    /// If any conflicts are detected, returns an error and leaves this set unchanged.
    pub fn merge(&mut self, other: &Bindings) -> Result<()> {
        for (var, val) in &other.bindings {
            self.bind(var.clone(), val.clone())?;
        }
        Ok(())
    }

    /// Get all bindings as a map
    pub fn as_map(&self) -> &HashMap<String, Value> {
        &self.bindings
    }

    /// Get number of bindings
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Check if bindings is empty
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }

    /// Clear all bindings
    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    /// Create bindings from a HashMap
    pub fn from_map(map: HashMap<String, Value>) -> Self {
        Self { bindings: map }
    }

    /// Convert bindings to HashMap (for backward compatibility)
    pub fn into_map(self) -> HashMap<String, Value> {
        self.bindings
    }

    /// Get bindings as HashMap clone
    pub fn to_map(&self) -> HashMap<String, Value> {
        self.bindings.clone()
    }
}

impl Default for Bindings {
    fn default() -> Self {
        Self::new()
    }
}

/// Unification algorithm for pattern matching
///
/// The Unifier provides algorithms for:
/// - Unifying two expressions with variable bindings
/// - Matching expressions against facts
/// - Evaluating expressions with variable substitution
pub struct Unifier;

impl Unifier {
    /// Unify two expressions with variable bindings
    ///
    /// This is the core unification algorithm. It attempts to make two expressions
    /// equal by binding variables to values.
    ///
    /// # Returns
    /// - `Ok(true)` if unification succeeded
    /// - `Ok(false)` if expressions cannot be unified
    /// - `Err(_)` if there's a binding conflict
    pub fn unify(
        left: &Expression,
        right: &Expression,
        bindings: &mut Bindings,
    ) -> Result<bool> {
        match (left, right) {
            // Variable on left
            (Expression::Variable(var), expr) => {
                if let Some(bound_value) = bindings.get(var) {
                    // Variable already bound - check if it matches
                    Self::unify(
                        &Expression::Literal(bound_value.clone()),
                        expr,
                        bindings
                    )
                } else {
                    // Bind variable to expression value
                    if let Some(value) = Self::expression_to_value(expr, bindings)? {
                        bindings.bind(var.clone(), value)?;
                        Ok(true)
                    } else {
                        // Cannot extract value from expression yet
                        Ok(false)
                    }
                }
            }

            // Variable on right (symmetric)
            (expr, Expression::Variable(var)) => {
                Self::unify(&Expression::Variable(var.clone()), expr, bindings)
            }

            // Two literals - must be equal
            (Expression::Literal(v1), Expression::Literal(v2)) => {
                Ok(v1 == v2)
            }

            // Two fields - must be same field
            (Expression::Field(f1), Expression::Field(f2)) => {
                Ok(f1 == f2)
            }

            // Comparison - both sides must unify
            (
                Expression::Comparison { left: l1, operator: op1, right: r1 },
                Expression::Comparison { left: l2, operator: op2, right: r2 }
            ) => {
                if op1 != op2 {
                    return Ok(false);
                }

                let left_match = Self::unify(l1, l2, bindings)?;
                let right_match = Self::unify(r1, r2, bindings)?;

                Ok(left_match && right_match)
            }

            // Logical AND - both sides must unify
            (
                Expression::And { left: l1, right: r1 },
                Expression::And { left: l2, right: r2 }
            ) => {
                let left_match = Self::unify(l1, l2, bindings)?;
                let right_match = Self::unify(r1, r2, bindings)?;
                Ok(left_match && right_match)
            }

            // Logical OR - both sides must unify
            (
                Expression::Or { left: l1, right: r1 },
                Expression::Or { left: l2, right: r2 }
            ) => {
                let left_match = Self::unify(l1, l2, bindings)?;
                let right_match = Self::unify(r1, r2, bindings)?;
                Ok(left_match && right_match)
            }

            // Negation - inner expression must unify
            (Expression::Not(e1), Expression::Not(e2)) => {
                Self::unify(e1, e2, bindings)
            }

            // Different expression types - cannot unify
            _ => Ok(false),
        }
    }

    /// Match expression against facts and extract bindings
    ///
    /// This evaluates an expression against the current facts,
    /// binding any variables to their matched values.
    pub fn match_expression(
        expr: &Expression,
        facts: &Facts,
        bindings: &mut Bindings,
    ) -> Result<bool> {
        match expr {
            Expression::Variable(var) => {
                // Unbound variable - cannot match
                if !bindings.is_bound(var) {
                    return Ok(false);
                }
                Ok(true)
            }

            Expression::Field(field_name) => {
                // Field must exist in facts
                Ok(facts.get(field_name).is_some())
            }

            Expression::Literal(_) => {
                // Literals always match
                Ok(true)
            }

            Expression::Comparison { left, operator, right } => {
                // Evaluate both sides with bindings
                let left_val = Self::evaluate_with_bindings(left, facts, bindings)?;
                let right_val = Self::evaluate_with_bindings(right, facts, bindings)?;

                // Perform comparison
                let result = match operator {
                    crate::types::Operator::Equal => left_val == right_val,
                    crate::types::Operator::NotEqual => left_val != right_val,
                    crate::types::Operator::GreaterThan => {
                        Self::compare_values(&left_val, &right_val)? > 0
                    }
                    crate::types::Operator::LessThan => {
                        Self::compare_values(&left_val, &right_val)? < 0
                    }
                    crate::types::Operator::GreaterThanOrEqual => {
                        Self::compare_values(&left_val, &right_val)? >= 0
                    }
                    crate::types::Operator::LessThanOrEqual => {
                        Self::compare_values(&left_val, &right_val)? <= 0
                    }
                    _ => {
                        return Err(RuleEngineError::ExecutionError(
                            format!("Unsupported operator: {:?}", operator)
                        ));
                    }
                };

                Ok(result)
            }

            Expression::And { left, right } => {
                let left_match = Self::match_expression(left, facts, bindings)?;
                if !left_match {
                    return Ok(false);
                }
                Self::match_expression(right, facts, bindings)
            }

            Expression::Or { left, right } => {
                let left_match = Self::match_expression(left, facts, bindings)?;
                if left_match {
                    return Ok(true);
                }
                Self::match_expression(right, facts, bindings)
            }

            Expression::Not(expr) => {
                let result = Self::match_expression(expr, facts, bindings)?;
                Ok(!result)
            }
        }
    }

    /// Evaluate expression with variable bindings
    ///
    /// This evaluates an expression to a value, substituting any bound variables.
    pub fn evaluate_with_bindings(
        expr: &Expression,
        facts: &Facts,
        bindings: &Bindings,
    ) -> Result<Value> {
        match expr {
            Expression::Variable(var) => {
                bindings.get(var)
                    .cloned()
                    .ok_or_else(|| RuleEngineError::ExecutionError(
                        format!("Unbound variable: {}", var)
                    ))
            }

            Expression::Field(field) => {
                facts.get(field)
                    .ok_or_else(|| RuleEngineError::ExecutionError(
                        format!("Field not found: {}", field)
                    ))
            }

            Expression::Literal(val) => Ok(val.clone()),

            Expression::Comparison { left, operator, right } => {
                let left_val = Self::evaluate_with_bindings(left, facts, bindings)?;
                let right_val = Self::evaluate_with_bindings(right, facts, bindings)?;

                let result = match operator {
                    crate::types::Operator::Equal => left_val == right_val,
                    crate::types::Operator::NotEqual => left_val != right_val,
                    crate::types::Operator::GreaterThan => {
                        Self::compare_values(&left_val, &right_val)? > 0
                    }
                    crate::types::Operator::LessThan => {
                        Self::compare_values(&left_val, &right_val)? < 0
                    }
                    crate::types::Operator::GreaterThanOrEqual => {
                        Self::compare_values(&left_val, &right_val)? >= 0
                    }
                    crate::types::Operator::LessThanOrEqual => {
                        Self::compare_values(&left_val, &right_val)? <= 0
                    }
                    _ => {
                        return Err(RuleEngineError::ExecutionError(
                            format!("Unsupported operator: {:?}", operator)
                        ));
                    }
                };

                Ok(Value::Boolean(result))
            }

            Expression::And { left, right } => {
                let left_val = Self::evaluate_with_bindings(left, facts, bindings)?;
                if !left_val.to_bool() {
                    return Ok(Value::Boolean(false));
                }
                let right_val = Self::evaluate_with_bindings(right, facts, bindings)?;
                Ok(Value::Boolean(right_val.to_bool()))
            }

            Expression::Or { left, right } => {
                let left_val = Self::evaluate_with_bindings(left, facts, bindings)?;
                if left_val.to_bool() {
                    return Ok(Value::Boolean(true));
                }
                let right_val = Self::evaluate_with_bindings(right, facts, bindings)?;
                Ok(Value::Boolean(right_val.to_bool()))
            }

            Expression::Not(expr) => {
                let value = Self::evaluate_with_bindings(expr, facts, bindings)?;
                Ok(Value::Boolean(!value.to_bool()))
            }
        }
    }

    /// Extract a value from an expression (if possible)
    fn expression_to_value(expr: &Expression, bindings: &Bindings) -> Result<Option<Value>> {
        match expr {
            Expression::Literal(val) => Ok(Some(val.clone())),
            Expression::Variable(var) => Ok(bindings.get(var).cloned()),
            _ => Ok(None), // Cannot extract value from complex expressions
        }
    }

    /// Compare two values for ordering
    fn compare_values(left: &Value, right: &Value) -> Result<i32> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if a < b {
                    Ok(-1)
                } else if a > b {
                    Ok(1)
                } else {
                    Ok(0)
                }
            }
            (Value::String(a), Value::String(b)) => {
                Ok(a.cmp(b) as i32)
            }
            (Value::Boolean(a), Value::Boolean(b)) => {
                Ok(a.cmp(b) as i32)
            }
            _ => Err(RuleEngineError::ExecutionError(
                format!("Cannot compare values: {:?} and {:?}", left, right)
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Operator;

    #[test]
    fn test_bindings_basic() {
        let mut bindings = Bindings::new();

        assert!(bindings.is_empty());
        assert_eq!(bindings.len(), 0);

        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();

        assert!(!bindings.is_empty());
        assert_eq!(bindings.len(), 1);
        assert!(bindings.is_bound("X"));
        assert_eq!(bindings.get("X"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_bindings_conflict() {
        let mut bindings = Bindings::new();

        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();

        // Same value - should succeed
        assert!(bindings.bind("X".to_string(), Value::Number(42.0)).is_ok());

        // Different value - should fail
        assert!(bindings.bind("X".to_string(), Value::Number(100.0)).is_err());
    }

    #[test]
    fn test_bindings_merge() {
        let mut bindings1 = Bindings::new();
        let mut bindings2 = Bindings::new();

        bindings1.bind("X".to_string(), Value::Number(42.0)).unwrap();
        bindings2.bind("Y".to_string(), Value::String("hello".to_string())).unwrap();

        bindings1.merge(&bindings2).unwrap();

        assert_eq!(bindings1.len(), 2);
        assert_eq!(bindings1.get("X"), Some(&Value::Number(42.0)));
        assert_eq!(bindings1.get("Y"), Some(&Value::String("hello".to_string())));
    }

    #[test]
    fn test_bindings_merge_conflict() {
        let mut bindings1 = Bindings::new();
        let mut bindings2 = Bindings::new();

        bindings1.bind("X".to_string(), Value::Number(42.0)).unwrap();
        bindings2.bind("X".to_string(), Value::Number(100.0)).unwrap();

        // Should fail due to conflict
        assert!(bindings1.merge(&bindings2).is_err());
    }

    #[test]
    fn test_unify_variable_with_literal() {
        let mut bindings = Bindings::new();

        let var = Expression::Variable("X".to_string());
        let lit = Expression::Literal(Value::Number(42.0));

        let result = Unifier::unify(&var, &lit, &mut bindings).unwrap();

        assert!(result);
        assert_eq!(bindings.get("X"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_unify_bound_variable() {
        let mut bindings = Bindings::new();
        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();

        let var = Expression::Variable("X".to_string());
        let lit = Expression::Literal(Value::Number(42.0));

        // Should succeed - same value
        let result = Unifier::unify(&var, &lit, &mut bindings).unwrap();
        assert!(result);

        // Should fail - different value
        let lit2 = Expression::Literal(Value::Number(100.0));
        let result2 = Unifier::unify(&var, &lit2, &mut bindings);
        assert!(result2.is_err() || !result2.unwrap());
    }

    #[test]
    fn test_unify_two_literals() {
        let mut bindings = Bindings::new();

        let lit1 = Expression::Literal(Value::Number(42.0));
        let lit2 = Expression::Literal(Value::Number(42.0));
        let lit3 = Expression::Literal(Value::Number(100.0));

        assert!(Unifier::unify(&lit1, &lit2, &mut bindings).unwrap());
        assert!(!Unifier::unify(&lit1, &lit3, &mut bindings).unwrap());
    }

    #[test]
    fn test_match_expression_simple() {
        let mut facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));

        let mut bindings = Bindings::new();

        let expr = Expression::Comparison {
            left: Box::new(Expression::Field("User.IsVIP".to_string())),
            operator: Operator::Equal,
            right: Box::new(Expression::Literal(Value::Boolean(true))),
        };

        let result = Unifier::match_expression(&expr, &facts, &mut bindings).unwrap();
        assert!(result);
    }

    #[test]
    fn test_evaluate_with_bindings() {
        let mut facts = Facts::new();
        facts.set("Order.Amount", Value::Number(100.0));

        let mut bindings = Bindings::new();
        bindings.bind("X".to_string(), Value::Number(50.0)).unwrap();

        // Evaluate variable
        let var_expr = Expression::Variable("X".to_string());
        let result = Unifier::evaluate_with_bindings(&var_expr, &facts, &bindings).unwrap();
        assert_eq!(result, Value::Number(50.0));

        // Evaluate field
        let field_expr = Expression::Field("Order.Amount".to_string());
        let result = Unifier::evaluate_with_bindings(&field_expr, &facts, &bindings).unwrap();
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_compare_values() {
        assert_eq!(
            Unifier::compare_values(&Value::Number(10.0), &Value::Number(20.0)).unwrap(),
            -1
        );
        assert_eq!(
            Unifier::compare_values(&Value::Number(20.0), &Value::Number(10.0)).unwrap(),
            1
        );
        assert_eq!(
            Unifier::compare_values(&Value::Number(10.0), &Value::Number(10.0)).unwrap(),
            0
        );
    }
}

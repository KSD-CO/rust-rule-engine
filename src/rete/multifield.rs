//! Multi-field (Multislot) Variables - CLIPS-inspired Feature
//!
//! This module implements CLIPS-style multi-field variables for pattern matching
//! on arrays and collections. Multi-field variables allow:
//!
//! - Collecting all values: `Order.items $?all_items`
//! - Checking containment: `Product.tags contains "electronics"`
//! - Getting array length: `Order.items count > 0`
//! - Accessing elements: `Order.items first`, `Order.items last`
//!
//! ## CLIPS Reference
//!
//! ```clips
//! (deftemplate order
//!   (slot order-id)
//!   (multislot items))
//!
//! (defrule process-order
//!   (order (order-id ?id) (items $?all-items))
//!   =>
//!   (foreach ?item $?all-items
//!     (process ?item)))
//! ```
//!
//! ## Rust API
//!
//! ```rust,ignore
//! use rust_rule_engine::rete::{MultifieldOp, PatternConstraint};
//!
//! // Pattern: Order.items $?all_items
//! let constraint = PatternConstraint::MultiField {
//!     field: "items".to_string(),
//!     variable: Some("$?all_items".to_string()),
//!     operator: MultifieldOp::Collect,
//!     value: None,
//! };
//!
//! // Pattern: Product.tags contains "electronics"
//! let constraint = PatternConstraint::MultiField {
//!     field: "tags".to_string(),
//!     variable: None,
//!     operator: MultifieldOp::Contains,
//!     value: Some(FactValue::String("electronics".to_string())),
//! };
//! ```

use crate::rete::facts::{FactValue, TypedFacts};
use crate::errors::{Result, RuleEngineError};
use std::collections::HashMap;

/// Multi-field operations for pattern matching
///
/// These operations enable CLIPS-style multi-field variable matching and manipulation.
#[derive(Debug, Clone, PartialEq)]
pub enum MultifieldOp {
    /// Collect all values into a variable: `$?var`
    ///
    /// Example: `Order.items $?all_items` binds all items to `$?all_items`
    Collect,

    /// Check if array contains a specific value
    ///
    /// Example: `Product.tags contains "electronics"`
    Contains,

    /// Get the count/length of the array
    ///
    /// Example: `Order.items count` returns the number of items
    Count,

    /// Get the first element of the array
    ///
    /// Example: `Order.items first` returns the first item
    First,

    /// Get the last element of the array
    ///
    /// Example: `Order.items last` returns the last item
    Last,

    /// Get a specific element by index (0-based)
    ///
    /// Example: `Order.items[0]` returns the first item
    Index(usize),

    /// Get a slice of the array [start:end]
    ///
    /// Example: `Order.items[1:3]` returns items at index 1 and 2
    Slice(usize, usize),

    /// Check if array is empty
    ///
    /// Example: `Order.items empty`
    IsEmpty,

    /// Check if array is not empty
    ///
    /// Example: `Order.items not_empty`
    NotEmpty,
}

impl MultifieldOp {
    /// Evaluate a multi-field operation on facts
    ///
    /// Returns:
    /// - `Some(Vec<FactValue>)` - Collection of values (for Collect, Slice)
    /// - `Some(vec![FactValue::Integer(n)])` - Numeric result (for Count, Index)
    /// - `Some(vec![FactValue::Boolean(b)])` - Boolean result (for Contains, IsEmpty, etc.)
    /// - `None` - Operation failed (field not found, invalid type, etc.)
    pub fn evaluate(
        &self,
        facts: &TypedFacts,
        field: &str,
        value: Option<&FactValue>,
    ) -> Option<Vec<FactValue>> {
        // Get the field value
        let field_value = facts.get(field)?;

        // Ensure it's an array
        let array = match field_value {
            FactValue::Array(arr) => arr,
            _ => return None, // Not an array
        };

        match self {
            MultifieldOp::Collect => {
                // Return all values
                Some(array.clone())
            }

            MultifieldOp::Contains => {
                // Check if array contains the specified value
                let search_value = value?;
                let contains = array.contains(search_value);
                Some(vec![FactValue::Boolean(contains)])
            }

            MultifieldOp::Count => {
                // Return array length
                Some(vec![FactValue::Integer(array.len() as i64)])
            }

            MultifieldOp::First => {
                // Return first element
                array.first().cloned().map(|v| vec![v])
            }

            MultifieldOp::Last => {
                // Return last element
                array.last().cloned().map(|v| vec![v])
            }

            MultifieldOp::Index(idx) => {
                // Return element at index
                array.get(*idx).cloned().map(|v| vec![v])
            }

            MultifieldOp::Slice(start, end) => {
                // Return slice of array
                let end = (*end).min(array.len());
                if *start >= end {
                    return Some(Vec::new());
                }
                Some(array[*start..end].to_vec())
            }

            MultifieldOp::IsEmpty => {
                // Check if array is empty
                Some(vec![FactValue::Boolean(array.is_empty())])
            }

            MultifieldOp::NotEmpty => {
                // Check if array is not empty
                Some(vec![FactValue::Boolean(!array.is_empty())])
            }
        }
    }

    /// Get a string representation of the operation
    pub fn to_string(&self) -> String {
        match self {
            MultifieldOp::Collect => "collect".to_string(),
            MultifieldOp::Contains => "contains".to_string(),
            MultifieldOp::Count => "count".to_string(),
            MultifieldOp::First => "first".to_string(),
            MultifieldOp::Last => "last".to_string(),
            MultifieldOp::Index(idx) => format!("[{}]", idx),
            MultifieldOp::Slice(start, end) => format!("[{}:{}]", start, end),
            MultifieldOp::IsEmpty => "empty".to_string(),
            MultifieldOp::NotEmpty => "not_empty".to_string(),
        }
    }
}

/// Helper function to evaluate multi-field patterns in rules
///
/// This function combines the multi-field operation with variable binding.
/// It returns both the result values and optional variable bindings.
///
/// # Arguments
///
/// * `facts` - The facts to evaluate against
/// * `field` - The field name (e.g., "items")
/// * `operator` - The multi-field operation
/// * `variable` - Optional variable name for binding (e.g., "$?all_items")
/// * `value` - Optional value for operations like Contains
/// * `bindings` - Existing variable bindings
///
/// # Returns
///
/// `Some(HashMap<Variable, FactValue>)` - New bindings including multi-field results
/// `None` - Pattern doesn't match
pub fn evaluate_multifield_pattern(
    facts: &TypedFacts,
    field: &str,
    operator: &MultifieldOp,
    variable: Option<&str>,
    value: Option<&FactValue>,
    bindings: &HashMap<String, FactValue>,
) -> Option<HashMap<String, FactValue>> {
    // Evaluate the multi-field operation
    let result = operator.evaluate(facts, field, value)?;

    // Create new bindings
    let mut new_bindings = bindings.clone();

    // If there's a variable, bind the result
    if let Some(var_name) = variable {
        // For Collect operation, bind as array
        if matches!(operator, MultifieldOp::Collect) {
            new_bindings.insert(var_name.to_string(), FactValue::Array(result));
        } else {
            // For single-value results, unwrap
            if result.len() == 1 {
                new_bindings.insert(var_name.to_string(), result[0].clone());
            } else {
                // Multiple values, bind as array
                new_bindings.insert(var_name.to_string(), FactValue::Array(result));
            }
        }
    } else {
        // No variable binding, just check if operation succeeded
        // For boolean operations, check the result
        if result.len() == 1 {
            if let FactValue::Boolean(b) = result[0] {
                if !b {
                    return None; // Pattern doesn't match
                }
            }
        }
    }

    Some(new_bindings)
}

/// Parse multi-field variable syntax
///
/// Recognizes patterns like:
/// - `$?var` - Multi-field variable (collects all values)
/// - `$var` - Single-value binding (not multi-field)
///
/// Returns `Some(variable_name)` if it's a multi-field variable, `None` otherwise
pub fn parse_multifield_variable(input: &str) -> Option<String> {
    let trimmed = input.trim();
    if trimmed.starts_with("$?") {
        Some(trimmed[2..].to_string())
    } else {
        None
    }
}

/// Check if a string is a multi-field variable
pub fn is_multifield_variable(input: &str) -> bool {
    input.trim().starts_with("$?")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_facts_with_array() -> TypedFacts {
        let mut facts = TypedFacts::new();
        facts.set("items", FactValue::Array(vec![
            FactValue::String("item1".to_string()),
            FactValue::String("item2".to_string()),
            FactValue::String("item3".to_string()),
        ]));
        facts.set("tags", FactValue::Array(vec![
            FactValue::String("electronics".to_string()),
            FactValue::String("gadgets".to_string()),
        ]));
        facts
    }

    #[test]
    fn test_collect_operation() {
        let facts = create_test_facts_with_array();
        let op = MultifieldOp::Collect;

        let result = op.evaluate(&facts, "items", None);
        assert!(result.is_some());

        let values = result.unwrap();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], FactValue::String("item1".to_string()));
    }

    #[test]
    fn test_contains_operation() {
        let facts = create_test_facts_with_array();
        let op = MultifieldOp::Contains;
        let search = FactValue::String("electronics".to_string());

        let result = op.evaluate(&facts, "tags", Some(&search));
        assert!(result.is_some());

        let values = result.unwrap();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], FactValue::Boolean(true));
    }

    #[test]
    fn test_count_operation() {
        let facts = create_test_facts_with_array();
        let op = MultifieldOp::Count;

        let result = op.evaluate(&facts, "items", None);
        assert!(result.is_some());

        let values = result.unwrap();
        assert_eq!(values[0], FactValue::Integer(3));
    }

    #[test]
    fn test_first_last_operations() {
        let facts = create_test_facts_with_array();

        let first = MultifieldOp::First.evaluate(&facts, "items", None).unwrap();
        assert_eq!(first[0], FactValue::String("item1".to_string()));

        let last = MultifieldOp::Last.evaluate(&facts, "items", None).unwrap();
        assert_eq!(last[0], FactValue::String("item3".to_string()));
    }

    #[test]
    fn test_index_operation() {
        let facts = create_test_facts_with_array();
        let op = MultifieldOp::Index(1);

        let result = op.evaluate(&facts, "items", None).unwrap();
        assert_eq!(result[0], FactValue::String("item2".to_string()));
    }

    #[test]
    fn test_slice_operation() {
        let facts = create_test_facts_with_array();
        let op = MultifieldOp::Slice(0, 2);

        let result = op.evaluate(&facts, "items", None).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], FactValue::String("item1".to_string()));
        assert_eq!(result[1], FactValue::String("item2".to_string()));
    }

    #[test]
    fn test_is_empty_operation() {
        let mut facts = TypedFacts::new();
        facts.set("empty_array", FactValue::Array(Vec::new()));

        let op = MultifieldOp::IsEmpty;
        let result = op.evaluate(&facts, "empty_array", None).unwrap();
        assert_eq!(result[0], FactValue::Boolean(true));
    }

    #[test]
    fn test_parse_multifield_variable() {
        assert_eq!(parse_multifield_variable("$?items"), Some("items".to_string()));
        assert_eq!(parse_multifield_variable("$?all"), Some("all".to_string()));
        assert_eq!(parse_multifield_variable("$single"), None);
        assert_eq!(parse_multifield_variable("items"), None);
    }

    #[test]
    fn test_is_multifield_variable() {
        assert!(is_multifield_variable("$?items"));
        assert!(is_multifield_variable("$?all"));
        assert!(!is_multifield_variable("$single"));
        assert!(!is_multifield_variable("items"));
    }

    #[test]
    fn test_evaluate_multifield_pattern_with_binding() {
        let facts = create_test_facts_with_array();
        let bindings = HashMap::new();

        let result = evaluate_multifield_pattern(
            &facts,
            "items",
            &MultifieldOp::Collect,
            Some("$?all_items"),
            None,
            &bindings,
        );

        assert!(result.is_some());
        let new_bindings = result.unwrap();

        assert!(new_bindings.contains_key("$?all_items"));
        if let FactValue::Array(arr) = &new_bindings["$?all_items"] {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected array binding");
        }
    }

    #[test]
    fn test_evaluate_multifield_pattern_contains() {
        let facts = create_test_facts_with_array();
        let bindings = HashMap::new();
        let search = FactValue::String("electronics".to_string());

        let result = evaluate_multifield_pattern(
            &facts,
            "tags",
            &MultifieldOp::Contains,
            None,
            Some(&search),
            &bindings,
        );

        assert!(result.is_some()); // Should match

        // Test with non-existent value
        let search2 = FactValue::String("nonexistent".to_string());
        let result2 = evaluate_multifield_pattern(
            &facts,
            "tags",
            &MultifieldOp::Contains,
            None,
            Some(&search2),
            &bindings,
        );

        assert!(result2.is_none()); // Should not match
    }
}

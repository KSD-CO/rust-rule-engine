//! Typed facts system for RETE-UL
//!
//! This module provides a strongly-typed facts system that supports:
//! - Multiple data types (String, Integer, Float, Boolean, Array, Object)
//! - Type-safe operations
//! - Efficient conversions
//! - Better operator support

use std::collections::HashMap;
use std::fmt;

/// Strongly-typed fact value
#[derive(Debug, Clone, PartialEq)]
pub enum FactValue {
    /// String value
    String(String),
    /// Integer value (i64)
    Integer(i64),
    /// Float value (f64)
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<FactValue>),
    /// Null/None value
    Null,
}

impl FactValue {
    /// Convert to string representation
    pub fn as_string(&self) -> String {
        match self {
            FactValue::String(s) => s.clone(),
            FactValue::Integer(i) => i.to_string(),
            FactValue::Float(f) => f.to_string(),
            FactValue::Boolean(b) => b.to_string(),
            FactValue::Array(arr) => format!("{:?}", arr),
            FactValue::Null => "null".to_string(),
        }
    }

    /// Try to convert to integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            FactValue::Integer(i) => Some(*i),
            FactValue::Float(f) => Some(*f as i64),
            FactValue::String(s) => s.parse().ok(),
            FactValue::Boolean(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Try to convert to float
    pub fn as_float(&self) -> Option<f64> {
        match self {
            FactValue::Float(f) => Some(*f),
            FactValue::Integer(i) => Some(*i as f64),
            FactValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Convert to number (f64) for arithmetic operations
    pub fn as_number(&self) -> Option<f64> {
        match self {
            FactValue::Float(f) => Some(*f),
            FactValue::Integer(i) => Some(*i as f64),
            FactValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Try to convert to boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            FactValue::Boolean(b) => Some(*b),
            FactValue::Integer(i) => Some(*i != 0),
            FactValue::String(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "1" => Some(true),
                "false" | "no" | "0" => Some(false),
                _ => None,
            },
            FactValue::Null => Some(false),
            _ => None,
        }
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, FactValue::Null)
    }

    /// Compare with operator
    pub fn compare(&self, operator: &str, other: &FactValue) -> bool {
        match operator {
            "==" => self == other,
            "!=" => self != other,
            ">" => self.compare_gt(other),
            "<" => self.compare_lt(other),
            ">=" => self.compare_gte(other),
            "<=" => self.compare_lte(other),
            "contains" => self.contains(other),
            "startsWith" => self.starts_with(other),
            "endsWith" => self.ends_with(other),
            "matches" => self.matches_pattern(other),
            "in" => self.in_array(other),
            _ => false,
        }
    }

    fn compare_gt(&self, other: &FactValue) -> bool {
        match (self.as_float(), other.as_float()) {
            (Some(a), Some(b)) => a > b,
            _ => false,
        }
    }

    fn compare_lt(&self, other: &FactValue) -> bool {
        match (self.as_float(), other.as_float()) {
            (Some(a), Some(b)) => a < b,
            _ => false,
        }
    }

    fn compare_gte(&self, other: &FactValue) -> bool {
        match (self.as_float(), other.as_float()) {
            (Some(a), Some(b)) => a >= b,
            _ => self == other,
        }
    }

    fn compare_lte(&self, other: &FactValue) -> bool {
        match (self.as_float(), other.as_float()) {
            (Some(a), Some(b)) => a <= b,
            _ => self == other,
        }
    }

    fn contains(&self, other: &FactValue) -> bool {
        match (self, other) {
            (FactValue::String(s), FactValue::String(pattern)) => s.contains(pattern),
            (FactValue::Array(arr), val) => arr.contains(val),
            _ => false,
        }
    }

    fn starts_with(&self, other: &FactValue) -> bool {
        match (self, other) {
            (FactValue::String(s), FactValue::String(prefix)) => s.starts_with(prefix),
            _ => false,
        }
    }

    fn ends_with(&self, other: &FactValue) -> bool {
        match (self, other) {
            (FactValue::String(s), FactValue::String(suffix)) => s.ends_with(suffix),
            _ => false,
        }
    }

    fn matches_pattern(&self, other: &FactValue) -> bool {
        match (self, other) {
            (FactValue::String(s), FactValue::String(pattern)) => {
                // Simple wildcard matching (* and ?)
                wildcard_match(s, pattern)
            }
            _ => false,
        }
    }

    fn in_array(&self, other: &FactValue) -> bool {
        match other {
            FactValue::Array(arr) => arr.contains(self),
            _ => false,
        }
    }
}

impl fmt::Display for FactValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl From<String> for FactValue {
    fn from(s: String) -> Self {
        FactValue::String(s)
    }
}

impl From<&str> for FactValue {
    fn from(s: &str) -> Self {
        FactValue::String(s.to_string())
    }
}

impl From<i64> for FactValue {
    fn from(i: i64) -> Self {
        FactValue::Integer(i)
    }
}

impl From<i32> for FactValue {
    fn from(i: i32) -> Self {
        FactValue::Integer(i as i64)
    }
}

impl From<f64> for FactValue {
    fn from(f: f64) -> Self {
        FactValue::Float(f)
    }
}

impl From<bool> for FactValue {
    fn from(b: bool) -> Self {
        FactValue::Boolean(b)
    }
}

impl From<Vec<FactValue>> for FactValue {
    fn from(arr: Vec<FactValue>) -> Self {
        FactValue::Array(arr)
    }
}

/// Convert from types::Value to FactValue
impl From<crate::types::Value> for FactValue {
    fn from(value: crate::types::Value) -> Self {
        match value {
            crate::types::Value::String(s) => FactValue::String(s),
            crate::types::Value::Number(n) => FactValue::Float(n),
            crate::types::Value::Integer(i) => FactValue::Integer(i),
            crate::types::Value::Boolean(b) => FactValue::Boolean(b),
            crate::types::Value::Array(arr) => {
                FactValue::Array(arr.into_iter().map(|v| v.into()).collect())
            }
            crate::types::Value::Object(obj) => {
                // Convert object to string representation
                FactValue::String(format!("{:?}", obj))
            }
            crate::types::Value::Null => FactValue::Null,
            crate::types::Value::Expression(expr) => FactValue::String(expr),
        }
    }
}

/// Typed facts collection
#[derive(Debug, Clone)]
pub struct TypedFacts {
    data: HashMap<String, FactValue>,
    /// Metadata: mapping from fact type to handle for retraction
    /// Format: "FactType" -> FactHandle
    pub(crate) fact_handles: HashMap<String, super::FactHandle>,
}

impl TypedFacts {
    /// Create new empty facts collection
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            fact_handles: HashMap::new(),
        }
    }

    /// Set metadata about which handle corresponds to which fact type
    pub fn set_fact_handle(&mut self, fact_type: String, handle: super::FactHandle) {
        self.fact_handles.insert(fact_type, handle);
    }

    /// Get handle for a fact type (for retraction)
    pub fn get_fact_handle(&self, fact_type: &str) -> Option<super::FactHandle> {
        self.fact_handles.get(fact_type).copied()
    }

    /// Set a fact
    pub fn set<K: Into<String>, V: Into<FactValue>>(&mut self, key: K, value: V) {
        self.data.insert(key.into(), value.into());
    }

    /// Get a fact
    pub fn get(&self, key: &str) -> Option<&FactValue> {
        self.data.get(key)
    }

    /// Remove a fact
    pub fn remove(&mut self, key: &str) -> Option<FactValue> {
        self.data.remove(key)
    }

    /// Check if key exists
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Get all facts
    pub fn get_all(&self) -> &HashMap<String, FactValue> {
        &self.data
    }

    /// Clear all facts
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Convert to string-based HashMap (for backward compatibility)
    pub fn to_string_map(&self) -> HashMap<String, String> {
        self.data
            .iter()
            .map(|(k, v)| (k.clone(), v.as_string()))
            .collect()
    }

    /// Create from string-based HashMap (for backward compatibility)
    pub fn from_string_map(map: &HashMap<String, String>) -> Self {
        let mut facts = Self::new();
        for (k, v) in map {
            // Try to parse as different types
            if let Ok(i) = v.parse::<i64>() {
                facts.set(k.clone(), i);
            } else if let Ok(f) = v.parse::<f64>() {
                facts.set(k.clone(), f);
            } else if let Ok(b) = v.parse::<bool>() {
                facts.set(k.clone(), b);
            } else {
                facts.set(k.clone(), v.clone());
            }
        }
        facts
    }

    /// Evaluate condition with typed comparison
    pub fn evaluate_condition(&self, field: &str, operator: &str, value: &FactValue) -> bool {
        if let Some(fact_value) = self.get(field) {
            fact_value.compare(operator, value)
        } else {
            false
        }
    }
}

impl Default for TypedFacts {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple wildcard pattern matching
/// Supports * (any characters) and ? (single character)
fn wildcard_match(text: &str, pattern: &str) -> bool {
    let text_chars: Vec<char> = text.chars().collect();
    let pattern_chars: Vec<char> = pattern.chars().collect();

    wildcard_match_impl(&text_chars, &pattern_chars, 0, 0)
}

fn wildcard_match_impl(text: &[char], pattern: &[char], ti: usize, pi: usize) -> bool {
    if pi == pattern.len() {
        return ti == text.len();
    }

    if pattern[pi] == '*' {
        // Match zero or more characters
        for i in ti..=text.len() {
            if wildcard_match_impl(text, pattern, i, pi + 1) {
                return true;
            }
        }
        false
    } else if ti < text.len() && (pattern[pi] == '?' || pattern[pi] == text[ti]) {
        // Match single character or exact match
        wildcard_match_impl(text, pattern, ti + 1, pi + 1)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_value_types() {
        let s = FactValue::String("hello".to_string());
        let i = FactValue::Integer(42);
        let f = FactValue::Float(std::f64::consts::PI);
        let b = FactValue::Boolean(true);

        assert_eq!(s.as_string(), "hello");
        assert_eq!(i.as_integer(), Some(42));
        assert_eq!(f.as_float(), Some(std::f64::consts::PI));
        assert_eq!(b.as_boolean(), Some(true));
    }

    #[test]
    fn test_comparisons() {
        let a = FactValue::Integer(10);
        let b = FactValue::Integer(20);

        assert!(a.compare("<", &b));
        assert!(b.compare(">", &a));
        assert!(a.compare("<=", &a));
        assert!(a.compare("!=", &b));
    }

    #[test]
    fn test_string_operations() {
        let text = FactValue::String("hello world".to_string());
        let pattern = FactValue::String("world".to_string());
        let prefix = FactValue::String("hello".to_string());

        assert!(text.compare("contains", &pattern));
        assert!(text.compare("startsWith", &prefix));
    }

    #[test]
    fn test_wildcard_matching() {
        let text = FactValue::String("hello world".to_string());

        assert!(text.compare("matches", &FactValue::String("hello*".to_string())));
        assert!(text.compare("matches", &FactValue::String("*world".to_string())));
        assert!(text.compare("matches", &FactValue::String("hello?world".to_string())));
        assert!(!text.compare("matches", &FactValue::String("hello?earth".to_string())));
    }

    #[test]
    fn test_array_operations() {
        let arr = FactValue::Array(vec![
            FactValue::Integer(1),
            FactValue::Integer(2),
            FactValue::Integer(3),
        ]);

        let val = FactValue::Integer(2);
        assert!(val.compare("in", &arr));

        let val2 = FactValue::Integer(5);
        assert!(!val2.compare("in", &arr));
    }

    #[test]
    fn test_typed_facts() {
        let mut facts = TypedFacts::new();
        facts.set("age", 25i64);
        facts.set("name", "John");
        facts.set("score", 95.5);
        facts.set("active", true);

        assert_eq!(facts.get("age").unwrap().as_integer(), Some(25));
        assert_eq!(facts.get("name").unwrap().as_string(), "John");
        assert_eq!(facts.get("score").unwrap().as_float(), Some(95.5));
        assert_eq!(facts.get("active").unwrap().as_boolean(), Some(true));
    }

    #[test]
    fn test_evaluate_condition() {
        let mut facts = TypedFacts::new();
        facts.set("age", 25i64);
        facts.set("name", "John Smith");

        assert!(facts.evaluate_condition("age", ">", &FactValue::Integer(18)));
        assert!(facts.evaluate_condition("age", "<=", &FactValue::Integer(30)));
        assert!(facts.evaluate_condition(
            "name",
            "contains",
            &FactValue::String("Smith".to_string())
        ));
        assert!(facts.evaluate_condition(
            "name",
            "startsWith",
            &FactValue::String("John".to_string())
        ));
    }
}

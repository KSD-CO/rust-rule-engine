//! AlphaNode: checks single condition on a fact

use super::facts::{FactValue, TypedFacts};

#[derive(Debug, Clone)]
pub struct AlphaNode {
    pub field: String,
    pub operator: String,
    pub value: String,
}


impl AlphaNode {
    /// Match with string-based facts (backward compatible)
    pub fn matches(&self, fact_field: &str, fact_value: &str) -> bool {
        if self.field != fact_field {
            return false;
        }
        match self.operator.as_str() {
            "==" => fact_value == self.value,
            "!=" => fact_value != self.value,
            ">" => parse_num(fact_value) > parse_num(&self.value),
            "<" => parse_num(fact_value) < parse_num(&self.value),
            ">=" => parse_num(fact_value) >= parse_num(&self.value),
            "<=" => parse_num(fact_value) <= parse_num(&self.value),
            "contains" => fact_value.contains(&self.value),
            "startsWith" => fact_value.starts_with(&self.value),
            "endsWith" => fact_value.ends_with(&self.value),
            "matches" => wildcard_match(fact_value, &self.value),
            _ => false,
        }
    }

    /// Match with typed facts (new!)
    pub fn matches_typed(&self, facts: &TypedFacts) -> bool {
        // Parse the value string into appropriate FactValue
        let expected_value = self.parse_value_string(&self.value);
        facts.evaluate_condition(&self.field, &self.operator, &expected_value)
    }

    /// Parse value string into FactValue
    fn parse_value_string(&self, s: &str) -> FactValue {
        // Try to parse as different types
        if let Ok(i) = s.parse::<i64>() {
            FactValue::Integer(i)
        } else if let Ok(f) = s.parse::<f64>() {
            FactValue::Float(f)
        } else if let Ok(b) = s.parse::<bool>() {
            FactValue::Boolean(b)
        } else if s == "null" {
            FactValue::Null
        } else {
            FactValue::String(s.to_string())
        }
    }

    /// Create with typed value
    pub fn with_typed_value(field: String, operator: String, value: FactValue) -> Self {
        Self {
            field,
            operator,
            value: value.as_string(),
        }
    }
}

fn parse_num(s: &str) -> f64 {
    s.parse::<f64>().unwrap_or(0.0)
}

/// Simple wildcard pattern matching (for backward compatibility)
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
        for i in ti..=text.len() {
            if wildcard_match_impl(text, pattern, i, pi + 1) {
                return true;
            }
        }
        false
    } else if ti < text.len() && (pattern[pi] == '?' || pattern[pi] == text[ti]) {
        wildcard_match_impl(text, pattern, ti + 1, pi + 1)
    } else {
        false
    }
}

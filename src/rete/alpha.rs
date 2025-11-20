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
        // Check if this is a Test CE (arithmetic expression)
        // Test CE fields look like "test(User.Age % 3 == 0)"
        if self.field.starts_with("test(") && self.field.ends_with(')') {
            // Extract the expression: "test(expr)" -> "expr"
            let expr = &self.field[5..self.field.len() - 1];
            
            // Try to evaluate as arithmetic expression
            if let Some(result) = self.evaluate_arithmetic_rete(expr, facts) {
                // Compare result with expected value
                let expected_value = self.parse_value_string(&self.value);
                return match (&result, &expected_value) {
                    (FactValue::Boolean(r), FactValue::Boolean(e)) => r == e,
                    _ => false,
                };
            }
            return false;
        }
        
        // Check if the value is a variable reference (field name in facts)
        // This enables variable-to-variable comparison like "L1 > L1Min"
        let expected_value = if let Some(var_value) = facts.get(&self.value) {
            // Value is a field name - use the field's value for comparison
            var_value.clone()
        } else {
            // Value is a literal - parse it
            self.parse_value_string(&self.value)
        };

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

    /// Evaluate arithmetic expression for RETE
    /// Handles expressions like "User.Age % 3 == 0", "Price * 2 > 100"
    fn evaluate_arithmetic_rete(&self, expr: &str, facts: &TypedFacts) -> Option<FactValue> {
        // Split by comparison operators
        let ops = ["==", "!=", ">=", "<=", ">", "<"];
        for op in &ops {
            if let Some(pos) = expr.find(op) {
                let left = expr[..pos].trim();
                let right = expr[pos + op.len()..].trim();
                
                // Evaluate left side (arithmetic expression)
                let left_val = self.evaluate_arithmetic_expr(left, facts)?;
                
                // Evaluate right side
                let right_val = if let Some(val) = facts.get(right) {
                    val.clone()
                } else if let Ok(i) = right.parse::<i64>() {
                    FactValue::Integer(i)
                } else if let Ok(f) = right.parse::<f64>() {
                    FactValue::Float(f)
                } else {
                    return None;
                };
                
                // Compare values
                let result = left_val.compare(op, &right_val);
                return Some(FactValue::Boolean(result));
            }
        }
        None
    }

    /// Evaluate arithmetic expression (handles +, -, *, /, %)
    fn evaluate_arithmetic_expr(&self, expr: &str, facts: &TypedFacts) -> Option<FactValue> {
        let expr = expr.trim();
        
        // Try arithmetic operators in order of precedence (reverse)
        let ops = ["+", "-", "*", "/", "%"];
        
        for op in &ops {
            if let Some(pos) = expr.rfind(op) {
                // Skip if operator is at the start (negative number)
                if pos == 0 {
                    continue;
                }
                
                let left = expr[..pos].trim();
                let right = expr[pos + 1..].trim();
                
                let left_val = if let Some(val) = facts.get(left) {
                    val.as_number()?
                } else if let Ok(f) = left.parse::<f64>() {
                    f
                } else {
                    // Recursive evaluation
                    self.evaluate_arithmetic_expr(left, facts)?.as_number()?
                };
                
                let right_val = if let Some(val) = facts.get(right) {
                    val.as_number()?
                } else if let Ok(f) = right.parse::<f64>() {
                    f
                } else {
                    self.evaluate_arithmetic_expr(right, facts)?.as_number()?
                };
                
                let result = match *op {
                    "+" => left_val + right_val,
                    "-" => left_val - right_val,
                    "*" => left_val * right_val,
                    "/" => if right_val != 0.0 { left_val / right_val } else { return None; },
                    "%" => left_val % right_val,
                    _ => return None,
                };
                
                // Return Integer if result is whole number, otherwise Float
                if result.fract() == 0.0 {
                    return Some(FactValue::Integer(result as i64));
                } else {
                    return Some(FactValue::Float(result));
                }
            }
        }
        
        // Base case: just a field reference or literal
        if let Some(val) = facts.get(expr) {
            Some(val.clone())
        } else if let Ok(i) = expr.parse::<i64>() {
            Some(FactValue::Integer(i))
        } else if let Ok(f) = expr.parse::<f64>() {
            Some(FactValue::Float(f))
        } else {
            None
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

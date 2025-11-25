/// Expression AST for backward chaining queries
///
/// This module provides a proper Abstract Syntax Tree (AST) for parsing
/// and evaluating backward chaining query expressions.
///
/// # Example
/// ```ignore
/// use rust_rule_engine::backward::expression::{Expression, ExpressionParser};
///
/// let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000")?;
/// let result = expr.evaluate(&facts)?;
/// ```

use crate::types::{Value, Operator};
use crate::errors::{Result, RuleEngineError};
use crate::Facts;

/// Expression AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Field reference (e.g., "User.IsVIP", "Order.Amount")
    Field(String),

    /// Literal value (e.g., true, false, 42, "hello")
    Literal(Value),

    /// Binary comparison (e.g., "X == Y", "A > B")
    Comparison {
        left: Box<Expression>,
        operator: Operator,
        right: Box<Expression>,
    },

    /// Logical AND operation (e.g., "A && B")
    And {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Logical OR operation (e.g., "A || B")
    Or {
        left: Box<Expression>,
        right: Box<Expression>,
    },

    /// Negation (e.g., "!X")
    Not(Box<Expression>),

    /// Variable (for future unification support, e.g., "?X", "?Customer")
    Variable(String),
}

impl Expression {
    /// Evaluate expression against facts
    pub fn evaluate(&self, facts: &Facts) -> Result<Value> {
        match self {
            Expression::Field(name) => {
                facts.get(name)
                    .or_else(|| facts.get_nested(name))
                    .ok_or_else(|| RuleEngineError::ExecutionError(
                        format!("Field not found: {}", name)
                    ))
            }

            Expression::Literal(value) => Ok(value.clone()),

            Expression::Comparison { left, operator, right } => {
                // Special handling for NotEqual when field doesn't exist
                // If field doesn't exist, treat as Null
                let left_val = left.evaluate(facts).unwrap_or(Value::Null);
                let right_val = right.evaluate(facts).unwrap_or(Value::Null);

                let result = operator.evaluate(&left_val, &right_val);
                Ok(Value::Boolean(result))
            }

            Expression::And { left, right } => {
                let left_val = left.evaluate(facts)?;
                if !left_val.to_bool() {
                    return Ok(Value::Boolean(false));
                }
                let right_val = right.evaluate(facts)?;
                Ok(Value::Boolean(right_val.to_bool()))
            }

            Expression::Or { left, right } => {
                let left_val = left.evaluate(facts)?;
                if left_val.to_bool() {
                    return Ok(Value::Boolean(true));
                }
                let right_val = right.evaluate(facts)?;
                Ok(Value::Boolean(right_val.to_bool()))
            }

            Expression::Not(expr) => {
                let value = expr.evaluate(facts)?;
                Ok(Value::Boolean(!value.to_bool()))
            }

            Expression::Variable(var) => {
                Err(RuleEngineError::ExecutionError(
                    format!("Cannot evaluate unbound variable: {}", var)
                ))
            }
        }
    }

    /// Check if expression is satisfied (returns true/false)
    pub fn is_satisfied(&self, facts: &Facts) -> bool {
        self.evaluate(facts)
            .map(|v| v.to_bool())
            .unwrap_or(false)
    }

    /// Extract all field references from expression
    pub fn extract_fields(&self) -> Vec<String> {
        let mut fields = Vec::new();
        self.extract_fields_recursive(&mut fields);
        fields
    }

    fn extract_fields_recursive(&self, fields: &mut Vec<String>) {
        match self {
            Expression::Field(name) => {
                if !fields.contains(name) {
                    fields.push(name.clone());
                }
            }
            Expression::Comparison { left, right, .. } => {
                left.extract_fields_recursive(fields);
                right.extract_fields_recursive(fields);
            }
            Expression::And { left, right } | Expression::Or { left, right } => {
                left.extract_fields_recursive(fields);
                right.extract_fields_recursive(fields);
            }
            Expression::Not(expr) => {
                expr.extract_fields_recursive(fields);
            }
            _ => {}
        }
    }

    /// Convert to human-readable string
    pub fn to_string(&self) -> String {
        match self {
            Expression::Field(name) => name.clone(),
            Expression::Literal(val) => format!("{:?}", val),
            Expression::Comparison { left, operator, right } => {
                format!("{} {:?} {}", left.to_string(), operator, right.to_string())
            }
            Expression::And { left, right } => {
                format!("({} && {})", left.to_string(), right.to_string())
            }
            Expression::Or { left, right } => {
                format!("({} || {})", left.to_string(), right.to_string())
            }
            Expression::Not(expr) => {
                format!("!{}", expr.to_string())
            }
            Expression::Variable(var) => var.clone(),
        }
    }
}

/// Expression parser using recursive descent parsing
pub struct ExpressionParser {
    input: Vec<char>,
    position: usize,
}

impl ExpressionParser {
    /// Create a new parser
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    /// Parse expression from string
    pub fn parse(input: &str) -> Result<Expression> {
        let mut parser = Self::new(input.trim());
        parser.parse_expression()
    }

    /// Parse full expression (handles ||)
    fn parse_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_and_expression()?;

        while self.peek_operator("||") {
            self.consume_operator("||");
            let right = self.parse_and_expression()?;
            left = Expression::Or {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse AND expression (handles &&)
    fn parse_and_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;

        while self.peek_operator("&&") {
            self.consume_operator("&&");
            let right = self.parse_comparison()?;
            left = Expression::And {
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse comparison (e.g., "X == Y", "A > 5")
    fn parse_comparison(&mut self) -> Result<Expression> {
        let left = self.parse_primary()?;

        // Check for comparison operators (check longer operators first)
        let operator = if self.peek_operator("==") {
            self.consume_operator("==");
            Operator::Equal
        } else if self.peek_operator("!=") {
            self.consume_operator("!=");
            Operator::NotEqual
        } else if self.peek_operator(">=") {
            self.consume_operator(">=");
            Operator::GreaterThanOrEqual
        } else if self.peek_operator("<=") {
            self.consume_operator("<=");
            Operator::LessThanOrEqual
        } else if self.peek_operator(">") {
            self.consume_operator(">");
            Operator::GreaterThan
        } else if self.peek_operator("<") {
            self.consume_operator("<");
            Operator::LessThan
        } else {
            // No comparison operator - return just the left side
            return Ok(left);
        };

        let right = self.parse_primary()?;

        Ok(Expression::Comparison {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    /// Parse primary expression (field, literal, variable, or parenthesized)
    fn parse_primary(&mut self) -> Result<Expression> {
        self.skip_whitespace();

        // Handle negation
        if self.peek_char() == Some('!') {
            self.consume_char();
            let expr = self.parse_primary()?;
            return Ok(Expression::Not(Box::new(expr)));
        }

        // Handle parentheses
        if self.peek_char() == Some('(') {
            self.consume_char();
            let expr = self.parse_expression()?;
            self.skip_whitespace();
            if self.peek_char() != Some(')') {
                return Err(RuleEngineError::ParseError {
                    message: format!("Expected closing parenthesis at position {}", self.position),
                });
            }
            self.consume_char();
            return Ok(expr);
        }

        // Handle variables (?X, ?Customer)
        if self.peek_char() == Some('?') {
            self.consume_char();
            let name = self.consume_identifier()?;
            return Ok(Expression::Variable(format!("?{}", name)));
        }

        // Try to parse literal
        if let Some(value) = self.try_parse_literal()? {
            return Ok(Expression::Literal(value));
        }

        // Handle field reference
        let field_name = self.consume_field_path()?;
        Ok(Expression::Field(field_name))
    }

    fn consume_field_path(&mut self) -> Result<String> {
        let mut path = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                path.push(ch);
                self.consume_char();
            } else {
                break;
            }
        }

        if path.is_empty() {
            return Err(RuleEngineError::ParseError {
                message: format!("Expected field name at position {}", self.position),
            });
        }

        Ok(path)
    }

    fn consume_identifier(&mut self) -> Result<String> {
        let mut ident = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.consume_char();
            } else {
                break;
            }
        }

        if ident.is_empty() {
            return Err(RuleEngineError::ParseError {
                message: format!("Expected identifier at position {}", self.position),
            });
        }

        Ok(ident)
    }

    fn try_parse_literal(&mut self) -> Result<Option<Value>> {
        self.skip_whitespace();

        // Boolean literals
        if self.peek_word("true") {
            self.consume_word("true");
            return Ok(Some(Value::Boolean(true)));
        }
        if self.peek_word("false") {
            self.consume_word("false");
            return Ok(Some(Value::Boolean(false)));
        }

        // Null literal
        if self.peek_word("null") {
            self.consume_word("null");
            return Ok(Some(Value::Null));
        }

        // String literals
        if self.peek_char() == Some('"') {
            self.consume_char();
            let mut s = String::new();
            let mut escaped = false;

            while let Some(ch) = self.peek_char() {
                if escaped {
                    // Handle escape sequences
                    let escaped_char = match ch {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        _ => ch,
                    };
                    s.push(escaped_char);
                    escaped = false;
                    self.consume_char();
                } else if ch == '\\' {
                    escaped = true;
                    self.consume_char();
                } else if ch == '"' {
                    self.consume_char();
                    return Ok(Some(Value::String(s)));
                } else {
                    s.push(ch);
                    self.consume_char();
                }
            }

            return Err(RuleEngineError::ParseError {
                message: format!("Unterminated string at position {}", self.position),
            });
        }

        // Number literals
        if let Some(ch) = self.peek_char() {
            if ch.is_numeric() || ch == '-' {
                let start_pos = self.position;
                let mut num_str = String::new();
                let mut has_dot = false;

                while let Some(ch) = self.peek_char() {
                    if ch.is_numeric() {
                        num_str.push(ch);
                        self.consume_char();
                    } else if ch == '.' && !has_dot {
                        has_dot = true;
                        num_str.push(ch);
                        self.consume_char();
                    } else if ch == '-' && num_str.is_empty() {
                        num_str.push(ch);
                        self.consume_char();
                    } else {
                        break;
                    }
                }

                if !num_str.is_empty() && num_str != "-" {
                    if has_dot {
                        if let Ok(n) = num_str.parse::<f64>() {
                            return Ok(Some(Value::Number(n)));
                        }
                    } else if let Ok(i) = num_str.parse::<i64>() {
                        return Ok(Some(Value::Number(i as f64)));
                    }
                }

                // Failed to parse - reset position
                self.position = start_pos;
            }
        }

        Ok(None)
    }

    fn peek_char(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn consume_char(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
        }
    }

    fn peek_operator(&mut self, op: &str) -> bool {
        self.skip_whitespace();
        let remaining: String = self.input[self.position..].iter().collect();
        remaining.starts_with(op)
    }

    fn consume_operator(&mut self, op: &str) {
        self.skip_whitespace();
        for _ in 0..op.len() {
            self.consume_char();
        }
    }

    fn peek_word(&mut self, word: &str) -> bool {
        self.skip_whitespace();
        let remaining: String = self.input[self.position..].iter().collect();

        if remaining.starts_with(word) {
            // Make sure it's a complete word (not prefix)
            let next_pos = self.position + word.len();
            if next_pos >= self.input.len() {
                return true;
            }
            let next_char = self.input[next_pos];
            !next_char.is_alphanumeric() && next_char != '_'
        } else {
            false
        }
    }

    fn consume_word(&mut self, word: &str) {
        self.skip_whitespace();
        if self.peek_word(word) {
            for _ in 0..word.len() {
                self.consume_char();
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.consume_char();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_field() {
        let expr = ExpressionParser::parse("User.IsVIP").unwrap();
        match expr {
            Expression::Field(name) => {
                assert_eq!(name, "User.IsVIP");
            }
            _ => panic!("Expected field expression"),
        }
    }

    #[test]
    fn test_parse_literal_boolean() {
        let expr = ExpressionParser::parse("true").unwrap();
        match expr {
            Expression::Literal(Value::Boolean(true)) => {}
            _ => panic!("Expected boolean literal"),
        }
    }

    #[test]
    fn test_parse_literal_number() {
        let expr = ExpressionParser::parse("42.5").unwrap();
        match expr {
            Expression::Literal(Value::Number(n)) => {
                assert!((n - 42.5).abs() < 0.001);
            }
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_parse_literal_string() {
        let expr = ExpressionParser::parse(r#""hello world""#).unwrap();
        match expr {
            Expression::Literal(Value::String(s)) => {
                assert_eq!(s, "hello world");
            }
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_parse_simple_comparison() {
        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        match expr {
            Expression::Comparison { operator, .. } => {
                assert_eq!(operator, Operator::Equal);
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_all_comparison_operators() {
        let operators = vec![
            ("a == b", Operator::Equal),
            ("a != b", Operator::NotEqual),
            ("a > b", Operator::GreaterThan),
            ("a >= b", Operator::GreaterThanOrEqual),
            ("a < b", Operator::LessThan),
            ("a <= b", Operator::LessThanOrEqual),
        ];

        for (input, expected_op) in operators {
            let expr = ExpressionParser::parse(input).unwrap();
            match expr {
                Expression::Comparison { operator, .. } => {
                    assert_eq!(operator, expected_op, "Failed for: {}", input);
                }
                _ => panic!("Expected comparison for: {}", input),
            }
        }
    }

    #[test]
    fn test_parse_logical_and() {
        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        match expr {
            Expression::And { .. } => {}
            _ => panic!("Expected logical AND, got: {:?}", expr),
        }
    }

    #[test]
    fn test_parse_logical_or() {
        let expr = ExpressionParser::parse("a == true || b == true").unwrap();
        match expr {
            Expression::Or { .. } => {}
            _ => panic!("Expected logical OR"),
        }
    }

    #[test]
    fn test_parse_negation() {
        let expr = ExpressionParser::parse("!User.IsBanned").unwrap();
        match expr {
            Expression::Not(_) => {}
            _ => panic!("Expected negation"),
        }
    }

    #[test]
    fn test_parse_parentheses() {
        let expr = ExpressionParser::parse("(a == true || b == true) && c == true").unwrap();
        match expr {
            Expression::And { left, .. } => {
                match *left {
                    Expression::Or { .. } => {}
                    _ => panic!("Expected OR inside AND"),
                }
            }
            _ => panic!("Expected AND"),
        }
    }

    #[test]
    fn test_parse_variable() {
        let expr = ExpressionParser::parse("?X == true").unwrap();
        match expr {
            Expression::Comparison { left, .. } => {
                match *left {
                    Expression::Variable(var) => {
                        assert_eq!(var, "?X");
                    }
                    _ => panic!("Expected variable"),
                }
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_evaluate_simple() {
        let mut facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));

        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_comparison() {
        let mut facts = Facts::new();
        facts.set("Order.Amount", Value::Number(1500.0));

        let expr = ExpressionParser::parse("Order.Amount > 1000").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_logical_and() {
        let mut facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));
        facts.set("Order.Amount", Value::Number(1500.0));

        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_logical_or() {
        let mut facts = Facts::new();
        facts.set("a", Value::Boolean(false));
        facts.set("b", Value::Boolean(true));

        let expr = ExpressionParser::parse("a == true || b == true").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_is_satisfied() {
        let mut facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));

        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        assert!(expr.is_satisfied(&facts));
    }

    #[test]
    fn test_extract_fields() {
        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        let fields = expr.extract_fields();

        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"User.IsVIP".to_string()));
        assert!(fields.contains(&"Order.Amount".to_string()));
    }

    #[test]
    fn test_parse_error_unclosed_parenthesis() {
        let result = ExpressionParser::parse("(a == true");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_error_unterminated_string() {
        let result = ExpressionParser::parse(r#""hello"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_complex_expression() {
        let expr = ExpressionParser::parse(
            "(User.IsVIP == true && Order.Amount > 1000) || (User.Points >= 100 && Order.Discount < 0.5)"
        ).unwrap();

        // Just check it parses without panicking
        match expr {
            Expression::Or { .. } => {}
            _ => panic!("Expected OR at top level"),
        }
    }

    #[test]
    fn test_to_string() {
        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        let s = expr.to_string();
        assert!(s.contains("User.IsVIP"));
        assert!(s.contains("true"));
    }
}

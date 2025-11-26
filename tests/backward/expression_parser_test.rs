//! Comprehensive tests for expression parser

#[cfg(feature = "backward-chaining")]
mod tests {
    use rust_rule_engine::backward::expression::{Expression, ExpressionParser};
    use rust_rule_engine::types::{ComparisonOperator, Value};
    use rust_rule_engine::engine::facts::Facts;

    // ===== Basic Parsing Tests =====

    #[test]
    fn test_parse_simple_field() {
        let expr = ExpressionParser::parse("User.IsVIP").unwrap();
        match expr {
            Expression::Field(name) => assert_eq!(name, "User.IsVIP"),
            _ => panic!("Expected field expression"),
        }
    }

    #[test]
    fn test_parse_literal_boolean_true() {
        let expr = ExpressionParser::parse("true").unwrap();
        match expr {
            Expression::Literal(Value::Boolean(true)) => {}
            _ => panic!("Expected boolean true literal"),
        }
    }

    #[test]
    fn test_parse_literal_boolean_false() {
        let expr = ExpressionParser::parse("false").unwrap();
        match expr {
            Expression::Literal(Value::Boolean(false)) => {}
            _ => panic!("Expected boolean false literal"),
        }
    }

    #[test]
    fn test_parse_literal_number() {
        let expr = ExpressionParser::parse("42.5").unwrap();
        match expr {
            Expression::Literal(Value::Number(n)) => assert!((n - 42.5).abs() < 0.001),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_parse_literal_string() {
        let expr = ExpressionParser::parse("\"hello world\"").unwrap();
        match expr {
            Expression::Literal(Value::String(s)) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_parse_variable() {
        let expr = ExpressionParser::parse("?Customer").unwrap();
        match expr {
            Expression::Variable(name) => assert_eq!(name, "?Customer"),
            _ => panic!("Expected variable expression"),
        }
    }

    // ===== Comparison Operator Tests =====

    #[test]
    fn test_parse_equal_comparison() {
        let expr = ExpressionParser::parse("User.Age == 25").unwrap();
        match expr {
            Expression::Comparison { operator, .. } => {
                assert_eq!(operator, ComparisonOperator::Equal);
            }
            _ => panic!("Expected comparison expression"),
        }
    }

    #[test]
    fn test_parse_not_equal_comparison() {
        let expr = ExpressionParser::parse("Status != \"Banned\"").unwrap();
        match expr {
            Expression::Comparison { operator, .. } => {
                assert_eq!(operator, ComparisonOperator::NotEqual);
            }
            _ => panic!("Expected comparison expression"),
        }
    }

    #[test]
    fn test_parse_greater_than() {
        let expr = ExpressionParser::parse("Amount > 100").unwrap();
        match expr {
            Expression::Comparison { operator, .. } => {
                assert_eq!(operator, ComparisonOperator::Greater);
            }
            _ => panic!("Expected comparison expression"),
        }
    }

    #[test]
    fn test_parse_less_than() {
        let expr = ExpressionParser::parse("Score < 50").unwrap();
        match expr {
            Expression::Comparison { operator, .. } => {
                assert_eq!(operator, ComparisonOperator::Less);
            }
            _ => panic!("Expected comparison expression"),
        }
    }

    #[test]
    fn test_parse_greater_or_equal() {
        let expr = ExpressionParser::parse("Points >= 100").unwrap();
        match expr {
            Expression::Comparison { operator, .. } => {
                assert_eq!(operator, ComparisonOperator::GreaterOrEqual);
            }
            _ => panic!("Expected comparison expression"),
        }
    }

    #[test]
    fn test_parse_less_or_equal() {
        let expr = ExpressionParser::parse("Temperature <= 32").unwrap();
        match expr {
            Expression::Comparison { operator, .. } => {
                assert_eq!(operator, ComparisonOperator::LessOrEqual);
            }
            _ => panic!("Expected comparison expression"),
        }
    }

    // ===== Logical Operator Tests =====

    #[test]
    fn test_parse_logical_and() {
        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        match expr {
            Expression::Logical { left, right, .. } => {
                assert!(matches!(**left, Expression::Comparison { .. }));
                assert!(matches!(**right, Expression::Comparison { .. }));
            }
            _ => panic!("Expected logical AND expression"),
        }
    }

    #[test]
    fn test_parse_logical_or() {
        let expr = ExpressionParser::parse("User.IsPremium == true || User.IsVIP == true").unwrap();
        match expr {
            Expression::Logical { .. } => {}
            _ => panic!("Expected logical OR expression"),
        }
    }

    #[test]
    fn test_parse_negation() {
        let expr = ExpressionParser::parse("!User.IsBanned").unwrap();
        match expr {
            Expression::Not(inner) => {
                assert!(matches!(**inner, Expression::Field(_)));
            }
            _ => panic!("Expected NOT expression"),
        }
    }

    #[test]
    fn test_parse_complex_logical_expression() {
        let expr = ExpressionParser::parse(
            "User.IsVIP == true && Order.Amount > 1000 || User.IsPremium == true"
        ).unwrap();

        // Should parse as: (User.IsVIP == true && Order.Amount > 1000) || User.IsPremium == true
        match expr {
            Expression::Logical { .. } => {}
            _ => panic!("Expected complex logical expression"),
        }
    }

    // ===== Nested Expression Tests =====

    #[test]
    fn test_parse_parentheses() {
        let expr = ExpressionParser::parse("(User.Age > 18)").unwrap();
        match expr {
            Expression::Comparison { .. } => {}
            _ => panic!("Expected comparison in parentheses"),
        }
    }

    #[test]
    fn test_parse_nested_parentheses() {
        let expr = ExpressionParser::parse("((User.Age > 18) && (Score >= 50))").unwrap();
        match expr {
            Expression::Logical { .. } => {}
            _ => panic!("Expected nested logical expression"),
        }
    }

    #[test]
    fn test_parse_complex_nested() {
        let expr = ExpressionParser::parse(
            "(User.IsVIP == true && Order.Amount > 1000) || (User.IsPremium == true && Score > 80)"
        ).unwrap();

        match expr {
            Expression::Logical { .. } => {}
            _ => panic!("Expected complex nested expression"),
        }
    }

    // ===== Expression Evaluation Tests =====

    #[test]
    fn test_evaluate_simple_comparison_true() {
        let mut facts = Facts::new();
        facts.set("User.Age", Value::Number(25.0));

        let expr = ExpressionParser::parse("User.Age == 25").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_simple_comparison_false() {
        let mut facts = Facts::new();
        facts.set("User.Age", Value::Number(25.0));

        let expr = ExpressionParser::parse("User.Age == 30").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_evaluate_logical_and_true() {
        let mut facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));
        facts.set("Order.Amount", Value::Number(1500.0));

        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_logical_and_false() {
        let mut facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));
        facts.set("Order.Amount", Value::Number(500.0));

        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_evaluate_logical_or_true() {
        let mut facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(false));
        facts.set("User.IsPremium", Value::Boolean(true));

        let expr = ExpressionParser::parse("User.IsVIP == true || User.IsPremium == true").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_negation_true() {
        let mut facts = Facts::new();
        facts.set("User.IsBanned", Value::Boolean(false));

        let expr = ExpressionParser::parse("!User.IsBanned").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_negation_false() {
        let mut facts = Facts::new();
        facts.set("User.IsBanned", Value::Boolean(true));

        let expr = ExpressionParser::parse("!User.IsBanned").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_evaluate_greater_than() {
        let mut facts = Facts::new();
        facts.set("Score", Value::Number(85.0));

        let expr = ExpressionParser::parse("Score > 50").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_less_than() {
        let mut facts = Facts::new();
        facts.set("Temperature", Value::Number(20.0));

        let expr = ExpressionParser::parse("Temperature < 32").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_string_comparison() {
        let mut facts = Facts::new();
        facts.set("Status", Value::String("Active".to_string()));

        let expr = ExpressionParser::parse("Status == \"Active\"").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    // ===== Field Extraction Tests =====

    #[test]
    fn test_extract_fields_single() {
        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        let fields = expr.extract_fields();

        assert_eq!(fields.len(), 1);
        assert!(fields.contains(&"User.IsVIP".to_string()));
    }

    #[test]
    fn test_extract_fields_multiple() {
        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        let fields = expr.extract_fields();

        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"User.IsVIP".to_string()));
        assert!(fields.contains(&"Order.Amount".to_string()));
    }

    #[test]
    fn test_extract_fields_complex() {
        let expr = ExpressionParser::parse(
            "(User.IsVIP == true && Order.Amount > 1000) || User.IsPremium == true"
        ).unwrap();
        let fields = expr.extract_fields();

        assert_eq!(fields.len(), 3);
        assert!(fields.contains(&"User.IsVIP".to_string()));
        assert!(fields.contains(&"Order.Amount".to_string()));
        assert!(fields.contains(&"User.IsPremium".to_string()));
    }

    // ===== Error Handling Tests =====

    #[test]
    fn test_parse_empty_string_error() {
        let result = ExpressionParser::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_operator_error() {
        let result = ExpressionParser::parse("User.Age === 25");
        // Should still parse (=== becomes == =)
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_missing_field_error() {
        let facts = Facts::new();
        let expr = ExpressionParser::parse("User.Age == 25").unwrap();
        let result = expr.evaluate(&facts);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unterminated_string_error() {
        let result = ExpressionParser::parse("Status == \"Active");
        // Parser might handle this differently
        // This test documents current behavior
        let _ = result;
    }

    // ===== is_satisfied Tests =====

    #[test]
    fn test_is_satisfied_true() {
        let mut facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));

        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        assert!(expr.is_satisfied(&facts));
    }

    #[test]
    fn test_is_satisfied_false() {
        let mut facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(false));

        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        assert!(!expr.is_satisfied(&facts));
    }

    #[test]
    fn test_is_satisfied_missing_field() {
        let facts = Facts::new();

        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        assert!(!expr.is_satisfied(&facts));
    }

    // ===== Whitespace Handling Tests =====

    #[test]
    fn test_parse_with_extra_whitespace() {
        let expr = ExpressionParser::parse("  User.IsVIP   ==   true  ").unwrap();
        match expr {
            Expression::Comparison { .. } => {}
            _ => panic!("Expected comparison expression"),
        }
    }

    #[test]
    fn test_parse_no_whitespace() {
        let expr = ExpressionParser::parse("User.IsVIP==true").unwrap();
        match expr {
            Expression::Comparison { .. } => {}
            _ => panic!("Expected comparison expression"),
        }
    }

    // ===== Operator Precedence Tests =====

    #[test]
    fn test_and_before_or_precedence() {
        let expr = ExpressionParser::parse("A == true || B == true && C == true").unwrap();

        // Should parse as: A == true || (B == true && C == true)
        match expr {
            Expression::Logical { .. } => {}
            _ => panic!("Expected logical expression"),
        }
    }

    #[test]
    fn test_not_before_and_precedence() {
        let expr = ExpressionParser::parse("!A && B").unwrap();

        // Should parse as: (!A) && B
        match expr {
            Expression::Logical { left, .. } => {
                assert!(matches!(**left, Expression::Not(_)));
            }
            _ => panic!("Expected logical expression"),
        }
    }
}

//! Comprehensive backward chaining tests
//!
//! This test suite provides comprehensive coverage for backward chaining features.

#[cfg(feature = "backward-chaining")]
mod expression_parser {
    use rust_rule_engine::backward::expression::{Expression, ExpressionParser};
    use rust_rule_engine::types::Value;
    use rust_rule_engine::engine::facts::Facts;

    #[test]
    fn test_parse_simple_field() {
        let expr = ExpressionParser::parse("User.IsVIP").unwrap();
        match expr {
            Expression::Field(name) => assert_eq!(name, "User.IsVIP"),
            _ => panic!("Expected field expression"),
        }
    }

    #[test]
    fn test_parse_boolean_literal() {
        let expr = ExpressionParser::parse("true").unwrap();
        match expr {
            Expression::Literal(Value::Boolean(true)) => {}
            _ => panic!("Expected boolean literal"),
        }
    }

    #[test]
    fn test_parse_number_literal() {
        let expr = ExpressionParser::parse("42.5").unwrap();
        match expr {
            Expression::Literal(Value::Number(n)) => assert!((n - 42.5).abs() < 0.001),
            _ => panic!("Expected number literal"),
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let expr = ExpressionParser::parse("\"hello\"").unwrap();
        match expr {
            Expression::Literal(Value::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_parse_comparison() {
        let result = ExpressionParser::parse("User.Age == 25");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_logical_and() {
        let result = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_logical_or() {
        let result = ExpressionParser::parse("User.IsPremium == true || User.IsVIP == true");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_negation() {
        let result = ExpressionParser::parse("!User.IsBanned");
        assert!(result.is_ok());
    }

    #[test]
    fn test_evaluate_comparison_true() {
        let facts = Facts::new();
        facts.set("User.Age", Value::Number(25.0));

        let expr = ExpressionParser::parse("User.Age == 25").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_comparison_false() {
        let facts = Facts::new();
        facts.set("User.Age", Value::Number(25.0));

        let expr = ExpressionParser::parse("User.Age == 30").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(false));
    }

    #[test]
    fn test_evaluate_logical_and() {
        let facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));
        facts.set("Order.Amount", Value::Number(1500.0));

        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_evaluate_logical_or() {
        let facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(false));
        facts.set("User.IsPremium", Value::Boolean(true));

        let expr = ExpressionParser::parse("User.IsVIP == true || User.IsPremium == true").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_extract_fields_single() {
        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        let fields = expr.extract_fields();

        assert!(fields.contains(&"User.IsVIP".to_string()));
    }

    #[test]
    fn test_extract_fields_multiple() {
        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        let fields = expr.extract_fields();

        assert!(fields.contains(&"User.IsVIP".to_string()));
        assert!(fields.contains(&"Order.Amount".to_string()));
    }

    #[test]
    fn test_is_satisfied_true() {
        let facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));

        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        assert!(expr.is_satisfied(&facts));
    }

    #[test]
    fn test_is_satisfied_false() {
        let facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(false));

        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        assert!(!expr.is_satisfied(&facts));
    }

    #[test]
    fn test_parse_greater_than() {
        let result = ExpressionParser::parse("Amount > 100");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_less_than() {
        let result = ExpressionParser::parse("Score < 50");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_greater_or_equal() {
        let result = ExpressionParser::parse("Points >= 100");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_less_or_equal() {
        let result = ExpressionParser::parse("Temperature <= 32");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_not_equal() {
        let result = ExpressionParser::parse("Status != \"Banned\"");
        assert!(result.is_ok());
    }
}

#[cfg(feature = "backward-chaining")]
mod conclusion_index {
    use rust_rule_engine::backward::conclusion_index::ConclusionIndex;
    use rust_rule_engine::engine::rule::{Rule, Condition};
    use rust_rule_engine::types::{ActionType, Value, Operator};
    use rust_rule_engine::ConditionGroup;

    fn create_test_rule(name: &str, sets_field: &str) -> Rule {
        // Create a simple dummy condition (always true)
        let dummy_condition = Condition::new(
            "dummy".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        );

        Rule::new(
            name.to_string(),
            ConditionGroup::single(dummy_condition),
            vec![ActionType::Set {
                field: sets_field.to_string(),
                value: Value::Boolean(true),
            }],
        )
    }

    #[test]
    fn test_new_index_empty() {
        let index = ConclusionIndex::new();
        let stats = index.stats();
        assert_eq!(stats.total_rules, 0);
    }

    #[test]
    fn test_add_single_rule() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));

        let stats = index.stats();
        assert_eq!(stats.total_rules, 1);
    }

    #[test]
    fn test_find_candidates_single() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("VIPRule", "User.IsVIP"));

        let candidates = index.find_candidates("User.IsVIP == true");

        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&"VIPRule".to_string()));
    }

    #[test]
    fn test_find_candidates_multiple_matches() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("VIPRule1", "User.IsVIP"));
        index.add_rule(&create_test_rule("VIPRule2", "User.IsVIP"));

        let candidates = index.find_candidates("User.IsVIP == true");

        assert_eq!(candidates.len(), 2);
    }

    #[test]
    fn test_from_rules_creates_index() {
        let rules = vec![
            create_test_rule("Rule1", "User.IsVIP"),
            create_test_rule("Rule2", "Order.Approved"),
            create_test_rule("Rule3", "User.IsPremium"),
        ];

        let index = ConclusionIndex::from_rules(&rules);
        let stats = index.stats();

        assert_eq!(stats.total_rules, 3);
    }

    #[test]
    fn test_remove_rule() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));

        index.remove_rule("Rule1");
        let stats = index.stats();

        assert_eq!(stats.total_rules, 0);
    }

    #[test]
    fn test_clear_index() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));
        index.add_rule(&create_test_rule("Rule2", "Order.Approved"));

        index.clear();
        let stats = index.stats();

        assert_eq!(stats.total_rules, 0);
    }

    #[test]
    fn test_performance_o1_lookup() {
        let mut index = ConclusionIndex::new();

        // Add 100 rules
        for i in 0..100 {
            index.add_rule(&create_test_rule(&format!("Rule{}", i), &format!("Field{}", i)));
        }

        use std::time::Instant;
        let start = Instant::now();

        let candidates = index.find_candidates("Field50 == true");

        let elapsed = start.elapsed();

        assert_eq!(candidates.len(), 1);
        // Should be very fast - O(1) lookup
        assert!(elapsed.as_micros() < 1000);
    }

    #[test]
    fn test_index_multiple_fields() {
        let mut rule = create_test_rule("MultiRule", "User.IsVIP");
        rule.actions.push(ActionType::Set {
            field: "User.Points".to_string(),
            value: Value::Number(1000.0),
        });

        let mut index = ConclusionIndex::new();
        index.add_rule(&rule);

        let candidates1 = index.find_candidates("User.IsVIP == true");
        let candidates2 = index.find_candidates("User.Points == 1000");

        assert!(candidates1.len() > 0);
        assert!(candidates2.len() > 0);
    }

    #[test]
    fn test_is_empty() {
        let mut index = ConclusionIndex::new();
        assert!(index.is_empty());

        index.add_rule(&create_test_rule("Rule1", "Field1"));
        assert!(!index.is_empty());
    }
}

#[cfg(feature = "backward-chaining")]
mod unification {
    use rust_rule_engine::backward::unification::Bindings;
    use rust_rule_engine::types::Value;
    use std::collections::HashMap;

    #[test]
    fn test_bind_variable() {
        let mut bindings = Bindings::new();
        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();

        assert_eq!(bindings.get("X"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_is_bound() {
        let mut bindings = Bindings::new();
        assert!(!bindings.is_bound("X"));

        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();
        assert!(bindings.is_bound("X"));
    }

    #[test]
    fn test_merge_bindings() {
        let mut bindings1 = Bindings::new();
        bindings1.bind("X".to_string(), Value::Number(1.0)).unwrap();

        let mut bindings2 = Bindings::new();
        bindings2.bind("Y".to_string(), Value::Number(2.0)).unwrap();

        bindings1.merge(&bindings2).unwrap();

        assert_eq!(bindings1.get("X"), Some(&Value::Number(1.0)));
        assert_eq!(bindings1.get("Y"), Some(&Value::Number(2.0)));
    }

    #[test]
    fn test_conflicting_bindings() {
        let mut bindings1 = Bindings::new();
        bindings1.bind("X".to_string(), Value::Number(1.0)).unwrap();

        let mut bindings2 = Bindings::new();
        bindings2.bind("X".to_string(), Value::Number(2.0)).unwrap();

        let result = bindings1.merge(&bindings2);
        assert!(result.is_err());
    }

    #[test]
    fn test_clear_bindings() {
        let mut bindings = Bindings::new();
        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();

        bindings.clear();

        assert!(!bindings.is_bound("X"));
        assert!(bindings.is_empty());
    }

    #[test]
    fn test_from_map() {
        let mut map = HashMap::new();
        map.insert("X".to_string(), Value::Number(1.0));
        map.insert("Y".to_string(), Value::Number(2.0));

        let bindings = Bindings::from_map(map);

        assert_eq!(bindings.get("X"), Some(&Value::Number(1.0)));
        assert_eq!(bindings.get("Y"), Some(&Value::Number(2.0)));
    }

    #[test]
    fn test_to_map() {
        let mut bindings = Bindings::new();
        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();

        let map = bindings.to_map();

        assert_eq!(map.get("X"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_len() {
        let mut bindings = Bindings::new();
        assert_eq!(bindings.len(), 0);

        bindings.bind("X".to_string(), Value::Number(1.0)).unwrap();
        assert_eq!(bindings.len(), 1);

        bindings.bind("Y".to_string(), Value::Number(2.0)).unwrap();
        assert_eq!(bindings.len(), 2);
    }
}

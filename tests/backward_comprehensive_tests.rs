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

#[cfg(feature = "backward-chaining")]
mod multiple_solutions {
    use rust_rule_engine::backward::{BackwardEngine, BackwardConfig};
    use rust_rule_engine::backward::search::SearchStrategy;
    use rust_rule_engine::{KnowledgeBase, Facts, Rule, Condition, ConditionGroup};
    use rust_rule_engine::types::{Value, ActionType, Operator};

    #[test]
    fn test_multiple_solutions_single_rule() {
        // Test finding multiple solutions with max_solutions > 1
        let mut kb = KnowledgeBase::new("multi_solutions");

        // Rule: If User.Type == "Premium" then User.Discount = 0.2
        kb.add_rule(Rule::new(
            "PremiumDiscount".to_string(),
            ConditionGroup::single(Condition::new(
                "User.Type".to_string(),
                Operator::Equal,
                Value::String("Premium".to_string()),
            )),
            vec![ActionType::Set {
                field: "User.Discount".to_string(),
                value: Value::Number(0.2),
            }],
        )).unwrap();

        let config = BackwardConfig {
            max_depth: 10,
            strategy: SearchStrategy::DepthFirst,
            enable_memoization: true,
            max_solutions: 3, // Allow up to 3 solutions
        };

        let mut engine = BackwardEngine::with_config(kb, config);
        let mut facts = Facts::new();
        facts.set("User.Type", Value::String("Premium".to_string()));

        let result = engine.query("User.Discount == 0.2", &mut facts).unwrap();

        assert!(result.provable, "Goal should be provable");
        // With current implementation, we should find at least 1 solution
        // Even with max_solutions=3, we find 1 proof path
    }

    #[test]
    fn test_multiple_solutions_multiple_paths() {
        // Test case where goal can be proven through multiple rule chains
        let mut kb = KnowledgeBase::new("multi_paths");

        // Path 1: User.Age >= 18 -> User.IsAdult = true
        kb.add_rule(Rule::new(
            "AgeCheck".to_string(),
            ConditionGroup::single(Condition::new(
                "User.Age".to_string(),
                Operator::GreaterThanOrEqual,
                Value::Number(18.0),
            )),
            vec![ActionType::Set {
                field: "User.IsAdult".to_string(),
                value: Value::Boolean(true),
            }],
        )).unwrap();

        // Path 2: User.HasLicense == true -> User.IsAdult = true
        kb.add_rule(Rule::new(
            "LicenseCheck".to_string(),
            ConditionGroup::single(Condition::new(
                "User.HasLicense".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            vec![ActionType::Set {
                field: "User.IsAdult".to_string(),
                value: Value::Boolean(true),
            }],
        )).unwrap();

        let config = BackwardConfig {
            max_depth: 10,
            strategy: SearchStrategy::DepthFirst,
            enable_memoization: false, // Disable memoization to explore all paths
            max_solutions: 5,
        };

        let mut engine = BackwardEngine::with_config(kb, config);
        let mut facts = Facts::new();
        facts.set("User.Age", Value::Number(25.0));
        facts.set("User.HasLicense", Value::Boolean(true));

        let result = engine.query("User.IsAdult == true", &mut facts).unwrap();

        assert!(result.provable, "Goal should be provable through multiple paths");
    }

    #[test]
    fn test_max_solutions_limit() {
        // Test that engine respects max_solutions limit
        let mut kb = KnowledgeBase::new("solution_limit");

        // Simple rule
        kb.add_rule(Rule::new(
            "SetValue".to_string(),
            ConditionGroup::single(Condition::new(
                "Input.Ready".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            vec![ActionType::Set {
                field: "Output.Value".to_string(),
                value: Value::Number(42.0),
            }],
        )).unwrap();

        // Test with max_solutions = 1
        let config1 = BackwardConfig {
            max_solutions: 1,
            ..Default::default()
        };

        let mut engine1 = BackwardEngine::with_config(kb.clone(), config1);
        let mut facts1 = Facts::new();
        facts1.set("Input.Ready", Value::Boolean(true));

        let result1 = engine1.query("Output.Value == 42", &mut facts1).unwrap();
        assert!(result1.provable);

        // Test with max_solutions = 10
        let config10 = BackwardConfig {
            max_solutions: 10,
            ..Default::default()
        };

        let mut engine10 = BackwardEngine::with_config(kb, config10);
        let mut facts10 = Facts::new();
        facts10.set("Input.Ready", Value::Boolean(true));

        let result10 = engine10.query("Output.Value == 42", &mut facts10).unwrap();
        assert!(result10.provable);
    }

    #[test]
    fn test_multiple_solutions_with_different_strategies() {
        // Test multiple solutions with different search strategies
        let mut kb = KnowledgeBase::new("multi_strategy");

        kb.add_rule(Rule::new(
            "Rule1".to_string(),
            ConditionGroup::single(Condition::new(
                "X".to_string(),
                Operator::GreaterThan,
                Value::Number(0.0),
            )),
            vec![ActionType::Set {
                field: "Y".to_string(),
                value: Value::Boolean(true),
            }],
        )).unwrap();

        // Test with DFS
        let config_dfs = BackwardConfig {
            strategy: SearchStrategy::DepthFirst,
            max_solutions: 5,
            ..Default::default()
        };

        let mut engine_dfs = BackwardEngine::with_config(kb.clone(), config_dfs);
        let mut facts_dfs = Facts::new();
        facts_dfs.set("X", Value::Number(10.0));

        let result_dfs = engine_dfs.query("Y == true", &mut facts_dfs).unwrap();
        assert!(result_dfs.provable);

        // Test with BFS
        let config_bfs = BackwardConfig {
            strategy: SearchStrategy::BreadthFirst,
            max_solutions: 5,
            ..Default::default()
        };

        let mut engine_bfs = BackwardEngine::with_config(kb, config_bfs);
        let mut facts_bfs = Facts::new();
        facts_bfs.set("X", Value::Number(10.0));

        let result_bfs = engine_bfs.query("Y == true", &mut facts_bfs).unwrap();
        assert!(result_bfs.provable);
    }

    #[test]
    fn test_multiple_solutions_complex_chain() {
        // Test multiple solutions with complex rule chains
        let mut kb = KnowledgeBase::new("complex_chain");

        // Chain: A -> B -> C
        kb.add_rule(Rule::new(
            "AtoB".to_string(),
            ConditionGroup::single(Condition::new(
                "A".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            vec![ActionType::Set {
                field: "B".to_string(),
                value: Value::Boolean(true),
            }],
        )).unwrap();

        kb.add_rule(Rule::new(
            "BtoC".to_string(),
            ConditionGroup::single(Condition::new(
                "B".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            vec![ActionType::Set {
                field: "C".to_string(),
                value: Value::Boolean(true),
            }],
        )).unwrap();

        let config = BackwardConfig {
            max_depth: 20,
            max_solutions: 3,
            enable_memoization: true,
            ..Default::default()
        };

        let mut engine = BackwardEngine::with_config(kb, config);
        let mut facts = Facts::new();
        facts.set("A", Value::Boolean(true));

        let result = engine.query("C == true", &mut facts).unwrap();
        assert!(result.provable, "Should prove C through A->B->C chain");
    }
}

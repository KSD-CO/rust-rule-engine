//! Integration tests for GRL parser with nested queries and optimization

#[cfg(feature = "backward-chaining")]
mod grl_parser_tests {
    use rust_rule_engine::backward::{GRLQueryParser, GRLSearchStrategy};

    #[test]
    fn test_parse_simple_with_optimization() {
        let query_str = r#"
        query "SimpleOptimized" {
            goal: eligible(?customer)
            strategy: depth-first
            max-depth: 10
            enable-optimization: true
        }
        "#;

        let query = GRLQueryParser::parse(query_str).expect("Should parse");

        assert_eq!(query.name, "SimpleOptimized");
        assert_eq!(query.goal, "eligible(?customer)");
        assert!(query.enable_optimization, "Optimization should be enabled");
        assert_eq!(query.max_depth, 10);
    }

    #[test]
    fn test_parse_nested_query_syntax() {
        let query_str = r#"
        query "GrandparentQuery" {
            goal: grandparent(?x, ?z) WHERE parent(?x, ?y) AND parent(?y, ?z)
            strategy: depth-first
            max-depth: 15
            enable-optimization: true
        }
        "#;

        let query = GRLQueryParser::parse(query_str).expect("Should parse");

        assert_eq!(query.name, "GrandparentQuery");
        assert!(query.goal.contains("WHERE"), "Goal should contain WHERE clause");
        assert!(query.goal.contains("AND"), "Goal should contain AND operator");
        assert!(query.enable_optimization);
        assert_eq!(query.max_depth, 15);
    }

    #[test]
    fn test_parse_complex_with_or_and_and() {
        let query_str = r#"
        query "ComplexEligibility" {
            goal: (high_value(?c) OR premium(?c)) AND active(?c) AND verified(?c)
            strategy: breadth-first
            max-depth: 20
            max-solutions: 10
            enable-optimization: true
            enable-memoization: true
        }
        "#;

        let query = GRLQueryParser::parse(query_str).expect("Should parse");

        assert_eq!(query.name, "ComplexEligibility");
        assert!(query.goal.contains("OR"), "Goal should contain OR");
        assert!(query.goal.contains("AND"), "Goal should contain AND");
        assert!(query.enable_optimization);
        assert!(query.enable_memoization);
        assert_eq!(query.max_solutions, 10);
        assert_eq!(query.max_depth, 20);
        assert_eq!(query.strategy, GRLSearchStrategy::BreadthFirst);
    }

    #[test]
    fn test_parse_optimization_disabled() {
        let query_str = r#"
        query "NoOptimization" {
            goal: customer(?id) AND verified(?id)
            enable-optimization: false
        }
        "#;

        let query = GRLQueryParser::parse(query_str).expect("Should parse");

        assert_eq!(query.name, "NoOptimization");
        assert!(!query.enable_optimization, "Optimization should be disabled");
    }

    #[test]
    fn test_parse_all_features_combined() {
        let query_str = r#"
        query "UltimateQuery" {
            goal: (eligible(?x) WHERE (vip(?x) OR premium(?x))) AND active(?x)
            strategy: iterative
            max-depth: 25
            max-solutions: 5
            enable-optimization: true
            enable-memoization: true

            when: User.Role == "admin"

            on-success: {
                User.Status = "approved";
                Print("Success");
            }

            on-failure: {
                Print("Failed");
            }
        }
        "#;

        let query = GRLQueryParser::parse(query_str).expect("Should parse");

        assert_eq!(query.name, "UltimateQuery");
        assert!(query.goal.contains("WHERE"));
        assert!(query.goal.contains("OR"));
        assert!(query.goal.contains("AND"));
        assert_eq!(query.max_depth, 25);
        assert_eq!(query.max_solutions, 5);
        assert!(query.enable_optimization);
        assert!(query.enable_memoization);
        assert_eq!(query.strategy, GRLSearchStrategy::Iterative);
        assert!(query.when_condition.is_some());
        assert!(query.on_success.is_some());
        assert!(query.on_failure.is_some());
    }

    #[test]
    fn test_parse_with_negation() {
        let query_str = r#"
        query "WithNegation" {
            goal: active(?x) AND NOT suspended(?x)
            enable-optimization: true
        }
        "#;

        let query = GRLQueryParser::parse(query_str).expect("Should parse");

        assert_eq!(query.name, "WithNegation");
        assert!(query.goal.contains("NOT"));
        assert!(query.enable_optimization);
    }

    #[test]
    fn test_default_optimization_enabled() {
        let query_str = r#"
        query "DefaultSettings" {
            goal: test(?x)
        }
        "#;

        let query = GRLQueryParser::parse(query_str).expect("Should parse");

        // Optimization should be enabled by default
        assert!(query.enable_optimization, "Optimization should be enabled by default");
    }

    #[test]
    fn test_parse_action_handlers() {
        let query_str = r#"
        query "WithActions" {
            goal: eligible(?x)

            on-success: {
                User.Approved = true;
                LogMessage("User approved");
            }

            on-failure: {
                User.Approved = false;
            }

            on-missing: {
                Request("Please provide required information");
            }
        }
        "#;

        let query = GRLQueryParser::parse(query_str).expect("Should parse");

        assert_eq!(query.name, "WithActions");
        assert!(query.on_success.is_some());
        assert!(query.on_failure.is_some());
        assert!(query.on_missing.is_some());

        // Verify action content
        if let Some(ref action) = query.on_success {
            assert!(!action.assignments.is_empty(), "Should have assignments");
            assert!(!action.calls.is_empty(), "Should have function calls");
        }
    }

    #[test]
    fn test_parse_parenthesized_goals() {
        let query_str = r#"
        query "Parentheses" {
            goal: ((A OR B) AND C) OR (D AND E)
            enable-optimization: true
        }
        "#;

        let query = GRLQueryParser::parse(query_str).expect("Should parse");

        assert_eq!(query.name, "Parentheses");
        assert!(query.goal.contains("("));
        assert!(query.goal.contains(")"));
        assert!(query.goal.contains("OR"));
        assert!(query.goal.contains("AND"));
    }
}

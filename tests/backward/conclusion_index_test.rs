//! Comprehensive tests for ConclusionIndex

#[cfg(feature = "backward-chaining")]
mod tests {
    use rust_rule_engine::backward::conclusion_index::ConclusionIndex;
    use rust_rule_engine::engine::rule::Rule;
    use rust_rule_engine::types::{ActionType, Condition, Value};

    fn create_test_rule(name: &str, sets_field: &str) -> Rule {
        Rule {
            name: name.to_string(),
            conditions: vec![],
            actions: vec![ActionType::Set {
                field: sets_field.to_string(),
                value: Value::Boolean(true),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn test_new_index_empty() {
        let index = ConclusionIndex::new();
        let stats = index.stats();

        assert_eq!(stats.total_rules, 0);
        assert_eq!(stats.indexed_fields, 0);
    }

    #[test]
    fn test_add_single_rule() {
        let mut index = ConclusionIndex::new();
        let rule = create_test_rule("Rule1", "User.IsVIP");

        index.add_rule(&rule);
        let stats = index.stats();

        assert_eq!(stats.total_rules, 1);
        assert!(stats.indexed_fields >= 1);
    }

    #[test]
    fn test_add_multiple_rules() {
        let mut index = ConclusionIndex::new();

        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));
        index.add_rule(&create_test_rule("Rule2", "Order.Approved"));
        index.add_rule(&create_test_rule("Rule3", "Payment.Valid"));

        let stats = index.stats();
        assert_eq!(stats.total_rules, 3);
    }

    #[test]
    fn test_find_candidate_rules_single_field() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("VIPRule", "User.IsVIP"));

        let candidates = index.find_candidate_rules(&vec!["User.IsVIP".to_string()]);

        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&"VIPRule".to_string()));
    }

    #[test]
    fn test_find_candidate_rules_multiple_matches() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("VIPRule1", "User.IsVIP"));
        index.add_rule(&create_test_rule("VIPRule2", "User.IsVIP"));

        let candidates = index.find_candidate_rules(&vec!["User.IsVIP".to_string()]);

        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&"VIPRule1".to_string()));
        assert!(candidates.contains(&"VIPRule2".to_string()));
    }

    #[test]
    fn test_find_candidate_rules_no_match() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));

        let candidates = index.find_candidate_rules(&vec!["Order.Approved".to_string()]);

        assert_eq!(candidates.len(), 0);
    }

    #[test]
    fn test_find_candidate_rules_nested_field() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "Order.Customer.IsVIP"));

        let candidates = index.find_candidate_rules(&vec!["Order.Customer.IsVIP".to_string()]);

        assert!(candidates.len() >= 1);
        assert!(candidates.contains(&"Rule1".to_string()));
    }

    #[test]
    fn test_find_candidate_rules_base_object() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "Order.Status"));

        // Should match both "Order.Status" and base "Order"
        let candidates = index.find_candidate_rules(&vec!["Order.Status".to_string()]);

        assert!(candidates.contains(&"Rule1".to_string()));
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
        assert!(stats.indexed_fields >= 3);
    }

    #[test]
    fn test_remove_rule() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));
        index.add_rule(&create_test_rule("Rule2", "Order.Approved"));

        index.remove_rule("Rule1");
        let stats = index.stats();

        assert_eq!(stats.total_rules, 1);

        let candidates = index.find_candidate_rules(&vec!["User.IsVIP".to_string()]);
        assert_eq!(candidates.len(), 0);
    }

    #[test]
    fn test_clear_index() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));
        index.add_rule(&create_test_rule("Rule2", "Order.Approved"));

        index.clear();
        let stats = index.stats();

        assert_eq!(stats.total_rules, 0);
        assert_eq!(stats.indexed_fields, 0);
    }

    #[test]
    fn test_index_multiple_actions_per_rule() {
        let mut rule = create_test_rule("MultiActionRule", "User.IsVIP");
        rule.actions.push(ActionType::Set {
            field: "User.Points".to_string(),
            value: Value::Number(1000.0),
        });
        rule.actions.push(ActionType::Set {
            field: "User.Tier".to_string(),
            value: Value::String("Gold".to_string()),
        });

        let mut index = ConclusionIndex::new();
        index.add_rule(&rule);

        // Should find rule by any of its conclusions
        assert!(index.find_candidate_rules(&vec!["User.IsVIP".to_string()]).len() > 0);
        assert!(index.find_candidate_rules(&vec!["User.Points".to_string()]).len() > 0);
        assert!(index.find_candidate_rules(&vec!["User.Tier".to_string()]).len() > 0);
    }

    #[test]
    fn test_index_statistics() {
        let rules = vec![
            create_test_rule("Rule1", "Field1"),
            create_test_rule("Rule2", "Field1"), // Same field
            create_test_rule("Rule3", "Field2"),
            create_test_rule("Rule4", "Field3"),
        ];

        let index = ConclusionIndex::from_rules(&rules);
        let stats = index.stats();

        assert_eq!(stats.total_rules, 4);
        assert!(stats.avg_rules_per_field >= 1.0);
    }

    #[test]
    fn test_duplicate_rule_names() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "Field1"));
        index.add_rule(&create_test_rule("Rule1", "Field2")); // Same name, different field

        // Latest should override
        let stats = index.stats();
        assert_eq!(stats.total_rules, 1);
    }

    #[test]
    fn test_find_by_multiple_fields() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "Field1"));
        index.add_rule(&create_test_rule("Rule2", "Field2"));
        index.add_rule(&create_test_rule("Rule3", "Field3"));

        let candidates = index.find_candidate_rules(&vec![
            "Field1".to_string(),
            "Field2".to_string(),
        ]);

        // Should find both Rule1 and Rule2
        assert!(candidates.len() >= 2);
        assert!(candidates.contains(&"Rule1".to_string()));
        assert!(candidates.contains(&"Rule2".to_string()));
    }

    #[test]
    fn test_performance_with_many_rules() {
        let mut index = ConclusionIndex::new();

        // Add 100 rules
        for i in 0..100 {
            let rule = create_test_rule(&format!("Rule{}", i), &format!("Field{}", i));
            index.add_rule(&rule);
        }

        let stats = index.stats();
        assert_eq!(stats.total_rules, 100);

        // Lookup should be O(1) - test that it completes quickly
        use std::time::Instant;
        let start = Instant::now();

        let candidates = index.find_candidate_rules(&vec!["Field50".to_string()]);

        let elapsed = start.elapsed();

        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains(&"Rule50".to_string()));

        // Should complete in microseconds, not milliseconds
        assert!(elapsed.as_micros() < 1000, "Lookup took too long: {:?}", elapsed);
    }

    #[test]
    fn test_index_with_method_call_actions() {
        let mut rule = Rule {
            name: "MethodCallRule".to_string(),
            conditions: vec![],
            actions: vec![ActionType::MethodCall {
                object: "Logger".to_string(),
                method: "log".to_string(),
                args: vec![],
            }],
            ..Default::default()
        };

        let mut index = ConclusionIndex::new();
        index.add_rule(&rule);

        // Should index the object
        let candidates = index.find_candidate_rules(&vec!["Logger".to_string()]);
        assert!(candidates.contains(&"MethodCallRule".to_string()));
    }

    #[test]
    fn test_index_handles_dots_in_field_names() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "Order.Customer.Address.City"));

        // Should handle deeply nested fields
        let candidates = index.find_candidate_rules(&vec![
            "Order.Customer.Address.City".to_string()
        ]);

        assert!(candidates.len() >= 1);
        assert!(candidates.contains(&"Rule1".to_string()));
    }

    #[test]
    fn test_case_sensitivity() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));

        // Field names should be case-sensitive
        let candidates_exact = index.find_candidate_rules(&vec!["User.IsVIP".to_string()]);
        let candidates_wrong_case = index.find_candidate_rules(&vec!["user.isvip".to_string()]);

        assert_eq!(candidates_exact.len(), 1);
        assert_eq!(candidates_wrong_case.len(), 0);
    }
}

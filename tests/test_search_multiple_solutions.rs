//! Direct test for SearchResult.solutions field

#[cfg(feature = "backward-chaining")]
use rust_rule_engine::backward::goal::Goal;
#[cfg(feature = "backward-chaining")]
use rust_rule_engine::backward::search::DepthFirstSearch;
#[cfg(feature = "backward-chaining")]
use rust_rule_engine::types::{ActionType, Operator, Value};
#[cfg(feature = "backward-chaining")]
use rust_rule_engine::{Condition, ConditionGroup, Facts, KnowledgeBase, Rule};

#[cfg(feature = "backward-chaining")]
#[test]
fn test_dfs_finds_multiple_solutions() {
    println!("\n=== Testing DFS with max_solutions > 1 ===\n");

    // Create knowledge base with multiple rules that can prove same goal
    let kb = KnowledgeBase::new("multi_solutions");

    // Rule 1: User.Type == "Premium" -> User.Access = true
    kb.add_rule(Rule::new(
        "PremiumRule".to_string(),
        ConditionGroup::single(Condition::new(
            "User.Type".to_string(),
            Operator::Equal,
            Value::String("Premium".to_string()),
        )),
        vec![ActionType::Set {
            field: "User.Access".to_string(),
            value: Value::Boolean(true),
        }],
    ))
    .unwrap();

    // Rule 2: User.HasLicense == true -> User.Access = true
    kb.add_rule(Rule::new(
        "LicenseRule".to_string(),
        ConditionGroup::single(Condition::new(
            "User.HasLicense".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "User.Access".to_string(),
            value: Value::Boolean(true),
        }],
    ))
    .unwrap();

    // Create goal
    let mut goal = Goal::new("User.Access == true".to_string());

    // Manually add candidate rules (simulating what BackwardEngine does)
    goal.add_candidate_rule("PremiumRule".to_string());
    goal.add_candidate_rule("LicenseRule".to_string());

    // Test with max_solutions = 1
    println!("Test 1: max_solutions = 1");
    let mut dfs1 = DepthFirstSearch::new(10, kb.clone()).with_max_solutions(1);

    let mut facts1 = Facts::new();
    facts1.set("User.Type", Value::String("Premium".to_string()));
    facts1.set("User.HasLicense", Value::Boolean(true));

    let result1 = dfs1.search_with_execution(&mut goal.clone(), &mut facts1, &kb);

    println!("  Success: {}", result1.success);
    println!("  Solutions count: {}", result1.solutions.len());
    println!("  Path: {:?}", result1.path);

    assert!(result1.success, "Should find a solution");
    assert!(
        result1.solutions.len() <= 1,
        "Should have at most 1 solution"
    );

    // Test with max_solutions = 5
    println!("\nTest 2: max_solutions = 5");
    let mut dfs5 = DepthFirstSearch::new(10, kb.clone()).with_max_solutions(5);

    let mut facts5 = Facts::new();
    facts5.set("User.Type", Value::String("Premium".to_string()));
    facts5.set("User.HasLicense", Value::Boolean(true));

    let mut goal5 = Goal::new("User.Access == true".to_string());
    goal5.add_candidate_rule("PremiumRule".to_string());
    goal5.add_candidate_rule("LicenseRule".to_string());

    let result5 = dfs5.search_with_execution(&mut goal5, &mut facts5, &kb);

    println!("  Success: {}", result5.success);
    println!("  Solutions count: {}", result5.solutions.len());
    println!("  Main path: {:?}", result5.path);

    println!("\n  All solutions:");
    for (i, solution) in result5.solutions.iter().enumerate() {
        println!(
            "    Solution {}: path = {:?}, bindings = {:?}",
            i + 1,
            solution.path,
            solution.bindings
        );
    }

    assert!(result5.success, "Should find solutions");

    if result5.solutions.len() > 1 {
        println!("\n✅ SUCCESS: Found {} solutions!", result5.solutions.len());

        // Verify they use different paths/rules
        let mut unique_paths = std::collections::HashSet::new();
        for sol in &result5.solutions {
            unique_paths.insert(sol.path.clone());
        }

        println!("   Unique paths: {}", unique_paths.len());
        assert!(
            unique_paths.len() > 1,
            "Solutions should use different rule paths"
        );
        println!("✅ VERIFIED: Solutions use different rules\n");
    } else {
        println!(
            "\n❌ ISSUE: Only found {} solution(s)",
            result5.solutions.len()
        );
        println!("   Expected multiple solutions because:");
        println!("   - Both PremiumRule and LicenseRule can prove the goal");
        println!("   - Both rules' conditions are satisfied in facts");
        println!("   - max_solutions = 5 > 1");

        panic!(
            "Expected multiple solutions but only found {}",
            result5.solutions.len()
        );
    }
}

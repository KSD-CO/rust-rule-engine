//! GRL Parser Test - Verify Nested Queries and Optimizer Integration
//!
//! This example tests parsing of GRL queries with:
//! - Nested queries (subqueries)
//! - Optimization settings
//! - Complex goal expressions
//!
//! Run with: cargo run --example grl_parsing_test --features backward-chaining

use rust_rule_engine::backward::{GRLQueryParser, GRLQuery};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=".repeat(80));
    println!("GRL Parser Test - Nested Queries & Optimization");
    println!("=".repeat(80));
    println!();

    // Test 1: Simple query with optimization
    test_simple_with_optimization()?;

    // Test 2: Query with nested subquery syntax
    test_nested_query_syntax()?;

    // Test 3: Complex query with multiple features
    test_complex_query()?;

    // Test 4: Query with optimization disabled
    test_optimization_disabled()?;

    // Test 5: All features combined
    test_all_features_combined()?;

    println!();
    println!("=".repeat(80));
    println!("✅ All Parser Tests Passed!");
    println!("=".repeat(80));

    Ok(())
}

fn test_simple_with_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 1: Simple Query with Optimization");
    println!("-".repeat(80));

    let query_str = r#"
    query "SimpleOptimized" {
        goal: eligible(?customer)
        strategy: depth-first
        max-depth: 10
        enable-optimization: true
    }
    "#;

    let query = GRLQueryParser::parse(query_str)?;

    assert_eq!(query.name, "SimpleOptimized");
    assert_eq!(query.goal, "eligible(?customer)");
    assert!(query.enable_optimization, "Optimization should be enabled");

    println!("✅ Parsed successfully:");
    println!("   Name: {}", query.name);
    println!("   Goal: {}", query.goal);
    println!("   Optimization: {}", query.enable_optimization);
    println!();

    Ok(())
}

fn test_nested_query_syntax() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 2: Nested Query Syntax");
    println!("-".repeat(80));

    let query_str = r#"
    query "GrandparentQuery" {
        goal: grandparent(?x, ?z) WHERE parent(?x, ?y) AND parent(?y, ?z)
        strategy: depth-first
        max-depth: 15
        enable-optimization: true
    }
    "#;

    let query = GRLQueryParser::parse(query_str)?;

    assert_eq!(query.name, "GrandparentQuery");
    assert!(query.goal.contains("WHERE"), "Goal should contain WHERE clause");
    assert!(query.goal.contains("AND"), "Goal should contain AND operator");
    assert!(query.enable_optimization);

    println!("✅ Parsed successfully:");
    println!("   Name: {}", query.name);
    println!("   Goal: {}", query.goal);
    println!("   Contains WHERE: {}", query.goal.contains("WHERE"));
    println!("   Optimization: {}", query.enable_optimization);
    println!();

    Ok(())
}

fn test_complex_query() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 3: Complex Query with Multiple Features");
    println!("-".repeat(80));

    let query_str = r#"
    query "ComplexEligibility" {
        goal: (high_value(?c) OR premium(?c)) AND active(?c) AND verified(?c)
        strategy: breadth-first
        max-depth: 20
        max-solutions: 10
        enable-optimization: true
        enable-memoization: true

        on-success: {
            Print("Customer qualified!");
        }

        on-failure: {
            Print("Customer not qualified");
        }
    }
    "#;

    let query = GRLQueryParser::parse(query_str)?;

    assert_eq!(query.name, "ComplexEligibility");
    assert!(query.goal.contains("OR"), "Goal should contain OR operator");
    assert!(query.goal.contains("AND"), "Goal should contain AND operator");
    assert!(query.enable_optimization);
    assert!(query.enable_memoization);
    assert_eq!(query.max_solutions, 10);
    assert!(query.on_success.is_some(), "Should have on-success action");
    assert!(query.on_failure.is_some(), "Should have on-failure action");

    println!("✅ Parsed successfully:");
    println!("   Name: {}", query.name);
    println!("   Goal: {}", query.goal);
    println!("   Strategy: {:?}", query.strategy);
    println!("   Max Solutions: {}", query.max_solutions);
    println!("   Optimization: {}", query.enable_optimization);
    println!("   Memoization: {}", query.enable_memoization);
    println!("   Has on-success: {}", query.on_success.is_some());
    println!("   Has on-failure: {}", query.on_failure.is_some());
    println!();

    Ok(())
}

fn test_optimization_disabled() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 4: Query with Optimization Disabled");
    println!("-".repeat(80));

    let query_str = r#"
    query "NoOptimization" {
        goal: customer(?id) AND verified(?id) AND active(?id)
        strategy: depth-first
        enable-optimization: false
    }
    "#;

    let query = GRLQueryParser::parse(query_str)?;

    assert_eq!(query.name, "NoOptimization");
    assert!(!query.enable_optimization, "Optimization should be disabled");

    println!("✅ Parsed successfully:");
    println!("   Name: {}", query.name);
    println!("   Goal: {}", query.goal);
    println!("   Optimization: {}", query.enable_optimization);
    println!();

    Ok(())
}

fn test_all_features_combined() -> Result<(), Box<dyn std::error::Error>> {
    println!("Test 5: All Features Combined");
    println!("-".repeat(80));

    let query_str = r#"
    query "UltimateQuery" {
        goal: (eligible(?x) WHERE (vip(?x) OR premium(?x))) AND active(?x) AND NOT suspended(?x)
        strategy: iterative
        max-depth: 25
        max-solutions: 5
        enable-optimization: true
        enable-memoization: true

        when: User.Role == "admin"

        on-success: {
            User.Status = "approved";
            Print("Query succeeded");
        }

        on-failure: {
            User.Status = "rejected";
            Print("Query failed");
        }

        on-missing: {
            Print("Missing required facts");
        }
    }
    "#;

    let query = GRLQueryParser::parse(query_str)?;

    // Verify all parsed fields
    assert_eq!(query.name, "UltimateQuery");
    assert!(query.goal.contains("WHERE"), "Should have nested WHERE");
    assert!(query.goal.contains("OR"), "Should have OR operator");
    assert!(query.goal.contains("AND"), "Should have AND operator");
    assert!(query.goal.contains("NOT"), "Should have NOT operator");
    assert_eq!(query.max_depth, 25);
    assert_eq!(query.max_solutions, 5);
    assert!(query.enable_optimization);
    assert!(query.enable_memoization);
    assert!(query.when_condition.is_some(), "Should have when condition");
    assert!(query.on_success.is_some());
    assert!(query.on_failure.is_some());
    assert!(query.on_missing.is_some());

    println!("✅ Parsed successfully with ALL features:");
    println!("   Name: {}", query.name);
    println!("   Goal: {}", query.goal);
    println!("   Strategy: {:?}", query.strategy);
    println!("   Max Depth: {}", query.max_depth);
    println!("   Max Solutions: {}", query.max_solutions);
    println!("   Optimization: {}", query.enable_optimization);
    println!("   Memoization: {}", query.enable_memoization);
    println!("   Has WHEN condition: {}", query.when_condition.is_some());
    println!("   Has on-success: {}", query.on_success.is_some());
    println!("   Has on-failure: {}", query.on_failure.is_some());
    println!("   Has on-missing: {}", query.on_missing.is_some());

    // Verify action parsing
    if let Some(ref action) = query.on_success {
        println!("   Success actions:");
        println!("     - Assignments: {}", action.assignments.len());
        println!("     - Function calls: {}", action.calls.len());
    }

    println!();

    // Print statistics
    print_statistics(&query);

    Ok(())
}

fn print_statistics(query: &GRLQuery) {
    println!("📊 Query Statistics:");
    println!("   Features used:");

    let mut features = Vec::new();

    if query.goal.contains("WHERE") {
        features.push("Nested queries (WHERE)");
    }
    if query.goal.contains("OR") || query.goal.contains("||") {
        features.push("Disjunction (OR)");
    }
    if query.goal.contains("AND") || query.goal.contains("&&") {
        features.push("Conjunction (AND)");
    }
    if query.goal.contains("NOT") {
        features.push("Negation (NOT)");
    }
    if query.enable_optimization {
        features.push("Query optimization");
    }
    if query.enable_memoization {
        features.push("Memoization");
    }
    if query.when_condition.is_some() {
        features.push("Conditional execution");
    }
    if query.on_success.is_some() || query.on_failure.is_some() || query.on_missing.is_some() {
        features.push("Action handlers");
    }

    for (i, feature) in features.iter().enumerate() {
        println!("   {}. {}", i + 1, feature);
    }

    println!();
    println!("   Total features: {}", features.len());
    println!();
}

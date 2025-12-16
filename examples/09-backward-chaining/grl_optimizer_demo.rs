//! GRL Query with Optimizer Demo
//!
//! This example demonstrates using query optimization in GRL syntax.
//! It shows how the optimizer automatically reorders goals for better performance.
//!
//! Run with: cargo run --example grl_optimizer_demo --features backward-chaining

use rust_rule_engine::backward::{BackwardEngine, GRLQueryParser};
use rust_rule_engine::{Facts, KnowledgeBase, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("GRL Query Optimizer Demo");
    println!("{}", "=".repeat(80));
    println!();

    // Demo 1: Query with optimization enabled (default)
    demo_with_optimization()?;

    // Demo 2: Query with optimization disabled
    demo_without_optimization()?;

    // Demo 3: Complex query benefiting from optimization
    demo_complex_optimization()?;

    println!();
    println!("{}", "=".repeat(80));
    println!("Demo Complete!");
    println!("{}", "=".repeat(80));

    Ok(())
}

fn demo_with_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 1: Query with Optimization Enabled");
    println!("{}", "-".repeat(80));

    let query_str = r#"
    query "FindEligibleCustomers" {
        goal: eligible(?customer)
        strategy: depth-first
        max-depth: 10
        enable-optimization: true

        on-success: {
            Print("Found eligible customer");
        }
    }
    "#;

    println!("Query Definition:");
    println!("{}", query_str);
    println!();

    let query = GRLQueryParser::parse(query_str)?;

    println!(
        "Optimization Status: {}",
        if query.enable_optimization {
            "ENABLED"
        } else {
            "DISABLED"
        }
    );
    println!("Strategy: {:?}", query.strategy);
    println!("Max Depth: {}", query.max_depth);
    println!();

    // Setup engine and facts
    let kb = KnowledgeBase::new("test");
    let bc_engine = BackwardEngine::new(kb);
    let facts = Facts::new();

    // Add some test facts
    facts.set("customer.vip", Value::Boolean(true));
    facts.set("customer.id", Value::String("C123".to_string()));

    println!("Note: With optimization enabled, goals will be automatically reordered");
    println!("      for optimal evaluation performance.");
    println!();

    Ok(())
}

fn demo_without_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 2: Query with Optimization Disabled");
    println!("{}", "-".repeat(80));

    let query_str = r#"
    query "SimpleQuery" {
        goal: customer(?id)
        strategy: depth-first
        enable-optimization: false
    }
    "#;

    println!("Query Definition:");
    println!("{}", query_str);
    println!();

    let query = GRLQueryParser::parse(query_str)?;

    println!(
        "Optimization Status: {}",
        if query.enable_optimization {
            "ENABLED"
        } else {
            "DISABLED"
        }
    );
    println!();

    println!("Note: With optimization disabled, goals are evaluated in the order");
    println!("      they appear in the query (no reordering).");
    println!();

    Ok(())
}

fn demo_complex_optimization() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 3: Complex Query Benefiting from Optimization");
    println!("{}", "-".repeat(80));

    let query_str = r#"
    query "ComplexEligibility" {
        goal: high_value(?customer) && active(?customer) && verified(?customer)
        strategy: depth-first
        max-depth: 15
        max-solutions: 5
        enable-optimization: true
        enable-memoization: true

        on-success: {
            Print("Found high-value active verified customer");
        }

        on-failure: {
            Print("No matching customers found");
        }
    }
    "#;

    println!("Query Definition:");
    println!("{}", query_str);
    println!();

    let query = GRLQueryParser::parse(query_str)?;

    println!("Query Analysis:");
    println!("  Name: {}", query.name);
    println!("  Goal: {}", query.goal);
    println!(
        "  Optimization: {}",
        if query.enable_optimization {
            "ENABLED"
        } else {
            "DISABLED"
        }
    );
    println!(
        "  Memoization: {}",
        if query.enable_memoization {
            "ENABLED"
        } else {
            "DISABLED"
        }
    );
    println!("  Max Solutions: {}", query.max_solutions);
    println!();

    println!("Optimization Benefits:");
    println!("  - Goals are reordered by selectivity (most selective first)");
    println!("  - Example: If 'verified' is most selective, it's evaluated first");
    println!("  - This reduces the search space dramatically:");
    println!("    * Without: eval all customers → check active → check value → check verified");
    println!("    * With:    eval verified → check value → check active");
    println!("  - Can achieve 10-100x speedup on large datasets!");
    println!();

    println!("Memoization Benefits:");
    println!("  - Intermediate results are cached");
    println!("  - Repeated subgoals return immediately from cache");
    println!("  - Especially effective for queries with shared subgoals");
    println!();

    Ok(())
}

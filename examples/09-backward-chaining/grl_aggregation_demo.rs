//! GRL Aggregation Queries Demo
//!
//! Demonstrates how to use aggregation functions in GRL query syntax
//!
//! Run: cargo run --example grl_aggregation_demo --features backward-chaining

use rust_rule_engine::backward::{BackwardEngine, GRLQuery};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::KnowledgeBase;
use rust_rule_engine::types::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 GRL Aggregation Queries Demo");
    println!("{}", "=".repeat(80));
    println!();

    // Demo 1: Load and execute GRL aggregation queries from string
    demo_inline_grl_aggregation()?;

    // Demo 2: Programmatic GRL query creation
    demo_programmatic_aggregation()?;

    println!("\n{}", "=".repeat(80));
    println!("✅ All GRL aggregation demos completed successfully!");

    Ok(())
}

/// Demo 1: Inline GRL aggregation queries
fn demo_inline_grl_aggregation() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 1: Inline GRL Aggregation Queries");
    println!("{}", "-".repeat(80));

    // Note: Full GRL parser integration would be in future phase
    // For now, we demonstrate the goal pattern format

    let kb = KnowledgeBase::new("AggregationDemo");
    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();

    // Add some employee data
    facts.set("employee1.name", Value::String("Alice".to_string()));
    facts.set("employee1.salary", Value::Integer(80000));

    facts.set("employee2.name", Value::String("Bob".to_string()));
    facts.set("employee2.salary", Value::Integer(90000));

    facts.set("employee3.name", Value::String("Charlie".to_string()));
    facts.set("employee3.salary", Value::Integer(75000));

    println!("Employee Data:");
    println!("  Alice - $80,000");
    println!("  Bob - $90,000");
    println!("  Charlie - $75,000");
    println!();

    // Example GRL query format (for documentation)
    let example_grl = r#"
query "TotalPayroll" {
    goal: sum(?salary) WHERE salary(?name, ?salary)
    on-success: {
        Payroll.Total = result;
        LogMessage("Payroll calculated");
    }
}
    "#;

    println!("Example GRL Query:");
    println!("{}", example_grl);

    // Direct aggregation query execution
    // (GRL parser integration would go here in future phase)
    println!("Note: Direct aggregation via query_aggregate() API:");
    println!("  let total = engine.query_aggregate(");
    println!("      \"sum(?salary) WHERE salary(?name, ?salary)\",");
    println!("      &mut facts");
    println!("  )?;");

    println!();
    Ok(())
}

/// Demo 2: Programmatic GRL query creation
fn demo_programmatic_aggregation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Demo 2: Programmatic GRL Query Creation");
    println!("{}", "-".repeat(80));

    let kb = KnowledgeBase::new("ProgrammaticDemo");
    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();

    // Add product data
    let products = vec![
        ("laptop", 999.99),
        ("mouse", 29.99),
        ("keyboard", 79.99),
        ("monitor", 299.99),
    ];

    for (i, (name, price)) in products.iter().enumerate() {
        facts.set(&format!("product{}.name", i), Value::String(name.to_string()));
        facts.set(&format!("product{}.price", i), Value::Number(*price));
    }

    println!("Product Data:");
    for (name, price) in &products {
        println!("  {} - ${:.2}", name, price);
    }
    println!();

    // Create GRL query programmatically
    let count_query = GRLQuery::new(
        "CountProducts".to_string(),
        "count(?x) WHERE product(?x)".to_string(),
    )
    .with_max_depth(10)
    .with_memoization(true);

    let avg_query = GRLQuery::new(
        "AveragePrice".to_string(),
        "avg(?price) WHERE product(?name, ?price)".to_string(),
    )
    .with_max_depth(10);

    let min_query = GRLQuery::new(
        "MinPrice".to_string(),
        "min(?price) WHERE product(?name, ?price)".to_string(),
    );

    let max_query = GRLQuery::new(
        "MaxPrice".to_string(),
        "max(?price) WHERE product(?name, ?price)".to_string(),
    );

    println!("Created GRL Queries:");
    println!("  1. {} - goal: {}", count_query.name, count_query.goal);
    println!("  2. {} - goal: {}", avg_query.name, avg_query.goal);
    println!("  3. {} - goal: {}", min_query.name, min_query.goal);
    println!("  4. {} - goal: {}", max_query.name, max_query.goal);
    println!();

    println!("Query Properties:");
    println!("  Count Query:");
    println!("    - Max Depth: {}", count_query.max_depth);
    println!("    - Memoization: {}", count_query.enable_memoization);
    println!("    - Max Solutions: {}", count_query.max_solutions);
    println!();

    // These queries can be executed directly using engine.query_aggregate()
    println!("Execution:");
    println!("  // For full GRL support, use:");
    println!("  // let result = GRLQueryExecutor::execute(&count_query, &mut engine, &mut facts)?;");
    println!("  ");
    println!("  // For now, use direct API:");
    println!("  // let count = engine.query_aggregate(&count_query.goal, &mut facts)?;");

    println!();
    Ok(())
}

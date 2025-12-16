//! Nested Queries from GRL File Demo
//!
//! This example demonstrates parsing and executing nested queries
//! from a .grl file with proper syntax support.
//!
//! Run with: cargo run --example nested_grl_file_demo --features backward-chaining

use rust_rule_engine::backward::{BackwardEngine, GRLQueryExecutor, GRLQueryParser};
use rust_rule_engine::{Facts, KnowledgeBase, Value};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(80));
    println!("Nested Queries from GRL File Demo");
    println!("{}", "=".repeat(80));
    println!();

    // Read GRL file
    let grl_content = fs::read_to_string("examples/rules/09-backward-chaining/nested_queries.grl")?;

    println!("📁 Loaded GRL file: nested_queries.grl");
    println!("File size: {} bytes", grl_content.len());
    println!();

    // Parse all queries from the file
    let queries = parse_multiple_queries(&grl_content)?;

    println!("📊 Parsed {} queries from file", queries.len());
    println!();

    // Setup backward chaining engine
    let kb = KnowledgeBase::new("demo");
    let mut bc_engine = BackwardEngine::new(kb);

    // Add some rules for testing
    setup_test_rules(&mut bc_engine)?;

    // Setup test facts
    let mut facts = setup_test_facts();

    // Execute each query
    for (i, query) in queries.iter().enumerate() {
        println!("{}. Executing query: \"{}\"", i + 1, query.name);
        println!("   Goal: {}", query.goal);
        println!(
            "   Optimization: {}",
            if query.enable_optimization {
                "enabled"
            } else {
                "disabled"
            }
        );
        println!("   Strategy: {:?}", query.strategy);

        // Check if query has nested syntax
        if query.goal.contains(" WHERE ") {
            println!("   ✓ Contains nested WHERE clause");
        }
        if query.goal.contains(" OR ") {
            println!("   ✓ Contains OR operator");
        }
        if query.goal.contains(" NOT ") {
            println!("   ✓ Contains NOT operator");
        }

        // Execute the query
        match GRLQueryExecutor::execute(query, &mut bc_engine, &mut facts) {
            Ok(result) => {
                println!(
                    "   Result: {}",
                    if result.provable {
                        "✓ Provable"
                    } else {
                        "✗ Not provable"
                    }
                );
                println!("   Solutions: {}", result.solutions.len());
                println!(
                    "   Stats: {} goals explored, {} rules evaluated",
                    result.stats.goals_explored, result.stats.rules_evaluated
                );
            }
            Err(e) => {
                println!("   Error: {}", e);
            }
        }

        println!();
    }

    // Detailed analysis of first query
    if let Some(first_query) = queries.first() {
        println!("{}", "=".repeat(80));
        println!("Detailed Analysis of First Query");
        println!("{}", "=".repeat(80));
        println!();

        analyze_query(first_query);
    }

    println!("{}", "=".repeat(80));
    println!("Demo Complete!");
    println!("{}", "=".repeat(80));

    Ok(())
}

/// Parse multiple queries from a GRL file
fn parse_multiple_queries(
    content: &str,
) -> Result<Vec<rust_rule_engine::backward::GRLQuery>, Box<dyn std::error::Error>> {
    let mut queries = Vec::new();

    // Split by query definitions
    let parts: Vec<&str> = content.split("query \"").collect();

    for part in parts.iter().skip(1) {
        // Reconstruct query definition
        let query_str = format!("query \"{}", part);

        // Find the end of this query (next query or end of file)
        let end_idx = if let Some(idx) = query_str[6..].find("query \"") {
            idx + 6
        } else {
            query_str.len()
        };

        let single_query = &query_str[..end_idx];

        // Parse the query
        match GRLQueryParser::parse(single_query) {
            Ok(query) => queries.push(query),
            Err(e) => {
                eprintln!("Warning: Failed to parse query: {}", e);
            }
        }
    }

    Ok(queries)
}

/// Setup test rules for backward chaining
fn setup_test_rules(engine: &mut BackwardEngine) -> Result<(), Box<dyn std::error::Error>> {
    // Add some basic inference rules
    // These would normally be loaded from a knowledge base

    println!("📝 Setting up test rules...");
    println!("   (In production, these would be loaded from your knowledge base)");
    println!();

    Ok(())
}

/// Setup test facts
fn setup_test_facts() -> Facts {
    let facts = Facts::new();

    // Customer data
    facts.set("Customer.ID", Value::String("C123".to_string()));
    facts.set("Customer.TotalSpent", Value::Number(15000.0));
    facts.set("Customer.IsVIP", Value::Boolean(true));
    facts.set("Customer.IsPremium", Value::Boolean(false));
    facts.set("Customer.IsActive", Value::Boolean(true));
    facts.set("Customer.LoyaltyYears", Value::Number(5.0));

    // Item data
    facts.set("Item.ID", Value::String("ITEM-001".to_string()));
    facts.set("Item.Price", Value::Number(500.0));
    facts.set("Item.InStock", Value::Boolean(true));
    facts.set("Item.Category", Value::String("Electronics".to_string()));
    facts.set("Item.Sold", Value::Boolean(false));

    println!("📦 Test facts loaded:");
    println!("   - Customer data (VIP, active, $15k spent)");
    println!("   - Item data (in stock, $500, electronics)");
    println!();

    facts
}

/// Analyze a query in detail
fn analyze_query(query: &rust_rule_engine::backward::GRLQuery) {
    println!("Query Name: {}", query.name);
    println!("Goal Expression: {}", query.goal);
    println!();

    println!("Configuration:");
    println!("  Strategy: {:?}", query.strategy);
    println!("  Max Depth: {}", query.max_depth);
    println!("  Max Solutions: {}", query.max_solutions);
    println!("  Optimization: {}", query.enable_optimization);
    println!("  Memoization: {}", query.enable_memoization);
    println!();

    println!("Features Used:");
    let mut features = Vec::new();

    if query.goal.contains(" WHERE ") {
        features.push("Nested queries (WHERE clause)");
    }
    if query.goal.contains(" OR ") {
        features.push("Disjunction (OR)");
    }
    if query.goal.contains(" AND ") {
        features.push("Conjunction (AND)");
    }
    if query.goal.contains(" NOT ") {
        features.push("Negation (NOT)");
    }
    if query.enable_optimization {
        features.push("Query optimization");
    }
    if query.enable_memoization {
        features.push("Result memoization");
    }

    for (i, feature) in features.iter().enumerate() {
        println!("  {}. {}", i + 1, feature);
    }

    if features.is_empty() {
        println!("  (Simple query, no advanced features)");
    }

    println!();

    println!("Action Handlers:");
    if query.on_success.is_some() {
        println!("  ✓ on-success handler defined");
    }
    if query.on_failure.is_some() {
        println!("  ✓ on-failure handler defined");
    }
    if query.on_missing.is_some() {
        println!("  ✓ on-missing handler defined");
    }
    if query.on_success.is_none() && query.on_failure.is_none() && query.on_missing.is_none() {
        println!("  (No action handlers)");
    }

    println!();
}

//! Nested Query Demo for Backward Chaining
//!
//! This example demonstrates nested queries (subqueries) in backward chaining.
//! Nested queries allow using the results of one query as input to another,
//! enabling complex multi-level reasoning.
//!
//! Run with: cargo run --example nested_query_demo --features backward-chaining

use rust_rule_engine::backward::{
    NestedQuery, Query, NestedQueryParser, NestedQueryStats,
    Goal,
};

fn main() {
    println!("{}", "=".repeat(80));
    println!("Nested Query Demo - Backward Chaining");
    println!("{}", "=".repeat(80));
    println!();

    // Demo 1: Parsing nested queries
    demo_nested_query_parsing();

    // Demo 2: Query variable extraction
    demo_query_variables();

    // Demo 3: Shared variables in nested queries
    demo_shared_variables();

    // Demo 4: Nested query statistics
    demo_nested_stats();

    println!();
    println!("{}", "=".repeat(80));
    println!("Demo Complete!");
    println!("{}", "=".repeat(80));
}

fn demo_nested_query_parsing() {
    println!("📝 Demo 1: Nested Query Parsing");
    println!("{}", "-".repeat(80));

    let query1 = "grandparent(?x, ?z) WHERE parent(?x, ?y) AND (parent(?y, ?z) WHERE child(?z, ?y))";
    println!("Query: {}", query1);
    println!("Has nested: {}", NestedQueryParser::has_nested(query1));
    println!();

    let query2 = "parent(?x, ?y) WHERE person(?x) AND person(?y)";
    println!("Query: {}", query2);
    println!("Has nested: {}", NestedQueryParser::has_nested(query2));
    println!();

    let query3 = "high_value(?customer) WHERE (vip(?customer) OR (purchase(?customer, ?item, ?amount) WHERE ?amount > 1000))";
    println!("Query: {}", query3);
    println!("Has nested: {}", NestedQueryParser::has_nested(query3));
    println!();
}

fn demo_query_variables() {
    println!("📝 Demo 2: Query Variable Extraction");
    println!("{}", "-".repeat(80));

    let mut query = Query::new("test_query".to_string());

    // Add goals with variables
    query.add_goal(Goal::new("parent(?x, ?y)".to_string()));
    query.add_goal(Goal::new("age(?x, ?age)".to_string()));
    query.add_goal(Goal::new("income(?x, ?salary)".to_string()));

    let vars = query.variables();
    println!("Goals:");
    for goal in &query.goals {
        println!("  - {}", goal.pattern);
    }
    println!();

    println!("Extracted variables:");
    for var in &vars {
        println!("  - {}", var);
    }
    println!();

    println!("Total unique variables: {}", vars.len());
    println!();
}

fn demo_shared_variables() {
    println!("📝 Demo 3: Shared Variables in Nested Queries");
    println!("{}", "-".repeat(80));

    // Example 1: Grandparent query
    println!("Example 1: Grandparent Query");
    println!("Outer: grandparent(?x, ?z)");
    println!("Inner: parent(?y, ?z)");
    println!();

    let outer_goal = Goal::new("grandparent(?x, ?z)".to_string());
    let mut subquery = Query::new("parent(?y, ?z)".to_string());
    subquery.add_goal(Goal::new("parent(?y, ?z)".to_string()));

    let nested = NestedQuery::new(outer_goal, subquery);

    println!("Shared variables: {:?}", nested.shared_variables);
    println!("Has shared variables: {}", nested.has_shared_variables());
    println!();

    // Example 2: Independent queries (no shared variables)
    println!("Example 2: Independent Queries");
    println!("Outer: employee(?x)");
    println!("Inner: department(?y)");
    println!();

    let outer_goal2 = Goal::new("employee(?x)".to_string());
    let mut subquery2 = Query::new("department(?y)".to_string());
    subquery2.add_goal(Goal::new("department(?y)".to_string()));

    let nested2 = NestedQuery::new(outer_goal2, subquery2);

    println!("Shared variables: {:?}", nested2.shared_variables);
    println!("Has shared variables: {}", nested2.has_shared_variables());
    println!();
}

fn demo_nested_stats() {
    println!("📝 Demo 4: Nested Query Statistics");
    println!("{}", "-".repeat(80));

    let mut stats = NestedQueryStats::new();

    // Simulate evaluating nested queries
    println!("Simulating nested query evaluations...");
    stats.record_evaluation(1, 5);
    println!("  Evaluation 1: depth=1, solutions=5");

    stats.record_evaluation(2, 10);
    println!("  Evaluation 2: depth=2, solutions=10");

    stats.record_evaluation(2, 8);
    println!("  Evaluation 3: depth=2, solutions=8");

    stats.record_evaluation(3, 15);
    println!("  Evaluation 4: depth=3, solutions=15");

    println!();
    println!("Statistics:");
    println!("  Total nested queries: {}", stats.total_nested);
    println!("  Total subquery evaluations: {}", stats.total_subquery_evals);
    println!("  Maximum depth: {}", stats.max_depth);
    println!("  Average solutions per subquery: {:.2}", stats.avg_subquery_solutions);
    println!();

    println!("Summary: {}", stats.summary());
    println!();
}

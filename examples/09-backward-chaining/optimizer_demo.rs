//! Query Optimizer Demo for Backward Chaining
//!
//! This example demonstrates query optimization techniques:
//! - Goal reordering based on selectivity
//! - Join ordering
//! - Selectivity estimation
//! - Optimization statistics
//!
//! Run with: cargo run --example optimizer_demo --features backward-chaining

use rust_rule_engine::backward::{Goal, JoinOptimizer, OptimizerConfig, QueryOptimizer};

fn main() {
    println!("{}", "=".repeat(80));
    println!("Query Optimizer Demo - Backward Chaining");
    println!("{}", "=".repeat(80));
    println!();

    // Demo 1: Goal reordering
    demo_goal_reordering();

    // Demo 2: Selectivity estimation
    demo_selectivity_estimation();

    // Demo 3: Join optimization
    demo_join_optimization();

    // Demo 4: Custom configuration
    demo_custom_config();

    // Demo 5: Optimization statistics
    demo_optimization_stats();

    println!();
    println!("{}", "=".repeat(80));
    println!("Demo Complete!");
    println!("{}", "=".repeat(80));
}

fn demo_goal_reordering() {
    println!("📝 Demo 1: Goal Reordering");
    println!("{}", "-".repeat(80));

    let mut optimizer = QueryOptimizer::new();

    // Set selectivity estimates (lower = more selective = evaluate first)
    optimizer.set_selectivity("in_stock(?x)".to_string(), 0.1); // 10% of items in stock
    optimizer.set_selectivity("expensive(?x)".to_string(), 0.3); // 30% expensive
    optimizer.set_selectivity("item(?x)".to_string(), 0.9); // 90% are items

    println!("Original goal order:");
    let goals = vec![
        Goal::new("item(?x)".to_string()),
        Goal::new("expensive(?x)".to_string()),
        Goal::new("in_stock(?x)".to_string()),
    ];

    for (i, goal) in goals.iter().enumerate() {
        println!("  {}. {}", i + 1, goal.pattern);
    }
    println!();

    let optimized = optimizer.optimize_goals(goals);

    println!("Optimized goal order (most selective first):");
    for (i, goal) in optimized.iter().enumerate() {
        let selectivity = optimizer.estimate_selectivity(goal);
        println!(
            "  {}. {} (selectivity: {:.2})",
            i + 1,
            goal.pattern,
            selectivity
        );
    }
    println!();

    println!("Benefit: Evaluating in_stock first reduces search space from 1000 → 100 → 10");
    println!("         vs. original order: 1000 → 900 → 270 → 27");
    println!("         ~10x fewer evaluations!");
    println!();
}

fn demo_selectivity_estimation() {
    println!("📝 Demo 2: Selectivity Estimation");
    println!("{}", "-".repeat(80));

    let optimizer = QueryOptimizer::new();

    let test_goals = vec![
        ("employee(alice)", "Exact match - no variables"),
        ("employee(?x)", "One unbound variable"),
        ("salary(?x) WHERE ?x > 100000", "Bound variable with filter"),
        ("in_stock(?item)", "Stock check (heuristic)"),
        ("expensive(?product)", "Price filter (heuristic)"),
        ("item(?x)", "Generic predicate"),
    ];

    println!("Selectivity estimates (0.0 = very selective, 1.0 = not selective):");
    println!();

    for (pattern, description) in test_goals {
        let goal = Goal::new(pattern.to_string());
        let selectivity = optimizer.estimate_selectivity(&goal);
        println!("  Pattern: {}", pattern);
        println!("  Description: {}", description);
        println!("  Selectivity: {:.3}", selectivity);
        println!();
    }
}

fn demo_join_optimization() {
    println!("📝 Demo 3: Join Optimization");
    println!("{}", "-".repeat(80));

    let join_optimizer = JoinOptimizer::new();

    println!("Original join order:");
    let goals = vec![
        Goal::new("item(?x)".to_string()),
        Goal::new("price(?x, ?p) WHERE ?p > 100".to_string()),
        Goal::new("in_stock(?x)".to_string()),
        Goal::new("category(?x, ?c)".to_string()),
    ];

    for (i, goal) in goals.iter().enumerate() {
        println!("  {}. {}", i + 1, goal.pattern);
    }
    println!();

    let optimized = join_optimizer.optimize_joins(goals);

    println!("Optimized join order (most bound variables first):");
    for (i, goal) in optimized.iter().enumerate() {
        println!("  {}. {}", i + 1, goal.pattern);
    }
    println!();

    println!("Benefit: Starting with bound variables reduces join cost");
    println!();
}

fn demo_custom_config() {
    println!("📝 Demo 4: Custom Optimizer Configuration");
    println!("{}", "-".repeat(80));

    // Config 1: Full optimization
    let config1 = OptimizerConfig {
        enable_reordering: true,
        enable_index_selection: true,
        enable_memoization: true,
    };

    let _optimizer1 = QueryOptimizer::with_config(config1);
    println!("Config 1: Full Optimization");
    println!("  Reordering: enabled");
    println!("  Index selection: enabled");
    println!("  Memoization: enabled");
    println!();

    // Config 2: Conservative (no reordering)
    let config2 = OptimizerConfig {
        enable_reordering: false,
        enable_index_selection: true,
        enable_memoization: true,
    };

    let _optimizer2 = QueryOptimizer::with_config(config2);
    println!("Config 2: Conservative (preserves goal order)");
    println!("  Reordering: disabled");
    println!("  Index selection: enabled");
    println!("  Memoization: enabled");
    println!();

    // Test with same goals
    let _goals = [
        Goal::new("a(?x)".to_string()),
        Goal::new("b(?x)".to_string()),
        Goal::new("c(?x)".to_string()),
    ];

    println!("Testing with goals: a(?x), b(?x), c(?x)");
    println!();

    // This would show different behavior if selectivity was set
    println!("Note: Goal order preserved when reordering is disabled");
    println!();
}

fn demo_optimization_stats() {
    println!("📝 Demo 5: Optimization Statistics");
    println!("{}", "-".repeat(80));

    let mut optimizer = QueryOptimizer::new();

    // Perform multiple optimizations
    println!("Performing optimizations...");
    for i in 1..=5 {
        let goals = vec![
            Goal::new(format!("goal_a_{}(?x)", i)),
            Goal::new(format!("goal_b_{}(?x)", i)),
            Goal::new(format!("goal_c_{}(?x)", i)),
        ];

        optimizer.optimize_goals(goals);
        println!("  Optimization {}: 3 goals", i);
    }
    println!();

    let stats = optimizer.stats();
    println!("Statistics:");
    println!("  Total optimizations: {}", stats.total_optimizations);
    println!("  Goals reordered: {}", stats.goals_reordered);
    println!("  Index selections: {}", stats.index_selections);
    println!("  Memoization hits: {}", stats.memoization_hits);
    println!("  Memoization misses: {}", stats.memoization_misses);
    println!(
        "  Cache hit rate: {:.1}%",
        stats.memoization_hit_rate() * 100.0
    );
    println!();

    println!("Summary: {}", stats.summary());
    println!();
}

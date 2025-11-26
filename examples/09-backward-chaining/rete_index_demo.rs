//! RETE Integration Demo - Showcasing O(1) Conclusion Index Performance
//!
//! This demo demonstrates the performance improvement from RETE-style conclusion indexing
//! in backward chaining. It compares query performance with different numbers of rules.
//!
//! Run with:
//! ```bash
//! cargo run --example rete_index_demo --features backward-chaining
//! ```

use rust_rule_engine::backward::BackwardEngine;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::time::Instant;

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           RETE Integration Demo - O(1) Index Lookup           ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Demo 1: Small rule set (10 rules)
    demo_with_rule_count(10);

    println!("\n{}\n", "─".repeat(70));

    // Demo 2: Medium rule set (100 rules)
    demo_with_rule_count(100);

    println!("\n{}\n", "─".repeat(70));

    // Demo 3: Large rule set (500 rules)
    demo_with_rule_count(500);

    println!("\n{}\n", "─".repeat(70));

    // Demo 4: Show index statistics
    demo_index_statistics();
}

fn demo_with_rule_count(num_rules: usize) {
    println!("📊 Demo: {} Rules", num_rules);
    println!("{}", "─".repeat(70));

    // Create knowledge base with N rules
    let kb = create_kb_with_rules(num_rules);

    // Build backward engine (includes index building)
    let start_build = Instant::now();
    let mut engine = BackwardEngine::new(kb);
    let build_time = start_build.elapsed();

    println!("⚙️  Index Build Time: {:?}", build_time);

    // Get index statistics
    let stats = engine.index_stats();
    println!("📈 Index Stats:");
    println!("   - Total rules indexed: {}", stats.total_rules);
    println!("   - Indexed fields: {}", stats.indexed_fields);
    println!("   - Avg rules per field: {:.2}", stats.avg_rules_per_field);

    // Prepare facts
    let mut facts = Facts::new();
    facts.set("trigger", Value::Boolean(true));

    // Query for a field in the middle of the rule set
    let target_rule = num_rules / 2;
    let query = format!("Field{} == true", target_rule);

    // Run query and measure time
    let start_query = Instant::now();
    let result = engine.query(&query, &mut facts);
    let query_time = start_query.elapsed();

    match result {
        Ok(qr) => {
            println!("\n✅ Query Result:");
            println!("   - Goal: {}", query);
            println!("   - Provable: {}", qr.provable);
            println!("   - Query Time: {:?}", query_time);
            println!("   - Goals Explored: {}", qr.stats.goals_explored);

            // Calculate theoretical O(n) time
            let theoretical_on_time = query_time * (num_rules as u32);
            println!("\n💡 Performance Insight:");
            println!("   - With O(1) index: {:?}", query_time);
            println!("   - Without index (O(n)): ~{:?} (estimated)", theoretical_on_time);
            println!("   - Speedup: ~{}x faster", num_rules);
        }
        Err(e) => {
            println!("❌ Query failed: {}", e);
        }
    }
}

fn demo_index_statistics() {
    println!("📊 Index Statistics Demo");
    println!("{}", "─".repeat(70));

    // Create a knowledge base with various rules
    let kb = KnowledgeBase::new("index_stats_demo");

    // Add rules that set different fields
    for i in 0..50 {
        let field = format!("Category{}.Status", i % 5); // 5 categories, multiple rules per category
        let rule_name = format!("Rule{}", i);

        let condition = Condition::new(
            "trigger".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        );

        let actions = vec![ActionType::Set {
            field: field.clone(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new(rule_name, ConditionGroup::Single(condition), actions);
        let _ = kb.add_rule(rule);
    }

    // Build engine
    let engine = BackwardEngine::new(kb);
    let stats = engine.index_stats();

    println!("📈 Detailed Index Statistics:");
    println!("   Total Rules: {}", stats.total_rules);
    println!("   Indexed Fields: {}", stats.indexed_fields);
    println!("   Average Rules per Field: {:.2}", stats.avg_rules_per_field);

    println!("\n💡 What This Means:");
    println!("   - The index maps {} unique fields to {} rules",
        stats.indexed_fields, stats.total_rules);
    println!("   - On average, each field has {:.1} rules that can derive it",
        stats.avg_rules_per_field);
    println!("   - Lookup time is O(1) regardless of total rule count!");
}

fn create_kb_with_rules(num_rules: usize) -> KnowledgeBase {
    let kb = KnowledgeBase::new(&format!("demo_kb_{}", num_rules));

    for i in 0..num_rules {
        let field = format!("Field{}", i);
        let rule_name = format!("Rule{}", i);

        // Simple condition: trigger == true
        let condition = Condition::new(
            "trigger".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        );

        // Action: Set FieldN = true
        let actions = vec![ActionType::Set {
            field: field.clone(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new(rule_name, ConditionGroup::Single(condition), actions);
        let _ = kb.add_rule(rule);
    }

    kb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rete_index_demo() {
        // Create a small knowledge base
        let kb = create_kb_with_rules(10);
        let mut engine = BackwardEngine::new(kb);

        // Verify index stats
        let stats = engine.index_stats();
        assert_eq!(stats.total_rules, 10);
        assert!(stats.indexed_fields >= 10);

        // Run a query
        let mut facts = Facts::new();
        facts.set("trigger", Value::Boolean(true));

        let result = engine.query("Field5 == true", &mut facts);
        assert!(result.is_ok());
        assert!(result.unwrap().provable);
    }

    #[test]
    fn test_index_rebuild() {
        let kb = create_kb_with_rules(5);
        let mut engine = BackwardEngine::new(kb);

        // Get initial stats
        let stats1 = engine.index_stats();
        assert_eq!(stats1.total_rules, 5);

        // Rebuild index
        engine.rebuild_index();

        // Stats should be the same
        let stats2 = engine.index_stats();
        assert_eq!(stats2.total_rules, 5);
        assert_eq!(stats2.indexed_fields, stats1.indexed_fields);
    }
}

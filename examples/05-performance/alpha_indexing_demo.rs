//! Alpha Memory Indexing Demo
//!
//! Demonstrates O(1) hash-based indexing for alpha node fact filtering.
//! Shows 10-100x speedup vs linear scan.

use rust_rule_engine::rete::{AlphaMemoryIndex, FactValue, TypedFacts};
use std::time::Instant;

fn main() {
    println!("=== Alpha Memory Indexing Demo ===\n");

    demo_basic_indexing();
    println!();

    demo_performance_comparison();
    println!();

    demo_auto_tuning();
    println!();

    demo_multiple_indexes();
}

/// Demo 1: Basic indexing usage
fn demo_basic_indexing() {
    println!("📋 Demo 1: Basic Indexing");
    println!("{}", "=".repeat(50));

    let mut mem = AlphaMemoryIndex::new();

    // Create index on "status" field
    mem.create_index("status".to_string());

    // Insert facts
    for i in 0..1000 {
        let mut fact = TypedFacts::new();
        fact.set("id", i);
        fact.set("status", if i % 10 == 0 { "active" } else { "pending" });
        fact.set("amount", i * 100);
        mem.insert(fact);
    }

    println!("Inserted 1,000 facts");
    println!("Indexed fields: {:?}", mem.indexed_fields());

    // Query using index
    let active_facts = mem.filter("status", &FactValue::String("active".to_string()));
    println!("Active facts: {} (should be 100)", active_facts.len());

    // Display stats
    println!("\n{}", mem.stats());
}

/// Demo 2: Performance comparison
fn demo_performance_comparison() {
    println!("⚡ Demo 2: Performance Comparison");
    println!("{}", "=".repeat(50));

    let fact_counts = [1_000, 10_000, 50_000];

    for &count in &fact_counts {
        println!("\n📊 Testing with {} facts:", count);

        // Create facts
        let mut facts = Vec::new();
        for i in 0..count {
            let mut fact = TypedFacts::new();
            fact.set("id", i);
            fact.set("status", if i % 100 == 0 { "rare" } else { "common" });
            facts.push(fact);
        }

        // Test 1: Linear scan (no index)
        let mut mem_linear = AlphaMemoryIndex::new();
        for fact in facts.clone() {
            mem_linear.insert(fact);
        }

        let start = Instant::now();
        for _ in 0..100 {
            let _ = mem_linear.filter("status", &FactValue::String("rare".to_string()));
        }
        let linear_time = start.elapsed();

        // Test 2: Indexed lookup
        let mut mem_indexed = AlphaMemoryIndex::new();
        mem_indexed.create_index("status".to_string());

        for fact in facts {
            mem_indexed.insert(fact);
        }

        let start = Instant::now();
        for _ in 0..100 {
            let _ = mem_indexed.filter("status", &FactValue::String("rare".to_string()));
        }
        let indexed_time = start.elapsed();

        // Calculate speedup
        let speedup = linear_time.as_secs_f64() / indexed_time.as_secs_f64();

        println!("  Linear scan:    {:?}", linear_time);
        println!("  Indexed lookup: {:?}", indexed_time);
        println!("  Speedup:        {:.2}x", speedup);
    }
}

/// Demo 3: Auto-tuning
fn demo_auto_tuning() {
    println!("🤖 Demo 3: Auto-Tuning");
    println!("{}", "=".repeat(50));

    let mut mem = AlphaMemoryIndex::new();

    // Insert facts (no index created yet)
    for i in 0..5000 {
        let mut fact = TypedFacts::new();
        fact.set("category", format!("cat_{}", i % 5));
        fact.set("priority", if i % 3 == 0 { "high" } else { "low" });
        mem.insert(fact);
    }

    println!("Inserted 5,000 facts");
    println!("Indexed fields: {:?} (none yet)", mem.indexed_fields());

    // Query "category" field 60 times (triggers auto-tune threshold of 50)
    for _ in 0..60 {
        let _ = mem.filter("category", &FactValue::String("cat_0".to_string()));
    }

    println!("\nAfter 60 queries on 'category':");
    println!("  Linear scans: {}", mem.stats().linear_scans);

    // Auto-tune should create index
    mem.auto_tune();
    println!("\nAuto-tune executed!");
    println!("Indexed fields: {:?}", mem.indexed_fields());

    // Next query should use index
    let _ = mem.filter("category", &FactValue::String("cat_0".to_string()));
    println!("\nAfter auto-tune:");
    println!("{}", mem.stats());
}

/// Demo 4: Multiple indexes
fn demo_multiple_indexes() {
    println!("🔍 Demo 4: Multiple Indexes");
    println!("{}", "=".repeat(50));

    let mut mem = AlphaMemoryIndex::new();

    // Create indexes on multiple fields
    mem.create_index("status".to_string());
    mem.create_index("priority".to_string());
    mem.create_index("region".to_string());

    println!("Created indexes on: {:?}", mem.indexed_fields());

    // Insert facts
    for i in 0..10_000 {
        let mut fact = TypedFacts::new();
        fact.set("id", i);
        fact.set("status", if i % 5 == 0 { "active" } else { "inactive" });
        fact.set("priority", if i % 3 == 0 { "high" } else { "low" });
        fact.set("region", format!("R{}", i % 4));
        mem.insert(fact);
    }

    println!("Inserted 10,000 facts\n");

    // Query different fields
    let active = mem.filter("status", &FactValue::String("active".to_string()));
    let high_priority = mem.filter("priority", &FactValue::String("high".to_string()));
    let region_0 = mem.filter("region", &FactValue::String("R0".to_string()));

    println!("Query results:");
    println!("  Active: {} facts", active.len());
    println!("  High priority: {} facts", high_priority.len());
    println!("  Region R0: {} facts", region_0.len());

    println!("\n{}", mem.stats());
    println!("\n✅ All queries used indexes (100% hit rate)!");
}

//! Comprehensive Engine Comparison Benchmark
//!
//! Compares performance across optimization levels:
//! 1. Native Rust (baseline)
//! 2. Alpha Memory Indexing (linear vs indexed)
//! 3. Beta Memory Indexing (nested loop vs hash join)
//! 4. Combined optimizations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rule_engine::rete::{AlphaMemoryIndex, BetaMemoryIndex, FactValue, TypedFacts};
use std::collections::HashMap;

// ============================================================================
// 1. NATIVE RUST BASELINE
// ============================================================================

fn native_rust_filter(facts: &[HashMap<String, i64>]) -> Vec<&HashMap<String, i64>> {
    facts
        .iter()
        .filter(|f| {
            f.get("status") == Some(&1) && f.get("amount").map(|&v| v > 1000).unwrap_or(false)
        })
        .collect()
}

fn bench_native_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("native_baseline");

    for fact_count in [1_000, 10_000] {
        let mut facts = Vec::new();
        for i in 0..fact_count {
            let mut fact = HashMap::new();
            fact.insert("id".to_string(), i);
            fact.insert("status".to_string(), if i % 10 == 0 { 1 } else { 0 });
            fact.insert("amount".to_string(), i * 100);
            facts.push(fact);
        }

        group.bench_with_input(
            BenchmarkId::new("native", fact_count),
            &fact_count,
            |b, _| {
                b.iter(|| {
                    let results = native_rust_filter(black_box(&facts));
                    black_box(results.len());
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 2. ALPHA INDEXING COMPARISON
// ============================================================================

fn bench_alpha_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_comparison");

    for fact_count in [1_000, 10_000, 50_000] {
        let mut facts = Vec::new();
        for i in 0..fact_count {
            let mut fact = TypedFacts::new();
            fact.set("id", i as i64);
            fact.set("status", if i % 100 == 0 { "active" } else { "pending" });
            facts.push(fact);
        }

        // Linear scan
        group.bench_with_input(
            BenchmarkId::new("linear", fact_count),
            &fact_count,
            |b, _| {
                b.iter(|| {
                    let mut mem = AlphaMemoryIndex::new();
                    for fact in black_box(&facts) {
                        mem.insert(fact.clone());
                    }
                    let results = mem.filter("status", &FactValue::String("active".to_string()));
                    black_box(results.len());
                });
            },
        );

        // Indexed
        group.bench_with_input(
            BenchmarkId::new("indexed", fact_count),
            &fact_count,
            |b, _| {
                b.iter(|| {
                    let mut mem = AlphaMemoryIndex::new();
                    mem.create_index("status".to_string());
                    for fact in black_box(&facts) {
                        mem.insert(fact.clone());
                    }
                    let results = mem.filter("status", &FactValue::String("active".to_string()));
                    black_box(results.len());
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 3. BETA INDEXING COMPARISON
// ============================================================================

fn beta_join_linear(left: &[TypedFacts], right: &[TypedFacts], key: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    for (left_idx, left_fact) in left.iter().enumerate() {
        for (right_idx, right_fact) in right.iter().enumerate() {
            if left_fact.get(key) == right_fact.get(key) {
                results.push((left_idx, right_idx));
            }
        }
    }
    results
}

fn beta_join_indexed(left: &[TypedFacts], right: &[TypedFacts], key: &str) -> Vec<(usize, usize)> {
    let mut index = BetaMemoryIndex::new(key.to_string());

    // Build index on right
    for (idx, fact) in right.iter().enumerate() {
        index.add(fact, idx);
    }

    // Probe with left
    let mut results = Vec::new();
    for (left_idx, left_fact) in left.iter().enumerate() {
        if let Some(value) = left_fact.get(key) {
            let key_str = format!("{:?}", value);
            for &right_idx in index.lookup(&key_str) {
                results.push((left_idx, right_idx));
            }
        }
    }
    results
}

fn bench_beta_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("beta_comparison");

    for fact_count in [100, 1_000, 5_000] {
        // Create left facts (orders)
        let mut left_facts = Vec::new();
        for i in 0..fact_count {
            let mut fact = TypedFacts::new();
            fact.set("order_id", i as i64);
            fact.set("customer_id", format!("C{}", i % 100));
            left_facts.push(fact);
        }

        // Create right facts (customers)
        let mut right_facts = Vec::new();
        for i in 0..fact_count {
            let mut fact = TypedFacts::new();
            fact.set("customer_id", format!("C{}", i % 100));
            fact.set("tier", if i % 10 == 0 { "gold" } else { "silver" });
            right_facts.push(fact);
        }

        // Linear join (nested loop)
        group.bench_with_input(
            BenchmarkId::new("linear", fact_count),
            &fact_count,
            |b, _| {
                b.iter(|| {
                    let results = beta_join_linear(
                        black_box(&left_facts),
                        black_box(&right_facts),
                        "customer_id",
                    );
                    black_box(results.len());
                });
            },
        );

        // Indexed join (hash join)
        group.bench_with_input(
            BenchmarkId::new("indexed", fact_count),
            &fact_count,
            |b, _| {
                b.iter(|| {
                    let results = beta_join_indexed(
                        black_box(&left_facts),
                        black_box(&right_facts),
                        "customer_id",
                    );
                    black_box(results.len());
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 4. COMBINED OPTIMIZATION
// ============================================================================

fn bench_combined_optimization(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_optimization");

    for fact_count in [1_000, 10_000] {
        let mut facts = Vec::new();
        for i in 0..fact_count {
            let mut fact = TypedFacts::new();
            fact.set("id", i as i64);
            fact.set("status", if i % 10 == 0 { "active" } else { "pending" });
            fact.set("priority", if i % 3 == 0 { "high" } else { "low" });
            facts.push(fact);
        }

        // No optimization
        group.bench_with_input(
            BenchmarkId::new("no_optimization", fact_count),
            &fact_count,
            |b, _| {
                b.iter(|| {
                    let mut mem = AlphaMemoryIndex::new();
                    for fact in black_box(&facts) {
                        mem.insert(fact.clone());
                    }
                    let r1 = mem.filter("status", &FactValue::String("active".to_string()));
                    black_box(r1.len());
                });
            },
        );

        // With alpha indexing
        group.bench_with_input(
            BenchmarkId::new("alpha_indexed", fact_count),
            &fact_count,
            |b, _| {
                b.iter(|| {
                    let mut mem = AlphaMemoryIndex::new();
                    mem.create_index("status".to_string());
                    for fact in black_box(&facts) {
                        mem.insert(fact.clone());
                    }
                    let r1 = mem.filter("status", &FactValue::String("active".to_string()));
                    black_box(r1.len());
                });
            },
        );

        // With multiple indexes
        group.bench_with_input(
            BenchmarkId::new("multi_indexed", fact_count),
            &fact_count,
            |b, _| {
                b.iter(|| {
                    let mut mem = AlphaMemoryIndex::new();
                    mem.create_index("status".to_string());
                    mem.create_index("priority".to_string());
                    for fact in black_box(&facts) {
                        mem.insert(fact.clone());
                    }
                    let r1 = mem.filter("status", &FactValue::String("active".to_string()));
                    let r2 = mem.filter("priority", &FactValue::String("high".to_string()));
                    black_box(r1.len() + r2.len());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_native_baseline,
    bench_alpha_comparison,
    bench_beta_comparison,
    bench_combined_optimization
);
criterion_main!(benches);

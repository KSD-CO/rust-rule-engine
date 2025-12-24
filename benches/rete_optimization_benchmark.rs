//! RETE Optimization Benchmarks
//!
//! Compares performance before and after optimizations:
//! - Node sharing
//! - Alpha memory compaction
//! - Beta memory indexing
//! - Token pooling

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rule_engine::rete::optimization::{
    BetaMemoryIndex, CompactAlphaMemory, NodeSharingRegistry, TokenPool,
};
use rust_rule_engine::rete::{AlphaNode, TypedFacts};
use std::time::Duration;

// ===========================================================================
// 1. NODE SHARING BENCHMARKS - SCALED
// ===========================================================================

fn bench_node_sharing(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_sharing");

    // Test with different scales to find the tipping point
    for size in [100, 1000, 10000].iter() {
        let nodes: Vec<AlphaNode> = (0..*size)
            .map(|i| AlphaNode {
                field: format!("field{}", i % 100), // 100 unique patterns
                operator: ">".to_string(),
                value: "50".to_string(),
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("without_sharing", size), size, |b, _| {
            b.iter(|| {
                let mut stored_nodes = Vec::new();
                for node in &nodes {
                    stored_nodes.push(node.clone());
                }
                black_box(stored_nodes.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("with_sharing", size), size, |b, _| {
            b.iter(|| {
                let mut registry = NodeSharingRegistry::new();
                for (idx, node) in nodes.iter().enumerate() {
                    registry.register(node, idx);
                }
                black_box(registry.stats())
            });
        });
    }

    group.finish();
}

// ===========================================================================
// 2. ALPHA MEMORY COMPACTION BENCHMARKS - SCALED
// ===========================================================================

fn bench_alpha_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_memory");

    // Test with different scales
    for size in [100, 1000, 10000].iter() {
        let facts: Vec<TypedFacts> = (0..*size)
            .map(|i| {
                let mut fact = TypedFacts::new();
                fact.set("id", (i % 100) as i64); // 100 unique facts
                fact.set("value", (i % 100) as i64);
                fact
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("vec_storage", size), size, |b, _| {
            b.iter(|| {
                let mut vec_memory: Vec<TypedFacts> = Vec::new();
                for fact in &facts {
                    vec_memory.push(fact.clone());
                }
                black_box(vec_memory.len())
            });
        });

        group.bench_with_input(BenchmarkId::new("compact_storage", size), size, |b, _| {
            b.iter(|| {
                let mut compact_memory = CompactAlphaMemory::new();
                for fact in &facts {
                    compact_memory.add(fact);
                }
                black_box(compact_memory.len())
            });
        });
    }

    group.finish();
}

// ===========================================================================
// 3. BETA MEMORY INDEXING BENCHMARKS - SCALED (Most Important!)
// ===========================================================================

fn bench_beta_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("beta_indexing");
    group.sample_size(10); // Reduce sample size for large datasets

    // Test with different scales - THIS IS WHERE OPTIMIZATION SHINES!
    for size in [100, 1000, 5000].iter() {
        let left_facts: Vec<TypedFacts> = (0..*size)
            .map(|i| {
                let mut fact = TypedFacts::new();
                fact.set("user_id", format!("user{}", i % 100));
                fact.set("order_id", i as i64);
                fact
            })
            .collect();

        let right_facts: Vec<TypedFacts> = (0..*size)
            .map(|i| {
                let mut fact = TypedFacts::new();
                fact.set("user_id", format!("user{}", i % 100));
                fact.set("action", "purchase");
                fact
            })
            .collect();

        group.bench_with_input(BenchmarkId::new("nested_loop_join", size), size, |b, _| {
            b.iter(|| {
                let mut matches = 0;
                for left in &left_facts {
                    for right in &right_facts {
                        if left.get("user_id") == right.get("user_id") {
                            matches += 1;
                        }
                    }
                }
                black_box(matches)
            });
        });

        group.bench_with_input(BenchmarkId::new("indexed_join", size), size, |b, _| {
            b.iter(|| {
                let mut index = BetaMemoryIndex::new("user_id".to_string());

                for (i, right) in right_facts.iter().enumerate() {
                    index.add(right, i);
                }

                let mut matches = 0;
                for left in &left_facts {
                    if let Some(user_id) = left.get("user_id") {
                        let key = format!("{:?}", user_id);
                        matches += index.lookup(&key).len();
                    }
                }
                black_box(matches)
            });
        });
    }

    group.finish();
}

// ===========================================================================
// 4. TOKEN POOLING BENCHMARKS - SCALED
// ===========================================================================

fn bench_token_pooling(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_pooling");

    // Test with different iteration counts - pooling shines with high volume
    for iterations in [1000, 10000, 100000].iter() {
        group.bench_with_input(
            BenchmarkId::new("without_pooling", iterations),
            iterations,
            |b, &iters| {
                b.iter(|| {
                    // Create new tokens each time
                    for _ in 0..iters {
                        let fact = TypedFacts::new();
                        black_box(fact);
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_pooling", iterations),
            iterations,
            |b, &iters| {
                b.iter(|| {
                    let mut pool = TokenPool::new(1000);

                    for _ in 0..iters {
                        let mut token = pool.acquire();
                        let fact = TypedFacts::new();
                        token.set_fact(fact);
                        black_box(&token);
                        pool.release(token);
                    }
                });
            },
        );
    }

    group.finish();
}

// ===========================================================================
// 5. END-TO-END BENCHMARK (All Optimizations Combined)
// ===========================================================================

fn bench_end_to_end(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end");
    group.measurement_time(Duration::from_secs(10));

    let num_rules = 1000;
    let num_facts = 10000;

    group.bench_with_input(
        BenchmarkId::new("rules", num_rules),
        &(num_rules, num_facts),
        |b, &(rules, facts)| {
            b.iter(|| {
                // Simulate full RETE cycle with optimizations
                let mut registry = NodeSharingRegistry::new();
                let mut memory = CompactAlphaMemory::new();
                let mut pool = TokenPool::new(1000);

                // Create rules (with node sharing)
                for i in 0..rules {
                    let node = AlphaNode {
                        field: format!("field{}", i % 100),
                        operator: ">".to_string(),
                        value: "50".to_string(),
                    };
                    registry.register(&node, i);
                }

                // Insert facts (with compaction and pooling)
                for i in 0..facts {
                    let fact = TypedFacts::new();
                    let mut fact_with_data = fact.clone();
                    fact_with_data.set("id", i as i64);
                    fact_with_data.set("value", (i % 100) as i64);

                    memory.add(&fact_with_data);

                    let token = pool.acquire_with_fact(fact_with_data);
                    pool.release(token);
                }

                black_box((registry.stats(), memory.len(), pool.stats()))
            });
        },
    );

    group.finish();
}

// ===========================================================================
// CRITERION CONFIGURATION
// ===========================================================================

criterion_group!(
    benches,
    bench_node_sharing,
    bench_alpha_memory,
    bench_beta_indexing,
    bench_token_pooling,
    bench_end_to_end
);

criterion_main!(benches);

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rule_engine::rete::{AlphaMemoryIndex, FactValue, TypedFacts};

/// Benchmark: Linear scan vs indexed lookup for different fact counts
fn bench_alpha_indexing(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_indexing");

    for fact_count in [100, 1_000, 10_000] {
        // Create facts
        let mut facts = Vec::new();
        for i in 0..fact_count {
            let mut fact = TypedFacts::new();
            fact.set("id", i);
            fact.set("status", if i % 10 == 0 { "active" } else { "pending" });
            fact.set("category", format!("cat_{}", i % 5));
            fact.set("priority", if i % 3 == 0 { "high" } else { "low" });
            facts.push(fact);
        }

        // Benchmark: Linear scan (no index)
        group.bench_with_input(
            BenchmarkId::new("linear_scan", fact_count),
            &fact_count,
            |b, _| {
                let mut mem = AlphaMemoryIndex::new();
                for fact in facts.clone() {
                    mem.insert(fact);
                }

                b.iter(|| {
                    let results = mem.filter(
                        "status",
                        black_box(&FactValue::String("active".to_string())),
                    );
                    black_box(results.len());
                });
            },
        );

        // Benchmark: Indexed lookup
        group.bench_with_input(
            BenchmarkId::new("indexed_lookup", fact_count),
            &fact_count,
            |b, _| {
                let mut mem = AlphaMemoryIndex::new();
                mem.create_index("status".to_string());

                for fact in facts.clone() {
                    mem.insert(fact);
                }

                b.iter(|| {
                    let results = mem.filter(
                        "status",
                        black_box(&FactValue::String("active".to_string())),
                    );
                    black_box(results.len());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Insert performance with and without indexes
fn bench_insert_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_insert");

    // Benchmark: Insert without index
    group.bench_function("no_index", |b| {
        b.iter(|| {
            let mut mem = AlphaMemoryIndex::new();

            for i in 0..1000 {
                let mut fact = TypedFacts::new();
                fact.set("id", i);
                fact.set("status", if i % 10 == 0 { "active" } else { "pending" });
                mem.insert(black_box(fact));
            }
        });
    });

    // Benchmark: Insert with single index
    group.bench_function("single_index", |b| {
        b.iter(|| {
            let mut mem = AlphaMemoryIndex::new();
            mem.create_index("status".to_string());

            for i in 0..1000 {
                let mut fact = TypedFacts::new();
                fact.set("id", i);
                fact.set("status", if i % 10 == 0 { "active" } else { "pending" });
                mem.insert(black_box(fact));
            }
        });
    });

    // Benchmark: Insert with multiple indexes
    group.bench_function("multiple_indexes", |b| {
        b.iter(|| {
            let mut mem = AlphaMemoryIndex::new();
            mem.create_index("status".to_string());
            mem.create_index("priority".to_string());
            mem.create_index("category".to_string());

            for i in 0..1000 {
                let mut fact = TypedFacts::new();
                fact.set("id", i);
                fact.set("status", if i % 10 == 0 { "active" } else { "pending" });
                fact.set("priority", if i % 3 == 0 { "high" } else { "low" });
                fact.set("category", format!("cat_{}", i % 5));
                mem.insert(black_box(fact));
            }
        });
    });

    group.finish();
}

/// Benchmark: High selectivity (1% match) - best case for indexing
fn bench_high_selectivity(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_selectivity");

    let fact_count = 10_000;
    let mut facts = Vec::new();

    for i in 0..fact_count {
        let mut fact = TypedFacts::new();
        fact.set("id", i);
        // Only 1% are "rare"
        fact.set("status", if i % 100 == 0 { "rare" } else { "common" });
        facts.push(fact);
    }

    // Linear scan
    group.bench_function("linear_1pct", |b| {
        let mut mem = AlphaMemoryIndex::new();
        for fact in facts.clone() {
            mem.insert(fact);
        }

        b.iter(|| {
            let results = mem.filter("status", black_box(&FactValue::String("rare".to_string())));
            black_box(results.len());
        });
    });

    // Indexed
    group.bench_function("indexed_1pct", |b| {
        let mut mem = AlphaMemoryIndex::new();
        mem.create_index("status".to_string());

        for fact in facts.clone() {
            mem.insert(fact);
        }

        b.iter(|| {
            let results = mem.filter("status", black_box(&FactValue::String("rare".to_string())));
            black_box(results.len());
        });
    });

    group.finish();
}

/// Benchmark: Auto-tuning overhead
fn bench_auto_tuning(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_auto_tune");

    group.bench_function("auto_tune_10k_facts", |b| {
        b.iter(|| {
            let mut mem = AlphaMemoryIndex::new();

            // Insert 10k facts
            for i in 0..10_000 {
                let mut fact = TypedFacts::new();
                fact.set("status", if i % 5 == 0 { "A" } else { "B" });
                fact.set("priority", if i % 3 == 0 { "high" } else { "low" });
                mem.insert(fact);
            }

            // Query 60 times to trigger auto-tune threshold
            for _ in 0..60 {
                let _ = mem.filter("status", &FactValue::String("A".to_string()));
            }

            // Auto-tune
            mem.auto_tune();
            black_box(&mem);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_alpha_indexing,
    bench_insert_performance,
    bench_high_selectivity,
    bench_auto_tuning
);
criterion_main!(benches);

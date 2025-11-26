//! Benchmark comparing O(n) vs O(1) rule lookup in backward chaining
//!
//! This benchmark measures the performance improvement from RETE-style conclusion index.
//!
//! Run with:
//! ```bash
//! cargo bench --features backward-chaining --bench backward_chaining_index_benchmark
//! ```

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rule_engine::backward::{BackwardEngine, ConclusionIndex};
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::types::{ActionType, Operator, Value};

/// Create a knowledge base with N rules
fn create_kb_with_rules(num_rules: usize) -> KnowledgeBase {
    let kb = KnowledgeBase::new("benchmark_kb");

    for i in 0..num_rules {
        let field = format!("Field{}", i);
        let rule_name = format!("Rule{}", i);

        let conditions = ConditionGroup::Single(Condition::new(
            "dummy".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ));

        let actions = vec![ActionType::Set {
            field: field.clone(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new(rule_name, conditions, actions);
        let _ = kb.add_rule(rule);
    }

    kb
}

/// Benchmark conclusion index creation
fn bench_index_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Index Creation");

    for num_rules in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_rules),
            num_rules,
            |b, &num_rules| {
                let kb = create_kb_with_rules(num_rules);
                let rules = kb.get_rules();

                b.iter(|| {
                    let index = ConclusionIndex::from_rules(&rules);
                    black_box(index);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark rule candidate lookup
fn bench_candidate_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("Candidate Lookup");

    for num_rules in [10, 50, 100, 500, 1000].iter() {
        let kb = create_kb_with_rules(*num_rules);
        let rules = kb.get_rules();
        let index = ConclusionIndex::from_rules(&rules);

        // Lookup a field in the middle to avoid cache effects
        let target_field = format!("Field{}", num_rules / 2);
        let query = format!("{} == true", target_field);

        group.bench_with_input(
            BenchmarkId::from_parameter(num_rules),
            num_rules,
            |b, _| {
                b.iter(|| {
                    let candidates = index.find_candidates(&query);
                    black_box(candidates);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark full backward chaining query with index
fn bench_query_with_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("Query with Index");

    for num_rules in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_rules),
            num_rules,
            |b, &num_rules| {
                let kb = create_kb_with_rules(num_rules);
                let mut engine = BackwardEngine::new(kb);
                let mut facts = Facts::new();
                facts.set("dummy", Value::Boolean(true));

                // Query a field in the middle
                let target_field = format!("Field{}", num_rules / 2);
                let query = format!("{} == true", target_field);

                b.iter(|| {
                    let result = engine.query(&query, &mut facts);
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark worst case: query for non-existent field
fn bench_query_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("Query Miss (No Match)");

    for num_rules in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_rules),
            num_rules,
            |b, &num_rules| {
                let kb = create_kb_with_rules(num_rules);
                let mut engine = BackwardEngine::new(kb);
                let mut facts = Facts::new();

                // Query for non-existent field
                let query = "NonExistentField == true";

                b.iter(|| {
                    let result = engine.query(query, &mut facts);
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark multiple queries in sequence
fn bench_multiple_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("Multiple Queries");

    for num_rules in [50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_rules),
            num_rules,
            |b, &num_rules| {
                let kb = create_kb_with_rules(num_rules);
                let mut engine = BackwardEngine::new(kb);
                let mut facts = Facts::new();
                facts.set("dummy", Value::Boolean(true));

                // Create 10 different queries
                let queries: Vec<String> = (0..10)
                    .map(|i| {
                        let idx = (num_rules / 10) * i;
                        format!("Field{} == true", idx)
                    })
                    .collect();

                b.iter(|| {
                    for query in &queries {
                        let result = engine.query(query, &mut facts);
                        black_box(result);
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_index_creation,
    bench_candidate_lookup,
    bench_query_with_index,
    bench_query_miss,
    bench_multiple_queries
);
criterion_main!(benches);

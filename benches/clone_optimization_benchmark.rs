//! Microbenchmarks for clone optimization hot paths
//!
//! Measures the specific operations optimized in the clone-reduction PR:
//! 1. Value::as_string_ref() vs as_string() in Operator::evaluate()
//! 2. FactValue::as_str() vs as_string() in RETE comparison/hashing
//! 3. KnowledgeBase: index-based iteration vs get_rules().clone()
//! 4. KnowledgeBase: rule_count() vs get_rules().len()
//! 5. KnowledgeBase: get_rule() vs get_rules().iter().find()
//! 6. Facts::with_value() vs Facts::get()

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rule_engine::rete::{FactValue, TypedFacts};
use rust_rule_engine::types::{Operator, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::hint::black_box;

// ============================================================================
// 1. Value::as_string_ref() vs as_string() — string operator evaluation
// ============================================================================

fn bench_value_string_operators(c: &mut Criterion) {
    let mut group = c.benchmark_group("value_string_ops");

    for count in [100, 1_000, 10_000] {
        // Pre-build pairs of string Values
        let pairs: Vec<(Value, Value)> = (0..count)
            .map(|i| {
                (
                    Value::String(format!("hello_world_string_{}", i)),
                    Value::String(format!("world_string_{}", i % 50)),
                )
            })
            .collect();

        // Benchmark as_string() (cloning) path — simulates the old code
        group.bench_with_input(
            BenchmarkId::new("as_string_clone", count),
            &pairs,
            |b, pairs| {
                b.iter(|| {
                    let mut matches = 0u32;
                    for (left, right) in pairs {
                        if let (Some(l), Some(r)) = (left.as_string(), right.as_string()) {
                            if l.contains(&r) {
                                matches += 1;
                            }
                        }
                    }
                    black_box(matches)
                });
            },
        );

        // Benchmark as_string_ref() (zero-copy) path — the optimized code
        group.bench_with_input(
            BenchmarkId::new("as_string_ref", count),
            &pairs,
            |b, pairs| {
                b.iter(|| {
                    let mut matches = 0u32;
                    for (left, right) in pairs {
                        if let (Some(l), Some(r)) = (left.as_string_ref(), right.as_string_ref()) {
                            if l.contains(r) {
                                matches += 1;
                            }
                        }
                    }
                    black_box(matches)
                });
            },
        );
    }

    group.finish();
}

// Benchmark full Operator::evaluate() with string operators
fn bench_operator_evaluate_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("operator_evaluate_strings");

    let operators = [
        ("Contains", Operator::Contains),
        ("StartsWith", Operator::StartsWith),
        ("EndsWith", Operator::EndsWith),
        ("Matches", Operator::Matches),
    ];

    let count = 10_000;
    let pairs: Vec<(Value, Value)> = (0..count)
        .map(|i| {
            (
                Value::String(format!("the_quick_brown_fox_jumps_over_{}", i)),
                Value::String(format!("fox_jumps_over_{}", i % 100)),
            )
        })
        .collect();

    for (name, op) in &operators {
        group.bench_with_input(BenchmarkId::new(*name, count), &pairs, |b, pairs| {
            b.iter(|| {
                let mut matches = 0u32;
                for (left, right) in pairs {
                    if op.evaluate(left, right) {
                        matches += 1;
                    }
                }
                black_box(matches)
            });
        });
    }

    group.finish();
}

// ============================================================================
// 2. FactValue::as_str() vs as_string() — RETE comparison & hashing
// ============================================================================

fn bench_factvalue_string_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("factvalue_string_cmp");

    for count in [100, 1_000, 10_000] {
        let pairs: Vec<(FactValue, FactValue)> = (0..count)
            .map(|i| {
                (
                    FactValue::String(format!("fact_value_{}", i)),
                    FactValue::String(format!("fact_value_{}", i)),
                )
            })
            .collect();

        // Old path: as_string() clones
        group.bench_with_input(
            BenchmarkId::new("as_string_clone", count),
            &pairs,
            |b, pairs| {
                b.iter(|| {
                    let mut matches = 0u32;
                    for (a, b_val) in pairs {
                        if a.as_string() == b_val.as_string() {
                            matches += 1;
                        }
                    }
                    black_box(matches)
                });
            },
        );

        // New path: as_str() zero-copy Cow
        group.bench_with_input(BenchmarkId::new("as_str_cow", count), &pairs, |b, pairs| {
            b.iter(|| {
                let mut matches = 0u32;
                for (a, b_val) in pairs {
                    if a.as_str() == b_val.as_str() {
                        matches += 1;
                    }
                }
                black_box(matches)
            });
        });
    }

    group.finish();
}

fn bench_factvalue_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("factvalue_hashing");

    for count in [100, 1_000, 10_000] {
        let mut facts = TypedFacts::new();
        for i in 0..count {
            facts.set(
                format!("field_{}", i),
                FactValue::String(format!("value_{}", i)),
            );
        }

        // Old path: as_string().hash()
        group.bench_with_input(
            BenchmarkId::new("as_string_hash", count),
            &facts,
            |b, facts| {
                b.iter(|| {
                    let mut hasher = DefaultHasher::new();
                    let mut sorted: Vec<_> = facts.get_all().iter().collect();
                    sorted.sort_by_key(|(k, _)| *k);
                    for (key, value) in sorted {
                        key.hash(&mut hasher);
                        value.as_string().hash(&mut hasher);
                    }
                    black_box(hasher.finish())
                });
            },
        );

        // New path: as_str().hash()
        group.bench_with_input(
            BenchmarkId::new("as_str_hash", count),
            &facts,
            |b, facts| {
                b.iter(|| {
                    let mut hasher = DefaultHasher::new();
                    let mut sorted: Vec<_> = facts.get_all().iter().collect();
                    sorted.sort_by_key(|(k, _)| *k);
                    for (key, value) in sorted {
                        key.hash(&mut hasher);
                        value.as_str().hash(&mut hasher);
                    }
                    black_box(hasher.finish())
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 3. KnowledgeBase: index-based iteration vs get_rules().clone()
// ============================================================================

fn bench_knowledge_base_iteration(c: &mut Criterion) {
    use rust_rule_engine::KnowledgeBase;

    let mut group = c.benchmark_group("kb_iteration");

    for rule_count in [10, 50, 200] {
        let kb = KnowledgeBase::new("bench");
        for i in 0..rule_count {
            let grl = format!(
                r#"rule Rule{i} "test" salience {} {{
                    when
                        Fact.value{} > {}
                    then
                        Fact.result = "matched";
                }}"#,
                rule_count - i,
                i % 10,
                i
            );
            kb.add_rules_from_grl(&grl).unwrap();
        }

        // Old path: get_rules().clone() + sort
        group.bench_with_input(
            BenchmarkId::new("get_rules_clone_sort", rule_count),
            &kb,
            |b, kb| {
                b.iter(|| {
                    let mut rules = kb.get_rules(); // already clones
                    rules.sort_by(|a, b| b.salience.cmp(&a.salience));
                    black_box(rules.len())
                });
            },
        );

        // New path: get_rules_by_salience() (returns indices only)
        group.bench_with_input(
            BenchmarkId::new("get_rules_by_salience", rule_count),
            &kb,
            |b, kb| {
                b.iter(|| {
                    let indices = kb.get_rules_by_salience();
                    black_box(indices.len())
                });
            },
        );

        // New path: index-based iteration with individual rule access
        group.bench_with_input(
            BenchmarkId::new("index_iterate_all", rule_count),
            &kb,
            |b, kb| {
                b.iter(|| {
                    let indices = kb.get_rules_by_salience();
                    let mut count = 0;
                    for &idx in &indices {
                        if let Some(rule) = kb.get_rule_by_index(idx) {
                            black_box(&rule.name);
                            count += 1;
                        }
                    }
                    black_box(count)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 4. KnowledgeBase: rule_count() vs get_rules().len()
// ============================================================================

fn bench_knowledge_base_count(c: &mut Criterion) {
    use rust_rule_engine::KnowledgeBase;

    let mut group = c.benchmark_group("kb_rule_count");

    for rule_count in [10, 50, 200] {
        let kb = KnowledgeBase::new("bench");
        for i in 0..rule_count {
            let grl = format!(
                r#"rule CountRule{i} "test" {{
                    when
                        Fact.x > {i}
                    then
                        Fact.y = "ok";
                }}"#
            );
            kb.add_rules_from_grl(&grl).unwrap();
        }

        // Old path: get_rules().len() — clones entire Vec<Rule>
        group.bench_with_input(
            BenchmarkId::new("get_rules_len", rule_count),
            &kb,
            |b, kb| {
                b.iter(|| black_box(kb.get_rules().len()));
            },
        );

        // New path: rule_count() — reads len under lock
        group.bench_with_input(BenchmarkId::new("rule_count", rule_count), &kb, |b, kb| {
            b.iter(|| black_box(kb.rule_count()));
        });
    }

    group.finish();
}

// ============================================================================
// 5. KnowledgeBase: get_rule(name) vs get_rules().iter().find()
// ============================================================================

fn bench_knowledge_base_lookup(c: &mut Criterion) {
    use rust_rule_engine::KnowledgeBase;

    let mut group = c.benchmark_group("kb_rule_lookup");

    for rule_count in [10, 50, 200] {
        let kb = KnowledgeBase::new("bench");
        let mut rule_names = Vec::new();
        for i in 0..rule_count {
            let name = format!("LookupRule{i}");
            rule_names.push(name.clone());
            let grl = format!(
                r#"rule {name} "test" {{
                    when
                        Fact.x > {i}
                    then
                        Fact.y = "ok";
                }}"#
            );
            kb.add_rules_from_grl(&grl).unwrap();
        }

        // Pick a target name in the middle
        let target = &rule_names[rule_count / 2];

        // Old path: get_rules().iter().find() — clones ALL rules, then linear search
        group.bench_with_input(
            BenchmarkId::new("get_rules_iter_find", rule_count),
            &(kb.clone(), target.clone()),
            |b, (kb, target)| {
                b.iter(|| {
                    let found = kb.get_rules().into_iter().find(|r| r.name == *target);
                    black_box(found.is_some())
                });
            },
        );

        // New path: get_rule(name) — indexed HashMap lookup, clones single rule
        group.bench_with_input(
            BenchmarkId::new("get_rule_indexed", rule_count),
            &(kb.clone(), target.clone()),
            |b, (kb, target)| {
                b.iter(|| {
                    let found = kb.get_rule(target);
                    black_box(found.is_some())
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// 6. Facts::with_value() vs Facts::get() — callback vs clone
// ============================================================================

fn bench_facts_access(c: &mut Criterion) {
    use rust_rule_engine::Facts;

    let mut group = c.benchmark_group("facts_access");

    for fact_count in [10, 100, 1_000] {
        let facts = Facts::new();
        for i in 0..fact_count {
            facts
                .add_value(
                    &format!("field_{}", i),
                    Value::String(format!("a]_long_string_value_for_field_number_{}", i)),
                )
                .unwrap();
        }

        let target_key = format!("field_{}", fact_count / 2);

        // Old path: facts.get() — clones the Value
        group.bench_with_input(
            BenchmarkId::new("get_clone", fact_count),
            &(facts.clone(), target_key.clone()),
            |b, (facts, key): &(Facts, String)| {
                b.iter(|| {
                    let val = facts.get(key);
                    black_box(val.map(|v| v.to_bool()))
                });
            },
        );

        // New path: facts.with_value() — borrow via callback, no clone
        group.bench_with_input(
            BenchmarkId::new("with_value_callback", fact_count),
            &(facts.clone(), target_key.clone()),
            |b, (facts, key): &(Facts, String)| {
                b.iter(|| {
                    let val = facts.with_value(key, |v| v.to_bool());
                    black_box(val)
                });
            },
        );

        // Stress test: repeated lookups (simulates condition evaluation loop)
        let keys: Vec<String> = (0..fact_count).map(|i| format!("field_{}", i)).collect();

        group.bench_with_input(
            BenchmarkId::new("get_clone_loop", fact_count),
            &(facts.clone(), keys.clone()),
            |b, (facts, keys): &(Facts, Vec<String>)| {
                b.iter(|| {
                    let mut count = 0u32;
                    for key in keys {
                        if let Some(v) = facts.get(key) {
                            if v.to_bool() {
                                count += 1;
                            }
                        }
                    }
                    black_box(count)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_value_loop", fact_count),
            &(facts.clone(), keys.clone()),
            |b, (facts, keys): &(Facts, Vec<String>)| {
                b.iter(|| {
                    let mut count = 0u32;
                    for key in keys {
                        if let Some(true) = facts.with_value(key, |v| v.to_bool()) {
                            count += 1;
                        }
                    }
                    black_box(count)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// CRITERION GROUPS
// ============================================================================

criterion_group!(
    benches,
    bench_value_string_operators,
    bench_operator_evaluate_strings,
    bench_factvalue_string_comparison,
    bench_factvalue_hashing,
    bench_knowledge_base_iteration,
    bench_knowledge_base_count,
    bench_knowledge_base_lookup,
    bench_facts_access,
);
criterion_main!(benches);

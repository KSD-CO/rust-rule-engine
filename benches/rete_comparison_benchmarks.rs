use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::rete::{
    auto_network::{Condition, ConditionGroup, Rule},
    network::build_rete_ul_from_condition_group,
    FactValue, IncrementalEngine, ReteUlEngine, TypedFacts, TypedReteUlRule,
};
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;

// ============================================================================
// Setup functions for Traditional Engine
// ============================================================================

fn setup_traditional_facts(user_count: usize) -> Facts {
    let facts = Facts::new();

    for i in 0..user_count {
        let mut user = HashMap::new();
        user.insert("Age".to_string(), Value::Integer(20 + (i % 50) as i64));
        user.insert(
            "Country".to_string(),
            Value::String(
                match i % 4 {
                    0 => "US",
                    1 => "UK",
                    2 => "CA",
                    _ => "AU",
                }
                .to_string(),
            ),
        );
        user.insert(
            "SpendingTotal".to_string(),
            Value::Number(100.0 + (i as f64 * 50.0)),
        );
        user.insert("IsVIP".to_string(), Value::Boolean(i % 10 == 0));

        facts
            .add_value(&format!("User{}", i), Value::Object(user))
            .unwrap();
    }

    facts
}

fn create_traditional_engine(rule_count: usize, user_count: usize) -> (RustRuleEngine, Facts) {
    let kb = KnowledgeBase::new("TraditionalBench");

    let mut rules = String::new();
    for i in 0..rule_count {
        let user_idx = i % user_count.max(1);
        rules.push_str(&format!(
            r#"
            rule "TradRule{}" salience {} {{
                when
                    User{}.Age > {} && User{}.SpendingTotal > {}
                then
                    log("Rule {} executed");
            }}
            "#,
            i,
            100 - (i % 20),
            user_idx,
            18 + (i % 10),
            user_idx,
            200.0 + (i as f64 * 10.0),
            i
        ));
    }

    let parsed_rules = GRLParser::parse_rules(&rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 5,
        ..Default::default()
    };

    let engine = RustRuleEngine::with_config(kb, config);
    let facts = setup_traditional_facts(user_count);
    (engine, facts)
}

// ============================================================================
// Setup functions for RETE Engine
// ============================================================================

fn setup_rete_facts(user_count: usize) -> HashMap<String, String> {
    let mut facts = HashMap::new();

    for i in 0..user_count {
        facts.insert(format!("User{}.Age", i), format!("{}", 20 + (i % 50)));
        facts.insert(
            format!("User{}.Country", i),
            match i % 4 {
                0 => "US",
                1 => "UK",
                2 => "CA",
                _ => "AU",
            }
            .to_string(),
        );
        facts.insert(
            format!("User{}.SpendingTotal", i),
            format!("{}", 100.0 + (i as f64 * 50.0)),
        );
        facts.insert(format!("User{}.IsVIP", i), (i % 10 == 0).to_string());
    }

    facts
}

fn create_rete_engine(rule_count: usize, user_count: usize) -> ReteUlEngine {
    let mut engine = ReteUlEngine::new();

    for i in 0..rule_count {
        let user_idx = i % user_count.max(1);
        let age_threshold = 18 + (i % 10);
        let spending_threshold = 200.0 + (i as f64 * 10.0);

        let rule = Rule {
            name: format!("ReteRule{}", i),
            conditions: ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition {
                    field: format!("User{}.Age", user_idx),
                    operator: ">".to_string(),
                    value: format!("{}", age_threshold),
                })),
                operator: "AND".to_string(),
                right: Box::new(ConditionGroup::Single(Condition {
                    field: format!("User{}.SpendingTotal", user_idx),
                    operator: ">".to_string(),
                    value: format!("{}", spending_threshold),
                })),
            },
            action: format!("log('Rule {} executed')", i),
        };

        engine.add_rule_from_definition(&rule, (100 - (i % 20)) as i32, false);
    }

    // Set facts
    let facts = setup_rete_facts(user_count);
    for (key, value) in facts {
        engine.set_fact(key, value);
    }

    engine
}

// ============================================================================
// Setup functions for Incremental Engine
// ============================================================================

fn setup_incremental_facts(user_count: usize) -> Vec<(String, TypedFacts)> {
    let mut facts = Vec::new();

    for i in 0..user_count {
        let mut typed_facts = TypedFacts::new();
        typed_facts.set(
            "Age".to_string(),
            FactValue::Integer((20 + (i % 50)) as i64),
        );
        typed_facts.set(
            "Country".to_string(),
            FactValue::String(
                match i % 4 {
                    0 => "US",
                    1 => "UK",
                    2 => "CA",
                    _ => "AU",
                }
                .to_string(),
            ),
        );
        typed_facts.set(
            "SpendingTotal".to_string(),
            FactValue::Float(100.0 + (i as f64 * 50.0)),
        );
        typed_facts.set("IsVIP".to_string(), FactValue::Boolean(i % 10 == 0));

        facts.push((format!("User{}", i), typed_facts));
    }

    facts
}

fn create_incremental_engine(rule_count: usize, user_count: usize) -> IncrementalEngine {
    let mut engine = IncrementalEngine::new();

    // Add facts to working memory
    let facts = setup_incremental_facts(user_count);
    for (fact_type, data) in facts {
        engine.insert(fact_type.clone(), data);
    }

    // Add rules
    for i in 0..rule_count {
        let user_idx = i % user_count.max(1);
        let age_threshold = 18 + (i % 10);
        let spending_threshold = 200.0 + (i as f64 * 10.0);

        let condition = ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: format!("User{}.Age", user_idx),
                operator: ">".to_string(),
                value: format!("{}", age_threshold),
            })),
            operator: "AND".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: format!("User{}.SpendingTotal", user_idx),
                operator: ">".to_string(),
                value: format!("{}", spending_threshold),
            })),
        };

        let node = build_rete_ul_from_condition_group(&condition);
        let rule_name = format!("IncrRule{}", i);

        let rule = TypedReteUlRule {
            name: rule_name.clone(),
            node,
            priority: (100 - (i % 20)) as i32,
            no_loop: false,
            action: std::sync::Arc::new(move |_facts: &mut TypedFacts, _results: &mut rust_rule_engine::rete::action_result::ActionResults| {
                // Action placeholder
            }),
        };

        engine.add_rule(rule, vec![format!("User{}", user_idx)]);
    }

    engine
}

// ============================================================================
// Benchmark: Simple Rule Execution - Traditional vs RETE
// ============================================================================

fn bench_simple_execution_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_execution");
    group.throughput(Throughput::Elements(10));

    // Traditional Engine
    let (mut trad_engine, trad_facts) = create_traditional_engine(10, 20);
    group.bench_function("traditional_10rules_20users", |b| {
        b.iter(|| {
            black_box(trad_engine.execute(&trad_facts).unwrap());
        })
    });

    // RETE Engine
    let mut rete_engine = create_rete_engine(10, 20);
    group.bench_function("rete_10rules_20users", |b| {
        b.iter(|| {
            black_box(rete_engine.fire_all());
        })
    });

    // Incremental Engine
    let mut incr_engine = create_incremental_engine(10, 20);
    group.bench_function("incremental_10rules_20users", |b| {
        b.iter(|| {
            black_box(incr_engine.fire_all());
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: Medium Scale - Traditional vs RETE
// ============================================================================

fn bench_medium_scale_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_scale");
    group.throughput(Throughput::Elements(50));
    group.measurement_time(Duration::from_secs(30));

    // Traditional Engine
    let (mut trad_engine, trad_facts) = create_traditional_engine(50, 100);
    group.bench_function("traditional_50rules_100users", |b| {
        b.iter(|| {
            black_box(trad_engine.execute(&trad_facts).unwrap());
        })
    });

    // RETE Engine
    let mut rete_engine = create_rete_engine(50, 100);
    group.bench_function("rete_50rules_100users", |b| {
        b.iter(|| {
            black_box(rete_engine.fire_all());
        })
    });

    // Incremental Engine
    let mut incr_engine = create_incremental_engine(50, 100);
    group.bench_function("incremental_50rules_100users", |b| {
        b.iter(|| {
            black_box(incr_engine.fire_all());
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: Large Scale - Traditional vs RETE
// ============================================================================

fn bench_large_scale_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_scale");
    group.throughput(Throughput::Elements(200));
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10);

    // Traditional Engine
    let (mut trad_engine, trad_facts) = create_traditional_engine(200, 500);
    group.bench_function("traditional_200rules_500users", |b| {
        b.iter(|| {
            black_box(trad_engine.execute(&trad_facts).unwrap());
        })
    });

    // RETE Engine
    let mut rete_engine = create_rete_engine(200, 500);
    group.bench_function("rete_200rules_500users", |b| {
        b.iter(|| {
            black_box(rete_engine.fire_all());
        })
    });

    // Incremental Engine
    let mut incr_engine = create_incremental_engine(200, 500);
    group.bench_function("incremental_200rules_500users", |b| {
        b.iter(|| {
            black_box(incr_engine.fire_all());
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: Rule Scalability
// ============================================================================

fn bench_rule_scalability_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_scalability");
    group.measurement_time(Duration::from_secs(20));

    for rule_count in [10, 25, 50, 100].iter() {
        let user_count = 100;

        // Traditional
        let (mut trad_engine, trad_facts) = create_traditional_engine(*rule_count, user_count);
        group.throughput(Throughput::Elements(*rule_count as u64));
        group.bench_with_input(
            BenchmarkId::new("traditional", rule_count),
            rule_count,
            |b, &_| {
                b.iter(|| {
                    black_box(trad_engine.execute(&trad_facts).unwrap());
                })
            },
        );

        // RETE
        let mut rete_engine = create_rete_engine(*rule_count, user_count);
        group.bench_with_input(BenchmarkId::new("rete", rule_count), rule_count, |b, &_| {
            b.iter(|| {
                black_box(rete_engine.fire_all());
            })
        });

        // Incremental
        let mut incr_engine = create_incremental_engine(*rule_count, user_count);
        group.bench_with_input(
            BenchmarkId::new("incremental", rule_count),
            rule_count,
            |b, &_| {
                b.iter(|| {
                    black_box(incr_engine.fire_all());
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// Benchmark: Incremental Updates (RETE advantage)
// ============================================================================

fn bench_incremental_updates(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_updates");
    group.measurement_time(Duration::from_secs(20));

    let rule_count = 50;
    let user_count = 100;

    // Traditional Engine - must re-evaluate all rules on every change
    let (mut trad_engine, _trad_facts) = create_traditional_engine(rule_count, user_count);
    group.bench_function("traditional_with_fact_update", |b| {
        b.iter(|| {
            // Simulate fact update by re-creating facts
            let updated_facts = setup_traditional_facts(user_count);
            black_box(trad_engine.execute(&updated_facts).unwrap());
        })
    });

    // Incremental Engine - only re-evaluates affected rules
    group.bench_function("incremental_with_fact_update", |b| {
        let mut engine = create_incremental_engine(rule_count, user_count);
        let mut counter = 0;

        b.iter(|| {
            counter += 1;
            // Update a single user's data
            let user_idx = counter % user_count;
            let mut typed_facts = TypedFacts::new();
            typed_facts.set("Age".to_string(), FactValue::Integer(25 + counter as i64));
            typed_facts.set(
                "SpendingTotal".to_string(),
                FactValue::Float(150.0 + counter as f64),
            );
            typed_facts.set("Country".to_string(), FactValue::String("US".to_string()));
            typed_facts.set("IsVIP".to_string(), FactValue::Boolean(true));

            // Insert as new fact (simulating update)
            engine.insert(format!("User{}", user_idx), typed_facts);

            black_box(engine.fire_all());
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: Memory Efficiency
// ============================================================================

fn bench_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    group.measurement_time(Duration::from_secs(15));

    // Traditional - creates new facts each time
    group.bench_function("traditional_fact_creation", |b| {
        b.iter(|| {
            black_box(setup_traditional_facts(100));
        })
    });

    // RETE - reuses fact structure
    group.bench_function("rete_fact_creation", |b| {
        b.iter(|| {
            black_box(setup_rete_facts(100));
        })
    });

    // Incremental - structured working memory
    group.bench_function("incremental_fact_creation", |b| {
        b.iter(|| {
            black_box(setup_incremental_facts(100));
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark: Complex Pattern Matching
// ============================================================================

fn bench_complex_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("complex_patterns");
    group.measurement_time(Duration::from_secs(20));

    // Traditional with complex nested conditions
    let complex_rules = r#"
        rule "ComplexVIPUpgrade" salience 20 {
            when
                User0.Age > 25 && User0.SpendingTotal > 5000.0 &&
                (User0.Country == "US" || User0.Country == "UK") &&
                User0.IsVIP != true
            then
                log("VIP upgrade");
        }
    "#;

    let facts = setup_traditional_facts(50);
    let kb = KnowledgeBase::new("ComplexBench");
    let parsed_rules = GRLParser::parse_rules(complex_rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }
    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 1,
        ..Default::default()
    };
    let mut trad_engine = RustRuleEngine::with_config(kb, config);

    group.bench_function("traditional_complex_pattern", |b| {
        b.iter(|| {
            black_box(trad_engine.execute(&facts).unwrap());
        })
    });

    // RETE with complex nested conditions
    let mut rete_engine = ReteUlEngine::new();
    let complex_condition = ConditionGroup::Compound {
        left: Box::new(ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: "User0.Age".to_string(),
                operator: ">".to_string(),
                value: "25".to_string(),
            })),
            operator: "AND".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "User0.SpendingTotal".to_string(),
                operator: ">".to_string(),
                value: "5000.0".to_string(),
            })),
        }),
        operator: "AND".to_string(),
        right: Box::new(ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: "User0.Country".to_string(),
                operator: "==".to_string(),
                value: "US".to_string(),
            })),
            operator: "OR".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "User0.Country".to_string(),
                operator: "==".to_string(),
                value: "UK".to_string(),
            })),
        }),
    };

    let rule = Rule {
        name: "ComplexVIPUpgrade".to_string(),
        conditions: complex_condition,
        action: "log('VIP upgrade')".to_string(),
    };

    rete_engine.add_rule_from_definition(&rule, 20, false);

    let rete_facts = setup_rete_facts(50);
    for (key, value) in rete_facts {
        rete_engine.set_fact(key, value);
    }

    group.bench_function("rete_complex_pattern", |b| {
        b.iter(|| {
            black_box(rete_engine.fire_all());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_execution_comparison,
    bench_medium_scale_comparison,
    bench_large_scale_comparison,
    bench_rule_scalability_comparison,
    bench_incremental_updates,
    bench_memory_efficiency,
    bench_complex_patterns,
);

criterion_main!(benches);

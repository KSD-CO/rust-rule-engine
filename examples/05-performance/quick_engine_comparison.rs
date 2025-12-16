/// Quick comparison benchmark: Native vs Parallel vs RETE
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::parallel::{ParallelConfig, ParallelRuleEngine};
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::RustRuleEngine;
use rust_rule_engine::rete::auto_network::{
    Condition as AutoCondition, ConditionGroup as AutoConditionGroup,
};
use rust_rule_engine::rete::network::build_rete_ul_from_condition_group;
use rust_rule_engine::rete::{FactValue, IncrementalEngine, TypedFacts, TypedReteUlRule};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::time::Instant;

fn main() {
    println!("🏁 Quick Benchmark: Native vs Parallel vs RETE");
    println!("================================================\n");

    // Test với số lượng rules tăng dần
    for rule_count in [10, 50, 100, 200, 500, 1000] {
        println!("📊 Testing with {} rules:", rule_count);
        run_comparison(rule_count);
        println!();
    }
}

fn run_comparison(rule_count: usize) {
    let iterations = 100;

    // Setup Native Engine
    let (mut native_engine, native_facts) = setup_native(rule_count);

    // Setup Parallel Engine
    let (parallel_engine, parallel_kb, parallel_facts) = setup_parallel(rule_count);

    // Warmup
    let _ = native_engine.execute(&native_facts);
    let _ = parallel_engine.execute_parallel(&parallel_kb, &parallel_facts, false);

    // Benchmark Native
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = native_engine.execute(&native_facts);
    }
    let native_time = start.elapsed();
    let native_avg = native_time.as_micros() / iterations;

    // Benchmark Parallel
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = parallel_engine.execute_parallel(&parallel_kb, &parallel_facts, false);
    }
    let parallel_time = start.elapsed();
    let parallel_avg = parallel_time.as_micros() / iterations;

    // Benchmark RETE (insert triggers automatic propagation)
    // Note: RETE uses incremental network, so we create fresh engine each time
    let start = Instant::now();
    for _ in 0..iterations {
        let mut engine = setup_rete(rule_count);
        let mut facts = TypedFacts::new();
        facts.set("User.age", FactValue::Integer(35));
        facts.set("User.score", FactValue::Integer(85));
        facts.set("User.premium", FactValue::Boolean(true));
        engine.insert("User".to_string(), facts); // Auto-propagates through RETE network
    }
    let rete_time = start.elapsed();
    let rete_avg = rete_time.as_micros() / iterations;

    // Calculate speedups
    let parallel_speedup = native_avg as f64 / parallel_avg as f64;
    let rete_speedup = native_avg as f64 / rete_avg as f64;

    println!("  Native:   {:6} μs", native_avg);
    println!(
        "  Parallel: {:6} μs ({:.2}x faster)",
        parallel_avg, parallel_speedup
    );

    if rete_speedup > 1.0 {
        println!(
            "  RETE:     {:6} μs ({:.2}x faster) ⚡",
            rete_avg, rete_speedup
        );
    } else {
        println!(
            "  RETE:     {:6} μs ({:.2}x slower) ⏱",
            rete_avg,
            1.0 / rete_speedup
        );
    }
}

fn setup_native(rule_count: usize) -> (RustRuleEngine, Facts) {
    let kb = KnowledgeBase::new("Native");

    for i in 0..rule_count {
        // Mix of simple and complex conditions
        let condition = match i % 4 {
            0 => {
                // Simple condition
                ConditionGroup::single(Condition::new(
                    "User.age".to_string(),
                    Operator::GreaterThan,
                    Value::Integer(18 + (i % 30) as i64),
                ))
            }
            1 => {
                // AND condition: age > 25 AND score >= 70
                ConditionGroup::and(
                    ConditionGroup::single(Condition::new(
                        "User.age".to_string(),
                        Operator::GreaterThan,
                        Value::Integer(25),
                    )),
                    ConditionGroup::single(Condition::new(
                        "User.score".to_string(),
                        Operator::GreaterThanOrEqual,
                        Value::Integer(70 + (i % 30) as i64),
                    )),
                )
            }
            2 => {
                // OR condition: premium == true OR score > 90
                ConditionGroup::or(
                    ConditionGroup::single(Condition::new(
                        "User.premium".to_string(),
                        Operator::Equal,
                        Value::Boolean(true),
                    )),
                    ConditionGroup::single(Condition::new(
                        "User.score".to_string(),
                        Operator::GreaterThan,
                        Value::Integer(90),
                    )),
                )
            }
            _ => {
                // Complex nested: (age > 30 AND score > 80) OR premium
                ConditionGroup::or(
                    ConditionGroup::and(
                        ConditionGroup::single(Condition::new(
                            "User.age".to_string(),
                            Operator::GreaterThan,
                            Value::Integer(30),
                        )),
                        ConditionGroup::single(Condition::new(
                            "User.score".to_string(),
                            Operator::GreaterThan,
                            Value::Integer(80),
                        )),
                    ),
                    ConditionGroup::single(Condition::new(
                        "User.premium".to_string(),
                        Operator::Equal,
                        Value::Boolean(true),
                    )),
                )
            }
        };

        let rule = Rule::new(
            format!("Rule{}", i),
            condition,
            vec![ActionType::Set {
                field: format!("result{}", i),
                value: Value::Boolean(true),
            }],
        );
        kb.add_rule(rule);
    }

    let engine = RustRuleEngine::new(kb);
    let facts = Facts::new();
    facts.set("User.age", Value::Integer(35));
    facts.set("User.score", Value::Integer(85));
    facts.set("User.premium", Value::Boolean(true));

    (engine, facts)
}

fn setup_parallel(rule_count: usize) -> (ParallelRuleEngine, KnowledgeBase, Facts) {
    let config = ParallelConfig::default();
    let engine = ParallelRuleEngine::new(config);
    let kb = KnowledgeBase::new("Parallel");

    for i in 0..rule_count {
        // Same complex conditions as Native
        let condition = match i % 4 {
            0 => ConditionGroup::single(Condition::new(
                "User.age".to_string(),
                Operator::GreaterThan,
                Value::Integer(18 + (i % 30) as i64),
            )),
            1 => ConditionGroup::and(
                ConditionGroup::single(Condition::new(
                    "User.age".to_string(),
                    Operator::GreaterThan,
                    Value::Integer(25),
                )),
                ConditionGroup::single(Condition::new(
                    "User.score".to_string(),
                    Operator::GreaterThanOrEqual,
                    Value::Integer(70 + (i % 30) as i64),
                )),
            ),
            2 => ConditionGroup::or(
                ConditionGroup::single(Condition::new(
                    "User.premium".to_string(),
                    Operator::Equal,
                    Value::Boolean(true),
                )),
                ConditionGroup::single(Condition::new(
                    "User.score".to_string(),
                    Operator::GreaterThan,
                    Value::Integer(90),
                )),
            ),
            _ => ConditionGroup::or(
                ConditionGroup::and(
                    ConditionGroup::single(Condition::new(
                        "User.age".to_string(),
                        Operator::GreaterThan,
                        Value::Integer(30),
                    )),
                    ConditionGroup::single(Condition::new(
                        "User.score".to_string(),
                        Operator::GreaterThan,
                        Value::Integer(80),
                    )),
                ),
                ConditionGroup::single(Condition::new(
                    "User.premium".to_string(),
                    Operator::Equal,
                    Value::Boolean(true),
                )),
            ),
        };

        let rule = Rule::new(
            format!("Rule{}", i),
            condition,
            vec![ActionType::Set {
                field: format!("result{}", i),
                value: Value::Boolean(true),
            }],
        );
        kb.add_rule(rule);
    }

    let facts = Facts::new();
    facts.set("User.age", Value::Integer(35));
    facts.set("User.score", Value::Integer(85));
    facts.set("User.premium", Value::Boolean(true));

    (engine, kb, facts)
}

fn setup_rete(rule_count: usize) -> IncrementalEngine {
    let mut engine = IncrementalEngine::new();

    for i in 0..rule_count {
        // Same complex conditions as Native/Parallel
        let condition = match i % 4 {
            0 => AutoConditionGroup::Single(AutoCondition {
                field: "User.age".to_string(),
                operator: ">".to_string(),
                value: format!("{}", 18 + (i % 30)),
            }),
            1 => AutoConditionGroup::Compound {
                left: Box::new(AutoConditionGroup::Single(AutoCondition {
                    field: "User.age".to_string(),
                    operator: ">".to_string(),
                    value: "25".to_string(),
                })),
                operator: "AND".to_string(),
                right: Box::new(AutoConditionGroup::Single(AutoCondition {
                    field: "User.score".to_string(),
                    operator: ">=".to_string(),
                    value: format!("{}", 70 + (i % 30)),
                })),
            },
            2 => AutoConditionGroup::Compound {
                left: Box::new(AutoConditionGroup::Single(AutoCondition {
                    field: "User.premium".to_string(),
                    operator: "==".to_string(),
                    value: "true".to_string(),
                })),
                operator: "OR".to_string(),
                right: Box::new(AutoConditionGroup::Single(AutoCondition {
                    field: "User.score".to_string(),
                    operator: ">".to_string(),
                    value: "90".to_string(),
                })),
            },
            _ => {
                // Complex nested: (age > 30 AND score > 80) OR premium
                AutoConditionGroup::Compound {
                    left: Box::new(AutoConditionGroup::Compound {
                        left: Box::new(AutoConditionGroup::Single(AutoCondition {
                            field: "User.age".to_string(),
                            operator: ">".to_string(),
                            value: "30".to_string(),
                        })),
                        operator: "AND".to_string(),
                        right: Box::new(AutoConditionGroup::Single(AutoCondition {
                            field: "User.score".to_string(),
                            operator: ">".to_string(),
                            value: "80".to_string(),
                        })),
                    }),
                    operator: "OR".to_string(),
                    right: Box::new(AutoConditionGroup::Single(AutoCondition {
                        field: "User.premium".to_string(),
                        operator: "==".to_string(),
                        value: "true".to_string(),
                    })),
                }
            }
        };

        let node = build_rete_ul_from_condition_group(&condition);

        let rule = TypedReteUlRule {
            name: format!("Rule{}", i),
            node,
            priority: 0,
            no_loop: false,
            action: std::sync::Arc::new(move |facts: &mut TypedFacts, _results: &mut rust_rule_engine::rete::action_result::ActionResults| {
                facts.set(format!("result{}", i), FactValue::Boolean(true));
            }),
        };

        engine.add_rule(rule, vec!["User".to_string()]);
    }

    engine
}

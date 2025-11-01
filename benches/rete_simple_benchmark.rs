use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use rust_rule_engine::rete::{
    ReteUlEngine,
    auto_network::{Rule, ConditionGroup, Condition},
};
use std::collections::HashMap;

// ============================================================================
// Traditional Engine Setup
// ============================================================================

fn create_traditional_facts() -> Facts {
    let facts = Facts::new();
    let mut user = HashMap::new();
    user.insert("age".to_string(), Value::Integer(30));
    user.insert("country".to_string(), Value::String("US".to_string()));
    user.insert("spending".to_string(), Value::Number(1500.0));
    facts.add_value("user", Value::Object(user)).unwrap();
    facts
}

fn create_traditional_engine_simple() -> (RustRuleEngine, Facts) {
    let kb = KnowledgeBase::new("SimpleTest");

    let rules = r#"
        rule "Rule1" salience 100 {
            when
                user.age > 25 && user.spending > 1000.0
            then
                log("Rule1 fired");
        }

        rule "Rule2" salience 90 {
            when
                user.country == "US"
            then
                log("Rule2 fired");
        }

        rule "Rule3" salience 80 {
            when
                user.age > 20 && user.country == "US"
            then
                log("Rule3 fired");
        }
    "#;

    let parsed_rules = GRLParser::parse_rules(rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 1,
        ..Default::default()
    };

    let engine = RustRuleEngine::with_config(kb, config);
    let facts = create_traditional_facts();
    (engine, facts)
}

// ============================================================================
// RETE Engine Setup
// ============================================================================

fn create_rete_facts() -> HashMap<String, String> {
    let mut facts = HashMap::new();
    facts.insert("user.age".to_string(), "30".to_string());
    facts.insert("user.country".to_string(), "US".to_string());
    facts.insert("user.spending".to_string(), "1500.0".to_string());
    facts
}

fn create_rete_engine_simple() -> ReteUlEngine {
    let mut engine = ReteUlEngine::new();

    // Rule 1: age > 25 AND spending > 1000
    let rule1 = Rule {
        name: "Rule1".to_string(),
        conditions: ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: "user.age".to_string(),
                operator: ">".to_string(),
                value: "25".to_string(),
            })),
            operator: "AND".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "user.spending".to_string(),
                operator: ">".to_string(),
                value: "1000.0".to_string(),
            })),
        },
        action: "log('Rule1 fired')".to_string(),
    };
    engine.add_rule_from_definition(&rule1, 100, false);

    // Rule 2: country == "US"
    let rule2 = Rule {
        name: "Rule2".to_string(),
        conditions: ConditionGroup::Single(Condition {
            field: "user.country".to_string(),
            operator: "==".to_string(),
            value: "US".to_string(),
        }),
        action: "log('Rule2 fired')".to_string(),
    };
    engine.add_rule_from_definition(&rule2, 90, false);

    // Rule 3: age > 20 AND country == "US"
    let rule3 = Rule {
        name: "Rule3".to_string(),
        conditions: ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: "user.age".to_string(),
                operator: ">".to_string(),
                value: "20".to_string(),
            })),
            operator: "AND".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "user.country".to_string(),
                operator: "==".to_string(),
                value: "US".to_string(),
            })),
        },
        action: "log('Rule3 fired')".to_string(),
    };
    engine.add_rule_from_definition(&rule3, 80, false);

    // Set facts
    let facts = create_rete_facts();
    for (key, value) in facts {
        engine.set_fact(key, value);
    }

    engine
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_3_rules_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("3_rules");

    // Traditional
    let (mut trad_engine, trad_facts) = create_traditional_engine_simple();
    group.bench_function("traditional", |b| {
        b.iter(|| {
            black_box(trad_engine.execute(&trad_facts).unwrap());
        })
    });

    // RETE
    let mut rete_engine = create_rete_engine_simple();
    group.bench_function("rete", |b| {
        b.iter(|| {
            black_box(rete_engine.fire_all());
        })
    });

    group.finish();
}

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    for &count in &[3, 10, 25] {
        // Traditional
        let kb = KnowledgeBase::new("ScalingTest");
        let mut rules_str = String::new();
        for i in 0..count {
            rules_str.push_str(&format!(
                r#"
                rule "Rule{}" salience {} {{
                    when
                        user.age > {} && user.spending > {}
                    then
                        log("Rule {} fired");
                }}
                "#,
                i, 100 - i, 20 + (i % 10), 500.0 + (i as f64 * 100.0), i
            ));
        }

        let parsed_rules = GRLParser::parse_rules(&rules_str).unwrap();
        for rule in parsed_rules {
            kb.add_rule(rule).unwrap();
        }

        let config = EngineConfig {
            debug_mode: false,
            max_cycles: 1,
            ..Default::default()
        };

        let mut trad_engine = RustRuleEngine::with_config(kb, config);
        let trad_facts = create_traditional_facts();

        group.bench_with_input(
            BenchmarkId::new("traditional", count),
            &count,
            |b, _| {
                b.iter(|| {
                    black_box(trad_engine.execute(&trad_facts).unwrap());
                })
            },
        );

        // RETE
        let mut rete_engine = ReteUlEngine::new();
        for i in 0..count {
            let rule = Rule {
                name: format!("Rule{}", i),
                conditions: ConditionGroup::Compound {
                    left: Box::new(ConditionGroup::Single(Condition {
                        field: "user.age".to_string(),
                        operator: ">".to_string(),
                        value: format!("{}", 20 + (i % 10)),
                    })),
                    operator: "AND".to_string(),
                    right: Box::new(ConditionGroup::Single(Condition {
                        field: "user.spending".to_string(),
                        operator: ">".to_string(),
                        value: format!("{}", 500.0 + (i as f64 * 100.0)),
                    })),
                },
                action: format!("log('Rule {} fired')", i),
            };
            rete_engine.add_rule_from_definition(&rule, (100 - i) as i32, false);
        }

        let facts = create_rete_facts();
        for (key, value) in facts {
            rete_engine.set_fact(key, value);
        }

        group.bench_with_input(
            BenchmarkId::new("rete", count),
            &count,
            |b, _| {
                b.iter(|| {
                    black_box(rete_engine.fire_all());
                })
            },
        );
    }

    group.finish();
}

fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");

    // Traditional - simple pattern
    let kb_simple = KnowledgeBase::new("SimplePattern");
    let simple_rule = r#"
        rule "SimplePattern" {
            when
                user.age > 25
            then
                log("match");
        }
    "#;
    let parsed = GRLParser::parse_rules(simple_rule).unwrap();
    for rule in parsed {
        kb_simple.add_rule(rule).unwrap();
    }
    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 1,
        ..Default::default()
    };
    let mut trad_simple = RustRuleEngine::with_config(kb_simple, config);
    let facts = create_traditional_facts();

    group.bench_function("traditional_simple_pattern", |b| {
        b.iter(|| {
            black_box(trad_simple.execute(&facts).unwrap());
        })
    });

    // RETE - simple pattern
    let mut rete_simple = ReteUlEngine::new();
    let rule = Rule {
        name: "SimplePattern".to_string(),
        conditions: ConditionGroup::Single(Condition {
            field: "user.age".to_string(),
            operator: ">".to_string(),
            value: "25".to_string(),
        }),
        action: "log('match')".to_string(),
    };
    rete_simple.add_rule_from_definition(&rule, 100, false);
    let rete_facts = create_rete_facts();
    for (k, v) in rete_facts {
        rete_simple.set_fact(k, v);
    }

    group.bench_function("rete_simple_pattern", |b| {
        b.iter(|| {
            black_box(rete_simple.fire_all());
        })
    });

    // Traditional - complex pattern
    let kb_complex = KnowledgeBase::new("ComplexPattern");
    let complex_rule = r#"
        rule "ComplexPattern" {
            when
                user.age > 25 && user.spending > 1000.0 && user.country == "US"
            then
                log("match");
        }
    "#;
    let parsed = GRLParser::parse_rules(complex_rule).unwrap();
    for rule in parsed {
        kb_complex.add_rule(rule).unwrap();
    }
    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 1,
        ..Default::default()
    };
    let mut trad_complex = RustRuleEngine::with_config(kb_complex, config);

    group.bench_function("traditional_complex_pattern", |b| {
        b.iter(|| {
            black_box(trad_complex.execute(&facts).unwrap());
        })
    });

    // RETE - complex pattern
    let mut rete_complex = ReteUlEngine::new();
    let rule = Rule {
        name: "ComplexPattern".to_string(),
        conditions: ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition {
                    field: "user.age".to_string(),
                    operator: ">".to_string(),
                    value: "25".to_string(),
                })),
                operator: "AND".to_string(),
                right: Box::new(ConditionGroup::Single(Condition {
                    field: "user.spending".to_string(),
                    operator: ">".to_string(),
                    value: "1000.0".to_string(),
                })),
            }),
            operator: "AND".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "user.country".to_string(),
                operator: "==".to_string(),
                value: "US".to_string(),
            })),
        },
        action: "log('match')".to_string(),
    };
    rete_complex.add_rule_from_definition(&rule, 100, false);
    let rete_facts = create_rete_facts();
    for (k, v) in rete_facts {
        rete_complex.set_fact(k, v);
    }

    group.bench_function("rete_complex_pattern", |b| {
        b.iter(|| {
            black_box(rete_complex.fire_all());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_3_rules_comparison,
    bench_scaling,
    bench_pattern_matching,
);

criterion_main!(benches);

use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("Testing Rule Updates Scenario");
    println!("=============================");

    let mut kb = KnowledgeBase::new("DynamicRules");

    // Start with some initial rules
    add_initial_rules(&mut kb);

    let config = EngineConfig::default();
    let mut engine = RustRuleEngine::with_config(kb, config);

    let mut facts = create_test_facts();

    // Test initial performance
    let initial_time = measure_execution_time(&mut engine, &facts);
    println!("Initial execution time: {:.2} µs", initial_time);

    // Add more rules dynamically
    let start_add = Instant::now();
    for i in 0..50 {
        let rule = create_dynamic_rule(i);
        engine.knowledge_base_mut().add_rule(rule).unwrap();
    }
    let add_time = start_add.elapsed().as_micros();
    println!("Time to add 50 rules: {:.2} µs", add_time);

    // Test performance after adding rules
    let after_add_time = measure_execution_time(&mut engine, &facts);
    println!("Execution time after adding rules: {:.2} µs", after_add_time);
    println!("Performance degradation: {:.2}x", after_add_time as f64 / initial_time as f64);

    // Test with different fact sets
    println!("\nTesting with varying fact complexity:");
    for complexity in [10, 50, 100, 200] {
        let complex_facts = create_complex_facts(complexity);
        let complex_time = measure_execution_time(&mut engine, &complex_facts);
        println!("  {} facts: {:.2} µs", complexity, complex_time);
    }
}

fn add_initial_rules(kb: &mut KnowledgeBase) {
    let rule1 = Rule::new(
        "InitialRule1".to_string(),
        ConditionGroup::single(Condition::new(
            "data.value".to_string(),
            Operator::GreaterThan,
            Value::Integer(50),
        )),
        vec![ActionType::Set {
            field: "data.result".to_string(),
            value: Value::String("high".to_string()),
        }],
    );

    let rule2 = Rule::new(
        "InitialRule2".to_string(),
        ConditionGroup::single(Condition::new(
            "data.status".to_string(),
            Operator::Equal,
            Value::String("active".to_string()),
        )),
        vec![ActionType::Set {
            field: "data.processed".to_string(),
            value: Value::Boolean(true),
        }],
    );

    kb.add_rule(rule1).unwrap();
    kb.add_rule(rule2).unwrap();
}

fn create_dynamic_rule(index: usize) -> Rule {
    let condition_value = (index * 10) as i64;
    Rule::new(
        format!("DynamicRule{}", index),
        ConditionGroup::single(Condition::new(
            "data.dynamic_value".to_string(),
            Operator::GreaterThan,
            Value::Integer(condition_value),
        )),
        vec![ActionType::Set {
            field: format!("data.dynamic_result_{}", index),
            value: Value::String(format!("triggered_{}", index)),
        }],
    )
}

fn create_test_facts() -> Facts {
    let mut facts = Facts::new();
    let mut data_props = HashMap::new();
    data_props.insert("value".to_string(), Value::Integer(75));
    data_props.insert("status".to_string(), Value::String("active".to_string()));
    data_props.insert("dynamic_value".to_string(), Value::Integer(100));
    facts.set("data", Value::Object(data_props));
    facts
}

fn create_complex_facts(count: usize) -> Facts {
    let mut facts = Facts::new();
    for i in 0..count {
        let mut props = HashMap::new();
        props.insert("id".to_string(), Value::Integer(i as i64));
        props.insert("value".to_string(), Value::Integer((i * 5) as i64));
        props.insert("active".to_string(), Value::Boolean(i % 2 == 0));
        facts.set(&format!("item{}", i), Value::Object(props));
    }
    facts
}

fn measure_execution_time(engine: &mut RustRuleEngine, facts: &Facts) -> u128 {
    let start = Instant::now();
    let _result = engine.execute(facts).unwrap();
    start.elapsed().as_micros()
}
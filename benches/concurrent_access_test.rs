use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn main() {
    println!("Testing Concurrent Access Scenario");
    println!("==================================");

    // Create a rule engine with some sample rules
    let rule_engine = Arc::new(Mutex::new(create_rule_engine()));

    let num_threads = 10;
    let facts_per_thread = 100;
    let mut handles = vec![];

    let start = Instant::now();

    for thread_id in 0..num_threads {
        let rule_engine_clone = Arc::clone(&rule_engine);
        let handle = thread::spawn(move || {
            let mut total_time = 0u128;
            let mut executions = 0;

            for fact_id in 0..facts_per_thread {
                let exec_start = Instant::now();

                // Lock and execute
                {
                    let mut facts = Facts::new();
                    let mut data_props = HashMap::new();
                    data_props.insert("thread".to_string(), Value::Integer(thread_id as i64));
                    data_props.insert("fact".to_string(), Value::Integer(fact_id as i64));
                    data_props.insert("value".to_string(), Value::Integer((fact_id * 10) as i64));
                    facts.set("data", Value::Object(data_props));

                    let mut engine = rule_engine_clone.lock().unwrap();
                    let _result = engine.execute(&facts).unwrap();
                }

                let exec_time = exec_start.elapsed().as_micros();
                total_time += exec_time;
                executions += 1;
            }

            (total_time, executions)
        });
        handles.push(handle);
    }

    let mut total_time_all = 0u128;
    let mut total_executions = 0;

    for handle in handles {
        let (thread_time, thread_executions) = handle.join().unwrap();
        total_time_all += thread_time;
        total_executions += thread_executions;
    }

    let total_duration = start.elapsed();
    let avg_time_per_execution = total_time_all as f64 / total_executions as f64;

    println!("Concurrent Access Test Results:");
    println!("- Threads: {}", num_threads);
    println!("- Facts per thread: {}", facts_per_thread);
    println!("- Total executions: {}", total_executions);
    println!("- Total time: {:.2} ms", total_duration.as_millis());
    println!("- Average time per execution: {:.2} µs", avg_time_per_execution);
    println!("- Throughput: {:.0} executions/second", total_executions as f64 / total_duration.as_secs_f64());
}

fn create_rule_engine() -> RustRuleEngine {
    let kb = KnowledgeBase::new("ConcurrentTest");

    // Rule 1: Check if value is greater than 50
    let adult_condition = ConditionGroup::single(Condition::new(
        "data.value".to_string(),
        Operator::GreaterThan,
        Value::Integer(50),
    ));
    let adult_action = ActionType::Set {
        field: "data.result".to_string(),
        value: Value::String("high".to_string()),
    };
    let adult_rule = Rule::new(
        "CheckValue".to_string(),
        adult_condition,
        vec![adult_action],
    );

    // Rule 2: Check thread ID
    let vip_condition = ConditionGroup::single(Condition::new(
        "data.thread".to_string(),
        Operator::GreaterThanOrEqual,
        Value::Integer(0),
    ));
    let vip_action = ActionType::Set {
        field: "data.processed".to_string(),
        value: Value::Boolean(true),
    };
    let vip_rule = Rule::new(
        "CheckThread".to_string(),
        vip_condition,
        vec![vip_action],
    );

    let _ = kb.add_rule(adult_rule);
    let _ = kb.add_rule(vip_rule);

    let config = EngineConfig::default();
    RustRuleEngine::with_config(kb, config)
}
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("Testing Streaming Scenarios");
    println!("===========================");

    let mut engine = create_streaming_engine();

    // Simulate streaming events
    let num_events = 10000;
    let mut total_time = 0u128;
    let mut processed_events = 0;

    let start = Instant::now();

    for event_id in 0..num_events {
        let event = create_stream_event(event_id);

        let exec_start = Instant::now();
        let _result = engine.execute(&event).unwrap();
        let exec_time = exec_start.elapsed().as_micros();

        total_time += exec_time;
        processed_events += 1;

        // Simulate real-time processing - small delay every 100 events
        if event_id % 100 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }

    let total_duration = start.elapsed();
    let avg_time_per_event = total_time as f64 / processed_events as f64;

    println!("Streaming Test Results:");
    println!("- Events processed: {}", processed_events);
    println!("- Total time: {:.2} ms", total_duration.as_millis());
    println!("- Average time per event: {:.2} µs", avg_time_per_event);
    println!("- Throughput: {:.0} events/second", processed_events as f64 / total_duration.as_secs_f64());

    // Test burst processing
    println!("\nTesting burst processing (1000 events in rapid succession):");
    let burst_start = Instant::now();
    let mut burst_time = 0u128;

    for burst_id in 0..1000 {
        let event = create_burst_event(burst_id);
        let exec_start = Instant::now();
        let _result = engine.execute(&event).unwrap();
        burst_time += exec_start.elapsed().as_micros();
    }

    let burst_duration = burst_start.elapsed();
    let burst_avg = burst_time as f64 / 1000.0;

    println!("- Burst time: {:.2} ms", burst_duration.as_millis());
    println!("- Burst average per event: {:.2} µs", burst_avg);
    println!("- Burst throughput: {:.0} events/second", 1000.0 / burst_duration.as_secs_f64());
}

fn create_streaming_engine() -> RustRuleEngine {
    let kb = KnowledgeBase::new("StreamingEngine");

    // Rule for price alerts
    let price_rule = Rule::new(
        "PriceAlert".to_string(),
        ConditionGroup::single(Condition::new(
            "event.price".to_string(),
            Operator::GreaterThan,
            Value::Integer(100),
        )),
        vec![ActionType::Set {
            field: "event.alert".to_string(),
            value: Value::String("high_price".to_string()),
        }],
    );

    // Rule for volume spikes
    let volume_rule = Rule::new(
        "VolumeSpike".to_string(),
        ConditionGroup::single(Condition::new(
            "event.volume".to_string(),
            Operator::GreaterThan,
            Value::Integer(1000),
        )),
        vec![ActionType::Set {
            field: "event.spike".to_string(),
            value: Value::Boolean(true),
        }],
    );

    // Rule for trend detection
    let trend_rule = Rule::new(
        "TrendDetection".to_string(),
        ConditionGroup::single(Condition::new(
            "event.change".to_string(),
            Operator::GreaterThan,
            Value::Number(5.0),
        )),
        vec![ActionType::Set {
            field: "event.trend".to_string(),
            value: Value::String("upward".to_string()),
        }],
    );

    kb.add_rule(price_rule).unwrap();
    kb.add_rule(volume_rule).unwrap();
    kb.add_rule(trend_rule).unwrap();

    let config = EngineConfig::default();
    RustRuleEngine::with_config(kb, config)
}

fn create_stream_event(event_id: usize) -> Facts {
    let mut facts = Facts::new();
    let mut event_props = HashMap::new();

    event_props.insert("id".to_string(), Value::Integer(event_id as i64));
    event_props.insert("price".to_string(), Value::Integer((50 + (event_id % 100)) as i64));
    event_props.insert("volume".to_string(), Value::Integer((500 + (event_id % 1000)) as i64));
    event_props.insert("change".to_string(), Value::Number((event_id % 10) as f64));

    facts.set("event", Value::Object(event_props));
    facts
}

fn create_burst_event(burst_id: usize) -> Facts {
    let mut facts = Facts::new();
    let mut event_props = HashMap::new();

    event_props.insert("id".to_string(), Value::Integer(burst_id as i64));
    event_props.insert("price".to_string(), Value::Integer(150)); // Always trigger price rule
    event_props.insert("volume".to_string(), Value::Integer(1500)); // Always trigger volume rule
    event_props.insert("change".to_string(), Value::Number(10.0)); // Always trigger trend rule

    facts.set("event", Value::Object(event_props));
    facts
}
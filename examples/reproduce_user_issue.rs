use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::Value;
use rust_rule_engine::RuleEngineBuilder;
use std::collections::HashMap;

#[derive(Debug)]
struct ExecuteGRLRequest {
    grl_code: String,
    facts: HashMap<String, serde_json::Value>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate your exact request
    let mut request_facts = HashMap::new();
    request_facts.insert("age".to_string(), serde_json::json!(25));
    request_facts.insert("status".to_string(), serde_json::json!("pending"));

    let request = ExecuteGRLRequest {
        grl_code: r#"rule "Simple Test" salience 10 {
  when    (age >= 18)  then
    set(status, "approved");
}"#
        .to_string(),
        facts: request_facts,
    };

    println!("🚀 Executing GRL code with rust-rule-engine:");
    println!("📜 GRL: {}", request.grl_code);
    println!("📊 Facts: {:?}", request.facts);

    // Create engine config EXACTLY like your code
    let config = EngineConfig {
        max_cycles: 5,
        debug_mode: true,
        ..Default::default()
    };

    println!(
        "🔧 Engine config: max_cycles={}, debug_mode={}",
        config.max_cycles, config.debug_mode
    );

    // Build rule engine with inline GRL EXACTLY like your code
    let mut engine = match RuleEngineBuilder::new().with_inline_grl(&request.grl_code) {
        Ok(builder) => {
            println!("✅ GRL parsed successfully");
            builder.with_config(config).build()
        }
        Err(e) => {
            println!("❌ Failed to parse GRL: {}", e);
            return Err(e.into());
        }
    };

    // Create facts from request EXACTLY like your code
    let facts = Facts::new();

    // Convert JSON facts to rust-rule-engine Values EXACTLY like your code
    for (key, json_value) in &request.facts {
        let value = match json_value {
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Number(f)
                } else {
                    Value::String(n.to_string())
                }
            }
            serde_json::Value::Bool(b) => Value::Boolean(*b),
            serde_json::Value::Array(arr) => {
                let vec_values: Vec<Value> = arr
                    .iter()
                    .map(|v| match v {
                        serde_json::Value::String(s) => Value::String(s.clone()),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                Value::Integer(i)
                            } else if let Some(f) = n.as_f64() {
                                Value::Number(f)
                            } else {
                                Value::String(n.to_string())
                            }
                        }
                        serde_json::Value::Bool(b) => Value::Boolean(*b),
                        _ => Value::String(v.to_string()),
                    })
                    .collect();
                Value::Array(vec_values)
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    let val = match v {
                        serde_json::Value::String(s) => Value::String(s.clone()),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                Value::Integer(i)
                            } else if let Some(f) = n.as_f64() {
                                Value::Number(f)
                            } else {
                                Value::String(n.to_string())
                            }
                        }
                        serde_json::Value::Bool(b) => Value::Boolean(*b),
                        _ => Value::String(v.to_string()),
                    };
                    map.insert(k.clone(), val);
                }
                Value::Object(map)
            }
            serde_json::Value::Null => Value::String("null".to_string()),
        };

        if let Err(e) = facts.add_value(key, value.clone()) {
            println!("⚠️ Failed to add fact {}: {}", key, e);
        } else {
            println!("✅ Added fact: {} = {:?}", key, value);
        }
    }

    // Verify facts are set correctly
    println!("\n🔍 Verifying facts:");
    println!("   age: {:?}", facts.get("age"));
    println!("   status: {:?}", facts.get("status"));

    // Execute rules EXACTLY like your code
    println!("\n🚀 Starting rule execution...");
    let result = match engine.execute(&facts) {
        Ok(result) => {
            println!("✅ Execution completed successfully");
            result
        }
        Err(e) => {
            println!("❌ Rule execution failed: {}", e);
            return Err(e.into());
        }
    };

    println!(
        "🎯 Rule execution complete: {} rules fired in {} cycles",
        result.rules_fired, result.cycle_count
    );

    // Debug final state
    println!("\n📋 Final facts state:");
    println!("   age: {:?}", facts.get("age"));
    println!("   status: {:?}", facts.get("status"));

    if result.rules_fired == 0 {
        println!("\n❌ ISSUE REPRODUCED: 0 rules fired!");
        println!("🔍 Debugging engine state...");

        // Check if this is related to max_cycles
        println!("   Cycles run: {}", result.cycle_count);
        println!("   Max cycles: 5");

        if result.cycle_count >= 5 {
            println!("   ⚠️  Hit max_cycles limit - possible infinite loop prevention");
        }
    } else {
        println!("\n✅ SUCCESS: Rule fired as expected!");
    }

    Ok(())
}

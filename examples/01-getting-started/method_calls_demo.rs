use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::expression;
use rust_rule_engine::parser::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::fs;

// Helper: flatten nested objects in Facts into a new Facts instance with dotted keys
fn flatten_facts_for_eval(orig: &Facts) -> Facts {
    let flat = Facts::new();

    fn recurse(flat: &Facts, prefix: &str, val: &Value) {
        match val {
            Value::Object(map) => {
                for (k, v) in map.iter() {
                    let key = if prefix.is_empty() {
                        k.clone()
                    } else {
                        format!("{}.{}", prefix, k)
                    };

                    recurse(flat, &key, v);
                }
            }
            _ => {
                flat.set(prefix, val.clone());
            }
        }
    }

    let all = orig.get_all_facts();
    for (k, v) in all.iter() {
        recurse(&flat, k, v);
    }

    flat
}

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Method Calls Demo with Rule File");
    println!("===================================\n");

    // Read rules from file
    let rule_file_path = "examples/rules/01-basic/method_calls.grl";
    println!("📄 Reading rules from file: {}", rule_file_path);

    let rule_content = fs::read_to_string(rule_file_path)
        .map_err(|e| format!("Failed to read rule file: {}", e))?;

    println!("📋 Rule file content:");
    println!("---");
    println!("{}", rule_content);
    println!("---\n");

    // Create facts
    let facts = Facts::new();

    // TestCar data
    let mut test_car_props = HashMap::new();
    test_car_props.insert("Speed".to_string(), Value::Number(30.0));
    test_car_props.insert("MaxSpeed".to_string(), Value::Number(100.0));
    test_car_props.insert("SpeedIncrement".to_string(), Value::Number(10.0));
    test_car_props.insert("SpeedUp".to_string(), Value::Boolean(true));

    facts.add_value("TestCar", Value::Object(test_car_props))?;

    println!("🏁 Initial state:");
    if let Some(car) = facts.get("TestCar") {
        println!("   TestCar = {car:?}");
    }
    println!();

    // Create knowledge base and add rules from GRL file
    let kb = KnowledgeBase::new("MethodCallsDemo");

    // Parse rules from GRL file
    println!("🔧 Parsing GRL file content...");
    let rules = GRLParser::parse_rules(&rule_content)
        .map_err(|e| format!("Failed to parse GRL file: {:?}", e))?;

    println!("✅ Successfully parsed {} rules from file", rules.len());
    for rule in &rules {
        println!("   📋 Rule: {} (salience: {})", rule.name, rule.salience);
        let _ = kb.add_rule(rule.clone());
    }
    println!();

    // Create engine
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 5,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register custom functions for speed control
    engine.register_function("increaseSpeed", |_args, facts| {
        if let Some(Value::Object(obj)) = facts.get("TestCar") {
            let current_speed = obj.get("Speed").cloned().unwrap_or(Value::Number(0.0));
            let increment = obj
                .get("SpeedIncrement")
                .cloned()
                .unwrap_or(Value::Number(10.0));

            if let (Value::Number(speed), Value::Number(inc)) = (current_speed, increment) {
                let new_speed = speed + inc;
                println!("🚗 Increasing speed: {} -> {}", speed, new_speed);
                // In real implementation, this would update the fact
                return Ok(Value::Number(new_speed));
            }
        }
        Ok(Value::String("Speed increase attempted".to_string()))
    });

    engine.register_function("decreaseSpeed", |_args, facts| {
        if let Some(Value::Object(obj)) = facts.get("TestCar") {
            let current_speed = obj.get("Speed").cloned().unwrap_or(Value::Number(0.0));
            let increment = obj
                .get("SpeedIncrement")
                .cloned()
                .unwrap_or(Value::Number(10.0));

            if let (Value::Number(speed), Value::Number(inc)) = (current_speed, increment) {
                let new_speed = (speed - inc).max(0.0);
                println!("🚗 Decreasing speed: {} -> {}", speed, new_speed);
                // In real implementation, this would update the fact
                return Ok(Value::Number(new_speed));
            }
        }
        Ok(Value::String("Speed decrease attempted".to_string()))
    });

    // Register action handlers for method calls parsed as custom actions
    engine.register_action_handler("TestCar.setSpeed", |params, facts| {
        if let Some(val) = params.get("0").cloned() {
            // If value is an expression, try to evaluate it against facts
            let resolved = match val {
                Value::Expression(expr) => {
                    // Create a flattened facts view where nested object fields are available as dotted keys
                    let flat = flatten_facts_for_eval(facts);
                    match expression::evaluate_expression(&expr, &flat) {
                        Ok(v) => v,
                        Err(_) => Value::Expression(expr),
                    }
                }
                other => other,
            };

            let _ = facts.set_nested("TestCar.Speed", resolved.clone());
            println!("  ⚙️ Action Handler: TestCar.Speed set to {:?}", resolved);
        }
        Ok(())
    });

    engine.register_action_handler("TestCar.setSpeedUp", |params, facts| {
        if let Some(val) = params.get("0").cloned() {
            let _ = facts.set_nested("TestCar.SpeedUp", val);
            println!(
                "  ⚙️ Action Handler: TestCar.SpeedUp set to {:?}",
                params.get("0")
            );
        }
        Ok(())
    });

    // Short-name handlers in case parser emits method calls as 'setSpeed'/'setSpeedUp'
    engine.register_action_handler("setSpeed", |params, facts| {
        if let Some(val) = params.get("0").cloned() {
            let resolved = match val {
                Value::Expression(expr) => {
                    let flat = flatten_facts_for_eval(facts);
                    match expression::evaluate_expression(&expr, &flat) {
                        Ok(v) => v,
                        Err(_) => Value::Expression(expr),
                    }
                }
                other => other,
            };

            let _ = facts.set_nested("TestCar.Speed", resolved.clone());
            println!(
                "  ⚙️ Action Handler (short): TestCar.Speed set to {:?}",
                resolved
            );
        }
        Ok(())
    });

    engine.register_action_handler("setSpeedUp", |params, facts| {
        if let Some(val) = params.get("0").cloned() {
            let _ = facts.set_nested("TestCar.SpeedUp", val);
            println!(
                "  ⚙️ Action Handler (short): TestCar.SpeedUp set to {:?}",
                params.get("0")
            );
        }
        Ok(())
    });

    // Execute rules
    println!("🚀 Executing method calls rules from file...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Method Calls Execution Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    println!("\n🏁 Final state:");
    if let Some(car) = facts.get("TestCar") {
        println!("   TestCar = {car:?}");
    }

    println!("\n🎯 Method Calls from Rule File Demonstrated:");
    println!("   📄 Rules defined in external .grl file");
    println!("   🔧 Method calls: setSpeed(), setSpeedUp()");
    println!("   📞 Custom functions: increaseSpeed(), decreaseSpeed()");
    println!("   🚗 Speed control simulation");
    println!("   ⚡ Salience-based rule execution order");

    Ok(())
}

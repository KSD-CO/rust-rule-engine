use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Custom Function Calls Demo");
    println!("=============================\n");

    // Create facts
    let facts = Facts::new();

    // Car data
    let mut car_props = HashMap::new();
    car_props.insert("Speed".to_string(), Value::Number(60.0));
    car_props.insert("MaxSpeed".to_string(), Value::Number(120.0));
    car_props.insert("Engine".to_string(), Value::String("V6".to_string()));
    car_props.insert("IsRunning".to_string(), Value::Boolean(true));

    // Driver data
    let mut driver_props = HashMap::new();
    driver_props.insert("Name".to_string(), Value::String("John Doe".to_string()));
    driver_props.insert("Experience".to_string(), Value::Integer(5));
    driver_props.insert("License".to_string(), Value::String("VALID".to_string()));

    facts.add_value("Car", Value::Object(car_props))?;
    facts.add_value("Driver", Value::Object(driver_props))?;

    println!("🏁 Initial state:");
    if let Some(car) = facts.get("Car") {
        println!("   Car = {car:?}");
    }
    if let Some(driver) = facts.get("Driver") {
        println!("   Driver = {driver:?}");
    }
    println!();

    // Create knowledge base
    let kb = KnowledgeBase::new("CustomFunctionRules");

    // Rule 1: Speed Check with custom function
    let speed_check_rule = Rule::new(
        "SpeedCheck".to_string(),
        ConditionGroup::single(Condition::new(
            "Car.Speed".to_string(),
            Operator::GreaterThan,
            Value::Number(80.0),
        )),
        vec![
            ActionType::Custom {
                action_type: "checkSpeedLimit".to_string(),
                params: std::collections::HashMap::from([("args", vec![Value::String("Car.Speed".to_string()), Value::Number(80.0)],
            },
            ActionType::Custom {
                action_type: "sendAlert".to_string(),
                params: std::collections::HashMap::from([("args", vec![
                    Value::String("Speed limit exceeded!".to_string()),
                    Value::String("Driver.Name".to_string()),
                ],
            },
        ],
    )
    .with_salience(20);

    // Rule 2: Driver validation
    let driver_validation_rule = Rule::new(
        "DriverValidation".to_string(),
        ConditionGroup::single(Condition::new(
            "Driver.License".to_string(),
            Operator::Equal,
            Value::String("VALID".to_string()),
        )),
        vec![
            ActionType::Custom {
                action_type: "validateDriver".to_string(),
                params: std::collections::HashMap::from([("args", vec![
                    Value::String("Driver.Name".to_string()),
                    Value::String("Driver.Experience".to_string()),
                ],
            },
            ActionType::Custom {
                action_type: "calculateInsurance".to_string(),
                params: std::collections::HashMap::from([("args", vec![
                    Value::String("Driver.Experience".to_string()),
                    Value::String("Car.Engine".to_string()),
                ],
            },
        ],
    )
    .with_salience(15);

    // Rule 3: Engine diagnostics
    let engine_diagnostics_rule = Rule::new(
        "EngineDiagnostics".to_string(),
        ConditionGroup::single(Condition::new(
            "Car.IsRunning".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![
            ActionType::Custom {
                action_type: "performDiagnostics".to_string(),
                params: std::collections::HashMap::from([("args", vec![
                    Value::String("Car.Engine".to_string()),
                    Value::String("Car.Speed".to_string()),
                ],
            },
            ActionType::Custom {
                action_type: "optimizePerformance".to_string(),
                params: std::collections::HashMap::from([("args", vec![
                    Value::String("Car.Speed".to_string()),
                    Value::String("Car.MaxSpeed".to_string()),
                ],
            },
        ],
    )
    .with_salience(10);

    // Add rules to knowledge base
    let _ = kb.add_rule(speed_check_rule);
    let _ = kb.add_rule(driver_validation_rule);
    let _ = kb.add_rule(engine_diagnostics_rule);

    // Create engine
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 3,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register custom functions
    println!("📝 Registering custom functions...");

    // Function 1: Check speed limit
    engine.register_function("checkSpeedLimit", |args, facts| {
        let _speed_field = args.get(0).unwrap().to_string();
        let limit = args.get(1).unwrap();

        // Get actual speed value from facts
        let speed = if let Some(car) = facts.get("Car") {
            if let Value::Object(obj) = car {
                obj.get("Speed").cloned().unwrap_or(Value::Number(0.0))
            } else {
                Value::Number(0.0)
            }
        } else {
            Value::Number(0.0)
        };

        let result = format!("Speed check: {:?} vs limit {:?}", speed, limit);
        println!("🚦 {}", result);
        Ok(Value::String(result))
    });

    // Function 2: Send alert
    engine.register_function("sendAlert", |args, facts| {
        let message = args.get(0).unwrap().to_string();
        let _driver_field = args.get(1).unwrap().to_string();

        // Get driver name from facts
        let driver_name = if let Some(driver) = facts.get("Driver") {
            if let Value::Object(obj) = driver {
                obj.get("Name")
                    .cloned()
                    .unwrap_or(Value::String("Unknown".to_string()))
            } else {
                Value::String("Unknown".to_string())
            }
        } else {
            Value::String("Unknown".to_string())
        };

        let alert = format!("🚨 ALERT to {:?}: {}", driver_name, message);
        println!("{}", alert);
        Ok(Value::String(alert))
    });

    // Function 3: Validate driver
    engine.register_function("validateDriver", |args, _facts| {
        let name_field = args.get(0).unwrap().to_string();
        let exp_field = args.get(1).unwrap().to_string();

        let result = format!(
            "✅ Driver validation passed for {} (experience: {})",
            name_field, exp_field
        );
        println!("{}", result);
        Ok(Value::String(result))
    });

    // Function 4: Calculate insurance
    engine.register_function("calculateInsurance", |args, _facts| {
        let exp_field = args.get(0).unwrap().to_string();
        let engine_field = args.get(1).unwrap().to_string();

        let result = format!(
            "💰 Insurance calculated: Experience {} + Engine {} = Premium rate",
            exp_field, engine_field
        );
        println!("{}", result);
        Ok(Value::String(result))
    });

    // Function 5: Perform diagnostics
    engine.register_function("performDiagnostics", |args, _facts| {
        let engine_field = args.get(0).unwrap().to_string();
        let speed_field = args.get(1).unwrap().to_string();

        let result = format!(
            "🔧 Diagnostics: Engine {} running at speed {} - All systems OK",
            engine_field, speed_field
        );
        println!("{}", result);
        Ok(Value::String(result))
    });

    // Function 6: Optimize performance
    engine.register_function("optimizePerformance", |args, _facts| {
        let current_speed = args.get(0).unwrap().to_string();
        let max_speed = args.get(1).unwrap().to_string();

        let result = format!(
            "⚡ Performance optimization: Current {} / Max {} - Efficiency tuned",
            current_speed, max_speed
        );
        println!("{}", result);
        Ok(Value::String(result))
    });

    println!("✅ Registered {} custom functions:", 6);
    println!("   🚦 checkSpeedLimit - Check if speed exceeds limit");
    println!("   🚨 sendAlert - Send alert message to driver");
    println!("   ✅ validateDriver - Validate driver credentials");
    println!("   💰 calculateInsurance - Calculate insurance premium");
    println!("   🔧 performDiagnostics - Run engine diagnostics");
    println!("   ⚡ optimizePerformance - Optimize engine performance");
    println!();

    // Execute rules
    println!("🚀 Executing rules with custom functions...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Custom Function Execution Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    println!("\n🏁 Final state:");
    if let Some(car) = facts.get("Car") {
        println!("   Car = {car:?}");
    }
    if let Some(driver) = facts.get("Driver") {
        println!("   Driver = {driver:?}");
    }

    println!("\n🎯 Custom Function Calls Demonstrated:");
    println!("   📞 User-defined functions called from rules");
    println!("   🔧 Custom business logic execution");
    println!("   🎪 Function registry system");
    println!("   📋 Rule-based custom function invocation");
    println!("   ⚡ Real-time function parameter resolution");

    Ok(())
}

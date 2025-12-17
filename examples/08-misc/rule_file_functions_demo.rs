use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser as SimpleGRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::fs;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Custom Functions from Rule File Demo");
    println!("=======================================\n");

    // Read rules from file
    // NOTE: the GRL files are located under examples/rules/04-use-cases/
    let rule_file_path = "examples/rules/04-use-cases/car_functions.grl";
    println!("📄 Reading rules from file: {}", rule_file_path);

    let rule_content = fs::read_to_string(rule_file_path)
        .map_err(|e| format!("Failed to read rule file: {}", e))?;

    println!("📋 Rule file content:");
    println!("---");
    println!("{}", rule_content);
    println!("---\n");

    // Create facts
    let facts = Facts::new();

    // Car data - set speed > 80 to trigger speed limit rule
    let mut car_props = HashMap::new();
    car_props.insert("Speed".to_string(), Value::Number(95.0)); // Above speed limit
    car_props.insert("MaxSpeed".to_string(), Value::Number(120.0));
    car_props.insert("Engine".to_string(), Value::String("V6".to_string()));
    car_props.insert("IsRunning".to_string(), Value::Boolean(true));

    // Driver data
    let mut driver_props = HashMap::new();
    driver_props.insert("Name".to_string(), Value::String("John Doe".to_string()));
    driver_props.insert("Experience".to_string(), Value::Integer(7)); // Experienced driver
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
    let kb = KnowledgeBase::new("RuleFileDemo");

    // Parse GRL file and add rules to knowledge base
    println!("🔧 Parsing GRL file content...");
    let parsed_rules = SimpleGRLParser::parse_rules(&rule_content)
        .map_err(|e| format!("Failed to parse GRL file: {:?}", e))?;

    println!(
        "✅ Successfully parsed {} rules from file",
        parsed_rules.len()
    );
    for rule in parsed_rules {
        println!("   📋 Rule: {} (salience: {})", rule.name, rule.salience);
        let _ = kb.add_rule(rule);
    }
    println!();

    // Create engine
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 3,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register all custom functions called from the rule file
    println!("📝 Registering custom functions from rule file...");

    // Functions from SpeedLimitCheck rule
    engine.register_function("checkSpeedLimit", |args, facts| {
        let _speed_field = args.first().unwrap().to_string();
        let limit = args.get(1).unwrap();

        let speed = if let Some(Value::Object(obj)) = facts.get("Car") {
            obj.get("Speed").cloned().unwrap_or(Value::Number(0.0))
        } else {
            Value::Number(0.0)
        };

        let result = format!("🚦 Speed check: {:?} vs limit {:?}", speed, limit);
        println!("{}", result);
        Ok(Value::String(result))
    });
    // Also register action handlers for the same names because parser maps some
    // 'then' function calls to ActionType::Custom, not function calls. This
    // ensures both styles are handled by this demo.
    engine.register_action_handler("checkSpeedLimit", |params, _facts| {
        // params use numeric keys "0", "1" etc.
        let speed = params.get("0").cloned().unwrap_or(Value::Number(0.0));
        let limit = params.get("1").cloned().unwrap_or(Value::Number(0.0));
        println!("🚦 Action Handler: Speed check: {:?} vs {:?}", speed, limit);
        Ok(())
    });

    engine.register_action_handler("sendAlert", |params, _facts| {
        let message = params.get("0").map(|v| v.to_string()).unwrap_or_default();
        println!("🚨 Action Handler Alert: {}", message);
        Ok(())
    });

    engine.register_action_handler("validateDriver", |params, _facts| {
        println!("✅ Action Handler: validateDriver called with {:?}", params);
        Ok(())
    });

    engine.register_action_handler("calculateInsurance", |params, _facts| {
        println!(
            "💰 Action Handler: calculateInsurance called with {:?}",
            params
        );
        Ok(())
    });

    engine.register_action_handler("performDiagnostics", |params, _facts| {
        println!(
            "🔧 Action Handler: performDiagnostics called with {:?}",
            params
        );
        Ok(())
    });

    engine.register_action_handler("optimizePerformance", |params, _facts| {
        println!(
            "⚡ Action Handler: optimizePerformance called with {:?}",
            params
        );
        Ok(())
    });

    engine.register_action_handler("scheduleMaintenanceCheck", |params, _facts| {
        println!(
            "🔧 Action Handler: scheduleMaintenanceCheck called with {:?}",
            params
        );
        Ok(())
    });

    engine.register_action_handler("updateMaintenanceRecord", |params, _facts| {
        println!(
            "📋 Action Handler: updateMaintenanceRecord called with {:?}",
            params
        );
        Ok(())
    });

    engine.register_action_handler("emergencyStop", |params, _facts| {
        println!("⛔ Action Handler: emergencyStop called with {:?}", params);
        Ok(())
    });
    engine.register_function("sendAlert", |args, facts| {
        let message = args.first().unwrap().to_string();
        let _driver_field = args.get(1).unwrap().to_string();

        let driver_name = if let Some(Value::Object(obj)) = facts.get("Driver") {
            obj.get("Name")
                .cloned()
                .unwrap_or(Value::String("Unknown".to_string()))
        } else {
            Value::String("Unknown".to_string())
        };

        let alert = format!("🚨 ALERT to {:?}: {}", driver_name, message);
        println!("{}", alert);
        Ok(Value::String(alert))
    });

    // Functions from DriverValidation rule
    engine.register_function("validateDriver", |args, _facts| {
        let name_field = args.first().unwrap().to_string();
        let exp_field = args.get(1).unwrap().to_string();

        let result = format!(
            "✅ Driver validation: {} (experience: {})",
            name_field, exp_field
        );
        println!("{}", result);
        Ok(Value::String(result))
    });

    engine.register_function("calculateInsurance", |args, _facts| {
        let exp_field = args.first().unwrap().to_string();
        let engine_field = args.get(1).unwrap().to_string();

        let result = format!(
            "💰 Insurance: Experience {} + Engine {} = Premium",
            exp_field, engine_field
        );
        println!("{}", result);
        Ok(Value::String(result))
    });

    // Functions from EngineDiagnostics rule
    engine.register_function("performDiagnostics", |args, _facts| {
        let engine_field = args.first().unwrap().to_string();
        let speed_field = args.get(1).unwrap().to_string();

        let result = format!(
            "🔧 Diagnostics: Engine {} at speed {} - OK",
            engine_field, speed_field
        );
        println!("{}", result);
        Ok(Value::String(result))
    });

    engine.register_function("optimizePerformance", |args, _facts| {
        let current_speed = args.first().unwrap().to_string();
        let max_speed = args.get(1).unwrap().to_string();

        let result = format!(
            "⚡ Performance: {} / {} - Optimized",
            current_speed, max_speed
        );
        println!("{}", result);
        Ok(Value::String(result))
    });

    // Functions from MaintenanceCheck rule
    engine.register_function("scheduleMaintenanceCheck", |args, _facts| {
        let engine_field = args.first().unwrap().to_string();
        let exp_field = args.get(1).unwrap().to_string();

        let result = format!(
            "🔧 Maintenance scheduled: Engine {} (driver exp: {})",
            engine_field, exp_field
        );
        println!("{}", result);
        Ok(Value::String(result))
    });

    engine.register_function("updateMaintenanceRecord", |args, _facts| {
        let name_field = args.first().unwrap().to_string();
        let engine_field = args.get(1).unwrap().to_string();

        let result = format!("📋 Record updated: {} - {}", name_field, engine_field);
        println!("{}", result);
        Ok(Value::String(result))
    });

    println!("✅ Registered {} custom functions from rule file:", 8);
    println!("   🚦 checkSpeedLimit");
    println!("   🚨 sendAlert");
    println!("   ✅ validateDriver");
    println!("   💰 calculateInsurance");
    println!("   🔧 performDiagnostics");
    println!("   ⚡ optimizePerformance");
    println!("   🔧 scheduleMaintenanceCheck");
    println!("   📋 updateMaintenanceRecord");
    println!();

    // Execute rules
    println!("🚀 Executing rules from file...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Rule File Execution Results:");
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

    println!("\n🎯 Rule File Custom Functions Demonstrated:");
    println!("   📄 Rules defined in external .grl file");
    println!("   📞 Custom functions called from rule file");
    println!("   🔧 Business logic separated from rule definitions");
    println!("   📋 File-based rule management");
    println!("   ⚡ Function registry with rule file integration");

    Ok(())
}

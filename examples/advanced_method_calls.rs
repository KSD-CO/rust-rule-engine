use rust_rule_engine::*;
use std::error::Error;

fn main() -> std::result::Result<(), Box<dyn Error>> {
    demo_method_calls()?;
    Ok(())
}

fn demo_method_calls() -> std::result::Result<(), Box<dyn Error>> {
    println!("=== Demo: Advanced Method Calls ===");

    // Create Knowledge Base
    let kb = KnowledgeBase::new("MethodCallsDemo");

    // Define rule with simpler condition first
    let speedup_rule = r#"rule "SpeedUp" salience 10 { 
        when TestCar.speedUp == true 
        then TestCar.Speed = 70; 
    }"#;

    // Parse and add rule
    let rules = GRLParser::parse_rules(speedup_rule)?;
    let rule = rules.into_iter().next().unwrap();
    let _ = kb.add_rule(rule);

    // Create engine with debug mode
    let config = EngineConfig {
        max_cycles: 10,
        timeout: None,
        enable_stats: true,
        debug_mode: true,
    };
    let engine = RustRuleEngine::with_config(kb, config);

    // Create facts
    let facts = Facts::new();

    // Create TestCar object using helper
    let test_car = FactHelper::create_test_car(
        true,  // speedUp
        50.0,  // speed
        100.0, // maxSpeed
        10.0,  // speedIncrement
    );

    // Create DistanceRecord object using helper
    let distance_record = FactHelper::create_distance_record(0.0);

    // Add facts
    facts.add_value("TestCar", test_car)?;
    facts.add_value("DistanceRecord", distance_record)?;

    println!("\n🏁 Initial state:");
    if let Some(car) = facts.get("TestCar") {
        if let Some(speed) = car.get_property("Speed") {
            println!("   TestCar.Speed = {:?}", speed);
        }
        if let Some(speed_up) = car.get_property("speedUp") {
            println!("   TestCar.speedUp = {:?}", speed_up);
        }
    }

    // Execute rules multiple cycles to see progression
    println!("\n🚀 Executing SpeedUp rule multiple times...");
    for i in 1..=3 {
        println!("\n--- Cycle {} ---", i);
        let result = engine.execute(&facts)?;

        println!("Rules fired: {}", result.rules_fired);

        if let Some(car) = facts.get("TestCar") {
            if let Some(speed) = car.get_property("Speed") {
                println!("TestCar.Speed after cycle {}: {:?}", i, speed);
            }
        }

        // Check if we reached max speed
        if let Some(car) = facts.get("TestCar") {
            if let Some(speed) = car.get_property("speed") {
                if let Some(max_speed) = car.get_property("maxSpeed") {
                    if speed.to_number() >= max_speed.to_number() {
                        println!("🏁 Max speed reached!");
                        break;
                    }
                }
            }
        }
    }

    println!("\n📊 Final Results:");
    if let Some(car) = facts.get("TestCar") {
        if let Some(speed) = car.get_property("Speed") {
            println!("   Final TestCar.Speed = {:?}", speed);
        }
    }

    Ok(())
}

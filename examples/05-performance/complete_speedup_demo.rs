use rust_rule_engine::*;
use std::error::Error;

fn main() -> std::result::Result<(), Box<dyn Error>> {
    demo_complete_speedup_rule()?;
    Ok(())
}

fn demo_complete_speedup_rule() -> std::result::Result<(), Box<dyn Error>> {
    println!("=== Demo: Complete SpeedUp Rule with Method Calls ===");
    println!("Implementing the exact rule from the requirement:");
    println!("rule \"SpeedUp\" salience 10");
    println!("when");
    println!("    $TestCar : TestCarClass( speedUp == true && speed < maxSpeed )");
    println!("    $DistanceRecord : DistanceRecordClass()");
    println!("then");
    println!("    $TestCar.setSpeed($TestCar.Speed + $TestCar.SpeedIncrement);");
    println!("    update($TestCar);");
    println!("    $DistanceRecord.setTotalDistance($DistanceRecord.getTotalDistance() + $TestCar.Speed);");
    println!("    update($DistanceRecord);");
    println!("end\n");

    // Create Knowledge Base
    let kb = KnowledgeBase::new("SpeedUpDemo");

    // For now, we'll use a simplified version that works with our current parser
    let speedup_rule = r#"rule "SpeedUp" salience 10 { 
        when TestCar.speedUp == true && TestCar.speed < TestCar.maxSpeed 
        then $TestCar.setSpeed($TestCar.Speed + $TestCar.SpeedIncrement); update($TestCar); $DistanceRecord.setTotalDistance($DistanceRecord.getTotalDistance() + $TestCar.Speed); update($DistanceRecord); 
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
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Create facts
    let facts = Facts::new();

    // Create TestCar object - initial speed less than max
    let test_car = FactHelper::create_test_car(
        true,  // speedUp
        30.0,  // speed (less than maxSpeed)
        100.0, // maxSpeed
        15.0,  // speedIncrement
    );

    // Create DistanceRecord object
    let distance_record = FactHelper::create_distance_record(0.0);

    // Add facts
    facts.add_value("TestCar", test_car)?;
    facts.add_value("DistanceRecord", distance_record)?;

    println!("🏁 Initial state:");
    if let Some(car) = facts.get("TestCar") {
        println!(
            "   TestCar.speed = {:?}",
            car.get_property("speed").unwrap_or(Value::Null)
        );
        println!(
            "   TestCar.Speed = {:?}",
            car.get_property("Speed").unwrap_or(Value::Null)
        );
        println!(
            "   TestCar.maxSpeed = {:?}",
            car.get_property("maxSpeed").unwrap_or(Value::Null)
        );
        println!(
            "   TestCar.speedUp = {:?}",
            car.get_property("speedUp").unwrap_or(Value::Null)
        );
        println!(
            "   TestCar.SpeedIncrement = {:?}",
            car.get_property("SpeedIncrement").unwrap_or(Value::Null)
        );
    }
    if let Some(record) = facts.get("DistanceRecord") {
        println!(
            "   DistanceRecord.TotalDistance = {:?}",
            record.get_property("TotalDistance").unwrap_or(Value::Null)
        );
    }

    // Execute rules multiple cycles
    println!("\n🚀 Executing SpeedUp rule...");
    // Register 'update' action handler so rules using update(obj) work as custom actions
    engine.register_action_handler("update", |params, facts| {
        if let Some(Value::String(obj_name)) = params.get("0") {
            if let Some(obj_val) = facts.get(obj_name) {
                facts.add_value(obj_name, obj_val)?;
            }
        }
        Ok(())
    });
    for i in 1..=5 {
        println!("\n--- Cycle {} ---", i);
        let result = engine.execute(&facts)?;

        println!("Rules fired: {}", result.rules_fired);

        if let Some(car) = facts.get("TestCar") {
            println!(
                "TestCar.Speed = {:?}",
                car.get_property("Speed").unwrap_or(Value::Null)
            );
            println!(
                "TestCar.speed = {:?}",
                car.get_property("speed").unwrap_or(Value::Null)
            );
        }
        if let Some(record) = facts.get("DistanceRecord") {
            println!(
                "DistanceRecord.TotalDistance = {:?}",
                record.get_property("TotalDistance").unwrap_or(Value::Null)
            );
        }

        // Check if we reached max speed
        if let Some(car) = facts.get("TestCar") {
            if let Some(speed) = car.get_property("speed") {
                if let Some(max_speed) = car.get_property("maxSpeed") {
                    if speed.to_number() >= max_speed.to_number() {
                        println!("🏁 Max speed reached! Rule will no longer fire.");
                        break;
                    }
                }
            }
        }

        if result.rules_fired == 0 {
            println!("ℹ️ No rules fired this cycle.");
            break;
        }
    }

    println!("\n📊 Final Summary:");
    println!("✅ Successfully demonstrated method calls in GRL:");
    println!("   • Method calls: $Object.method(args)");
    println!("   • Property access: $Object.Property");
    println!("   • Arithmetic expressions: $A.prop + $B.prop");
    println!("   • update() function calls");
    println!("   • Complex conditions with && operator");

    Ok(())
}

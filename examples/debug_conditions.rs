use rust_rule_engine::*;
use std::error::Error;

fn main() -> std::result::Result<(), Box<dyn Error>> {
    test_simple_conditions()?;
    Ok(())
}

fn test_simple_conditions() -> std::result::Result<(), Box<dyn Error>> {
    println!("=== Testing Simple Conditions ===");

    // Create Knowledge Base
    let kb = KnowledgeBase::new("TestConditions");

    // Test compound condition with &&
    let compound_rule = r#"rule "CompoundRule" { when TestCar.speedUp == true && TestCar.speed < TestCar.maxSpeed then TestCar.result = "compound_fired"; }"#;

    println!("Testing rule: {}", compound_rule);

    // Parse and add rule
    let rules = GRLParser::parse_rules(compound_rule)?;
    let rule = rules.into_iter().next().unwrap();
    println!("Parsed rule: {:?}", rule);

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

    // Create simple TestCar object with all needed properties
    let test_car = FactHelper::create_object(vec![
        ("speedUp", Value::Boolean(true)),
        ("speed", Value::Number(30.0)),
        ("maxSpeed", Value::Number(100.0)),
        ("result", Value::String("not_fired".to_string())),
    ]);

    facts.add_value("TestCar", test_car)?;

    println!("\n🏁 Before execution:");
    if let Some(car) = facts.get("TestCar") {
        println!("   TestCar.speedUp = {:?}", car.get_property("speedUp"));
        println!("   TestCar.speed = {:?}", car.get_property("speed"));
        println!("   TestCar.maxSpeed = {:?}", car.get_property("maxSpeed"));
        println!("   TestCar.result = {:?}", car.get_property("result"));
    }

    // Execute rules
    println!("\n🚀 Executing rule...");
    let result = engine.execute(&facts)?;

    println!("Rules fired: {}", result.rules_fired);

    println!("\n🏁 After execution:");
    if let Some(car) = facts.get("TestCar") {
        println!("   TestCar.speedUp = {:?}", car.get_property("speedUp"));
        println!("   TestCar.result = {:?}", car.get_property("result"));
    }

    Ok(())
}

use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Demo: Generic Method Calls ===\n");

    // Create objects with various properties
    let mut car_props = HashMap::new();
    car_props.insert("Speed".to_string(), Value::Number(50.0));
    car_props.insert("Fuel".to_string(), Value::Number(80.0));
    car_props.insert("Brand".to_string(), Value::String("Toyota".to_string()));

    let mut account_props = HashMap::new();
    account_props.insert("Balance".to_string(), Value::Number(1000.0));
    account_props.insert("Owner".to_string(), Value::String("John Doe".to_string()));
    account_props.insert("Active".to_string(), Value::Boolean(true));

    // Create facts
    let facts = Facts::new();
    facts.add_value("Car", Value::Object(car_props))?;
    facts.add_value("Account", Value::Object(account_props))?;

    println!("🏁 Initial state:");
    if let Some(car) = facts.get("Car") {
        println!("   Car = {car:?}");
    }
    if let Some(account) = facts.get("Account") {
        println!("   Account = {account:?}");
    }
    println!();

    // Create knowledge base
    let kb = KnowledgeBase::new("GenericMethodDemo");

    // Rule 1: Speed up car if speed < 60
    let speed_rule = Rule::new(
        "SpeedUpCar".to_string(),
        ConditionGroup::single(Condition::new(
            "Car.Speed".to_string(),
            Operator::LessThan,
            Value::Number(60.0),
        )),
        vec![ActionType::MethodCall {
            object: "Car".to_string(),
            method: "setSpeed".to_string(), // Generic setter
            args: vec![Value::Number(75.0)],
        }],
    )
    .with_salience(10);

    // Rule 2: Withdraw from account if balance > 500
    let withdraw_rule = Rule::new(
        "WithdrawMoney".to_string(),
        ConditionGroup::single(Condition::new(
            "Account.Balance".to_string(),
            Operator::GreaterThan,
            Value::Number(500.0),
        )),
        vec![ActionType::MethodCall {
            object: "Account".to_string(),
            method: "setBalance".to_string(), // Generic setter
            args: vec![Value::Number(800.0)],
        }],
    )
    .with_salience(5);

    // Rule 3: Update car brand using generic setter
    let brand_rule = Rule::new(
        "UpdateBrand".to_string(),
        ConditionGroup::single(Condition::new(
            "Car.Brand".to_string(),
            Operator::Equal,
            Value::String("Toyota".to_string()),
        )),
        vec![ActionType::MethodCall {
            object: "Car".to_string(),
            method: "setBrand".to_string(), // Generic setter
            args: vec![Value::String("Honda".to_string())],
        }],
    )
    .with_salience(3);

    // Add rules to knowledge base
    let _ = kb.add_rule(speed_rule);
    let _ = kb.add_rule(withdraw_rule);
    let _ = kb.add_rule(brand_rule);

    // Create engine with debug mode
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 10,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Execute rules
    println!("🚀 Executing rules with generic method calls...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Execution Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    println!("\n🏁 Final state:");
    if let Some(car) = facts.get("Car") {
        println!("   Car = {car:?}");
    }
    if let Some(account) = facts.get("Account") {
        println!("   Account = {account:?}");
    }

    // Demonstrate generic getters
    println!("\n🔍 Testing generic getters:");
    if let Some(car_value) = facts.get("Car") {
        if let Value::Object(car_obj) = car_value {
            println!(
                "   Car.Speed via getter would return: {:?}",
                car_obj.get("Speed")
            );
            println!(
                "   Car.Brand via getter would return: {:?}",
                car_obj.get("Brand")
            );
            println!(
                "   Car.Fuel via getter would return: {:?}",
                car_obj.get("Fuel")
            );
        }
    }

    Ok(())
}

use rust_rule_engine::{Facts, RuleEngineBuilder, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Debugging No-Loop Behavior");
    println!("===============================");

    let grl_rules = r#"
        rule "TestNoLoop" no-loop salience 10 {
            when
                Customer.tier == "VIP" && Order.amount > 100
            then
                Order.total = 90;
                log("Applying 10% discount");
        }
    "#;

    // Create engine with default config
    let mut engine = RuleEngineBuilder::new().with_inline_grl(grl_rules)?.build();

    // Create facts
    let facts = Facts::new();

    let mut customer = HashMap::new();
    customer.insert("tier".to_string(), Value::String("VIP".to_string()));
    facts.add_value("Customer", Value::Object(customer))?;

    let mut order = HashMap::new();
    order.insert("amount".to_string(), Value::Integer(105));
    order.insert(
        "total".to_string(),
        Value::String("initial_value".to_string()),
    );
    facts.add_value("Order", Value::Object(order))?;

    println!("🏁 Initial state:");
    println!("   Customer.tier = VIP");
    println!("   Order.amount = 105");
    println!("   Order.total = initial_value");
    println!();

    // Execute rules
    let result = engine.execute(&facts)?;

    println!();
    println!("📊 Execution Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);

    println!();
    println!("🏁 Final state:");
    // Print facts directly since get_value method doesn't exist
    println!("   Facts updated during execution");

    Ok(())
}

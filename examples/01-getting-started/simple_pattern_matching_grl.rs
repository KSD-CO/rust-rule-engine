use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Simple Pattern Matching from GRL Demo");
    println!("=========================================");

    // Read simple GRL file
    let grl_content = fs::read_to_string("examples/rules/01-basic/simple_patterns.grl")?;
    println!("📄 Reading rules from: examples/rules/01-basic/simple_patterns.grl");

    // Parse rules from GRL
    let rules = GRLParser::parse_rules(&grl_content)?;
    println!("✅ Successfully parsed {} rules from GRL file", rules.len());

    for rule in &rules {
        println!("  📋 Rule: {} (salience: {})", rule.name, rule.salience);
    }

    println!("\n🔥 Testing Individual Pattern Types");
    println!("=====================================");

    // Test EXISTS pattern
    test_exists_pattern(&rules)?;

    // Test NOT pattern
    test_not_pattern(&rules)?;

    // Test FORALL pattern
    test_forall_pattern(&rules)?;

    println!("\n🎉 Simple Pattern Matching Demo completed successfully!");
    Ok(())
}

fn test_exists_pattern(
    rules: &[rust_rule_engine::engine::rule::Rule],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 1: EXISTS Pattern - VIP Customer Detection");
    println!("---------------------------------------------------");

    let kb = KnowledgeBase::new("ExistsTest");

    // Add the first rule (should be EXISTS pattern)
    let exists_rule = &rules[0];
    println!(
        "Testing rule: {} (salience: {})",
        exists_rule.name, exists_rule.salience
    );
    kb.add_rule(exists_rule.clone())?;

    let mut engine = RustRuleEngine::with_config(
        kb,
        EngineConfig {
            debug_mode: true,
            max_cycles: 1,
            ..Default::default()
        },
    );

    let facts = Facts::new();

    // Add customers - one VIP
    let mut customer1 = HashMap::new();
    customer1.insert("tier".to_string(), Value::String("Regular".to_string()));
    facts.add_value("Customer1", Value::Object(customer1))?;

    let mut customer2 = HashMap::new();
    customer2.insert("tier".to_string(), Value::String("VIP".to_string()));
    facts.add_value("Customer2", Value::Object(customer2))?;

    // Add system state
    facts.set(
        "System",
        Value::Object(HashMap::from([(
            "vipFound".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial facts:");
    println!("  Customer1.tier = Regular");
    println!("  Customer2.tier = VIP");
    println!("  System.vipFound = false");

    let result = engine.execute(&facts)?;
    println!("✅ EXISTS test: {} rules fired", result.rules_fired);

    if let Some(system) = facts.get("System") {
        println!("Final System state: {:?}", system);
    }

    Ok(())
}

fn test_not_pattern(
    rules: &[rust_rule_engine::engine::rule::Rule],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 2: NOT Pattern - No Pending Orders");
    println!("--------------------------------------------");

    let kb = KnowledgeBase::new("NotTest");

    // Add the second rule (should be NOT pattern)
    let not_rule = &rules[1];
    println!(
        "Testing rule: {} (salience: {})",
        not_rule.name, not_rule.salience
    );
    kb.add_rule(not_rule.clone())?;

    let mut engine = RustRuleEngine::with_config(
        kb,
        EngineConfig {
            debug_mode: true,
            max_cycles: 1,
            ..Default::default()
        },
    );

    let facts = Facts::new();

    // Add only completed orders (no pending)
    let mut order1 = HashMap::new();
    order1.insert("status".to_string(), Value::String("completed".to_string()));
    facts.add_value("Order1", Value::Object(order1))?;

    let mut order2 = HashMap::new();
    order2.insert("status".to_string(), Value::String("shipped".to_string()));
    facts.add_value("Order2", Value::Object(order2))?;

    // Add marketing state
    facts.set(
        "Marketing",
        Value::Object(HashMap::from([(
            "sendEmails".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial facts:");
    println!("  Order1.status = completed");
    println!("  Order2.status = shipped");
    println!("  No pending orders!");
    println!("  Marketing.sendEmails = false");

    let result = engine.execute(&facts)?;
    println!("✅ NOT test: {} rules fired", result.rules_fired);

    if let Some(marketing) = facts.get("Marketing") {
        println!("Final Marketing state: {:?}", marketing);
    }

    Ok(())
}

fn test_forall_pattern(
    rules: &[rust_rule_engine::engine::rule::Rule],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Test 3: FORALL Pattern - All Orders Processed");
    println!("------------------------------------------------");

    let kb = KnowledgeBase::new("ForallTest");

    // Add the third rule (should be FORALL pattern)
    let forall_rule = &rules[2];
    println!(
        "Testing rule: {} (salience: {})",
        forall_rule.name, forall_rule.salience
    );
    kb.add_rule(forall_rule.clone())?;

    let mut engine = RustRuleEngine::with_config(
        kb,
        EngineConfig {
            debug_mode: true,
            max_cycles: 1,
            ..Default::default()
        },
    );

    let facts = Facts::new();

    // Add multiple orders - all processed
    for i in 1..=3 {
        let mut order = HashMap::new();
        order.insert("status".to_string(), Value::String("processed".to_string()));
        facts.add_value(&format!("Order{}", i), Value::Object(order))?;
    }

    // Add shipping state
    facts.set(
        "Shipping",
        Value::Object(HashMap::from([(
            "readyToShip".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial facts:");
    println!("  Order1.status = processed");
    println!("  Order2.status = processed");
    println!("  Order3.status = processed");
    println!("  Shipping.readyToShip = false");

    let result = engine.execute(&facts)?;
    println!("✅ FORALL test: {} rules fired", result.rules_fired);

    if let Some(shipping) = facts.get("Shipping") {
        println!("Final Shipping state: {:?}", shipping);
    }

    Ok(())
}

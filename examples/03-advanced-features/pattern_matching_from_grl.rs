use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Pattern Matching from GRL File Demo");
    println!("======================================");

    // Read GRL file
    let grl_content = fs::read_to_string("examples/rules/pattern_matching.grl")?;
    println!("📄 Reading rules from: examples/rules/pattern_matching.grl");

    // Parse rules from GRL
    let rules = GRLParser::parse_rules(&grl_content)?;
    println!("✅ Successfully parsed {} rules from GRL file", rules.len());

    // Create knowledge base and add parsed rules
    let kb = KnowledgeBase::new("PatternMatchingFromGRL");
    for rule in rules {
        println!(
            "  📋 Adding rule: {} (salience: {})",
            rule.name, rule.salience
        );
        kb.add_rule(rule)?;
    }

    // Configure engine
    let mut engine = RustRuleEngine::with_config(
        kb,
        EngineConfig {
            debug_mode: true,
            max_cycles: 1,
            ..Default::default()
        },
    );

    println!("\n🧪 Testing Scenario 1: VIP Customer Exists");
    println!("------------------------------------------");
    demo_vip_customer_scenario(&mut engine)?;

    println!("\n🧪 Testing Scenario 2: No Pending Orders");
    println!("----------------------------------------");
    demo_no_pending_orders_scenario(&mut engine)?;

    println!("\n🧪 Testing Scenario 3: All Orders Processed");
    println!("-------------------------------------------");
    demo_all_orders_processed_scenario(&mut engine)?;

    println!("\n🧪 Testing Scenario 4: Complex Business Rule");
    println!("--------------------------------------------");
    demo_complex_business_scenario(&mut engine)?;

    println!("\n🧪 Testing Scenario 5: Advanced Inventory Management");
    println!("---------------------------------------------------");
    demo_advanced_inventory_scenario(&mut engine)?;

    println!("\n🎉 GRL Pattern Matching Demo completed successfully!");
    Ok(())
}

fn demo_vip_customer_scenario(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    // Add customers with one VIP
    let mut customer1 = HashMap::new();
    customer1.insert("tier".to_string(), Value::String("Regular".to_string()));
    facts.add_value("Customer1", Value::Object(customer1))?;

    let mut customer2 = HashMap::new();
    customer2.insert("tier".to_string(), Value::String("VIP".to_string()));
    facts.add_value("Customer2", Value::Object(customer2))?;

    // Add system state
    facts.set(
        "System",
        Value::Object(HashMap::from([
            ("premiumServiceActive".to_string(), Value::Boolean(false)),
            ("vipModeEnabled".to_string(), Value::Boolean(false)),
        ])),
    );

    // Add marketing state to prevent errors
    facts.set(
        "Marketing",
        Value::Object(HashMap::from([(
            "emailSent".to_string(),
            Value::Boolean(false),
        )])),
    );

    // Add inventory state to prevent errors
    facts.set(
        "Inventory",
        Value::Object(HashMap::from([
            ("autoReplenishment".to_string(), Value::Boolean(false)),
            ("priority".to_string(), Value::String("normal".to_string())),
        ])),
    );

    // Add shipping state to prevent errors
    facts.set(
        "Shipping",
        Value::Object(HashMap::from([(
            "enabled".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial facts:");
    println!("  Customer1.tier = Regular");
    println!("  Customer2.tier = VIP");
    println!("  System.premiumServiceActive = false");

    let result = engine.execute(&facts)?;
    println!("Result: {} rules fired", result.rules_fired);

    if let Some(system) = facts.get("System") {
        println!("Final System state: {:?}", system);
    }

    Ok(())
}

fn demo_no_pending_orders_scenario(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
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
            "emailSent".to_string(),
            Value::Boolean(false),
        )])),
    );

    // Add other required states to prevent errors
    facts.set(
        "System",
        Value::Object(HashMap::from([
            ("premiumServiceActive".to_string(), Value::Boolean(false)),
            ("vipModeEnabled".to_string(), Value::Boolean(false)),
        ])),
    );

    facts.set(
        "Inventory",
        Value::Object(HashMap::from([
            ("autoReplenishment".to_string(), Value::Boolean(false)),
            ("priority".to_string(), Value::String("normal".to_string())),
        ])),
    );

    facts.set(
        "Shipping",
        Value::Object(HashMap::from([(
            "enabled".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial facts:");
    println!("  Order1.status = completed");
    println!("  Order2.status = shipped");
    println!("  Marketing.emailSent = false");

    let result = engine.execute(&facts)?;
    println!("Result: {} rules fired", result.rules_fired);

    if let Some(marketing) = facts.get("Marketing") {
        println!("Final Marketing state: {:?}", marketing);
    }

    Ok(())
}

fn demo_all_orders_processed_scenario(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    // Add all processed orders
    for i in 1..=3 {
        let mut order = HashMap::new();
        order.insert("status".to_string(), Value::String("processed".to_string()));
        facts.add_value(&format!("Order{}", i), Value::Object(order))?;
    }

    // Add shipping state
    facts.set(
        "Shipping",
        Value::Object(HashMap::from([(
            "enabled".to_string(),
            Value::Boolean(false),
        )])),
    );

    // Add other required states to prevent errors
    facts.set(
        "System",
        Value::Object(HashMap::from([
            ("premiumServiceActive".to_string(), Value::Boolean(false)),
            ("vipModeEnabled".to_string(), Value::Boolean(false)),
        ])),
    );

    facts.set(
        "Marketing",
        Value::Object(HashMap::from([(
            "emailSent".to_string(),
            Value::Boolean(false),
        )])),
    );

    facts.set(
        "Inventory",
        Value::Object(HashMap::from([
            ("autoReplenishment".to_string(), Value::Boolean(false)),
            ("priority".to_string(), Value::String("normal".to_string())),
        ])),
    );

    println!("Initial facts:");
    println!("  Order1.status = processed");
    println!("  Order2.status = processed");
    println!("  Order3.status = processed");
    println!("  Shipping.enabled = false");

    let result = engine.execute(&facts)?;
    println!("Result: {} rules fired", result.rules_fired);

    if let Some(shipping) = facts.get("Shipping") {
        println!("Final Shipping state: {:?}", shipping);
    }

    Ok(())
}

fn demo_complex_business_scenario(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    // Add VIP customer
    let mut customer1 = HashMap::new();
    customer1.insert("tier".to_string(), Value::String("VIP".to_string()));
    facts.add_value("Customer1", Value::Object(customer1))?;

    // Add only low priority alerts (no high priority)
    let mut alert1 = HashMap::new();
    alert1.insert("priority".to_string(), Value::String("low".to_string()));
    facts.add_value("Alert1", Value::Object(alert1))?;

    // Add system state
    facts.set(
        "System",
        Value::Object(HashMap::from([(
            "vipModeEnabled".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial facts:");
    println!("  Customer1.tier = VIP");
    println!("  Alert1.priority = low (no high priority alerts)");
    println!("  System.vipModeEnabled = false");

    let result = engine.execute(&facts)?;
    println!("Result: {} rules fired", result.rules_fired);

    if let Some(system) = facts.get("System") {
        println!("Final System state: {:?}", system);
    }

    Ok(())
}

fn demo_advanced_inventory_scenario(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    // Add electronics products
    let mut product1 = HashMap::new();
    product1.insert(
        "category".to_string(),
        Value::String("electronics".to_string()),
    );
    facts.add_value("Product1", Value::Object(product1))?;

    // Add active suppliers
    for i in 1..=2 {
        let mut supplier = HashMap::new();
        supplier.insert("status".to_string(), Value::String("active".to_string()));
        facts.add_value(&format!("Supplier{}", i), Value::Object(supplier))?;
    }

    // Add only non-critical alerts
    let mut alert1 = HashMap::new();
    alert1.insert("type".to_string(), Value::String("warning".to_string()));
    facts.add_value("Alert1", Value::Object(alert1))?;

    // Add inventory state
    facts.set(
        "Inventory",
        Value::Object(HashMap::from([
            ("autoReplenishment".to_string(), Value::Boolean(false)),
            ("priority".to_string(), Value::String("normal".to_string())),
        ])),
    );

    println!("Initial facts:");
    println!("  Product1.category = electronics");
    println!("  Supplier1.status = active");
    println!("  Supplier2.status = active");
    println!("  Alert1.type = warning (no critical alerts)");
    println!("  Inventory.autoReplenishment = false");

    let result = engine.execute(&facts)?;
    println!("Result: {} rules fired", result.rules_fired);

    if let Some(inventory) = facts.get("Inventory") {
        println!("Final Inventory state: {:?}", inventory);
    }

    Ok(())
}

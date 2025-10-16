use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Pattern Matching Demo - EXISTS, NOT, FORALL");
    println!("================================================");

    // Demo 1: EXISTS pattern matching
    demo_exists_pattern()?;

    // Demo 2: NOT pattern matching
    demo_not_pattern()?;

    // Demo 3: FORALL pattern matching
    demo_forall_pattern()?;

    // Demo 4: Combined pattern matching
    demo_combined_patterns()?;

    println!("\n🎉 Pattern Matching Demo completed successfully!");
    Ok(())
}

fn demo_exists_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 1: EXISTS Pattern Matching");
    println!("-----------------------------------");

    let kb = KnowledgeBase::new("ExistsDemo");

    // Rule: If ANY customer is VIP, activate premium service
    let exists_rule = Rule::new(
        "ActivatePremiumService".to_string(),
        ConditionGroup::exists(ConditionGroup::Single(Condition::new(
            "Customer.tier".to_string(),
            Operator::Equal,
            Value::String("VIP".to_string()),
        ))),
        vec![
            ActionType::Set {
                field: "System.premiumServiceActive".to_string(),
                value: Value::Boolean(true),
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String(
                    "Premium service activated - VIP customer detected".to_string(),
                )],
            },
        ],
    )
    .with_salience(20)
    .with_no_loop(true);

    kb.add_rule(exists_rule)?;

    let mut engine = RustRuleEngine::with_config(
        kb,
        EngineConfig {
            debug_mode: true,
            max_cycles: 1, // Prevent infinite loops in demo
            ..Default::default()
        },
    );

    let facts = Facts::new();

    // Add some customers - one is VIP
    let mut customer1 = HashMap::new();
    customer1.insert("tier".to_string(), Value::String("Regular".to_string()));
    facts.add_value("Customer1", Value::Object(customer1))?;

    let mut customer2 = HashMap::new();
    customer2.insert("tier".to_string(), Value::String("VIP".to_string()));
    facts.add_value("Customer2", Value::Object(customer2))?;

    let mut customer3 = HashMap::new();
    customer3.insert("tier".to_string(), Value::String("Bronze".to_string()));
    facts.add_value("Customer3", Value::Object(customer3))?;

    // Add system state
    facts.set(
        "System",
        Value::Object(HashMap::from([(
            "premiumServiceActive".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial state:");
    println!("  Customer1.tier = Regular");
    println!("  Customer2.tier = VIP");
    println!("  Customer3.tier = Bronze");
    println!("  System.premiumServiceActive = false");

    let result = engine.execute(&facts)?;

    println!("\nResult: {} rules fired", result.rules_fired);
    if let Some(system) = facts.get("System") {
        println!("Final System state: {:?}", system);
    }

    Ok(())
}

fn demo_not_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 2: NOT Pattern Matching");
    println!("--------------------------------");

    let kb = KnowledgeBase::new("NotDemo");

    // Rule: If NO orders are pending, send marketing email
    let not_rule = Rule::new(
        "SendMarketingEmail".to_string(),
        ConditionGroup::not(ConditionGroup::exists(ConditionGroup::Single(
            Condition::new(
                "Order.status".to_string(),
                Operator::Equal,
                Value::String("pending".to_string()),
            ),
        ))),
        vec![
            ActionType::Set {
                field: "Marketing.emailSent".to_string(),
                value: Value::Boolean(true),
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String(
                    "Marketing email sent - no pending orders".to_string(),
                )],
            },
        ],
    )
    .with_salience(15)
    .with_no_loop(true);

    kb.add_rule(not_rule)?;

    let mut engine = RustRuleEngine::with_config(
        kb,
        EngineConfig {
            debug_mode: true,
            max_cycles: 1, // Prevent infinite loops in demo
            ..Default::default()
        },
    );

    let facts = Facts::new();

    // Add completed orders only (no pending ones)
    let mut order1 = HashMap::new();
    order1.insert("status".to_string(), Value::String("completed".to_string()));
    facts.add_value("Order1", Value::Object(order1))?;

    let mut order2 = HashMap::new();
    order2.insert("status".to_string(), Value::String("shipped".to_string()));
    facts.add_value("Order2", Value::Object(order2))?;

    facts.set(
        "Marketing",
        Value::Object(HashMap::from([(
            "emailSent".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial state:");
    println!("  Order1.status = completed");
    println!("  Order2.status = shipped");
    println!("  No pending orders");
    println!("  Marketing.emailSent = false");

    let result = engine.execute(&facts)?;

    println!("\nResult: {} rules fired", result.rules_fired);
    if let Some(marketing) = facts.get("Marketing") {
        println!("Final Marketing state: {:?}", marketing);
    }

    Ok(())
}

fn demo_forall_pattern() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 3: FORALL Pattern Matching");
    println!("-----------------------------------");

    let kb = KnowledgeBase::new("ForallDemo");

    // Rule: If ALL orders are processed, enable shipping
    let forall_rule = Rule::new(
        "EnableShipping".to_string(),
        ConditionGroup::forall(ConditionGroup::Single(Condition::new(
            "Order.status".to_string(),
            Operator::Equal,
            Value::String("processed".to_string()),
        ))),
        vec![
            ActionType::Set {
                field: "Shipping.enabled".to_string(),
                value: Value::Boolean(true),
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String(
                    "Shipping enabled - all orders processed".to_string(),
                )],
            },
        ],
    )
    .with_salience(10)
    .with_no_loop(true);

    kb.add_rule(forall_rule)?;

    let mut engine = RustRuleEngine::with_config(
        kb,
        EngineConfig {
            debug_mode: true,
            max_cycles: 1, // Prevent infinite loops in demo
            ..Default::default()
        },
    );

    let facts = Facts::new();

    // Add multiple orders - all processed
    let mut order1 = HashMap::new();
    order1.insert("status".to_string(), Value::String("processed".to_string()));
    facts.add_value("Order1", Value::Object(order1))?;

    let mut order2 = HashMap::new();
    order2.insert("status".to_string(), Value::String("processed".to_string()));
    facts.add_value("Order2", Value::Object(order2))?;

    let mut order3 = HashMap::new();
    order3.insert("status".to_string(), Value::String("processed".to_string()));
    facts.add_value("Order3", Value::Object(order3))?;

    facts.set(
        "Shipping",
        Value::Object(HashMap::from([(
            "enabled".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial state:");
    println!("  Order1.status = processed");
    println!("  Order2.status = processed");
    println!("  Order3.status = processed");
    println!("  Shipping.enabled = false");

    let result = engine.execute(&facts)?;

    println!("\nResult: {} rules fired", result.rules_fired);
    if let Some(shipping) = facts.get("Shipping") {
        println!("Final Shipping state: {:?}", shipping);
    }

    // Now add a pending order and test again
    println!("\n--- Adding pending order ---");
    let mut order4 = HashMap::new();
    order4.insert("status".to_string(), Value::String("pending".to_string()));
    facts.add_value("Order4", Value::Object(order4))?;

    // Reset shipping state
    facts.set(
        "Shipping",
        Value::Object(HashMap::from([(
            "enabled".to_string(),
            Value::Boolean(false),
        )])),
    );

    let result2 = engine.execute(&facts)?;
    println!(
        "Result after adding pending order: {} rules fired",
        result2.rules_fired
    );
    if let Some(shipping) = facts.get("Shipping") {
        println!("Final Shipping state: {:?}", shipping);
    }

    Ok(())
}

fn demo_combined_patterns() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 4: Combined Pattern Matching");
    println!("-------------------------------------");

    let kb = KnowledgeBase::new("CombinedDemo");

    // Complex rule combining multiple patterns
    let complex_rule = Rule::new(
        "ComplexBusinessRule".to_string(),
        ConditionGroup::and(
            // At least one VIP customer exists
            ConditionGroup::exists(ConditionGroup::Single(Condition::new(
                "Customer.tier".to_string(),
                Operator::Equal,
                Value::String("VIP".to_string()),
            ))),
            // No high-priority alerts exist
            ConditionGroup::not(ConditionGroup::exists(ConditionGroup::Single(
                Condition::new(
                    "Alert.priority".to_string(),
                    Operator::Equal,
                    Value::String("high".to_string()),
                ),
            ))),
        ),
        vec![
            ActionType::Set {
                field: "System.vipModeEnabled".to_string(),
                value: Value::Boolean(true),
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String(
                    "VIP mode enabled - VIP customer present and no high alerts".to_string(),
                )],
            },
        ],
    )
    .with_salience(25)
    .with_no_loop(true);

    kb.add_rule(complex_rule)?;

    let mut engine = RustRuleEngine::with_config(
        kb,
        EngineConfig {
            debug_mode: true,
            max_cycles: 1, // Prevent infinite loops in demo
            ..Default::default()
        },
    );

    let facts = Facts::new();

    // Add mixed customers
    let mut customer1 = HashMap::new();
    customer1.insert("tier".to_string(), Value::String("VIP".to_string()));
    facts.add_value("Customer1", Value::Object(customer1))?;

    let mut customer2 = HashMap::new();
    customer2.insert("tier".to_string(), Value::String("Regular".to_string()));
    facts.add_value("Customer2", Value::Object(customer2))?;

    // Add low priority alerts only
    let mut alert1 = HashMap::new();
    alert1.insert("priority".to_string(), Value::String("low".to_string()));
    facts.add_value("Alert1", Value::Object(alert1))?;

    facts.set(
        "System",
        Value::Object(HashMap::from([(
            "vipModeEnabled".to_string(),
            Value::Boolean(false),
        )])),
    );

    println!("Initial state:");
    println!("  Customer1.tier = VIP");
    println!("  Customer2.tier = Regular");
    println!("  Alert1.priority = low");
    println!("  System.vipModeEnabled = false");

    let result = engine.execute(&facts)?;

    println!("\nResult: {} rules fired", result.rules_fired);
    if let Some(system) = facts.get("System") {
        println!("Final System state: {:?}", system);
    }

    Ok(())
}

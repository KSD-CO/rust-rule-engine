//! Multi-field Variables from GRL File Demo
//!
//! This example demonstrates loading and executing multi-field patterns from
//! a GRL rule file. It showcases the parser's ability to recognize CLIPS-style
//! multi-field syntax.
//!
//! Run with: `cargo run --example multifield_grl_demo`

use rust_rule_engine::{Facts, KnowledgeBase, RustRuleEngine, Value};
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Multi-field Variables from GRL File Demo ===\n");

    // Create knowledge base
    let kb = KnowledgeBase::new("multifield_demo");

    // Load rules from GRL file
    let grl_file = "examples/rules/multifield_patterns.grl";
    println!("📂 Loading rules from: {}\n", grl_file);

    match fs::read_to_string(grl_file) {
        Ok(grl_content) => {
            match kb.add_rules_from_grl(&grl_content) {
                Ok(count) => {
                    println!("✅ Loaded {} rules successfully\n", count);
                }
                Err(e) => {
                    println!("❌ Error parsing rules: {}\n", e);
                    println!("Note: Multi-field GRL parsing is partially implemented.");
                    println!("Some patterns may not parse correctly yet.\n");
                    println!("Demonstrating with hardcoded examples instead...\n");
                    return demo_with_code();
                }
            }
        }
        Err(e) => {
            println!("❌ Error reading file: {}\n", e);
            println!("Demonstrating with hardcoded examples instead...\n");
            return demo_with_code();
        }
    }

    // Create engine
    let mut engine = RustRuleEngine::new(kb);

    // Demo 1: Array count operation
    demo_array_count(&mut engine)?;

    // Demo 2: Array empty/not_empty
    demo_array_empty(&mut engine)?;

    // Demo 3: Array contains
    demo_array_contains(&mut engine)?;

    Ok(())
}

/// Demo array count operation
fn demo_array_count(engine: &mut RustRuleEngine) -> Result<(), Box<dyn Error>> {
    println!("--- Demo 1: Array Count Operation ---");

    let mut facts = Facts::new();
    facts.set("Order.status", Value::String("pending".to_string()));
    facts.set(
        "Order.items",
        Value::Array(vec![
            Value::String("ITEM-1".to_string()),
            Value::String("ITEM-2".to_string()),
            Value::String("ITEM-3".to_string()),
        ]),
    );

    println!("Facts:");
    println!("  Order.status = pending");
    println!("  Order.items = [ITEM-1, ITEM-2, ITEM-3] (count: 3)");
    println!();

    // Execute rules
    let result = engine.execute(&mut facts)?;
    println!("Rules fired: {}\n", result.rules_fired);

    Ok(())
}

/// Demo array empty/not_empty
fn demo_array_empty(engine: &mut RustRuleEngine) -> Result<(), Box<dyn Error>> {
    println!("--- Demo 2: Array Empty Check ---");

    let mut facts = Facts::new();
    facts.set(
        "ShoppingCart.items",
        Value::Array(Vec::new()), // Empty array
    );

    println!("Facts:");
    println!("  ShoppingCart.items = [] (empty)");
    println!();

    let result = engine.execute(&mut facts)?;
    println!("Rules fired: {}", result.rules_fired);
    println!("Result: ShoppingCart.status = {:?}\n", facts.get("ShoppingCart.status"));

    Ok(())
}

/// Demo array contains
fn demo_array_contains(engine: &mut RustRuleEngine) -> Result<(), Box<dyn Error>> {
    println!("--- Demo 3: Array Contains Operation ---");

    let mut facts = Facts::new();
    facts.set(
        "Product.tags",
        Value::Array(vec![
            Value::String("electronics".to_string()),
            Value::String("gadgets".to_string()),
            Value::String("sale".to_string(),),
        ]),
    );

    println!("Facts:");
    println!("  Product.tags = [electronics, gadgets, sale]");
    println!();

    let result = engine.execute(&mut facts)?;
    println!("Rules fired: {}", result.rules_fired);
    println!("Product.category = {:?}\n", facts.get("Product.category"));

    Ok(())
}

/// Fallback demo using code instead of GRL file
fn demo_with_code() -> Result<(), Box<dyn Error>> {
    use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
    use rust_rule_engine::types::{ActionType, Operator};

    // Create knowledge base
    let kb = KnowledgeBase::new("multifield_demo");

    // Rule 1: Check array count
    let rule1 = Rule::new(
        "CheckItemCount".to_string(),
        ConditionGroup::single(Condition::with_multifield_count(
            "Order.items".to_string(),
            Operator::GreaterThan,
            Value::Integer(0),
        )),
        vec![ActionType::Set {
            field: "Order.valid".to_string(),
            value: Value::Boolean(true),
        }],
    )
    .with_priority(100);

    kb.add_rule(rule1)?;

    // Rule 2: Check array empty
    let rule2 = Rule::new(
        "CheckCartEmpty".to_string(),
        ConditionGroup::single(Condition::with_multifield_empty("ShoppingCart.items".to_string())),
        vec![ActionType::Set {
            field: "ShoppingCart.status".to_string(),
            value: Value::String("empty".to_string()),
        }],
    )
    .with_priority(90);

    kb.add_rule(rule2)?;

    println!("✅ Created rules programmatically\n");

    // Create engine
    let mut engine = RustRuleEngine::new(kb);

    // Test with facts
    let mut facts = Facts::new();
    facts.set(
        "Order.items",
        Value::Array(vec![
            Value::String("ITEM-1".to_string()),
            Value::String("ITEM-2".to_string()),
        ]),
    );

    println!("Testing with Order.items = [ITEM-1, ITEM-2]");
    let result = engine.execute(&mut facts)?;
    println!("Rules fired: {}", result.rules_fired);
    println!("Order.valid = {:?}\n", facts.get("Order.valid"));

    // Test empty cart
    let mut facts2 = Facts::new();
    facts2.set("ShoppingCart.items", Value::Array(Vec::new()));

    println!("Testing with ShoppingCart.items = []");
    let result2 = engine.execute(&mut facts2)?;
    println!("Rules fired: {}", result2.rules_fired);
    println!("ShoppingCart.status = {:?}\n", facts2.get("ShoppingCart.status"));

    Ok(())
}

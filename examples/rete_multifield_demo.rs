//! RETE Incremental Engine Multi-field Demo
//!
//! This example demonstrates multi-field (multislot) variable support
//! in the RETE incremental engine, loading rules from GRL files.
//!
//! Run with: `cargo run --example rete_multifield_demo`

use rust_rule_engine::rete::{
    propagation::IncrementalEngine,
    grl_loader::GrlReteLoader,
    facts::{TypedFacts, FactValue},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== RETE Incremental Engine Multi-field Demo ===\n");

    // Create RETE engine
    let mut engine = IncrementalEngine::new();

    // Load multifield rules from GRL file
    println!("📂 Loading multifield rules from GRL file...");
    let rules_path = "examples/rules/multifield_patterns.grl";
    let count = GrlReteLoader::load_from_file(rules_path, &mut engine)?;
    println!("✅ Loaded {} rules into RETE engine\n", count);

    // Test 1: Empty cart
    println!("--- Test 1: Empty Shopping Cart ---");
    let mut cart_facts = TypedFacts::new();
    cart_facts.set("items", FactValue::Array(Vec::new()));
    cart_facts.set("status", FactValue::String("empty".to_string()));

    engine.insert("ShoppingCart".to_string(), cart_facts);
    let fired = engine.fire_all();
    println!("Fired rules: {:?}\n", fired);

    // Test 2: Order with items (count)
    println!("--- Test 2: Order with Multiple Items ---");
    let mut order_facts = TypedFacts::new();
    order_facts.set("status", FactValue::String("pending".to_string()));
    order_facts.set("items", FactValue::Array(vec![
        FactValue::String("ITEM-1".to_string()),
        FactValue::String("ITEM-2".to_string()),
        FactValue::String("ITEM-3".to_string()),
        FactValue::String("ITEM-4".to_string()),
    ]));

    engine.insert("Order".to_string(), order_facts);
    let fired = engine.fire_all();
    println!("Fired rules: {:?}\n", fired);

    // Test 3: Product with electronics tag
    println!("--- Test 3: Product with Electronics Tag ---");
    let mut product_facts = TypedFacts::new();
    product_facts.set("tags", FactValue::Array(vec![
        FactValue::String("electronics".to_string()),
        FactValue::String("gadgets".to_string()),
    ]));

    engine.insert("Product".to_string(), product_facts);
    let fired = engine.fire_all();
    println!("Fired rules: {:?}\n", fired);

    // Test 4: Bulk order (count >= 5)
    println!("--- Test 4: Bulk Order (5+ items) ---");
    let mut bulk_order = TypedFacts::new();
    bulk_order.set("items", FactValue::Array(vec![
        FactValue::String("ITEM-1".to_string()),
        FactValue::String("ITEM-2".to_string()),
        FactValue::String("ITEM-3".to_string()),
        FactValue::String("ITEM-4".to_string()),
        FactValue::String("ITEM-5".to_string()),
    ]));

    engine.insert("Order".to_string(), bulk_order);
    let fired = engine.fire_all();
    println!("Fired rules: {:?}\n", fired);

    // Test 5: Product with sale tag
    println!("--- Test 5: Product with Sale Tag ---");
    let mut sale_product = TypedFacts::new();
    sale_product.set("tags", FactValue::Array(vec![
        FactValue::String("sale".to_string()),
        FactValue::String("clearance".to_string()),
    ]));

    engine.insert("Product".to_string(), sale_product);
    let fired = engine.fire_all();
    println!("Fired rules: {:?}\n", fired);

    // Test 6: Premium electronics with multiple items
    println!("--- Test 6: Premium Electronics Bulk Order ---");
    let mut premium_order = TypedFacts::new();
    premium_order.set("items", FactValue::Array(vec![
        FactValue::String("ITEM-1".to_string()),
        FactValue::String("ITEM-2".to_string()),
        FactValue::String("ITEM-3".to_string()),
        FactValue::String("ITEM-4".to_string()),
    ]));
    engine.insert("Order".to_string(), premium_order);

    let mut premium_product = TypedFacts::new();
    premium_product.set("tags", FactValue::Array(vec![
        FactValue::String("electronics".to_string()),
        FactValue::String("premium".to_string()),
    ]));
    engine.insert("Product".to_string(), premium_product);

    let fired = engine.fire_all();
    println!("Fired rules: {:?}\n", fired);

    // Test 7: Queue with tasks (first/last)
    println!("--- Test 7: Queue with Tasks ---");
    let mut queue_facts = TypedFacts::new();
    queue_facts.set("tasks", FactValue::Array(vec![
        FactValue::String("Task A".to_string()),
        FactValue::String("Task B".to_string()),
        FactValue::String("Task C".to_string()),
    ]));

    engine.insert("Queue".to_string(), queue_facts);
    let fired = engine.fire_all();
    println!("Fired rules: {:?}\n", fired);

    // Test 8: Low inventory
    println!("--- Test 8: Low Inventory Alert ---");
    let mut inventory_facts = TypedFacts::new();
    inventory_facts.set("items", FactValue::Array(vec![
        FactValue::String("Item-1".to_string()),
        FactValue::String("Item-2".to_string()),
        FactValue::String("Item-3".to_string()),
    ]));

    engine.insert("Inventory".to_string(), inventory_facts);
    let fired = engine.fire_all();
    println!("Fired rules: {:?}\n", fired);

    println!("\n🎉 RETE Multi-field Demo Completed!");

    Ok(())
}

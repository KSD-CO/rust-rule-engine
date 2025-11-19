//! Multi-field Variables Demo - CLIPS-inspired Feature
//!
//! This example demonstrates the new multi-field (multislot) variable support
//! that enables pattern matching on arrays and collections.
//!
//! ## Features Demonstrated
//!
//! 1. **Collect Operation** - Bind all array values to a variable
//! 2. **Contains Operation** - Check if array contains a specific value
//! 3. **Count Operation** - Get array length
//! 4. **First/Last Operations** - Access first/last elements
//! 5. **Template Multislot Fields** - CLIPS-style multislot definitions
//!
//! ## CLIPS Comparison
//!
//! ```clips
//! (deftemplate order
//!   (slot order-id)
//!   (multislot items))
//!
//! (defrule process-order
//!   (order (order-id ?id) (items $?all-items))
//!   =>
//!   (foreach ?item $?all-items
//!     (printout t "Processing " ?item crlf)))
//! ```
//!
//! Run with: `cargo run --example multifield_demo`

use rust_rule_engine::rete::{
    TemplateBuilder, FieldType, FactValue, TypedFacts,
    PatternConstraint, Pattern, MultifieldOp,
};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-field Variables Demo ===\n");

    // Demo 1: Template with Multislot Fields
    demo_multislot_template();

    // Demo 2: Collect Operation
    demo_collect_operation();

    // Demo 3: Contains Operation
    demo_contains_operation();

    // Demo 4: Count Operation
    demo_count_operation();

    // Demo 5: First/Last Operations
    demo_first_last_operations();

    // Demo 6: Pattern Matching with Multi-field
    demo_pattern_matching();

    Ok(())
}

/// Demo 1: Define templates with multislot fields
fn demo_multislot_template() {
    println!("--- Demo 1: Template with Multislot Fields ---");

    let order_template = TemplateBuilder::new("Order")
        .required_string("order_id")
        .multislot_field("items", FieldType::String)  // CLIPS-style
        .float_field("total")
        .build();

    println!("✅ Created Order template with multislot 'items' field");

    // Create an instance
    let mut order = TypedFacts::new();
    order.set("order_id", FactValue::String("ORD-001".to_string()));
    order.set("items", FactValue::Array(vec![
        FactValue::String("ITEM-1".to_string()),
        FactValue::String("ITEM-2".to_string()),
        FactValue::String("ITEM-3".to_string()),
    ]));
    order.set("total", FactValue::Float(150.0));

    // Validate against template
    match order_template.validate(&order) {
        Ok(_) => println!("✅ Order validated successfully"),
        Err(e) => println!("❌ Validation error: {}", e),
    }

    println!();
}

/// Demo 2: Collect all values into a variable
fn demo_collect_operation() {
    println!("--- Demo 2: Collect Operation ($?var) ---");

    let mut facts = TypedFacts::new();
    facts.set("items", FactValue::Array(vec![
        FactValue::String("apple".to_string()),
        FactValue::String("banana".to_string()),
        FactValue::String("cherry".to_string()),
    ]));

    let op = MultifieldOp::Collect;
    let result = op.evaluate(&facts, "items", None);

    println!("Pattern: items $?all_items");
    if let Some(values) = result {
        println!("✅ Collected {} items:", values.len());
        for (i, item) in values.iter().enumerate() {
            println!("   [{}] {}", i, item);
        }
    }

    println!();
}

/// Demo 3: Check if array contains a value
fn demo_contains_operation() {
    println!("--- Demo 3: Contains Operation ---");

    let mut product = TypedFacts::new();
    product.set("tags", FactValue::Array(vec![
        FactValue::String("electronics".to_string()),
        FactValue::String("gadgets".to_string()),
        FactValue::String("sale".to_string()),
    ]));

    let op = MultifieldOp::Contains;
    let search = FactValue::String("electronics".to_string());

    println!("Pattern: Product.tags contains \"electronics\"");
    if let Some(result) = op.evaluate(&product, "tags", Some(&search)) {
        if let FactValue::Boolean(contains) = result[0] {
            println!("✅ Result: {}", contains);
        }
    }

    // Test with non-existent value
    let search2 = FactValue::String("furniture".to_string());
    println!("\nPattern: Product.tags contains \"furniture\"");
    if let Some(result) = op.evaluate(&product, "tags", Some(&search2)) {
        if let FactValue::Boolean(contains) = result[0] {
            println!("✅ Result: {}", contains);
        }
    }

    println!();
}

/// Demo 4: Get array length
fn demo_count_operation() {
    println!("--- Demo 4: Count Operation ---");

    let mut order = TypedFacts::new();
    order.set("items", FactValue::Array(vec![
        FactValue::String("ITEM-1".to_string()),
        FactValue::String("ITEM-2".to_string()),
        FactValue::String("ITEM-3".to_string()),
        FactValue::String("ITEM-4".to_string()),
        FactValue::String("ITEM-5".to_string()),
    ]));

    let op = MultifieldOp::Count;

    println!("Pattern: Order.items count");
    if let Some(result) = op.evaluate(&order, "items", None) {
        if let FactValue::Integer(count) = result[0] {
            println!("✅ Item count: {}", count);
        }
    }

    println!();
}

/// Demo 5: Access first and last elements
fn demo_first_last_operations() {
    println!("--- Demo 5: First/Last Operations ---");

    let mut queue = TypedFacts::new();
    queue.set("tasks", FactValue::Array(vec![
        FactValue::String("Task A".to_string()),
        FactValue::String("Task B".to_string()),
        FactValue::String("Task C".to_string()),
    ]));

    println!("Pattern: Queue.tasks first");
    if let Some(result) = MultifieldOp::First.evaluate(&queue, "tasks", None) {
        println!("✅ First task: {}", result[0]);
    }

    println!("\nPattern: Queue.tasks last");
    if let Some(result) = MultifieldOp::Last.evaluate(&queue, "tasks", None) {
        println!("✅ Last task: {}", result[0]);
    }

    println!();
}

/// Demo 6: Pattern matching with multi-field constraints
fn demo_pattern_matching() {
    println!("--- Demo 6: Pattern Matching with Multi-field ---");

    let mut order = TypedFacts::new();
    order.set("order_id", FactValue::String("ORD-001".to_string()));
    order.set("items", FactValue::Array(vec![
        FactValue::String("ITEM-1".to_string()),
        FactValue::String("ITEM-2".to_string()),
    ]));

    // Create a pattern with multi-field constraint
    let pattern = Pattern::new("Order".to_string())
        .with_constraint(PatternConstraint::Simple {
            field: "order_id".to_string(),
            operator: "==".to_string(),
            value: FactValue::String("ORD-001".to_string()),
        })
        .with_constraint(PatternConstraint::MultiField {
            field: "items".to_string(),
            variable: Some("$?all_items".to_string()),
            operator: MultifieldOp::Collect,
            value: None,
        });

    println!("Pattern:");
    println!("  Order.order_id == \"ORD-001\"");
    println!("  Order.items $?all_items");

    let bindings = HashMap::new();
    if let Some(new_bindings) = pattern.matches(&order, &bindings) {
        println!("\n✅ Pattern matched!");
        println!("Variable bindings:");
        for (var, value) in new_bindings {
            println!("  {} = {:?}", var, value);
        }
    } else {
        println!("❌ Pattern did not match");
    }

    println!();
}

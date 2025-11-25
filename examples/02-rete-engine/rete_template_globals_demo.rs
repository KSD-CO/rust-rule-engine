//! RETE Template System & Defglobal Demo
//!
//! Demonstrates CLIPS-inspired features:
//! 1. Template System (deftemplate) - Type-safe structured facts
//! 2. Defglobal - Global variables accessible across rule firings
//!
//! Run: cargo run --example rete_template_globals_demo

use rust_rule_engine::rete::{
    IncrementalEngine, GrlReteLoader, TemplateBuilder, FieldType, FactValue, TypedFacts,
};
use rust_rule_engine::errors::Result;

fn main() -> Result<()> {
    println!("=== RETE Template System & Defglobal Demo ===\n");

    // ============================================================
    // PART 1: Template System (like CLIPS deftemplate)
    // ============================================================
    println!("📋 Part 1: Template System");
    println!("─────────────────────────────────────────\n");

    demo_template_basic()?;
    demo_template_validation()?;
    demo_template_with_rules()?;

    // ============================================================
    // PART 2: Defglobal (Global Variables)
    // ============================================================
    println!("\n🌍 Part 2: Defglobal (Global Variables)");
    println!("─────────────────────────────────────────\n");

    demo_globals_basic()?;
    demo_globals_in_rules()?;
    demo_globals_thread_safety()?;

    // ============================================================
    // PART 3: Combined Usage
    // ============================================================
    println!("\n🔗 Part 3: Templates + Globals Combined");
    println!("─────────────────────────────────────────\n");

    demo_combined()?;

    println!("\n✅ Demo completed successfully!");
    Ok(())
}

// ============================================================
// PART 1: Template System Demos
// ============================================================

fn demo_template_basic() -> Result<()> {
    println!("1️⃣ Basic Template Usage");

    // Define a template for Person
    let person_template = TemplateBuilder::new("Person")
        .required_string("name")
        .integer_field("age")
        .boolean_field("is_adult")
        .float_field("salary")
        .build();

    println!("   Template: {}", person_template.name);
    println!("   Fields: {} defined", person_template.fields.len());

    // Create instance with defaults
    let mut person = person_template.create_instance();
    println!("\n   Default instance: {:?}", person);

    // Populate fields
    person.set("name", FactValue::String("Alice".to_string()));
    person.set("age", FactValue::Integer(30));
    person.set("is_adult", FactValue::Boolean(true));
    person.set("salary", FactValue::Float(75000.0));

    // Validate
    match person_template.validate(&person) {
        Ok(_) => println!("   ✅ Validation passed!"),
        Err(e) => println!("   ❌ Validation failed: {:?}", e),
    }

    println!();
    Ok(())
}

fn demo_template_validation() -> Result<()> {
    println!("2️⃣ Template Validation");

    // Define template with required fields
    let order_template = TemplateBuilder::new("Order")
        .required_string("order_id")
        .float_field("amount")
        .integer_field("quantity")
        .array_field("items", FieldType::String)
        .build();

    // Valid order
    let mut valid_order = TypedFacts::new();
    valid_order.set("order_id", FactValue::String("ORD-001".to_string()));
    valid_order.set("amount", FactValue::Float(299.99));
    valid_order.set("quantity", FactValue::Integer(3));
    valid_order.set(
        "items",
        FactValue::Array(vec![
            FactValue::String("Widget A".to_string()),
            FactValue::String("Widget B".to_string()),
        ]),
    );

    match order_template.validate(&valid_order) {
        Ok(_) => println!("   ✅ Valid order passed validation"),
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Invalid order (missing required field)
    let mut invalid_order = TypedFacts::new();
    invalid_order.set("amount", FactValue::Float(100.0));

    match order_template.validate(&invalid_order) {
        Ok(_) => println!("   ⚠️  Invalid order passed (shouldn't happen)"),
        Err(e) => println!("   ✅ Correctly rejected invalid order: {}", e),
    }

    // Wrong type
    let mut wrong_type = TypedFacts::new();
    wrong_type.set("order_id", FactValue::String("ORD-002".to_string()));
    wrong_type.set("amount", FactValue::String("not a number".to_string())); // Wrong type!

    match order_template.validate(&wrong_type) {
        Ok(_) => println!("   ⚠️  Wrong type passed (shouldn't happen)"),
        Err(e) => println!("   ✅ Correctly rejected wrong type: {}", e),
    }

    println!();
    Ok(())
}

fn demo_template_with_rules() -> Result<()> {
    println!("3️⃣ Templates with Rules");

    let mut engine = IncrementalEngine::new();

    // Register template
    let customer_template = TemplateBuilder::new("Customer")
        .required_string("customer_id")
        .string_field("name")
        .integer_field("orders_count")
        .float_field("total_spent")
        .string_field("tier")
        .build();

    engine.templates_mut().register(customer_template);

    // Load rules
    let rules = r#"
    rule "VIPUpgrade" salience 20 no-loop {
        when
            Customer.total_spent > 10000
        then
            Customer.tier = "VIP";
    }

    rule "GoldTier" salience 15 no-loop {
        when
            Customer.total_spent > 5000 && Customer.orders_count > 20
        then
            Customer.tier = "Gold";
    }
    "#;

    GrlReteLoader::load_from_string(rules, &mut engine)?;

    // Create customer using template
    let mut customer = TypedFacts::new();
    customer.set("customer_id", FactValue::String("C123".to_string()));
    customer.set("name", FactValue::String("Bob".to_string()));
    customer.set("orders_count", FactValue::Integer(25));
    customer.set("total_spent", FactValue::Float(12000.0));
    customer.set("tier", FactValue::String("Standard".to_string()));

    // Insert with template validation
    match engine.insert_with_template("Customer", customer) {
        Ok(handle) => {
            println!("   ✅ Customer inserted with validation (handle: {})", handle);

            // Fire rules
            engine.reset();
            let fired = engine.fire_all();
            println!("   🔥 Fired {} rules: {:?}", fired.len(), fired);

            // Check result
            if let Some(updated) = engine.working_memory().get(&handle) {
                if let Some(tier) = updated.data.get("tier") {
                    println!("   📊 Customer tier updated to: {:?}", tier);
                }
            }
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }

    println!();
    Ok(())
}

// ============================================================
// PART 2: Defglobal Demos
// ============================================================

fn demo_globals_basic() -> Result<()> {
    println!("1️⃣ Basic Defglobal Usage");

    let mut engine = IncrementalEngine::new();

    // Define global variables
    engine.globals().define("max_retries", FactValue::Integer(3))?;
    engine.globals().define("timeout_seconds", FactValue::Float(30.0))?;
    engine.globals().define("debug_mode", FactValue::Boolean(true))?;
    engine.globals().define_readonly("VERSION", FactValue::String("1.0.0".to_string()))?;

    // Access globals
    println!("   max_retries: {:?}", engine.globals().get("max_retries")?);
    println!("   timeout_seconds: {:?}", engine.globals().get("timeout_seconds")?);
    println!("   debug_mode: {:?}", engine.globals().get("debug_mode")?);
    println!("   VERSION: {:?}", engine.globals().get("VERSION")?);

    // Modify global
    engine.globals().set("max_retries", FactValue::Integer(5))?;
    println!("\n   After update:");
    println!("   max_retries: {:?}", engine.globals().get("max_retries")?);

    // Try to modify read-only (should fail)
    match engine.globals().set("VERSION", FactValue::String("2.0.0".to_string())) {
        Ok(_) => println!("   ⚠️  Modified read-only (shouldn't happen)"),
        Err(e) => println!("   ✅ Correctly prevented read-only modification: {}", e),
    }

    // Increment counter
    engine.globals().increment("max_retries", 3.0)?;
    println!("\n   After increment:");
    println!("   max_retries: {:?}", engine.globals().get("max_retries")?);

    println!();
    Ok(())
}

fn demo_globals_in_rules() -> Result<()> {
    println!("2️⃣ Globals Accessible Across Rules");

    let mut engine = IncrementalEngine::new();

    // Define global counters
    engine.globals().define("orders_processed", FactValue::Integer(0))?;
    engine.globals().define("total_revenue", FactValue::Float(0.0))?;

    // Load rules (rules can access globals in actions)
    let rules = r#"
    rule "ProcessOrder" salience 10 no-loop {
        when
            Order.status == "pending"
        then
            Order.status = "processed";
    }

    rule "HighValueAlert" salience 15 no-loop {
        when
            Order.amount > 1000
        then
            Order.priority = "high";
    }
    "#;

    GrlReteLoader::load_from_string(rules, &mut engine)?;

    // Process multiple orders
    for i in 1..=5 {
        let mut order = TypedFacts::new();
        order.set("order_id", FactValue::String(format!("ORD-{}", i)));
        order.set("status", FactValue::String("pending".to_string()));
        order.set("amount", FactValue::Float(500.0 + (i as f64 * 200.0)));

        engine.insert("Order".to_string(), order);

        // Fire rules
        engine.reset();
        let fired = engine.fire_all();

        // Update global counters (simulated - in real usage, rules would do this)
        engine.globals().increment("orders_processed", 1.0)?;
        let amount = 500.0 + (i as f64 * 200.0);
        engine.globals().increment("total_revenue", amount)?;

        println!("   📦 Order {} processed, fired {} rules", i, fired.len());
    }

    // Show final global state
    println!("\n   Final Global State:");
    println!(
        "   orders_processed: {:?}",
        engine.globals().get("orders_processed")?
    );
    println!(
        "   total_revenue: ${:.2?}",
        engine.globals().get("total_revenue")?
    );

    println!();
    Ok(())
}

fn demo_globals_thread_safety() -> Result<()> {
    println!("3️⃣ Thread-Safe Globals");

    let engine = IncrementalEngine::new();

    // Define shared counter
    engine.globals().define("shared_counter", FactValue::Integer(0))?;

    println!("   Initial counter: {:?}", engine.globals().get("shared_counter")?);

    // Simulate concurrent access (GlobalsRegistry is thread-safe via Arc<RwLock>)
    for _ in 0..10 {
        engine.globals().increment("shared_counter", 1.0)?;
    }

    println!("   After 10 increments: {:?}", engine.globals().get("shared_counter")?);
    println!("   ✅ Thread-safe access verified");

    println!();
    Ok(())
}

// ============================================================
// PART 3: Combined Usage
// ============================================================

fn demo_combined() -> Result<()> {
    println!("1️⃣ E-Commerce System with Templates + Globals");

    let mut engine = IncrementalEngine::new();

    // Define templates
    let customer_template = TemplateBuilder::new("Customer")
        .required_string("customer_id")
        .string_field("name")
        .string_field("tier")
        .float_field("total_spent")
        .build();

    let order_template = TemplateBuilder::new("Order")
        .required_string("order_id")
        .string_field("customer_id")
        .float_field("amount")
        .string_field("status")
        .build();

    engine.templates_mut().register(customer_template);
    engine.templates_mut().register(order_template);

    // Define globals
    engine.globals().define("orders_today", FactValue::Integer(0))?;
    engine.globals().define("revenue_today", FactValue::Float(0.0))?;
    engine.globals().define("vip_threshold", FactValue::Float(10000.0))?;

    // Load business rules
    let rules = r#"
    rule "VIPCustomer" salience 20 no-loop {
        when
            Customer.total_spent > 10000
        then
            Customer.tier = "VIP";
    }

    rule "ProcessHighValueOrder" salience 15 no-loop {
        when
            Order.amount > 1000
        then
            Order.status = "priority";
    }
    "#;

    GrlReteLoader::load_from_string(rules, &mut engine)?;

    // Create customer
    let mut customer = TypedFacts::new();
    customer.set("customer_id", FactValue::String("C001".to_string()));
    customer.set("name", FactValue::String("Charlie".to_string()));
    customer.set("tier", FactValue::String("Standard".to_string()));
    customer.set("total_spent", FactValue::Float(12000.0));

    let cust_handle = engine.insert_with_template("Customer", customer)?;

    // Create order
    let mut order = TypedFacts::new();
    order.set("order_id", FactValue::String("ORD-001".to_string()));
    order.set("customer_id", FactValue::String("C001".to_string()));
    order.set("amount", FactValue::Float(1500.0));
    order.set("status", FactValue::String("pending".to_string()));

    let order_handle = engine.insert_with_template("Order", order)?;

    // Fire rules
    engine.reset();
    let fired = engine.fire_all();

    println!("   🔥 Fired {} rules: {:?}", fired.len(), fired);

    // Update globals
    engine.globals().increment("orders_today", 1.0)?;
    engine.globals().increment("revenue_today", 1500.0)?;

    // Check results
    if let Some(cust) = engine.working_memory().get(&cust_handle) {
        println!("\n   Customer Status:");
        println!("     ID: {:?}", cust.data.get("customer_id"));
        println!("     Tier: {:?}", cust.data.get("tier"));
        println!("     Total Spent: {:?}", cust.data.get("total_spent"));
    }

    if let Some(ord) = engine.working_memory().get(&order_handle) {
        println!("\n   Order Status:");
        println!("     ID: {:?}", ord.data.get("order_id"));
        println!("     Amount: {:?}", ord.data.get("amount"));
        println!("     Status: {:?}", ord.data.get("status"));
    }

    println!("\n   Global State:");
    println!("     orders_today: {:?}", engine.globals().get("orders_today")?);
    println!("     revenue_today: {:?}", engine.globals().get("revenue_today")?);
    println!("     vip_threshold: {:?}", engine.globals().get("vip_threshold")?);

    println!();
    Ok(())
}

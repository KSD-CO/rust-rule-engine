//! RETE Deffacts System Demo
//!
//! Demonstrates CLIPS-inspired deffacts feature:
//! - Initial fact definitions that auto-load into working memory
//! - Similar to CLIPS deffacts and Drools declared facts
//! - Useful for seed data, default entities, initial state
//!
//! Run: cargo run --example rete_deffacts_demo

use rust_rule_engine::rete::{
    IncrementalEngine, GrlReteLoader, DeffactsBuilder, TemplateBuilder, FactValue, TypedFacts,
};
use rust_rule_engine::errors::Result;

fn main() -> Result<()> {
    println!("=== RETE Deffacts System Demo ===\n");

    // ============================================================
    // PART 1: Basic Deffacts Usage
    // ============================================================
    println!("📦 Part 1: Basic Deffacts");
    println!("─────────────────────────────────────────\n");

    demo_deffacts_basic()?;
    demo_deffacts_multiple_sets()?;

    // ============================================================
    // PART 2: Deffacts with Templates
    // ============================================================
    println!("\n📋 Part 2: Deffacts with Templates");
    println!("─────────────────────────────────────────\n");

    demo_deffacts_with_templates()?;

    // ============================================================
    // PART 3: Deffacts with Rules
    // ============================================================
    println!("\n🔥 Part 3: Deffacts with Rules");
    println!("─────────────────────────────────────────\n");

    demo_deffacts_with_rules()?;

    // ============================================================
    // PART 4: Reset and Reload
    // ============================================================
    println!("\n🔄 Part 4: Reset and Reload");
    println!("─────────────────────────────────────────\n");

    demo_deffacts_reset()?;

    // ============================================================
    // PART 5: Real-World Example - E-Commerce System
    // ============================================================
    println!("\n🛒 Part 5: E-Commerce System with Deffacts");
    println!("─────────────────────────────────────────\n");

    demo_ecommerce_system()?;

    println!("\n✅ Demo completed successfully!");
    Ok(())
}

// ============================================================
// PART 1: Basic Deffacts
// ============================================================

fn demo_deffacts_basic() -> Result<()> {
    println!("1️⃣ Basic Deffacts Usage");

    let mut engine = IncrementalEngine::new();

    // Create initial person facts
    let mut person1 = TypedFacts::new();
    person1.set("name", FactValue::String("Alice".to_string()));
    person1.set("age", FactValue::Integer(30));
    person1.set("is_adult", FactValue::Boolean(true));

    let mut person2 = TypedFacts::new();
    person2.set("name", FactValue::String("Bob".to_string()));
    person2.set("age", FactValue::Integer(25));
    person2.set("is_adult", FactValue::Boolean(true));

    // Create deffacts using builder
    let initial_people = DeffactsBuilder::new("initial-people")
        .add_fact("Person", person1)
        .add_fact("Person", person2)
        .with_description("Initial person data for system startup")
        .build();

    println!("   Created deffacts: '{}'", initial_people.name);
    println!("   Description: {:?}", initial_people.description);
    println!("   Fact count: {}", initial_people.fact_count());

    // Register deffacts
    engine.deffacts_mut().register(initial_people)?;

    println!("\n   Registered deffacts sets: {:?}", engine.deffacts().list_deffacts());
    println!("   Total facts across all deffacts: {}", engine.deffacts().total_fact_count());

    // Load deffacts into working memory
    let handles = engine.load_deffacts();
    println!("\n   ✅ Loaded {} facts into working memory", handles.len());

    // Verify facts in working memory
    for (i, handle) in handles.iter().enumerate() {
        if let Some(fact) = engine.working_memory().get(handle) {
            println!("   Fact {}: type={}, data={:?}", i + 1, fact.fact_type, fact.data);
        }
    }

    println!();
    Ok(())
}

fn demo_deffacts_multiple_sets() -> Result<()> {
    println!("2️⃣ Multiple Deffacts Sets");

    let mut engine = IncrementalEngine::new();

    // First deffacts set - Users
    let mut admin = TypedFacts::new();
    admin.set("username", FactValue::String("admin".to_string()));
    admin.set("role", FactValue::String("administrator".to_string()));
    admin.set("active", FactValue::Boolean(true));

    let users_deffacts = DeffactsBuilder::new("system-users")
        .add_fact("User", admin)
        .with_description("System administrator accounts")
        .build();

    // Second deffacts set - Configuration
    let mut config = TypedFacts::new();
    config.set("max_users", FactValue::Integer(1000));
    config.set("debug_mode", FactValue::Boolean(false));
    config.set("timeout", FactValue::Float(30.0));

    let config_deffacts = DeffactsBuilder::new("system-config")
        .add_fact("Config", config)
        .with_description("System configuration settings")
        .build();

    // Register both
    engine.deffacts_mut().register(users_deffacts)?;
    engine.deffacts_mut().register(config_deffacts)?;

    println!("   Registered {} deffacts sets", engine.deffacts().len());

    // Load all deffacts
    let handles = engine.load_deffacts();
    println!("   ✅ Loaded {} facts from all deffacts", handles.len());

    // Show what was loaded
    for handle in &handles {
        if let Some(fact) = engine.working_memory().get(handle) {
            println!("   - {} fact loaded", fact.fact_type);
        }
    }

    println!();
    Ok(())
}

// ============================================================
// PART 2: Deffacts with Templates
// ============================================================

fn demo_deffacts_with_templates() -> Result<()> {
    println!("1️⃣ Deffacts with Template Validation");

    let mut engine = IncrementalEngine::new();

    // Define template for Customer
    let customer_template = TemplateBuilder::new("Customer")
        .required_string("customer_id")
        .string_field("name")
        .string_field("tier")
        .float_field("total_spent")
        .integer_field("loyalty_points")
        .build();

    engine.templates_mut().register(customer_template);

    // Create initial customers
    let mut customer1 = TypedFacts::new();
    customer1.set("customer_id", FactValue::String("C001".to_string()));
    customer1.set("name", FactValue::String("Premium Corp".to_string()));
    customer1.set("tier", FactValue::String("VIP".to_string()));
    customer1.set("total_spent", FactValue::Float(50000.0));
    customer1.set("loyalty_points", FactValue::Integer(5000));

    let mut customer2 = TypedFacts::new();
    customer2.set("customer_id", FactValue::String("C002".to_string()));
    customer2.set("name", FactValue::String("Startup Inc".to_string()));
    customer2.set("tier", FactValue::String("Standard".to_string()));
    customer2.set("total_spent", FactValue::Float(2000.0));
    customer2.set("loyalty_points", FactValue::Integer(200));

    // Create deffacts
    let customers_deffacts = DeffactsBuilder::new("initial-customers")
        .add_fact("Customer", customer1)
        .add_fact("Customer", customer2)
        .with_description("Initial VIP and standard customers")
        .build();

    engine.deffacts_mut().register(customers_deffacts)?;

    println!("   Template registered: Customer");
    println!("   Deffacts registered: initial-customers");

    // Load deffacts (will validate against template)
    let handles = engine.load_deffacts();

    println!("\n   ✅ Loaded {} customers (validated against template)", handles.len());

    for handle in &handles {
        if let Some(fact) = engine.working_memory().get(handle) {
            println!("\n   Customer:");
            println!("     ID: {:?}", fact.data.get("customer_id"));
            println!("     Name: {:?}", fact.data.get("name"));
            println!("     Tier: {:?}", fact.data.get("tier"));
            println!("     Total Spent: ${:?}", fact.data.get("total_spent"));
        }
    }

    println!();
    Ok(())
}

// ============================================================
// PART 3: Deffacts with Rules
// ============================================================

fn demo_deffacts_with_rules() -> Result<()> {
    println!("1️⃣ Deffacts Processing with Rules");

    let mut engine = IncrementalEngine::new();

    // Create initial orders (pending processing)
    let mut order1 = TypedFacts::new();
    order1.set("order_id", FactValue::String("ORD-001".to_string()));
    order1.set("amount", FactValue::Float(1500.0));
    order1.set("status", FactValue::String("pending".to_string()));
    order1.set("priority", FactValue::String("normal".to_string()));

    let mut order2 = TypedFacts::new();
    order2.set("order_id", FactValue::String("ORD-002".to_string()));
    order2.set("amount", FactValue::Float(500.0));
    order2.set("status", FactValue::String("pending".to_string()));
    order2.set("priority", FactValue::String("normal".to_string()));

    let mut order3 = TypedFacts::new();
    order3.set("order_id", FactValue::String("ORD-003".to_string()));
    order3.set("amount", FactValue::Float(5000.0));
    order3.set("status", FactValue::String("pending".to_string()));
    order3.set("priority", FactValue::String("normal".to_string()));

    // Create deffacts for pending orders
    let orders_deffacts = DeffactsBuilder::new("pending-orders")
        .add_fact("Order", order1)
        .add_fact("Order", order2)
        .add_fact("Order", order3)
        .with_description("Orders waiting to be processed")
        .build();

    engine.deffacts_mut().register(orders_deffacts)?;

    // Load business rules
    let rules = r#"
    rule "HighValueOrder" salience 20 no-loop {
        when
            Order.amount > 1000
        then
            Order.priority = "high";
    }

    rule "ProcessOrder" salience 10 no-loop {
        when
            Order.status == "pending"
        then
            Order.status = "processing";
    }
    "#;

    GrlReteLoader::load_from_string(rules, &mut engine)?;

    println!("   Loaded 2 business rules");
    println!("   Registered deffacts with {} orders", engine.deffacts().total_fact_count());

    // Load deffacts
    let handles = engine.load_deffacts();
    println!("\n   ✅ Loaded {} orders from deffacts", handles.len());

    // Fire rules
    engine.reset();
    let fired = engine.fire_all();

    println!("   🔥 Fired {} rules", fired.len());

    // Show results
    println!("\n   Order Processing Results:");
    for handle in &handles {
        if let Some(order) = engine.working_memory().get(handle) {
            println!("\n   Order: {:?}", order.data.get("order_id"));
            println!("     Amount: ${:?}", order.data.get("amount"));
            println!("     Status: {:?}", order.data.get("status"));
            println!("     Priority: {:?}", order.data.get("priority"));
        }
    }

    println!();
    Ok(())
}

// ============================================================
// PART 4: Reset and Reload
// ============================================================

fn demo_deffacts_reset() -> Result<()> {
    println!("1️⃣ Reset and Reload Deffacts");

    let mut engine = IncrementalEngine::new();

    // Create initial state
    let mut state = TypedFacts::new();
    state.set("counter", FactValue::Integer(0));
    state.set("status", FactValue::String("initialized".to_string()));

    let state_deffacts = DeffactsBuilder::new("initial-state")
        .add_fact("State", state)
        .with_description("Initial system state")
        .build();

    engine.deffacts_mut().register(state_deffacts)?;

    // Load deffacts
    let handles = engine.load_deffacts();
    println!("   Initial load: {} facts", handles.len());

    // Modify the fact
    if let Some(handle) = handles.first() {
        let mut modified_state = TypedFacts::new();
        modified_state.set("counter", FactValue::Integer(100));
        modified_state.set("status", FactValue::String("modified".to_string()));

        let _ = engine.update(*handle, modified_state);

        if let Some(fact) = engine.working_memory().get(handle) {
            println!("\n   After modification:");
            println!("     counter: {:?}", fact.data.get("counter"));
            println!("     status: {:?}", fact.data.get("status"));
        }
    }

    // Reset - this will reload deffacts to original state
    println!("\n   🔄 Resetting engine...");
    let new_handles = engine.reset_with_deffacts();
    println!("   Reloaded {} facts from deffacts", new_handles.len());

    // Verify reset to original state
    if let Some(handle) = new_handles.first() {
        if let Some(fact) = engine.working_memory().get(handle) {
            println!("\n   After reset:");
            println!("     counter: {:?}", fact.data.get("counter"));
            println!("     status: {:?}", fact.data.get("status"));
            println!("   ✅ State restored to original deffacts values");
        }
    }

    println!();
    Ok(())
}

// ============================================================
// PART 5: Real-World Example
// ============================================================

fn demo_ecommerce_system() -> Result<()> {
    println!("1️⃣ Complete E-Commerce System");

    let mut engine = IncrementalEngine::new();

    // Define templates
    let product_template = TemplateBuilder::new("Product")
        .required_string("product_id")
        .string_field("name")
        .float_field("price")
        .integer_field("stock")
        .boolean_field("available")
        .build();

    let customer_template = TemplateBuilder::new("Customer")
        .required_string("customer_id")
        .string_field("name")
        .string_field("tier")
        .float_field("discount")
        .build();

    engine.templates_mut().register(product_template);
    engine.templates_mut().register(customer_template);

    // Create initial products (seed data)
    let mut product1 = TypedFacts::new();
    product1.set("product_id", FactValue::String("P001".to_string()));
    product1.set("name", FactValue::String("Laptop Pro".to_string()));
    product1.set("price", FactValue::Float(1299.99));
    product1.set("stock", FactValue::Integer(50));
    product1.set("available", FactValue::Boolean(true));

    let mut product2 = TypedFacts::new();
    product2.set("product_id", FactValue::String("P002".to_string()));
    product2.set("name", FactValue::String("Wireless Mouse".to_string()));
    product2.set("price", FactValue::Float(29.99));
    product2.set("stock", FactValue::Integer(200));
    product2.set("available", FactValue::Boolean(true));

    let mut product3 = TypedFacts::new();
    product3.set("product_id", FactValue::String("P003".to_string()));
    product3.set("name", FactValue::String("USB-C Cable".to_string()));
    product3.set("price", FactValue::Float(15.99));
    product3.set("stock", FactValue::Integer(0));
    product3.set("available", FactValue::Boolean(true));

    // Create initial customers
    let mut customer1 = TypedFacts::new();
    customer1.set("customer_id", FactValue::String("C001".to_string()));
    customer1.set("name", FactValue::String("Gold Member".to_string()));
    customer1.set("tier", FactValue::String("gold".to_string()));
    customer1.set("discount", FactValue::Float(0.0));

    let mut customer2 = TypedFacts::new();
    customer2.set("customer_id", FactValue::String("C002".to_string()));
    customer2.set("name", FactValue::String("Regular Customer".to_string()));
    customer2.set("tier", FactValue::String("standard".to_string()));
    customer2.set("discount", FactValue::Float(0.0));

    // Create deffacts
    let products_deffacts = DeffactsBuilder::new("catalog-products")
        .add_fact("Product", product1)
        .add_fact("Product", product2)
        .add_fact("Product", product3)
        .with_description("Initial product catalog")
        .build();

    let customers_deffacts = DeffactsBuilder::new("registered-customers")
        .add_fact("Customer", customer1)
        .add_fact("Customer", customer2)
        .with_description("Registered customer accounts")
        .build();

    engine.deffacts_mut().register(products_deffacts)?;
    engine.deffacts_mut().register(customers_deffacts)?;

    // Load business rules
    let rules = r#"
    rule "OutOfStock" salience 25 no-loop {
        when
            Product.stock == 0
        then
            Product.available = false;
    }

    rule "GoldDiscount" salience 20 no-loop {
        when
            Customer.tier == "gold"
        then
            Customer.discount = 0.15;
    }

    rule "LowStockAlert" salience 15 no-loop {
        when
            Product.stock < 10 && Product.stock > 0
        then
            Product.available = true;
    }
    "#;

    GrlReteLoader::load_from_string(rules, &mut engine)?;

    println!("   System Configuration:");
    println!("   - Templates: 2 (Product, Customer)");
    println!("   - Deffacts: 2 sets ({} total facts)", engine.deffacts().total_fact_count());
    println!("   - Rules: 3 business rules");

    // Initialize system by loading deffacts
    println!("\n   🚀 Initializing system...");
    let handles = engine.load_deffacts();
    println!("   ✅ Loaded {} facts from deffacts", handles.len());

    // Fire rules to process initial data
    engine.reset();
    let fired = engine.fire_all();
    println!("   🔥 Fired {} rules during initialization", fired.len());

    // Show system state
    println!("\n   📊 System State After Initialization:");

    println!("\n   Products:");
    let mut product_count = 0;
    for handle in &handles {
        if let Some(fact) = engine.working_memory().get(handle) {
            if fact.fact_type == "Product" {
                product_count += 1;
                println!("\n     Product {}:", product_count);
                println!("       ID: {:?}", fact.data.get("product_id"));
                println!("       Name: {:?}", fact.data.get("name"));
                println!("       Price: ${:?}", fact.data.get("price"));
                println!("       Stock: {:?}", fact.data.get("stock"));
                println!("       Available: {:?}", fact.data.get("available"));
            }
        }
    }

    println!("\n   Customers:");
    let mut customer_count = 0;
    for handle in &handles {
        if let Some(fact) = engine.working_memory().get(handle) {
            if fact.fact_type == "Customer" {
                customer_count += 1;
                println!("\n     Customer {}:", customer_count);
                println!("       ID: {:?}", fact.data.get("customer_id"));
                println!("       Name: {:?}", fact.data.get("name"));
                println!("       Tier: {:?}", fact.data.get("tier"));
                println!("       Discount: {:?}", fact.data.get("discount"));
            }
        }
    }

    println!("\n   ✅ E-Commerce system initialized with deffacts!");

    println!();
    Ok(())
}

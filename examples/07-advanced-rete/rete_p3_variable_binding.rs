use rust_rule_engine::rete::facts::{FactValue, TypedFacts};
/// Demo: Variable Binding and Multi-Object Pattern Matching (P3 Feature)
///
/// This example demonstrates advanced Drools-style features:
/// - Variable binding ($var) across patterns
/// - Multi-object pattern matching
/// - Join conditions between facts
/// - Complex cross-fact constraints
use rust_rule_engine::rete::pattern::{MultiPattern, PatternBuilder};
use rust_rule_engine::rete::working_memory::WorkingMemory;

fn main() {
    println!("\n🔗 Variable Binding & Multi-Pattern Demo (Drools-style)");
    println!("========================================================\n");

    // Example 1: Simple Variable Binding
    println!("📋 Example 1: Simple Variable Binding");
    println!("-------------------------------------");

    let mut wm = WorkingMemory::new();

    let mut person = TypedFacts::new();
    person.set("name", "John Smith");
    person.set("age", 30i64);
    person.set("salary", 50000.0);

    wm.insert("Person".to_string(), person);

    // Pattern: Person($name, $age)
    let pattern = PatternBuilder::for_type("Person")
        .bind("name", "$name")
        .bind("age", "$age")
        .where_field("age", ">", FactValue::Integer(18))
        .build();

    let facts = wm.get_by_type("Person")[0];
    let matches = pattern.matches(&facts.data, &std::collections::HashMap::new());

    if let Some(bindings) = matches {
        println!("Pattern matched!");
        println!("  Bound variables:");
        for (var, value) in &bindings {
            println!("    {} = {}", var, value);
        }
    }

    // Example 2: Cross-Pattern Variable Binding (JOIN!)
    println!("\n📋 Example 2: Cross-Pattern JOIN with Variable Binding");
    println!("------------------------------------------------------");
    println!("Drools pattern: Person($name) AND Order(customer == $name)\n");

    let mut wm2 = WorkingMemory::new();

    // Insert multiple persons
    let mut john = TypedFacts::new();
    john.set("name", "John");
    john.set("age", 30i64);
    wm2.insert("Person".to_string(), john);

    let mut jane = TypedFacts::new();
    jane.set("name", "Jane");
    jane.set("age", 25i64);
    wm2.insert("Person".to_string(), jane);

    // Insert orders
    let mut order1 = TypedFacts::new();
    order1.set("customer", "John");
    order1.set("amount", 1000.0);
    wm2.insert("Order".to_string(), order1);

    let mut order2 = TypedFacts::new();
    order2.set("customer", "John");
    order2.set("amount", 500.0);
    wm2.insert("Order".to_string(), order2);

    let mut order3 = TypedFacts::new();
    order3.set("customer", "Jane");
    order3.set("amount", 750.0);
    wm2.insert("Order".to_string(), order3);

    // Multi-pattern: Person binds $name, Order uses $name
    let person_pattern = PatternBuilder::for_type("Person")
        .bind("name", "$name")
        .bind("age", "$age")
        .build();

    let order_pattern = PatternBuilder::for_type("Order")
        .where_var("customer", "==", "$name")
        .bind("amount", "$amount")
        .build();

    let multi = MultiPattern::new("PersonWithOrders".to_string())
        .with_pattern(person_pattern)
        .with_pattern(order_pattern);

    let matches = multi.match_all(&wm2);

    println!("Found {} person-order combinations:", matches.len());
    for (handles, bindings) in &matches {
        println!("  Person-Order pair:");
        println!("    Handles: {:?}", handles);
        println!("    Name: {}", bindings.get("$name").unwrap());
        println!("    Age: {}", bindings.get("$age").unwrap());
        println!("    Order amount: {}", bindings.get("$amount").unwrap());
        println!();
    }

    // Example 3: Complex Multi-Pattern with Constraints
    println!("\n📋 Example 3: Complex Multi-Pattern with Amount Constraint");
    println!("----------------------------------------------------------");
    println!("Drools: Person($name, $age) AND Order(customer == $name, amount > $age * 10)\n");

    let mut wm3 = WorkingMemory::new();

    let mut alice = TypedFacts::new();
    alice.set("name", "Alice");
    alice.set("age", 40i64);
    alice.set("min_order", 400i64); // age * 10
    wm3.insert("Person".to_string(), alice);

    let mut bob = TypedFacts::new();
    bob.set("name", "Bob");
    bob.set("age", 50i64);
    bob.set("min_order", 500i64); // age * 10
    wm3.insert("Person".to_string(), bob);

    let mut order_alice = TypedFacts::new();
    order_alice.set("customer", "Alice");
    order_alice.set("amount", 500.0); // > 40 * 10
    wm3.insert("Order".to_string(), order_alice);

    let mut order_bob = TypedFacts::new();
    order_bob.set("customer", "Bob");
    order_bob.set("amount", 300.0); // < 50 * 10
    wm3.insert("Order".to_string(), order_bob);

    // Pattern with cross-fact constraint
    let person_p3 = PatternBuilder::for_type("Person")
        .bind("name", "$name")
        .bind("min_order", "$minOrder")
        .build();

    let order_p3 = PatternBuilder::for_type("Order")
        .where_var("customer", "==", "$name")
        .where_var("amount", ">", "$minOrder")
        .build();

    let multi3 = MultiPattern::new("HighValueOrders".to_string())
        .with_pattern(person_p3)
        .with_pattern(order_p3);

    let matches3 = multi3.match_all(&wm3);

    println!("Found {} high-value person-order pairs:", matches3.len());
    for (_, bindings) in &matches3 {
        println!(
            "  Customer: {}, Min: {}",
            bindings.get("$name").unwrap(),
            bindings.get("$minOrder").unwrap()
        );
    }

    // Example 4: Three-Way Join
    println!("\n📋 Example 4: Three-Way Join (Person-Order-Product)");
    println!("--------------------------------------------------");
    println!("Drools: Person($name) AND Order(customer == $name, $product) AND Product(id == $product)\n");

    let mut wm4 = WorkingMemory::new();

    let mut customer = TypedFacts::new();
    customer.set("name", "Charlie");
    wm4.insert("Person".to_string(), customer);

    let mut order_charlie = TypedFacts::new();
    order_charlie.set("customer", "Charlie");
    order_charlie.set("product_id", "PROD123");
    wm4.insert("Order".to_string(), order_charlie);

    let mut product = TypedFacts::new();
    product.set("id", "PROD123");
    product.set("name", "Laptop");
    product.set("price", 999.99);
    wm4.insert("Product".to_string(), product);

    // Three-way join
    let person_p4 = PatternBuilder::for_type("Person")
        .bind("name", "$customerName")
        .build();

    let order_p4 = PatternBuilder::for_type("Order")
        .where_var("customer", "==", "$customerName")
        .bind("product_id", "$productId")
        .build();

    let product_p4 = PatternBuilder::for_type("Product")
        .where_var("id", "==", "$productId")
        .bind("name", "$productName")
        .bind("price", "$price")
        .build();

    let multi4 = MultiPattern::new("CustomerOrderProduct".to_string())
        .with_pattern(person_p4)
        .with_pattern(order_p4)
        .with_pattern(product_p4);

    let matches4 = multi4.match_all(&wm4);

    println!(
        "Found {} complete customer-order-product chains:",
        matches4.len()
    );
    for (handles, bindings) in &matches4 {
        println!("  Chain: {} handles", handles.len());
        println!("    Customer: {}", bindings.get("$customerName").unwrap());
        println!("    Product ID: {}", bindings.get("$productId").unwrap());
        println!(
            "    Product Name: {}",
            bindings.get("$productName").unwrap()
        );
        println!("    Price: {}", bindings.get("$price").unwrap());
    }

    // Statistics
    println!("\n📊 Working Memory Statistics");
    println!("============================");
    println!("{}", wm4.stats());

    // Summary
    println!("\n✨ Variable Binding & Multi-Pattern Features");
    println!("===========================================");
    println!("✅ Variable binding: $var syntax");
    println!("✅ Cross-pattern joins: Bind in pattern A, use in pattern B");
    println!("✅ Multi-object matching: Match across 2, 3, or more fact types");
    println!("✅ Complex constraints: field op $var with bound variables");
    println!("✅ Efficient: Only evaluates valid combinations");
    println!("\n🚀 Similar to Drools pattern matching:");
    println!("   Person($name, $age)");
    println!("   Order(customer == $name, amount > $age * 100)");
}

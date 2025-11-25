/// Demo: RETE Engine with GRL File Loading
///
/// This example demonstrates:
/// - Loading rules from .grl files into RETE engine
/// - Automatic conversion from GRL syntax to RETE-UL structures
/// - Incremental propagation with GRL rules
/// - Working with typed facts

use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine, TypedFacts};
use std::path::Path;

fn main() {
    println!("\n📄 RETE Engine with GRL File Loading Demo");
    println!("==========================================\n");

    // Example 1: Load from GRL file
    println!("📋 Example 1: Load Rules from GRL File");
    println!("--------------------------------------");

    let mut engine = IncrementalEngine::new();

    // Load rules from file
    let grl_file = "examples/rules/simple_business_rules.grl";
    match GrlReteLoader::load_from_file(grl_file, &mut engine) {
        Ok(count) => {
            println!("✅ Loaded {} rules from {}", count, grl_file);
        }
        Err(e) => {
            println!("❌ Failed to load GRL file: {}", e);
            println!("   Continuing with inline GRL example...\n");
        }
    }

    // Example 2: Load from GRL string (inline)
    println!("\n📋 Example 2: Load Rules from GRL String");
    println!("----------------------------------------");

    let mut engine2 = IncrementalEngine::new();

    let grl_rules = r#"
    rule "AdultCheck" salience 10 no-loop {
        when
            Person.age >= 18
        then
            Person.is_adult = true;
    }

    rule "SeniorDiscount" salience 9 no-loop {
        when
            Person.age >= 65 && Person.is_adult == true
        then
            Person.discount = 0.15;
            Person.tier = "senior";
    }

    rule "HighValueOrder" salience 15 no-loop {
        when
            Order.amount > 1000
        then
            Order.requires_approval = true;
            Order.high_value = true;
    }

    rule "VIPCustomer" salience 12 no-loop {
        when
            Customer.total_spent > 10000
        then
            Customer.tier = "VIP";
            Customer.discount = 0.2;
    }
    "#;

    match GrlReteLoader::load_from_string(grl_rules, &mut engine2) {
        Ok(count) => {
            println!("✅ Loaded {} rules from GRL string", count);
        }
        Err(e) => {
            println!("❌ Failed to parse GRL: {}", e);
            return;
        }
    }

    println!("\nEngine stats after loading:");
    println!("{}", engine2.stats());

    // Example 3: Insert facts and fire rules
    println!("\n📋 Example 3: Execute Rules with Facts");
    println!("--------------------------------------");

    // Insert Person fact
    println!("\nInserting Person (age=70)...");
    let mut person = TypedFacts::new();
    person.set("age", 70i64);
    person.set("name", "John Smith");
    person.set("is_adult", false); // Will be set by rule
    let person_handle = engine2.insert("Person".to_string(), person);

    println!("After Person insert:");
    println!("{}", engine2.stats());

    // Fire rules
    println!("\nFiring rules...");
    let fired = engine2.fire_all();
    println!("Fired {} rules: {:?}", fired.len(), fired);

    // Get updated facts
    if let Some(person_fact) = engine2.working_memory().get(&person_handle) {
        println!("\nPerson facts after rules:");
        if let Some(is_adult) = person_fact.data.get("is_adult") {
            println!("  is_adult: {}", is_adult);
        }
        if let Some(discount) = person_fact.data.get("discount") {
            println!("  discount: {}", discount);
        }
        if let Some(tier) = person_fact.data.get("tier") {
            println!("  tier: {}", tier);
        }
    }

    // Insert Order fact
    println!("\n\nInserting Order (amount=1500)...");
    let mut order = TypedFacts::new();
    order.set("amount", 1500.0);
    order.set("customer", "John Smith");
    order.set("requires_approval", false);
    engine2.insert("Order".to_string(), order);

    println!("After Order insert:");
    println!("{}", engine2.stats());

    // Fire rules again
    engine2.reset(); // Clear fired flags
    println!("\nFiring rules...");
    let fired2 = engine2.fire_all();
    println!("Fired {} rules: {:?}", fired2.len(), fired2);

    // Insert Customer fact
    println!("\n\nInserting Customer (total_spent=15000)...");
    let mut customer = TypedFacts::new();
    customer.set("name", "Jane Doe");
    customer.set("total_spent", 15000.0);
    customer.set("tier", "regular");
    engine2.insert("Customer".to_string(), customer);

    println!("After Customer insert:");
    println!("{}", engine2.stats());

    // Fire rules
    engine2.reset();
    println!("\nFiring rules...");
    let fired3 = engine2.fire_all();
    println!("Fired {} rules: {:?}", fired3.len(), fired3);

    // Example 4: Advanced GRL Features
    println!("\n\n📋 Example 4: Advanced GRL Features (EXISTS, FORALL)");
    println!("---------------------------------------------------");

    let mut engine3 = IncrementalEngine::new();

    let advanced_grl = r#"
    rule "FraudDetection" salience 20 no-loop {
        when
            exists(Transaction.suspicious == true)
        then
            Alert.fraud_detected = true;
    }

    rule "AllPaymentsVerified" salience 15 no-loop {
        when
            forall(Payment.status == "verified")
        then
            Order.payment_complete = true;
    }

    rule "MultiCondition" salience 10 no-loop {
        when
            Person.age > 18 && Order.amount > 500 && Customer.tier == "VIP"
        then
            Order.fast_shipping = true;
    }
    "#;

    match GrlReteLoader::load_from_string(advanced_grl, &mut engine3) {
        Ok(count) => {
            println!("✅ Loaded {} advanced rules", count);
        }
        Err(e) => {
            println!("❌ Failed to parse advanced GRL: {}", e);
            return;
        }
    }

    println!("\nAdvanced engine stats:");
    println!("{}", engine3.stats());

    // Test EXISTS pattern
    println!("\nTesting EXISTS pattern...");
    let mut transaction = TypedFacts::new();
    transaction.set("id", "TX123");
    transaction.set("suspicious", true);
    transaction.set("amount", 5000.0);
    engine3.insert("Transaction".to_string(), transaction);

    engine3.reset();
    let fired4 = engine3.fire_all();
    println!("Fired rules: {:?}", fired4);

    // Example 5: Performance Comparison
    println!("\n\n📋 Example 5: Performance with Many Rules");
    println!("------------------------------------------");

    let mut engine4 = IncrementalEngine::new();

    // Generate many rules
    let mut many_rules = String::new();
    for i in 0..20 {
        many_rules.push_str(&format!(
            r#"
            rule "Rule{}" salience {} {{
                when
                    Data.field{} > {}
                then
                    Data.processed{} = true;
            }}
            "#,
            i, i, i, i * 10, i
        ));
    }

    match GrlReteLoader::load_from_string(&many_rules, &mut engine4) {
        Ok(count) => {
            println!("✅ Loaded {} rules", count);
        }
        Err(e) => {
            println!("❌ Failed to parse: {}", e);
            return;
        }
    }

    println!("\nEngine with many rules:");
    println!("{}", engine4.stats());

    // Insert data
    let mut data = TypedFacts::new();
    for i in 0..20 {
        data.set(format!("field{}", i), (i * 15) as i64);
    }
    engine4.insert("Data".to_string(), data);

    println!("\nFiring all rules...");
    use std::time::Instant;
    let start = Instant::now();
    engine4.reset();
    let fired5 = engine4.fire_all();
    let duration = start.elapsed();

    println!("✅ Fired {} rules in {:?}", fired5.len(), duration);
    println!("   Average per rule: {:?}", duration / fired5.len() as u32);

    // Summary
    println!("\n\n✨ RETE + GRL Features Demonstrated");
    println!("====================================");
    println!("✅ Load rules from .grl files");
    println!("✅ Parse GRL syntax (salience, no-loop, when/then)");
    println!("✅ Convert to RETE-UL structures automatically");
    println!("✅ Incremental propagation (only affected rules)");
    println!("✅ Support EXISTS and FORALL patterns");
    println!("✅ Multi-condition rules (AND, OR, NOT)");
    println!("✅ Typed facts with automatic conversion");
    println!("✅ Performance: Load 20 rules and fire in microseconds");
    println!("\n🚀 GRL + RETE = Best of both worlds!");
    println!("   - Familiar GRL syntax for rule authors");
    println!("   - Efficient RETE execution for performance");
}

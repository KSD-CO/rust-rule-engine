/// Demo: CallFunction action in RETE engine
/// Shows how custom functions are called from GRL rules
use rust_rule_engine::rete::{FactValue, GrlReteLoader, IncrementalEngine, TypedFacts};
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 RETE CallFunction Demo");
    println!("========================\n");

    let mut engine = IncrementalEngine::new();

    // Track function calls
    let call_log = Arc::new(Mutex::new(Vec::<String>::new()));
    let call_log_clone = call_log.clone();

    // Register custom function: sendEmail
    engine.register_function("sendEmail", move |args, _facts| {
        if let Some(FactValue::String(email)) = args.first() {
            let msg = format!("Email sent to: {}", email);
            println!("📧 {}", msg);
            call_log_clone.lock().unwrap().push(msg);
        }
        Ok(FactValue::Boolean(true))
    });

    let call_log_clone2 = call_log.clone();

    // Register custom function: processPayment
    engine.register_function("processPayment", move |args, _facts| {
        if let Some(FactValue::String(method)) = args.first() {
            let msg = format!("Payment processed via: {}", method);
            println!("💳 {}", msg);
            call_log_clone2.lock().unwrap().push(msg);
        }
        Ok(FactValue::Boolean(true))
    });

    // Load GRL rules with function calls
    let grl = r#"
        rule "ProcessOrder" salience 100 no-loop {
            when
                Order.status == "pending"
            then
                Log("Processing order payment");
                processPayment(Order.paymentMethod);
                Order.status = "shipped";
        }
        
        rule "NotifyCustomer" salience 90 no-loop {
            when
                Order.status == "shipped"
            then
                Log("Order shipped, notifying customer");
                sendEmail(Customer.email);
        }
    "#;

    let count = GrlReteLoader::load_from_string(grl, &mut engine)?;
    println!("✅ Loaded {} rules\n", count);

    // DEBUG: Check what actions were parsed
    println!("🔍 DEBUG: Parsing GRL to check actions...");
    use rust_rule_engine::parser::grl::GRLParser;
    let parsed_rules = GRLParser::parse_rules(grl)?;
    for rule in &parsed_rules {
        println!("  Rule '{}': {} actions", rule.name, rule.actions.len());
        for (i, action) in rule.actions.iter().enumerate() {
            println!("    Action {}: {:?}", i, action);
        }
    }
    println!();

    // Insert Customer fact
    let mut customer = TypedFacts::new();
    customer.set(
        "email",
        FactValue::String("customer@example.com".to_string()),
    );
    engine.insert("Customer".to_string(), customer);

    // Insert Order fact
    let mut order = TypedFacts::new();
    order.set("status", FactValue::String("pending".to_string()));
    order.set(
        "paymentMethod",
        FactValue::String("credit_card".to_string()),
    );
    engine.insert("Order".to_string(), order);

    println!("🏁 Firing rules...\n");
    engine.reset();
    let fired = engine.fire_all();

    println!("\n📊 Results:");
    println!("  Rules fired: {}", fired.len());
    for rule in &fired {
        println!("    - {}", rule);
    }

    let logs = call_log.lock().unwrap();
    println!("\n📝 Function calls:");
    for log in logs.iter() {
        println!("    ✓ {}", log);
    }

    // Verify both functions were called
    assert_eq!(logs.len(), 2, "Expected 2 function calls");
    assert!(logs.iter().any(|l| l.contains("Payment processed")));
    assert!(logs.iter().any(|l| l.contains("Email sent")));

    println!("\n✅ CallFunction action works correctly!");

    Ok(())
}

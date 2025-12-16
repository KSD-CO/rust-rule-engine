/// Demo: Retract with RETE Engine - True Working Memory
///
/// This example demonstrates retract() with RETE-UL engine:
/// - Real WorkingMemory with FactHandles
/// - Actual retract() method (not just marking)
/// - CLIPS/Drools-style working memory management
/// - FactHandle tracking like Drools
use rust_rule_engine::rete::{FactValue, IncrementalEngine, TemplateBuilder, TypedFacts};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🗑️ Retract Demo - RETE Engine with Real Working Memory");
    println!("========================================================\n");

    // Example 1: Session Management with RETE
    println!("📋 Example 1: Session Management (RETE)");
    println!("---------------------------------------");

    let mut engine = IncrementalEngine::new();

    // Define Session template
    let session_template = TemplateBuilder::new("Session")
        .required_string("id")
        .boolean_field("expired")
        .boolean_field("active")
        .build();
    engine.templates_mut().register(session_template);

    // Insert session fact
    let mut session = TypedFacts::new();
    session.set("id", FactValue::String("sess-12345".to_string()));
    session.set("expired", FactValue::Boolean(false));
    session.set("active", FactValue::Boolean(true));

    let session_handle = engine.insert_with_template("Session", session)?;
    println!("✅ Inserted session: {:?}", session_handle);

    // Check working memory
    println!(
        "📊 Working memory stats: {}",
        engine.working_memory().stats()
    );

    // Simulate session expiration
    let mut updated_session = TypedFacts::new();
    updated_session.set("id", FactValue::String("sess-12345".to_string()));
    updated_session.set("expired", FactValue::Boolean(true));
    updated_session.set("active", FactValue::Boolean(true));

    engine.update(session_handle, updated_session)?;
    println!("🔄 Updated session to expired=true");

    // Now retract the session
    println!("\n🗑️ Retracting session...");
    engine.retract(session_handle)?;
    println!("✅ Session retracted!");

    // Check working memory again
    println!(
        "📊 Working memory stats: {}",
        engine.working_memory().stats()
    );

    // Try to get retracted fact (should be None)
    let retrieved = engine.working_memory().get(&session_handle);
    println!("🔍 Try to get retracted fact: {:?}", retrieved);
    assert!(
        retrieved.is_none(),
        "Retracted fact should not be accessible!"
    );

    // Example 2: User Management
    println!("\n📋 Example 2: User Management (RETE)");
    println!("------------------------------------");

    let mut engine2 = IncrementalEngine::new();

    let user_template = TemplateBuilder::new("User")
        .required_string("username")
        .boolean_field("verified")
        .integer_field("loginAttempts")
        .string_field("status")
        .build();
    engine2.templates_mut().register(user_template);

    // Insert multiple users
    let mut user1 = TypedFacts::new();
    user1.set("username", FactValue::String("valid_user".to_string()));
    user1.set("verified", FactValue::Boolean(true));
    user1.set("loginAttempts", FactValue::Integer(1));
    user1.set("status", FactValue::String("active".to_string()));

    let mut user2 = TypedFacts::new();
    user2.set("username", FactValue::String("suspicious_user".to_string()));
    user2.set("verified", FactValue::Boolean(false));
    user2.set("loginAttempts", FactValue::Integer(5));
    user2.set("status", FactValue::String("pending".to_string()));

    let h1 = engine2.insert_with_template("User", user1)?;
    let h2 = engine2.insert_with_template("User", user2)?;

    println!("✅ Inserted 2 users");
    println!(
        "📊 Active facts: {}",
        engine2.working_memory().stats().active_facts
    );

    // Retract suspicious user
    println!("\n🗑️ Removing suspicious user...");
    engine2.retract(h2)?;

    println!(
        "📊 Active facts: {}",
        engine2.working_memory().stats().active_facts
    );
    println!(
        "📊 Retracted facts: {}",
        engine2.working_memory().stats().retracted_facts
    );

    // Example 3: Bulk Operations
    println!("\n📋 Example 3: Bulk Retraction (RETE)");
    println!("------------------------------------");

    let mut engine3 = IncrementalEngine::new();

    let order_template = TemplateBuilder::new("Order")
        .required_string("id")
        .string_field("status")
        .boolean_field("shipped")
        .build();
    engine3.templates_mut().register(order_template);

    // Insert 10 orders
    let mut handles = Vec::new();
    for i in 1..=10 {
        let mut order = TypedFacts::new();
        order.set("id", FactValue::String(format!("ORD-{:03}", i)));
        order.set(
            "status",
            FactValue::String(if i % 2 == 0 { "completed" } else { "pending" }.to_string()),
        );
        order.set("shipped", FactValue::Boolean(i % 2 == 0));

        let h = engine3.insert_with_template("Order", order)?;
        handles.push(h);
    }

    println!("✅ Inserted 10 orders");
    println!(
        "📊 Active facts: {}",
        engine3.working_memory().stats().active_facts
    );

    // Retract all completed orders (5 orders)
    println!("\n🗑️ Retracting completed orders...");
    let mut retracted_count = 0;
    for (i, handle) in handles.iter().enumerate() {
        if i % 2 == 1 {
            // Even index means completed (i+1 is even number)
            engine3.retract(*handle)?;
            retracted_count += 1;
        }
    }

    println!("✅ Retracted {} completed orders", retracted_count);
    println!(
        "📊 Active facts: {}",
        engine3.working_memory().stats().active_facts
    );
    println!(
        "📊 Retracted facts: {}",
        engine3.working_memory().stats().retracted_facts
    );

    // Example 4: Query after Retraction
    println!("\n📋 Example 4: Query After Retraction");
    println!("------------------------------------");

    // Get all active orders (should be 5 pending orders)
    let all_facts = engine3.working_memory().get_all_facts();
    println!("🔍 Active orders remaining: {}", all_facts.len());

    for fact in all_facts {
        let id = fact.data.get("id").unwrap().as_string();
        let status = fact.data.get("status").unwrap().as_string();
        println!("  - {}: {}", id, status);
    }

    // Summary
    println!("\n✨ RETE Working Memory Features");
    println!("================================");
    println!("✅ Real FactHandle system (like Drools)");
    println!("✅ Actual retract() method removes facts");
    println!("✅ Working memory statistics tracking");
    println!("✅ Type-safe templates for facts");
    println!("✅ Insert/Update/Retract operations");
    println!("✅ Query remaining facts after retraction");
    println!("\n🚀 This is the TRUE RETE/Drools-style working memory!");

    Ok(())
}

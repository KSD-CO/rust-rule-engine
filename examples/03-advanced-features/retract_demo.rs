/// Demo: Retract Facts - CLIPS-style
///
/// This example demonstrates the retract() action in GRL rules:
/// - retract($Object) - Remove facts from working memory (CLIPS-style)
/// - Marks facts as retracted to prevent further rule matches
/// - Useful for cleanup, session management, and workflow completion
use rust_rule_engine::engine::engine::RustRuleEngine;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::{Facts, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🗑️ Retract Demo - CLIPS-style Fact Retraction");
    println!("===============================================\n");

    // Example 1: Retract expired session
    println!("📋 Example 1: Retract Expired Session");
    println!("-------------------------------------");

    // Load and parse GRL rules
    let grl_content = std::fs::read_to_string("examples/rules/03-advanced/retract_demo.grl")?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    let kb = KnowledgeBase::new("retract_demo");
    for rule in &rules {
        kb.add_rule(rule.clone());
    }

    let mut engine = RustRuleEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("Session.expired", Value::Boolean(true));
    facts.set("Session.active", Value::Boolean(true));
    facts.set("Session.id", Value::String("sess-12345".to_string()));

    println!("Before retract:");
    println!("  Session.expired: {:?}", facts.get("Session.expired"));
    println!("  Session.active: {:?}", facts.get("Session.active"));

    engine.execute(&mut facts)?;

    println!("\nAfter retract:");
    println!("  Session.active: {:?}", facts.get("Session.active"));
    println!(
        "  _retracted_Session: {:?}",
        facts.get("_retracted_Session")
    );

    // Example 2: Delete invalid user
    println!("\n📋 Example 2: Delete Invalid User");
    println!("----------------------------------");

    let kb2 = KnowledgeBase::new("retract_demo");
    for rule in &rules {
        kb2.add_rule(rule.clone());
    }
    let mut engine2 = RustRuleEngine::new(kb2);

    let mut facts2 = Facts::new();
    facts2.set("User.verified", Value::Boolean(false));
    facts2.set("User.loginAttempts", Value::Integer(5));
    facts2.set(
        "User.username",
        Value::String("suspicious_user".to_string()),
    );

    println!("Before delete:");
    println!("  User.verified: {:?}", facts2.get("User.verified"));
    println!(
        "  User.loginAttempts: {:?}",
        facts2.get("User.loginAttempts")
    );

    engine2.execute(&mut facts2)?;

    println!("\nAfter delete:");
    println!("  User.status: {:?}", facts2.get("User.status"));
    println!("  _retracted_User: {:?}", facts2.get("_retracted_User"));

    // Example 3: Cleanup completed order
    println!("\n📋 Example 3: Cleanup Completed Order");
    println!("-------------------------------------");

    let kb3 = KnowledgeBase::new("retract_demo");
    for rule in &rules {
        kb3.add_rule(rule.clone());
    }
    let mut engine3 = RustRuleEngine::new(kb3);

    let mut facts3 = Facts::new();
    facts3.set("Order.status", Value::String("completed".to_string()));
    facts3.set("Order.shipped", Value::Boolean(true));
    facts3.set("Order.id", Value::String("ORD-98765".to_string()));

    println!("Before retract:");
    println!("  Order.status: {:?}", facts3.get("Order.status"));
    println!("  Order.shipped: {:?}", facts3.get("Order.shipped"));

    engine3.execute(&mut facts3)?;

    println!("\nAfter retract:");
    println!("  Order.archived: {:?}", facts3.get("Order.archived"));
    println!("  _retracted_Order: {:?}", facts3.get("_retracted_Order"));

    // Example 4: Remove discontinued product
    println!("\n📋 Example 4: Remove Discontinued Product");
    println!("------------------------------------------");

    let kb4 = KnowledgeBase::new("retract_demo");
    for rule in &rules {
        kb4.add_rule(rule.clone());
    }
    let mut engine4 = RustRuleEngine::new(kb4);

    let mut facts4 = Facts::new();
    facts4.set("Product.stock", Value::Integer(0));
    facts4.set("Product.discontinued", Value::Boolean(true));
    facts4.set("Product.name", Value::String("Old Widget".to_string()));

    println!("Before delete:");
    println!("  Product.stock: {:?}", facts4.get("Product.stock"));
    println!(
        "  Product.discontinued: {:?}",
        facts4.get("Product.discontinued")
    );

    engine4.execute(&mut facts4)?;

    println!("\nAfter delete:");
    println!(
        "  _retracted_Product: {:?}",
        facts4.get("_retracted_Product")
    );

    // Summary
    println!("\n✨ Retract Feature Summary");
    println!("==========================");
    println!("✅ retract($Object) - CLIPS-style syntax");
    println!("✅ Marks facts as retracted in working memory");
    println!("✅ Prevents retracted facts from matching future rules");
    println!("✅ Useful for cleanup, session management, workflows");
    println!("\n📖 Similar to CLIPS:");
    println!("   CLIPS: (retract ?f)");
    println!("   Rust Rule Engine: retract($Object)");

    Ok(())
}

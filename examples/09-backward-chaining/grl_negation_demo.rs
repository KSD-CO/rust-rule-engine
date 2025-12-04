//! GRL Negation Queries Demo
//!
//! Demonstrates loading and executing negation queries from GRL files
//!
//! Run: cargo run --example grl_negation_demo --features backward-chaining

use rust_rule_engine::backward::{BackwardEngine, GRLQuery};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::KnowledgeBase;
use rust_rule_engine::types::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 GRL Negation Queries Demo");
    println!("{}", "=".repeat(80));
    println!();

    // Demo 1: Programmatic GRL queries with NOT
    demo_programmatic_negation()?;

    // Demo 2: Real-world scenarios
    demo_real_world_negation()?;

    println!("\n{}", "=".repeat(80));
    println!("✅ All GRL negation demos completed successfully!");

    Ok(())
}

/// Demo 1: Create GRL queries programmatically with NOT
fn demo_programmatic_negation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Demo 1: Programmatic GRL Queries with NOT");
    println!("{}", "-".repeat(80));

    let kb = KnowledgeBase::new("NegationDemo");
    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();

    // Setup facts
    facts.set("alice.name", Value::String("Alice".to_string()));
    facts.set("alice.is_banned", Value::Boolean(false));

    facts.set("bob.name", Value::String("Bob".to_string()));
    facts.set("bob.is_banned", Value::Boolean(true));

    facts.set("charlie.name", Value::String("Charlie".to_string()));
    // Charlie has no is_banned field (closed-world: not banned)

    println!("Users:");
    println!("  Alice: is_banned = false");
    println!("  Bob: is_banned = true");
    println!("  Charlie: (no banned field)");
    println!();

    // Create GRL query with NOT
    let not_banned_query = GRLQuery::new(
        "NotBannedUsers".to_string(),
        "NOT User.IsBanned == true".to_string(),
    )
    .with_max_depth(10)
    .with_memoization(true);

    println!("Created GRL Query:");
    println!("  Name: {}", not_banned_query.name);
    println!("  Goal: {}", not_banned_query.goal);
    println!("  Max Depth: {}", not_banned_query.max_depth);
    println!("  Memoization: {}", not_banned_query.enable_memoization);
    println!();

    // Test with each user (using the goal string directly)
    println!("Testing NOT banned query:");

    println!("  Alice (is_banned=false):");
    let result = engine.query("NOT alice.is_banned == true", &mut facts)?;
    println!("    Result: {} ✓", if result.provable { "NOT banned" } else { "IS banned" });

    println!("  Bob (is_banned=true):");
    let result = engine.query("NOT bob.is_banned == true", &mut facts)?;
    println!("    Result: {} ✓", if result.provable { "NOT banned" } else { "IS banned" });

    println!("  Charlie (no field):");
    let result = engine.query("NOT charlie.is_banned == true", &mut facts)?;
    println!("    Result: {} ✓ (closed-world assumption)",
        if result.provable { "NOT banned" } else { "IS banned" });

    println!();
    Ok(())
}

/// Demo 2: Real-world negation scenarios
fn demo_real_world_negation() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Demo 2: Real-World Negation Scenarios");
    println!("{}", "-".repeat(80));

    let kb = KnowledgeBase::new("RealWorldDemo");
    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();

    // Scenario 1: E-commerce order approval
    println!("Scenario 1: E-commerce Order Auto-Approval");
    facts.set("order1.id", Value::String("ORD-001".to_string()));
    facts.set("order1.amount", Value::Number(50.0));
    // No requires_approval field = auto-approved

    facts.set("order2.id", Value::String("ORD-002".to_string()));
    facts.set("order2.amount", Value::Number(5000.0));
    facts.set("order2.requires_approval", Value::Boolean(true));

    println!("  Order 1: $50 (no approval flag)");
    println!("  Order 2: $5000 (requires_approval=true)");
    println!();

    // Create query: Orders that do NOT require approval
    let auto_approved = GRLQuery::new(
        "AutoApprovedOrders".to_string(),
        "NOT Order.RequiresApproval == true".to_string(),
    );

    println!("  Query: {} - {}", auto_approved.name, auto_approved.goal);
    println!("  Order 1 (NOT requires_approval):");
    let result = engine.query("NOT order1.requires_approval == true", &mut facts)?;
    println!("    → {} (can auto-process)",
        if result.provable { "✅ Auto-approved" } else { "❌ Needs approval" });

    println!("  Order 2 (requires_approval=true):");
    let result = engine.query("NOT order2.requires_approval == true", &mut facts)?;
    println!("    → {} (needs manual review)",
        if result.provable { "✅ Auto-approved" } else { "❌ Needs approval" });
    println!();

    // Scenario 2: User access control
    println!("Scenario 2: User Access Control");
    facts.set("user1.email", Value::String("user1@example.com".to_string()));
    facts.set("user1.is_active", Value::Boolean(true));
    // No is_suspended field = not suspended

    facts.set("user2.email", Value::String("user2@example.com".to_string()));
    facts.set("user2.is_active", Value::Boolean(true));
    facts.set("user2.is_suspended", Value::Boolean(true));

    println!("  User 1: active, (no suspension record)");
    println!("  User 2: active, is_suspended=true");
    println!();

    // Query: Active AND NOT suspended
    let access_query = GRLQuery::new(
        "AllowAccess".to_string(),
        "NOT User.IsSuspended == true".to_string(),
    );

    println!("  Query: {} - {}", access_query.name, access_query.goal);

    println!("  User 1 (NOT suspended):");
    let active1 = engine.query("user1.is_active == true", &mut facts)?;
    let not_suspended1 = engine.query("NOT user1.is_suspended == true", &mut facts)?;
    println!("    Active: {} | Not Suspended: {}", active1.provable, not_suspended1.provable);
    println!("    → {} access",
        if active1.provable && not_suspended1.provable { "✅ Allow" } else { "❌ Deny" });

    println!("  User 2 (IS suspended):");
    let active2 = engine.query("user2.is_active == true", &mut facts)?;
    let not_suspended2 = engine.query("NOT user2.is_suspended == true", &mut facts)?;
    println!("    Active: {} | Not Suspended: {}", active2.provable, not_suspended2.provable);
    println!("    → {} access (suspended)",
        if active2.provable && not_suspended2.provable { "✅ Allow" } else { "❌ Deny" });
    println!();

    // Scenario 3: Inventory availability
    println!("Scenario 3: Inventory Availability Check");
    facts.set("item1.sku", Value::String("SKU-001".to_string()));
    facts.set("item1.qty", Value::Integer(100));
    // No reserved field = not reserved

    facts.set("item2.sku", Value::String("SKU-002".to_string()));
    facts.set("item2.qty", Value::Integer(50));
    facts.set("item2.reserved", Value::Boolean(true));

    println!("  Item 1: qty=100, (not reserved)");
    println!("  Item 2: qty=50, reserved=true");
    println!();

    let available_query = GRLQuery::new(
        "AvailableInventory".to_string(),
        "NOT Item.Reserved == true".to_string(),
    );

    println!("  Query: {} - {}", available_query.name, available_query.goal);

    println!("  Item 1:");
    let result = engine.query("NOT item1.reserved == true", &mut facts)?;
    println!("    → {} for sale",
        if result.provable { "✅ Available" } else { "❌ Reserved" });

    println!("  Item 2:");
    let result = engine.query("NOT item2.reserved == true", &mut facts)?;
    println!("    → {} (reserved for customer)",
        if result.provable { "✅ Available" } else { "❌ Reserved" });
    println!();

    Ok(())
}

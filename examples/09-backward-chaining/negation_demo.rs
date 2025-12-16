//! Negation in Backward Chaining Demo
//!
//! Demonstrates how to use NOT keyword for negated goals in backward chaining
//!
//! Run: cargo run --example negation_demo --features backward-chaining

use rust_rule_engine::backward::BackwardEngine;
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::types::Value;
use rust_rule_engine::KnowledgeBase;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚫 Negation in Backward Chaining Demo");
    println!("{}", "=".repeat(80));
    println!();

    // Demo 1: Simple NOT - User is not banned
    demo_not_banned()?;

    // Demo 2: NOT with rules - Available items (not sold)
    demo_available_items()?;

    // Demo 3: NOT with multiple conditions - Eligible users
    demo_eligible_users()?;

    // Demo 4: Closed-world assumption - Non-members
    demo_closed_world()?;

    println!("\n{}", "=".repeat(80));
    println!("✅ All negation demos completed successfully!");

    Ok(())
}

/// Demo 1: Simple NOT - Check if user is NOT banned
fn demo_not_banned() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 1: Simple NOT - User is NOT Banned");
    println!("{}", "-".repeat(80));

    let kb = KnowledgeBase::new("NotBannedDemo");
    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();

    // Set up facts: Alice is NOT banned, Bob IS banned
    facts.set("alice.name", Value::String("Alice".to_string()));
    facts.set("alice.is_banned", Value::Boolean(false));

    facts.set("bob.name", Value::String("Bob".to_string()));
    facts.set("bob.is_banned", Value::Boolean(true));

    println!("Facts:");
    println!("  Alice: is_banned = false");
    println!("  Bob: is_banned = true");
    println!();

    // Query 1: Normal positive query - Check if Bob is banned
    println!("Query 1: Is Bob banned?");
    let result = engine.query("bob.is_banned == true", &mut facts)?;
    println!(
        "  Result: {}",
        if result.provable {
            "✅ YES (proven)"
        } else {
            "❌ NO (not proven)"
        }
    );
    println!();

    // Query 2: Negated query - Alice is NOT banned
    println!("Query 2: Is Alice NOT banned? (NOT alice.is_banned == true)");
    let result = engine.query("NOT alice.is_banned == true", &mut facts)?;
    println!(
        "  Result: {}",
        if result.provable {
            "✅ YES (Alice is not banned)"
        } else {
            "❌ NO"
        }
    );
    println!();

    // Query 3: Negated query - Bob is NOT banned (should fail)
    println!("Query 3: Is Bob NOT banned? (NOT bob.is_banned == true)");
    let result = engine.query("NOT bob.is_banned == true", &mut facts)?;
    println!(
        "  Result: {}",
        if result.provable {
            "✅ YES"
        } else {
            "❌ NO (Bob IS banned, so NOT fails)"
        }
    );
    println!();

    Ok(())
}

/// Demo 2: Available Items (NOT Sold) - Simplified without rules
fn demo_available_items() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Demo 2: Available Items (NOT Sold)");
    println!("{}", "-".repeat(80));

    let kb = KnowledgeBase::new("AvailableItemsDemo");
    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();

    // Add facts about items
    facts.set("laptop.name", Value::String("Laptop".to_string()));
    facts.set("laptop.price", Value::Number(999.99));

    facts.set("mouse.name", Value::String("Mouse".to_string()));
    facts.set("mouse.price", Value::Number(29.99));
    facts.set("mouse.sold", Value::Boolean(true)); // Mouse is sold!

    facts.set("keyboard.name", Value::String("Keyboard".to_string()));
    facts.set("keyboard.price", Value::Number(79.99));

    println!("Items:");
    println!("  Laptop - $999.99 (not sold)");
    println!("  Mouse - $29.99 (SOLD)");
    println!("  Keyboard - $79.99 (not sold)");
    println!();

    // Query: Which items are available (NOT sold)?
    println!("Query: Is Laptop available? (NOT laptop.sold == true)");
    let result = engine.query("NOT laptop.sold == true", &mut facts)?;
    println!(
        "  Result: {} (Laptop has no 'sold' field, so it's available)",
        if result.provable {
            "✅ AVAILABLE"
        } else {
            "❌ NOT AVAILABLE"
        }
    );
    println!();

    println!("Query: Is Mouse available? (NOT mouse.sold == true)");
    let result = engine.query("NOT mouse.sold == true", &mut facts)?;
    println!(
        "  Result: {} (Mouse IS sold)",
        if result.provable {
            "✅ AVAILABLE"
        } else {
            "❌ NOT AVAILABLE"
        }
    );
    println!();

    println!("Query: Is Keyboard available? (NOT keyboard.sold == true)");
    let result = engine.query("NOT keyboard.sold == true", &mut facts)?;
    println!(
        "  Result: {} (Keyboard has no 'sold' field, so it's available)",
        if result.provable {
            "✅ AVAILABLE"
        } else {
            "❌ NOT AVAILABLE"
        }
    );
    println!();

    Ok(())
}

/// Demo 3: NOT with multiple conditions - Eligible users
fn demo_eligible_users() -> Result<(), Box<dyn std::error::Error>> {
    println!("👥 Demo 3: Eligible Users (NOT Banned AND Active)");
    println!("{}", "-".repeat(80));

    let kb = KnowledgeBase::new("EligibleUsersDemo");
    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();

    // User 1: Alice - Active, not banned
    facts.set("alice.name", Value::String("Alice".to_string()));
    facts.set("alice.is_active", Value::Boolean(true));
    // No banned field = not banned (closed-world assumption)

    // User 2: Bob - Active but banned
    facts.set("bob.name", Value::String("Bob".to_string()));
    facts.set("bob.is_active", Value::Boolean(true));
    facts.set("bob.is_banned", Value::Boolean(true));

    // User 3: Charlie - Not active, not banned
    facts.set("charlie.name", Value::String("Charlie".to_string()));
    facts.set("charlie.is_active", Value::Boolean(false));

    println!("Users:");
    println!("  Alice: is_active=true, is_banned=(not set, i.e., false)");
    println!("  Bob: is_active=true, is_banned=true");
    println!("  Charlie: is_active=false, is_banned=(not set)");
    println!();

    // Check eligibility: Active AND NOT banned
    println!("Eligibility Check: User must be active AND NOT banned");
    println!();

    println!("Is Alice eligible?");
    let active = engine.query("alice.is_active == true", &mut facts)?;
    let not_banned = engine.query("NOT alice.is_banned == true", &mut facts)?;
    let eligible = active.provable && not_banned.provable;
    println!("  is_active: {}", active.provable);
    println!("  NOT is_banned: {}", not_banned.provable);
    println!(
        "  Result: {} (Active AND NOT Banned)",
        if eligible {
            "✅ ELIGIBLE"
        } else {
            "❌ NOT ELIGIBLE"
        }
    );
    println!();

    println!("Is Bob eligible?");
    let active = engine.query("bob.is_active == true", &mut facts)?;
    let not_banned = engine.query("NOT bob.is_banned == true", &mut facts)?;
    let eligible = active.provable && not_banned.provable;
    println!("  is_active: {}", active.provable);
    println!("  NOT is_banned: {}", not_banned.provable);
    println!(
        "  Result: {} (Active but IS banned)",
        if eligible {
            "✅ ELIGIBLE"
        } else {
            "❌ NOT ELIGIBLE"
        }
    );
    println!();

    println!("Is Charlie eligible?");
    let active = engine.query("charlie.is_active == true", &mut facts)?;
    let not_banned = engine.query("NOT charlie.is_banned == true", &mut facts)?;
    let eligible = active.provable && not_banned.provable;
    println!("  is_active: {}", active.provable);
    println!("  NOT is_banned: {}", not_banned.provable);
    println!(
        "  Result: {} (NOT active, even though not banned)",
        if eligible {
            "✅ ELIGIBLE"
        } else {
            "❌ NOT ELIGIBLE"
        }
    );
    println!();

    Ok(())
}

/// Demo 4: Closed-world assumption - Non-members
fn demo_closed_world() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌍 Demo 4: Closed-World Assumption (Non-Members)");
    println!("{}", "-".repeat(80));

    let kb = KnowledgeBase::new("ClosedWorldDemo");
    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();

    // Only define members explicitly
    facts.set("alice.is_member", Value::Boolean(true));
    facts.set("bob.is_member", Value::Boolean(true));
    // Charlie is NOT defined as a member

    println!("Members:");
    println!("  Alice: is_member = true");
    println!("  Bob: is_member = true");
    println!("  Charlie: (no membership record)");
    println!();

    println!("Closed-World Assumption: If not stated, assume FALSE");
    println!();

    // Check who is NOT a member
    println!("Is Alice a NON-member? (NOT alice.is_member == true)");
    let result = engine.query("NOT alice.is_member == true", &mut facts)?;
    println!(
        "  Result: {} (Alice IS a member)",
        if result.provable {
            "✅ YES, non-member"
        } else {
            "❌ NO, IS a member"
        }
    );
    println!();

    println!("Is Charlie a NON-member? (NOT charlie.is_member == true)");
    let result = engine.query("NOT charlie.is_member == true", &mut facts)?;
    println!(
        "  Result: {} (No membership record = non-member under closed-world)",
        if result.provable {
            "✅ YES, non-member"
        } else {
            "❌ NO, IS a member"
        }
    );
    println!();

    println!("Explanation:");
    println!("  Under the closed-world assumption, if a fact is not explicitly stated");
    println!("  (or cannot be derived from rules), it is assumed to be FALSE.");
    println!("  Therefore, NOT charlie.is_member succeeds because there's no evidence");
    println!("  that Charlie is a member.");
    println!();

    Ok(())
}

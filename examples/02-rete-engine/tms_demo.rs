/// Demo: Truth Maintenance System (TMS)
///
/// This example demonstrates the Truth Maintenance System features:
/// - Explicit vs Logical assertions
/// - Automatic cascade retraction
/// - Multiple justifications
/// - Dependency tracking
///
/// This demo loads rules from examples/rules/03-advanced/tms_demo.grl

use rust_rule_engine::rete::{
    IncrementalEngine, TypedFacts, FactValue, GrlReteLoader,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 Truth Maintenance System (TMS) Demo");
    println!("========================================\n");

    // Load rules from GRL file
    println!("📁 Loading rules from examples/rules/03-advanced/tms_demo.grl...");
    let mut engine = IncrementalEngine::new();
    GrlReteLoader::load_from_file("examples/rules/03-advanced/tms_demo.grl", &mut engine)?;
    println!("✅ Rules loaded successfully\n");

    // Example 1: Explicit vs Logical Facts
    println!("📋 Example 1: Explicit vs Logical Facts");
    println!("----------------------------------------");
    
    // Explicit fact: User asserted, will NOT be auto-retracted
    let mut customer = TypedFacts::new();
    customer.set("name", FactValue::String("Alice".to_string()));
    customer.set("totalSpent", FactValue::Float(15000.0));
    customer.set("yearsActive", FactValue::Integer(6));
    
    let customer_handle = engine.insert("Customer".to_string(), customer);
    println!("✅ Inserted explicit fact: Customer (handle: {})", customer_handle);
    println!("   - Type: {}", if engine.tms().is_explicit(customer_handle) { "Explicit" } else { "Logical" });
    
    // Fire rules - InferPremiumTier should trigger
    println!("\n🔥 Firing rules...");
    engine.reset();
    let fired = engine.fire_all();
    println!("   Rules fired: {:?}", fired);
    
    // Check for derived CustomerTier fact
    let all_facts = engine.working_memory().get_all_facts();
    let tier_facts: Vec<_> = all_facts.iter()
        .filter(|f| f.fact_type == "CustomerTier")
        .collect();
    
    if let Some(tier_fact) = tier_facts.first() {
        let tier_handle = tier_fact.handle;
        println!("\n✅ Logical fact created by rule: CustomerTier (handle: {})", tier_handle);
        println!("   - Type: {}", if engine.tms().is_logical(tier_handle) { "Logical (derived)" } else { "Explicit" });
        println!("   - Depends on: Customer fact");
        
        let justifications = engine.tms().get_justifications(tier_handle);
        println!("   - Justifications: {}", justifications.len());
        for (i, just) in justifications.iter().enumerate() {
            println!("     {}. Rule: {:?}, Premises: {:?}", 
                i + 1, 
                just.source_rule, 
                just.premise_facts
            );
        }
    }

    // Example 2: Automatic Cascade Retraction
    println!("\n📋 Example 2: Automatic Cascade Retraction");
    println!("-------------------------------------------");
    
    let mut engine2 = IncrementalEngine::new();
    GrlReteLoader::load_from_file("examples/rules/03-advanced/tms_demo.grl", &mut engine2)?;
    
    // Build dependency chain: Order → Discount → Loyalty Points
    let mut order = TypedFacts::new();
    order.set("id", FactValue::String("ORD-001".to_string()));
    order.set("amount", FactValue::Float(5000.0));
    let order_handle = engine2.insert("Order".to_string(), order);
    
    // Manually create logical facts to demonstrate cascade
    // (In real usage, rules would create these automatically)
    let mut discount = TypedFacts::new();
    discount.set("orderId", FactValue::String("ORD-001".to_string()));
    discount.set("amount", FactValue::Float(500.0));
    discount.set("reason", FactValue::String("Bulk order discount".to_string()));
    let discount_handle = engine2.insert_logical(
        "Discount".to_string(),
        discount,
        "ApplyBulkDiscount".to_string(),
        vec![order_handle],
    );
    
    let mut loyalty = TypedFacts::new();
    loyalty.set("orderId", FactValue::String("ORD-001".to_string()));
    loyalty.set("points", FactValue::Integer(50));
    let loyalty_handle = engine2.insert_logical(
        "LoyaltyPoints".to_string(),
        loyalty,
        "AwardDiscountPoints".to_string(),
        vec![discount_handle],
    );
    
    println!("✅ Created dependency chain:");
    println!("   Order (explicit) → Discount (logical) → LoyaltyPoints (logical)");
    println!("   - Order: {}", engine2.tms().has_valid_justification(order_handle));
    println!("   - Discount: {}", engine2.tms().has_valid_justification(discount_handle));
    println!("   - LoyaltyPoints: {}", engine2.tms().has_valid_justification(loyalty_handle));
    
    println!("\n🗑️ Retracting Order...");
    engine2.retract(order_handle)?;
    
    println!("✅ After retraction:");
    println!("   - Order: {}", engine2.tms().has_valid_justification(order_handle));
    println!("   - Discount: {} (cascade retracted!)", engine2.tms().has_valid_justification(discount_handle));
    println!("   - LoyaltyPoints: {} (cascade retracted!)", engine2.tms().has_valid_justification(loyalty_handle));

    // Example 3: Multiple Justifications
    println!("\n📋 Example 3: Multiple Justifications");
    println!("--------------------------------------");
    
    let mut engine3 = IncrementalEngine::new();
    GrlReteLoader::load_from_file("examples/rules/03-advanced/tms_demo.grl", &mut engine3)?;
    
    // Two different rules can derive the same fact
    let mut condition1 = TypedFacts::new();
    condition1.set("type", FactValue::String("high_value".to_string()));
    let cond1_handle = engine3.insert("Condition".to_string(), condition1);
    
    let mut condition2 = TypedFacts::new();
    condition2.set("type", FactValue::String("long_term".to_string()));
    let cond2_handle = engine3.insert("Condition".to_string(), condition2);
    
    // Premium status derived from condition 1
    let mut premium = TypedFacts::new();
    premium.set("status", FactValue::String("Premium".to_string()));
    let premium_handle = engine3.insert_logical(
        "Status".to_string(),
        premium,
        "HighValueRule".to_string(),
        vec![cond1_handle],
    );
    
    // Add second justification (same fact, different rule)
    engine3.tms_mut().add_logical_justification(
        premium_handle,
        "LongTermRule".to_string(),
        vec![cond2_handle],
    );
    
    let justs = engine3.tms().get_justifications(premium_handle);
    println!("✅ Premium status has {} justifications:", justs.len());
    for (i, just) in justs.iter().enumerate() {
        println!("   {}. {:?} (premises: {:?})", i + 1, just.source_rule, just.premise_facts);
    }
    
    println!("\n🗑️ Retracting first condition...");
    engine3.retract(cond1_handle)?;
    
    println!("   - Premium status still valid? {} (has other justification!)", 
        engine3.tms().has_valid_justification(premium_handle));
    
    println!("\n🗑️ Retracting second condition...");
    engine3.retract(cond2_handle)?;
    
    println!("   - Premium status still valid? {} (no justifications left)", 
        engine3.tms().has_valid_justification(premium_handle));

    // Example 4: Diamond Dependency
    println!("\n📋 Example 4: Diamond Dependency Pattern");
    println!("-----------------------------------------");
    
    let mut engine4 = IncrementalEngine::new();
    GrlReteLoader::load_from_file("examples/rules/03-advanced/tms_demo.grl", &mut engine4)?;
    
    println!("Creating diamond pattern:");
    println!("         Root");
    println!("        /    \\");
    println!("    Left      Right");
    println!("        \\    /");
    println!("         Leaf");
    
    let mut root = TypedFacts::new();
    root.set("value", FactValue::Integer(100));
    let root_handle = engine4.insert("Root".to_string(), root);
    
    let mut left = TypedFacts::new();
    left.set("branch", FactValue::String("left".to_string()));
    let left_handle = engine4.insert_logical(
        "Branch".to_string(),
        left,
        "LeftBranch".to_string(),
        vec![root_handle],
    );
    
    let mut right = TypedFacts::new();
    right.set("branch", FactValue::String("right".to_string()));
    let right_handle = engine4.insert_logical(
        "Branch".to_string(),
        right,
        "RightBranch".to_string(),
        vec![root_handle],
    );
    
    let mut leaf = TypedFacts::new();
    leaf.set("result", FactValue::Boolean(true));
    let leaf_handle = engine4.insert_logical(
        "Leaf".to_string(),
        leaf,
        "MergeBranches".to_string(),
        vec![left_handle, right_handle],
    );
    
    println!("\n✅ All facts valid:");
    println!("   - Root: {}", engine4.tms().has_valid_justification(root_handle));
    println!("   - Left: {}", engine4.tms().has_valid_justification(left_handle));
    println!("   - Right: {}", engine4.tms().has_valid_justification(right_handle));
    println!("   - Leaf: {}", engine4.tms().has_valid_justification(leaf_handle));
    
    println!("\n🗑️ Retracting root...");
    engine4.retract(root_handle)?;
    
    println!("✅ After cascade:");
    println!("   - Root: {}", engine4.tms().has_valid_justification(root_handle));
    println!("   - Left: {} (cascade)", engine4.tms().has_valid_justification(left_handle));
    println!("   - Right: {} (cascade)", engine4.tms().has_valid_justification(right_handle));
    println!("   - Leaf: {} (cascade)", engine4.tms().has_valid_justification(leaf_handle));

    // Example 5: TMS Statistics
    println!("\n📋 Example 5: TMS Statistics");
    println!("-----------------------------");
    
    let mut engine5 = IncrementalEngine::new();
    GrlReteLoader::load_from_file("examples/rules/03-advanced/tms_demo.grl", &mut engine5)?;
    
    // Add various facts
    for i in 0..5 {
        let mut fact = TypedFacts::new();
        fact.set("id", FactValue::Integer(i));
        engine5.insert("Explicit".to_string(), fact);
    }
    
    let mut base = TypedFacts::new();
    base.set("base", FactValue::Boolean(true));
    let base_handle = engine5.insert("Base".to_string(), base);
    
    for i in 0..3 {
        let mut fact = TypedFacts::new();
        fact.set("id", FactValue::Integer(i));
        engine5.insert_logical(
            "Derived".to_string(),
            fact,
            format!("Rule{}", i),
            vec![base_handle],
        );
    }
    
    let stats = engine5.tms().stats();
    println!("{}", stats);
    println!("   - Explicit facts: {}", stats.explicit_facts);
    println!("   - Logical facts: {}", stats.logical_facts);
    println!("   - Total justifications: {}", stats.total_justifications);

    // Summary
    println!("\n✨ TMS Features Summary");
    println!("=======================");
    println!("✅ Explicit assertions - User facts that won't auto-retract");
    println!("✅ Logical assertions - Rule-derived facts that auto-retract");
    println!("✅ Cascade retraction - Automatic cleanup of dependent facts");
    println!("✅ Multiple justifications - Facts supported by multiple rules");
    println!("✅ Dependency tracking - Full support chain visibility");
    println!("✅ Statistics - Monitor TMS state");
    
    println!("\n🎯 Use Cases:");
    println!("   • Auto-cleanup derived data when source changes");
    println!("   • Maintain consistency in complex rule systems");
    println!("   • Track why facts exist (debugging)");
    println!("   • Implement belief revision systems");
    println!("   • Build expert systems with truth maintenance");

    Ok(())
}

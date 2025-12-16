//! Multiple Solutions Demo
//!
//! This example demonstrates the backward chaining engine's ability to find
//! multiple solutions (proof paths) for a single goal.
//!
//! Run with:
//! ```
//! cargo run --example multiple_solutions_demo --features backward-chaining
//! ```

use rust_rule_engine::backward::{BackwardConfig, BackwardEngine};
use rust_rule_engine::types::Value;
use rust_rule_engine::{Facts, KnowledgeBase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multiple Solutions Demo ===\n");

    // Scenario: A user can get a discount through MULTIPLE different paths
    // Path 1: VIP status (1000+ points)
    // Path 2: Birthday this month
    // Path 3: First-time buyer
    // Path 4: Referral code

    demo_multiple_discount_paths()?;
    println!("\n{}\n", "=".repeat(60));
    demo_multiple_access_paths()?;

    Ok(())
}

/// Demo 1: Multiple ways to get a discount
fn demo_multiple_discount_paths() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 1: Multiple Discount Qualification Paths\n");

    let kb = KnowledgeBase::new("discount_system");

    // Load discount rules from GRL file
    let rules_grl = include_str!("../rules/09-backward-chaining/discount_rules.grl");
    kb.add_rules_from_grl(rules_grl)?;

    println!(
        "📋 Loaded {} discount qualification rules:",
        kb.get_rules().len()
    );
    for rule in kb.get_rules() {
        println!("   • {}", rule.name);
    }
    println!();

    // Test scenario: user who qualifies through MULTIPLE paths
    println!("User Profile:");
    println!("  Points: 1500 (VIP)");
    println!("  Birthday Month: Yes");
    println!("  First Time: No");
    println!("  Referral Code: Yes\n");

    // Find ONE solution (default)
    println!("Finding 1 solution (default):");
    let config1 = BackwardConfig {
        max_solutions: 1,
        enable_memoization: false, // Disable to see all paths
        ..Default::default()
    };

    let mut engine1 = BackwardEngine::with_config(kb.clone(), config1);
    let mut facts1 = Facts::new();
    facts1.set("User.Points", Value::Number(1500.0));
    facts1.set("User.IsBirthdayMonth", Value::Boolean(true));
    facts1.set("User.IsFirstTime", Value::Boolean(false));
    facts1.set("User.HasReferralCode", Value::Boolean(true));
    let result1 = engine1.query("User.HasDiscount == true", &mut facts1)?;

    println!("  ✅ Goal provable: {}", result1.provable);
    println!("  Solutions found: {}", result1.solutions.len());
    if !result1.solutions.is_empty() {
        println!("  Path: {:?}", result1.solutions[0].path);
    }

    // Find ALL solutions (max 10)
    println!("\nFinding up to 10 solutions:");
    let config_all = BackwardConfig {
        max_solutions: 10,
        enable_memoization: false, // Must disable to find all paths
        ..Default::default()
    };

    let mut engine_all = BackwardEngine::with_config(kb, config_all);
    let mut facts_all = Facts::new();
    facts_all.set("User.Points", Value::Number(1500.0));
    facts_all.set("User.IsBirthdayMonth", Value::Boolean(true));
    facts_all.set("User.IsFirstTime", Value::Boolean(false));
    facts_all.set("User.HasReferralCode", Value::Boolean(true));
    let result_all = engine_all.query("User.HasDiscount == true", &mut facts_all)?;

    println!("  ✅ Goal provable: {}", result_all.provable);
    println!("  Solutions found: {} 🎉", result_all.solutions.len());
    println!("\n  All qualifying paths:");
    for (i, solution) in result_all.solutions.iter().enumerate() {
        println!("    {}. {:?}", i + 1, solution.path);
    }

    println!(
        "\n💡 This user qualifies for discount through {} different paths!",
        result_all.solutions.len()
    );

    Ok(())
}

/// Demo 2: Multiple ways to access a resource
fn demo_multiple_access_paths() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Demo 2: Multiple Resource Access Paths\n");

    let kb = KnowledgeBase::new("access_system");

    // Load access rules from GRL file
    let rules_grl = include_str!("../rules/09-backward-chaining/access_rules.grl");
    kb.add_rules_from_grl(rules_grl)?;

    println!("📋 Loaded {} access control rules:", kb.get_rules().len());
    for rule in kb.get_rules() {
        println!("   • {}", rule.name);
    }
    println!();

    // Test with a user who has multiple access rights
    let mut facts = Facts::new();
    facts.set("User.Role", Value::String("Admin".to_string())); // ✅ Admin (Path 1)
    facts.set("User.IsOwner", Value::Boolean(true)); // ✅ Owner (Path 2)
    facts.set("User.IsCollaborator", Value::Boolean(false)); // ❌ Not collaborator
    facts.set("Resource.IsPublic", Value::Boolean(false)); // ❌ Private resource

    println!("Access Check:");
    println!("  User Role: Admin");
    println!("  Is Owner: Yes");
    println!("  Is Collaborator: No");
    println!("  Resource Public: No\n");

    // Find all possible access paths
    let config = BackwardConfig {
        max_solutions: 10,
        enable_memoization: false,
        ..Default::default()
    };

    let mut engine = BackwardEngine::with_config(kb, config);
    let result = engine.query("User.CanAccessResource == true", &mut facts)?;

    println!("  ✅ Access granted: {}", result.provable);
    println!("  Access paths found: {}", result.solutions.len());
    println!("\n  User can access resource via:");
    for (i, solution) in result.solutions.iter().enumerate() {
        let path_name = &solution.path[0];
        let reason = match path_name.as_str() {
            "AdminAccess" => "Administrator privileges",
            "OwnerAccess" => "Resource ownership",
            "CollaboratorAccess" => "Collaboration permissions",
            "PublicAccess" => "Public resource",
            _ => "Unknown",
        };
        println!("    {}. {} ({})", i + 1, reason, path_name);
    }

    println!("\n💡 Having multiple access paths increases system resilience!");

    Ok(())
}

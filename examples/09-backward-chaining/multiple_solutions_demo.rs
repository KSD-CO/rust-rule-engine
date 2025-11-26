//! Multiple Solutions Demo
//!
//! This example demonstrates the backward chaining engine's ability to find
//! multiple solutions (proof paths) for a single goal.
//!
//! Run with:
//! ```
//! cargo run --example multiple_solutions_demo --features backward-chaining
//! ```

use rust_rule_engine::backward::{BackwardEngine, BackwardConfig};
use rust_rule_engine::backward::search::SearchStrategy;
use rust_rule_engine::{KnowledgeBase, Facts, Rule, Condition, ConditionGroup};
use rust_rule_engine::types::{Value, ActionType, Operator};

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

    let mut kb = KnowledgeBase::new("discount_system");

    // Path 1: VIP members (1000+ points) get discount
    kb.add_rule(Rule::new(
        "VIPDiscount".to_string(),
        ConditionGroup::single(Condition::new(
            "User.Points".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Number(1000.0),
        )),
        vec![ActionType::Set {
            field: "User.HasDiscount".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // Path 2: Birthday this month gets discount
    kb.add_rule(Rule::new(
        "BirthdayDiscount".to_string(),
        ConditionGroup::single(Condition::new(
            "User.IsBirthdayMonth".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "User.HasDiscount".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // Path 3: First-time buyers get discount
    kb.add_rule(Rule::new(
        "FirstTimerDiscount".to_string(),
        ConditionGroup::single(Condition::new(
            "User.IsFirstTime".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "User.HasDiscount".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // Path 4: Referral code gets discount
    kb.add_rule(Rule::new(
        "ReferralDiscount".to_string(),
        ConditionGroup::single(Condition::new(
            "User.HasReferralCode".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "User.HasDiscount".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // Test with a user who qualifies through MULTIPLE paths
    let mut facts = Facts::new();
    facts.set("User.Points", Value::Number(1500.0));           // ✅ VIP (Path 1)
    facts.set("User.IsBirthdayMonth", Value::Boolean(true));   // ✅ Birthday (Path 2)
    facts.set("User.IsFirstTime", Value::Boolean(false));      // ❌ Not first time
    facts.set("User.HasReferralCode", Value::Boolean(true));   // ✅ Referral (Path 3)

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
    let mut facts1 = facts.clone();
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
    let mut facts_all = facts.clone();
    let result_all = engine_all.query("User.HasDiscount == true", &mut facts_all)?;

    println!("  ✅ Goal provable: {}", result_all.provable);
    println!("  Solutions found: {} 🎉", result_all.solutions.len());
    println!("\n  All qualifying paths:");
    for (i, solution) in result_all.solutions.iter().enumerate() {
        println!("    {}. {:?}", i + 1, solution.path);
    }

    println!("\n💡 This user qualifies for discount through {} different paths!",
             result_all.solutions.len());

    Ok(())
}

/// Demo 2: Multiple ways to access a resource
fn demo_multiple_access_paths() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Demo 2: Multiple Resource Access Paths\n");

    let mut kb = KnowledgeBase::new("access_system");

    // Path 1: Admin role
    kb.add_rule(Rule::new(
        "AdminAccess".to_string(),
        ConditionGroup::single(Condition::new(
            "User.Role".to_string(),
            Operator::Equal,
            Value::String("Admin".to_string()),
        )),
        vec![ActionType::Set {
            field: "User.CanAccessResource".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // Path 2: Owner of resource
    kb.add_rule(Rule::new(
        "OwnerAccess".to_string(),
        ConditionGroup::single(Condition::new(
            "User.IsOwner".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "User.CanAccessResource".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // Path 3: Collaborator with permissions
    kb.add_rule(Rule::new(
        "CollaboratorAccess".to_string(),
        ConditionGroup::single(Condition::new(
            "User.IsCollaborator".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "User.CanAccessResource".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // Path 4: Public resource
    kb.add_rule(Rule::new(
        "PublicAccess".to_string(),
        ConditionGroup::single(Condition::new(
            "Resource.IsPublic".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "User.CanAccessResource".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // Test with a user who has multiple access rights
    let mut facts = Facts::new();
    facts.set("User.Role", Value::String("Admin".to_string()));  // ✅ Admin (Path 1)
    facts.set("User.IsOwner", Value::Boolean(true));             // ✅ Owner (Path 2)
    facts.set("User.IsCollaborator", Value::Boolean(false));     // ❌ Not collaborator
    facts.set("Resource.IsPublic", Value::Boolean(false));       // ❌ Private resource

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

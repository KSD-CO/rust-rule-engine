//! Disjunction (OR) Demo for Backward Chaining
//!
//! This example demonstrates OR patterns in backward chaining queries.
//! Shows how multiple rules can lead to the same conclusion (implicit OR).

use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::types::Value;
use rust_rule_engine::backward::{BackwardEngine, Disjunction, DisjunctionParser, DisjunctionResult, Goal};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n🔀 Backward Chaining Disjunction (OR) Demo");
    println!("{}", "=".repeat(80));

    // Demo 1: OR pattern parsing
    demo_1_or_parsing()?;

    // Demo 2: Disjunction data structures
    demo_2_disjunction_structures()?;

    // Demo 3: Implicit OR through multiple rules (from GRL file)
    demo_3_implicit_or_from_grl()?;

    // Demo 4: Result merging and deduplication
    demo_4_result_merging()?;

    println!("\n{}", "=".repeat(80));
    println!("✅ All disjunction demos completed!");
    println!("{}", "=".repeat(80));

    Ok(())
}

/// Demo 1: OR Pattern Parsing
fn demo_1_or_parsing() -> Result<(), Box<dyn Error>> {
    println!("\n📝 Demo 1: OR Pattern Parsing");
    println!("{}", "-".repeat(80));

    // Test different OR patterns
    let patterns = vec![
        ("(manager(?person) OR senior(?person))", "Simple OR"),
        ("(A OR B OR C)", "Triple OR"),
        ("(vip(?x) OR total_spent(?x, ?amt) > 10000)", "Complex OR"),
        ("manager(?person)", "No OR (should fail)"),
    ];

    for (pattern, description) in patterns {
        println!("\nPattern: {}", pattern);
        println!("Description: {}", description);

        match DisjunctionParser::parse(pattern) {
            Some(disj) => {
                println!("✅ Parsed as disjunction");
                println!("   Branches: {}", disj.branch_count());
                for (i, branch) in disj.branches.iter().enumerate() {
                    println!("   Branch {}: {}", i + 1, branch.pattern);
                }
            }
            None => {
                println!("❌ Not a disjunction pattern");
            }
        }
    }

    Ok(())
}

/// Demo 2: Disjunction Data Structures
fn demo_2_disjunction_structures() -> Result<(), Box<dyn Error>> {
    println!("\n🏗️  Demo 2: Disjunction Data Structures");
    println!("{}", "-".repeat(80));

    // Create goals
    let goal1 = Goal::new("manager(?person)".to_string());
    let goal2 = Goal::new("senior(?person)".to_string());
    let goal3 = Goal::new("director(?person)".to_string());

    println!("\n1️⃣ Creating disjunction from two goals:");
    let mut disj = Disjunction::from_pair(goal1.clone(), goal2.clone());
    println!("   Pattern: {}", disj.pattern);
    println!("   Branches: {}", disj.branch_count());

    println!("\n2️⃣ Adding a third branch:");
    disj.add_branch(goal3.clone());
    println!("   Branches: {}", disj.branch_count());

    println!("\n3️⃣ Creating from vector:");
    let goals = vec![
        Goal::new("condition_a".to_string()),
        Goal::new("condition_b".to_string()),
        Goal::new("condition_c".to_string()),
    ];
    let disj2 = Disjunction::new(goals, "(A OR B OR C)".to_string());
    println!("   Branches: {}", disj2.branch_count());

    Ok(())
}

/// Demo 3: Implicit OR through Multiple Rules (from GRL file)
fn demo_3_implicit_or_from_grl() -> Result<(), Box<dyn Error>> {
    println!("\n🎯 Demo 3: Implicit OR through Multiple Rules");
    println!("{}", "-".repeat(80));

    // Load rules from GRL file
    let grl_content = include_str!("disjunction_rules.grl");
    let mut kb = KnowledgeBase::new("disjunction_demo");
    kb.add_rules_from_grl(grl_content)?;

    println!("\n📋 Loaded rules from disjunction_rules.grl");
    println!("   Total rules: {}", kb.get_rules().len());

    // Demo 3.1: Employee eligibility (manager OR senior OR director)
    println!("\n1️⃣ Test: Employee Eligibility (Manager OR Senior OR Director)");
    println!("{}", "-".repeat(60));

    // Test case: Manager
    let mut facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(true));
    facts.set("Employee.IsSenior", Value::Boolean(false));
    facts.set("Employee.IsDirector", Value::Boolean(false));

    let mut engine = BackwardEngine::new(kb.clone());
    let result = engine.query("Employee.IsEligible == true", &mut facts)?;

    println!("   Scenario: Manager");
    println!("   Result: {}", if result.provable { "✅ ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    // Test case: Senior
    facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(false));
    facts.set("Employee.IsSenior", Value::Boolean(true));
    facts.set("Employee.IsDirector", Value::Boolean(false));

    let result = engine.query("Employee.IsEligible == true", &mut facts)?;
    println!("   Scenario: Senior");
    println!("   Result: {}", if result.provable { "✅ ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    // Test case: Director
    facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(false));
    facts.set("Employee.IsSenior", Value::Boolean(false));
    facts.set("Employee.IsDirector", Value::Boolean(true));

    let result = engine.query("Employee.IsEligible == true", &mut facts)?;
    println!("   Scenario: Director");
    println!("   Result: {}", if result.provable { "✅ ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    // Test case: None (should fail)
    facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(false));
    facts.set("Employee.IsSenior", Value::Boolean(false));
    facts.set("Employee.IsDirector", Value::Boolean(false));

    let result = engine.query("Employee.IsEligible == true", &mut facts)?;
    println!("   Scenario: Regular employee");
    println!("   Result: {}", if result.provable { "✅ ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    // Demo 3.2: Discount eligibility (VIP OR high spender)
    println!("\n2️⃣ Test: Discount Eligibility (VIP OR High Spender)");
    println!("{}", "-".repeat(60));

    // Test case: VIP
    facts = Facts::new();
    facts.set("Customer.IsVIP", Value::Boolean(true));
    facts.set("Customer.TotalSpent", Value::Number(5000.0));

    let result = engine.query("Customer.GetsDiscount == true", &mut facts)?;
    println!("   Scenario: VIP with $5,000 spent");
    println!("   Result: {}", if result.provable { "✅ DISCOUNT" } else { "❌ NO DISCOUNT" });

    // Test case: High spender (not VIP)
    facts = Facts::new();
    facts.set("Customer.IsVIP", Value::Boolean(false));
    facts.set("Customer.TotalSpent", Value::Number(15000.0));

    let result = engine.query("Customer.GetsDiscount == true", &mut facts)?;
    println!("   Scenario: Non-VIP with $15,000 spent");
    println!("   Result: {}", if result.provable { "✅ DISCOUNT" } else { "❌ NO DISCOUNT" });

    // Test case: Neither
    facts = Facts::new();
    facts.set("Customer.IsVIP", Value::Boolean(false));
    facts.set("Customer.TotalSpent", Value::Number(5000.0));

    let result = engine.query("Customer.GetsDiscount == true", &mut facts)?;
    println!("   Scenario: Non-VIP with $5,000 spent");
    println!("   Result: {}", if result.provable { "✅ DISCOUNT" } else { "❌ NO DISCOUNT" });

    // Demo 3.3: Access control (Employee OR Contractor with badge)
    println!("\n3️⃣ Test: Access Control (Employee OR Contractor with Active Badge)");
    println!("{}", "-".repeat(60));

    // Test case: Employee with badge
    facts = Facts::new();
    facts.set("Person.IsEmployee", Value::Boolean(true));
    facts.set("Person.IsContractor", Value::Boolean(false));
    facts.set("Person.HasActiveBadge", Value::Boolean(true));

    let result = engine.query("Person.HasAccess == true", &mut facts)?;
    println!("   Scenario: Employee with active badge");
    println!("   Result: {}", if result.provable { "✅ ACCESS GRANTED" } else { "❌ ACCESS DENIED" });

    // Test case: Contractor with badge
    facts = Facts::new();
    facts.set("Person.IsEmployee", Value::Boolean(false));
    facts.set("Person.IsContractor", Value::Boolean(true));
    facts.set("Person.HasActiveBadge", Value::Boolean(true));

    let result = engine.query("Person.HasAccess == true", &mut facts)?;
    println!("   Scenario: Contractor with active badge");
    println!("   Result: {}", if result.provable { "✅ ACCESS GRANTED" } else { "❌ ACCESS DENIED" });

    // Test case: Employee without badge
    facts = Facts::new();
    facts.set("Person.IsEmployee", Value::Boolean(true));
    facts.set("Person.IsContractor", Value::Boolean(false));
    facts.set("Person.HasActiveBadge", Value::Boolean(false));

    let result = engine.query("Person.HasAccess == true", &mut facts)?;
    println!("   Scenario: Employee without badge");
    println!("   Result: {}", if result.provable { "✅ ACCESS GRANTED" } else { "❌ ACCESS DENIED" });

    Ok(())
}

/// Demo 4: Result Merging and Deduplication
fn demo_4_result_merging() -> Result<(), Box<dyn Error>> {
    println!("\n🔗 Demo 4: Result Merging and Deduplication");
    println!("{}", "-".repeat(80));

    let mut result = DisjunctionResult::new();

    println!("\n1️⃣ Adding solutions from multiple branches:");
    result.add_branch_solutions(0, vec![
        rust_rule_engine::backward::Bindings::new(),
        rust_rule_engine::backward::Bindings::new(),
    ]);
    println!("   Branch 0: 2 solutions added");
    println!("   Total: {} solutions", result.solution_count());

    result.add_branch_solutions(2, vec![
        rust_rule_engine::backward::Bindings::new(),
    ]);
    println!("   Branch 2: 1 solution added");
    println!("   Total: {} solutions", result.solution_count());
    println!("   Successful branches: {:?}", result.successful_branches);

    println!("\n2️⃣ Testing deduplication:");
    let mut dup_result = DisjunctionResult::new();
    let dup_bindings = rust_rule_engine::backward::Bindings::new();
    dup_result.add_branch_solutions(0, vec![
        dup_bindings.clone(),
        dup_bindings.clone(),
        dup_bindings,
    ]);

    println!("   Before: {} solutions", dup_result.solution_count());
    dup_result.deduplicate();
    println!("   After: {} solutions", dup_result.solution_count());

    Ok(())
}

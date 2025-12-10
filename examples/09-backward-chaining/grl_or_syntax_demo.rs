//! GRL OR Syntax Demo for Backward Chaining
//!
//! This example demonstrates explicit OR syntax in GRL query goals.
//! Shows how to use `||` operator directly in query goals instead of implicit OR through multiple rules.

use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::types::Value;
use rust_rule_engine::backward::{BackwardEngine, GRLQueryParser, GRLQueryExecutor};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n🔀 GRL OR Syntax Demo for Backward Chaining");
    println!("{}", "=".repeat(80));

    // Demo 1: Simple OR in query goal
    demo_1_simple_or()?;

    // Demo 2: Multiple OR branches
    demo_2_multiple_or()?;

    // Demo 3: Complex AND + OR combination
    demo_3_complex_and_or()?;

    println!("\n{}", "=".repeat(80));
    println!("✅ All GRL OR syntax demos completed!");
    println!("{}", "=".repeat(80));

    Ok(())
}

/// Demo 1: Simple OR in Query Goal
fn demo_1_simple_or() -> Result<(), Box<dyn Error>> {
    println!("\n📝 Demo 1: Simple OR in Query Goal");
    println!("{}", "-".repeat(80));

    // Load rules
    let grl_content = r#"
        rule "ManagerEligible" {
            when
                Employee.IsManager == true
            then
                Employee.IsEligible = true;
        }

        rule "SeniorEligible" {
            when
                Employee.IsSenior == true
            then
                Employee.IsEligible = true;
        }

        query "CheckEligibility" {
            goal: Employee.IsManager == true || Employee.IsSenior == true
            on-success: {
                LogMessage("Employee is eligible");
            }
        }
    "#;

    let mut kb = KnowledgeBase::new("demo1");
    kb.add_rules_from_grl(grl_content)?;

    // Parse query
    let queries = GRLQueryParser::parse_queries(grl_content)?;
    let query = &queries[0];

    println!("Query: {}", query.name);
    println!("Goal: {}", query.goal);

    // Test case 1: Manager (first branch succeeds)
    println!("\n1️⃣ Test: Manager (first OR branch)");
    let mut facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(true));
    facts.set("Employee.IsSenior", Value::Boolean(false));

    let mut engine = BackwardEngine::new(kb.clone());
    let result = GRLQueryExecutor::execute(&query, &mut engine, &mut facts)?;
    println!("   Result: {}", if result.provable { "✅ ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    // Test case 2: Senior (second branch succeeds)
    println!("\n2️⃣ Test: Senior (second OR branch)");
    let mut facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(false));
    facts.set("Employee.IsSenior", Value::Boolean(true));

    let result = GRLQueryExecutor::execute(&query, &mut engine, &mut facts)?;
    println!("   Result: {}", if result.provable { "✅ ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    // Test case 3: Neither (both branches fail)
    println!("\n3️⃣ Test: Regular employee (no OR branch succeeds)");
    let mut facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(false));
    facts.set("Employee.IsSenior", Value::Boolean(false));

    let result = GRLQueryExecutor::execute(&query, &mut engine, &mut facts)?;
    println!("   Result: {}", if result.provable { "✅ ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    Ok(())
}

/// Demo 2: Multiple OR Branches
fn demo_2_multiple_or() -> Result<(), Box<dyn Error>> {
    println!("\n📝 Demo 2: Multiple OR Branches");
    println!("{}", "-".repeat(80));

    let grl_content = r#"
        rule "VIPDiscount" {
            when
                Customer.IsVIP == true
            then
                Customer.GetsDiscount = true;
        }

        rule "HighSpenderDiscount" {
            when
                Customer.TotalSpent > 10000
            then
                Customer.GetsDiscount = true;
        }

        rule "LoyaltyDiscount" {
            when
                Customer.LoyaltyYears > 5
            then
                Customer.GetsDiscount = true;
        }

        query "CheckDiscount" {
            goal: Customer.IsVIP == true || Customer.TotalSpent > 10000 || Customer.LoyaltyYears > 5
            on-success: {
                Customer.DiscountRate = 0.15;
            }
        }
    "#;

    let mut kb = KnowledgeBase::new("demo2");
    kb.add_rules_from_grl(grl_content)?;

    let queries = GRLQueryParser::parse_queries(grl_content)?;
    let query = &queries[0];

    println!("Query goal has 3 OR branches:");
    let branches: Vec<&str> = query.goal.split("||").collect();
    for (i, branch) in branches.iter().enumerate() {
        println!("   Branch {}: {}", i + 1, branch.trim());
    }

    // Test: High spender (middle branch)
    println!("\n1️⃣ Test: High spender (middle branch succeeds)");
    let mut facts = Facts::new();
    facts.set("Customer.IsVIP", Value::Boolean(false));
    facts.set("Customer.TotalSpent", Value::Number(15000.0));
    facts.set("Customer.LoyaltyYears", Value::Number(2.0));

    let mut engine = BackwardEngine::new(kb.clone());
    let result = GRLQueryExecutor::execute(&query, &mut engine, &mut facts)?;
    println!("   Result: {}", if result.provable { "✅ DISCOUNT" } else { "❌ NO DISCOUNT" });

    // Test: Loyalty member (last branch)
    println!("\n2️⃣ Test: Loyalty member (last branch succeeds)");
    let mut facts = Facts::new();
    facts.set("Customer.IsVIP", Value::Boolean(false));
    facts.set("Customer.TotalSpent", Value::Number(5000.0));
    facts.set("Customer.LoyaltyYears", Value::Number(7.0));

    let result = GRLQueryExecutor::execute(&query, &mut engine, &mut facts)?;
    println!("   Result: {}", if result.provable { "✅ DISCOUNT" } else { "❌ NO DISCOUNT" });

    Ok(())
}

/// Demo 3: Complex AND + OR Combination
fn demo_3_complex_and_or() -> Result<(), Box<dyn Error>> {
    println!("\n📝 Demo 3: Complex AND + OR Combination");
    println!("{}", "-".repeat(80));

    let grl_content = r#"
        rule "ManagerActiveBonus" {
            when
                Employee.IsManager == true &&
                Employee.Active == true
            then
                Employee.BonusEligible = true;
        }

        rule "SeniorExperienceBonus" {
            when
                Employee.IsSenior == true &&
                Employee.YearsExperience > 5
            then
                Employee.BonusEligible = true;
        }

        query "CheckBonus" {
            goal: Employee.IsManager == true && Employee.Active == true || Employee.IsSenior == true && Employee.YearsExperience > 5
            on-success: {
                Employee.BonusAmount = 5000;
            }
        }
    "#;

    let mut kb = KnowledgeBase::new("demo3");
    kb.add_rules_from_grl(grl_content)?;

    let queries = GRLQueryParser::parse_queries(grl_content)?;
    let query = &queries[0];

    println!("Complex goal with operator precedence: A && B || C && D");
    println!("Evaluates as: (A && B) || (C && D) due to AND precedence");
    println!("Goal: {}", query.goal);

    // Test case 1: Active manager (first AND succeeds)
    println!("\n1️⃣ Test: Active manager (first AND group succeeds)");
    let mut facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(true));
    facts.set("Employee.Active", Value::Boolean(true));
    facts.set("Employee.IsSenior", Value::Boolean(false));
    facts.set("Employee.YearsExperience", Value::Number(2.0));

    let mut engine = BackwardEngine::new(kb.clone());
    let result = GRLQueryExecutor::execute(&query, &mut engine, &mut facts)?;
    println!("   Result: {}", if result.provable { "✅ BONUS ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    // Test case 2: Experienced senior (second AND succeeds)
    println!("\n2️⃣ Test: Experienced senior (second AND group succeeds)");
    let mut facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(false));
    facts.set("Employee.Active", Value::Boolean(true));
    facts.set("Employee.IsSenior", Value::Boolean(true));
    facts.set("Employee.YearsExperience", Value::Number(8.0));

    let result = GRLQueryExecutor::execute(&query, &mut engine, &mut facts)?;
    println!("   Result: {}", if result.provable { "✅ BONUS ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    // Test case 3: Inactive manager (first AND fails, second AND fails)
    println!("\n3️⃣ Test: Inactive manager (both AND groups fail)");
    let mut facts = Facts::new();
    facts.set("Employee.IsManager", Value::Boolean(true));
    facts.set("Employee.Active", Value::Boolean(false));
    facts.set("Employee.IsSenior", Value::Boolean(false));
    facts.set("Employee.YearsExperience", Value::Number(2.0));

    let result = GRLQueryExecutor::execute(&query, &mut engine, &mut facts)?;
    println!("   Result: {}", if result.provable { "✅ BONUS ELIGIBLE" } else { "❌ NOT ELIGIBLE" });

    Ok(())
}

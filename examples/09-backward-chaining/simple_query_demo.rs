//! Simple Backward Chaining Query Demo
//!
//! Demonstrates basic backward chaining queries:
//! - Creating goals
//! - Querying if goals can be proven
//! - Viewing proof traces
//!
//! Run with: cargo run --example simple_query_demo --features backward-chaining

#![cfg(feature = "backward-chaining")]

use rust_rule_engine::backward::{BackwardEngine, BackwardConfig};
use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::parser::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Simple Backward Chaining Query Demo");
    println!("=====================================\n");

    demo_basic_query()?;
    demo_proof_trace()?;
    demo_missing_facts()?;
    demo_memoization()?;

    Ok(())
}

fn demo_basic_query() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 1: Basic Query");
    println!("---------------------");

    // Create knowledge base with rules
    let mut kb = KnowledgeBase::new("BasicQuery");
    
    let rules = r#"
    rule "VIPStatus" salience 100 {
        when
            User.Score >= 80
        then
            User.IsVIP = true;
    }
    
    rule "CalculateScore" salience 90 {
        when
            User.SpendingTotal > 1000
        then
            User.Score = 85;
    }
    "#;
    
    kb.add_rules_from_grl(rules)?;

    println!("📋 Rules loaded:");
    for rule in kb.get_rules() {
        println!("   • {}", rule.name);
    }

    // Create backward chaining engine
    let mut bc_engine = BackwardEngine::new(kb);

    // Create facts
    let facts = Facts::new();
    facts.set("User", Value::Object({
        let mut user = HashMap::new();
        user.insert("SpendingTotal".to_string(), Value::Number(1500.0));
        user.insert("Score".to_string(), Value::Number(0.0));
        user.insert("IsVIP".to_string(), Value::Boolean(false));
        user
    }));

    println!("\n💾 Initial Facts:");
    if let Some(user) = facts.get("User") {
        println!("   User: {:?}", user);
    }

    // Query: Can we prove User.IsVIP == true?
    println!("\n🔍 Query: Can 'User.IsVIP == true' be proven?");
    let result = bc_engine.query("User.IsVIP == true", &mut facts)?;

    if result.provable {
        println!("✅ YES! Goal is provable");
        println!("📊 Stats:");
        println!("   • Goals explored: {}", result.stats.goals_explored);
        println!("   • Rules evaluated: {}", result.stats.rules_evaluated);
        println!("   • Max depth: {}", result.stats.max_depth);
    } else {
        println!("❌ NO! Goal is not provable");
        println!("Missing facts: {:?}", result.missing_facts);
    }

    Ok(())
}

fn demo_proof_trace() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📝 Demo 2: Proof Trace");
    println!("---------------------");

    let mut kb = KnowledgeBase::new("ProofTrace");
    
    let rules = r#"
    rule "ApplyDiscount" {
        when
            User.IsVIP == true && Order.Amount > 100
        then
            Order.Discount = 0.2;
    }
    
    rule "VIPCheck" {
        when
            User.LoyaltyPoints > 500
        then
            User.IsVIP = true;
    }
    "#;
    
    kb.add_rules_from_grl(rules)?;

    let mut bc_engine = BackwardEngine::new(kb);
    let facts = Facts::new();

    println!("🔍 Query: 'Order.Discount == 0.2'");
    let result = bc_engine.query("Order.Discount == 0.2", &mut facts)?;

    println!("\n📜 Proof Trace:");
    result.proof_trace.print();

    Ok(())
}

fn demo_missing_facts() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📝 Demo 3: Missing Facts Detection");
    println!("----------------------------------");

    let mut kb = KnowledgeBase::new("MissingFacts");
    
    let rules = r#"
    rule "ApproveCredit" {
        when
            Applicant.CreditScore > 700 && Applicant.Income > 50000
        then
            Application.Approved = true;
    }
    "#;
    
    kb.add_rules_from_grl(rules)?;

    let mut bc_engine = BackwardEngine::new(kb);
    let facts = Facts::new();
    
    // Only set CreditScore, not Income
    facts.set("Applicant", Value::Object({
        let mut applicant = HashMap::new();
        applicant.insert("CreditScore".to_string(), Value::Number(750.0));
        applicant
    }));

    println!("🔍 Query: 'Application.Approved == true'");
    let result = bc_engine.query("Application.Approved == true", &mut facts)?;

    if !result.provable {
        println!("❌ Cannot prove goal");
        println!("\n⚠️  Missing required facts:");
        for fact in &result.missing_facts {
            println!("   • {}", fact);
        }
    }

    Ok(())
}

fn demo_memoization() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📝 Demo 4: Memoization (Caching)");
    println!("--------------------------------");

    let kb = KnowledgeBase::new("Memoization");
    
    let config = BackwardConfig {
        enable_memoization: true,
        max_depth: 10,
        ..Default::default()
    };
    
    let mut bc_engine = BackwardEngine::with_config(kb, config);
    let facts = Facts::new();

    println!("🔍 First query (fresh search):");
    let start = std::time::Instant::now();
    let result1 = bc_engine.query("User.IsVIP == true", &mut facts)?;
    let duration1 = start.elapsed();
    println!("   Time: {:?}", duration1);
    println!("   Goals explored: {}", result1.stats.goals_explored);

    println!("\n🔍 Second query (cached):");
    let start = std::time::Instant::now();
    let result2 = bc_engine.query("User.IsVIP == true", &mut facts)?;
    let duration2 = start.elapsed();
    println!("   Time: {:?}", duration2);
    println!("   Goals explored: {}", result2.stats.goals_explored);

    if duration2 < duration1 {
        println!("\n✅ Caching improved performance!");
    }

    Ok(())
}

/// GRL Query Syntax Demo
///
/// Demonstrates how to define and execute backward chaining queries
/// using GRL (Grule Rule Language) syntax instead of code.
///
/// This shows how queries can be defined in declarative .grl files
/// and loaded at runtime, similar to how forward rules are defined.
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, Value};

#[cfg(feature = "backward-chaining")]
use rust_rule_engine::backward::{BackwardEngine, GRLQueryExecutor, GRLQueryParser};

fn main() {
    #[cfg(not(feature = "backward-chaining"))]
    {
        println!("❌ This example requires the 'backward-chaining' feature");
        println!("   Run with: cargo run --features backward-chaining --example grl_query_demo");
        return;
    }

    #[cfg(feature = "backward-chaining")]
    {
        println!("=== GRL Query Syntax Demo ===\n");

        demo_1_simple_query();
        demo_2_query_with_actions();
        demo_3_medical_diagnosis_grl();
        demo_4_multiple_queries();
    }
}

#[cfg(feature = "backward-chaining")]
fn demo_1_simple_query() {
    println!("📋 Demo 1: Simple GRL Query");
    println!("─────────────────────────────\n");

    // Define rules using GRL
    let grl_rules = r#"
    rule "VIPRule" {
        when
            User.LoyaltyPoints >= 1000
        then
            User.IsVIP = true;
    }
    "#;

    let kb = KnowledgeBase::new("demo1");
    let rules = GRLParser::parse_rules(grl_rules).unwrap();
    for rule in rules {
        kb.add_rule(rule);
    }

    // GRL Query syntax
    let query_str = r#"
    query "CheckVIPStatus" {
        goal: User.IsVIP == true
        strategy: depth-first
        max-depth: 5
    }
    "#;

    // Parse query
    let query = GRLQueryParser::parse(query_str).unwrap();
    println!("✓ Parsed query: {}", query.name);

    // Create facts
    let mut facts = Facts::new();
    facts.set("User.LoyaltyPoints", Value::Number(1200.0));

    // Execute query
    let mut bc_engine = BackwardEngine::new(kb);
    let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts).unwrap();

    println!("Query: {}", query.name);
    println!("Goal: {}", query.goal);
    println!("Provable: {}", result.provable);

    if result.provable {
        println!("✅ VIP status confirmed!");
    }

    println!();
}

#[cfg(feature = "backward-chaining")]
fn demo_2_query_with_actions() {
    println!("📋 Demo 2: Query with On-Success Actions");
    println!("───────────────────────────────────────────\n");

    let grl_rules = r#"
    rule "HighSpenderVIP" {
        when
            Order.Total >= 5000
        then
            Customer.IsVIP = true;
    }
    "#;

    let kb = KnowledgeBase::new("demo2");
    let rules = GRLParser::parse_rules(grl_rules).unwrap();
    for rule in rules {
        kb.add_rule(rule);
    }

    // GRL Query with actions
    let query_str = r#"
    query "CheckVIPWithActions" {
        goal: Customer.IsVIP == true
        strategy: depth-first
        
        on-success: {
            Customer.DiscountRate = 0.2;
            Customer.FreeShipping = true;
            LogMessage("VIP benefits applied");
        }
        
        on-failure: {
            Customer.DiscountRate = 0.0;
            LogMessage("Standard customer");
        }
    }
    "#;

    let query = GRLQueryParser::parse(query_str).unwrap();

    // Customer with high order
    let mut facts = Facts::new();
    facts.set("Order.Total", Value::Number(6000.0));

    let mut bc_engine = BackwardEngine::new(kb);
    let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts).unwrap();

    println!("✅ Query executed: {}", query.name);
    println!("   Provable: {}", result.provable);

    if result.provable {
        println!("   ✓ On-success actions executed");
        if let Some(discount) = facts.get("Customer.DiscountRate") {
            println!("   ✓ Discount rate set: {:?}", discount);
        }
        if let Some(shipping) = facts.get("Customer.FreeShipping") {
            println!("   ✓ Free shipping: {:?}", shipping);
        }
    }

    println!();
}

#[cfg(feature = "backward-chaining")]
fn demo_3_medical_diagnosis_grl() {
    println!("📋 Demo 3: Medical Diagnosis with GRL Queries");
    println!("────────────────────────────────────────────────\n");

    let grl_rules = r#"
    rule "DiagnoseFlu" {
        when
            Patient.HasFever == true &&
            Patient.HasCough == true &&
            Patient.HasFatigue == true
        then
            Diagnosis.Disease = "Influenza";
    }

    rule "FeverFromTemp" {
        when
            Patient.Temperature >= 38.0
        then
            Patient.HasFever = true;
    }

    rule "CoughFromRespiratory" {
        when
            Patient.RespiratorySymptoms == true
        then
            Patient.HasCough = true;
    }
    "#;

    let kb = KnowledgeBase::new("demo3");
    let rules = GRLParser::parse_rules(grl_rules).unwrap();
    for rule in rules {
        kb.add_rule(rule);
    }

    // GRL Query for flu diagnosis
    let query_str = r#"
    query "CheckFluDiagnosis" {
        goal: Diagnosis.Disease == "Influenza"
        strategy: depth-first
        max-depth: 10
        enable-memoization: true
        
        on-success: {
            Treatment.Recommended = "Rest and fluids";
            Diagnosis.Confidence = 0.85;
            LogMessage("Flu diagnosis confirmed");
        }
        
        on-failure: {
            Action.Next = "Consider other diagnoses";
            LogMessage("Flu not confirmed");
        }
        
        on-missing: {
            LogMessage("Missing patient data");
        }
    }
    "#;

    let query = GRLQueryParser::parse(query_str).unwrap();

    // Patient data
    let mut facts = Facts::new();
    facts.set("Patient.Temperature", Value::Number(38.5));
    facts.set("Patient.RespiratorySymptoms", Value::Boolean(true));
    facts.set("Patient.HasFatigue", Value::Boolean(true));

    let mut bc_engine = BackwardEngine::new(kb);
    let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts).unwrap();

    println!("🏥 Medical Query: {}", query.name);
    println!("   Goal: Check for Influenza");
    println!(
        "   Result: {}",
        if result.provable {
            "✅ POSITIVE"
        } else {
            "❌ NEGATIVE"
        }
    );

    if result.provable {
        println!("\n📊 Diagnosis Details:");
        if let Some(treatment) = facts.get("Treatment.Recommended") {
            println!("   Treatment: {:?}", treatment);
        }
        if let Some(confidence) = facts.get("Diagnosis.Confidence") {
            println!("   Confidence: {:?}", confidence);
        }

        println!("\n🔍 Reasoning Chain:");
        println!("   → Patient has fever (Temperature >= 38.0)");
        println!("   → Patient has cough (Respiratory symptoms present)");
        println!("   → Patient has fatigue");
        println!("   → All conditions met → Influenza diagnosis");
    }

    println!();
}

#[cfg(feature = "backward-chaining")]
fn demo_4_multiple_queries() {
    println!("📋 Demo 4: Multiple Queries from GRL File");
    println!("────────────────────────────────────────────\n");

    let grl_rules = r#"
    rule "GoldTierRule" {
        when
            User.Points >= 10000
        then
            User.Tier = "Gold";
    }

    rule "SilverTierRule" {
        when
            User.Points >= 5000
        then
            User.Tier = "Silver";
    }

    rule "VIPFromGold" {
        when
            User.Tier == "Gold"
        then
            User.IsVIP = true;
    }
    "#;

    let kb = KnowledgeBase::new("demo4");
    let rules = GRLParser::parse_rules(grl_rules).unwrap();
    for rule in rules {
        kb.add_rule(rule);
    }

    // Multiple queries in one file
    let queries_str = r#"
    query "CheckGoldTier" {
        goal: User.Tier == "Gold"
        strategy: depth-first
    }
    
    query "CheckVIPStatus" {
        goal: User.IsVIP == true
        strategy: breadth-first
        max-depth: 8
    }
    
    query "CheckSilverTier" {
        goal: User.Tier == "Silver"
        strategy: depth-first
    }
    "#;

    // Parse all queries
    let queries = GRLQueryParser::parse_queries(queries_str).unwrap();
    println!("✓ Parsed {} queries from file", queries.len());

    // User data
    let mut facts = Facts::new();
    facts.set("User.Points", Value::Number(12000.0));

    // Execute all queries
    let mut bc_engine = BackwardEngine::new(kb);
    let results = GRLQueryExecutor::execute_queries(&queries, &mut bc_engine, &mut facts).unwrap();

    println!("\n📊 Query Results:");
    for (query, result) in queries.iter().zip(results.iter()) {
        let status = if result.provable {
            "✅ PASS"
        } else {
            "❌ FAIL"
        };
        println!("   {} - {}: {}", status, query.name, query.goal);
    }

    println!();
}

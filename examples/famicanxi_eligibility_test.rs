use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 FamiCanxi Product Eligibility Rule Test");
    println!("==========================================");

    // Create knowledge base and engine
    let kb = KnowledgeBase::new("FamiCanxiEligibility");

    // Create FamiCanxi Product Eligibility Rule
    // Rule: (L1 > L1Min) && (CM2 > Cm2Min) && (productCode == 1)
    // Action: setLevelApprove(1)
    // Note: Since we can't compare variables directly, we'll use conditions with field paths
    let famicanxi_rule = Rule::new(
        "FamiCanxi Product Eligibility Rule".to_string(),
        ConditionGroup::and(
            ConditionGroup::and(
                ConditionGroup::single(Condition::new(
                    "application.L1GreaterThanMin".to_string(),
                    Operator::Equal,
                    Value::Boolean(true),
                )),
                ConditionGroup::single(Condition::new(
                    "application.CM2GreaterThanMin".to_string(),
                    Operator::Equal,
                    Value::Boolean(true),
                )),
            ),
            ConditionGroup::single(Condition::new(
                "application.productCode".to_string(),
                Operator::Equal,
                Value::Number(1.0),
            )),
        ),
        vec![
            ActionType::Set {
                field: "levelApprove".to_string(),
                value: Value::Number(1.0),
            },
            ActionType::Log {
                message: "FamiCanxi product approved - Level 1".to_string(),
            },
        ],
    )
    .with_salience(50);

    let mut knowledge_base = kb;
    knowledge_base.add_rule(famicanxi_rule)?;
    println!("✅ Created FamiCanxi eligibility rule (salience: 50)\n");

    // Configure engine with max_cycles = 1 to prevent multiple firings
    let config = EngineConfig {
        max_cycles: 1,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(knowledge_base, config);

    // Test different scenarios
    println!("🎭 Testing Scenarios:\n");

    // Scenario 1: Eligible customer (all conditions met)
    println!("📋 Scenario 1: Eligible Customer");
    let facts1 = Facts::new();

    // L1=100 > L1Min=50 ✓
    // CM2=80 > Cm2Min=60 ✓
    // productCode=1 ✓
    let mut app1 = HashMap::new();
    app1.insert("L1GreaterThanMin".to_string(), Value::Boolean(true));
    app1.insert("CM2GreaterThanMin".to_string(), Value::Boolean(true));
    app1.insert("productCode".to_string(), Value::Number(1.0));
    facts1.add_value("application", Value::Object(app1))?;
    facts1.add_value("levelApprove", Value::Number(0.0))?;

    println!("   Input: L1 > L1Min ✓, CM2 > Cm2Min ✓, productCode=1 ✓");
    match engine.execute(&facts1) {
        Ok(result) => {
            println!("   ✅ Rules fired: {}", result.rules_fired);
            println!("   📊 Rules evaluated: {}", result.rules_evaluated);
            if let Some(Value::Number(level)) = facts1.get("levelApprove") {
                println!("   🎯 Level Approved: {}", level);
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Scenario 2: L1 too low
    println!("\n📋 Scenario 2: L1 Below Minimum");
    let facts2 = Facts::new();

    // L1=40 < L1Min=50 ✗
    // CM2=80 > Cm2Min=60 ✓
    // productCode=1 ✓
    let mut app2 = HashMap::new();
    app2.insert("L1GreaterThanMin".to_string(), Value::Boolean(false));
    app2.insert("CM2GreaterThanMin".to_string(), Value::Boolean(true));
    app2.insert("productCode".to_string(), Value::Number(1.0));
    facts2.add_value("application", Value::Object(app2))?;
    facts2.add_value("levelApprove", Value::Number(0.0))?;

    println!("   Input: L1 > L1Min ✗, CM2 > Cm2Min ✓, productCode=1 ✓");
    match engine.execute(&facts2) {
        Ok(result) => {
            println!("   ⚪ Rules fired: {} (expected: 0)", result.rules_fired);
            if let Some(Value::Number(level)) = facts2.get("levelApprove") {
                println!("   🎯 Level Approved: {} (unchanged)", level);
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Scenario 3: CM2 too low
    println!("\n📋 Scenario 3: CM2 Below Minimum");
    let facts3 = Facts::new();

    // L1=100 > L1Min=50 ✓
    // CM2=50 < Cm2Min=60 ✗
    // productCode=1 ✓
    let mut app3 = HashMap::new();
    app3.insert("L1GreaterThanMin".to_string(), Value::Boolean(true));
    app3.insert("CM2GreaterThanMin".to_string(), Value::Boolean(false));
    app3.insert("productCode".to_string(), Value::Number(1.0));
    facts3.add_value("application", Value::Object(app3))?;
    facts3.add_value("levelApprove", Value::Number(0.0))?;

    println!("   Input: L1 > L1Min ✓, CM2 > Cm2Min ✗, productCode=1 ✓");
    match engine.execute(&facts3) {
        Ok(result) => {
            println!("   ⚪ Rules fired: {} (expected: 0)", result.rules_fired);
            if let Some(Value::Number(level)) = facts3.get("levelApprove") {
                println!("   🎯 Level Approved: {} (unchanged)", level);
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Scenario 4: Wrong product code
    println!("\n📋 Scenario 4: Wrong Product Code");
    let facts4 = Facts::new();

    // L1=100 > L1Min=50 ✓
    // CM2=80 > Cm2Min=60 ✓
    // productCode=2 (not 1) ✗
    let mut app4 = HashMap::new();
    app4.insert("L1GreaterThanMin".to_string(), Value::Boolean(true));
    app4.insert("CM2GreaterThanMin".to_string(), Value::Boolean(true));
    app4.insert("productCode".to_string(), Value::Number(2.0));
    facts4.add_value("application", Value::Object(app4))?;
    facts4.add_value("levelApprove", Value::Number(0.0))?;

    println!("   Input: L1 > L1Min ✓, CM2 > Cm2Min ✓, productCode=2 ✗");
    match engine.execute(&facts4) {
        Ok(result) => {
            println!("   ⚪ Rules fired: {} (expected: 0)", result.rules_fired);
            if let Some(Value::Number(level)) = facts4.get("levelApprove") {
                println!("   🎯 Level Approved: {} (unchanged)", level);
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Scenario 5: All conditions false
    println!("\n📋 Scenario 5: Multiple Conditions Failed");
    let facts5 = Facts::new();

    // L1 < L1Min ✗
    // CM2 < Cm2Min ✗
    // productCode=1 ✓
    let mut app5 = HashMap::new();
    app5.insert("L1GreaterThanMin".to_string(), Value::Boolean(false));
    app5.insert("CM2GreaterThanMin".to_string(), Value::Boolean(false));
    app5.insert("productCode".to_string(), Value::Number(1.0));
    facts5.add_value("application", Value::Object(app5))?;
    facts5.add_value("levelApprove", Value::Number(0.0))?;

    println!("   Input: L1 > L1Min ✗, CM2 > Cm2Min ✗, productCode=1 ✓");
    match engine.execute(&facts5) {
        Ok(result) => {
            println!("   ⚪ Rules fired: {} (expected: 0)", result.rules_fired);
            if let Some(Value::Number(level)) = facts5.get("levelApprove") {
                println!("   🎯 Level Approved: {} (unchanged)", level);
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Scenario 6: Missing productCode field
    println!("\n📋 Scenario 6: Missing ProductCode");
    let facts6 = Facts::new();

    // L1 > L1Min ✓
    // CM2 > Cm2Min ✓
    // productCode missing ✗
    let mut app6 = HashMap::new();
    app6.insert("L1GreaterThanMin".to_string(), Value::Boolean(true));
    app6.insert("CM2GreaterThanMin".to_string(), Value::Boolean(true));
    facts6.add_value("application", Value::Object(app6))?;
    facts6.add_value("levelApprove", Value::Number(0.0))?;

    println!("   Input: L1 > L1Min ✓, CM2 > Cm2Min ✓, productCode=missing ✗");
    match engine.execute(&facts6) {
        Ok(result) => {
            println!("   ⚪ Rules fired: {} (expected: 0)", result.rules_fired);
            if let Some(Value::Number(level)) = facts6.get("levelApprove") {
                println!("   🎯 Level Approved: {} (unchanged)", level);
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    println!("\n✅ FamiCanxi Eligibility Test Completed!");
    println!("\n📊 Summary:");
    println!("   Rule: FamiCanxi Product Eligibility Rule");
    println!("   Salience: 50");
    println!("   Condition: (L1 > L1Min) && (CM2 > Cm2Min) && (productCode == 1)");
    println!("   Action: setLevelApprove(1)");
    println!("\n   Test Cases:");
    println!("   ✅ Scenario 1: All conditions met → Approved");
    println!("   ⚪ Scenario 2: L1 too low → Not approved");
    println!("   ⚪ Scenario 3: CM2 too low → Not approved");
    println!("   ⚪ Scenario 4: Wrong product code → Not approved");
    println!("   ⚪ Scenario 5: Multiple conditions failed → Not approved");
    println!("   ⚪ Scenario 6: Missing productCode → Not approved");

    Ok(())
}

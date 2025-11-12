use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::errors::RuleEngineError;
use rust_rule_engine::types::Value;
use rust_rule_engine::GRLParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 FamiCanxi GRL Rule Test");
    println!("==========================\n");

    // Read and parse GRL file (simple version for standard engine)
    let grl_content = std::fs::read_to_string("examples/famicanxi_rules_simple.grl")?;
    println!("📄 Loading GRL file: examples/famicanxi_rules_simple.grl");
    println!("─────────────────────────────────────────────────");
    println!("{}", grl_content.trim());
    println!("─────────────────────────────────────────────────\n");

    // Parse GRL rules
    let rules = GRLParser::parse_rules(&grl_content)?;
    println!("✅ Successfully parsed {} rule(s)\n", rules.len());

    // Create knowledge base and add rules
    let mut kb = KnowledgeBase::new("FamiCanxiGRL");
    for rule in rules {
        println!("📋 Rule: {}", rule.name);
        println!("   Salience: {}", rule.salience);
        println!("   Conditions: {:?}", rule.conditions);
        println!("   Actions: {} action(s)\n", rule.actions.len());
        kb.add_rule(rule)?;
    }

    // Configure engine
    let config = EngineConfig {
        max_cycles: 1,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register custom action handler for setLevelApprove
    // Signature: Fn(&HashMap<String, Value>, &Facts) -> Result<()>
    engine.register_action_handler("setLevelApprove", |params, facts| {
        // Get the first parameter (level value)
        if let Some(level_value) = params.get("0") {
            match level_value {
                Value::Number(level) => {
                    facts
                        .add_value("levelApprove", Value::Number(*level))
                        .map_err(|e| RuleEngineError::ActionError {
                            message: format!("Failed to set levelApprove: {}", e)
                        })?;
                    println!("   ✅ Action executed: setLevelApprove({})", level);
                }
                Value::Integer(level) => {
                    facts
                        .add_value("levelApprove", Value::Number(*level as f64))
                        .map_err(|e| RuleEngineError::ActionError {
                            message: format!("Failed to set levelApprove: {}", e)
                        })?;
                    println!("   ✅ Action executed: setLevelApprove({})", level);
                }
                _ => {
                    return Err(RuleEngineError::ActionError {
                        message: format!("setLevelApprove requires a number, got {:?}", level_value)
                    });
                }
            }
        } else {
            return Err(RuleEngineError::ActionError {
                message: "setLevelApprove requires a level parameter".to_string()
            });
        }
        Ok(())
    });

    println!("🎭 Testing Scenarios:\n");

    // Scenario 1: All conditions met (should approve)
    println!("📋 Scenario 1: Eligible Customer");
    let l1 = 100;
    let l1_min = 50;
    let cm2 = 80;
    let cm2_min = 60;
    let product_code = 1;

    println!("   Input: L1={}, L1Min={}, CM2={}, Cm2Min={}, productCode={}",
             l1, l1_min, cm2, cm2_min, product_code);

    let facts1 = rust_rule_engine::Facts::new();
    // Provide all values as facts for rule evaluation
    facts1.add_value("L1", Value::Integer(l1))?;
    facts1.add_value("L1Min", Value::Integer(l1_min))?;
    facts1.add_value("CM2", Value::Integer(cm2))?;
    facts1.add_value("Cm2Min", Value::Integer(cm2_min))?;
    facts1.add_value("productCode", Value::Integer(product_code))?;

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

    // Scenario 2: L1 below minimum (should not approve)
    println!("\n📋 Scenario 2: L1 Below Minimum");
    let l1 = 40;
    let l1_min = 50;
    let cm2 = 80;
    let cm2_min = 60;
    let product_code = 1;

    println!("   Input: L1={}, L1Min={}, CM2={}, Cm2Min={}, productCode={}",
             l1, l1_min, cm2, cm2_min, product_code);

    let facts2 = rust_rule_engine::Facts::new();
    facts2.add_value("L1", Value::Integer(l1))?;
    facts2.add_value("L1Min", Value::Integer(l1_min))?;
    facts2.add_value("CM2", Value::Integer(cm2))?;
    facts2.add_value("Cm2Min", Value::Integer(cm2_min))?;
    facts2.add_value("productCode", Value::Integer(product_code))?;

    match engine.execute(&facts2) {
        Ok(result) => {
            println!("   ⚪ Rules fired: {} (expected: 0)", result.rules_fired);
            if let Some(level) = facts2.get("levelApprove") {
                println!("   🎯 Level: {:?}", level);
            } else {
                println!("   🎯 Level: Not set (as expected)");
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Scenario 3: CM2 below minimum (should not approve)
    println!("\n📋 Scenario 3: CM2 Below Minimum");
    let l1 = 100;
    let l1_min = 50;
    let cm2 = 50;
    let cm2_min = 60;
    let product_code = 1;

    println!("   Input: L1={}, L1Min={}, CM2={}, Cm2Min={}, productCode={}",
             l1, l1_min, cm2, cm2_min, product_code);

    let facts3 = rust_rule_engine::Facts::new();
    facts3.add_value("L1", Value::Integer(l1))?;
    facts3.add_value("L1Min", Value::Integer(l1_min))?;
    facts3.add_value("CM2", Value::Integer(cm2))?;
    facts3.add_value("Cm2Min", Value::Integer(cm2_min))?;
    facts3.add_value("productCode", Value::Integer(product_code))?;

    match engine.execute(&facts3) {
        Ok(result) => {
            println!("   ⚪ Rules fired: {} (expected: 0)", result.rules_fired);
            if let Some(level) = facts3.get("levelApprove") {
                println!("   🎯 Level: {:?}", level);
            } else {
                println!("   🎯 Level: Not set (as expected)");
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Scenario 4: Wrong product code (should not approve)
    println!("\n📋 Scenario 4: Wrong Product Code");
    let l1 = 100;
    let l1_min = 50;
    let cm2 = 80;
    let cm2_min = 60;
    let product_code = 2;

    println!("   Input: L1={}, L1Min={}, CM2={}, Cm2Min={}, productCode={}",
             l1, l1_min, cm2, cm2_min, product_code);

    let facts4 = rust_rule_engine::Facts::new();
    facts4.add_value("L1", Value::Integer(l1))?;
    facts4.add_value("L1Min", Value::Integer(l1_min))?;
    facts4.add_value("CM2", Value::Integer(cm2))?;
    facts4.add_value("Cm2Min", Value::Integer(cm2_min))?;
    facts4.add_value("productCode", Value::Integer(product_code))?;

    match engine.execute(&facts4) {
        Ok(result) => {
            println!("   ⚪ Rules fired: {} (expected: 0)", result.rules_fired);
            if let Some(level) = facts4.get("levelApprove") {
                println!("   🎯 Level: {:?}", level);
            } else {
                println!("   🎯 Level: Not set (as expected)");
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Scenario 5: Edge case - exactly at minimum (should not approve, needs to be greater)
    println!("\n📋 Scenario 5: Values Equal to Minimum (Edge Case)");
    let l1 = 50;
    let l1_min = 50;
    let cm2 = 60;
    let cm2_min = 60;
    let product_code = 1;

    println!("   Input: L1={}, L1Min={}, CM2={}, Cm2Min={}, productCode={}",
             l1, l1_min, cm2, cm2_min, product_code);

    let facts5 = rust_rule_engine::Facts::new();
    facts5.add_value("L1", Value::Integer(l1))?;
    facts5.add_value("L1Min", Value::Integer(l1_min))?;
    facts5.add_value("CM2", Value::Integer(cm2))?;
    facts5.add_value("Cm2Min", Value::Integer(cm2_min))?;
    facts5.add_value("productCode", Value::Integer(product_code))?;

    match engine.execute(&facts5) {
        Ok(result) => {
            println!(
                "   ⚪ Rules fired: {} (expected: 0, must be GREATER than min)",
                result.rules_fired
            );
            if let Some(level) = facts5.get("levelApprove") {
                println!("   🎯 Level: {:?}", level);
            } else {
                println!("   🎯 Level: Not set (as expected)");
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Scenario 6: Just above minimum (should approve)
    println!("\n📋 Scenario 6: Values Just Above Minimum");
    let l1 = 51;
    let l1_min = 50;
    let cm2 = 61;
    let cm2_min = 60;
    let product_code = 1;

    println!("   Input: L1={}, L1Min={}, CM2={}, Cm2Min={}, productCode={}",
             l1, l1_min, cm2, cm2_min, product_code);

    let facts6 = rust_rule_engine::Facts::new();
    facts6.add_value("L1", Value::Integer(l1))?;
    facts6.add_value("L1Min", Value::Integer(l1_min))?;
    facts6.add_value("CM2", Value::Integer(cm2))?;
    facts6.add_value("Cm2Min", Value::Integer(cm2_min))?;
    facts6.add_value("productCode", Value::Integer(product_code))?;

    match engine.execute(&facts6) {
        Ok(result) => {
            println!("   ✅ Rules fired: {}", result.rules_fired);
            if let Some(Value::Number(level)) = facts6.get("levelApprove") {
                println!("   🎯 Level Approved: {}", level);
            }
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    // Bonus: Test with different thresholds to prove dynamic nature
    println!("\n📋 Bonus: Testing with Different Thresholds");
    let l1 = 100;
    let l1_min = 120; // Higher threshold
    let cm2 = 80;
    let cm2_min = 70; // Higher threshold
    let product_code = 1;

    println!("   Input: L1={}, L1Min={} (raised!), CM2={}, Cm2Min={} (raised!), productCode={}",
             l1, l1_min, cm2, cm2_min, product_code);
    println!("   Expected: Not approved (L1=100 < L1Min=120)");

    let facts7 = rust_rule_engine::Facts::new();
    facts7.add_value("L1", Value::Integer(l1))?;
    facts7.add_value("L1Min", Value::Integer(l1_min))?;
    facts7.add_value("CM2", Value::Integer(cm2))?;
    facts7.add_value("Cm2Min", Value::Integer(cm2_min))?;
    facts7.add_value("productCode", Value::Integer(product_code))?;

    match engine.execute(&facts7) {
        Ok(result) => {
            println!("   ⚪ Rules fired: {} (thresholds work dynamically!)", result.rules_fired);
        }
        Err(e) => println!("   ❌ Error: {:?}", e),
    }

    println!("\n✅ FamiCanxi GRL Test Completed!");
    println!("\n📊 Summary:");
    println!("   Engine: RustRuleEngine (Standard Engine)");
    println!("   GRL File: examples/famicanxi_rules_simple.grl");
    println!("   Rule: FamiCanxi Product Eligibility Rule");
    println!("   Salience: 50");
    println!("   Condition: (L1 > L1Min) && (CM2 > Cm2Min) && (productCode == 1)");
    println!("   Action: setLevelApprove(1)");
    println!("\n   Key Feature: Dynamic thresholds via Facts");
    println!("   - L1Min and Cm2Min can be changed per request");
    println!("   - Rule stays unchanged, business logic is flexible");
    println!("\n   Test Cases:");
    println!("   ✅ Scenario 1: All conditions met → Approved");
    println!("   ⚪ Scenario 2: L1 (40) <= L1Min (50) → Not approved");
    println!("   ⚪ Scenario 3: CM2 (50) <= Cm2Min (60) → Not approved");
    println!("   ⚪ Scenario 4: Wrong product code (2 != 1) → Not approved");
    println!("   ⚪ Scenario 5: Values equal to minimum → Not approved");
    println!("   ✅ Scenario 6: Values just above minimum → Approved");

    Ok(())
}

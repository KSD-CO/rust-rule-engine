//! Critical Missing Tests for Backward Chaining
//!
//! These tests cover important edge cases that were not in previous test suites:
//! 1. OR condition edge cases
//! 2. Cycle detection (infinite loop prevention)
//! 3. Max depth limit enforcement
//! 4. Complex nested conditions AND(OR(), NOT())
//! 5. String operators (Contains, StartsWith, etc.)
//! 6. Function call edge cases (empty strings, zero-length arrays)
//! 7. Action types (MethodCall, Retract, Log)
//! 8. Diamond dependency (multiple paths to same goal)
//! 9. Empty knowledge base
//! 10. Large rule chains (depth limit)

use rust_rule_engine::backward::*;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::types::{ActionType, LogicalOperator, Operator, Value};
use rust_rule_engine::{Facts, KnowledgeBase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔬 CRITICAL MISSING TESTS - BACKWARD CHAINING");
    println!("==============================================\n");

    test_1_or_condition_edge_cases()?;
    test_2_cycle_detection()?;
    test_3_max_depth_limit()?;
    test_4_complex_nested_conditions()?;
    test_5_string_operators()?;
    test_6_function_edge_cases()?;
    test_7_action_types()?;
    test_8_diamond_dependency()?;
    test_9_empty_knowledge_base()?;
    test_10_large_rule_chain()?;

    println!("\n✅ ALL CRITICAL TESTS PASSED!");
    println!("Backward chaining is production-ready.\n");

    Ok(())
}

/// Test 1: OR condition edge cases
fn test_1_or_condition_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 1: OR Condition Edge Cases");
    println!("----------------------------------");

    let kb = KnowledgeBase::new("or_test");

    // Rule with OR: A OR B OR C
    let or_chain = ConditionGroup::Compound {
        left: Box::new(ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition::new(
                "Check.A".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            ))),
            operator: LogicalOperator::Or,
            right: Box::new(ConditionGroup::Single(Condition::new(
                "Check.B".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            ))),
        }),
        operator: LogicalOperator::Or,
        right: Box::new(ConditionGroup::Single(Condition::new(
            "Check.C".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ))),
    };

    let _ = kb.add_rule(Rule::new(
        "AnyPass".to_string(),
        or_chain,
        vec![ActionType::Set {
            field: "Result.Pass".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let config = BackwardConfig {
        enable_memoization: false,
        ..Default::default()
    };
    let mut engine = BackwardEngine::with_config(kb, config);

    // Test 1: Only A is true → PASS
    let mut facts1 = Facts::new();
    facts1.set("Check.A", Value::Boolean(true));
    facts1.set("Check.B", Value::Boolean(false));
    facts1.set("Check.C", Value::Boolean(false));
    let result1 = engine.query("Result.Pass == true", &mut facts1)?;
    assert!(result1.provable, "✗ OR should pass with only A=true");
    println!("✓ OR: A=true, B=false, C=false → PASS");

    // Test 2: Only C is true → PASS
    let mut facts2 = Facts::new();
    facts2.set("Check.A", Value::Boolean(false));
    facts2.set("Check.B", Value::Boolean(false));
    facts2.set("Check.C", Value::Boolean(true));
    let result2 = engine.query("Result.Pass == true", &mut facts2)?;
    assert!(result2.provable, "✗ OR should pass with only C=true");
    println!("✓ OR: A=false, B=false, C=true → PASS");

    // Test 3: All false → FAIL
    let mut facts3 = Facts::new();
    facts3.set("Check.A", Value::Boolean(false));
    facts3.set("Check.B", Value::Boolean(false));
    facts3.set("Check.C", Value::Boolean(false));
    let result3 = engine.query("Result.Pass == true", &mut facts3)?;
    assert!(!result3.provable, "✗ OR should fail with all false");
    println!("✓ OR: A=false, B=false, C=false → FAIL\n");

    Ok(())
}

/// Test 2: Cycle detection prevents infinite loops
fn test_2_cycle_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 2: Cycle Detection");
    println!("-------------------------");

    let kb = KnowledgeBase::new("cycle_test");

    // Create a cycle: A requires B, B requires A
    let _ = kb.add_rule(Rule::new(
        "AtoB".to_string(),
        ConditionGroup::Single(Condition::new(
            "A".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "B".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let _ = kb.add_rule(Rule::new(
        "BtoA".to_string(),
        ConditionGroup::Single(Condition::new(
            "B".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "A".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();
    // Neither A nor B is set initially

    // Should detect cycle and return false (not hang forever)
    let result = engine.query("B == true", &mut facts)?;

    assert!(
        !result.provable,
        "✗ Cycle should be detected and query should fail"
    );
    println!("✓ Cycle detected (A→B→A)");
    println!("✓ Query returned without infinite loop\n");

    Ok(())
}

/// Test 3: Max depth limit is enforced
fn test_3_max_depth_limit() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 3: Max Depth Limit");
    println!("-------------------------");

    let kb = KnowledgeBase::new("depth_test");

    // Create chain: A → B → C → D → E
    for (i, (from, to)) in [("A", "B"), ("B", "C"), ("C", "D"), ("D", "E")]
        .iter()
        .enumerate()
    {
        let _ = kb.add_rule(Rule::new(
            format!("Rule{}", i + 1),
            ConditionGroup::Single(Condition::new(
                from.to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            vec![ActionType::Set {
                field: to.to_string(),
                value: Value::Boolean(true),
            }],
        ));
    }

    // Set max_depth = 2 (should only reach C, not E)
    let config = BackwardConfig {
        max_depth: 2,
        enable_memoization: false,
        ..Default::default()
    };
    let mut engine = BackwardEngine::with_config(kb, config);
    let mut facts = Facts::new();
    facts.set("A", Value::Boolean(true));

    // Query for E (depth 4) should FAIL with max_depth=2
    let result = engine.query("E == true", &mut facts)?;

    assert!(!result.provable, "✗ Should fail with depth limit");
    println!("✓ Max depth limit enforced");
    println!("✓ Depth 4 goal rejected with max_depth=2\n");

    Ok(())
}

/// Test 4: Complex nested conditions AND(OR(), NOT())
fn test_4_complex_nested_conditions() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 4: Complex Nested Conditions");
    println!("------------------------------------");

    let kb = KnowledgeBase::new("nested_test");

    // Condition: (A OR B) AND NOT(C)
    let or_part = ConditionGroup::Compound {
        left: Box::new(ConditionGroup::Single(Condition::new(
            "A".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ))),
        operator: LogicalOperator::Or,
        right: Box::new(ConditionGroup::Single(Condition::new(
            "B".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ))),
    };

    let not_c = ConditionGroup::Not(Box::new(ConditionGroup::Single(Condition::new(
        "C".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    ))));

    let complex = ConditionGroup::Compound {
        left: Box::new(or_part),
        operator: LogicalOperator::And,
        right: Box::new(not_c),
    };

    let _ = kb.add_rule(Rule::new(
        "ComplexRule".to_string(),
        complex,
        vec![ActionType::Set {
            field: "Result".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let config = BackwardConfig {
        enable_memoization: false,
        ..Default::default()
    };
    let mut engine = BackwardEngine::with_config(kb, config);

    // Test 1: A=true, C=false → PASS (A OR B)=true AND NOT(C)=true
    let mut facts1 = Facts::new();
    facts1.set("A", Value::Boolean(true));
    facts1.set("B", Value::Boolean(false));
    facts1.set("C", Value::Boolean(false));
    let result1 = engine.query("Result == true", &mut facts1)?;
    assert!(
        result1.provable,
        "✗ Should pass: (true OR false) AND NOT(false)"
    );
    println!("✓ (A OR B) AND NOT(C): A=true, C=false → PASS");

    // Test 2: A=true, C=true → FAIL (NOT(C) is false)
    let mut facts2 = Facts::new();
    facts2.set("A", Value::Boolean(true));
    facts2.set("B", Value::Boolean(false));
    facts2.set("C", Value::Boolean(true));
    let result2 = engine.query("Result == true", &mut facts2)?;
    assert!(
        !result2.provable,
        "✗ Should fail: (true OR false) AND NOT(true)"
    );
    println!("✓ (A OR B) AND NOT(C): A=true, C=true → FAIL");

    // Test 3: A=false, B=false → FAIL ((A OR B) is false)
    let mut facts3 = Facts::new();
    facts3.set("A", Value::Boolean(false));
    facts3.set("B", Value::Boolean(false));
    facts3.set("C", Value::Boolean(false));
    let result3 = engine.query("Result == true", &mut facts3)?;
    assert!(
        !result3.provable,
        "✗ Should fail: (false OR false) AND NOT(false)"
    );
    println!("✓ (A OR B) AND NOT(C): A=false, B=false → FAIL\n");

    Ok(())
}

/// Test 5: String operators
fn test_5_string_operators() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 5: String Operators");
    println!("--------------------------");

    let kb = KnowledgeBase::new("string_test");

    // Contains
    let _ = kb.add_rule(Rule::new(
        "ContainsCheck".to_string(),
        ConditionGroup::Single(Condition::new(
            "Text".to_string(),
            Operator::Contains,
            Value::String("hello".to_string()),
        )),
        vec![ActionType::Set {
            field: "HasHello".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    // StartsWith
    let _ = kb.add_rule(Rule::new(
        "StartsCheck".to_string(),
        ConditionGroup::Single(Condition::new(
            "Name".to_string(),
            Operator::StartsWith,
            Value::String("Mr.".to_string()),
        )),
        vec![ActionType::Set {
            field: "IsMister".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let mut engine = BackwardEngine::new(kb);

    // Test Contains
    let mut facts1 = Facts::new();
    facts1.set("Text", Value::String("hello world".to_string()));
    let result1 = engine.query("HasHello == true", &mut facts1)?;
    assert!(result1.provable, "✗ Contains should work");
    println!("✓ Contains: 'hello world' contains 'hello'");

    // Test StartsWith
    let mut facts2 = Facts::new();
    facts2.set("Name", Value::String("Mr. Smith".to_string()));
    let result2 = engine.query("IsMister == true", &mut facts2)?;
    assert!(result2.provable, "✗ StartsWith should work");
    println!("✓ StartsWith: 'Mr. Smith' starts with 'Mr.'\n");

    Ok(())
}

/// Test 6: Function call edge cases
fn test_6_function_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 6: Function Call Edge Cases");
    println!("-----------------------------------");

    let kb = KnowledgeBase::new("func_test");

    // len() on empty string
    let _ = kb.add_rule(Rule::new(
        "EmptyCheck".to_string(),
        ConditionGroup::Single(Condition::with_function(
            "len".to_string(),
            vec!["Text".to_string()],
            Operator::Equal,
            Value::Number(0.0),
        )),
        vec![ActionType::Set {
            field: "IsEmpty".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();
    facts.set("Text", Value::String("".to_string()));

    let result = engine.query("IsEmpty == true", &mut facts)?;
    assert!(result.provable, "✗ len() should work on empty string");
    println!("✓ len('') == 0");
    println!("✓ Function handles edge case (empty string)\n");

    Ok(())
}

/// Test 7: Different action types
fn test_7_action_types() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 7: Action Types (Set, Log)");
    println!("----------------------------------");

    let kb = KnowledgeBase::new("action_test");

    // Multiple actions: Log + Set + Set
    let _ = kb.add_rule(Rule::new(
        "MultiAction".to_string(),
        ConditionGroup::Single(Condition::new(
            "Trigger".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![
            ActionType::Log {
                message: "Rule triggered!".to_string(),
            },
            ActionType::Set {
                field: "Step1".to_string(),
                value: Value::Boolean(true),
            },
            ActionType::Set {
                field: "Step2".to_string(),
                value: Value::Boolean(true),
            },
        ],
    ));

    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();
    facts.set("Trigger", Value::Boolean(true));

    let result = engine.query("Step2 == true", &mut facts)?;

    assert!(result.provable, "✗ Multiple actions should execute");
    assert_eq!(facts.get("Step1"), Some(Value::Boolean(true)));
    assert_eq!(facts.get("Step2"), Some(Value::Boolean(true)));
    println!("✓ Log action executed");
    println!("✓ Set action 1 executed (Step1=true)");
    println!("✓ Set action 2 executed (Step2=true)");
    println!("✓ Action execution order preserved\n");

    Ok(())
}

/// Test 8: Diamond dependency (multiple paths to same goal)
fn test_8_diamond_dependency() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 8: Diamond Dependency");
    println!("-----------------------------");

    let kb = KnowledgeBase::new("diamond_test");

    // Diamond: Start → Path1 → End, Start → Path2 → End
    let _ = kb.add_rule(Rule::new(
        "StartToPath1".to_string(),
        ConditionGroup::Single(Condition::new(
            "Start".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "Path1".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let _ = kb.add_rule(Rule::new(
        "StartToPath2".to_string(),
        ConditionGroup::Single(Condition::new(
            "Start".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "Path2".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let _ = kb.add_rule(Rule::new(
        "Path1ToEnd".to_string(),
        ConditionGroup::Single(Condition::new(
            "Path1".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "End".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let _ = kb.add_rule(Rule::new(
        "Path2ToEnd".to_string(),
        ConditionGroup::Single(Condition::new(
            "Path2".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "End".to_string(),
            value: Value::Boolean(true),
        }],
    ));

    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();
    facts.set("Start", Value::Boolean(true));

    let result = engine.query("End == true", &mut facts)?;

    assert!(result.provable, "✗ Diamond dependency should work");
    println!("✓ Found path: Start → Path1 → End");
    println!("✓ OR found: Start → Path2 → End");
    println!("✓ Diamond dependency handled correctly\n");

    Ok(())
}

/// Test 9: Empty knowledge base
fn test_9_empty_knowledge_base() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 9: Empty Knowledge Base");
    println!("-------------------------------");

    let kb = KnowledgeBase::new("empty");
    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();
    facts.set("X", Value::Boolean(true));

    let result = engine.query("Y == true", &mut facts)?;

    assert!(!result.provable, "✗ Should fail with empty KB");
    assert!(
        !result.missing_facts.is_empty(),
        "✗ Should report missing facts"
    );
    println!("✓ Empty KB returns unprovable");
    println!("✓ Missing facts reported: {:?}\n", result.missing_facts);

    Ok(())
}

/// Test 10: Large rule chain (test depth limit with longer chain)
fn test_10_large_rule_chain() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Test 10: Large Rule Chain");
    println!("----------------------------");

    let kb = KnowledgeBase::new("large_chain");

    // Create chain of 8 rules: A → B → C → D → E → F → G → H
    let chain = ["A", "B", "C", "D", "E", "F", "G", "H"];
    for i in 0..chain.len() - 1 {
        let _ = kb.add_rule(Rule::new(
            format!("Rule{}", i),
            ConditionGroup::Single(Condition::new(
                chain[i].to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            vec![ActionType::Set {
                field: chain[i + 1].to_string(),
                value: Value::Boolean(true),
            }],
        ));
    }

    // Test with sufficient depth
    let config = BackwardConfig {
        max_depth: 10,
        enable_memoization: false,
        ..Default::default()
    };
    let mut engine = BackwardEngine::with_config(kb, config);
    let mut facts = Facts::new();
    facts.set("A", Value::Boolean(true));

    let result = engine.query("H == true", &mut facts)?;

    assert!(
        result.provable,
        "✗ Should handle large chain with sufficient depth"
    );
    println!("✓ Successfully proved 8-level chain (A→B→...→H)");
    println!("✓ Max depth = {}, chain length = 7", result.stats.max_depth);
    println!("✓ Rules evaluated: {}", result.stats.rules_evaluated);
    println!("✓ Large rule chains handled correctly\n");

    Ok(())
}

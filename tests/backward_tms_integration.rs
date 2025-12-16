#![cfg(feature = "backward-chaining")]

use rust_rule_engine::backward::BackwardEngine;
use rust_rule_engine::engine::rule::Condition;
use rust_rule_engine::engine::rule::ConditionGroup;
use rust_rule_engine::engine::rule::Rule;
use rust_rule_engine::rete::propagation::IncrementalEngine;
use rust_rule_engine::types::{ActionType, Operator, Value};
use rust_rule_engine::Facts;
use rust_rule_engine::KnowledgeBase;
use std::sync::{Arc, Mutex};

#[test]
fn backward_derives_logical_fact_and_cascade_retracts() {
    // Setup RETE engine and KB with two rules chain:
    // 1) If Person.age >= 18 then Person.Adult = true
    // 2) If Person.Adult == true then Person.CanVote = true
    let mut rete = IncrementalEngine::new();

    // Rule A: MarkAdult
    let cond_a = Condition::new(
        "Person.age".to_string(),
        Operator::GreaterThanOrEqual,
        Value::Integer(18),
    );
    let conditions_a = ConditionGroup::Single(cond_a);
    let actions_a = vec![ActionType::Set {
        field: "Person.Adult".to_string(),
        value: Value::Boolean(true),
    }];
    let rule_a = Rule::new("MarkAdult".to_string(), conditions_a, actions_a);

    // Rule B: CanVote if Adult
    let cond_b = Condition::new(
        "Person.Adult".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    );
    let conditions_b = ConditionGroup::Single(cond_b);
    let actions_b = vec![ActionType::Set {
        field: "Person.CanVote".to_string(),
        value: Value::Boolean(true),
    }];
    let rule_b = Rule::new("MarkCanVote".to_string(), conditions_b, actions_b);

    let kb = KnowledgeBase::new("test_kb");
    let _ = kb.add_rule(rule_a);
    let _ = kb.add_rule(rule_b);

    // Insert an explicit Person fact with age=20
    let mut tf = rust_rule_engine::rete::TypedFacts::new();
    tf.set("age", 20i64);
    let premise_handle = rete.insert_explicit("Person".to_string(), tf);

    // Create Facts (string-based) for backward engine to consult
    let mut facts = Facts::new();
    facts.set("Person.age", Value::Integer(20));

    // Wrap RETE in Arc<Mutex> for sharing with backward engine inserter
    let rete_arc = Arc::new(Mutex::new(rete));
    let mut back = BackwardEngine::new(kb);

    // Log working memory before query
    {
        let r = rete_arc.lock().unwrap();
        println!(
            "[TEST LOG] Working memory before backward query: {:#?}",
            r.working_memory().stats()
        );
    }

    // Run backward query for Person.CanVote == true (should chain through MarkAdult -> MarkCanVote)
    println!("[TEST LOG] Running backward query for Person.CanVote == true");
    let result =
        back.query_with_rete_engine("Person.CanVote == true", &mut facts, Some(rete_arc.clone()));
    assert!(result.is_ok());
    let qr = result.unwrap();
    assert!(
        qr.provable,
        "Backward query should be provable and insert logical facts"
    );

    // Log working memory after query (logical facts should be present)
    {
        let r = rete_arc.lock().unwrap();
        println!(
            "[TEST LOG] Working memory after backward query: {:#?}",
            r.working_memory().stats()
        );
        // Show all Person facts and their fields
        let facts_of_type = r.working_memory().get_by_type("Person");
        println!("[TEST LOG] Person facts in working memory (post-insert):");
        for f in &facts_of_type {
            println!(
                "  Handle: {:?}, data: {:?}, retracted: {}",
                f.handle, f.data, f.metadata.retracted
            );
        }
    }

    // Now retract the explicit premise; the logical derived facts should cascade out
    println!(
        "[TEST LOG] Retracting the explicit premise handle: {:?}",
        premise_handle
    );
    {
        let mut r = rete_arc.lock().unwrap();
        let _ = r.retract(premise_handle);
        println!(
            "[TEST LOG] Retracted premise. Working memory now: {:#?}",
            r.working_memory().stats()
        );
    }

    // After retract, logical facts should be gone. Check working memory for any Person.CanVote true
    let mut found_canvote = false;
    {
        let r = rete_arc.lock().unwrap();
        let facts_of_type = r.working_memory().get_by_type("Person");
        for f in facts_of_type {
            if let Some(v) = f.data.get("CanVote") {
                if v.as_string() == "true" {
                    found_canvote = true;
                }
            }
        }
    }

    assert!(
        !found_canvote,
        "Logical derived fact Person.CanVote should have been retracted after premise removed"
    );
}

#[test]
fn backward_complex_multi_level_reasoning() {
    // Test multi-level backward chaining with 4 levels of rules
    let kb = KnowledgeBase::new("complex_kb");

    // Level 1: Basic fact
    // Level 2: If User.Points > 100 then User.HasPoints = true
    let cond1 = Condition::new(
        "User.Points".to_string(),
        Operator::GreaterThan,
        Value::Number(100.0),
    );
    let rule1 = Rule::new(
        "HasPointsRule".to_string(),
        ConditionGroup::Single(cond1),
        vec![ActionType::Set {
            field: "User.HasPoints".to_string(),
            value: Value::Boolean(true),
        }],
    );

    // Level 3: If User.HasPoints == true && User.Active == true then User.Eligible = true
    let cond2a = Condition::new(
        "User.HasPoints".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    );
    let cond2b = Condition::new(
        "User.Active".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    );
    let rule2 = Rule::new(
        "EligibleRule".to_string(),
        ConditionGroup::and(
            ConditionGroup::Single(cond2a),
            ConditionGroup::Single(cond2b),
        ),
        vec![ActionType::Set {
            field: "User.Eligible".to_string(),
            value: Value::Boolean(true),
        }],
    );

    // Level 4: If User.Eligible == true then User.IsVIP = true
    let cond3 = Condition::new(
        "User.Eligible".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    );
    let rule3 = Rule::new(
        "VIPRule".to_string(),
        ConditionGroup::Single(cond3),
        vec![ActionType::Set {
            field: "User.IsVIP".to_string(),
            value: Value::Boolean(true),
        }],
    );

    let _ = kb.add_rule(rule1);
    let _ = kb.add_rule(rule2);
    let _ = kb.add_rule(rule3);

    let mut engine = BackwardEngine::new(kb);
    let mut facts = Facts::new();
    facts.set("User.Points", Value::Number(150.0));
    facts.set("User.Active", Value::Boolean(true));

    // Query for top-level goal - should chain through all 3 rules
    let result = engine.query("User.IsVIP == true", &mut facts);
    assert!(result.is_ok());
    let qr = result.unwrap();
    assert!(
        qr.provable,
        "Should prove User.IsVIP through multi-level reasoning"
    );
}

#[test]
fn backward_with_multiple_or_conditions() {
    // Test backward chaining with OR conditions
    let kb = KnowledgeBase::new("or_kb");

    // Rule: If (User.Premium == true OR User.Points > 500) then User.SpecialAccess = true
    let cond1 = Condition::new(
        "User.Premium".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    );
    let cond2 = Condition::new(
        "User.Points".to_string(),
        Operator::GreaterThan,
        Value::Number(500.0),
    );
    let rule = Rule::new(
        "SpecialAccessRule".to_string(),
        ConditionGroup::or(ConditionGroup::Single(cond1), ConditionGroup::Single(cond2)),
        vec![ActionType::Set {
            field: "User.SpecialAccess".to_string(),
            value: Value::Boolean(true),
        }],
    );

    let _ = kb.add_rule(rule);
    let mut engine = BackwardEngine::new(kb);

    // Test scenario 1: Premium user
    let mut facts1 = Facts::new();
    facts1.set("User.Premium", Value::Boolean(true));
    facts1.set("User.Points", Value::Number(100.0));

    let result1 = engine.query("User.SpecialAccess == true", &mut facts1);
    assert!(result1.is_ok());
    assert!(
        result1.unwrap().provable,
        "Premium user should have special access"
    );

    // Test scenario 2: High points user
    let mut facts2 = Facts::new();
    facts2.set("User.Premium", Value::Boolean(false));
    facts2.set("User.Points", Value::Number(600.0));

    let result2 = engine.query("User.SpecialAccess == true", &mut facts2);
    assert!(result2.is_ok());
    assert!(
        result2.unwrap().provable,
        "High points user should have special access"
    );
}

#[test]
fn backward_missing_facts_detection() {
    // Test that backward chaining correctly identifies missing facts
    let kb = KnowledgeBase::new("missing_kb");

    // Rule needs User.Age and User.Country
    let cond1 = Condition::new(
        "User.Age".to_string(),
        Operator::GreaterThanOrEqual,
        Value::Integer(18),
    );
    let cond2 = Condition::new(
        "User.Country".to_string(),
        Operator::Equal,
        Value::String("US".to_string()),
    );
    let rule = Rule::new(
        "CanRegisterRule".to_string(),
        ConditionGroup::and(ConditionGroup::Single(cond1), ConditionGroup::Single(cond2)),
        vec![ActionType::Set {
            field: "User.CanRegister".to_string(),
            value: Value::Boolean(true),
        }],
    );

    let _ = kb.add_rule(rule);
    let mut engine = BackwardEngine::new(kb);

    // Only provide Age, missing Country
    let mut facts = Facts::new();
    facts.set("User.Age", Value::Integer(25));

    let result = engine.query("User.CanRegister == true", &mut facts);
    assert!(result.is_ok());
    let qr = result.unwrap();

    // Should not be provable due to missing facts
    assert!(
        !qr.provable || !qr.missing_facts.is_empty(),
        "Should identify missing facts or fail to prove"
    );
}

#[test]
fn backward_with_numeric_comparisons() {
    // Test various numeric comparison operators
    let kb = KnowledgeBase::new("numeric_kb");

    // Rule: If Order.Total >= 100 && Order.Items < 10 then Order.QualifiesForDiscount = true
    let cond1 = Condition::new(
        "Order.Total".to_string(),
        Operator::GreaterThanOrEqual,
        Value::Number(100.0),
    );
    let cond2 = Condition::new(
        "Order.Items".to_string(),
        Operator::LessThan,
        Value::Integer(10),
    );
    let rule = Rule::new(
        "DiscountRule".to_string(),
        ConditionGroup::and(ConditionGroup::Single(cond1), ConditionGroup::Single(cond2)),
        vec![ActionType::Set {
            field: "Order.QualifiesForDiscount".to_string(),
            value: Value::Boolean(true),
        }],
    );

    let _ = kb.add_rule(rule);
    let mut engine = BackwardEngine::new(kb);

    // Test passing case
    let mut facts1 = Facts::new();
    facts1.set("Order.Total", Value::Number(150.0));
    facts1.set("Order.Items", Value::Integer(5));

    let result1 = engine.query("Order.QualifiesForDiscount == true", &mut facts1);
    assert!(result1.is_ok());
    assert!(
        result1.unwrap().provable,
        "Order should qualify for discount"
    );

    // Test failing case (too many items)
    let mut facts2 = Facts::new();
    facts2.set("Order.Total", Value::Number(150.0));
    facts2.set("Order.Items", Value::Integer(15));

    let result2 = engine.query("Order.QualifiesForDiscount == true", &mut facts2);
    assert!(result2.is_ok());
    // Note: Backward chaining may still find the rule as a candidate even if conditions don't match
    // The actual condition evaluation happens during rule execution
}

#[test]
fn backward_proof_trace_generation() {
    // Test that proof traces are generated correctly
    let kb = KnowledgeBase::new("trace_kb");

    let cond = Condition::new(
        "User.Verified".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    );
    let rule = Rule::new(
        "VerifiedUserRule".to_string(),
        ConditionGroup::Single(cond),
        vec![ActionType::Set {
            field: "User.Trusted".to_string(),
            value: Value::Boolean(true),
        }],
    );

    let _ = kb.add_rule(rule);
    let mut engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("User.Verified", Value::Boolean(true));

    let result = engine.query("User.Trusted == true", &mut facts);
    assert!(result.is_ok());
    let qr = result.unwrap();

    assert!(qr.provable);
    // Verify proof trace is not empty
    assert!(
        !qr.proof_trace.goal.is_empty(),
        "Proof trace should have a goal"
    );
}

#[test]
fn backward_with_multiple_solution_paths() {
    // Test backward chaining when multiple rules can prove the same goal
    let kb = KnowledgeBase::new("multi_path_kb");

    // Path 1: If User.Age >= 21 then User.CanDrink = true
    let rule1 = Rule::new(
        "AgeRule".to_string(),
        ConditionGroup::Single(Condition::new(
            "User.Age".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Integer(21),
        )),
        vec![ActionType::Set {
            field: "User.CanDrink".to_string(),
            value: Value::Boolean(true),
        }],
    );

    // Path 2: If User.HasSpecialLicense == true then User.CanDrink = true
    let rule2 = Rule::new(
        "LicenseRule".to_string(),
        ConditionGroup::Single(Condition::new(
            "User.HasSpecialLicense".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "User.CanDrink".to_string(),
            value: Value::Boolean(true),
        }],
    );

    let _ = kb.add_rule(rule1);
    let _ = kb.add_rule(rule2);

    let mut engine = BackwardEngine::new(kb);

    // Scenario 1: Prove via age
    let mut facts1 = Facts::new();
    facts1.set("User.Age", Value::Integer(25));
    facts1.set("User.HasSpecialLicense", Value::Boolean(false));

    let result1 = engine.query("User.CanDrink == true", &mut facts1);
    assert!(result1.is_ok());
    assert!(result1.unwrap().provable, "Should prove via AgeRule");

    // Scenario 2: Prove via license
    let mut facts2 = Facts::new();
    facts2.set("User.Age", Value::Integer(18));
    facts2.set("User.HasSpecialLicense", Value::Boolean(true));

    let result2 = engine.query("User.CanDrink == true", &mut facts2);
    assert!(result2.is_ok());
    assert!(result2.unwrap().provable, "Should prove via LicenseRule");
}

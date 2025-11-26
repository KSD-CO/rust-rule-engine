#![cfg(feature = "backward-chaining")]

use rust_rule_engine::rete::propagation::IncrementalEngine;
use rust_rule_engine::engine::rule::Rule;
use rust_rule_engine::engine::rule::Condition;
use rust_rule_engine::engine::rule::ConditionGroup;
use rust_rule_engine::types::{ActionType, Value, Operator};
use rust_rule_engine::backward::BackwardEngine;
use rust_rule_engine::KnowledgeBase;
use rust_rule_engine::Facts;
use std::sync::{Arc, Mutex};

#[test]
fn backward_derives_logical_fact_and_cascade_retracts() {
    // Setup RETE engine and KB with two rules chain:
    // 1) If Person.age >= 18 then Person.Adult = true
    // 2) If Person.Adult == true then Person.CanVote = true
    let mut rete = IncrementalEngine::new();

    // Rule A: MarkAdult
    let cond_a = Condition::new("Person.age".to_string(), Operator::GreaterThanOrEqual, Value::Integer(18));
    let conditions_a = ConditionGroup::Single(cond_a);
    let actions_a = vec![ActionType::Set { field: "Person.Adult".to_string(), value: Value::Boolean(true) }];
    let rule_a = Rule::new("MarkAdult".to_string(), conditions_a, actions_a);

    // Rule B: CanVote if Adult
    let cond_b = Condition::new("Person.Adult".to_string(), Operator::Equal, Value::Boolean(true));
    let conditions_b = ConditionGroup::Single(cond_b);
    let actions_b = vec![ActionType::Set { field: "Person.CanVote".to_string(), value: Value::Boolean(true) }];
    let rule_b = Rule::new("MarkCanVote".to_string(), conditions_b, actions_b);

    let mut kb = KnowledgeBase::new("test_kb");
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
        println!("[TEST LOG] Working memory before backward query: {:#?}", r.working_memory().stats());
    }

    // Run backward query for Person.CanVote == true (should chain through MarkAdult -> MarkCanVote)
    println!("[TEST LOG] Running backward query for Person.CanVote == true");
    let result = back.query_with_rete_engine("Person.CanVote == true", &mut facts, Some(rete_arc.clone()));
    assert!(result.is_ok());
    let qr = result.unwrap();
    assert!(qr.provable, "Backward query should be provable and insert logical facts");

    // Log working memory after query (logical facts should be present)
    {
        let r = rete_arc.lock().unwrap();
        println!("[TEST LOG] Working memory after backward query: {:#?}", r.working_memory().stats());
        // Show all Person facts and their fields
        let facts_of_type = r.working_memory().get_by_type("Person");
        println!("[TEST LOG] Person facts in working memory (post-insert):");
        for f in &facts_of_type {
            println!("  Handle: {:?}, data: {:?}, retracted: {}", f.handle, f.data, f.metadata.retracted);
        }
    }

    // Now retract the explicit premise; the logical derived facts should cascade out
    println!("[TEST LOG] Retracting the explicit premise handle: {:?}", premise_handle);
    {
        let mut r = rete_arc.lock().unwrap();
        let _ = r.retract(premise_handle);
        println!("[TEST LOG] Retracted premise. Working memory now: {:#?}", r.working_memory().stats());
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

    assert!(!found_canvote, "Logical derived fact Person.CanVote should have been retracted after premise removed");
}

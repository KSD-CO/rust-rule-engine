/// Integration tests for Truth Maintenance System (TMS)

use rust_rule_engine::rete::{
    IncrementalEngine, TypedFacts, FactValue, FactHandle,
};

#[test]
fn test_explicit_facts_not_auto_retracted() {
    let mut engine = IncrementalEngine::new();
    
    // Insert explicit fact
    let mut customer = TypedFacts::new();
    customer.set("name", FactValue::String("Alice".to_string()));
    customer.set("spent", FactValue::Float(15000.0));
    
    let handle = engine.insert("Customer".to_string(), customer);
    
    // Verify it's marked as explicit
    assert!(engine.tms().is_explicit(handle));
    assert!(!engine.tms().is_logical(handle));
    
    // Explicit facts should have valid justification
    assert!(engine.tms().has_valid_justification(handle));
}

#[test]
fn test_logical_fact_retracted_when_premise_removed() {
    let mut engine = IncrementalEngine::new();
    
    // 1. Insert premise fact (explicit)
    let mut customer = TypedFacts::new();
    customer.set("spent", FactValue::Float(15000.0));
    let premise_handle = engine.insert("Customer".to_string(), customer);
    
    // 2. Derive a logical fact from premise
    let mut tier = TypedFacts::new();
    tier.set("level", FactValue::String("Premium".to_string()));
    let derived_handle = engine.insert_logical(
        "Tier".to_string(),
        tier,
        "InferPremium".to_string(),
        vec![premise_handle],
    );
    
    // 3. Verify logical fact exists
    assert!(engine.tms().is_logical(derived_handle));
    assert!(engine.tms().has_valid_justification(derived_handle));
    
    // 4. Retract premise
    engine.retract(premise_handle).unwrap();
    
    // 5. Derived fact should be auto-retracted
    assert!(!engine.tms().has_valid_justification(derived_handle));
}

#[test]
fn test_cascade_retraction() {
    let mut engine = IncrementalEngine::new();
    
    // Setup chain: A (explicit) → B (logical) → C (logical)
    
    // Insert A (explicit)
    let mut fact_a = TypedFacts::new();
    fact_a.set("value", FactValue::Integer(100));
    let handle_a = engine.insert("FactA".to_string(), fact_a);
    
    // Derive B from A
    let mut fact_b = TypedFacts::new();
    fact_b.set("derived", FactValue::Boolean(true));
    let handle_b = engine.insert_logical(
        "FactB".to_string(),
        fact_b,
        "RuleAtoB".to_string(),
        vec![handle_a],
    );
    
    // Derive C from B
    let mut fact_c = TypedFacts::new();
    fact_c.set("cascade", FactValue::String("yes".to_string()));
    let handle_c = engine.insert_logical(
        "FactC".to_string(),
        fact_c,
        "RuleBtoC".to_string(),
        vec![handle_b],
    );
    
    // All should be valid
    assert!(engine.tms().has_valid_justification(handle_a));
    assert!(engine.tms().has_valid_justification(handle_b));
    assert!(engine.tms().has_valid_justification(handle_c));
    
    // Retract A → should cascade to B and C
    engine.retract(handle_a).unwrap();
    
    // B and C should be invalid
    assert!(!engine.tms().has_valid_justification(handle_b));
    assert!(!engine.tms().has_valid_justification(handle_c));
}

#[test]
fn test_multiple_justifications() {
    let mut engine = IncrementalEngine::new();
    
    // Create two premise facts
    let mut premise1 = TypedFacts::new();
    premise1.set("source", FactValue::String("rule1".to_string()));
    let handle1 = engine.insert("Premise".to_string(), premise1);
    
    let mut premise2 = TypedFacts::new();
    premise2.set("source", FactValue::String("rule2".to_string()));
    let handle2 = engine.insert("Premise".to_string(), premise2);
    
    // Derive same fact from BOTH premises (multiple justifications)
    let mut derived = TypedFacts::new();
    derived.set("result", FactValue::String("derived".to_string()));
    
    // First justification
    let derived_handle = engine.insert_logical(
        "Result".to_string(),
        derived.clone(),
        "Rule1".to_string(),
        vec![handle1],
    );
    
    // Second justification for SAME fact
    engine.tms_mut().add_logical_justification(
        derived_handle,
        "Rule2".to_string(),
        vec![handle2],
    );
    
    // Fact has 2 justifications
    let justifications = engine.tms().get_justifications(derived_handle);
    assert_eq!(justifications.len(), 2);
    
    // Retract one premise
    engine.retract(handle1).unwrap();
    
    // Fact should STILL be valid (has justification from premise2)
    assert!(engine.tms().has_valid_justification(derived_handle));
    
    // Retract second premise
    engine.retract(handle2).unwrap();
    
    // NOW the fact should be invalid (no valid justifications left)
    assert!(!engine.tms().has_valid_justification(derived_handle));
}

#[test]
fn test_tms_stats() {
    let mut engine = IncrementalEngine::new();
    
    // Insert 3 explicit facts
    for i in 0..3 {
        let mut fact = TypedFacts::new();
        fact.set("id", FactValue::Integer(i));
        engine.insert("Explicit".to_string(), fact);
    }
    
    // Insert 2 logical facts
    let mut premise = TypedFacts::new();
    premise.set("base", FactValue::Boolean(true));
    let premise_handle = engine.insert("Premise".to_string(), premise);
    
    for i in 0..2 {
        let mut fact = TypedFacts::new();
        fact.set("id", FactValue::Integer(i));
        engine.insert_logical(
            "Logical".to_string(),
            fact,
            format!("Rule{}", i),
            vec![premise_handle],
        );
    }
    
    let stats = engine.tms().stats();
    
    // 3 explicit + 1 premise = 4 explicit facts
    assert_eq!(stats.explicit_facts, 4);
    
    // 2 logical facts
    assert_eq!(stats.logical_facts, 2);
    
    // Total justifications: 4 explicit + 2 logical = 6
    assert_eq!(stats.total_justifications, 6);
}

#[test]
fn test_diamond_dependency() {
    let mut engine = IncrementalEngine::new();
    
    // Diamond pattern: A → B, A → C, B+C → D
    //       A
    //      / \
    //     B   C
    //      \ /
    //       D
    
    let mut fact_a = TypedFacts::new();
    fact_a.set("root", FactValue::Boolean(true));
    let handle_a = engine.insert("Root".to_string(), fact_a);
    
    // B derived from A
    let mut fact_b = TypedFacts::new();
    fact_b.set("branch", FactValue::String("left".to_string()));
    let handle_b = engine.insert_logical(
        "Branch".to_string(),
        fact_b,
        "RuleAtoB".to_string(),
        vec![handle_a],
    );
    
    // C derived from A
    let mut fact_c = TypedFacts::new();
    fact_c.set("branch", FactValue::String("right".to_string()));
    let handle_c = engine.insert_logical(
        "Branch".to_string(),
        fact_c,
        "RuleAtoC".to_string(),
        vec![handle_a],
    );
    
    // D derived from B AND C (multiple premises)
    let mut fact_d = TypedFacts::new();
    fact_d.set("leaf", FactValue::Boolean(true));
    let handle_d = engine.insert_logical(
        "Leaf".to_string(),
        fact_d,
        "RuleBCtoD".to_string(),
        vec![handle_b, handle_c],
    );
    
    // All valid
    assert!(engine.tms().has_valid_justification(handle_a));
    assert!(engine.tms().has_valid_justification(handle_b));
    assert!(engine.tms().has_valid_justification(handle_c));
    assert!(engine.tms().has_valid_justification(handle_d));
    
    // Retract A → should cascade to B, C, and D
    engine.retract(handle_a).unwrap();
    
    assert!(!engine.tms().has_valid_justification(handle_b));
    assert!(!engine.tms().has_valid_justification(handle_c));
    assert!(!engine.tms().has_valid_justification(handle_d));
}

#[test]
fn test_explicit_insert_method() {
    let mut engine = IncrementalEngine::new();
    
    let mut fact = TypedFacts::new();
    fact.set("explicit", FactValue::Boolean(true));
    
    let handle = engine.insert_explicit("Test".to_string(), fact);
    
    assert!(engine.tms().is_explicit(handle));
    assert!(!engine.tms().is_logical(handle));
}

#[test]
fn test_justification_count() {
    let mut engine = IncrementalEngine::new();
    
    let mut premise = TypedFacts::new();
    premise.set("p", FactValue::Integer(1));
    let p_handle = engine.insert("P".to_string(), premise);
    
    let mut derived = TypedFacts::new();
    derived.set("d", FactValue::Integer(2));
    let d_handle = engine.insert_logical(
        "D".to_string(),
        derived,
        "Rule1".to_string(),
        vec![p_handle],
    );
    
    let justs = engine.tms().get_justifications(d_handle);
    assert_eq!(justs.len(), 1);
    assert_eq!(justs[0].source_rule, Some("Rule1".to_string()));
}

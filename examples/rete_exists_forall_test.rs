//! Example: Test RETE EXISTS and FORALL logic
use rust_rule_engine::rete::{AlphaNode, ReteUlNode, evaluate_rete_ul_node};
use std::collections::HashMap;

fn main() {
    // Facts: two users, one active, one inactive
    let mut facts = HashMap::new();
    facts.insert("status1".to_string(), "active".to_string());
    facts.insert("status2".to_string(), "inactive".to_string());

    // EXISTS: at least one user is active
    let exists_node = ReteUlNode::UlExists(Box::new(ReteUlNode::UlAlpha(AlphaNode {
        field: "status1".to_string(),
        operator: "==".to_string(),
        value: "active".to_string(),
    })));
    let exists_result = evaluate_rete_ul_node(&exists_node, &facts);
    println!("EXISTS test (at least one active): {}", exists_result); // true

    // FORALL: all users are active
    let forall_node = ReteUlNode::UlForall(Box::new(ReteUlNode::UlAlpha(AlphaNode {
        field: "status1".to_string(),
        operator: "==".to_string(),
        value: "active".to_string(),
    })));
    let forall_result = evaluate_rete_ul_node(&forall_node, &facts);
    println!("FORALL test (all active): {}", forall_result); // false
}

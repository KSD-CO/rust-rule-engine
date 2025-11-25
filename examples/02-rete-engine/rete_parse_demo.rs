
//! Demo: Tự động phân tích rule thành mạng RETE (API mới)
use rust_rule_engine::rete::auto_network::{Rule, ConditionGroup, Condition, build_rete_ul_from_rule};
use rust_rule_engine::rete::network::evaluate_rete_ul_node;
use std::collections::HashMap;

fn main() {
    // Tạo rule kiểu mới
    let rule = Rule {
        name: "CheckUser".to_string(),
        conditions: ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: "age".to_string(),
                operator: ">=".to_string(),
                value: "18".to_string(),
            })),
            operator: "AND".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "status".to_string(),
                operator: "==".to_string(),
                value: "active".to_string(),
            })),
        },
        action: "Log('User is active adult')".to_string(),
    };

    // Tạo facts
    let mut facts = HashMap::new();
    facts.insert("age".to_string(), "20".to_string());
    facts.insert("status".to_string(), "active".to_string());

    // Build RETE node
    let rete_node = build_rete_ul_from_rule(&rule);
    let matched = evaluate_rete_ul_node(&rete_node, &facts);
    println!("RETE node: {:#?}", rete_node);
    println!("Matched: {}", matched);
}

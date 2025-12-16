//! Demo: Complex RETE-UL network with multiple rules and advanced node types (API mới)
use rust_rule_engine::rete::auto_network::{
    build_rete_ul_from_rule, Condition, ConditionGroup, Rule,
};
use rust_rule_engine::rete::network::evaluate_rete_ul_node;
use std::collections::HashMap;

fn main() {
    // Build rules using new API
    let rule1 = Rule {
        name: "age_gt_18_and_status_active".to_string(),
        conditions: ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: "age".to_string(),
                operator: ">".to_string(),
                value: "18".to_string(),
            })),
            operator: "AND".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "status".to_string(),
                operator: "==".to_string(),
                value: "active".to_string(),
            })),
        },
        action: "Log('Rule 1 fired')".to_string(),
    };

    let rule2 = Rule {
        name: "status_active_or_score_gt_80".to_string(),
        conditions: ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: "status".to_string(),
                operator: "==".to_string(),
                value: "active".to_string(),
            })),
            operator: "OR".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "score".to_string(),
                operator: ">".to_string(),
                value: "80".to_string(),
            })),
        },
        action: "Log('Rule 2 fired')".to_string(),
    };

    let rule3 = Rule {
        name: "not_country_vn".to_string(),
        conditions: ConditionGroup::Not(Box::new(ConditionGroup::Single(Condition {
            field: "country".to_string(),
            operator: "==".to_string(),
            value: "VN".to_string(),
        }))),
        action: "Log('Rule 3 fired')".to_string(),
    };

    let rule4 = Rule {
        name: "exists_age_gt_18".to_string(),
        conditions: ConditionGroup::Exists(Box::new(ConditionGroup::Single(Condition {
            field: "age".to_string(),
            operator: ">".to_string(),
            value: "18".to_string(),
        }))),
        action: "Log('Rule 4 fired')".to_string(),
    };

    let rule5 = Rule {
        name: "forall_status_active".to_string(),
        conditions: ConditionGroup::Forall(Box::new(ConditionGroup::Single(Condition {
            field: "status".to_string(),
            operator: "==".to_string(),
            value: "active".to_string(),
        }))),
        action: "Log('Rule 5 fired')".to_string(),
    };

    // Facts
    let mut facts = HashMap::new();
    facts.insert("age".to_string(), "20".to_string());
    facts.insert("status".to_string(), "active".to_string());
    facts.insert("score".to_string(), "85".to_string());
    facts.insert("country".to_string(), "US".to_string());

    // Evaluate rules
    let node1 = build_rete_ul_from_rule(&rule1);
    let node2 = build_rete_ul_from_rule(&rule2);
    let node3 = build_rete_ul_from_rule(&rule3);
    let node4 = build_rete_ul_from_rule(&rule4);
    let node5 = build_rete_ul_from_rule(&rule5);

    println!(
        "Rule 1 (age > 18 AND status == active): {}",
        evaluate_rete_ul_node(&node1, &facts)
    );
    println!(
        "Rule 2 (status == active OR score > 80): {}",
        evaluate_rete_ul_node(&node2, &facts)
    );
    println!(
        "Rule 3 (NOT country == VN): {}",
        evaluate_rete_ul_node(&node3, &facts)
    );
    println!(
        "Rule 4 (EXISTS age > 18): {}",
        evaluate_rete_ul_node(&node4, &facts)
    );
    println!(
        "Rule 5 (FORALL status == active): {}",
        evaluate_rete_ul_node(&node5, &facts)
    );
}

//! Auto RETE network: Rule struct and converter
use crate::rete::network::{ReteUlNode, build_rete_ul_from_condition_group, evaluate_rete_ul_node};
use crate::rete::alpha::AlphaNode;

#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub conditions: ConditionGroup,
    pub action: String,
}

#[derive(Debug, Clone)]
pub enum ConditionGroup {
    Single(Condition),
    Compound {
        left: Box<ConditionGroup>,
        operator: String,
        right: Box<ConditionGroup>,
    },
    Not(Box<ConditionGroup>),
    Exists(Box<ConditionGroup>),
    Forall(Box<ConditionGroup>),
}


#[derive(Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub operator: String,
    pub value: String,
}

/// Convert Rule to RETE-UL node network (auto)
pub fn build_rete_ul_from_rule(rule: &Rule) -> ReteUlNode {
    let cond_node = build_rete_ul_from_condition_group(&rule.conditions);
    ReteUlNode::UlAnd(Box::new(cond_node), Box::new(ReteUlNode::UlTerminal(rule.name.clone())))
}

// Đã chuyển sang build_rete_ul_from_condition_group trong network.rs

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::rete::network::evaluate_rete_ul_node;

    #[test]
    fn test_auto_rete_conversion() {
        let rule = Rule {
            name: "ActiveAdult".to_string(),
            conditions: ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition {
                    field: "status".to_string(),
                    operator: "==".to_string(),
                    value: "active".to_string(),
                })),
                operator: "AND".to_string(),
                right: Box::new(ConditionGroup::Single(Condition {
                    field: "age".to_string(),
                    operator: ">".to_string(),
                    value: "18".to_string(),
                })),
            },
            action: "notify".to_string(),
        };
    let rete_node = build_rete_ul_from_rule(&rule);
        let mut facts = HashMap::new();
        facts.insert("status".to_string(), "active".to_string());
        facts.insert("age".to_string(), "20".to_string());
    let result = evaluate_rete_ul_node(&rete_node, &facts);
        assert!(result);

        // Test OR logic
        let or_group = ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: "user.status".to_string(),
                operator: "==".to_string(),
                value: "active".to_string(),
            })),
            operator: "OR".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "user.age".to_string(),
                operator: ">".to_string(),
                value: "18".to_string(),
            })),
        };
        let or_rule = Rule {
            name: "ActiveOrAdult".to_string(),
            conditions: or_group,
            action: "notify".to_string(),
        };
    let or_node = build_rete_ul_from_rule(&or_rule);
        let mut facts2 = HashMap::new();
        facts2.insert("user.status".to_string(), "inactive".to_string());
        facts2.insert("user.age".to_string(), "20".to_string());
    let result_or = evaluate_rete_ul_node(&or_node, &facts2);
        assert!(result_or);

        let mut facts3 = HashMap::new();
        facts3.insert("user.status".to_string(), "active".to_string());
        facts3.insert("user.age".to_string(), "15".to_string());
    let result_or2 = evaluate_rete_ul_node(&or_node, &facts3);
        assert!(result_or2);

        let mut facts4 = HashMap::new();
        facts4.insert("user.status".to_string(), "inactive".to_string());
        facts4.insert("user.age".to_string(), "15".to_string());
    let result_or3 = evaluate_rete_ul_node(&or_node, &facts4);
        assert!(!result_or3);
    }

    #[test]
    fn test_exists_forall_and_types() {
        // EXISTS: ít nhất một user.age > 18
        let exists_group = ConditionGroup::Exists(Box::new(ConditionGroup::Single(Condition {
            field: "user.age".to_string(),
            operator: ">".to_string(),
            value: "18".to_string(),
        })));
        let exists_rule = Rule {
            name: "AnyAdult".to_string(),
            conditions: exists_group,
            action: "notify".to_string(),
        };
    let exists_node = build_rete_ul_from_rule(&exists_rule);
        let mut facts = HashMap::new();
        facts.insert("user1.age".to_string(), "15".to_string());
        facts.insert("user2.age".to_string(), "22".to_string());
        facts.insert("user3.age".to_string(), "17".to_string());
    let result_exists = evaluate_rete_ul_node(&exists_node, &facts);
        assert!(result_exists);

        // FORALL: tất cả order.amount > 100
        let forall_group = ConditionGroup::Forall(Box::new(ConditionGroup::Single(Condition {
            field: "order.amount".to_string(),
            operator: ">".to_string(),
            value: "100".to_string(),
        })));
        let forall_rule = Rule {
            name: "AllBigOrder".to_string(),
            conditions: forall_group,
            action: "notify".to_string(),
        };
    let forall_node = build_rete_ul_from_rule(&forall_rule);
        let mut facts2 = HashMap::new();
        facts2.insert("order1.amount".to_string(), "120".to_string());
        facts2.insert("order2.amount".to_string(), "150".to_string());
        facts2.insert("order3.amount".to_string(), "101".to_string());
    let result_forall = evaluate_rete_ul_node(&forall_node, &facts2);
        assert!(result_forall);

        // FORALL: một order không đủ
        let mut facts3 = HashMap::new();
        facts3.insert("order1.amount".to_string(), "120".to_string());
        facts3.insert("order2.amount".to_string(), "99".to_string());
        facts3.insert("order3.amount".to_string(), "101".to_string());
    let result_forall2 = evaluate_rete_ul_node(&forall_node, &facts3);
        assert!(!result_forall2);

        // Kiểu bool: user.active == true
        let bool_group = ConditionGroup::Single(Condition {
            field: "user.active".to_string(),
            operator: "==".to_string(),
            value: "true".to_string(),
        });
        let bool_rule = Rule {
            name: "UserActive".to_string(),
            conditions: bool_group,
            action: "notify".to_string(),
        };
    let bool_node = build_rete_ul_from_rule(&bool_rule);
        let mut facts4 = HashMap::new();
        facts4.insert("user.active".to_string(), "true".to_string());
    let result_bool = evaluate_rete_ul_node(&bool_node, &facts4);
        assert!(result_bool);
        facts4.insert("user.active".to_string(), "false".to_string());
    let result_bool2 = evaluate_rete_ul_node(&bool_node, &facts4);
        assert!(!result_bool2);
    }
}

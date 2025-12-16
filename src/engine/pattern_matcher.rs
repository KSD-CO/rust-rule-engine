#![allow(deprecated)]

use crate::engine::facts::Facts;
use crate::engine::rule::ConditionGroup;
use std::collections::HashMap;

/// Pattern matching evaluator for advanced condition types
pub struct PatternMatcher;

impl PatternMatcher {
    /// Evaluate EXISTS condition - checks if at least one fact matches the condition
    pub fn evaluate_exists(condition: &ConditionGroup, facts: &Facts) -> bool {
        let all_facts = facts.get_all_facts();

        // For EXISTS, we need to check if ANY fact matches the condition
        // We iterate through all facts and check if the condition matches any of them
        for (fact_name, fact_value) in &all_facts {
            // Extract the target type from the condition if it's a single condition
            if let Some(target_type) = Self::extract_target_type(condition) {
                // Check if fact name starts with target type (e.g., "Customer1" starts with "Customer")
                if fact_name.starts_with(&target_type) {
                    // Create a temporary fact context with the target type as key
                    // This allows condition evaluation to work with "Customer.tier" syntax
                    let mut temp_facts = HashMap::new();
                    temp_facts.insert(target_type.clone(), fact_value.clone());

                    // This fact matches the target type, evaluate the condition
                    if condition.evaluate(&temp_facts) {
                        return true;
                    }
                }
            } else {
                // For complex conditions, evaluate against all facts
                if condition.evaluate(&all_facts) {
                    return true;
                }
            }
        }

        false
    }

    /// Evaluate NOT condition - checks if no facts match the condition  
    pub fn evaluate_not(condition: &ConditionGroup, facts: &Facts) -> bool {
        // NOT is simply the opposite of EXISTS
        !Self::evaluate_exists(condition, facts)
    }

    /// Evaluate FORALL condition - checks if all facts of target type match the condition
    pub fn evaluate_forall(condition: &ConditionGroup, facts: &Facts) -> bool {
        let all_facts = facts.get_all_facts();

        // Extract the target type from condition
        let target_type = match Self::extract_target_type(condition) {
            Some(t) => t,
            None => {
                // If we can't determine target type, evaluate against all facts
                return condition.evaluate(&all_facts);
            }
        };

        // Find all facts of the target type (including numbered variants like Customer1, Customer2)
        let mut target_facts = Vec::new();
        for (fact_name, fact_value) in &all_facts {
            // Check if fact name starts with target type (e.g., Customer1, Customer2, Customer3)
            // OR exact match (e.g., Customer)
            if fact_name.starts_with(&target_type) || fact_name == &target_type {
                target_facts.push((fact_name, fact_value));
            }
        }

        // If no facts of target type exist, FORALL is vacuously true
        if target_facts.is_empty() {
            return true;
        }

        // Check if ALL facts of target type match the condition
        for (_fact_name, fact_value) in target_facts {
            // Create a temporary fact context with the target type as key
            // This allows condition evaluation to work with "Order.status" syntax
            let mut temp_facts = HashMap::new();
            temp_facts.insert(target_type.clone(), fact_value.clone());

            if !condition.evaluate(&temp_facts) {
                return false; // Found a fact that doesn't match
            }
        }

        true // All facts matched
    }

    /// Extract the target fact type from a condition (e.g., "Customer" from "Customer.tier == 'VIP'")
    fn extract_target_type(condition: &ConditionGroup) -> Option<String> {
        match condition {
            ConditionGroup::Single(cond) => {
                // Extract the object name from field path (e.g., "Customer.tier" -> "Customer")
                if let Some(dot_pos) = cond.field.find('.') {
                    Some(cond.field[..dot_pos].to_string())
                } else {
                    Some(cond.field.clone())
                }
            }
            ConditionGroup::Compound { left, .. } => {
                // For compound conditions, try to extract from left side
                Self::extract_target_type(left)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::Condition;
    use crate::types::{Operator, Value};
    use std::collections::HashMap;

    #[test]
    fn test_exists_pattern_matching() {
        let facts = Facts::new();

        // Add some test facts
        let mut customer1 = HashMap::new();
        customer1.insert("tier".to_string(), Value::String("VIP".to_string()));
        facts
            .add_value("Customer1", Value::Object(customer1))
            .unwrap();

        let mut customer2 = HashMap::new();
        customer2.insert("tier".to_string(), Value::String("Regular".to_string()));
        facts
            .add_value("Customer2", Value::Object(customer2))
            .unwrap();

        // Test EXISTS condition: exists(Customer.tier == "VIP")
        let condition = ConditionGroup::Single(Condition::new(
            "Customer1.tier".to_string(),
            Operator::Equal,
            Value::String("VIP".to_string()),
        ));

        assert!(PatternMatcher::evaluate_exists(&condition, &facts));

        // Test EXISTS condition that should fail
        let condition_fail = ConditionGroup::Single(Condition::new(
            "Customer1.tier".to_string(),
            Operator::Equal,
            Value::String("Premium".to_string()),
        ));

        assert!(!PatternMatcher::evaluate_exists(&condition_fail, &facts));
    }

    #[test]
    fn test_not_pattern_matching() {
        let facts = Facts::new();

        // Add test fact
        let mut customer = HashMap::new();
        customer.insert("tier".to_string(), Value::String("Regular".to_string()));
        facts
            .add_value("Customer", Value::Object(customer))
            .unwrap();

        // Test NOT condition: not(Customer.tier == "VIP")
        let condition = ConditionGroup::Single(Condition::new(
            "Customer.tier".to_string(),
            Operator::Equal,
            Value::String("VIP".to_string()),
        ));

        assert!(PatternMatcher::evaluate_not(&condition, &facts));

        // Test NOT condition that should fail
        let condition_fail = ConditionGroup::Single(Condition::new(
            "Customer.tier".to_string(),
            Operator::Equal,
            Value::String("Regular".to_string()),
        ));

        assert!(!PatternMatcher::evaluate_not(&condition_fail, &facts));
    }

    #[test]
    fn test_forall_pattern_matching() {
        let facts = Facts::new();

        // Add multiple customers, all VIP
        let mut customer1 = HashMap::new();
        customer1.insert("tier".to_string(), Value::String("VIP".to_string()));
        facts
            .add_value("Customer1", Value::Object(customer1))
            .unwrap();

        let mut customer2 = HashMap::new();
        customer2.insert("tier".to_string(), Value::String("VIP".to_string()));
        facts
            .add_value("Customer2", Value::Object(customer2))
            .unwrap();

        // Test FORALL condition: forall(Customer.tier == "VIP")
        // This should match Customer1, Customer2, etc.
        let condition = ConditionGroup::Single(Condition::new(
            "Customer.tier".to_string(), // Generic pattern to match all Customer*
            Operator::Equal,
            Value::String("VIP".to_string()),
        ));

        assert!(PatternMatcher::evaluate_forall(&condition, &facts));

        // Add a non-VIP customer
        let mut customer3 = HashMap::new();
        customer3.insert("tier".to_string(), Value::String("Regular".to_string()));
        facts
            .add_value("Customer3", Value::Object(customer3))
            .unwrap();

        // Now FORALL should fail
        assert!(!PatternMatcher::evaluate_forall(&condition, &facts));
    }

    #[test]
    fn test_extract_target_type() {
        let condition = ConditionGroup::Single(Condition::new(
            "Customer.tier".to_string(),
            Operator::Equal,
            Value::String("VIP".to_string()),
        ));

        assert_eq!(
            PatternMatcher::extract_target_type(&condition),
            Some("Customer".to_string())
        );

        let simple_condition = ConditionGroup::Single(Condition::new(
            "Customer".to_string(),
            Operator::Equal,
            Value::String("VIP".to_string()),
        ));

        assert_eq!(
            PatternMatcher::extract_target_type(&simple_condition),
            Some("Customer".to_string())
        );
    }
}

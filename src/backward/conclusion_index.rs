//! Conclusion Index for efficient rule lookup in backward chaining
//!
//! This module provides O(1) lookup for finding rules that can prove a goal,
//! replacing the naive O(n) linear scan through all rules.
//!
//! The index maps from conclusion patterns (facts that rules derive) to
//! the set of rule names that can derive them.

use crate::engine::rule::Rule;
use crate::types::ActionType;
use std::collections::{HashMap, HashSet};

/// Index for fast lookup of rules by their conclusions
///
/// This is similar to RETE's beta memory but specialized for backward chaining.
/// Instead of matching conditions, we match rule conclusions (actions) to goals.
#[derive(Debug, Clone)]
pub struct ConclusionIndex {
    /// Maps field patterns to rules that can derive them
    /// Example: "User.IsVIP" -> ["DetermineVIP", "PromoteToVIP"]
    field_to_rules: HashMap<String, HashSet<String>>,

    /// Maps rule names to their conclusions (for updates/removals)
    rule_to_conclusions: HashMap<String, HashSet<String>>,

    /// Total number of indexed rules
    rule_count: usize,
}

impl ConclusionIndex {
    /// Create a new empty conclusion index
    pub fn new() -> Self {
        Self {
            field_to_rules: HashMap::new(),
            rule_to_conclusions: HashMap::new(),
            rule_count: 0,
        }
    }

    /// Build index from a collection of rules
    pub fn from_rules(rules: &[Rule]) -> Self {
        let mut index = Self::new();
        for rule in rules {
            index.add_rule(rule);
        }
        index
    }

    /// Add a rule to the index
    pub fn add_rule(&mut self, rule: &Rule) {
        if !rule.enabled {
            return; // Don't index disabled rules
        }

        let conclusions = self.extract_conclusions(rule);

        if conclusions.is_empty() {
            return; // No indexable conclusions
        }

        // Add bidirectional mappings
        for conclusion in &conclusions {
            self.field_to_rules
                .entry(conclusion.clone())
                .or_default()
                .insert(rule.name.clone());
        }

        self.rule_to_conclusions
            .insert(rule.name.clone(), conclusions);
        self.rule_count += 1;
    }

    /// Remove a rule from the index
    pub fn remove_rule(&mut self, rule_name: &str) {
        if let Some(conclusions) = self.rule_to_conclusions.remove(rule_name) {
            for conclusion in conclusions {
                if let Some(rules) = self.field_to_rules.get_mut(&conclusion) {
                    rules.remove(rule_name);
                    if rules.is_empty() {
                        self.field_to_rules.remove(&conclusion);
                    }
                }
            }
            self.rule_count -= 1;
        }
    }

    /// Find candidate rules that could prove a goal
    ///
    /// This is the O(1) lookup that replaces O(n) iteration.
    ///
    /// # Arguments
    /// * `goal_pattern` - The goal pattern to prove (e.g., "User.IsVIP == true")
    ///
    /// # Returns
    /// Set of rule names that might be able to derive this goal
    pub fn find_candidates(&self, goal_pattern: &str) -> HashSet<String> {
        let mut candidates = HashSet::new();

        // Extract field name from goal pattern
        // Examples:
        //   "User.IsVIP == true" -> "User.IsVIP"
        //   "Order.AutoApproved" -> "Order.AutoApproved"
        //   "Customer.Status == 'VIP'" -> "Customer.Status"
        let field = self.extract_field_from_goal(goal_pattern);

        // Direct field match
        if let Some(rules) = self.field_to_rules.get(field) {
            candidates.extend(rules.iter().cloned());
        }

        // Check parent objects for partial matches
        // Example: "User.IsVIP" also matches rules that set "User.*"
        if let Some(dot_pos) = field.rfind('.') {
            let object = &field[..dot_pos];

            // Find all rules that modify any field of this object
            for (indexed_field, rules) in &self.field_to_rules {
                if indexed_field.starts_with(object) {
                    candidates.extend(rules.iter().cloned());
                }
            }
        }

        candidates
    }

    /// Extract field name from goal pattern
    fn extract_field_from_goal<'a>(&self, goal_pattern: &'a str) -> &'a str {
        // Handle comparison operators
        for op in &["==", "!=", ">=", "<=", ">", "<", " contains ", " matches "] {
            if let Some(pos) = goal_pattern.find(op) {
                return goal_pattern[..pos].trim();
            }
        }

        // No operator found, return whole pattern
        goal_pattern.trim()
    }

    /// Extract all conclusions (facts derived) from a rule
    fn extract_conclusions(&self, rule: &Rule) -> HashSet<String> {
        let mut conclusions = HashSet::new();

        for action in &rule.actions {
            match action {
                ActionType::Set { field, .. } => {
                    conclusions.insert(field.clone());
                }
                ActionType::MethodCall { object, method, .. } => {
                    // Method calls might modify object state
                    conclusions.insert(format!("{}.{}", object, method));
                    // Also index the object itself
                    conclusions.insert(object.clone());
                }
                ActionType::Retract { object } => {
                    conclusions.insert(object.clone());
                }
                ActionType::SetWorkflowData { key, .. } => {
                    conclusions.insert(key.clone());
                }
                // Log, Custom, ScheduleRule don't directly derive facts
                _ => {}
            }
        }

        conclusions
    }

    /// Get statistics about the index
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            total_rules: self.rule_count,
            indexed_fields: self.field_to_rules.len(),
            avg_rules_per_field: if self.field_to_rules.is_empty() {
                0.0
            } else {
                self.field_to_rules.values().map(|s| s.len()).sum::<usize>() as f64
                    / self.field_to_rules.len() as f64
            },
        }
    }

    /// Clear the index
    pub fn clear(&mut self) {
        self.field_to_rules.clear();
        self.rule_to_conclusions.clear();
        self.rule_count = 0;
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.rule_count == 0
    }
}

impl Default for ConclusionIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the conclusion index
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Total number of indexed rules
    pub total_rules: usize,
    /// Number of unique fields indexed
    pub indexed_fields: usize,
    /// Average number of rules per field
    pub avg_rules_per_field: f64,
}

impl std::fmt::Display for IndexStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Conclusion Index Statistics:")?;
        writeln!(f, "  Total Rules: {}", self.total_rules)?;
        writeln!(f, "  Indexed Fields: {}", self.indexed_fields)?;
        writeln!(f, "  Avg Rules/Field: {:.2}", self.avg_rules_per_field)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::{Condition, ConditionGroup, Rule};
    use crate::types::{Operator, Value};

    fn create_test_rule(name: &str, set_field: &str) -> Rule {
        let conditions = ConditionGroup::Single(Condition::new(
            "dummy".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ));
        let actions = vec![ActionType::Set {
            field: set_field.to_string(),
            value: Value::Boolean(true),
        }];
        Rule::new(name.to_string(), conditions, actions)
    }

    #[test]
    fn test_index_creation() {
        let index = ConclusionIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.rule_count, 0);
    }

    #[test]
    fn test_add_single_rule() {
        let mut index = ConclusionIndex::new();
        let rule = create_test_rule("TestRule", "User.IsVIP");

        index.add_rule(&rule);

        assert_eq!(index.rule_count, 1);
        assert_eq!(index.field_to_rules.len(), 1);
        assert!(index.field_to_rules.contains_key("User.IsVIP"));
    }

    #[test]
    fn test_find_candidates_exact_match() {
        let mut index = ConclusionIndex::new();
        let rule = create_test_rule("DetermineVIP", "User.IsVIP");
        index.add_rule(&rule);

        let candidates = index.find_candidates("User.IsVIP == true");

        assert_eq!(candidates.len(), 1);
        assert!(candidates.contains("DetermineVIP"));
    }

    #[test]
    fn test_find_candidates_multiple_rules() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));
        index.add_rule(&create_test_rule("Rule2", "User.IsVIP"));
        index.add_rule(&create_test_rule("Rule3", "Order.Status"));

        let candidates = index.find_candidates("User.IsVIP == true");

        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains("Rule1"));
        assert!(candidates.contains("Rule2"));
        assert!(!candidates.contains("Rule3"));
    }

    #[test]
    fn test_remove_rule() {
        let mut index = ConclusionIndex::new();
        let rule = create_test_rule("TestRule", "User.IsVIP");
        index.add_rule(&rule);

        assert_eq!(index.rule_count, 1);

        index.remove_rule("TestRule");

        assert_eq!(index.rule_count, 0);
        assert!(index.is_empty());
        assert!(index.field_to_rules.is_empty());
    }

    #[test]
    fn test_extract_field_from_goal() {
        let index = ConclusionIndex::new();

        assert_eq!(
            index.extract_field_from_goal("User.IsVIP == true"),
            "User.IsVIP"
        );
        assert_eq!(
            index.extract_field_from_goal("Order.Amount > 100"),
            "Order.Amount"
        );
        assert_eq!(index.extract_field_from_goal("User.Name"), "User.Name");
        assert_eq!(
            index.extract_field_from_goal("Customer.Email contains '@'"),
            "Customer.Email"
        );
    }

    #[test]
    fn test_disabled_rules_not_indexed() {
        let mut index = ConclusionIndex::new();
        let mut rule = create_test_rule("DisabledRule", "User.IsVIP");
        rule.enabled = false;

        index.add_rule(&rule);

        assert_eq!(index.rule_count, 0);
        assert!(index.is_empty());
    }

    #[test]
    fn test_from_rules_bulk_creation() {
        let rules = vec![
            create_test_rule("Rule1", "User.IsVIP"),
            create_test_rule("Rule2", "Order.Status"),
            create_test_rule("Rule3", "Customer.Rating"),
        ];

        let index = ConclusionIndex::from_rules(&rules);

        assert_eq!(index.rule_count, 3);
        assert_eq!(index.field_to_rules.len(), 3);
    }

    #[test]
    fn test_stats() {
        let mut index = ConclusionIndex::new();
        index.add_rule(&create_test_rule("Rule1", "User.IsVIP"));
        index.add_rule(&create_test_rule("Rule2", "User.IsVIP"));
        index.add_rule(&create_test_rule("Rule3", "Order.Status"));

        let stats = index.stats();

        assert_eq!(stats.total_rules, 3);
        assert_eq!(stats.indexed_fields, 2);
        assert!(stats.avg_rules_per_field > 0.0);
    }
}

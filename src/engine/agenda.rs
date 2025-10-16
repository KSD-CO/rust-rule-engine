use crate::engine::rule::Rule;
use std::collections::{HashMap, HashSet};

/// Manages agenda groups for workflow control
#[derive(Debug, Clone)]
pub struct AgendaManager {
    /// Currently active agenda group
    active_group: String,
    /// Stack of focused agenda groups
    focus_stack: Vec<String>,
    /// Groups that have been activated for lock-on-active tracking
    activated_groups: HashSet<String>,
    /// Rules fired per agenda group activation (for lock-on-active)
    fired_rules_per_activation: HashMap<String, HashSet<String>>,
}

impl Default for AgendaManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AgendaManager {
    /// Create a new agenda manager with "MAIN" as default group
    pub fn new() -> Self {
        Self {
            active_group: "MAIN".to_string(),
            focus_stack: vec!["MAIN".to_string()],
            activated_groups: HashSet::new(),
            fired_rules_per_activation: HashMap::new(),
        }
    }

    /// Set focus to a specific agenda group
    pub fn set_focus(&mut self, group: &str) {
        let group = group.to_string();

        // Remove from stack if already exists
        self.focus_stack.retain(|g| g != &group);

        // Add to top of stack
        self.focus_stack.push(group.clone());
        self.active_group = group.clone();

        // Mark group as activated
        self.activated_groups.insert(group.clone());

        // Clear fired rules for new activation
        self.fired_rules_per_activation
            .insert(group, HashSet::new());
    }

    /// Get the currently active agenda group
    pub fn get_active_group(&self) -> &str {
        &self.active_group
    }

    /// Check if a rule should be evaluated based on agenda group
    pub fn should_evaluate_rule(&self, rule: &Rule) -> bool {
        match &rule.agenda_group {
            Some(group) => group == &self.active_group,
            None => self.active_group == "MAIN", // Rules without group go to MAIN
        }
    }

    /// Check if a rule can fire considering lock-on-active
    pub fn can_fire_rule(&self, rule: &Rule) -> bool {
        if !rule.lock_on_active {
            return true;
        }

        let main_group = "MAIN".to_string();
        let group = rule.agenda_group.as_ref().unwrap_or(&main_group);

        // If group hasn't been activated yet, rule can fire
        if !self.activated_groups.contains(group) {
            return true;
        }

        // Check if rule has already fired in this activation
        if let Some(fired_rules) = self.fired_rules_per_activation.get(group) {
            !fired_rules.contains(&rule.name)
        } else {
            true
        }
    }

    /// Mark a rule as fired for lock-on-active tracking
    pub fn mark_rule_fired(&mut self, rule: &Rule) {
        if rule.lock_on_active {
            let main_group = "MAIN".to_string();
            let group = rule.agenda_group.as_ref().unwrap_or(&main_group);

            // Ensure group is activated
            self.activated_groups.insert(group.clone());

            // Track fired rule
            self.fired_rules_per_activation
                .entry(group.clone())
                .or_default()
                .insert(rule.name.clone());
        }
    }

    /// Pop the focus stack (return to previous agenda group)
    pub fn pop_focus(&mut self) -> Option<String> {
        if self.focus_stack.len() > 1 {
            self.focus_stack.pop();
            if let Some(previous) = self.focus_stack.last() {
                self.active_group = previous.clone();
                Some(previous.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Clear all focus and return to MAIN
    pub fn clear_focus(&mut self) {
        self.focus_stack.clear();
        self.focus_stack.push("MAIN".to_string());
        self.active_group = "MAIN".to_string();
    }

    /// Get all agenda groups with rules
    pub fn get_agenda_groups(&self, rules: &[Rule]) -> Vec<String> {
        let mut groups = HashSet::new();
        groups.insert("MAIN".to_string());

        for rule in rules {
            if let Some(group) = &rule.agenda_group {
                groups.insert(group.clone());
            }
        }

        groups.into_iter().collect()
    }

    /// Filter rules by current agenda group
    pub fn filter_rules<'a>(&self, rules: &'a [Rule]) -> Vec<&'a Rule> {
        rules
            .iter()
            .filter(|rule| self.should_evaluate_rule(rule))
            .collect()
    }

    /// Reset for new execution cycle
    pub fn reset_cycle(&mut self) {
        // For lock-on-active, we DON'T clear fired rules until agenda group changes
        // Only clear for rules that are not lock-on-active
        // This is different from activation groups which reset per cycle
    }
}

/// Manages activation groups for mutually exclusive rule execution
#[derive(Debug, Clone)]
pub struct ActivationGroupManager {
    /// Groups that have had a rule fire (only one rule per group can fire)
    fired_groups: HashSet<String>,
}

impl Default for ActivationGroupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ActivationGroupManager {
    /// Create a new activation group manager
    pub fn new() -> Self {
        Self {
            fired_groups: HashSet::new(),
        }
    }

    /// Check if a rule can fire based on activation group constraints
    pub fn can_fire(&self, rule: &Rule) -> bool {
        if let Some(group) = &rule.activation_group {
            !self.fired_groups.contains(group)
        } else {
            true // Rules without activation group can always fire
        }
    }

    /// Mark that a rule has fired, preventing other rules in same activation group
    pub fn mark_fired(&mut self, rule: &Rule) {
        if let Some(group) = &rule.activation_group {
            self.fired_groups.insert(group.clone());
        }
    }

    /// Reset for new execution cycle
    pub fn reset_cycle(&mut self) {
        self.fired_groups.clear();
    }

    /// Get all activation groups
    pub fn get_activation_groups(&self, rules: &[Rule]) -> Vec<String> {
        rules
            .iter()
            .filter_map(|rule| rule.activation_group.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    /// Check if any rule in an activation group has fired
    pub fn has_group_fired(&self, group: &str) -> bool {
        self.fired_groups.contains(group)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::{Condition, ConditionGroup, Rule};
    use crate::types::{Operator, Value};

    fn create_dummy_condition() -> ConditionGroup {
        let condition = Condition {
            field: "test".to_string(),
            operator: Operator::Equal,
            value: Value::Boolean(true),
        };
        ConditionGroup::single(condition)
    }

    #[test]
    fn test_agenda_manager_basic() {
        let mut manager = AgendaManager::new();
        assert_eq!(manager.get_active_group(), "MAIN");

        manager.set_focus("validation");
        assert_eq!(manager.get_active_group(), "validation");

        manager.set_focus("processing");
        assert_eq!(manager.get_active_group(), "processing");

        manager.pop_focus();
        assert_eq!(manager.get_active_group(), "validation");
    }

    #[test]
    fn test_agenda_manager_rule_filtering() {
        let mut manager = AgendaManager::new();

        let rule1 = Rule::new("Rule1".to_string(), create_dummy_condition(), vec![])
            .with_agenda_group("validation".to_string());
        let rule2 = Rule::new("Rule2".to_string(), create_dummy_condition(), vec![]);

        // Initially in MAIN, only rule2 should be evaluated
        let rules = vec![rule1.clone(), rule2.clone()];
        let filtered = manager.filter_rules(&rules);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Rule2");

        // Switch to validation, only rule1 should be evaluated
        manager.set_focus("validation");
        let filtered = manager.filter_rules(&rules);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "Rule1");
    }

    #[test]
    fn test_activation_group_manager() {
        let mut manager = ActivationGroupManager::new();

        let rule1 = Rule::new("Rule1".to_string(), create_dummy_condition(), vec![])
            .with_activation_group("discount".to_string());
        let rule2 = Rule::new("Rule2".to_string(), create_dummy_condition(), vec![])
            .with_activation_group("discount".to_string());

        // Both rules can fire initially
        assert!(manager.can_fire(&rule1));
        assert!(manager.can_fire(&rule2));

        // After rule1 fires, rule2 cannot fire
        manager.mark_fired(&rule1);
        assert!(!manager.can_fire(&rule2));
        assert!(manager.has_group_fired("discount"));

        // After reset, both can fire again
        manager.reset_cycle();
        assert!(manager.can_fire(&rule1));
        assert!(manager.can_fire(&rule2));
    }

    #[test]
    fn test_lock_on_active() {
        let mut manager = AgendaManager::new();

        let rule = Rule::new("TestRule".to_string(), create_dummy_condition(), vec![])
            .with_lock_on_active(true);

        // Rule can fire initially (MAIN group not activated yet)
        assert!(manager.can_fire_rule(&rule));

        // Mark rule as fired - this should activate MAIN group and track the rule
        manager.mark_rule_fired(&rule);

        // Now rule cannot fire again in the same activation
        assert!(!manager.can_fire_rule(&rule));

        // After cycle reset, still cannot fire (lock-on-active persists until group change)
        manager.reset_cycle();
        assert!(!manager.can_fire_rule(&rule));

        // After switching to different group and back, rule can fire again
        manager.set_focus("validation");
        manager.set_focus("MAIN");
        assert!(manager.can_fire_rule(&rule));
    }
}

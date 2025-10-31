//! Advanced Agenda System (Drools-style)
//!
//! This module implements advanced agenda features similar to Drools:
//! - Activation Groups: Only one rule in a group can fire
//! - Agenda Groups: Sequential execution of rule groups
//! - Ruleflow Groups: Workflow-based execution
//! - Auto-focus: Automatic agenda group switching
//! - Lock-on-active: Prevent re-activation during rule firing

use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;

/// Activation represents a rule that is ready to fire
#[derive(Debug, Clone)]
pub struct Activation {
    /// Rule name
    pub rule_name: String,
    /// Priority/salience (higher fires first)
    pub salience: i32,
    /// Activation group (only one rule in group can fire)
    pub activation_group: Option<String>,
    /// Agenda group (for sequential execution)
    pub agenda_group: String,
    /// Ruleflow group (for workflow execution)
    pub ruleflow_group: Option<String>,
    /// No-loop flag
    pub no_loop: bool,
    /// Lock-on-active flag
    pub lock_on_active: bool,
    /// Auto-focus flag
    pub auto_focus: bool,
    /// Creation timestamp (for conflict resolution)
    pub created_at: std::time::Instant,
    /// Internal ID
    id: usize,
}

impl Activation {
    /// Create a new activation
    pub fn new(rule_name: String, salience: i32) -> Self {
        Self {
            rule_name,
            salience,
            activation_group: None,
            agenda_group: "MAIN".to_string(),
            ruleflow_group: None,
            no_loop: true,
            lock_on_active: false,
            auto_focus: false,
            created_at: std::time::Instant::now(),
            id: 0,
        }
    }

    /// Builder: Set activation group
    pub fn with_activation_group(mut self, group: String) -> Self {
        self.activation_group = Some(group);
        self
    }

    /// Builder: Set agenda group
    pub fn with_agenda_group(mut self, group: String) -> Self {
        self.agenda_group = group;
        self
    }

    /// Builder: Set ruleflow group
    pub fn with_ruleflow_group(mut self, group: String) -> Self {
        self.ruleflow_group = Some(group);
        self
    }

    /// Builder: Set no-loop
    pub fn with_no_loop(mut self, no_loop: bool) -> Self {
        self.no_loop = no_loop;
        self
    }

    /// Builder: Set lock-on-active
    pub fn with_lock_on_active(mut self, lock: bool) -> Self {
        self.lock_on_active = lock;
        self
    }

    /// Builder: Set auto-focus
    pub fn with_auto_focus(mut self, auto_focus: bool) -> Self {
        self.auto_focus = auto_focus;
        self
    }
}

// Implement ordering for priority queue (higher salience first)
impl PartialEq for Activation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Activation {}

impl PartialOrd for Activation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Activation {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by salience (higher is better)
        match self.salience.cmp(&other.salience) {
            Ordering::Equal => {
                // Then by creation time (earlier is better = reverse)
                other.created_at.cmp(&self.created_at)
            }
            other_order => other_order,
        }
    }
}

/// Advanced Agenda (Drools-style)
pub struct AdvancedAgenda {
    /// All activations by agenda group
    activations: HashMap<String, BinaryHeap<Activation>>,
    /// Current focus (agenda group)
    focus: String,
    /// Focus stack
    focus_stack: Vec<String>,
    /// Fired rules (for no-loop)
    fired_rules: HashSet<String>,
    /// Fired activation groups
    fired_activation_groups: HashSet<String>,
    /// Locked groups (lock-on-active)
    locked_groups: HashSet<String>,
    /// Active ruleflow groups
    active_ruleflow_groups: HashSet<String>,
    /// Next activation ID
    next_id: usize,
}

impl AdvancedAgenda {
    /// Create a new agenda with "MAIN" as default focus
    pub fn new() -> Self {
        let mut agenda = Self {
            activations: HashMap::new(),
            focus: "MAIN".to_string(),
            focus_stack: Vec::new(),
            fired_rules: HashSet::new(),
            fired_activation_groups: HashSet::new(),
            locked_groups: HashSet::new(),
            active_ruleflow_groups: HashSet::new(),
            next_id: 0,
        };
        agenda.activations.insert("MAIN".to_string(), BinaryHeap::new());
        agenda
    }

    /// Add an activation to the agenda
    pub fn add_activation(&mut self, mut activation: Activation) {
        // Auto-focus: switch to this agenda group if requested
        if activation.auto_focus && activation.agenda_group != self.focus {
            self.set_focus(activation.agenda_group.clone());
        }

        // Check activation group: if group already fired, skip
        if let Some(ref group) = activation.activation_group {
            if self.fired_activation_groups.contains(group) {
                return; // Skip this activation
            }
        }

        // Check ruleflow group: if not active, skip
        if let Some(ref group) = activation.ruleflow_group {
            if !self.active_ruleflow_groups.contains(group) {
                return; // Skip this activation
            }
        }

        // Assign ID
        activation.id = self.next_id;
        self.next_id += 1;

        // Add to appropriate agenda group
        self.activations
            .entry(activation.agenda_group.clone())
            .or_insert_with(BinaryHeap::new)
            .push(activation);
    }

    /// Get the next activation to fire (from current focus)
    pub fn get_next_activation(&mut self) -> Option<Activation> {
        loop {
            // Try to get from current focus
            if let Some(heap) = self.activations.get_mut(&self.focus) {
                while let Some(activation) = heap.pop() {
                    // Check no-loop
                    if activation.no_loop && self.fired_rules.contains(&activation.rule_name) {
                        continue;
                    }

                    // Check lock-on-active
                    if activation.lock_on_active && self.locked_groups.contains(&activation.agenda_group) {
                        continue;
                    }

                    // Check activation group
                    if let Some(ref group) = activation.activation_group {
                        if self.fired_activation_groups.contains(group) {
                            continue;
                        }
                    }

                    return Some(activation);
                }
            }

            // No more activations in current focus, try to pop focus stack
            if let Some(prev_focus) = self.focus_stack.pop() {
                self.focus = prev_focus;
            } else {
                return None; // Agenda is empty
            }
        }
    }

    /// Mark a rule as fired
    pub fn mark_rule_fired(&mut self, activation: &Activation) {
        self.fired_rules.insert(activation.rule_name.clone());

        // If has activation group, mark group as fired (no other rules in group can fire)
        if let Some(ref group) = activation.activation_group {
            self.fired_activation_groups.insert(group.clone());
        }

        // Lock the agenda group if lock-on-active
        if activation.lock_on_active {
            self.locked_groups.insert(activation.agenda_group.clone());
        }
    }

    /// Set focus to a specific agenda group
    pub fn set_focus(&mut self, group: String) {
        if group != self.focus {
            self.focus_stack.push(self.focus.clone());
            self.focus = group;
        }
    }

    /// Get current focus
    pub fn get_focus(&self) -> &str {
        &self.focus
    }

    /// Clear all agenda groups
    pub fn clear(&mut self) {
        self.activations.clear();
        self.activations.insert("MAIN".to_string(), BinaryHeap::new());
        self.focus = "MAIN".to_string();
        self.focus_stack.clear();
        self.fired_rules.clear();
        self.fired_activation_groups.clear();
        self.locked_groups.clear();
    }

    /// Reset fired flags (for re-evaluation)
    pub fn reset_fired_flags(&mut self) {
        self.fired_rules.clear();
        self.fired_activation_groups.clear();
        self.locked_groups.clear();
    }

    /// Activate a ruleflow group (make rules in this group eligible to fire)
    pub fn activate_ruleflow_group(&mut self, group: String) {
        self.active_ruleflow_groups.insert(group);
    }

    /// Deactivate a ruleflow group
    pub fn deactivate_ruleflow_group(&mut self, group: &str) {
        self.active_ruleflow_groups.remove(group);
    }

    /// Check if ruleflow group is active
    pub fn is_ruleflow_group_active(&self, group: &str) -> bool {
        self.active_ruleflow_groups.contains(group)
    }

    /// Get agenda statistics
    pub fn stats(&self) -> AgendaStats {
        let total_activations: usize = self.activations.values().map(|heap| heap.len()).sum();
        let groups = self.activations.len();

        AgendaStats {
            total_activations,
            groups,
            focus: self.focus.clone(),
            fired_rules: self.fired_rules.len(),
            fired_activation_groups: self.fired_activation_groups.len(),
            active_ruleflow_groups: self.active_ruleflow_groups.len(),
        }
    }
}

impl Default for AdvancedAgenda {
    fn default() -> Self {
        Self::new()
    }
}

/// Agenda statistics
#[derive(Debug, Clone)]
pub struct AgendaStats {
    pub total_activations: usize,
    pub groups: usize,
    pub focus: String,
    pub fired_rules: usize,
    pub fired_activation_groups: usize,
    pub active_ruleflow_groups: usize,
}

impl std::fmt::Display for AgendaStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Agenda Stats: {} activations, {} groups, focus='{}', {} fired rules",
            self.total_activations, self.groups, self.focus, self.fired_rules
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_activation() {
        let mut agenda = AdvancedAgenda::new();

        let act1 = Activation::new("Rule1".to_string(), 10);
        let act2 = Activation::new("Rule2".to_string(), 20);

        agenda.add_activation(act1);
        agenda.add_activation(act2);

        // Higher salience fires first
        let next = agenda.get_next_activation().unwrap();
        assert_eq!(next.rule_name, "Rule2");
    }

    #[test]
    fn test_activation_groups() {
        let mut agenda = AdvancedAgenda::new();

        let act1 = Activation::new("Rule1".to_string(), 10)
            .with_activation_group("group1".to_string());
        let act2 = Activation::new("Rule2".to_string(), 20)
            .with_activation_group("group1".to_string());

        agenda.add_activation(act1);
        agenda.add_activation(act2);

        // First activation fires
        let first = agenda.get_next_activation().unwrap();
        agenda.mark_rule_fired(&first);

        // Second activation should be skipped (same group)
        let second = agenda.get_next_activation();
        assert!(second.is_none());
    }

    #[test]
    fn test_agenda_groups() {
        let mut agenda = AdvancedAgenda::new();

        let act1 = Activation::new("Rule1".to_string(), 10)
            .with_agenda_group("group_a".to_string());
        let act2 = Activation::new("Rule2".to_string(), 20)
            .with_agenda_group("group_b".to_string());

        agenda.add_activation(act1);
        agenda.add_activation(act2);

        // MAIN is empty, nothing fires
        assert!(agenda.get_next_activation().is_none());

        // Set focus to group_a
        agenda.set_focus("group_a".to_string());
        let next = agenda.get_next_activation().unwrap();
        assert_eq!(next.rule_name, "Rule1");
    }

    #[test]
    fn test_auto_focus() {
        let mut agenda = AdvancedAgenda::new();

        let act = Activation::new("Rule1".to_string(), 10)
            .with_agenda_group("special".to_string())
            .with_auto_focus(true);

        agenda.add_activation(act);

        // Auto-focus should switch to "special"
        assert_eq!(agenda.get_focus(), "special");
    }

    #[test]
    fn test_ruleflow_groups() {
        let mut agenda = AdvancedAgenda::new();

        let act = Activation::new("Rule1".to_string(), 10)
            .with_ruleflow_group("flow1".to_string());

        // Without activating ruleflow group, activation is not added
        agenda.add_activation(act.clone());
        assert_eq!(agenda.stats().total_activations, 0);

        // Activate ruleflow group
        agenda.activate_ruleflow_group("flow1".to_string());
        agenda.add_activation(act);
        assert_eq!(agenda.stats().total_activations, 1);
    }
}

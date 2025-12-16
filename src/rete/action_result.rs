//! Action Results
//!
//! This module defines the results that actions can return to communicate
//! with the engine (e.g., retract facts, activate agenda groups, etc.)

use super::FactHandle;

/// Result of executing a rule action
/// Actions can return side effects that need to be handled by the engine
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Retract a fact by handle
    Retract(FactHandle),

    /// Retract a fact by fact type (first matching fact)
    RetractByType(String),

    /// Update/modify a fact (trigger re-evaluation)
    Update(FactHandle),

    /// Activate an agenda group
    ActivateAgendaGroup(String),

    /// Insert a new fact
    InsertFact {
        fact_type: String,
        data: super::TypedFacts,
    },

    /// Insert a logical fact (with justification)
    InsertLogicalFact {
        fact_type: String,
        data: super::TypedFacts,
        rule_name: String,
        premises: Vec<FactHandle>,
    },

    /// Call a custom function (requires function registry)
    CallFunction {
        function_name: String,
        args: Vec<String>,
    },

    /// Schedule a rule to fire later
    ScheduleRule { rule_name: String, delay_ms: u64 },

    /// No side effect, just modify facts
    None,
}

/// Container for multiple action results
#[derive(Debug, Clone, Default)]
pub struct ActionResults {
    pub results: Vec<ActionResult>,
}

impl ActionResults {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add(&mut self, result: ActionResult) {
        self.results.push(result);
    }

    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    pub fn len(&self) -> usize {
        self.results.len()
    }
}

//! Truth Maintenance System (TMS)
//!
//! This module implements a Truth Maintenance System similar to CLIPS and Drools.
//!
//! Key features:
//! - Justification tracking: Track WHY facts exist
//! - Logical assertions: Facts derived by rules vs. explicitly asserted
//! - Automatic retraction: Auto-retract derived facts when premises change
//! - Dependency chains: Track support relationships between facts
//!
//! Example:
//! ```ignore
//! // Rule derives a fact
//! rule "InferPremium" {
//!     when Customer.spent > 10000
//!     then logicalAssert(Customer.tier = "Premium");
//! }
//!
//! // When Customer.spent changes to 5000:
//! // → TMS auto-retracts Customer.tier = "Premium"
//! ```

use super::working_memory::FactHandle;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Type of justification for a fact
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JustificationType {
    /// Fact was explicitly asserted by user
    Explicit,
    /// Fact was logically derived by a rule
    Logical,
}

/// Justification records WHY a fact exists
///
/// A fact can have multiple justifications (multiple rules can derive the same fact).
/// The fact is only retracted when ALL justifications are removed.
#[derive(Debug, Clone)]
pub struct Justification {
    /// The fact being justified
    pub fact_handle: FactHandle,

    /// Type of justification
    pub justification_type: JustificationType,

    /// Rule that created this fact (if logical)
    pub source_rule: Option<String>,

    /// Premise facts that caused this derivation
    /// If any of these are retracted, this justification is invalid
    pub premise_facts: Vec<FactHandle>,

    /// When this justification was created
    pub created_at: Instant,

    /// Unique ID for this justification
    pub id: u64,
}

impl Justification {
    /// Create a new explicit justification (user asserted)
    pub fn explicit(fact_handle: FactHandle, id: u64) -> Self {
        Self {
            fact_handle,
            justification_type: JustificationType::Explicit,
            source_rule: None,
            premise_facts: Vec::new(),
            created_at: Instant::now(),
            id,
        }
    }

    /// Create a new logical justification (rule derived)
    pub fn logical(
        fact_handle: FactHandle,
        source_rule: String,
        premise_facts: Vec<FactHandle>,
        id: u64,
    ) -> Self {
        Self {
            fact_handle,
            justification_type: JustificationType::Logical,
            source_rule: Some(source_rule),
            premise_facts,
            created_at: Instant::now(),
            id,
        }
    }

    /// Check if this justification is still valid
    /// (all premise facts still exist)
    pub fn is_valid(&self, retracted_facts: &HashSet<FactHandle>) -> bool {
        // Explicit facts are always valid
        if self.justification_type == JustificationType::Explicit {
            return true;
        }

        // Logical facts are valid if all premises still exist
        !self
            .premise_facts
            .iter()
            .any(|h| retracted_facts.contains(h))
    }
}

/// Truth Maintenance System
///
/// Tracks justifications for facts and automatically maintains consistency
/// by retracting derived facts when their premises become invalid.
pub struct TruthMaintenanceSystem {
    /// All justifications, indexed by justification ID
    justifications: HashMap<u64, Justification>,

    /// Map: fact handle → list of justification IDs supporting it
    fact_justifications: HashMap<FactHandle, Vec<u64>>,

    /// Map: fact handle → list of justification IDs that depend on it (as premise)
    fact_dependents: HashMap<FactHandle, Vec<u64>>,

    /// Facts that are logically asserted (derived by rules)
    logical_facts: HashSet<FactHandle>,

    /// Facts that are explicitly asserted (by user)
    explicit_facts: HashSet<FactHandle>,

    /// Facts that have been retracted (for validation)
    retracted_facts: HashSet<FactHandle>,

    /// Next justification ID
    next_justification_id: u64,
}

impl TruthMaintenanceSystem {
    /// Create a new TMS
    pub fn new() -> Self {
        Self {
            justifications: HashMap::new(),
            fact_justifications: HashMap::new(),
            fact_dependents: HashMap::new(),
            logical_facts: HashSet::new(),
            explicit_facts: HashSet::new(),
            retracted_facts: HashSet::new(),
            next_justification_id: 1,
        }
    }

    /// Add an explicit justification (user asserted fact)
    pub fn add_explicit_justification(&mut self, fact_handle: FactHandle) {
        let id = self.next_justification_id;
        self.next_justification_id += 1;

        let justification = Justification::explicit(fact_handle, id);

        self.justifications.insert(id, justification);
        self.fact_justifications
            .entry(fact_handle)
            .or_default()
            .push(id);

        self.explicit_facts.insert(fact_handle);
    }

    /// Add a logical justification (rule derived fact)
    ///
    /// # Arguments
    /// * `fact_handle` - The derived fact
    /// * `source_rule` - Name of the rule that derived it
    /// * `premise_facts` - Facts that were matched in the rule's WHEN clause
    pub fn add_logical_justification(
        &mut self,
        fact_handle: FactHandle,
        source_rule: String,
        premise_facts: Vec<FactHandle>,
    ) {
        let id = self.next_justification_id;
        self.next_justification_id += 1;

        let justification =
            Justification::logical(fact_handle, source_rule, premise_facts.clone(), id);

        // Add to main storage
        self.justifications.insert(id, justification);

        // Index by fact
        self.fact_justifications
            .entry(fact_handle)
            .or_default()
            .push(id);

        // Index by premises (for dependency tracking)
        for premise in premise_facts {
            self.fact_dependents.entry(premise).or_default().push(id);
        }

        self.logical_facts.insert(fact_handle);
    }

    /// Check if a fact is logically asserted (derived by rules)
    pub fn is_logical(&self, fact_handle: FactHandle) -> bool {
        self.logical_facts.contains(&fact_handle)
    }

    /// Check if a fact is explicitly asserted (by user)
    pub fn is_explicit(&self, fact_handle: FactHandle) -> bool {
        self.explicit_facts.contains(&fact_handle)
    }

    /// Get all justifications for a fact
    pub fn get_justifications(&self, fact_handle: FactHandle) -> Vec<&Justification> {
        if let Some(just_ids) = self.fact_justifications.get(&fact_handle) {
            just_ids
                .iter()
                .filter_map(|id| self.justifications.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Check if a fact still has valid justifications
    pub fn has_valid_justification(&self, fact_handle: FactHandle) -> bool {
        if let Some(just_ids) = self.fact_justifications.get(&fact_handle) {
            just_ids
                .iter()
                .filter_map(|id| self.justifications.get(id))
                .any(|j| j.is_valid(&self.retracted_facts))
        } else {
            false
        }
    }

    /// Retract a fact and cascade to dependent facts
    ///
    /// Returns a list of facts that should be retracted due to cascade
    pub fn retract_with_cascade(&mut self, fact_handle: FactHandle) -> Vec<FactHandle> {
        let mut to_retract = Vec::new();

        // Mark this fact as retracted
        self.retracted_facts.insert(fact_handle);
        self.logical_facts.remove(&fact_handle);
        self.explicit_facts.remove(&fact_handle);

        // Find all justifications that depend on this fact
        if let Some(dependent_just_ids) = self.fact_dependents.get(&fact_handle) {
            for just_id in dependent_just_ids.clone() {
                if let Some(justification) = self.justifications.get(&just_id) {
                    let dependent_fact = justification.fact_handle;

                    // Check if the dependent fact still has other valid justifications
                    if !self.has_valid_justification(dependent_fact) {
                        // No valid justifications left → cascade retract
                        if !self.retracted_facts.contains(&dependent_fact) {
                            to_retract.push(dependent_fact);

                            // Recursive cascade
                            let cascaded = self.retract_with_cascade(dependent_fact);
                            to_retract.extend(cascaded);
                        }
                    }
                }
            }
        }

        to_retract
    }

    /// Remove all justifications for a fact
    pub fn remove_justifications(&mut self, fact_handle: FactHandle) {
        if let Some(just_ids) = self.fact_justifications.remove(&fact_handle) {
            for id in just_ids {
                self.justifications.remove(&id);
            }
        }
    }

    /// Get statistics about the TMS
    pub fn stats(&self) -> TmsStats {
        TmsStats {
            total_justifications: self.justifications.len(),
            logical_facts: self.logical_facts.len(),
            explicit_facts: self.explicit_facts.len(),
            retracted_facts: self.retracted_facts.len(),
        }
    }

    /// Clear all TMS data
    pub fn clear(&mut self) {
        self.justifications.clear();
        self.fact_justifications.clear();
        self.fact_dependents.clear();
        self.logical_facts.clear();
        self.explicit_facts.clear();
        self.retracted_facts.clear();
        self.next_justification_id = 1;
    }

    /// Get all logical facts
    pub fn get_logical_facts(&self) -> &HashSet<FactHandle> {
        &self.logical_facts
    }

    /// Get all explicit facts
    pub fn get_explicit_facts(&self) -> &HashSet<FactHandle> {
        &self.explicit_facts
    }
}

impl Default for TruthMaintenanceSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// TMS statistics
#[derive(Debug, Clone)]
pub struct TmsStats {
    pub total_justifications: usize,
    pub logical_facts: usize,
    pub explicit_facts: usize,
    pub retracted_facts: usize,
}

impl std::fmt::Display for TmsStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TMS Stats: {} justifications, {} logical facts, {} explicit facts, {} retracted",
            self.total_justifications,
            self.logical_facts,
            self.explicit_facts,
            self.retracted_facts
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_justification() {
        let mut tms = TruthMaintenanceSystem::new();
        let fact = FactHandle::new(1);

        tms.add_explicit_justification(fact);

        assert!(tms.is_explicit(fact));
        assert!(!tms.is_logical(fact));
        assert!(tms.has_valid_justification(fact));
    }

    #[test]
    fn test_logical_justification() {
        let mut tms = TruthMaintenanceSystem::new();
        let premise = FactHandle::new(1);
        let derived = FactHandle::new(2);

        tms.add_explicit_justification(premise);
        tms.add_logical_justification(derived, "TestRule".to_string(), vec![premise]);

        assert!(tms.is_logical(derived));
        assert!(!tms.is_explicit(derived));
        assert!(tms.has_valid_justification(derived));
    }

    #[test]
    fn test_cascade_retraction() {
        let mut tms = TruthMaintenanceSystem::new();

        // Setup: Fact A → Rule → Fact B → Rule → Fact C
        let fact_a = FactHandle::new(1);
        let fact_b = FactHandle::new(2);
        let fact_c = FactHandle::new(3);

        tms.add_explicit_justification(fact_a);
        tms.add_logical_justification(fact_b, "Rule1".to_string(), vec![fact_a]);
        tms.add_logical_justification(fact_c, "Rule2".to_string(), vec![fact_b]);

        // Retract A → should cascade to B and C
        let cascaded = tms.retract_with_cascade(fact_a);

        assert!(cascaded.contains(&fact_b));
        assert!(cascaded.contains(&fact_c));
        assert!(!tms.has_valid_justification(fact_b));
        assert!(!tms.has_valid_justification(fact_c));
    }

    #[test]
    fn test_multiple_justifications() {
        let mut tms = TruthMaintenanceSystem::new();

        let premise1 = FactHandle::new(1);
        let premise2 = FactHandle::new(2);
        let derived = FactHandle::new(3);

        tms.add_explicit_justification(premise1);
        tms.add_explicit_justification(premise2);

        // Derived fact has TWO justifications
        tms.add_logical_justification(derived, "Rule1".to_string(), vec![premise1]);
        tms.add_logical_justification(derived, "Rule2".to_string(), vec![premise2]);

        // Retract one premise
        let cascaded = tms.retract_with_cascade(premise1);

        // Derived fact should STILL be valid (has justification from premise2)
        assert!(tms.has_valid_justification(derived));
        assert!(cascaded.is_empty());

        // Retract second premise
        let cascaded = tms.retract_with_cascade(premise2);

        // NOW the derived fact should be retracted
        assert!(!tms.has_valid_justification(derived));
        assert!(cascaded.contains(&derived));
    }
}

//! Incremental Propagation Engine (P3 Feature - Advanced)
//!
//! This module implements incremental updates similar to Drools:
//! - Only propagate changed facts through the network
//! - Track affected rules and activations
//! - Efficient re-evaluation after updates

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use super::working_memory::{WorkingMemory, FactHandle};
use super::network::{ReteUlNode, TypedReteUlRule};
use super::facts::{TypedFacts, FactValue};
use super::agenda::{AdvancedAgenda, Activation};
use super::template::TemplateRegistry;
use super::globals::GlobalsRegistry;
use super::deffacts::DeffactsRegistry;
use crate::errors::{Result, RuleEngineError};

/// Track which rules are affected by which fact types
#[derive(Debug)]
pub struct RuleDependencyGraph {
    /// Map: fact_type -> set of rule indices that depend on it
    fact_type_to_rules: HashMap<String, HashSet<usize>>,
    /// Map: rule index -> set of fact types it depends on
    rule_to_fact_types: HashMap<usize, HashSet<String>>,
}

impl RuleDependencyGraph {
    /// Create new dependency graph
    pub fn new() -> Self {
        Self {
            fact_type_to_rules: HashMap::new(),
            rule_to_fact_types: HashMap::new(),
        }
    }

    /// Add dependency: rule depends on fact type
    pub fn add_dependency(&mut self, rule_idx: usize, fact_type: String) {
        self.fact_type_to_rules
            .entry(fact_type.clone())
            .or_insert_with(HashSet::new)
            .insert(rule_idx);

        self.rule_to_fact_types
            .entry(rule_idx)
            .or_insert_with(HashSet::new)
            .insert(fact_type);
    }

    /// Get rules affected by a fact type change
    pub fn get_affected_rules(&self, fact_type: &str) -> HashSet<usize> {
        self.fact_type_to_rules
            .get(fact_type)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }

    /// Get fact types that a rule depends on
    pub fn get_rule_dependencies(&self, rule_idx: usize) -> HashSet<String> {
        self.rule_to_fact_types
            .get(&rule_idx)
            .cloned()
            .unwrap_or_else(HashSet::new)
    }
}

impl Default for RuleDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for custom test functions in RETE engine
/// Functions take a slice of FactValues and return a FactValue (typically Boolean)
pub type ReteCustomFunction = Arc<dyn Fn(&[FactValue], &TypedFacts) -> Result<FactValue> + Send + Sync>;

/// Incremental Propagation Engine
/// Only re-evaluates rules affected by changed facts
pub struct IncrementalEngine {
    /// Working memory
    working_memory: WorkingMemory,
    /// Rules
    rules: Vec<TypedReteUlRule>,
    /// Dependency graph
    dependencies: RuleDependencyGraph,
    /// Advanced agenda
    agenda: AdvancedAgenda,
    /// Track which facts each rule last matched
    rule_matched_facts: HashMap<usize, HashSet<FactHandle>>,
    /// Template registry for type-safe facts
    templates: TemplateRegistry,
    /// Global variables registry
    globals: GlobalsRegistry,
    /// Deffacts registry for initial facts
    deffacts: DeffactsRegistry,
    /// Custom functions for Test CE support
    custom_functions: HashMap<String, ReteCustomFunction>,
}

impl IncrementalEngine {
    /// Create new incremental engine
    pub fn new() -> Self {
        Self {
            working_memory: WorkingMemory::new(),
            rules: Vec::new(),
            dependencies: RuleDependencyGraph::new(),
            agenda: AdvancedAgenda::new(),
            rule_matched_facts: HashMap::new(),
            custom_functions: HashMap::new(),
            templates: TemplateRegistry::new(),
            globals: GlobalsRegistry::new(),
            deffacts: DeffactsRegistry::new(),
        }
    }

    /// Add rule and register its dependencies
    pub fn add_rule(&mut self, rule: TypedReteUlRule, depends_on: Vec<String>) {
        let rule_idx = self.rules.len();

        // Register dependencies
        for fact_type in depends_on {
            self.dependencies.add_dependency(rule_idx, fact_type);
        }

        self.rules.push(rule);
    }

    /// Insert fact into working memory
    pub fn insert(&mut self, fact_type: String, data: TypedFacts) -> FactHandle {
        let handle = self.working_memory.insert(fact_type.clone(), data);

        // Trigger incremental propagation for this fact type
        self.propagate_changes_for_type(&fact_type);

        handle
    }

    /// Update fact in working memory
    pub fn update(&mut self, handle: FactHandle, data: TypedFacts) -> Result<()> {
        // Get fact type before update
        let fact_type = self.working_memory
            .get(&handle)
            .map(|f| f.fact_type.clone())
            .ok_or_else(|| RuleEngineError::FieldNotFound {
                field: format!("FactHandle {} not found", handle),
            })?;

        self.working_memory.update(handle, data).map_err(|e| RuleEngineError::EvaluationError {
            message: e,
        })?;

        // Trigger incremental propagation for this fact type
        self.propagate_changes_for_type(&fact_type);

        Ok(())
    }

    /// Retract fact from working memory
    pub fn retract(&mut self, handle: FactHandle) -> Result<()> {
        // Get fact type before retract
        let fact_type = self.working_memory
            .get(&handle)
            .map(|f| f.fact_type.clone())
            .ok_or_else(|| RuleEngineError::FieldNotFound {
                field: format!("FactHandle {} not found", handle),
            })?;

        self.working_memory.retract(handle).map_err(|e| RuleEngineError::EvaluationError {
            message: e,
        })?;

        // Trigger incremental propagation for this fact type
        self.propagate_changes_for_type(&fact_type);

        Ok(())
    }

    /// Propagate changes for a specific fact type (incremental!)
    fn propagate_changes_for_type(&mut self, fact_type: &str) {
        // Get affected rules
        let affected_rules = self.dependencies.get_affected_rules(fact_type);

        if affected_rules.is_empty() {
            return; // No rules depend on this fact type
        }

        // Flatten working memory to TypedFacts for evaluation
        let facts = self.working_memory.to_typed_facts();

        // Re-evaluate only affected rules
        for &rule_idx in &affected_rules {
            let rule = &self.rules[rule_idx];

            // Evaluate rule condition
            let matches = super::network::evaluate_rete_ul_node_typed(&rule.node, &facts);

            if matches {
                // Create activation
                let activation = Activation::new(rule.name.clone(), rule.priority)
                    .with_no_loop(rule.no_loop);

                self.agenda.add_activation(activation);
            }
        }
    }

    /// Fire all pending activations
    pub fn fire_all(&mut self) -> Vec<String> {
        let mut fired_rules = Vec::new();

        while let Some(activation) = self.agenda.get_next_activation() {
            // Find rule
            if let Some((idx, rule)) = self.rules
                .iter_mut()
                .enumerate()
                .find(|(_, r)| r.name == activation.rule_name)
            {
                // Execute action
                let mut facts = self.working_memory.to_typed_facts();
                (rule.action)(&mut facts);

                // Track fired rule
                fired_rules.push(activation.rule_name.clone());
                self.agenda.mark_rule_fired(&activation);

                // TODO: Update working memory with changed facts
                // This is complex and would require tracking what changed
            }
        }

        fired_rules
    }

    /// Get working memory
    pub fn working_memory(&self) -> &WorkingMemory {
        &self.working_memory
    }

    /// Get mutable working memory
    pub fn working_memory_mut(&mut self) -> &mut WorkingMemory {
        &mut self.working_memory
    }

    /// Get agenda
    pub fn agenda(&self) -> &AdvancedAgenda {
        &self.agenda
    }

    /// Get mutable agenda
    pub fn agenda_mut(&mut self) -> &mut AdvancedAgenda {
        &mut self.agenda
    }

    /// Set conflict resolution strategy
    ///
    /// Controls how conflicting rules in the agenda are ordered.
    /// Available strategies: Salience (default), LEX, MEA, Depth, Breadth, Simplicity, Complexity, Random
    pub fn set_conflict_resolution_strategy(
        &mut self,
        strategy: super::agenda::ConflictResolutionStrategy,
    ) {
        self.agenda.set_strategy(strategy);
    }

    /// Get current conflict resolution strategy
    pub fn conflict_resolution_strategy(&self) -> super::agenda::ConflictResolutionStrategy {
        self.agenda.strategy()
    }

    /// Get statistics
    pub fn stats(&self) -> IncrementalEngineStats {
        IncrementalEngineStats {
            rules: self.rules.len(),
            working_memory: self.working_memory.stats(),
            agenda: self.agenda.stats(),
            dependencies: self.dependencies.fact_type_to_rules.len(),
        }
    }

    /// Clear fired flags and reset agenda
    pub fn reset(&mut self) {
        self.agenda.reset_fired_flags();
    }

    /// Get template registry
    pub fn templates(&self) -> &TemplateRegistry {
        &self.templates
    }

    /// Get mutable template registry
    pub fn templates_mut(&mut self) -> &mut TemplateRegistry {
        &mut self.templates
    }

    /// Register a custom function for Test CE support
    ///
    /// # Example
    /// ```
    /// use rust_rule_engine::rete::{IncrementalEngine, FactValue};
    ///
    /// let mut engine = IncrementalEngine::new();
    /// engine.register_function(
    ///     "is_valid_email",
    ///     |args, _facts| {
    ///         if let Some(FactValue::String(email)) = args.first() {
    ///             Ok(FactValue::Boolean(email.contains('@')))
    ///         } else {
    ///             Ok(FactValue::Boolean(false))
    ///         }
    ///     }
    /// );
    /// ```
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[FactValue], &TypedFacts) -> Result<FactValue> + Send + Sync + 'static,
    {
        self.custom_functions.insert(name.to_string(), Arc::new(func));
    }

    /// Get a custom function by name (for Test CE evaluation)
    pub fn get_function(&self, name: &str) -> Option<&ReteCustomFunction> {
        self.custom_functions.get(name)
    }

    /// Get global variables registry
    pub fn globals(&self) -> &GlobalsRegistry {
        &self.globals
    }

    /// Get mutable global variables registry
    pub fn globals_mut(&mut self) -> &mut GlobalsRegistry {
        &mut self.globals
    }

    /// Get deffacts registry
    pub fn deffacts(&self) -> &DeffactsRegistry {
        &self.deffacts
    }

    /// Get mutable deffacts registry
    pub fn deffacts_mut(&mut self) -> &mut DeffactsRegistry {
        &mut self.deffacts
    }

    /// Load all registered deffacts into working memory
    /// Returns handles of all inserted facts
    pub fn load_deffacts(&mut self) -> Vec<FactHandle> {
        let mut handles = Vec::new();

        // Get all facts from all registered deffacts
        let all_facts = self.deffacts.get_all_facts();

        for (_deffacts_name, fact_instance) in all_facts {
            // Check if template exists for this fact type
            let handle = if self.templates.get(&fact_instance.fact_type).is_some() {
                // Use template validation
                match self.insert_with_template(&fact_instance.fact_type, fact_instance.data) {
                    Ok(h) => h,
                    Err(_) => continue, // Skip invalid facts
                }
            } else {
                // Insert without template validation
                self.insert(fact_instance.fact_type, fact_instance.data)
            };

            handles.push(handle);
        }

        handles
    }

    /// Load a specific deffacts set by name
    /// Returns handles of inserted facts or error if deffacts not found
    pub fn load_deffacts_by_name(&mut self, name: &str) -> crate::errors::Result<Vec<FactHandle>> {
        // Clone the facts to avoid borrow checker issues
        let facts_to_insert = {
            let deffacts = self.deffacts.get(name).ok_or_else(|| {
                crate::errors::RuleEngineError::EvaluationError {
                    message: format!("Deffacts '{}' not found", name),
                }
            })?;
            deffacts.facts.clone()
        };

        let mut handles = Vec::new();

        for fact_instance in facts_to_insert {
            // Check if template exists for this fact type
            let handle = if self.templates.get(&fact_instance.fact_type).is_some() {
                // Use template validation
                self.insert_with_template(&fact_instance.fact_type, fact_instance.data)?
            } else {
                // Insert without template validation
                self.insert(fact_instance.fact_type, fact_instance.data)
            };

            handles.push(handle);
        }

        Ok(handles)
    }

    /// Reset engine and reload all deffacts (similar to CLIPS reset)
    /// Clears working memory and agenda, then loads all deffacts
    pub fn reset_with_deffacts(&mut self) -> Vec<FactHandle> {
        // Clear working memory and agenda
        self.working_memory = WorkingMemory::new();
        self.agenda.clear();
        self.rule_matched_facts.clear();

        // Reload all deffacts
        self.load_deffacts()
    }

    /// Insert a typed fact with template validation
    pub fn insert_with_template(
        &mut self,
        template_name: &str,
        data: TypedFacts,
    ) -> crate::errors::Result<FactHandle> {
        // Validate against template
        self.templates.validate(template_name, &data)?;

        // Insert into working memory
        Ok(self.insert(template_name.to_string(), data))
    }
}

impl Default for IncrementalEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Engine statistics
#[derive(Debug)]
pub struct IncrementalEngineStats {
    pub rules: usize,
    pub working_memory: super::working_memory::WorkingMemoryStats,
    pub agenda: super::agenda::AgendaStats,
    pub dependencies: usize,
}

impl std::fmt::Display for IncrementalEngineStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Engine Stats: {} rules, {} fact types tracked\nWM: {}\nAgenda: {}",
            self.rules,
            self.dependencies,
            self.working_memory,
            self.agenda
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rete::network::ReteUlNode;
    use crate::rete::alpha::AlphaNode;

    #[test]
    fn test_dependency_graph() {
        let mut graph = RuleDependencyGraph::new();

        graph.add_dependency(0, "Person".to_string());
        graph.add_dependency(1, "Person".to_string());
        graph.add_dependency(1, "Order".to_string());

        let affected = graph.get_affected_rules("Person");
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&0));
        assert!(affected.contains(&1));

        let deps = graph.get_rule_dependencies(1);
        assert_eq!(deps.len(), 2);
        assert!(deps.contains("Person"));
        assert!(deps.contains("Order"));
    }

    #[test]
    fn test_incremental_propagation() {
        let mut engine = IncrementalEngine::new();

        // Add rule that depends on "Person" type
        let node = ReteUlNode::UlAlpha(AlphaNode {
            field: "Person.age".to_string(),
            operator: ">".to_string(),
            value: "18".to_string(),
        });

        let rule = TypedReteUlRule {
            name: "IsAdult".to_string(),
            node,
            priority: 0,
            no_loop: true,
            action: Box::new(|_| {}),
        };

        engine.add_rule(rule, vec!["Person".to_string()]);

        // Insert Person fact
        let mut person = TypedFacts::new();
        person.set("age", 25i64);
        let handle = engine.insert("Person".to_string(), person);

        // Check that rule was activated
        let stats = engine.stats();
        assert!(stats.agenda.total_activations > 0);

        // Update person
        let mut updated = TypedFacts::new();
        updated.set("age", 15i64); // Now under 18
        engine.update(handle, updated).unwrap();

        // Rule should be re-evaluated (incrementally)
    }
}

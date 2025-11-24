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
use super::tms::TruthMaintenanceSystem;
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
    /// Truth Maintenance System
    tms: TruthMaintenanceSystem,
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
            tms: TruthMaintenanceSystem::new(),
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
        
        // Default: Treat as explicit assertion (backward compatible)
        self.tms.add_explicit_justification(handle);

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

        // TMS: Handle cascade retraction
        let cascaded_facts = self.tms.retract_with_cascade(handle);
        
        // Actually retract cascaded facts from working memory
        for cascaded_handle in cascaded_facts {
            if let Ok(fact_type) = self.working_memory
                .get(&cascaded_handle)
                .map(|f| f.fact_type.clone())
                .ok_or_else(|| RuleEngineError::FieldNotFound {
                    field: format!("FactHandle {} not found", cascaded_handle),
                })
            {
                let _ = self.working_memory.retract(cascaded_handle);
                // Propagate for each cascaded fact
                self.propagate_changes_for_type(&fact_type);
            }
        }

        // Trigger incremental propagation for this fact type
        self.propagate_changes_for_type(&fact_type);

        Ok(())
    }
    
    /// Insert a fact with explicit assertion (user provided)
    /// This fact will NOT be auto-retracted by TMS
    pub fn insert_explicit(&mut self, fact_type: String, data: TypedFacts) -> FactHandle {
        let handle = self.working_memory.insert(fact_type.clone(), data);
        
        // Add explicit justification in TMS
        self.tms.add_explicit_justification(handle);
        
        // Trigger incremental propagation for this fact type
        self.propagate_changes_for_type(&fact_type);
        
        handle
    }
    
    /// Insert a fact with logical assertion (derived by a rule)
    /// This fact WILL be auto-retracted if its premises become invalid
    ///
    /// # Arguments
    /// * `fact_type` - Type of the fact (e.g., "Customer")
    /// * `data` - The fact data
    /// * `source_rule` - Name of the rule deriving this fact
    /// * `premise_handles` - Handles of facts matched in the rule's WHEN clause
    pub fn insert_logical(
        &mut self,
        fact_type: String,
        data: TypedFacts,
        source_rule: String,
        premise_handles: Vec<FactHandle>,
    ) -> FactHandle {
        let handle = self.working_memory.insert(fact_type.clone(), data);
        
        // Add logical justification in TMS
        self.tms.add_logical_justification(handle, source_rule, premise_handles);
        
        // Trigger incremental propagation for this fact type
        self.propagate_changes_for_type(&fact_type);
        
        handle
    }
    
    /// Get TMS reference
    pub fn tms(&self) -> &TruthMaintenanceSystem {
        &self.tms
    }
    
    /// Get mutable TMS reference
    pub fn tms_mut(&mut self) -> &mut TruthMaintenanceSystem {
        &mut self.tms
    }

    /// Propagate changes for a specific fact type (incremental!)
    fn propagate_changes_for_type(&mut self, fact_type: &str) {
        // Get affected rules
        let affected_rules = self.dependencies.get_affected_rules(fact_type);

        if affected_rules.is_empty() {
            return; // No rules depend on this fact type
        }

        // Get facts of this type
        let facts_of_type = self.working_memory.get_by_type(fact_type);
        
        // Re-evaluate only affected rules, checking each fact individually
        for &rule_idx in &affected_rules {
            let rule = &self.rules[rule_idx];

            // Check each fact of this type against the rule
            for fact in &facts_of_type {
                // Create TypedFacts for just this fact
                let mut single_fact_data = TypedFacts::new();
                for (key, value) in fact.data.get_all() {
                    single_fact_data.set(format!("{}.{}", fact_type, key), value.clone());
                }
                // Store handle for Retract action
                single_fact_data.set_fact_handle(fact_type.to_string(), fact.handle);
                
                // Evaluate rule condition with this single fact
                let matches = super::network::evaluate_rete_ul_node_typed(&rule.node, &single_fact_data);

                if matches {
                    // Create activation for this specific fact match
                    let activation = Activation::new(rule.name.clone(), rule.priority)
                        .with_no_loop(rule.no_loop)
                        .with_matched_fact(fact.handle);

                    self.agenda.add_activation(activation);
                }
            }
        }
    }

    /// Propagate changes for all fact types (re-evaluate all rules)
    fn propagate_changes(&mut self) {
        // Get all fact types
        let fact_types: Vec<String> = self.working_memory.get_all_facts()
            .iter()
            .map(|f| f.fact_type.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        // Evaluate each fact type using per-fact evaluation
        for fact_type in fact_types {
            let facts_of_type = self.working_memory.get_by_type(&fact_type);
            
            for (rule_idx, rule) in self.rules.iter().enumerate() {
                // Skip if rule has no-loop and already fired
                if rule.no_loop && self.agenda.has_fired(&rule.name) {
                    continue;
                }
                
                // Check each fact against the rule
                for fact in &facts_of_type {
                    let mut single_fact_data = TypedFacts::new();
                    for (key, value) in fact.data.get_all() {
                        single_fact_data.set(format!("{}.{}", fact_type, key), value.clone());
                    }
                    
                    let matches = super::network::evaluate_rete_ul_node_typed(&rule.node, &single_fact_data);
                    
                    if matches {
                        let activation = Activation::new(rule.name.clone(), rule.priority)
                            .with_no_loop(rule.no_loop)
                            .with_matched_fact(fact.handle);
                        
                        self.agenda.add_activation(activation);
                    }
                }
            }
        }
    }

    /// Fire all pending activations
    pub fn fire_all(&mut self) -> Vec<String> {
        let mut fired_rules = Vec::new();
        let max_iterations = 1000; // Prevent infinite loops
        let mut iteration_count = 0;

        while let Some(activation) = self.agenda.get_next_activation() {
            iteration_count += 1;
            if iteration_count > max_iterations {
                eprintln!("WARNING: Maximum iterations ({}) reached in fire_all(). Possible infinite loop!", max_iterations);
                break;
            }
            
            // Find rule
            if let Some((idx, rule)) = self.rules
                .iter_mut()
                .enumerate()
                .find(|(_, r)| r.name == activation.rule_name)
            {
                // Validate that matched fact still exists (hasn't been retracted)
                if let Some(matched_handle) = activation.matched_fact_handle {
                    if self.working_memory.get(&matched_handle).is_none() {
                        // Fact was retracted, skip this activation
                        continue;
                    }
                }
                
                // Execute action on a copy of all facts
                let original_facts = self.working_memory.to_typed_facts();
                let mut modified_facts = original_facts.clone();
                
                // Inject matched fact handle if available
                if let Some(matched_handle) = activation.matched_fact_handle {
                    // Find the fact type from working memory
                    if let Some(fact) = self.working_memory.get(&matched_handle) {
                        modified_facts.set_fact_handle(fact.fact_type.clone(), matched_handle);
                    }
                }
                
                let mut action_results = super::ActionResults::new();
                (rule.action)(&mut modified_facts, &mut action_results);

                // Update working memory: detect changes and apply them
                // Build a map of fact_type -> updated fields
                let mut updates_by_type: HashMap<String, Vec<(String, FactValue)>> = HashMap::new();
                
                for (key, value) in modified_facts.get_all() {
                    // Keys are in format "FactType.field" or "FactType.handle.field"
                    // We want to extract the FactType and field name
                    if let Some(original_value) = original_facts.get(key) {
                        if original_value != value {
                            // Value changed! Extract fact type and field
                            let parts: Vec<&str> = key.split('.').collect();
                            if parts.len() >= 2 {
                                let fact_type = parts[0].to_string();
                                // Field is the last part (skip handle if present)
                                let field = if parts.len() == 2 {
                                    parts[1].to_string()
                                } else {
                                    parts[parts.len() - 1].to_string()
                                };
                                
                                updates_by_type
                                    .entry(fact_type)
                                    .or_insert_with(Vec::new)
                                    .push((field, value.clone()));
                            }
                        }
                    } else {
                        // New field added
                        let parts: Vec<&str> = key.split('.').collect();
                        if parts.len() >= 2 {
                            let fact_type = parts[0].to_string();
                            let field = if parts.len() == 2 {
                                parts[1].to_string()
                            } else {
                                parts[parts.len() - 1].to_string()
                            };
                            
                            updates_by_type
                                .entry(fact_type)
                                .or_insert_with(Vec::new)
                                .push((field, value.clone()));
                        }
                    }
                }
                
                // Apply updates to working memory facts
                for (fact_type, field_updates) in updates_by_type {
                    // Get handles of all facts of this type (collect to avoid borrow issues)
                    let fact_handles: Vec<FactHandle> = self.working_memory
                        .get_by_type(&fact_type)
                        .iter()
                        .map(|f| f.handle)
                        .collect();
                    
                    for handle in fact_handles {
                        if let Some(fact) = self.working_memory.get(&handle) {
                            let mut updated_data = fact.data.clone();
                            
                            // Apply all field updates
                            for (field, value) in &field_updates {
                                updated_data.set(field, value.clone());
                            }
                            
                            let _ = self.working_memory.update(handle, updated_data);
                        }
                    }
                }

                // Re-evaluate matches after working memory update
                // This allows subsequent rules to see the updated values
                self.propagate_changes();

                // Process action results (retractions, agenda activations, etc.)
                self.process_action_results(action_results);

                // Track fired rule
                fired_rules.push(activation.rule_name.clone());
                self.agenda.mark_rule_fired(&activation);
            }
        }

        fired_rules
    }

    /// Process action results from rule execution
    fn process_action_results(&mut self, results: super::ActionResults) {
        for result in results.results {
            match result {
                super::ActionResult::Retract(handle) => {
                    // Retract fact by handle
                    if let Err(e) = self.retract(handle) {
                        eprintln!("❌ Failed to retract fact {:?}: {}", handle, e);
                    }
                }
                super::ActionResult::RetractByType(fact_type) => {
                    // Retract first fact of this type
                    let facts_of_type = self.working_memory.get_by_type(&fact_type);
                    if let Some(fact) = facts_of_type.first() {
                        let handle = fact.handle;
                        if let Err(e) = self.retract(handle) {
                            eprintln!("❌ Failed to retract fact {:?}: {}", handle, e);
                        }
                    }
                }
                super::ActionResult::Update(handle) => {
                    // Re-evaluate rules that depend on this fact type
                    if let Some(fact) = self.working_memory.get(&handle) {
                        let fact_type = fact.fact_type.clone();
                        self.propagate_changes_for_type(&fact_type);
                    }
                }
                super::ActionResult::ActivateAgendaGroup(group) => {
                    // Activate agenda group
                    self.agenda.set_focus(group);
                }
                super::ActionResult::InsertFact { fact_type, data } => {
                    // Insert new explicit fact
                    self.insert_explicit(fact_type, data);
                }
                super::ActionResult::InsertLogicalFact { fact_type, data, rule_name, premises } => {
                    // Insert new logical fact
                    let _handle = self.insert_logical(fact_type, data, rule_name, premises);
                }
                super::ActionResult::CallFunction { function_name, args } => {
                    // Log function calls (requires function registry to actually execute)
                    println!("🔧 Function call queued: {}({:?})", function_name, args);
                    // TODO: Implement function registry
                }
                super::ActionResult::ScheduleRule { rule_name, delay_ms } => {
                    // Log scheduled rules (requires scheduler to actually execute)
                    println!("⏰ Rule scheduled: {} after {}ms", rule_name, delay_ms);
                    // TODO: Implement rule scheduler
                }
                super::ActionResult::None => {
                    // No action needed
                }
            }
        }
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
            action: std::sync::Arc::new(|_, _| {}),
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

//! Incremental Propagation Engine (P3 Feature - Advanced)
//!
//! This module implements incremental updates similar to Drools:
//! - Only propagate changed facts through the network
//! - Track affected rules and activations
//! - Efficient re-evaluation after updates

use super::agenda::{Activation, AdvancedAgenda};
use super::deffacts::DeffactsRegistry;
use super::facts::{FactValue, TypedFacts};
use super::globals::GlobalsRegistry;
use super::network::TypedReteUlRule;
use super::template::TemplateRegistry;
use super::tms::TruthMaintenanceSystem;
use super::working_memory::{FactHandle, WorkingMemory};
use crate::errors::{Result, RuleEngineError};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

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
            .or_default()
            .insert(rule_idx);

        self.rule_to_fact_types
            .entry(rule_idx)
            .or_default()
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
pub type ReteCustomFunction =
    Arc<dyn Fn(&[FactValue], &TypedFacts) -> Result<FactValue> + Send + Sync>;

#[derive(Debug, Clone)]
struct ScheduledRuleTask {
    rule_name: String,
    execute_at: Instant,
}

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
    /// Scheduled rule executions waiting for their delay to expire
    scheduled_rules: Vec<ScheduledRuleTask>,
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
            scheduled_rules: Vec::new(),
        }
    }

    fn apply_fact_updates(&mut self, original_facts: &TypedFacts, modified_facts: &TypedFacts) {
        let mut updates_by_type: HashMap<String, Vec<(String, FactValue)>> = HashMap::new();

        for (key, value) in modified_facts.get_all() {
            if original_facts.get(key) == Some(value) {
                continue;
            }

            let parts: Vec<&str> = key.split('.').collect();
            if parts.len() < 2 {
                continue;
            }

            let fact_type = parts[0].to_string();
            let field = parts[parts.len() - 1].to_string();

            updates_by_type
                .entry(fact_type)
                .or_default()
                .push((field, value.clone()));
        }

        if updates_by_type.is_empty() {
            return;
        }

        for (fact_type, field_updates) in updates_by_type {
            let fact_handles: Vec<FactHandle> = self
                .working_memory
                .get_by_type(&fact_type)
                .iter()
                .map(|f| f.handle)
                .collect();

            for handle in fact_handles {
                if let Some(fact) = self.working_memory.get(&handle) {
                    let mut updated_data = fact.data.clone();

                    for (field, value) in &field_updates {
                        updated_data.set(field, value.clone());
                    }

                    let _ = self.working_memory.update(handle, updated_data);
                }
            }
        }
        self.propagate_changes();
    }

    fn schedule_rule(&mut self, rule_name: String, delay_ms: u64) {
        self.scheduled_rules.push(ScheduledRuleTask {
            rule_name,
            execute_at: Instant::now() + Duration::from_millis(delay_ms),
        });
    }

    fn enqueue_rule_activations_by_name(&mut self, rule_name: &str) -> bool {
        let Some((rule_idx, rule)) = self
            .rules
            .iter()
            .enumerate()
            .find(|(_, rule)| rule.name == rule_name)
        else {
            eprintln!("❌ Scheduled rule '{}' not found", rule_name);
            return false;
        };

        let rule_name = rule.name.clone();
        let priority = rule.priority;
        let no_loop = rule.no_loop;
        let node = rule.node.clone();
        let dependent_types = self.dependencies.get_rule_dependencies(rule_idx);

        if dependent_types.is_empty() {
            let facts = self.working_memory.to_typed_facts();
            if super::network::evaluate_rete_ul_node_typed(&node, &facts) {
                let activation = Activation::new(rule_name, priority).with_no_loop(no_loop);
                self.agenda.add_activation(activation);
                return true;
            }

            return false;
        }

        let mut queued = false;

        for fact_type in dependent_types {
            let facts_of_type = self.working_memory.get_by_type(&fact_type);

            for fact in facts_of_type {
                let mut single_fact_data = TypedFacts::new();
                for (key, value) in fact.data.get_all() {
                    single_fact_data.set(format!("{}.{}", fact_type, key), value.clone());
                }
                single_fact_data.set_fact_handle(fact_type.clone(), fact.handle);

                if super::network::evaluate_rete_ul_node_typed(&node, &single_fact_data) {
                    let activation = Activation::new(rule_name.clone(), priority)
                        .with_no_loop(no_loop)
                        .with_matched_fact(fact.handle);
                    self.agenda.add_activation(activation);
                    queued = true;
                }
            }
        }

        queued
    }

    fn fire_activation(&mut self, activation: Activation) -> Option<String> {
        let Some((_idx, rule)) = self
            .rules
            .iter_mut()
            .enumerate()
            .find(|(_, r)| r.name == activation.rule_name)
        else {
            return None;
        };

        if let Some(matched_handle) = activation.matched_fact_handle {
            if self.working_memory.get(&matched_handle).is_none() {
                return None;
            }
        }

        let original_facts = self.working_memory.to_typed_facts();
        let mut modified_facts = original_facts.clone();

        if let Some(matched_handle) = activation.matched_fact_handle {
            if let Some(fact) = self.working_memory.get(&matched_handle) {
                modified_facts.set_fact_handle(fact.fact_type.clone(), matched_handle);
            }
        }

        let mut action_results = super::ActionResults::new();
        (rule.action)(&mut modified_facts, &mut action_results);

        self.apply_fact_updates(&original_facts, &modified_facts);
        self.process_action_results(action_results);
        self.agenda.mark_rule_fired(&activation);

        Some(activation.rule_name)
    }

    fn fire_queued_activations(&mut self, max_iterations: usize) -> Vec<String> {
        let mut fired_rules = Vec::new();
        let mut iteration_count = 0;

        while let Some(activation) = self.agenda.get_next_activation() {
            iteration_count += 1;
            if iteration_count > max_iterations {
                eprintln!(
                    "WARNING: Maximum iterations ({}) reached in fire_all(). Possible infinite loop!",
                    max_iterations
                );
                break;
            }

            if let Some(rule_name) = self.fire_activation(activation) {
                fired_rules.push(rule_name);
            }
        }

        fired_rules
    }

    fn drain_scheduled_rules_to_quiescence(&mut self, max_iterations: usize) -> Vec<String> {
        let mut fired_rules = Vec::new();
        let mut iteration_count = 0;

        loop {
            let ready_fired = self.run_ready_scheduled_rules();
            if ready_fired.is_empty() {
                break;
            }

            iteration_count += ready_fired.len();
            if iteration_count > max_iterations {
                eprintln!(
                    "WARNING: Maximum iterations ({}) reached while draining scheduled rules.",
                    max_iterations
                );
                break;
            }

            fired_rules.extend(ready_fired);
        }

        fired_rules
    }

    /// Execute scheduled rules that are due at the start of this pass.
    pub fn run_ready_scheduled_rules(&mut self) -> Vec<String> {
        let now = Instant::now();
        let mut ready = Vec::new();
        let mut pending = Vec::new();

        for task in self.scheduled_rules.drain(..) {
            if task.execute_at <= now {
                ready.push(task);
            } else {
                pending.push(task);
            }
        }

        self.scheduled_rules = pending;

        for task in ready {
            self.enqueue_rule_activations_by_name(&task.rule_name);
        }

        self.fire_queued_activations(1000)
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
        let fact_type = self
            .working_memory
            .get(&handle)
            .map(|f| f.fact_type.clone())
            .ok_or_else(|| RuleEngineError::FieldNotFound {
                field: format!("FactHandle {} not found", handle),
            })?;

        self.working_memory
            .update(handle, data)
            .map_err(|e| RuleEngineError::EvaluationError { message: e })?;

        // Trigger incremental propagation for this fact type
        self.propagate_changes_for_type(&fact_type);

        Ok(())
    }

    /// Retract fact from working memory
    pub fn retract(&mut self, handle: FactHandle) -> Result<()> {
        // Get fact type before retract
        let fact_type = self
            .working_memory
            .get(&handle)
            .map(|f| f.fact_type.clone())
            .ok_or_else(|| RuleEngineError::FieldNotFound {
                field: format!("FactHandle {} not found", handle),
            })?;

        self.working_memory
            .retract(handle)
            .map_err(|e| RuleEngineError::EvaluationError { message: e })?;

        // TMS: Handle cascade retraction
        let cascaded_facts = self.tms.retract_with_cascade(handle);

        // Actually retract cascaded facts from working memory
        for cascaded_handle in cascaded_facts {
            if let Ok(fact_type) = self
                .working_memory
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
        self.tms
            .add_logical_justification(handle, source_rule, premise_handles);

        // Trigger incremental propagation for this fact type
        self.propagate_changes_for_type(&fact_type);

        handle
    }

    /// Resolve premise keys (format: "Type.field=value" or "Type.field=")
    /// to a Vec<FactHandle> by looking up facts of the given type and matching
    /// the field value when provided. If value is empty, return the most recent
    /// handle for that type (if any).
    pub fn resolve_premise_keys(&self, premise_keys: Vec<String>) -> Vec<FactHandle> {
        let mut handles = Vec::new();

        for key in premise_keys {
            // Split into type.field=value
            if let Some(eq_pos) = key.find('=') {
                let left = &key[..eq_pos];
                let value_part = &key[eq_pos + 1..];

                if let Some(dot_pos) = left.find('.') {
                    let fact_type = &left[..dot_pos];
                    let field = &left[dot_pos + 1..];

                    // Search facts of this type
                    let facts = self.working_memory.get_by_type(fact_type);
                    // If value_part is empty, pick last handle if any
                    if value_part.is_empty() {
                        // Prefer the most recent non-retracted fact for this type
                        if let Some(fact) = facts.iter().rev().find(|f| !f.metadata.retracted) {
                            handles.push(fact.handle);
                            continue;
                        }
                    } else {
                        // Try to match provided value text against the field in TypedFacts
                        // Parse the provided value into a FactValue-like expectation so we
                        // can compare numbers/booleans properly instead of relying on string equality.
                        fn parse_literal(s: &str) -> super::facts::FactValue {
                            let s = s.trim();
                            if s == "true" {
                                return super::facts::FactValue::Boolean(true);
                            }
                            if s == "false" {
                                return super::facts::FactValue::Boolean(false);
                            }
                            // Quoted string
                            if (s.starts_with('"') && s.ends_with('"'))
                                || (s.starts_with('\'') && s.ends_with('\''))
                            {
                                return super::facts::FactValue::String(
                                    s[1..s.len() - 1].to_string(),
                                );
                            }
                            // Integer
                            if let Ok(i) = s.parse::<i64>() {
                                return super::facts::FactValue::Integer(i);
                            }
                            // Float
                            if let Ok(f) = s.parse::<f64>() {
                                return super::facts::FactValue::Float(f);
                            }

                            // Fallback to string
                            super::facts::FactValue::String(s.to_string())
                        }

                        fn fact_value_equal(
                            a: &super::facts::FactValue,
                            b: &super::facts::FactValue,
                        ) -> bool {
                            use super::facts::FactValue;
                            match (a, b) {
                                (FactValue::Boolean(x), FactValue::Boolean(y)) => x == y,
                                (FactValue::Integer(x), FactValue::Integer(y)) => x == y,
                                (FactValue::Float(x), FactValue::Float(y)) => (x - y).abs() < 1e-9,
                                // Number cross-type comparisons
                                (FactValue::Integer(x), FactValue::Float(y)) => {
                                    ((*x as f64) - *y).abs() < 1e-9
                                }
                                (FactValue::Float(x), FactValue::Integer(y)) => {
                                    (*x - (*y as f64)).abs() < 1e-9
                                }
                                (FactValue::String(x), FactValue::String(y)) => x == y,
                                // Mixed string vs other: compare stringified forms
                                _ => a.as_str() == b.as_str(),
                            }
                        }

                        let expected = parse_literal(value_part);

                        // Prefer the most recent non-retracted matching fact for determinism
                        if let Some(fact) = facts.iter().rev().find(|fact| {
                            if fact.metadata.retracted {
                                return false;
                            }
                            if let Some(fv) = fact.data.get(field) {
                                fact_value_equal(fv, &expected) || fv.as_str() == value_part
                            } else {
                                false
                            }
                        }) {
                            handles.push(fact.handle);
                        }
                    }
                }
            }
        }

        handles
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
                let matches =
                    super::network::evaluate_rete_ul_node_typed(&rule.node, &single_fact_data);

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
        let fact_types: Vec<String> = self
            .working_memory
            .get_all_facts()
            .iter()
            .map(|f| f.fact_type.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Evaluate each fact type using per-fact evaluation
        for fact_type in fact_types {
            let facts_of_type = self.working_memory.get_by_type(&fact_type);

            for rule in self.rules.iter() {
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

                    let matches =
                        super::network::evaluate_rete_ul_node_typed(&rule.node, &single_fact_data);

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
        let mut fired_rules = self.fire_queued_activations(1000);
        fired_rules.extend(self.drain_scheduled_rules_to_quiescence(1000));

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
                super::ActionResult::InsertLogicalFact {
                    fact_type,
                    data,
                    rule_name,
                    premises,
                } => {
                    // Insert new logical fact
                    let _handle = self.insert_logical(fact_type, data, rule_name, premises);
                }
                super::ActionResult::CallFunction {
                    function_name,
                    args,
                } => {
                    // Try to execute function if registered
                    if let Some(func) = self.custom_functions.get(&function_name) {
                        // Convert string args to FactValues
                        let fact_values: Vec<FactValue> =
                            args.iter().map(|s| FactValue::String(s.clone())).collect();

                        // Execute function (ignore return value for actions)
                        let all_facts = self.working_memory.to_typed_facts();
                        match func(&fact_values, &all_facts) {
                            Ok(_) => println!("✅ Called function: {}", function_name),
                            Err(e) => eprintln!("❌ Function {} failed: {}", function_name, e),
                        }
                    } else {
                        // Function not registered, just log
                        println!("🔧 Function call queued: {}({:?})", function_name, args);
                    }
                }
                super::ActionResult::ScheduleRule {
                    rule_name,
                    delay_ms,
                } => {
                    self.schedule_rule(rule_name.clone(), delay_ms);
                    println!("⏰ Rule scheduled: {} after {}ms", rule_name, delay_ms);
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
        self.scheduled_rules.clear();
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
        self.custom_functions
            .insert(name.to_string(), Arc::new(func));
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
        self.scheduled_rules.clear();

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
            self.rules, self.dependencies, self.working_memory, self.agenda
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rete::alpha::AlphaNode;
    use crate::rete::network::ReteUlNode;
    use crate::rete::ActionResult;
    use std::sync::Arc;

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

    #[test]
    fn schedule_rule_zero_delay_executes_in_fire_all() {
        let mut engine = IncrementalEngine::new();

        let trigger_rule = TypedReteUlRule {
            name: "TriggerRule".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 10,
            no_loop: true,
            action: Arc::new(|_, results| {
                results.add(ActionResult::ScheduleRule {
                    rule_name: "ScheduledRule".to_string(),
                    delay_ms: 0,
                });
            }),
        };

        let scheduled_rule = TypedReteUlRule {
            name: "ScheduledRule".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 0,
            no_loop: true,
            action: Arc::new(|facts, _| {
                facts.set("Trigger.scheduled_fired", FactValue::Boolean(true));
            }),
        };

        let mut trigger = TypedFacts::new();
        trigger.set("active", FactValue::Boolean(true));
        engine.add_rule(trigger_rule, vec!["Trigger".to_string()]);
        engine.insert("Trigger".to_string(), trigger);
        engine.add_rule(scheduled_rule, vec!["Trigger".to_string()]);

        let fired = engine.fire_all();

        assert!(fired.iter().any(|name| name == "TriggerRule"));
        assert!(fired.iter().any(|name| name == "ScheduledRule"));
        assert_eq!(
            engine
                .working_memory()
                .to_typed_facts()
                .get("Trigger.scheduled_fired"),
            Some(&FactValue::Boolean(true))
        );
    }

    #[test]
    fn schedule_rule_waits_until_due() {
        let mut engine = IncrementalEngine::new();

        let trigger_rule = TypedReteUlRule {
            name: "TriggerRule".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 10,
            no_loop: true,
            action: Arc::new(|_, results| {
                results.add(ActionResult::ScheduleRule {
                    rule_name: "DelayedRule".to_string(),
                    delay_ms: 20,
                });
            }),
        };

        let delayed_rule = TypedReteUlRule {
            name: "DelayedRule".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 0,
            no_loop: true,
            action: Arc::new(|facts, _| {
                facts.set("Trigger.delayed_fired", FactValue::Boolean(true));
            }),
        };

        let mut trigger = TypedFacts::new();
        trigger.set("active", FactValue::Boolean(true));
        engine.add_rule(trigger_rule, vec!["Trigger".to_string()]);
        engine.insert("Trigger".to_string(), trigger);
        engine.add_rule(delayed_rule, vec!["Trigger".to_string()]);

        let fired = engine.fire_all();
        assert!(fired.iter().any(|name| name == "TriggerRule"));
        assert!(!fired.iter().any(|name| name == "DelayedRule"));

        let early = engine.run_ready_scheduled_rules();
        assert!(early.is_empty());

        std::thread::sleep(std::time::Duration::from_millis(25));

        let later = engine.run_ready_scheduled_rules();
        assert_eq!(later, vec!["DelayedRule".to_string()]);
        assert_eq!(
            engine
                .working_memory()
                .to_typed_facts()
                .get("Trigger.delayed_fired"),
            Some(&FactValue::Boolean(true))
        );
    }

    #[test]
    fn schedule_rule_unknown_target_is_ignored() {
        let mut engine = IncrementalEngine::new();

        let trigger_rule = TypedReteUlRule {
            name: "TriggerRule".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 10,
            no_loop: true,
            action: Arc::new(|_, results| {
                results.add(ActionResult::ScheduleRule {
                    rule_name: "MissingRule".to_string(),
                    delay_ms: 0,
                });
            }),
        };

        engine.add_rule(trigger_rule, vec!["Trigger".to_string()]);

        let mut trigger = TypedFacts::new();
        trigger.set("active", FactValue::Boolean(true));
        engine.insert("Trigger".to_string(), trigger);

        let fired = engine.fire_all();

        assert_eq!(fired, vec!["TriggerRule".to_string()]);
        assert!(engine.run_ready_scheduled_rules().is_empty());
    }

    #[test]
    fn scheduled_rule_preserves_matched_fact_handle() {
        let mut engine = IncrementalEngine::new();

        let trigger_rule = TypedReteUlRule {
            name: "TriggerRule".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 10,
            no_loop: true,
            action: Arc::new(|_, results| {
                results.add(ActionResult::ScheduleRule {
                    rule_name: "ScheduledRetractRule".to_string(),
                    delay_ms: 0,
                });
            }),
        };

        let scheduled_rule = TypedReteUlRule {
            name: "ScheduledRetractRule".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 0,
            no_loop: true,
            action: Arc::new(|facts, results| {
                let handle = facts
                    .get_fact_handle("Trigger")
                    .expect("matched Trigger handle should be available");
                results.add(ActionResult::Retract(handle));
            }),
        };

        let mut trigger = TypedFacts::new();
        trigger.set("active", FactValue::Boolean(true));

        engine.add_rule(trigger_rule, vec!["Trigger".to_string()]);
        let handle = engine.insert("Trigger".to_string(), trigger);
        engine.add_rule(scheduled_rule, vec!["Trigger".to_string()]);

        let fired = engine.fire_all();

        assert!(fired.iter().any(|name| name == "ScheduledRetractRule"));
        assert!(engine.working_memory().get(&handle).is_none());
    }

    #[test]
    fn scheduled_rules_respect_no_loop_semantics() {
        let mut engine = IncrementalEngine::new();

        let scheduled_rule = TypedReteUlRule {
            name: "ScheduledRule".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 0,
            no_loop: true,
            action: Arc::new(|facts, _| {
                let count = facts
                    .get("Trigger.fire_count")
                    .and_then(|value| value.as_integer())
                    .unwrap_or(0);
                facts.set("Trigger.fire_count", FactValue::Integer(count + 1));
            }),
        };

        let mut trigger = TypedFacts::new();
        trigger.set("active", FactValue::Boolean(true));
        trigger.set("fire_count", FactValue::Integer(0));
        engine.insert("Trigger".to_string(), trigger);
        engine.add_rule(scheduled_rule, vec!["Trigger".to_string()]);

        engine.schedule_rule("ScheduledRule".to_string(), 0);
        engine.schedule_rule("ScheduledRule".to_string(), 0);

        let fired = engine.run_ready_scheduled_rules();

        assert_eq!(fired, vec!["ScheduledRule".to_string()]);
        assert_eq!(
            engine
                .working_memory()
                .to_typed_facts()
                .get("Trigger.fire_count"),
            Some(&FactValue::Integer(1))
        );
    }

    #[test]
    fn reset_clears_pending_scheduled_rules() {
        let mut engine = IncrementalEngine::new();

        let scheduled_rule = TypedReteUlRule {
            name: "ScheduledRule".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 0,
            no_loop: true,
            action: Arc::new(|facts, _| {
                facts.set("Trigger.after_reset", FactValue::Boolean(true));
            }),
        };

        let mut trigger = TypedFacts::new();
        trigger.set("active", FactValue::Boolean(true));
        engine.insert("Trigger".to_string(), trigger);
        engine.add_rule(scheduled_rule, vec!["Trigger".to_string()]);

        engine.schedule_rule("ScheduledRule".to_string(), 0);
        engine.reset();

        let fired = engine.run_ready_scheduled_rules();

        assert!(fired.is_empty());
        assert_eq!(
            engine
                .working_memory()
                .to_typed_facts()
                .get("Trigger.after_reset"),
            None
        );
    }

    #[test]
    fn zero_delay_schedule_chain_drains_in_single_fire_all() {
        let mut engine = IncrementalEngine::new();

        let rule_a = TypedReteUlRule {
            name: "RuleA".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 20,
            no_loop: true,
            action: Arc::new(|_, results| {
                results.add(ActionResult::ScheduleRule {
                    rule_name: "RuleB".to_string(),
                    delay_ms: 0,
                });
            }),
        };

        let rule_b = TypedReteUlRule {
            name: "RuleB".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 10,
            no_loop: true,
            action: Arc::new(|_, results| {
                results.add(ActionResult::ScheduleRule {
                    rule_name: "RuleC".to_string(),
                    delay_ms: 0,
                });
            }),
        };

        let rule_c = TypedReteUlRule {
            name: "RuleC".to_string(),
            node: ReteUlNode::UlAlpha(AlphaNode {
                field: "Trigger.active".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            }),
            priority: 0,
            no_loop: true,
            action: Arc::new(|facts, _| {
                facts.set("Trigger.chain_complete", FactValue::Boolean(true));
            }),
        };

        let mut trigger = TypedFacts::new();
        trigger.set("active", FactValue::Boolean(true));

        engine.add_rule(rule_a, vec!["Trigger".to_string()]);
        engine.insert("Trigger".to_string(), trigger);
        engine.add_rule(rule_b, vec!["Trigger".to_string()]);
        engine.add_rule(rule_c, vec!["Trigger".to_string()]);

        let fired = engine.fire_all();

        assert!(fired.iter().any(|name| name == "RuleA"));
        assert!(fired.iter().any(|name| name == "RuleB"));
        assert!(fired.iter().any(|name| name == "RuleC"));
        assert_eq!(
            engine
                .working_memory()
                .to_typed_facts()
                .get("Trigger.chain_complete"),
            Some(&FactValue::Boolean(true))
        );
        assert!(engine.run_ready_scheduled_rules().is_empty());
    }
}

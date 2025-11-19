use crate::engine::{
    agenda::{ActivationGroupManager, AgendaManager},
    analytics::RuleAnalytics,
    facts::Facts,
    knowledge_base::KnowledgeBase,
    plugin::{PluginConfig, PluginHealth, PluginInfo, PluginManager, PluginStats, RulePlugin},
    workflow::WorkflowEngine,
};
use crate::errors::{Result, RuleEngineError};
use crate::types::{ActionType, Value};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Type for custom function implementations
pub type CustomFunction = Box<dyn Fn(&[Value], &Facts) -> Result<Value> + Send + Sync>;

/// Type for custom action handlers
pub type ActionHandler = Box<dyn Fn(&HashMap<String, Value>, &Facts) -> Result<()> + Send + Sync>;

/// Configuration options for the rule engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum number of execution cycles
    pub max_cycles: usize,
    /// Execution timeout
    pub timeout: Option<Duration>,
    /// Enable performance statistics collection
    pub enable_stats: bool,
    /// Enable debug mode with verbose logging
    pub debug_mode: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_cycles: 100,
            timeout: Some(Duration::from_secs(30)),
            enable_stats: true,
            debug_mode: false,
        }
    }
}

/// Result of rule engine execution
#[derive(Debug, Clone)]
pub struct GruleExecutionResult {
    /// Number of execution cycles
    pub cycle_count: usize,
    /// Number of rules evaluated
    pub rules_evaluated: usize,
    /// Number of rules that fired
    pub rules_fired: usize,
    /// Total execution time
    pub execution_time: Duration,
}

/// Rust Rule Engine - High-performance rule execution engine
pub struct RustRuleEngine {
    knowledge_base: KnowledgeBase,
    config: EngineConfig,
    custom_functions: HashMap<String, CustomFunction>,
    action_handlers: HashMap<String, ActionHandler>,
    analytics: Option<RuleAnalytics>,
    agenda_manager: AgendaManager,
    activation_group_manager: ActivationGroupManager,
    /// Track rules that have fired globally (for no-loop support)
    fired_rules_global: std::collections::HashSet<String>,
    /// Workflow engine for rule chaining and sequential execution
    workflow_engine: WorkflowEngine,
    /// Plugin manager for extensible functionality
    plugin_manager: PluginManager,
}

impl RustRuleEngine {
    /// Execute all rules and call callback when a rule is fired
    pub fn execute_with_callback<F>(&mut self, facts: &Facts, mut on_rule_fired: F) -> Result<GruleExecutionResult>
    where
        F: FnMut(&str, &str),
    {
        use chrono::Utc;
        let timestamp = Utc::now();
        let start_time = std::time::Instant::now();
        let mut cycle_count = 0;
        let mut rules_evaluated = 0;
        let mut rules_fired = 0;

        self.sync_workflow_agenda_activations();

        for cycle in 0..self.config.max_cycles {
            cycle_count = cycle + 1;
            let mut any_rule_fired = false;
            let mut fired_rules_in_cycle = std::collections::HashSet::new();
            self.activation_group_manager.reset_cycle();

            if let Some(timeout) = self.config.timeout {
                if start_time.elapsed() > timeout {
                    return Err(crate::errors::RuleEngineError::EvaluationError {
                        message: "Execution timeout exceeded".to_string(),
                    });
                }
            }

            let mut rules = self.knowledge_base.get_rules().clone();
            rules.sort_by(|a, b| b.salience.cmp(&a.salience));
            let rules: Vec<_> = rules
                .iter()
                .filter(|rule| self.agenda_manager.should_evaluate_rule(rule))
                .collect();

            for rule in &rules {
                if !rule.enabled {
                    continue;
                }
                if !rule.is_active_at(timestamp) {
                    continue;
                }
                if !self.agenda_manager.can_fire_rule(rule) {
                    continue;
                }
                if !self.activation_group_manager.can_fire(rule) {
                    continue;
                }
                if rule.no_loop && self.fired_rules_global.contains(&rule.name) {
                    continue;
                }
                rules_evaluated += 1;
                let condition_result = self.evaluate_conditions(&rule.conditions, facts)?;
                if condition_result {
                    for action in &rule.actions {
                        self.execute_action(action, facts)?;
                    }
                    rules_fired += 1;
                    any_rule_fired = true;
                    fired_rules_in_cycle.insert(rule.name.clone());
                    if rule.no_loop {
                        self.fired_rules_global.insert(rule.name.clone());
                    }
                    self.agenda_manager.mark_rule_fired(rule);
                    self.activation_group_manager.mark_fired(rule);
                    // Gọi callback khi rule fired
                    on_rule_fired(&rule.name, "facts"); // TODO: truyền id facts thực tế nếu có
                }
            }
            if !any_rule_fired {
                break;
            }
            self.sync_workflow_agenda_activations();
        }
        let execution_time = start_time.elapsed();
        Ok(crate::engine::GruleExecutionResult {
            cycle_count,
            rules_evaluated,
            rules_fired,
            execution_time,
        })
    }
    /// Create a new RustRuleEngine with default configuration
    pub fn new(knowledge_base: KnowledgeBase) -> Self {
        Self {
            knowledge_base,
            config: EngineConfig::default(),
            custom_functions: HashMap::new(),
            action_handlers: HashMap::new(),
            analytics: None,
            agenda_manager: AgendaManager::new(),
            activation_group_manager: ActivationGroupManager::new(),
            fired_rules_global: std::collections::HashSet::new(),
            workflow_engine: WorkflowEngine::new(),
            plugin_manager: PluginManager::with_default_config(),
        }
    }

    /// Create a new RustRuleEngine with custom configuration
    pub fn with_config(knowledge_base: KnowledgeBase, config: EngineConfig) -> Self {
        Self {
            knowledge_base,
            config,
            custom_functions: HashMap::new(),
            action_handlers: HashMap::new(),
            analytics: None,
            agenda_manager: AgendaManager::new(),
            activation_group_manager: ActivationGroupManager::new(),
            fired_rules_global: std::collections::HashSet::new(),
            workflow_engine: WorkflowEngine::new(),
            plugin_manager: PluginManager::with_default_config(),
        }
    }

    /// Register a custom function
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[Value], &Facts) -> Result<Value> + Send + Sync + 'static,
    {
        self.custom_functions
            .insert(name.to_string(), Box::new(func));
    }

    /// Register a custom action handler
    pub fn register_action_handler<F>(&mut self, action_type: &str, handler: F)
    where
        F: Fn(&HashMap<String, Value>, &Facts) -> Result<()> + Send + Sync + 'static,
    {
        self.action_handlers
            .insert(action_type.to_string(), Box::new(handler));
    }

    /// Enable analytics with custom configuration
    pub fn enable_analytics(&mut self, analytics: RuleAnalytics) {
        self.analytics = Some(analytics);
    }

    /// Reset global no-loop tracking (useful for testing or when facts change significantly)
    pub fn reset_no_loop_tracking(&mut self) {
        self.fired_rules_global.clear();
    }

    /// Disable analytics
    pub fn disable_analytics(&mut self) {
        self.analytics = None;
    }

    /// Get reference to analytics data
    pub fn analytics(&self) -> Option<&RuleAnalytics> {
        self.analytics.as_ref()
    }

    /// Enable debug mode for detailed execution logging
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.config.debug_mode = enabled;
    }

    /// Check if a custom function is registered
    pub fn has_function(&self, name: &str) -> bool {
        self.custom_functions.contains_key(name)
    }

    /// Check if a custom action handler is registered
    pub fn has_action_handler(&self, action_type: &str) -> bool {
        self.action_handlers.contains_key(action_type)
    }

    /// Get ready scheduled tasks
    pub fn get_ready_tasks(&mut self) -> Vec<crate::engine::workflow::ScheduledTask> {
        self.workflow_engine.get_ready_tasks()
    }

    /// Execute scheduled tasks that are ready
    pub fn execute_scheduled_tasks(&mut self, facts: &Facts) -> Result<()> {
        let ready_tasks = self.get_ready_tasks();
        for task in ready_tasks {
            if let Some(rule) = self
                .knowledge_base
                .get_rules()
                .iter()
                .find(|r| r.name == task.rule_name)
            {
                if self.config.debug_mode {
                    println!("⚡ Executing scheduled task: {}", task.rule_name);
                }

                // Execute just this one rule if conditions match
                if self.evaluate_conditions(&rule.conditions, facts)? {
                    for action in &rule.actions {
                        self.execute_action(action, facts)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Activate agenda group
    pub fn activate_agenda_group(&mut self, group: String) {
        self.workflow_engine.activate_agenda_group(group.clone());
        self.agenda_manager.set_focus(&group);
    }

    /// Get the knowledge base
    pub fn knowledge_base(&self) -> &KnowledgeBase {
        &self.knowledge_base
    }

    /// Get mutable reference to knowledge base
    pub fn knowledge_base_mut(&mut self) -> &mut KnowledgeBase {
        &mut self.knowledge_base
    }

    /// Sync workflow engine agenda activations with agenda manager
    fn sync_workflow_agenda_activations(&mut self) {
        // Process any pending agenda activations from workflow engine
        while let Some(agenda_group) = self.workflow_engine.get_next_pending_agenda_activation() {
            if self.config.debug_mode {
                println!("🔄 Syncing workflow agenda activation: {}", agenda_group);
            }
            self.agenda_manager.set_focus(&agenda_group);
        }
    }

    /// Set focus to a specific agenda group
    pub fn set_agenda_focus(&mut self, group: &str) {
        self.agenda_manager.set_focus(group);
    }

    /// Get the currently active agenda group
    pub fn get_active_agenda_group(&self) -> &str {
        self.agenda_manager.get_active_group()
    }

    /// Pop the agenda focus stack
    pub fn pop_agenda_focus(&mut self) -> Option<String> {
        self.agenda_manager.pop_focus()
    }

    /// Clear all agenda focus and return to MAIN
    pub fn clear_agenda_focus(&mut self) {
        self.agenda_manager.clear_focus();
    }

    /// Get all agenda groups that have rules
    pub fn get_agenda_groups(&self) -> Vec<String> {
        self.agenda_manager
            .get_agenda_groups(&self.knowledge_base.get_rules())
    }

    /// Get all activation groups that have rules
    pub fn get_activation_groups(&self) -> Vec<String> {
        self.activation_group_manager
            .get_activation_groups(&self.knowledge_base.get_rules())
    }

    // 🔄 Workflow Engine Methods

    /// Start a new workflow
    pub fn start_workflow(&mut self, workflow_name: Option<String>) -> String {
        self.workflow_engine.start_workflow(workflow_name)
    }

    /// Get workflow statistics
    pub fn get_workflow_stats(&self) -> crate::engine::workflow::WorkflowStats {
        self.workflow_engine.get_workflow_stats()
    }

    /// Get workflow state by ID
    pub fn get_workflow(
        &self,
        workflow_id: &str,
    ) -> Option<&crate::engine::workflow::WorkflowState> {
        self.workflow_engine.get_workflow(workflow_id)
    }

    /// Clean up completed workflows
    pub fn cleanup_completed_workflows(&mut self, older_than: Duration) {
        self.workflow_engine.cleanup_completed_workflows(older_than);
    }

    /// Execute workflow step by activating specific agenda group
    pub fn execute_workflow_step(
        &mut self,
        agenda_group: &str,
        facts: &Facts,
    ) -> Result<GruleExecutionResult> {
        // Set agenda focus to the specific group
        self.set_agenda_focus(agenda_group);

        // Execute rules in that group
        let result = self.execute(facts)?;

        // Process any workflow actions that were triggered
        self.process_workflow_actions(facts)?;

        Ok(result)
    }

    /// Execute a complete workflow by processing agenda groups sequentially
    pub fn execute_workflow(
        &mut self,
        agenda_groups: Vec<&str>,
        facts: &Facts,
    ) -> Result<crate::engine::workflow::WorkflowResult> {
        let start_time = Instant::now();
        let mut total_steps = 0;

        if self.config.debug_mode {
            println!(
                "🔄 Starting workflow execution with {} steps",
                agenda_groups.len()
            );
        }

        for (i, group) in agenda_groups.iter().enumerate() {
            if self.config.debug_mode {
                println!("📋 Executing workflow step {}: {}", i + 1, group);
            }

            let step_result = self.execute_workflow_step(group, facts)?;
            total_steps += 1;

            if step_result.rules_fired == 0 {
                if self.config.debug_mode {
                    println!("⏸️ No rules fired in step '{}', stopping workflow", group);
                }
                break;
            }
        }

        let execution_time = start_time.elapsed();

        Ok(crate::engine::workflow::WorkflowResult::success(
            total_steps,
            execution_time,
        ))
    }

    /// Process workflow-related actions and scheduled tasks
    fn process_workflow_actions(&mut self, facts: &Facts) -> Result<()> {
        // Process agenda group activations
        while let Some(group) = self.workflow_engine.get_next_agenda_group() {
            self.set_agenda_focus(&group);
        }

        // Process scheduled tasks
        let ready_tasks = self.workflow_engine.get_ready_tasks();
        for task in ready_tasks {
            if self.config.debug_mode {
                println!("⚡ Executing scheduled task: {}", task.rule_name);
            }

            // Find and execute the specific rule
            if let Some(rule) = self
                .knowledge_base
                .get_rules()
                .iter()
                .find(|r| r.name == task.rule_name)
            {
                // Execute just this one rule
                if self.evaluate_conditions(&rule.conditions, facts)? {
                    for action in &rule.actions {
                        self.execute_action(action, facts)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute all rules in the knowledge base against the given facts
    pub fn execute(&mut self, facts: &Facts) -> Result<GruleExecutionResult> {
        self.execute_at_time(facts, Utc::now())
    }

    /// Execute all rules at a specific timestamp (for date-effective/expires testing)
    pub fn execute_at_time(
        &mut self,
        facts: &Facts,
        timestamp: DateTime<Utc>,
    ) -> Result<GruleExecutionResult> {
        let start_time = Instant::now();
        let mut cycle_count = 0;
        let mut rules_evaluated = 0;
        let mut rules_fired = 0;

        // Process any pending agenda group activations from workflow engine
        self.sync_workflow_agenda_activations();

        if self.config.debug_mode {
            println!(
                "🚀 Starting rule execution with {} rules (agenda group: {})",
                self.knowledge_base.get_rules().len(),
                self.agenda_manager.get_active_group()
            );
        }

        for cycle in 0..self.config.max_cycles {
            cycle_count = cycle + 1;
            let mut any_rule_fired = false;
            let mut fired_rules_in_cycle = std::collections::HashSet::new();

            // Reset activation groups for each cycle
            self.activation_group_manager.reset_cycle();

            // Check for timeout
            if let Some(timeout) = self.config.timeout {
                if start_time.elapsed() > timeout {
                    return Err(RuleEngineError::EvaluationError {
                        message: "Execution timeout exceeded".to_string(),
                    });
                }
            }

            // Get rules sorted by salience (highest first)
            let mut rules = self.knowledge_base.get_rules().clone();
            rules.sort_by(|a, b| b.salience.cmp(&a.salience));

            // Filter rules by agenda group
            let rules: Vec<_> = rules
                .iter()
                .filter(|rule| self.agenda_manager.should_evaluate_rule(rule))
                .collect();

            for rule in &rules {
                if !rule.enabled {
                    continue;
                }

                // Check date effective/expires
                if !rule.is_active_at(timestamp) {
                    continue;
                }

                // Check agenda group constraints (lock-on-active)
                if !self.agenda_manager.can_fire_rule(rule) {
                    continue;
                }

                // Check activation group constraints
                if !self.activation_group_manager.can_fire(rule) {
                    continue;
                }

                // Check no-loop: if rule has no_loop=true and already fired globally, skip
                if rule.no_loop && self.fired_rules_global.contains(&rule.name) {
                    if self.config.debug_mode {
                        println!("⛔ Skipping '{}' due to no_loop (already fired)", rule.name);
                    }
                    continue;
                }

                rules_evaluated += 1;
                let rule_start = Instant::now();

                if self.config.debug_mode {
                    println!("📝 Evaluating rule: {} (no_loop={})", rule.name, rule.no_loop);
                }

                // Evaluate rule conditions
                let condition_result = self.evaluate_conditions(&rule.conditions, facts)?;
                if self.config.debug_mode {
                    println!(
                        "  🔍 Condition result for '{}': {}",
                        rule.name, condition_result
                    );
                }

                if condition_result {
                    if self.config.debug_mode {
                        println!(
                            "🔥 Rule '{}' fired (salience: {})",
                            rule.name, rule.salience
                        );
                    }

                    // Execute actions
                    for action in &rule.actions {
                        self.execute_action(action, facts)?;
                    }

                    let rule_duration = rule_start.elapsed();

                    // Record analytics if enabled
                    if let Some(analytics) = &mut self.analytics {
                        analytics.record_execution(&rule.name, rule_duration, true, true, None, 0);
                    }

                    rules_fired += 1;
                    any_rule_fired = true;

                    // Track that this rule fired in this cycle (for cycle counting)
                    fired_rules_in_cycle.insert(rule.name.clone());

                    // Track that this rule fired globally (for no-loop support)
                    if rule.no_loop {
                        self.fired_rules_global.insert(rule.name.clone());
                        if self.config.debug_mode {
                            println!("  🔒 Marked '{}' as fired (no_loop tracking)", rule.name);
                        }
                    }

                    // Mark rule as fired for agenda and activation group management
                    self.agenda_manager.mark_rule_fired(rule);
                    self.activation_group_manager.mark_fired(rule);
                } else {
                    let rule_duration = rule_start.elapsed();

                    // Record analytics for failed rules too
                    if let Some(analytics) = &mut self.analytics {
                        analytics.record_execution(
                            &rule.name,
                            rule_duration,
                            false,
                            false,
                            None,
                            0,
                        );
                    }
                }
            }

            // If no rules fired in this cycle, we're done
            if !any_rule_fired {
                break;
            }

            // Sync any new workflow agenda activations at the end of each cycle
            self.sync_workflow_agenda_activations();
        }

        let execution_time = start_time.elapsed();

        Ok(GruleExecutionResult {
            cycle_count,
            rules_evaluated,
            rules_fired,
            execution_time,
        })
    }

    /// Evaluate conditions against facts
    fn evaluate_conditions(
        &self,
        conditions: &crate::engine::rule::ConditionGroup,
        facts: &Facts,
    ) -> Result<bool> {
        use crate::engine::pattern_matcher::PatternMatcher;
        use crate::engine::rule::ConditionGroup;

        match conditions {
            ConditionGroup::Single(condition) => self.evaluate_single_condition(condition, facts),
            ConditionGroup::Compound {
                left,
                operator,
                right,
            } => {
                let left_result = self.evaluate_conditions(left, facts)?;
                let right_result = self.evaluate_conditions(right, facts)?;

                match operator {
                    crate::types::LogicalOperator::And => Ok(left_result && right_result),
                    crate::types::LogicalOperator::Or => Ok(left_result || right_result),
                    crate::types::LogicalOperator::Not => Err(RuleEngineError::EvaluationError {
                        message: "NOT operator should not appear in compound conditions"
                            .to_string(),
                    }),
                }
            }
            ConditionGroup::Not(condition) => {
                let result = self.evaluate_conditions(condition, facts)?;
                Ok(!result)
            }
            // Pattern matching conditions
            ConditionGroup::Exists(condition) => {
                Ok(PatternMatcher::evaluate_exists(condition, facts))
            }
            ConditionGroup::Forall(condition) => {
                Ok(PatternMatcher::evaluate_forall(condition, facts))
            }
            ConditionGroup::Accumulate {
                result_var,
                source_pattern,
                extract_field,
                source_conditions,
                function,
                function_arg,
            } => {
                // Evaluate accumulate and inject result into facts
                self.evaluate_accumulate(
                    result_var,
                    source_pattern,
                    extract_field,
                    source_conditions,
                    function,
                    function_arg,
                    facts,
                )?;
                // After injecting result, return true to continue
                Ok(true)
            }
        }
    }

    /// Evaluate accumulate condition and inject result into facts
    fn evaluate_accumulate(
        &self,
        result_var: &str,
        source_pattern: &str,
        extract_field: &str,
        source_conditions: &[String],
        function: &str,
        function_arg: &str,
        facts: &Facts,
    ) -> Result<()> {
        use crate::rete::accumulate::*;

        // 1. Collect all facts matching the source pattern
        let all_facts = facts.get_all_facts();
        let mut matching_values = Vec::new();

        // Find all facts that match the pattern (e.g., "Order.amount", "Order.status")
        let pattern_prefix = format!("{}.", source_pattern);

        // Group facts by instance (e.g., Order.1.amount, Order.1.status)
        let mut instances: HashMap<String, HashMap<String, Value>> = HashMap::new();

        for (key, value) in &all_facts {
            if key.starts_with(&pattern_prefix) {
                // Extract instance ID if present (e.g., "Order.1.amount" -> "1")
                let parts: Vec<&str> = key.strip_prefix(&pattern_prefix).unwrap().split('.').collect();

                if parts.len() >= 2 {
                    // Has instance ID: Order.1.amount
                    let instance_id = parts[0];
                    let field_name = parts[1..].join(".");

                    instances
                        .entry(instance_id.to_string())
                        .or_insert_with(HashMap::new)
                        .insert(field_name, value.clone());
                } else if parts.len() == 1 {
                    // No instance ID: Order.amount (single instance)
                    instances
                        .entry("default".to_string())
                        .or_insert_with(HashMap::new)
                        .insert(parts[0].to_string(), value.clone());
                }
            }
        }

        // 2. Filter instances by source conditions
        for (_instance_id, instance_facts) in instances {
            // Check if this instance matches all source conditions
            let mut matches = true;

            for condition_str in source_conditions {
                // Parse condition: "status == \"completed\""
                if !self.evaluate_condition_string(condition_str, &instance_facts) {
                    matches = false;
                    break;
                }
            }

            if matches {
                // Extract the field value
                if let Some(value) = instance_facts.get(extract_field) {
                    matching_values.push(value.clone());
                }
            }
        }

        // 3. Run accumulate function
        let result = match function {
            "sum" => {
                let mut state = SumFunction.init();
                for value in &matching_values {
                    state.accumulate(&self.value_to_fact_value(value));
                }
                self.fact_value_to_value(&state.get_result())
            }
            "count" => {
                let mut state = CountFunction.init();
                for value in &matching_values {
                    state.accumulate(&self.value_to_fact_value(value));
                }
                self.fact_value_to_value(&state.get_result())
            }
            "average" | "avg" => {
                let mut state = AverageFunction.init();
                for value in &matching_values {
                    state.accumulate(&self.value_to_fact_value(value));
                }
                self.fact_value_to_value(&state.get_result())
            }
            "min" => {
                let mut state = MinFunction.init();
                for value in &matching_values {
                    state.accumulate(&self.value_to_fact_value(value));
                }
                self.fact_value_to_value(&state.get_result())
            }
            "max" => {
                let mut state = MaxFunction.init();
                for value in &matching_values {
                    state.accumulate(&self.value_to_fact_value(value));
                }
                self.fact_value_to_value(&state.get_result())
            }
            _ => {
                return Err(RuleEngineError::EvaluationError {
                    message: format!("Unknown accumulate function: {}", function),
                });
            }
        };

        // 4. Inject result into facts
        // Use pattern.function as key to avoid collision
        let result_key = format!("{}.{}", source_pattern, function);

        facts.set(&result_key, result);

        if self.config.debug_mode {
            println!("    🧮 Accumulate result: {} = {:?}", result_key, facts.get(&result_key));
        }

        Ok(())
    }

    /// Helper: Convert Value to FactValue
    fn value_to_fact_value(&self, value: &Value) -> crate::rete::facts::FactValue {
        use crate::rete::facts::FactValue;
        match value {
            Value::Integer(i) => FactValue::Integer(*i),
            Value::Number(n) => FactValue::Float(*n),
            Value::String(s) => FactValue::String(s.clone()),
            Value::Boolean(b) => FactValue::Boolean(*b),
            _ => FactValue::String(value.to_string()),
        }
    }

    /// Helper: Convert FactValue to Value
    fn fact_value_to_value(&self, fact_value: &crate::rete::facts::FactValue) -> Value {
        use crate::rete::facts::FactValue;
        match fact_value {
            FactValue::Integer(i) => Value::Integer(*i),
            FactValue::Float(f) => Value::Number(*f),
            FactValue::String(s) => Value::String(s.clone()),
            FactValue::Boolean(b) => Value::Boolean(*b),
            FactValue::Array(_) => Value::String(format!("{:?}", fact_value)),
            FactValue::Null => Value::String("null".to_string()),
        }
    }

    /// Helper: Evaluate a condition string against facts
    fn evaluate_condition_string(&self, condition: &str, facts: &HashMap<String, Value>) -> bool {
        // Simple condition parser: "field == value" or "field != value", etc.
        let condition = condition.trim();

        // Try to parse operator
        let operators = ["==", "!=", ">=", "<=", ">", "<"];

        for op in &operators {
            if let Some(pos) = condition.find(op) {
                let field = condition[..pos].trim();
                let value_str = condition[pos + op.len()..].trim()
                    .trim_matches('"')
                    .trim_matches('\'');

                if let Some(field_value) = facts.get(field) {
                    return self.compare_values(field_value, op, value_str);
                } else {
                    return false;
                }
            }
        }

        false
    }

    /// Helper: Compare values
    fn compare_values(&self, field_value: &Value, operator: &str, value_str: &str) -> bool {
        match field_value {
            Value::String(s) => {
                match operator {
                    "==" => s == value_str,
                    "!=" => s != value_str,
                    _ => false,
                }
            }
            Value::Integer(i) => {
                if let Ok(num) = value_str.parse::<i64>() {
                    match operator {
                        "==" => *i == num,
                        "!=" => *i != num,
                        ">" => *i > num,
                        "<" => *i < num,
                        ">=" => *i >= num,
                        "<=" => *i <= num,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Value::Number(n) => {
                if let Ok(num) = value_str.parse::<f64>() {
                    match operator {
                        "==" => (*n - num).abs() < f64::EPSILON,
                        "!=" => (*n - num).abs() >= f64::EPSILON,
                        ">" => *n > num,
                        "<" => *n < num,
                        ">=" => *n >= num,
                        "<=" => *n <= num,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            Value::Boolean(b) => {
                if let Ok(bool_val) = value_str.parse::<bool>() {
                    match operator {
                        "==" => *b == bool_val,
                        "!=" => *b != bool_val,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Evaluate rule conditions - wrapper for evaluate_conditions for compatibility
    fn evaluate_rule_conditions(
        &self,
        rule: &crate::engine::rule::Rule,
        facts: &Facts,
    ) -> Result<bool> {
        self.evaluate_conditions(&rule.conditions, facts)
    }

    /// Check if a fact object has been retracted
    fn is_retracted(&self, object_name: &str, facts: &Facts) -> bool {
        let retract_key = format!("_retracted_{}", object_name);
        matches!(facts.get(&retract_key), Some(Value::Boolean(true)))
    }

    /// Evaluate a single condition
    fn evaluate_single_condition(
        &self,
        condition: &crate::engine::rule::Condition,
        facts: &Facts,
    ) -> Result<bool> {
        use crate::engine::rule::ConditionExpression;

        let result = match &condition.expression {
            ConditionExpression::Field(field_name) => {
                // Check if the fact object has been retracted
                // Extract object name from field (e.g., "Session.expired" -> "Session")
                if let Some(object_name) = field_name.split('.').next() {
                    if self.is_retracted(object_name, facts) {
                        if self.config.debug_mode {
                            println!("    🗑️ Skipping retracted fact: {}", object_name);
                        }
                        return Ok(false);
                    }
                }

                // Field condition - try nested first, then flat lookup
                let field_value = facts
                    .get_nested(field_name)
                    .or_else(|| facts.get(field_name));

                if self.config.debug_mode {
                    println!(
                        "    🔎 Evaluating field condition: {} {} {:?}",
                        field_name,
                        format!("{:?}", condition.operator).to_lowercase(),
                        condition.value
                    );
                    println!("      Field value: {:?}", field_value);
                }

                if let Some(value) = field_value {
                    // condition.operator.evaluate(&value, &condition.value)
                    // If the condition's right-hand value is a string that names another fact,
                    // try to resolve that fact and use its value for comparison. This allows
                    // rules like `L1 > L1Min` where the parser may have stored "L1Min"
                    // as a string literal.
                    let rhs = match &condition.value {
                        crate::types::Value::String(s) => {
                            // Try nested lookup first, then flat lookup
                            facts
                                .get_nested(s)
                                .or_else(|| facts.get(s))
                                .unwrap_or(crate::types::Value::String(s.clone()))
                        }
                        _ => condition.value.clone(),
                    };

                    if self.config.debug_mode {
                        println!("      Resolved RHS for comparison: {:?}", rhs);
                    }

                    condition.operator.evaluate(&value, &rhs)
                } else {
                    false
                }
            }
            ConditionExpression::FunctionCall { name, args } => {
                // Function call condition
                if self.config.debug_mode {
                    println!(
                        "    🔎 Evaluating function condition: {}({:?}) {} {:?}",
                        name,
                        args,
                        format!("{:?}", condition.operator).to_lowercase(),
                        condition.value
                    );
                }

                if let Some(function) = self.custom_functions.get(name) {
                    // Resolve arguments from facts
                    let arg_values: Vec<Value> = args
                        .iter()
                        .map(|arg| {
                            facts
                                .get_nested(arg)
                                .or_else(|| facts.get(arg))
                                .unwrap_or(Value::String(arg.clone()))
                        })
                        .collect();

                    // Call the function
                    match function(&arg_values, facts) {
                        Ok(result_value) => {
                            if self.config.debug_mode {
                                println!("      Function result: {:?}", result_value);
                            }
                            condition.operator.evaluate(&result_value, &condition.value)
                        }
                        Err(e) => {
                            if self.config.debug_mode {
                                println!("      Function error: {}", e);
                            }
                            false
                        }
                    }
                } else {
                    if self.config.debug_mode {
                        println!("      Function '{}' not found", name);
                    }
                    false
                }
            }
            ConditionExpression::Test { name, args } => {
                // Test CE condition - expects boolean result
                if self.config.debug_mode {
                    println!("    🧪 Evaluating test CE: test({}({:?}))", name, args);
                }

                if let Some(function) = self.custom_functions.get(name) {
                    // Resolve arguments from facts
                    let arg_values: Vec<Value> = args
                        .iter()
                        .map(|arg| {
                            let resolved = facts
                                .get_nested(arg)
                                .or_else(|| facts.get(arg))
                                .unwrap_or(Value::String(arg.clone()));
                            if self.config.debug_mode {
                                println!("      Resolving arg '{}' -> {:?}", arg, resolved);
                            }
                            resolved
                        })
                        .collect();

                    // Call the function
                    match function(&arg_values, facts) {
                        Ok(result_value) => {
                            if self.config.debug_mode {
                                println!("      Test result: {:?}", result_value);
                            }
                            // Test CE expects boolean result directly
                            match result_value {
                                Value::Boolean(b) => b,
                                Value::Integer(i) => i != 0,
                                Value::Number(f) => f != 0.0,
                                Value::String(s) => !s.is_empty(),
                                _ => false,
                            }
                        }
                        Err(e) => {
                            if self.config.debug_mode {
                                println!("      Test function error: {}", e);
                            }
                            false
                        }
                    }
                } else {
                    if self.config.debug_mode {
                        println!("      Test function '{}' not found", name);
                    }
                    false
                }
            }
            ConditionExpression::MultiField { field, operation, variable: _ } => {
                // Multi-field operation condition
                if self.config.debug_mode {
                    println!("    📦 Evaluating multi-field: {}.{}", field, operation);
                }

                // Get the field value
                let field_value = facts.get_nested(field).or_else(|| facts.get(field));

                if let Some(value) = field_value {
                    match operation.as_str() {
                        "empty" => {
                            matches!(value, Value::Array(arr) if arr.is_empty())
                        }
                        "not_empty" => {
                            matches!(value, Value::Array(arr) if !arr.is_empty())
                        }
                        "count" => {
                            if let Value::Array(arr) = value {
                                let count = Value::Integer(arr.len() as i64);
                                condition.operator.evaluate(&count, &condition.value)
                            } else {
                                false
                            }
                        }
                        "contains" => {
                            // Use existing contains operator
                            condition.operator.evaluate(&value, &condition.value)
                        }
                        _ => {
                            // Other operations (collect, first, last) not fully supported yet
                            // Return true to not block rule evaluation
                            if self.config.debug_mode {
                                println!("      ⚠️ Operation '{}' not fully implemented yet", operation);
                            }
                            true
                        }
                    }
                } else {
                    false
                }
            }
        };

        if self.config.debug_mode {
            println!("      Result: {}", result);
        }

        Ok(result)
    }

    /// Execute an action
    fn execute_action(&mut self, action: &ActionType, facts: &Facts) -> Result<()> {
        match action {
            ActionType::Set { field, value } => {
                // Evaluate expression if value is an Expression
                let evaluated_value = match value {
                    Value::Expression(expr) => {
                        // Evaluate the expression with current facts
                        crate::expression::evaluate_expression(expr, facts)?
                    }
                    _ => value.clone(),
                };

                // Try nested first, then fall back to flat key setting
                if let Err(_) = facts.set_nested(field, evaluated_value.clone()) {
                    // If nested fails, use flat key
                    facts.set(field, evaluated_value.clone());
                }
                if self.config.debug_mode {
                    println!("  ✅ Set {field} = {evaluated_value:?}");
                }
            }
            ActionType::Log { message } => {
                println!("📋 LOG: {}", message);
            }
            ActionType::Call { function, args } => {
                let result = self.execute_function_call(function, args, facts)?;
                if self.config.debug_mode {
                    println!("  📞 Called {function}({args:?}) -> {result}");
                }
            }
            ActionType::MethodCall {
                object,
                method,
                args,
            } => {
                let result = self.execute_method_call(object, method, args, facts)?;
                if self.config.debug_mode {
                    println!("  🔧 Called {object}.{method}({args:?}) -> {result}");
                }
            }
            ActionType::Update { object } => {
                if self.config.debug_mode {
                    println!("  🔄 Updated {object}");
                }
                // Update action is mainly for working memory management
                // In this implementation, it's mostly a no-op since we update in place
            }
            ActionType::Retract { object } => {
                if self.config.debug_mode {
                    println!("  🗑️ Retracted {object}");
                }
                // Mark fact as retracted in working memory
                facts.set(&format!("_retracted_{}", object), Value::Boolean(true));
            }
            ActionType::Custom {
                action_type,
                params,
            } => {
                if let Some(handler) = self.action_handlers.get(action_type) {
                    if self.config.debug_mode {
                        println!(
                            "  🎯 Executing custom action: {action_type} with params: {params:?}"
                        );
                    }

                    // Resolve parameter values from facts
                    let resolved_params = self.resolve_action_parameters(params, facts)?;

                    // Execute the registered handler
                    handler(&resolved_params, facts)?;
                } else {
                    if self.config.debug_mode {
                        println!("  ⚠️ No handler registered for custom action: {action_type}");
                        println!(
                            "     Available handlers: {:?}",
                            self.action_handlers.keys().collect::<Vec<_>>()
                        );
                    }

                    // Return error if no handler found
                    return Err(RuleEngineError::EvaluationError {
                        message: format!(
                            "No action handler registered for '{action_type}'. Use engine.register_action_handler() to add custom action handlers."
                        ),
                    });
                }
            }
            // 🔄 Workflow Actions
            ActionType::ActivateAgendaGroup { group } => {
                if self.config.debug_mode {
                    println!("  🎯 Activating agenda group: {}", group);
                }
                // Sync with both workflow engine and agenda manager immediately
                self.workflow_engine.activate_agenda_group(group.clone());
                self.agenda_manager.set_focus(group);
            }
            ActionType::ScheduleRule {
                rule_name,
                delay_ms,
            } => {
                if self.config.debug_mode {
                    println!(
                        "  ⏰ Scheduling rule '{}' to execute in {}ms",
                        rule_name, delay_ms
                    );
                }
                self.workflow_engine
                    .schedule_rule(rule_name.clone(), *delay_ms, None);
            }
            ActionType::CompleteWorkflow { workflow_name } => {
                if self.config.debug_mode {
                    println!("  ✅ Completing workflow: {}", workflow_name);
                }
                self.workflow_engine
                    .complete_workflow(workflow_name.clone());
            }
            ActionType::SetWorkflowData { key, value } => {
                if self.config.debug_mode {
                    println!("  💾 Setting workflow data: {} = {:?}", key, value);
                }
                // For now, we'll use a default workflow ID. Later this could be enhanced
                // to track current workflow context
                let workflow_id = "default_workflow";
                self.workflow_engine
                    .set_workflow_data(workflow_id, key.clone(), value.clone());
            }
        }
        Ok(())
    }

    /// Execute function call
    fn execute_function_call(
        &self,
        function: &str,
        args: &[Value],
        facts: &Facts,
    ) -> Result<String> {
        let function_lower = function.to_lowercase();

        // Handle built-in utility functions
        match function_lower.as_str() {
            "log" | "print" | "println" => self.handle_log_function(args),
            "update" | "refresh" => self.handle_update_function(args),
            "now" | "timestamp" => self.handle_timestamp_function(),
            "random" => self.handle_random_function(args),
            "format" | "sprintf" => self.handle_format_function(args),
            "length" | "size" | "count" => self.handle_length_function(args),
            "sum" | "add" => self.handle_sum_function(args),
            "max" | "maximum" => self.handle_max_function(args),
            "min" | "minimum" => self.handle_min_function(args),
            "avg" | "average" => self.handle_average_function(args),
            "round" => self.handle_round_function(args),
            "floor" => self.handle_floor_function(args),
            "ceil" | "ceiling" => self.handle_ceil_function(args),
            "abs" | "absolute" => self.handle_abs_function(args),
            "contains" | "includes" => self.handle_contains_function(args),
            "startswith" | "begins_with" => self.handle_starts_with_function(args),
            "endswith" | "ends_with" => self.handle_ends_with_function(args),
            "lowercase" | "tolower" => self.handle_lowercase_function(args),
            "uppercase" | "toupper" => self.handle_uppercase_function(args),
            "trim" | "strip" => self.handle_trim_function(args),
            "split" => self.handle_split_function(args),
            "join" => self.handle_join_function(args),
            _ => {
                // Try to call custom user-defined function
                self.handle_custom_function(function, args, facts)
            }
        }
    }

    /// Handle logging functions (log, print, println)
    fn handle_log_function(&self, args: &[Value]) -> Result<String> {
        let message = if args.is_empty() {
            "".to_string()
        } else if args.len() == 1 {
            args[0].to_string()
        } else {
            args.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        };

        println!("📋 {}", message);
        Ok(message)
    }

    /// Handle update/refresh functions
    fn handle_update_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            Ok(format!("Updated: {}", arg.to_string()))
        } else {
            Ok("Updated".to_string())
        }
    }

    /// Handle timestamp function
    fn handle_timestamp_function(&self) -> Result<String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RuleEngineError::EvaluationError {
                message: format!("Failed to get timestamp: {}", e),
            })?
            .as_secs();
        Ok(timestamp.to_string())
    }

    /// Handle random function
    fn handle_random_function(&self, args: &[Value]) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Simple pseudo-random based on current time (for deterministic behavior in tests)
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let random_value = hasher.finish();

        if args.is_empty() {
            Ok((random_value % 100).to_string()) // 0-99
        } else if let Some(Value::Number(max)) = args.first() {
            let max_val = *max as u64;
            Ok((random_value % max_val).to_string())
        } else {
            Ok(random_value.to_string())
        }
    }

    /// Handle format function (simple sprintf-like)
    fn handle_format_function(&self, args: &[Value]) -> Result<String> {
        if args.is_empty() {
            return Ok("".to_string());
        }

        let template = args[0].to_string();
        let values: Vec<String> = args[1..].iter().map(|v| v.to_string()).collect();

        // Simple placeholder replacement: {0}, {1}, etc.
        let mut result = template;
        for (i, value) in values.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), value);
        }

        Ok(result)
    }

    /// Handle length/size functions
    fn handle_length_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            match arg {
                Value::String(s) => Ok(s.len().to_string()),
                Value::Array(arr) => Ok(arr.len().to_string()),
                Value::Object(obj) => Ok(obj.len().to_string()),
                _ => Ok("1".to_string()), // Single value has length 1
            }
        } else {
            Ok("0".to_string())
        }
    }

    /// Handle sum function
    fn handle_sum_function(&self, args: &[Value]) -> Result<String> {
        let sum = args.iter().fold(0.0, |acc, val| match val {
            Value::Number(n) => acc + n,
            Value::Integer(i) => acc + (*i as f64),
            _ => acc,
        });
        Ok(sum.to_string())
    }

    /// Handle max function
    fn handle_max_function(&self, args: &[Value]) -> Result<String> {
        let max = args.iter().fold(f64::NEG_INFINITY, |acc, val| match val {
            Value::Number(n) => acc.max(*n),
            Value::Integer(i) => acc.max(*i as f64),
            _ => acc,
        });
        Ok(max.to_string())
    }

    /// Handle min function
    fn handle_min_function(&self, args: &[Value]) -> Result<String> {
        let min = args.iter().fold(f64::INFINITY, |acc, val| match val {
            Value::Number(n) => acc.min(*n),
            Value::Integer(i) => acc.min(*i as f64),
            _ => acc,
        });
        Ok(min.to_string())
    }

    /// Handle average function
    fn handle_average_function(&self, args: &[Value]) -> Result<String> {
        if args.is_empty() {
            return Ok("0".to_string());
        }

        let (sum, count) = args.iter().fold((0.0, 0), |(sum, count), val| match val {
            Value::Number(n) => (sum + n, count + 1),
            Value::Integer(i) => (sum + (*i as f64), count + 1),
            _ => (sum, count),
        });

        if count > 0 {
            Ok((sum / count as f64).to_string())
        } else {
            Ok("0".to_string())
        }
    }

    /// Handle mathematical functions
    fn handle_round_function(&self, args: &[Value]) -> Result<String> {
        if let Some(Value::Number(n)) = args.first() {
            Ok(n.round().to_string())
        } else if let Some(Value::Integer(i)) = args.first() {
            Ok(i.to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "round() requires a numeric argument".to_string(),
            })
        }
    }

    fn handle_floor_function(&self, args: &[Value]) -> Result<String> {
        if let Some(Value::Number(n)) = args.first() {
            Ok(n.floor().to_string())
        } else if let Some(Value::Integer(i)) = args.first() {
            Ok(i.to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "floor() requires a numeric argument".to_string(),
            })
        }
    }

    fn handle_ceil_function(&self, args: &[Value]) -> Result<String> {
        if let Some(Value::Number(n)) = args.first() {
            Ok(n.ceil().to_string())
        } else if let Some(Value::Integer(i)) = args.first() {
            Ok(i.to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "ceil() requires a numeric argument".to_string(),
            })
        }
    }

    fn handle_abs_function(&self, args: &[Value]) -> Result<String> {
        if let Some(Value::Number(n)) = args.first() {
            Ok(n.abs().to_string())
        } else if let Some(Value::Integer(i)) = args.first() {
            Ok(i.abs().to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "abs() requires a numeric argument".to_string(),
            })
        }
    }

    /// Handle string functions
    fn handle_contains_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let haystack = args[0].to_string();
            let needle = args[1].to_string();
            Ok(haystack.contains(&needle).to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "contains() requires 2 arguments".to_string(),
            })
        }
    }

    fn handle_starts_with_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let text = args[0].to_string();
            let prefix = args[1].to_string();
            Ok(text.starts_with(&prefix).to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "startswith() requires 2 arguments".to_string(),
            })
        }
    }

    fn handle_ends_with_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let text = args[0].to_string();
            let suffix = args[1].to_string();
            Ok(text.ends_with(&suffix).to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "endswith() requires 2 arguments".to_string(),
            })
        }
    }

    fn handle_lowercase_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            Ok(arg.to_string().to_lowercase())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "lowercase() requires 1 argument".to_string(),
            })
        }
    }

    fn handle_uppercase_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            Ok(arg.to_string().to_uppercase())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "uppercase() requires 1 argument".to_string(),
            })
        }
    }

    fn handle_trim_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            Ok(arg.to_string().trim().to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "trim() requires 1 argument".to_string(),
            })
        }
    }

    fn handle_split_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let text = args[0].to_string();
            let delimiter = args[1].to_string();
            let parts: Vec<String> = text.split(&delimiter).map(|s| s.to_string()).collect();
            Ok(format!("{:?}", parts)) // Return as debug string for now
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "split() requires 2 arguments".to_string(),
            })
        }
    }

    fn handle_join_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let delimiter = args[0].to_string();
            let parts: Vec<String> = args[1..].iter().map(|v| v.to_string()).collect();
            Ok(parts.join(&delimiter))
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "join() requires at least 2 arguments".to_string(),
            })
        }
    }

    /// Handle custom user-defined functions
    fn handle_custom_function(
        &self,
        function: &str,
        args: &[Value],
        facts: &Facts,
    ) -> Result<String> {
        // Check if we have a registered custom function
        if let Some(custom_func) = self.custom_functions.get(function) {
            if self.config.debug_mode {
                println!("🎯 Calling registered function: {}({:?})", function, args);
            }

            match custom_func(args, facts) {
                Ok(result) => Ok(result.to_string()),
                Err(e) => Err(e),
            }
        } else {
            // Function not found - return error or placeholder
            if self.config.debug_mode {
                println!("⚠️ Custom function '{}' not registered", function);
            }

            Err(RuleEngineError::EvaluationError {
                message: format!("Function '{}' is not registered. Use engine.register_function() to add custom functions.", function),
            })
        }
    }

    /// Execute method call on object
    fn execute_method_call(
        &self,
        object_name: &str,
        method: &str,
        args: &[Value],
        facts: &Facts,
    ) -> Result<String> {
        // Get the object from facts
        let Some(object_value) = facts.get(object_name) else {
            return Err(RuleEngineError::EvaluationError {
                message: format!("Object '{}' not found in facts", object_name),
            });
        };

        let method_lower = method.to_lowercase();

        // Handle setter methods (set + property name)
        if method_lower.starts_with("set") && args.len() == 1 {
            return self.handle_setter_method(object_name, method, &args[0], object_value, facts);
        }

        // Handle getter methods (get + property name)
        if method_lower.starts_with("get") && args.is_empty() {
            return self.handle_getter_method(object_name, method, &object_value);
        }

        // Handle built-in methods
        match method_lower.as_str() {
            "tostring" => Ok(object_value.to_string()),
            "update" => {
                facts.add_value(object_name, object_value)?;
                Ok(format!("Updated {}", object_name))
            }
            "reset" => self.handle_reset_method(object_name, object_value, facts),
            _ => self.handle_property_access_or_fallback(
                object_name,
                method,
                args.len(),
                &object_value,
            ),
        }
    }

    /// Handle setter method calls (setXxx)
    fn handle_setter_method(
        &self,
        object_name: &str,
        method: &str,
        new_value: &Value,
        mut object_value: Value,
        facts: &Facts,
    ) -> Result<String> {
        let property_name = Self::extract_property_name_from_setter(method);

        match object_value {
            Value::Object(ref mut obj) => {
                obj.insert(property_name.clone(), new_value.clone());
                facts.add_value(object_name, object_value)?;
                Ok(format!(
                    "Set {} to {}",
                    property_name,
                    new_value.to_string()
                ))
            }
            _ => Err(RuleEngineError::EvaluationError {
                message: format!("Cannot call setter on non-object type: {}", object_name),
            }),
        }
    }

    /// Handle getter method calls (getXxx)
    fn handle_getter_method(
        &self,
        object_name: &str,
        method: &str,
        object_value: &Value,
    ) -> Result<String> {
        let property_name = Self::extract_property_name_from_getter(method);

        match object_value {
            Value::Object(obj) => {
                if let Some(value) = obj.get(&property_name) {
                    Ok(value.to_string())
                } else {
                    Err(RuleEngineError::EvaluationError {
                        message: format!(
                            "Property '{}' not found on object '{}'",
                            property_name, object_name
                        ),
                    })
                }
            }
            _ => Err(RuleEngineError::EvaluationError {
                message: format!("Cannot call getter on non-object type: {}", object_name),
            }),
        }
    }

    /// Handle reset method call
    fn handle_reset_method(
        &self,
        object_name: &str,
        mut object_value: Value,
        facts: &Facts,
    ) -> Result<String> {
        match object_value {
            Value::Object(ref mut obj) => {
                obj.clear();
                facts.add_value(object_name, object_value)?;
                Ok(format!("Reset {}", object_name))
            }
            _ => Err(RuleEngineError::EvaluationError {
                message: format!("Cannot reset non-object type: {}", object_name),
            }),
        }
    }

    /// Handle property access or fallback to generic method call
    fn handle_property_access_or_fallback(
        &self,
        object_name: &str,
        method: &str,
        arg_count: usize,
        object_value: &Value,
    ) -> Result<String> {
        if let Value::Object(obj) = object_value {
            // Try exact property name match
            if let Some(value) = obj.get(method) {
                return Ok(value.to_string());
            }

            // Try capitalized property name
            let capitalized_method = Self::capitalize_first_letter(method);
            if let Some(value) = obj.get(&capitalized_method) {
                return Ok(value.to_string());
            }
        }

        // Fallback to generic response
        Ok(format!(
            "Called {}.{} with {} args",
            object_name, method, arg_count
        ))
    }

    /// Extract property name from setter method (setXxx -> Xxx)
    fn extract_property_name_from_setter(method: &str) -> String {
        let property_name = &method[3..]; // Remove "set" prefix
        Self::capitalize_first_letter(property_name)
    }

    /// Extract property name from getter method (getXxx -> Xxx)
    fn extract_property_name_from_getter(method: &str) -> String {
        let property_name = &method[3..]; // Remove "get" prefix
        Self::capitalize_first_letter(property_name)
    }

    /// Helper function to capitalize first letter of a string
    fn capitalize_first_letter(s: &str) -> String {
        if s.is_empty() {
            return String::new();
        }
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// Resolve action parameters by replacing fact references with actual values
    fn resolve_action_parameters(
        &self,
        params: &HashMap<String, Value>,
        facts: &Facts,
    ) -> Result<HashMap<String, Value>> {
        let mut resolved = HashMap::new();

        for (key, value) in params {
            let resolved_value = match value {
                Value::String(s) => {
                    // Check if string looks like a fact reference (contains dot)
                    if s.contains('.') {
                        // Try to get the value from facts
                        if let Some(fact_value) = facts.get_nested(s) {
                            fact_value
                        } else {
                            // If not found, keep original string
                            value.clone()
                        }
                    } else {
                        value.clone()
                    }
                }
                _ => value.clone(),
            };
            resolved.insert(key.clone(), resolved_value);
        }

        Ok(resolved)
    }

    // 🔌 Plugin System Methods

    /// Load a plugin into the engine
    pub fn load_plugin(
        &mut self,
        plugin: std::sync::Arc<dyn crate::engine::plugin::RulePlugin>,
    ) -> Result<()> {
        // First register the plugin actions with this engine
        plugin.register_actions(self)?;
        plugin.register_functions(self)?;

        // Then store it in the plugin manager
        self.plugin_manager.load_plugin(plugin)
    }

    /// Unload a plugin from the engine
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        self.plugin_manager.unload_plugin(name)
    }

    /// Hot reload a plugin
    pub fn hot_reload_plugin(
        &mut self,
        name: &str,
        new_plugin: std::sync::Arc<dyn crate::engine::plugin::RulePlugin>,
    ) -> Result<()> {
        // Unload old plugin
        self.plugin_manager.unload_plugin(name)?;

        // Register new plugin actions
        new_plugin.register_actions(self)?;
        new_plugin.register_functions(self)?;

        // Load new plugin
        self.plugin_manager.load_plugin(new_plugin)
    }

    /// Get plugin information
    pub fn get_plugin_info(&self, name: &str) -> Option<&crate::engine::plugin::PluginMetadata> {
        self.plugin_manager.get_plugin_info(name)
    }

    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugin_manager.list_plugins()
    }

    /// Get plugin statistics
    pub fn get_plugin_stats(&self) -> PluginStats {
        self.plugin_manager.get_stats()
    }

    /// Check health of all plugins
    pub fn plugin_health_check(&mut self) -> HashMap<String, crate::engine::plugin::PluginHealth> {
        self.plugin_manager.plugin_health_check()
    }

    /// Configure plugin manager
    pub fn configure_plugins(&mut self, config: PluginConfig) {
        self.plugin_manager = PluginManager::new(config);
    }
}

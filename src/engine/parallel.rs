use crate::engine::{facts::Facts, knowledge_base::KnowledgeBase, rule::Rule};
use crate::errors::{Result, RuleEngineError};
use crate::types::{ActionType, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// Configuration for parallel rule execution
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Enable parallel execution
    pub enabled: bool,
    /// Maximum number of worker threads
    pub max_threads: usize,
    /// Minimum rules per thread to justify parallelization
    pub min_rules_per_thread: usize,
    /// Enable dependency analysis
    pub dependency_analysis: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_threads: num_cpus::get(),
            min_rules_per_thread: 2,
            dependency_analysis: true,
        }
    }
}

/// Type alias for custom function storage
type CustomFunctionMap =
    HashMap<String, Box<dyn Fn(&[Value], &Facts) -> Result<Value> + Send + Sync>>;

/// Rule execution context for parallel processing
#[derive(Debug, Clone)]
pub struct RuleExecutionContext {
    /// The rule that was executed
    pub rule: Rule,
    /// Whether the rule fired successfully
    pub fired: bool,
    /// Error message if execution failed
    pub error: Option<String>,
    /// Time taken to execute this rule
    pub execution_time: Duration,
}

/// Parallel rule execution engine
pub struct ParallelRuleEngine {
    config: ParallelConfig,
    custom_functions: Arc<RwLock<CustomFunctionMap>>,
}

impl ParallelRuleEngine {
    /// Create new parallel rule engine
    pub fn new(config: ParallelConfig) -> Self {
        Self {
            config,
            custom_functions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a custom function
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[Value], &Facts) -> Result<Value> + Send + Sync + 'static,
    {
        let mut functions = self.custom_functions.write().unwrap();
        functions.insert(name.to_string(), Box::new(func));
    }

    /// Execute rules with parallel processing
    pub fn execute_parallel(
        &self,
        knowledge_base: &KnowledgeBase,
        facts: &Facts,
        debug_mode: bool,
    ) -> Result<ParallelExecutionResult> {
        let start_time = Instant::now();

        if debug_mode {
            println!(
                "🚀 Starting parallel rule execution with {} rules",
                knowledge_base.get_rules().len()
            );
        }

        // Group rules by salience for ordered execution
        let salience_groups = self.group_rules_by_salience(&knowledge_base.get_rules());

        let mut total_fired = 0;
        let mut total_evaluated = 0;
        let mut execution_contexts = Vec::new();

        // Execute rules by salience level (highest first)
        let mut salience_levels: Vec<_> = salience_groups.keys().copied().collect();
        salience_levels.sort_by(|a, b| b.cmp(a)); // Descending order

        for salience in salience_levels {
            let rules_at_level = &salience_groups[&salience];

            if debug_mode {
                println!(
                    "⚡ Processing {} rules at salience level {}",
                    rules_at_level.len(),
                    salience
                );
            }

            // Decide whether to use parallel execution for this level
            let should_parallelize = self.should_parallelize(rules_at_level);

            let contexts = if should_parallelize {
                self.execute_rules_parallel(rules_at_level, facts, debug_mode)?
            } else {
                self.execute_rules_sequential(rules_at_level, facts, debug_mode)?
            };

            // Count results
            for context in &contexts {
                total_evaluated += 1;
                if context.fired {
                    total_fired += 1;
                }
            }

            execution_contexts.extend(contexts);
        }

        Ok(ParallelExecutionResult {
            total_rules_evaluated: total_evaluated,
            total_rules_fired: total_fired,
            execution_time: start_time.elapsed(),
            parallel_speedup: self.calculate_speedup(&execution_contexts),
            execution_contexts,
        })
    }

    /// Group rules by their salience level
    fn group_rules_by_salience(&self, rules: &[Rule]) -> HashMap<i32, Vec<Rule>> {
        let mut groups = HashMap::new();
        for rule in rules {
            if rule.enabled {
                groups
                    .entry(rule.salience)
                    .or_insert_with(Vec::new)
                    .push(rule.clone());
            }
        }
        groups
    }

    /// Determine if rules should be executed in parallel
    fn should_parallelize(&self, rules: &[Rule]) -> bool {
        self.config.enabled && rules.len() >= self.config.min_rules_per_thread && rules.len() >= 2
    }

    /// Execute rules in parallel within the same salience level
    fn execute_rules_parallel(
        &self,
        rules: &[Rule],
        facts: &Facts,
        debug_mode: bool,
    ) -> Result<Vec<RuleExecutionContext>> {
        let results = Arc::new(Mutex::new(Vec::new()));
        let facts_arc = Arc::new(facts.clone());
        let functions_arc = Arc::clone(&self.custom_functions);

        // Create worker threads
        let chunk_size = rules.len().div_ceil(self.config.max_threads);
        let chunks: Vec<_> = rules.chunks(chunk_size).collect();

        let handles: Vec<_> = chunks
            .into_iter()
            .enumerate()
            .map(|(thread_id, chunk)| {
                let chunk = chunk.to_vec();
                let results_clone = Arc::clone(&results);
                let facts_clone = Arc::clone(&facts_arc);
                let functions_clone = Arc::clone(&functions_arc);

                thread::spawn(move || {
                    if debug_mode {
                        println!("  🧵 Thread {} processing {} rules", thread_id, chunk.len());
                    }

                    let mut thread_results = Vec::new();
                    for rule in chunk {
                        let start = Instant::now();
                        let fired = Self::evaluate_rule_conditions(&rule, &facts_clone);

                        if fired {
                            if debug_mode {
                                println!("    🔥 Rule '{}' fired", rule.name);
                            }

                            // Execute actions (simplified for demo)
                            for action in &rule.actions {
                                if let Err(e) = Self::execute_action_parallel(
                                    action,
                                    &facts_clone,
                                    &functions_clone,
                                ) {
                                    if debug_mode {
                                        println!("    ❌ Action failed: {}", e);
                                    }
                                }
                            }
                        }

                        thread_results.push(RuleExecutionContext {
                            rule: rule.clone(),
                            fired,
                            error: None,
                            execution_time: start.elapsed(),
                        });
                    }

                    let mut results = results_clone.lock().unwrap();
                    results.extend(thread_results);
                })
            })
            .collect();

        // Wait for all threads to complete
        for handle in handles {
            handle
                .join()
                .map_err(|_| RuleEngineError::EvaluationError {
                    message: "Thread panicked during parallel execution".to_string(),
                })?;
        }

        let results = results.lock().unwrap();
        Ok(results.clone())
    }

    /// Execute rules sequentially (fallback)
    fn execute_rules_sequential(
        &self,
        rules: &[Rule],
        facts: &Facts,
        debug_mode: bool,
    ) -> Result<Vec<RuleExecutionContext>> {
        let mut contexts = Vec::new();
        let functions_arc = Arc::clone(&self.custom_functions);

        for rule in rules {
            let start = Instant::now();
            let fired = Self::evaluate_rule_conditions(rule, facts);

            if fired && debug_mode {
                println!("    🔥 Rule '{}' fired", rule.name);
            }

            if fired {
                // Execute actions
                for action in &rule.actions {
                    if let Err(e) = Self::execute_action_parallel(action, facts, &functions_arc) {
                        if debug_mode {
                            println!("    ❌ Action failed: {}", e);
                        }
                    }
                }
            }

            contexts.push(RuleExecutionContext {
                rule: rule.clone(),
                fired,
                error: None,
                execution_time: start.elapsed(),
            });
        }

        Ok(contexts)
    }

    /// Simplified rule condition evaluation
    /// TODO: This is a simplified version for parallel demo purposes
    /// Real implementation should use proper condition evaluation like main engine
    fn evaluate_rule_conditions(rule: &Rule, _facts: &Facts) -> bool {
        // For demo purposes, just return true if rule has conditions
        // In real implementation, this would evaluate the actual conditions
        // using the same logic as engine.rs evaluate_conditions() method
        !rule.actions.is_empty()
    }

    /// Execute action with parallel-safe function calls
    fn execute_action_parallel(
        action: &ActionType,
        facts: &Facts,
        functions: &Arc<RwLock<CustomFunctionMap>>,
    ) -> Result<()> {
        match action {
            ActionType::Call { function, args } => {
                let functions_guard = functions.read().unwrap();
                if let Some(func) = functions_guard.get(function) {
                    let _result = func(args, facts)?;
                }
                Ok(())
            }
            ActionType::MethodCall { .. } => {
                // Simplified method call handling
                Ok(())
            }
            ActionType::Set { .. } => {
                // Simplified assignment handling
                Ok(())
            }
            ActionType::Log { message } => {
                println!("     📋 {}", message);
                Ok(())
            }
            ActionType::Update { .. } => {
                // Simplified update handling
                Ok(())
            }
            ActionType::Custom { .. } => {
                // Simplified custom action handling
                Ok(())
            }
            ActionType::ActivateAgendaGroup { .. } => {
                // Workflow actions not supported in parallel execution
                Ok(())
            }
            ActionType::ScheduleRule { .. } => {
                // Workflow actions not supported in parallel execution  
                Ok(())
            }
            ActionType::CompleteWorkflow { .. } => {
                // Workflow actions not supported in parallel execution
                Ok(())
            }
            ActionType::SetWorkflowData { .. } => {
                // Workflow actions not supported in parallel execution
                Ok(())
            }
        }
    }

    /// Calculate parallel speedup
    fn calculate_speedup(&self, contexts: &[RuleExecutionContext]) -> f64 {
        if contexts.is_empty() {
            return 1.0;
        }

        let total_time: Duration = contexts.iter().map(|c| c.execution_time).sum();
        let max_time = contexts
            .iter()
            .map(|c| c.execution_time)
            .max()
            .unwrap_or(Duration::ZERO);

        if max_time.as_nanos() > 0 {
            total_time.as_nanos() as f64 / max_time.as_nanos() as f64
        } else {
            1.0
        }
    }
}

/// Result of parallel rule execution
#[derive(Debug)]
pub struct ParallelExecutionResult {
    /// Total number of rules evaluated
    pub total_rules_evaluated: usize,
    /// Total number of rules that fired
    pub total_rules_fired: usize,
    /// Total execution time
    pub execution_time: Duration,
    /// Detailed execution contexts for each rule
    pub execution_contexts: Vec<RuleExecutionContext>,
    /// Parallel speedup factor
    pub parallel_speedup: f64,
}

impl ParallelExecutionResult {
    /// Get execution statistics
    pub fn get_stats(&self) -> String {
        format!(
            "📊 Parallel Execution Stats:\n   Rules evaluated: {}\n   Rules fired: {}\n   Execution time: {:?}\n   Parallel speedup: {:.2}x",
            self.total_rules_evaluated,
            self.total_rules_fired,
            self.execution_time,
            self.parallel_speedup
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::{Condition, ConditionGroup};
    use crate::types::{Operator, Value};

    #[test]
    fn test_parallel_config_default() {
        let config = ParallelConfig::default();
        assert!(config.enabled);
        assert!(config.max_threads > 0);
        assert_eq!(config.min_rules_per_thread, 2);
    }

    #[test]
    fn test_parallel_engine_creation() {
        let config = ParallelConfig::default();
        let engine = ParallelRuleEngine::new(config);
        assert!(engine.custom_functions.read().unwrap().is_empty());
    }

    #[test]
    fn test_salience_grouping() {
        let config = ParallelConfig::default();
        let engine = ParallelRuleEngine::new(config);

        let rules = vec![
            Rule::new(
                "Rule1".to_string(),
                ConditionGroup::Single(Condition::new(
                    "test".to_string(),
                    Operator::Equal,
                    Value::Boolean(true),
                )),
                vec![],
            )
            .with_priority(10),
            Rule::new(
                "Rule2".to_string(),
                ConditionGroup::Single(Condition::new(
                    "test".to_string(),
                    Operator::Equal,
                    Value::Boolean(true),
                )),
                vec![],
            )
            .with_priority(10),
            Rule::new(
                "Rule3".to_string(),
                ConditionGroup::Single(Condition::new(
                    "test".to_string(),
                    Operator::Equal,
                    Value::Boolean(true),
                )),
                vec![],
            )
            .with_priority(5),
        ];

        let groups = engine.group_rules_by_salience(&rules);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[&10].len(), 2);
        assert_eq!(groups[&5].len(), 1);
    }
}

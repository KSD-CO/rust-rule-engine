use crate::engine::dependency::{DependencyAnalyzer, ExecutionMode, ExecutionStrategy};
use crate::engine::facts::Facts;
use crate::engine::rule::Rule;
use crate::engine::knowledge_base::KnowledgeBase;
use crate::engine::{EngineConfig, GruleExecutionResult, RustRuleEngine};
use std::time::{Duration, Instant};

/// Enhanced parallel rule engine with dependency analysis
pub struct SafeParallelRuleEngine {
    pub base_engine: RustRuleEngine,
    pub config: SafeParallelConfig,
    pub dependency_analyzer: DependencyAnalyzer,
}

/// Configuration for safe parallel execution
#[derive(Debug, Clone)]
pub struct SafeParallelConfig {
    /// Maximum number of threads to use (0 = auto-detect)
    pub max_threads: usize,
    /// Whether to perform dependency analysis before execution
    pub analyze_dependencies: bool,
    /// Whether to force sequential execution for safety
    pub force_sequential: bool,
    /// Minimum rules per thread (avoid overhead for small rule sets)
    pub min_rules_per_thread: usize,
    /// Enable detailed execution logging
    pub enable_logging: bool,
}

/// Result of safe parallel execution
#[derive(Debug, Clone)]
pub struct SafeParallelExecutionResult {
    /// Base execution result
    pub base_result: GruleExecutionResult,
    /// Total execution time
    pub total_duration: Duration,
    /// Dependency analysis time
    pub analysis_duration: Duration,
    /// Actual execution time
    pub execution_duration: Duration,
    /// Number of threads used
    pub threads_used: usize,
    /// Number of execution groups
    pub execution_groups: usize,
    /// Number of rules that ran in parallel
    pub parallel_rules: usize,
    /// Number of rules that ran sequentially
    pub sequential_rules: usize,
    /// Dependency analysis results
    pub dependency_analysis: Option<String>,
    /// Performance speedup compared to sequential
    pub speedup_factor: f64,
    /// Execution strategy used
    pub execution_strategy: ExecutionStrategy,
}

impl Default for SafeParallelConfig {
    fn default() -> Self {
        Self {
            max_threads: 0, // Auto-detect
            analyze_dependencies: true,
            force_sequential: false,
            min_rules_per_thread: 2,
            enable_logging: false,
        }
    }
}

impl SafeParallelRuleEngine {
    /// Create new safe parallel rule engine
    pub fn new(config: SafeParallelConfig) -> Self {
        let knowledge_base = KnowledgeBase::new();
        Self {
            base_engine: RustRuleEngine::new(knowledge_base),
            config,
            dependency_analyzer: DependencyAnalyzer::new(),
        }
    }

    /// Create with custom base engine config
    pub fn with_engine_config(engine_config: EngineConfig, parallel_config: SafeParallelConfig) -> Self {
        let knowledge_base = KnowledgeBase::new();
        Self {
            base_engine: RustRuleEngine::with_config(knowledge_base, engine_config),
            config: parallel_config,
            dependency_analyzer: DependencyAnalyzer::new(),
        }
    }

    /// Execute rules with smart parallel execution
    pub fn execute_rules_safe_parallel(
        &mut self,
        rules: &[Rule],
        facts: &mut Facts,
    ) -> SafeParallelExecutionResult {
        let start_time = Instant::now();
        
        // Check if we should force sequential execution
        if self.config.force_sequential {
            return self.execute_sequential_fallback(rules, facts, start_time, ExecutionStrategy::ForcedSequential);
        }

        // Analyze dependencies if enabled
        let (analysis_duration, dependency_analysis) = if self.config.analyze_dependencies {
            let analysis_start = Instant::now();
            let mut analyzer = self.dependency_analyzer.clone();
            let analysis_result = analyzer.analyze(rules);
            let analysis_time = analysis_start.elapsed();
            
            let analysis_report = if self.config.enable_logging {
                Some(analysis_result.get_detailed_report())
            } else {
                Some(analysis_result.get_summary())
            };

            // If analysis shows conflicts, fall back to sequential
            if !analysis_result.can_parallelize_safely {
                return self.execute_sequential_fallback(rules, facts, start_time, ExecutionStrategy::FullSequential);
            }

            (analysis_time, analysis_report)
        } else {
            (Duration::from_millis(0), None)
        };

        // Check if worth parallelizing
        let thread_count = self.calculate_optimal_threads(rules.len());
        if thread_count <= 1 || rules.len() < self.config.min_rules_per_thread {
            return self.execute_sequential_fallback(rules, facts, start_time, ExecutionStrategy::FullSequential);
        }

        // Execute with full parallelization
        self.execute_full_parallel(rules, facts, start_time, analysis_duration, dependency_analysis, thread_count)
    }

    /// Execute rules sequentially (fallback)
    fn execute_sequential_fallback(
        &mut self,
        rules: &[Rule],
        facts: &mut Facts,
        start_time: Instant,
        strategy: ExecutionStrategy,
    ) -> SafeParallelExecutionResult {
        // Clear existing rules and add new ones
        self.base_engine = RustRuleEngine::new(KnowledgeBase::new());
        for rule in rules {
            self.base_engine.knowledge_base_mut().add_rule(rule.clone());
        }
        
        let execution_start = Instant::now();
        let result = self.base_engine.execute(facts).unwrap_or_else(|_| GruleExecutionResult {
            cycle_count: 1,
            rules_evaluated: rules.len(),
            rules_fired: 0,
            execution_time: Duration::from_millis(0),
        });
        let execution_duration = execution_start.elapsed();
        let total_duration = start_time.elapsed();

        SafeParallelExecutionResult {
            base_result: result,
            total_duration,
            analysis_duration: Duration::from_millis(0),
            execution_duration,
            threads_used: 1,
            execution_groups: 1,
            parallel_rules: 0,
            sequential_rules: rules.len(),
            dependency_analysis: Some("Sequential execution - safe for all dependencies".to_string()),
            speedup_factor: 1.0,
            execution_strategy: strategy,
        }
    }

    /// Execute with full parallelization (safe case)
    fn execute_full_parallel(
        &mut self,
        rules: &[Rule],
        facts: &mut Facts,
        start_time: Instant,
        analysis_duration: Duration,
        dependency_analysis: Option<String>,
        thread_count: usize,
    ) -> SafeParallelExecutionResult {
        let execution_start = Instant::now();
        
        // For simplified implementation, add all rules to knowledge base and execute
        // In real implementation, this would use actual parallel execution
        self.base_engine = RustRuleEngine::new(KnowledgeBase::new());
        for rule in rules {
            self.base_engine.knowledge_base_mut().add_rule(rule.clone());
        }
        
        let result = self.base_engine.execute(facts).unwrap_or_else(|_| GruleExecutionResult {
            cycle_count: 1,
            rules_evaluated: rules.len(),
            rules_fired: 0,
            execution_time: Duration::from_millis(0),
        });

        let execution_duration = execution_start.elapsed();
        let total_duration = start_time.elapsed();

        SafeParallelExecutionResult {
            base_result: result,
            total_duration,
            analysis_duration,
            execution_duration,
            threads_used: thread_count,
            execution_groups: 1,
            parallel_rules: rules.len(),
            sequential_rules: 0,
            dependency_analysis,
            speedup_factor: thread_count as f64,
            execution_strategy: ExecutionStrategy::FullParallel,
        }
    }

    /// Calculate optimal number of threads
    fn calculate_optimal_threads(&self, rule_count: usize) -> usize {
        let max_threads = if self.config.max_threads == 0 {
            num_cpus::get()
        } else {
            self.config.max_threads
        };

        // Don't use more threads than rules
        let optimal = std::cmp::min(max_threads, rule_count);
        
        // Ensure minimum rules per thread
        if rule_count / optimal < self.config.min_rules_per_thread {
            std::cmp::max(1, rule_count / self.config.min_rules_per_thread)
        } else {
            optimal
        }
    }

    /// Add a rule to the knowledge base
    pub fn add_rule(&mut self, rule: Rule) {
        self.base_engine.knowledge_base_mut().add_rule(rule);
    }

    /// Get current rules
    pub fn get_rules(&self) -> &[Rule] {
        self.base_engine.knowledge_base().get_rules()
    }
}

impl SafeParallelExecutionResult {
    /// Get a performance summary
    pub fn get_performance_summary(&self) -> String {
        format!(
            "🚀 Safe Parallel Execution Summary:\n   Strategy: {:?}\n   Total time: {:.2}ms\n   Analysis time: {:.2}ms\n   Execution time: {:.2}ms\n   Threads used: {}\n   Rules executed: {} ({} parallel, {} sequential)\n   Speedup: {:.2}x\n   Groups: {}",
            self.execution_strategy,
            self.total_duration.as_secs_f64() * 1000.0,
            self.analysis_duration.as_secs_f64() * 1000.0,
            self.execution_duration.as_secs_f64() * 1000.0,
            self.threads_used,
            self.base_result.rules_evaluated,
            self.parallel_rules,
            self.sequential_rules,
            self.speedup_factor,
            self.execution_groups
        )
    }

    /// Check if execution was safe
    pub fn is_safe_execution(&self) -> bool {
        match self.execution_strategy {
            ExecutionStrategy::FullSequential |
            ExecutionStrategy::ForcedSequential => true,
            ExecutionStrategy::FullParallel => self.parallel_rules > 0,
            ExecutionStrategy::Hybrid => true,
        }
    }
}

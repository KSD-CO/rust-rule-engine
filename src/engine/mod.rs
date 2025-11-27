/// Agenda and activation group management
pub mod agenda;
/// Advanced analytics and performance monitoring
pub mod analytics;
/// Dependency analysis for safe parallel execution
pub mod dependency;
/// Main rule execution engine
#[allow(clippy::module_inception)]
pub mod engine;
/// Facts (working memory) for rule execution
pub mod facts;
/// Knowledge base for rule storage and management
pub mod knowledge_base;
/// Parallel rule execution engine
pub mod parallel;
/// Pattern matching for complex conditions
pub mod pattern_matcher;
/// Plugin system for extensibility
pub mod plugin;
/// Rule execution engine and core functionality
pub mod rule;
/// Rule templates for dynamic rule generation
pub mod template;
/// Workflow engine for rule chaining and sequential execution
pub mod workflow;
pub mod coverage; // Adding coverage module
/// Shared condition evaluation logic for both forward and backward chaining
pub mod condition_evaluator;
/// Module system for namespace isolation (CLIPS-inspired defmodule)
pub mod module;

// Re-export main components for easy access
pub use agenda::{ActivationGroupManager, AgendaManager};
pub use analytics::{AnalyticsConfig, ExecutionEvent, OverallStats, RuleAnalytics, RuleMetrics};
pub use condition_evaluator::ConditionEvaluator;
pub use dependency::{
    DependencyAnalysisResult, DependencyAnalyzer, ExecutionGroup, ExecutionMode, ExecutionStrategy,
};
pub use engine::{EngineConfig, GruleExecutionResult, RustRuleEngine};
pub use parallel::{ParallelConfig, ParallelExecutionResult, ParallelRuleEngine};
pub use template::{ParameterType, RuleTemplate, TemplateManager};
pub use workflow::{
    ScheduledTask, WorkflowEngine, WorkflowResult, WorkflowState, WorkflowStats, WorkflowStatus,
};

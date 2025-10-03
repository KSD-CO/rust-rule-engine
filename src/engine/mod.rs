/// Main rule execution engine
#[allow(clippy::module_inception)]
pub mod engine;
/// Facts (working memory) for rule execution
pub mod facts;
/// Knowledge base for rule storage and management
pub mod knowledge_base;
/// Parallel rule execution engine
pub mod parallel;
/// Dependency analysis for safe parallel execution
pub mod dependency;
/// Rule definition and condition handling
pub mod rule;
/// Rule templates for dynamic rule generation
pub mod template;

// Re-export main components for easy access
pub use engine::{EngineConfig, GruleExecutionResult, RustRuleEngine};
pub use parallel::{ParallelConfig, ParallelExecutionResult, ParallelRuleEngine};
pub use template::{ParameterType, RuleTemplate, TemplateManager};
pub use dependency::{DependencyAnalyzer, DependencyAnalysisResult, ExecutionGroup, ExecutionMode, ExecutionStrategy};

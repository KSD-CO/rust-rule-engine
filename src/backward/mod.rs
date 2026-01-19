//! # Backward Chaining Module
//!
//! Goal-driven reasoning system for rust-rule-engine.
//! This module is only available when the `backward-chaining` feature is enabled.
//!
//! ## Overview
//!
//! Backward chaining works by starting from a goal and working backwards
//! to find rules and facts that can prove that goal.
//!
//! ## Example
//!
//! ```ignore
//! use rust_rule_engine::backward::*;
//!
//! let bc_engine = BackwardEngine::new(kb);
//! let result = bc_engine.query("User.IsVIP == true", &facts)?;
//!
//! if result.provable {
//!     println!("Goal is provable!");
//!     println!("Proof: {:?}", result.proof_trace);
//! }
//! ```

pub mod aggregation;
pub mod backward_engine;
pub mod conclusion_index;
pub mod disjunction;
pub mod explanation;
pub mod expression;
pub mod goal;
pub mod grl_query;
pub mod nested;
pub mod optimizer;
pub mod proof_graph;
pub mod proof_tree;
pub mod query;
pub mod rule_executor;
pub mod search;
pub mod unification;

// Re-export main types
pub use aggregation::{apply_aggregate, parse_aggregate_query, AggregateFunction, AggregateQuery};
pub use backward_engine::{BackwardConfig, BackwardEngine};
pub use conclusion_index::{ConclusionIndex, IndexStats};
pub use disjunction::{Disjunction, DisjunctionParser, DisjunctionResult};
pub use explanation::{Explanation, ExplanationBuilder, ExplanationStep, StepResult};
pub use expression::{Expression, ExpressionParser};
pub use goal::{Goal, GoalManager, GoalStatus};
pub use grl_query::{GRLQuery, GRLQueryExecutor, GRLQueryParser, GRLSearchStrategy, QueryAction};
pub use nested::{
    NestedQuery, NestedQueryEvaluator, NestedQueryParser, NestedQueryResult, NestedQueryStats,
    Query,
};
pub use optimizer::{JoinOptimizer, OptimizationStats, OptimizerConfig, QueryOptimizer};
pub use proof_graph::{
    FactKey, Justification, ProofGraph, ProofGraphNode, ProofGraphStats, SharedProofGraph,
};
pub use proof_tree::{ProofNode, ProofNodeType, ProofStats, ProofTree};
pub use query::{ProofTrace, QueryResult};
pub use rule_executor::RuleExecutor;
pub use search::{SearchResult, SearchStrategy, Solution};
pub use unification::{Bindings, Unifier};

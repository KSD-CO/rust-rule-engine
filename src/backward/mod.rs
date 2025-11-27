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

pub mod goal;
pub mod search;
pub mod backward_engine;
pub mod query;
pub mod grl_query;
pub mod expression;
pub mod rule_executor;
pub mod unification;
pub mod conclusion_index;

// Re-export main types
pub use goal::{Goal, GoalStatus, GoalManager};
pub use search::{SearchStrategy, SearchResult, Solution};
pub use backward_engine::{BackwardEngine, BackwardConfig};
pub use query::{QueryResult, ProofTrace};
pub use grl_query::{GRLQuery, GRLQueryParser, GRLQueryExecutor, GRLSearchStrategy, QueryAction};
pub use expression::{Expression, ExpressionParser};
pub use rule_executor::RuleExecutor;
pub use unification::{Bindings, Unifier};
pub use conclusion_index::{ConclusionIndex, IndexStats};

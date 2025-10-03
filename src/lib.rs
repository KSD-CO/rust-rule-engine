//! # Rust Rule Engine - GRL Edition
//!
//! A high-performance rule engine for Rust with **GRL (Grule Rule Language)** support,
//! featuring method calls, object interactions, and complex condition evaluation.
//!
//! ## Features
//!
//! - **🔥 GRL Support**: Full Grule-compatible syntax
//! - **🎯 Method Calls**: `$Object.method(args)` and property access
//! - **📊 Knowledge Base**: Centralized rule management with salience
//! - **💾 Working Memory**: Facts system for complex object interactions
//! - **⚡ High Performance**: Optimized execution with cycle detection
//! - **🔄 Arithmetic**: Complex calculations in conditions and actions
//! - **🛡️ Type Safety**: Rust's type system ensures runtime safety
//!
//! ## Quick Start
//!
//! ```rust
//! use rust_rule_engine::*;
//!
//! fn main() -> Result<()> {
//!     // Create Knowledge Base
//!     let kb = KnowledgeBase::new("Demo");
//!     
//!     // Define GRL rule
//!     let rule = r#"
//!     rule "AgeCheck" salience 10 {
//!         when
//!             User.Age >= 18 && User.Country == "US"
//!         then
//!             User.IsAdult = true;
//!             User.DiscountRate = 0.10;
//!     }
//!     "#;
//!     
//!     // Parse and add rule
//!     let rules = GRLParser::parse_rules(rule)?;
//!     let parsed_rule = rules.into_iter().next().unwrap();
//!     kb.add_rule(parsed_rule)?;
//!     
//!     // Create engine
//!     let engine = RustRuleEngine::new(kb);
//!     
//!     // Create facts
//!     let facts = Facts::new();
//!     let user = FactHelper::create_user("john", 25, "john@email.com", "US", false);
//!     facts.add_value("User", user)?;
//!     
//!     // Execute rules
//!     let result = engine.execute(&facts)?;
//!     println!("Rules fired: {}", result.rules_fired);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Advanced Features
//!
//! ### Method Calls and Object Interactions
//!
//! ```rust
//! # use rust_rule_engine::*;
//! let speedup_rule = r#"
//! rule "SpeedUp" salience 10 {
//!     when
//!         $TestCar : TestCarClass( speedUp == true && speed < maxSpeed )
//!     then
//!         $TestCar.setSpeed($TestCar.Speed + $TestCar.SpeedIncrement);
//!         update($TestCar);
//! }
//! "#;
//! ```
//!
//! ### E-commerce Rules
//!
//! ```rust
//! # use rust_rule_engine::*;
//! let discount_rule = r#"
//! rule "PremiumDiscount" salience 20 {
//!     when
//!         Customer.Membership == "premium" && Order.Total > 100
//!     then
//!         Order.DiscountRate = 0.15;
//!         Order.FreeShipping = true;
//! }
//! "#;
//! ```
//!
//! ## Core Components
//!
//! - [`KnowledgeBase`]: Manages collections of rules with metadata
//! - [`Facts`]: Working memory for data objects and rule evaluation  
//! - [`RustRuleEngine`]: Executes rules with configurable options
//! - [`GRLParser`]: Parses Grule Rule Language syntax
//! - [`FactHelper`]: Utility functions for creating data objects
//! - [`Value`]: Flexible data type system supporting objects, arrays, primitives

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Rule execution engine and related components
pub mod engine;
/// Error types and result handling
pub mod errors;
/// Rule parsing and language support  
pub mod parser;
/// Streaming rule engine for real-time event processing
#[cfg(feature = "streaming")]
pub mod streaming;
/// Core type definitions for values, operators, and actions
pub mod types;

// Re-export core types for easy access
pub use errors::{Result, RuleEngineError};
pub use types::{ActionType, LogicalOperator, Operator, Value};

// Re-export Grule-style components
pub use engine::engine::{EngineConfig, GruleExecutionResult, RustRuleEngine};
pub use engine::facts::{FactHelper, Facts};
pub use engine::knowledge_base::KnowledgeBase;
pub use engine::rule::{Condition, ConditionGroup, Rule};

// Re-export parsers
pub use parser::grl_parser::GRLParser;

/// Builder pattern for creating a RustRuleEngine with various configurations
pub struct RuleEngineBuilder {
    kb: KnowledgeBase,
    config: EngineConfig,
}

impl RuleEngineBuilder {
    /// Create a new RuleEngineBuilder
    pub fn new() -> Self {
        Self {
            kb: KnowledgeBase::new("DefaultKB"),
            config: EngineConfig::default(),
        }
    }

    /// Add rules from a .grl file
    pub fn with_rule_file<P: AsRef<std::path::Path>>(self, path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let rules = GRLParser::parse_rules(&content)?;

        for rule in rules {
            self.kb.add_rule(rule)?;
        }

        Ok(self)
    }

    /// Add rules from inline GRL string
    pub fn with_inline_grl(self, grl_content: &str) -> Result<Self> {
        let rules = GRLParser::parse_rules(grl_content)?;

        for rule in rules {
            self.kb.add_rule(rule)?;
        }

        Ok(self)
    }

    /// Set engine configuration
    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the RustRuleEngine
    pub fn build(self) -> RustRuleEngine {
        RustRuleEngine::with_config(self.kb, self.config)
    }
}

impl Default for RuleEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

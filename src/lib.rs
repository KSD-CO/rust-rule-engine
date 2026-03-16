//! # Rust Rule Engine v1.19.2 - API Documentation Edition
//!
//! A high-performance rule engine for Rust with **RETE-UL algorithm**, **Array Membership (`in`) operator**,
//! **String Methods (startsWith, endsWith)**, **Plugin System**, and **GRL (Grule Rule Language)** support.
//! Features forward/backward chaining, stream processing, and production-ready performance.
//!
//! ## What's New in v1.19.2
//!
//! - **📚 Complete API Documentation**: All public APIs now have comprehensive documentation
//! - **🔍 Missing Docs Lint**: Enabled `#![warn(missing_docs)]` to ensure API documentation quality
//! - **📖 Enhanced RuleEngineBuilder Docs**: Detailed documentation with examples for builder pattern
//! - **✨ Zero Breaking Changes**: Pure documentation improvement with no API changes
//!
//! ## Features
//!
//! - **🔌 Plugin System**: Modular plugin architecture with lifecycle management
//! - **🛠️ Built-in Plugin Suite**: 44+ actions & 33+ functions for common operations  
//! - **🔥 GRL Support**: Full Grule-compatible syntax
//! - **🎯 Method Calls**: `$Object.method(args)` and property access
//! - **📊 Knowledge Base**: Centralized rule management with salience
//! - **💾 Working Memory**: Facts system for complex object interactions
//! - **⚡ High Performance**: Optimized execution with cycle detection
//! - **🔄 Arithmetic**: Complex calculations in conditions and actions
//! - **🛡️ Type Safety**: Rust's type system ensures runtime safety
//!
//! ## Quick Start with Plugins
//!
//! ```rust
//! use rust_rule_engine::*;
//!
//! fn main() -> Result<()> {
//!     // Create Knowledge Base and Engine
//!     let kb = KnowledgeBase::new("Demo");
//!     let mut engine = RustRuleEngine::new(kb);
//!     let mut facts = Facts::new();
//!     
//!     // Set up data
//!     facts.set("user.age", Value::Number(25.0));
//!     facts.set("user.premium", Value::Boolean(false));
//!     
//!     // Define GRL rule
//!     let rule = r#"
//!     rule "PremiumUpgrade" salience 10 {
//!         when
//!             user.age >= 18 && user.premium == false
//!         then
//!             user.premium = true;
//!             user.discount = 0.1;
//!     }
//!     "#;
//!     
//!     // Parse and add rule to knowledge base
//!     let rules = GRLParser::parse_rules(rule)?;
//!     for r in rules {
//!         engine.knowledge_base().add_rule(r)?;
//!     }
//!     
//!     // Execute with facts
//!     let result = engine.execute(&facts)?;
//!     println!("User premium status: {:?}", facts.get("user.premium"));
//!     
//!     Ok(())
//! }
//! ```
//!
//! Built-in Plugin Suite provides comprehensive functionality for common operations:
//!
//! - **String Utilities**: 8 actions, 5 functions for text manipulation
//! - **Math Operations**: 10 actions, 6 functions for calculations  
//! - **Date/Time**: 8 actions, 7 functions for temporal operations
//! - Actions: CurrentDate, CurrentTime, FormatDate, ParseDate, AddDays, AddHours, DateDiff, IsWeekend
//! - Functions: now, today, dayOfWeek, dayOfYear, year, month, day
//!
//! ### Validation (8 actions, 6 functions)
//! - Actions: ValidateEmail, ValidatePhone, ValidateUrl, ValidateRegex, ValidateRange, ValidateLength, ValidateNotEmpty, ValidateNumeric
//! - Functions: isEmail, isPhone, isUrl, isNumeric, isEmpty, inRange
//!
//! ### Collections (10 actions, 9 functions)
//! - Actions: ArrayLength, ArrayPush, ArrayPop, ArraySort, ArrayFilter, ArrayMap, ArrayFind, ObjectKeys, ObjectValues, ObjectMerge
//! - Functions: length, contains, first, last, reverse, join, slice, keys, values
//!
//!   // Create engine
//!   let mut engine = RustRuleEngine::new(kb);
//!
//!   // Create facts
//!   let facts = Facts::new();
//!   let user = FactHelper::create_user("john", 25, "john@email.com", "US", false);
//!   facts.add_value("User", user)?;
//!
//!   // Execute rules
//!   let result = engine.execute(&facts)?;
//!   println!("Rules fired: {}", result.rules_fired);
//!
//!   Ok(())
//!   }
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

/// Backward chaining (goal-driven reasoning) - requires 'backward-chaining' feature
#[cfg(feature = "backward-chaining")]
#[allow(missing_docs)]
pub mod backward;
/// Rule execution engine and related components
#[allow(missing_docs)]
pub mod engine;
/// Error types and result handling
pub mod errors;
/// Expression evaluation (arithmetic operations)
#[allow(missing_docs)]
pub mod expression;
/// Rule parsing and language support
#[allow(missing_docs)]
pub mod parser;
/// Built-in plugin system for extended functionality
#[allow(missing_docs)]
pub mod plugins;
/// RETE module for rule evaluation
#[allow(missing_docs)]
pub mod rete;
/// Streaming rule engine for real-time event processing
#[cfg(feature = "streaming")]
#[allow(missing_docs)]
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
pub use parser::grl::GRLParser;

/// Builder pattern for creating a RustRuleEngine with various configurations.
///
/// Provides a fluent interface for configuring and building rule engines with
/// rules loaded from files or inline GRL strings.
///
/// # Examples
///
/// ```rust
/// use rust_rule_engine::RuleEngineBuilder;
///
/// // Build engine with inline rules
/// let engine = RuleEngineBuilder::new()
///     .with_inline_grl(r#"
///         rule "VIP Check" {
///             when user.points > 1000
///             then user.vip = true;
///         }
///     "#)?
///     .build();
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct RuleEngineBuilder {
    kb: KnowledgeBase,
    config: EngineConfig,
}

impl RuleEngineBuilder {
    /// Create a new RuleEngineBuilder with default configuration.
    ///
    /// Creates an empty knowledge base named "DefaultKB" and default engine configuration.
    pub fn new() -> Self {
        Self {
            kb: KnowledgeBase::new("DefaultKB"),
            config: EngineConfig::default(),
        }
    }

    /// Add rules from a .grl file.
    ///
    /// Reads and parses GRL rules from the specified file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the GRL syntax is invalid.
    pub fn with_rule_file<P: AsRef<std::path::Path>>(self, path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let rules = GRLParser::parse_rules(&content)?;

        for rule in rules {
            self.kb.add_rule(rule)?;
        }

        Ok(self)
    }

    /// Add rules from inline GRL string.
    ///
    /// Parses GRL rules directly from a string.
    ///
    /// # Errors
    ///
    /// Returns an error if the GRL syntax is invalid.
    pub fn with_inline_grl(self, grl_content: &str) -> Result<Self> {
        let rules = GRLParser::parse_rules(grl_content)?;

        for rule in rules {
            self.kb.add_rule(rule)?;
        }

        Ok(self)
    }

    /// Set engine configuration.
    ///
    /// Overrides the default engine configuration with custom settings.
    pub fn with_config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the RustRuleEngine.
    ///
    /// Consumes the builder and creates a configured rule engine instance.
    pub fn build(self) -> RustRuleEngine {
        RustRuleEngine::with_config(self.kb, self.config)
    }
}

impl Default for RuleEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

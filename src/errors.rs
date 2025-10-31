use thiserror::Error;

/// Error types that can occur in the rule engine
#[derive(Error, Debug)]
pub enum RuleEngineError {
    /// Parse error during rule parsing
    #[error("Parse error: {message}")]
    ParseError {
        /// Error message
        message: String,
    },

    /// Error during rule evaluation
    #[error("Evaluation error: {message}")]
    EvaluationError {
        /// Error message
        message: String,
    },

    /// Field not found in data context
    #[error("Field not found: {field}")]
    FieldNotFound {
        /// Field name that was not found
        field: String,
    },

    /// IO error for file operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Type mismatch error
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch {
        /// Expected type
        expected: String,
        /// Actual type
        actual: String,
    },

    /// Invalid operator error
    #[error("Invalid operator: {operator}")]
    InvalidOperator {
        /// Invalid operator
        operator: String,
    },

    /// Invalid logical operator error
    #[error("Invalid logical operator: {operator}")]
    InvalidLogicalOperator {
        /// Invalid logical operator
        operator: String,
    },

    /// Regex compilation or execution error
    #[error("Regex error: {message}")]
    RegexError {
        /// Error message
        message: String,
    },

    /// Action execution error
    #[error("Action execution error: {message}")]
    ActionError {
        /// Error message
        message: String,
    },

    /// General execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {message}")]
    SerializationError {
        /// Error message
        message: String,
    },

    /// Plugin system error
    #[error("Plugin error: {message}")]
    PluginError {
        /// Error message
        message: String,
    },
}

/// Convenient Result type alias for rule engine operations
pub type Result<T> = std::result::Result<T, RuleEngineError>;

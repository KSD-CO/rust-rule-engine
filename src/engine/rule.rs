#![allow(deprecated)]
#![allow(clippy::type_complexity)]

use crate::types::{ActionType, LogicalOperator, Operator, Value};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Expression in a condition - can be a field reference or function call
#[derive(Debug, Clone)]
pub enum ConditionExpression {
    /// Direct field reference (e.g., User.age)
    Field(String),
    /// Function call with arguments (e.g., aiSentiment(User.text))
    FunctionCall {
        /// Function name
        name: String,
        /// Function arguments (field names or literal values)
        args: Vec<String>,
    },
    /// Test CE - arbitrary expression that evaluates to boolean (CLIPS feature)
    /// Example: test(calculate_discount(Order.amount) > 10.0)
    Test {
        /// Function name for the test
        name: String,
        /// Function arguments
        args: Vec<String>,
    },
    /// Multi-field operation (CLIPS-inspired)
    /// Examples:
    /// - Order.items $?all_items (Collect)
    /// - Product.tags contains "value" (Contains)
    /// - Order.items count > 0 (Count)
    /// - Queue.tasks first (First)
    /// - Queue.tasks last (Last)
    /// - ShoppingCart.items empty (IsEmpty)
    MultiField {
        /// Field name (e.g., "Order.items")
        field: String,
        /// Multi-field operation type
        operation: String, // "collect", "contains", "count", "first", "last", "empty", "not_empty"
        /// Optional variable for binding (e.g., "$?all_items")
        variable: Option<String>,
    },
}

/// Represents a single condition in a rule
#[derive(Debug, Clone)]
pub struct Condition {
    /// The expression to evaluate (field or function call)
    pub expression: ConditionExpression,
    /// The comparison operator to use
    pub operator: Operator,
    /// The value to compare against
    pub value: Value,

    // Keep field for backward compatibility
    #[deprecated(note = "Use expression instead")]
    #[doc(hidden)]
    pub field: String,
}

impl Condition {
    /// Create a new condition with a field reference
    pub fn new(field: String, operator: Operator, value: Value) -> Self {
        Self {
            expression: ConditionExpression::Field(field.clone()),
            operator,
            value,
            field, // Keep for backward compatibility
        }
    }

    /// Create a new condition with a function call
    pub fn with_function(
        function_name: String,
        args: Vec<String>,
        operator: Operator,
        value: Value,
    ) -> Self {
        Self {
            expression: ConditionExpression::FunctionCall {
                name: function_name.clone(),
                args,
            },
            operator,
            value,
            field: function_name, // Use function name for backward compat
        }
    }

    /// Create a new Test CE condition
    /// The function must return a boolean value
    pub fn with_test(function_name: String, args: Vec<String>) -> Self {
        Self {
            expression: ConditionExpression::Test {
                name: function_name.clone(),
                args,
            },
            operator: Operator::Equal,   // Not used for Test CE
            value: Value::Boolean(true), // Not used for Test CE
            field: format!("test({})", function_name), // For backward compat
        }
    }

    /// Create multi-field collect condition
    /// Example: Order.items $?all_items
    pub fn with_multifield_collect(field: String, variable: String) -> Self {
        Self {
            expression: ConditionExpression::MultiField {
                field: field.clone(),
                operation: "collect".to_string(),
                variable: Some(variable),
            },
            operator: Operator::Equal,   // Not used for MultiField
            value: Value::Boolean(true), // Not used
            field,                       // For backward compat
        }
    }

    /// Create multi-field count condition
    /// Example: Order.items count > 0
    pub fn with_multifield_count(field: String, operator: Operator, value: Value) -> Self {
        Self {
            expression: ConditionExpression::MultiField {
                field: field.clone(),
                operation: "count".to_string(),
                variable: None,
            },
            operator,
            value,
            field, // For backward compat
        }
    }

    /// Create multi-field first condition
    /// Example: Queue.tasks first $first_task
    pub fn with_multifield_first(field: String, variable: Option<String>) -> Self {
        Self {
            expression: ConditionExpression::MultiField {
                field: field.clone(),
                operation: "first".to_string(),
                variable,
            },
            operator: Operator::Equal,   // Not used
            value: Value::Boolean(true), // Not used
            field,                       // For backward compat
        }
    }

    /// Create multi-field last condition
    /// Example: Queue.tasks last $last_task
    pub fn with_multifield_last(field: String, variable: Option<String>) -> Self {
        Self {
            expression: ConditionExpression::MultiField {
                field: field.clone(),
                operation: "last".to_string(),
                variable,
            },
            operator: Operator::Equal,   // Not used
            value: Value::Boolean(true), // Not used
            field,                       // For backward compat
        }
    }

    /// Create multi-field empty condition
    /// Example: ShoppingCart.items empty
    pub fn with_multifield_empty(field: String) -> Self {
        Self {
            expression: ConditionExpression::MultiField {
                field: field.clone(),
                operation: "empty".to_string(),
                variable: None,
            },
            operator: Operator::Equal,   // Not used
            value: Value::Boolean(true), // Not used
            field,                       // For backward compat
        }
    }

    /// Create multi-field not_empty condition
    /// Example: ShoppingCart.items not_empty
    pub fn with_multifield_not_empty(field: String) -> Self {
        Self {
            expression: ConditionExpression::MultiField {
                field: field.clone(),
                operation: "not_empty".to_string(),
                variable: None,
            },
            operator: Operator::Equal,   // Not used
            value: Value::Boolean(true), // Not used
            field,                       // For backward compat
        }
    }

    /// Evaluate this condition against the given facts
    pub fn evaluate(&self, facts: &HashMap<String, Value>) -> bool {
        match &self.expression {
            ConditionExpression::Field(field_name) => {
                // Get field value, or treat as Null if not found
                let field_value = get_nested_value(facts, field_name)
                    .cloned()
                    .unwrap_or(Value::Null);

                self.operator.evaluate(&field_value, &self.value)
            }
            ConditionExpression::FunctionCall { .. }
            | ConditionExpression::Test { .. }
            | ConditionExpression::MultiField { .. } => {
                // Function calls, Test CE, and MultiField need engine context
                // Will be handled by evaluate_with_engine
                false
            }
        }
    }

    /// Evaluate condition with access to engine's function registry
    /// This is needed for function call evaluation
    pub fn evaluate_with_engine(
        &self,
        facts: &HashMap<String, Value>,
        function_registry: &HashMap<
            String,
            std::sync::Arc<
                dyn Fn(Vec<Value>, &HashMap<String, Value>) -> crate::errors::Result<Value>
                    + Send
                    + Sync,
            >,
        >,
    ) -> bool {
        match &self.expression {
            ConditionExpression::Field(field_name) => {
                // Get field value, or treat as Null if not found
                let field_value = get_nested_value(facts, field_name)
                    .cloned()
                    .unwrap_or(Value::Null);

                self.operator.evaluate(&field_value, &self.value)
            }
            ConditionExpression::FunctionCall { name, args } => {
                // Call the function with arguments
                if let Some(function) = function_registry.get(name) {
                    // Resolve arguments from facts
                    let arg_values: Vec<Value> = args
                        .iter()
                        .map(|arg| {
                            get_nested_value(facts, arg)
                                .cloned()
                                .unwrap_or(Value::String(arg.clone()))
                        })
                        .collect();

                    // Call function
                    if let Ok(result) = function(arg_values, facts) {
                        // Compare function result with expected value
                        return self.operator.evaluate(&result, &self.value);
                    }
                }
                false
            }
            ConditionExpression::Test { name, args } => {
                // Test CE: Call the function and expect boolean result
                if let Some(function) = function_registry.get(name) {
                    // Resolve arguments from facts
                    let arg_values: Vec<Value> = args
                        .iter()
                        .map(|arg| {
                            get_nested_value(facts, arg)
                                .cloned()
                                .unwrap_or(Value::String(arg.clone()))
                        })
                        .collect();

                    // Call function
                    if let Ok(result) = function(arg_values, facts) {
                        // Test CE expects boolean result directly
                        match result {
                            Value::Boolean(b) => return b,
                            Value::Integer(i) => return i != 0,
                            Value::Number(f) => return f != 0.0,
                            Value::String(s) => return !s.is_empty(),
                            _ => return false,
                        }
                    }
                }
                false
            }
            ConditionExpression::MultiField {
                field,
                operation,
                variable: _,
            } => {
                // MultiField operations for array/collection handling
                if let Some(field_value) = get_nested_value(facts, field) {
                    match operation.as_str() {
                        "empty" => {
                            // Check if array is empty
                            matches!(field_value, Value::Array(arr) if arr.is_empty())
                        }
                        "not_empty" => {
                            // Check if array is not empty
                            matches!(field_value, Value::Array(arr) if !arr.is_empty())
                        }
                        "count" => {
                            // Get count and compare with value
                            if let Value::Array(arr) = field_value {
                                let count = Value::Integer(arr.len() as i64);
                                self.operator.evaluate(&count, &self.value)
                            } else {
                                false
                            }
                        }
                        "first" => {
                            // Get first element and compare with value
                            if let Value::Array(arr) = field_value {
                                if let Some(first) = arr.first() {
                                    self.operator.evaluate(first, &self.value)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        "last" => {
                            // Get last element and compare with value
                            if let Value::Array(arr) = field_value {
                                if let Some(last) = arr.last() {
                                    self.operator.evaluate(last, &self.value)
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        "contains" => {
                            // Check if array contains the specified value
                            if let Value::Array(arr) = field_value {
                                arr.iter()
                                    .any(|item| self.operator.evaluate(item, &self.value))
                            } else {
                                false
                            }
                        }
                        "collect" => {
                            // Collect operation: just check if array exists and has values
                            // Variable binding happens in RETE engine context
                            matches!(field_value, Value::Array(arr) if !arr.is_empty())
                        }
                        _ => {
                            // Unknown operation
                            false
                        }
                    }
                } else {
                    false
                }
            }
        }
    }
}

/// Group of conditions with logical operators
#[derive(Debug, Clone)]
pub enum ConditionGroup {
    /// A single condition
    Single(Condition),
    /// A compound condition with two sub-conditions and a logical operator
    Compound {
        /// The left side condition
        left: Box<ConditionGroup>,
        /// The logical operator (AND, OR)
        operator: LogicalOperator,
        /// The right side condition
        right: Box<ConditionGroup>,
    },
    /// A negated condition group
    Not(Box<ConditionGroup>),
    /// Pattern matching: check if at least one fact matches the condition
    Exists(Box<ConditionGroup>),
    /// Pattern matching: check if all facts of the target type match the condition
    Forall(Box<ConditionGroup>),
    /// Accumulate pattern: aggregate values from matching facts
    /// Example: accumulate(Order($amount: amount, status == "completed"), sum($amount))
    Accumulate {
        /// Variable to bind the result to (e.g., "$total")
        result_var: String,
        /// Source pattern to match facts (e.g., "Order")
        source_pattern: String,
        /// Field to extract from matching facts (e.g., "$amount: amount")
        extract_field: String,
        /// Conditions on the source pattern
        source_conditions: Vec<String>,
        /// Accumulate function to apply (sum, avg, count, min, max)
        function: String,
        /// Variable passed to function (e.g., "$amount" in "sum($amount)")
        function_arg: String,
    },
}

impl ConditionGroup {
    /// Create a single condition group
    pub fn single(condition: Condition) -> Self {
        ConditionGroup::Single(condition)
    }

    /// Create a compound condition using logical AND operator
    pub fn and(left: ConditionGroup, right: ConditionGroup) -> Self {
        ConditionGroup::Compound {
            left: Box::new(left),
            operator: LogicalOperator::And,
            right: Box::new(right),
        }
    }

    /// Create a compound condition using logical OR operator
    pub fn or(left: ConditionGroup, right: ConditionGroup) -> Self {
        ConditionGroup::Compound {
            left: Box::new(left),
            operator: LogicalOperator::Or,
            right: Box::new(right),
        }
    }

    /// Create a negated condition using logical NOT operator
    #[allow(clippy::should_implement_trait)]
    pub fn not(condition: ConditionGroup) -> Self {
        ConditionGroup::Not(Box::new(condition))
    }

    /// Create an exists condition - checks if at least one fact matches
    pub fn exists(condition: ConditionGroup) -> Self {
        ConditionGroup::Exists(Box::new(condition))
    }

    /// Create a forall condition - checks if all facts of target type match
    pub fn forall(condition: ConditionGroup) -> Self {
        ConditionGroup::Forall(Box::new(condition))
    }

    /// Create an accumulate condition - aggregates values from matching facts
    pub fn accumulate(
        result_var: String,
        source_pattern: String,
        extract_field: String,
        source_conditions: Vec<String>,
        function: String,
        function_arg: String,
    ) -> Self {
        ConditionGroup::Accumulate {
            result_var,
            source_pattern,
            extract_field,
            source_conditions,
            function,
            function_arg,
        }
    }

    /// Evaluate this condition group against facts
    pub fn evaluate(&self, facts: &HashMap<String, Value>) -> bool {
        match self {
            ConditionGroup::Single(condition) => condition.evaluate(facts),
            ConditionGroup::Compound {
                left,
                operator,
                right,
            } => {
                let left_result = left.evaluate(facts);
                let right_result = right.evaluate(facts);
                match operator {
                    LogicalOperator::And => left_result && right_result,
                    LogicalOperator::Or => left_result || right_result,
                    LogicalOperator::Not => !left_result, // For Not, we ignore right side
                }
            }
            ConditionGroup::Not(condition) => !condition.evaluate(facts),
            ConditionGroup::Exists(_)
            | ConditionGroup::Forall(_)
            | ConditionGroup::Accumulate { .. } => {
                // Pattern matching and accumulate conditions need Facts struct, not HashMap
                // For now, return false - these will be handled by the engine
                false
            }
        }
    }

    /// Evaluate this condition group against Facts (supports pattern matching)
    pub fn evaluate_with_facts(&self, facts: &crate::engine::facts::Facts) -> bool {
        use crate::engine::pattern_matcher::PatternMatcher;

        match self {
            ConditionGroup::Single(condition) => {
                let fact_map = facts.get_all_facts();
                condition.evaluate(&fact_map)
            }
            ConditionGroup::Compound {
                left,
                operator,
                right,
            } => {
                let left_result = left.evaluate_with_facts(facts);
                let right_result = right.evaluate_with_facts(facts);
                match operator {
                    LogicalOperator::And => left_result && right_result,
                    LogicalOperator::Or => left_result || right_result,
                    LogicalOperator::Not => !left_result,
                }
            }
            ConditionGroup::Not(condition) => !condition.evaluate_with_facts(facts),
            ConditionGroup::Exists(condition) => PatternMatcher::evaluate_exists(condition, facts),
            ConditionGroup::Forall(condition) => PatternMatcher::evaluate_forall(condition, facts),
            ConditionGroup::Accumulate { .. } => {
                // Accumulate conditions need special handling - they will be evaluated
                // during the engine execution phase, not here
                // For now, return true to allow the rule to continue evaluation
                true
            }
        }
    }
}

/// A rule with conditions and actions
#[derive(Debug, Clone)]
pub struct Rule {
    /// The unique name of the rule
    pub name: String,
    /// Optional description of what the rule does
    pub description: Option<String>,
    /// Priority of the rule (higher values execute first)
    pub salience: i32,
    /// Whether the rule is enabled for execution
    pub enabled: bool,
    /// Prevents the rule from activating itself in the same cycle
    pub no_loop: bool,
    /// Prevents the rule from firing again until agenda group changes
    pub lock_on_active: bool,
    /// Agenda group this rule belongs to (for workflow control)
    pub agenda_group: Option<String>,
    /// Activation group - only one rule in group can fire
    pub activation_group: Option<String>,
    /// Rule becomes effective from this date
    pub date_effective: Option<DateTime<Utc>>,
    /// Rule expires after this date
    pub date_expires: Option<DateTime<Utc>>,
    /// The conditions that must be met for the rule to fire
    pub conditions: ConditionGroup,
    /// The actions to execute when the rule fires
    pub actions: Vec<ActionType>,
}

impl Rule {
    /// Create a new rule with the given name, conditions, and actions
    pub fn new(name: String, conditions: ConditionGroup, actions: Vec<ActionType>) -> Self {
        Self {
            name,
            description: None,
            salience: 0,
            enabled: true,
            no_loop: false,
            lock_on_active: false,
            agenda_group: None,
            activation_group: None,
            date_effective: None,
            date_expires: None,
            conditions,
            actions,
        }
    }

    /// Add a description to the rule
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set the salience (priority) of the rule
    pub fn with_salience(mut self, salience: i32) -> Self {
        self.salience = salience;
        self
    }

    /// Set the priority of the rule (alias for salience)
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.salience = priority;
        self
    }

    /// Enable or disable no-loop behavior for this rule
    pub fn with_no_loop(mut self, no_loop: bool) -> Self {
        self.no_loop = no_loop;
        self
    }

    /// Enable or disable lock-on-active behavior for this rule
    pub fn with_lock_on_active(mut self, lock_on_active: bool) -> Self {
        self.lock_on_active = lock_on_active;
        self
    }

    /// Set the agenda group for this rule
    pub fn with_agenda_group(mut self, agenda_group: String) -> Self {
        self.agenda_group = Some(agenda_group);
        self
    }

    /// Set the activation group for this rule
    pub fn with_activation_group(mut self, activation_group: String) -> Self {
        self.activation_group = Some(activation_group);
        self
    }

    /// Set the effective date for this rule
    pub fn with_date_effective(mut self, date_effective: DateTime<Utc>) -> Self {
        self.date_effective = Some(date_effective);
        self
    }

    /// Set the expiration date for this rule
    pub fn with_date_expires(mut self, date_expires: DateTime<Utc>) -> Self {
        self.date_expires = Some(date_expires);
        self
    }

    /// Parse and set the effective date from ISO string
    pub fn with_date_effective_str(mut self, date_str: &str) -> Result<Self, chrono::ParseError> {
        let date = DateTime::parse_from_rfc3339(date_str)?.with_timezone(&Utc);
        self.date_effective = Some(date);
        Ok(self)
    }

    /// Parse and set the expiration date from ISO string
    pub fn with_date_expires_str(mut self, date_str: &str) -> Result<Self, chrono::ParseError> {
        let date = DateTime::parse_from_rfc3339(date_str)?.with_timezone(&Utc);
        self.date_expires = Some(date);
        Ok(self)
    }

    /// Check if this rule is active at the given timestamp
    pub fn is_active_at(&self, timestamp: DateTime<Utc>) -> bool {
        // Check if rule is effective
        if let Some(effective) = self.date_effective {
            if timestamp < effective {
                return false;
            }
        }

        // Check if rule has expired
        if let Some(expires) = self.date_expires {
            if timestamp >= expires {
                return false;
            }
        }

        true
    }

    /// Check if this rule is currently active (using current time)
    pub fn is_active(&self) -> bool {
        self.is_active_at(Utc::now())
    }

    /// Check if this rule matches the given facts
    pub fn matches(&self, facts: &HashMap<String, Value>) -> bool {
        self.enabled && self.conditions.evaluate(facts)
    }
}

/// Result of rule execution
#[derive(Debug, Clone)]
pub struct RuleExecutionResult {
    /// The name of the rule that was executed
    pub rule_name: String,
    /// Whether the rule's conditions matched and it fired
    pub matched: bool,
    /// List of actions that were executed
    pub actions_executed: Vec<String>,
    /// Time taken to execute the rule in milliseconds
    pub execution_time_ms: f64,
}

impl RuleExecutionResult {
    /// Create a new rule execution result
    pub fn new(rule_name: String) -> Self {
        Self {
            rule_name,
            matched: false,
            actions_executed: Vec::new(),
            execution_time_ms: 0.0,
        }
    }

    /// Mark the rule as matched
    pub fn matched(mut self) -> Self {
        self.matched = true;
        self
    }

    /// Set the actions that were executed
    pub fn with_actions(mut self, actions: Vec<String>) -> Self {
        self.actions_executed = actions;
        self
    }

    /// Set the execution time in milliseconds
    pub fn with_execution_time(mut self, time_ms: f64) -> Self {
        self.execution_time_ms = time_ms;
        self
    }
}

/// Helper function to get nested values from a HashMap
fn get_nested_value<'a>(data: &'a HashMap<String, Value>, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = data.get(parts[0])?;

    for part in parts.iter().skip(1) {
        match current {
            Value::Object(obj) => {
                current = obj.get(*part)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

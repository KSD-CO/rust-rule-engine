use crate::types::{ActionType, LogicalOperator, Operator, Value};
use std::collections::HashMap;

/// Represents a single condition in a rule
#[derive(Debug, Clone)]
pub struct Condition {
    /// The field name to evaluate
    pub field: String,
    /// The comparison operator to use
    pub operator: Operator,
    /// The value to compare against
    pub value: Value,
}

impl Condition {
    /// Create a new condition
    pub fn new(field: String, operator: Operator, value: Value) -> Self {
        Self {
            field,
            operator,
            value,
        }
    }

    /// Evaluate this condition against the given facts
    pub fn evaluate(&self, facts: &HashMap<String, Value>) -> bool {
        if let Some(field_value) = get_nested_value(facts, &self.field) {
            // Use the evaluate method from Operator
            self.operator.evaluate(field_value, &self.value)
        } else {
            false
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

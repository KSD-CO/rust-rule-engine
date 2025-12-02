use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a value that can be used in rule conditions and actions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    /// String value
    String(String),
    /// Floating point number
    Number(f64),
    /// Integer value
    Integer(i64),
    /// Boolean value
    Boolean(bool),
    /// Array of values
    Array(Vec<Value>),
    /// Object with key-value pairs
    Object(HashMap<String, Value>),
    /// Null value
    Null,
    /// Expression to be evaluated at runtime (e.g., "Order.quantity * Order.price")
    Expression(String),
}

impl Value {
    /// Convert Value to string representation
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(_) => "[Array]".to_string(),
            Value::Object(_) => "[Object]".to_string(),
            Value::Null => "null".to_string(),
            Value::Expression(expr) => format!("[Expr: {}]", expr),
        }
    }

    /// Convert Value to number if possible
    pub fn to_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Integer(i) => Some(*i as f64),
            Value::String(s) => s.parse::<f64>().ok(),
            _ => None,
        }
    }

    /// Get string value if this is a string
    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Get integer value if this is an integer
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get boolean value if this is a boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Get number value if this is a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Convert Value to boolean
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Number(n) => *n != 0.0,
            Value::Integer(i) => *i != 0,
            Value::Array(arr) => !arr.is_empty(),
            Value::Object(obj) => !obj.is_empty(),
            Value::Null => false,
            Value::Expression(_) => false, // Expression needs to be evaluated first
        }
    }

    /// Call a method on this value with given arguments
    pub fn call_method(&mut self, method: &str, args: Vec<Value>) -> Result<Value, String> {
        match self {
            Value::Object(ref mut obj) => match method {
                "setSpeed" => {
                    if let Some(Value::Number(speed)) = args.first() {
                        obj.insert("Speed".to_string(), Value::Number(*speed));
                        Ok(Value::Null)
                    } else if let Some(Value::Integer(speed)) = args.first() {
                        obj.insert("Speed".to_string(), Value::Integer(*speed));
                        Ok(Value::Null)
                    } else {
                        Err("setSpeed requires a number argument".to_string())
                    }
                }
                "setTotalDistance" => {
                    if let Some(Value::Number(distance)) = args.first() {
                        obj.insert("TotalDistance".to_string(), Value::Number(*distance));
                        Ok(Value::Null)
                    } else if let Some(Value::Integer(distance)) = args.first() {
                        obj.insert("TotalDistance".to_string(), Value::Integer(*distance));
                        Ok(Value::Null)
                    } else {
                        Err("setTotalDistance requires a number argument".to_string())
                    }
                }
                "getTotalDistance" => Ok(obj
                    .get("TotalDistance")
                    .cloned()
                    .unwrap_or(Value::Number(0.0))),
                "getSpeed" => Ok(obj.get("Speed").cloned().unwrap_or(Value::Number(0.0))),
                _ => Err(format!("Method {} not found", method)),
            },
            _ => Err("Cannot call method on non-object value".to_string()),
        }
    }

    /// Get a property from this object
    pub fn get_property(&self, property: &str) -> Option<Value> {
        match self {
            Value::Object(obj) => obj.get(property).cloned(),
            _ => None,
        }
    }

    /// Set a property on this object
    pub fn set_property(&mut self, property: &str, value: Value) -> Result<(), String> {
        match self {
            Value::Object(ref mut obj) => {
                obj.insert(property.to_string(), value);
                Ok(())
            }
            _ => Err("Cannot set property on non-object value".to_string()),
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<serde_json::Value> for Value {
    fn from(json_value: serde_json::Value) -> Self {
        match json_value {
            serde_json::Value::String(s) => Value::String(s),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Value::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    Value::Number(f)
                } else {
                    Value::Null
                }
            }
            serde_json::Value::Bool(b) => Value::Boolean(b),
            serde_json::Value::Array(arr) => {
                Value::Array(arr.into_iter().map(Value::from).collect())
            }
            serde_json::Value::Object(obj) => {
                let mut map = HashMap::new();
                for (k, v) in obj {
                    map.insert(k, Value::from(v));
                }
                Value::Object(map)
            }
            serde_json::Value::Null => Value::Null,
        }
    }
}

/// Comparison operators for rule conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operator {
    /// Equality comparison
    Equal,
    /// Inequality comparison
    NotEqual,
    /// Greater than comparison
    GreaterThan,
    /// Greater than or equal comparison
    GreaterThanOrEqual,
    /// Less than comparison
    LessThan,
    /// Less than or equal comparison
    LessThanOrEqual,
    /// String contains check
    Contains,
    /// String does not contain check
    NotContains,
    /// String starts with check
    StartsWith,
    /// String ends with check
    EndsWith,
    /// Regex pattern match
    Matches,
}

impl Operator {
    /// Parse operator from string representation
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "==" | "eq" => Some(Operator::Equal),
            "!=" | "ne" => Some(Operator::NotEqual),
            ">" | "gt" => Some(Operator::GreaterThan),
            ">=" | "gte" => Some(Operator::GreaterThanOrEqual),
            "<" | "lt" => Some(Operator::LessThan),
            "<=" | "lte" => Some(Operator::LessThanOrEqual),
            "contains" => Some(Operator::Contains),
            "not_contains" => Some(Operator::NotContains),
            "starts_with" => Some(Operator::StartsWith),
            "ends_with" => Some(Operator::EndsWith),
            "matches" => Some(Operator::Matches),
            _ => None,
        }
    }

    /// Evaluate the operator against two values
    pub fn evaluate(&self, left: &Value, right: &Value) -> bool {
        match self {
            Operator::Equal => {
                // Special handling for null comparison
                // "null" string should be treated as Value::Null
                if matches!(left, Value::Null) || matches!(right, Value::Null) {
                    // Convert "null" string to Value::Null for comparison
                    let left_is_null = matches!(left, Value::Null) 
                        || (matches!(left, Value::String(s) if s == "null"));
                    let right_is_null = matches!(right, Value::Null)
                        || (matches!(right, Value::String(s) if s == "null"));
                    
                    left_is_null == right_is_null
                } else {
                    left == right
                }
            }
            Operator::NotEqual => {
                // Special handling for null comparison
                if matches!(left, Value::Null) || matches!(right, Value::Null) {
                    let left_is_null = matches!(left, Value::Null)
                        || (matches!(left, Value::String(s) if s == "null"));
                    let right_is_null = matches!(right, Value::Null)
                        || (matches!(right, Value::String(s) if s == "null"));
                    
                    left_is_null != right_is_null
                } else {
                    left != right
                }
            }
            Operator::GreaterThan => {
                if let (Some(l), Some(r)) = (left.to_number(), right.to_number()) {
                    l > r
                } else {
                    false
                }
            }
            Operator::GreaterThanOrEqual => {
                if let (Some(l), Some(r)) = (left.to_number(), right.to_number()) {
                    l >= r
                } else {
                    false
                }
            }
            Operator::LessThan => {
                if let (Some(l), Some(r)) = (left.to_number(), right.to_number()) {
                    l < r
                } else {
                    false
                }
            }
            Operator::LessThanOrEqual => {
                if let (Some(l), Some(r)) = (left.to_number(), right.to_number()) {
                    l <= r
                } else {
                    false
                }
            }
            Operator::Contains => {
                if let (Some(l), Some(r)) = (left.as_string(), right.as_string()) {
                    l.contains(&r)
                } else {
                    false
                }
            }
            Operator::NotContains => {
                if let (Some(l), Some(r)) = (left.as_string(), right.as_string()) {
                    !l.contains(&r)
                } else {
                    false
                }
            }
            Operator::StartsWith => {
                if let (Some(l), Some(r)) = (left.as_string(), right.as_string()) {
                    l.starts_with(&r)
                } else {
                    false
                }
            }
            Operator::EndsWith => {
                if let (Some(l), Some(r)) = (left.as_string(), right.as_string()) {
                    l.ends_with(&r)
                } else {
                    false
                }
            }
            Operator::Matches => {
                // Simple regex match implementation
                if let (Some(l), Some(r)) = (left.as_string(), right.as_string()) {
                    // For now, just use contains as a simple match
                    l.contains(&r)
                } else {
                    false
                }
            }
        }
    }
}

/// Logical operators for combining conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicalOperator {
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Logical NOT
    Not,
}

impl LogicalOperator {
    /// Parse logical operator from string representation
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "and" | "&&" => Some(LogicalOperator::And),
            "or" | "||" => Some(LogicalOperator::Or),
            "not" | "!" => Some(LogicalOperator::Not),
            _ => None,
        }
    }
}

/// Represents the data context for rule evaluation
pub type Context = HashMap<String, Value>;

/// Action types that can be performed when a rule matches
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    /// Set a field to a specific value
    Set {
        /// Field name to set
        field: String,
        /// Value to set
        value: Value,
    },
    /// Log a message
    Log {
        /// Message to log
        message: String,
    },
    /// Call a method on an object
    MethodCall {
        /// Object name
        object: String,
        /// Method name
        method: String,
        /// Method arguments
        args: Vec<Value>,
    },
    /// Retract (delete) a fact from working memory
    Retract {
        /// Object/fact to retract
        object: String,
    },
    /// Custom action
    Custom {
        /// Action type identifier
        action_type: String,
        /// Action parameters
        params: HashMap<String, Value>,
    },
    /// Activate a specific agenda group for workflow progression
    ActivateAgendaGroup {
        /// Agenda group name to activate
        group: String,
    },
    /// Schedule a rule to execute after a delay
    ScheduleRule {
        /// Rule name to schedule
        rule_name: String,
        /// Delay in milliseconds
        delay_ms: u64,
    },
    /// Complete a workflow and trigger cleanup
    CompleteWorkflow {
        /// Workflow name to complete
        workflow_name: String,
    },
    /// Set workflow-specific data
    SetWorkflowData {
        /// Data key
        key: String,
        /// Data value
        value: Value,
    },
}

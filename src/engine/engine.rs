use crate::engine::{facts::Facts, knowledge_base::KnowledgeBase};
use crate::errors::{Result, RuleEngineError};
use crate::types::{ActionType, Value};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Type for custom function implementations
pub type CustomFunction = Box<dyn Fn(&[Value], &Facts) -> Result<Value> + Send + Sync>;

/// Configuration options for the rule engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum number of execution cycles
    pub max_cycles: usize,
    /// Execution timeout
    pub timeout: Option<Duration>,
    /// Enable performance statistics collection
    pub enable_stats: bool,
    /// Enable debug mode with verbose logging
    pub debug_mode: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_cycles: 100,
            timeout: Some(Duration::from_secs(30)),
            enable_stats: true,
            debug_mode: false,
        }
    }
}

/// Result of rule engine execution
#[derive(Debug, Clone)]
pub struct GruleExecutionResult {
    /// Number of execution cycles
    pub cycle_count: usize,
    /// Number of rules evaluated
    pub rules_evaluated: usize,
    /// Number of rules that fired
    pub rules_fired: usize,
    /// Total execution time
    pub execution_time: Duration,
}

/// Rust Rule Engine - High-performance rule execution engine
pub struct RustRuleEngine {
    knowledge_base: KnowledgeBase,
    config: EngineConfig,
    custom_functions: HashMap<String, CustomFunction>,
}

impl RustRuleEngine {
    /// Create a new RustRuleEngine with default configuration
    pub fn new(knowledge_base: KnowledgeBase) -> Self {
        Self {
            knowledge_base,
            config: EngineConfig::default(),
            custom_functions: HashMap::new(),
        }
    }

    /// Create a new RustRuleEngine with custom configuration
    pub fn with_config(knowledge_base: KnowledgeBase, config: EngineConfig) -> Self {
        Self {
            knowledge_base,
            config,
            custom_functions: HashMap::new(),
        }
    }

    /// Register a custom function
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[Value], &Facts) -> Result<Value> + Send + Sync + 'static,
    {
        self.custom_functions
            .insert(name.to_string(), Box::new(func));
    }

    /// Check if a custom function is registered
    pub fn has_function(&self, name: &str) -> bool {
        self.custom_functions.contains_key(name)
    }

    /// Get reference to the knowledge base
    pub fn knowledge_base(&self) -> &KnowledgeBase {
        &self.knowledge_base
    }

    /// Get mutable reference to the knowledge base
    pub fn knowledge_base_mut(&mut self) -> &mut KnowledgeBase {
        &mut self.knowledge_base
    }

    /// Execute all rules in the knowledge base against the given facts
    pub fn execute(&self, facts: &Facts) -> Result<GruleExecutionResult> {
        let start_time = Instant::now();
        let mut cycle_count = 0;
        let mut rules_evaluated = 0;
        let mut rules_fired = 0;

        if self.config.debug_mode {
            println!(
                "🚀 Starting rule execution with {} rules",
                self.knowledge_base.get_rules().len()
            );
        }

        for cycle in 0..self.config.max_cycles {
            cycle_count = cycle + 1;
            let mut any_rule_fired = false;

            // Check for timeout
            if let Some(timeout) = self.config.timeout {
                if start_time.elapsed() > timeout {
                    return Err(RuleEngineError::EvaluationError {
                        message: "Execution timeout exceeded".to_string(),
                    });
                }
            }

            // Get rules sorted by salience (highest first)
            let mut rules = self.knowledge_base.get_rules().clone();
            rules.sort_by(|a, b| b.salience.cmp(&a.salience));

            for rule in &rules {
                if !rule.enabled {
                    continue;
                }

                rules_evaluated += 1;

                if self.config.debug_mode {
                    println!("📝 Evaluating rule: {}", rule.name);
                }

                // Evaluate rule conditions
                if self.evaluate_conditions(&rule.conditions, facts)? {
                    if self.config.debug_mode {
                        println!(
                            "🔥 Rule '{}' fired (salience: {})",
                            rule.name, rule.salience
                        );
                    }

                    // Execute actions
                    for action in &rule.actions {
                        self.execute_action(action, facts)?;
                    }

                    rules_fired += 1;
                    any_rule_fired = true;
                }
            }

            // If no rules fired in this cycle, we're done
            if !any_rule_fired {
                break;
            }
        }

        let execution_time = start_time.elapsed();

        Ok(GruleExecutionResult {
            cycle_count,
            rules_evaluated,
            rules_fired,
            execution_time,
        })
    }

    /// Evaluate conditions against facts
    fn evaluate_conditions(
        &self,
        conditions: &crate::engine::rule::ConditionGroup,
        facts: &Facts,
    ) -> Result<bool> {
        use crate::engine::rule::ConditionGroup;

        match conditions {
            ConditionGroup::Single(condition) => self.evaluate_single_condition(condition, facts),
            ConditionGroup::Compound {
                left,
                operator,
                right,
            } => {
                let left_result = self.evaluate_conditions(left, facts)?;
                let right_result = self.evaluate_conditions(right, facts)?;

                match operator {
                    crate::types::LogicalOperator::And => Ok(left_result && right_result),
                    crate::types::LogicalOperator::Or => Ok(left_result || right_result),
                    crate::types::LogicalOperator::Not => Err(RuleEngineError::EvaluationError {
                        message: "NOT operator should not appear in compound conditions"
                            .to_string(),
                    }),
                }
            }
            ConditionGroup::Not(condition) => {
                let result = self.evaluate_conditions(condition, facts)?;
                Ok(!result)
            }
        }
    }

    /// Evaluate a single condition
    fn evaluate_single_condition(
        &self,
        condition: &crate::engine::rule::Condition,
        facts: &Facts,
    ) -> Result<bool> {
        // Get the field value from facts
        let field_value = facts.get_nested(&condition.field);

        if field_value.is_none() {
            return Ok(false); // Field not found, condition fails
        }

        let field_value = field_value.unwrap();

        // Compare using the operator
        Ok(condition.operator.evaluate(&field_value, &condition.value))
    }

    /// Execute an action
    fn execute_action(&self, action: &ActionType, facts: &Facts) -> Result<()> {
        match action {
            ActionType::Set { field, value } => {
                facts.set_nested(field, value.clone())?;
                if self.config.debug_mode {
                    println!("  ✅ Set {field} = {value:?}");
                }
            }
            ActionType::Log { message } => {
                println!("📋 LOG: {}", message);
            }
            ActionType::Call { function, args } => {
                let result = self.execute_function_call(function, args, facts)?;
                if self.config.debug_mode {
                    println!("  📞 Called {function}({args:?}) -> {result}");
                }
            }
            ActionType::MethodCall {
                object,
                method,
                args,
            } => {
                let result = self.execute_method_call(object, method, args, facts)?;
                if self.config.debug_mode {
                    println!("  🔧 Called {object}.{method}({args:?}) -> {result}");
                }
            }
            ActionType::Update { object } => {
                if self.config.debug_mode {
                    println!("  🔄 Updated {object}");
                }
                // Update action is mainly for working memory management
                // In this implementation, it's mostly a no-op since we update in place
            }
            ActionType::Custom {
                action_type,
                params,
            } => {
                if self.config.debug_mode {
                    println!("  🎯 Custom action: {action_type} with params: {params:?}");
                }
            }
        }
        Ok(())
    }

    /// Execute function call
    fn execute_function_call(
        &self,
        function: &str,
        args: &[Value],
        facts: &Facts,
    ) -> Result<String> {
        let function_lower = function.to_lowercase();

        // Handle built-in utility functions
        match function_lower.as_str() {
            "log" | "print" | "println" => self.handle_log_function(args),
            "update" | "refresh" => self.handle_update_function(args),
            "now" | "timestamp" => self.handle_timestamp_function(),
            "random" => self.handle_random_function(args),
            "format" | "sprintf" => self.handle_format_function(args),
            "length" | "size" | "count" => self.handle_length_function(args),
            "sum" | "add" => self.handle_sum_function(args),
            "max" | "maximum" => self.handle_max_function(args),
            "min" | "minimum" => self.handle_min_function(args),
            "avg" | "average" => self.handle_average_function(args),
            "round" => self.handle_round_function(args),
            "floor" => self.handle_floor_function(args),
            "ceil" | "ceiling" => self.handle_ceil_function(args),
            "abs" | "absolute" => self.handle_abs_function(args),
            "contains" | "includes" => self.handle_contains_function(args),
            "startswith" | "begins_with" => self.handle_starts_with_function(args),
            "endswith" | "ends_with" => self.handle_ends_with_function(args),
            "lowercase" | "tolower" => self.handle_lowercase_function(args),
            "uppercase" | "toupper" => self.handle_uppercase_function(args),
            "trim" | "strip" => self.handle_trim_function(args),
            "split" => self.handle_split_function(args),
            "join" => self.handle_join_function(args),
            _ => {
                // Try to call custom user-defined function
                self.handle_custom_function(function, args, facts)
            }
        }
    }

    /// Handle logging functions (log, print, println)
    fn handle_log_function(&self, args: &[Value]) -> Result<String> {
        let message = if args.is_empty() {
            "".to_string()
        } else if args.len() == 1 {
            args[0].to_string()
        } else {
            args.iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        };

        println!("📋 {}", message);
        Ok(message)
    }

    /// Handle update/refresh functions
    fn handle_update_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            Ok(format!("Updated: {}", arg.to_string()))
        } else {
            Ok("Updated".to_string())
        }
    }

    /// Handle timestamp function
    fn handle_timestamp_function(&self) -> Result<String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RuleEngineError::EvaluationError {
                message: format!("Failed to get timestamp: {}", e),
            })?
            .as_secs();
        Ok(timestamp.to_string())
    }

    /// Handle random function
    fn handle_random_function(&self, args: &[Value]) -> Result<String> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Simple pseudo-random based on current time (for deterministic behavior in tests)
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        let random_value = hasher.finish();

        if args.is_empty() {
            Ok((random_value % 100).to_string()) // 0-99
        } else if let Some(Value::Number(max)) = args.first() {
            let max_val = *max as u64;
            Ok((random_value % max_val).to_string())
        } else {
            Ok(random_value.to_string())
        }
    }

    /// Handle format function (simple sprintf-like)
    fn handle_format_function(&self, args: &[Value]) -> Result<String> {
        if args.is_empty() {
            return Ok("".to_string());
        }

        let template = args[0].to_string();
        let values: Vec<String> = args[1..].iter().map(|v| v.to_string()).collect();

        // Simple placeholder replacement: {0}, {1}, etc.
        let mut result = template;
        for (i, value) in values.iter().enumerate() {
            result = result.replace(&format!("{{{}}}", i), value);
        }

        Ok(result)
    }

    /// Handle length/size functions
    fn handle_length_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            match arg {
                Value::String(s) => Ok(s.len().to_string()),
                Value::Array(arr) => Ok(arr.len().to_string()),
                Value::Object(obj) => Ok(obj.len().to_string()),
                _ => Ok("1".to_string()), // Single value has length 1
            }
        } else {
            Ok("0".to_string())
        }
    }

    /// Handle sum function
    fn handle_sum_function(&self, args: &[Value]) -> Result<String> {
        let sum = args.iter().fold(0.0, |acc, val| match val {
            Value::Number(n) => acc + n,
            Value::Integer(i) => acc + (*i as f64),
            _ => acc,
        });
        Ok(sum.to_string())
    }

    /// Handle max function
    fn handle_max_function(&self, args: &[Value]) -> Result<String> {
        let max = args.iter().fold(f64::NEG_INFINITY, |acc, val| match val {
            Value::Number(n) => acc.max(*n),
            Value::Integer(i) => acc.max(*i as f64),
            _ => acc,
        });
        Ok(max.to_string())
    }

    /// Handle min function
    fn handle_min_function(&self, args: &[Value]) -> Result<String> {
        let min = args.iter().fold(f64::INFINITY, |acc, val| match val {
            Value::Number(n) => acc.min(*n),
            Value::Integer(i) => acc.min(*i as f64),
            _ => acc,
        });
        Ok(min.to_string())
    }

    /// Handle average function
    fn handle_average_function(&self, args: &[Value]) -> Result<String> {
        if args.is_empty() {
            return Ok("0".to_string());
        }

        let (sum, count) = args.iter().fold((0.0, 0), |(sum, count), val| match val {
            Value::Number(n) => (sum + n, count + 1),
            Value::Integer(i) => (sum + (*i as f64), count + 1),
            _ => (sum, count),
        });

        if count > 0 {
            Ok((sum / count as f64).to_string())
        } else {
            Ok("0".to_string())
        }
    }

    /// Handle mathematical functions
    fn handle_round_function(&self, args: &[Value]) -> Result<String> {
        if let Some(Value::Number(n)) = args.first() {
            Ok(n.round().to_string())
        } else if let Some(Value::Integer(i)) = args.first() {
            Ok(i.to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "round() requires a numeric argument".to_string(),
            })
        }
    }

    fn handle_floor_function(&self, args: &[Value]) -> Result<String> {
        if let Some(Value::Number(n)) = args.first() {
            Ok(n.floor().to_string())
        } else if let Some(Value::Integer(i)) = args.first() {
            Ok(i.to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "floor() requires a numeric argument".to_string(),
            })
        }
    }

    fn handle_ceil_function(&self, args: &[Value]) -> Result<String> {
        if let Some(Value::Number(n)) = args.first() {
            Ok(n.ceil().to_string())
        } else if let Some(Value::Integer(i)) = args.first() {
            Ok(i.to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "ceil() requires a numeric argument".to_string(),
            })
        }
    }

    fn handle_abs_function(&self, args: &[Value]) -> Result<String> {
        if let Some(Value::Number(n)) = args.first() {
            Ok(n.abs().to_string())
        } else if let Some(Value::Integer(i)) = args.first() {
            Ok(i.abs().to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "abs() requires a numeric argument".to_string(),
            })
        }
    }

    /// Handle string functions
    fn handle_contains_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let haystack = args[0].to_string();
            let needle = args[1].to_string();
            Ok(haystack.contains(&needle).to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "contains() requires 2 arguments".to_string(),
            })
        }
    }

    fn handle_starts_with_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let text = args[0].to_string();
            let prefix = args[1].to_string();
            Ok(text.starts_with(&prefix).to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "startswith() requires 2 arguments".to_string(),
            })
        }
    }

    fn handle_ends_with_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let text = args[0].to_string();
            let suffix = args[1].to_string();
            Ok(text.ends_with(&suffix).to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "endswith() requires 2 arguments".to_string(),
            })
        }
    }

    fn handle_lowercase_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            Ok(arg.to_string().to_lowercase())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "lowercase() requires 1 argument".to_string(),
            })
        }
    }

    fn handle_uppercase_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            Ok(arg.to_string().to_uppercase())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "uppercase() requires 1 argument".to_string(),
            })
        }
    }

    fn handle_trim_function(&self, args: &[Value]) -> Result<String> {
        if let Some(arg) = args.first() {
            Ok(arg.to_string().trim().to_string())
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "trim() requires 1 argument".to_string(),
            })
        }
    }

    fn handle_split_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let text = args[0].to_string();
            let delimiter = args[1].to_string();
            let parts: Vec<String> = text.split(&delimiter).map(|s| s.to_string()).collect();
            Ok(format!("{:?}", parts)) // Return as debug string for now
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "split() requires 2 arguments".to_string(),
            })
        }
    }

    fn handle_join_function(&self, args: &[Value]) -> Result<String> {
        if args.len() >= 2 {
            let delimiter = args[0].to_string();
            let parts: Vec<String> = args[1..].iter().map(|v| v.to_string()).collect();
            Ok(parts.join(&delimiter))
        } else {
            Err(RuleEngineError::EvaluationError {
                message: "join() requires at least 2 arguments".to_string(),
            })
        }
    }

    /// Handle custom user-defined functions
    fn handle_custom_function(
        &self,
        function: &str,
        args: &[Value],
        facts: &Facts,
    ) -> Result<String> {
        // Check if we have a registered custom function
        if let Some(custom_func) = self.custom_functions.get(function) {
            if self.config.debug_mode {
                println!("🎯 Calling registered function: {}({:?})", function, args);
            }

            match custom_func(args, facts) {
                Ok(result) => Ok(result.to_string()),
                Err(e) => Err(e),
            }
        } else {
            // Function not found - return error or placeholder
            if self.config.debug_mode {
                println!("⚠️ Custom function '{}' not registered", function);
            }

            Err(RuleEngineError::EvaluationError {
                message: format!("Function '{}' is not registered. Use engine.register_function() to add custom functions.", function),
            })
        }
    }

    /// Execute method call on object
    fn execute_method_call(
        &self,
        object_name: &str,
        method: &str,
        args: &[Value],
        facts: &Facts,
    ) -> Result<String> {
        // Get the object from facts
        let Some(object_value) = facts.get(object_name) else {
            return Err(RuleEngineError::EvaluationError {
                message: format!("Object '{}' not found in facts", object_name),
            });
        };

        let method_lower = method.to_lowercase();

        // Handle setter methods (set + property name)
        if method_lower.starts_with("set") && args.len() == 1 {
            return self.handle_setter_method(object_name, method, &args[0], object_value, facts);
        }

        // Handle getter methods (get + property name)
        if method_lower.starts_with("get") && args.is_empty() {
            return self.handle_getter_method(object_name, method, &object_value);
        }

        // Handle built-in methods
        match method_lower.as_str() {
            "tostring" => Ok(object_value.to_string()),
            "update" => {
                facts.add_value(object_name, object_value)?;
                Ok(format!("Updated {}", object_name))
            }
            "reset" => self.handle_reset_method(object_name, object_value, facts),
            _ => self.handle_property_access_or_fallback(
                object_name,
                method,
                args.len(),
                &object_value,
            ),
        }
    }

    /// Handle setter method calls (setXxx)
    fn handle_setter_method(
        &self,
        object_name: &str,
        method: &str,
        new_value: &Value,
        mut object_value: Value,
        facts: &Facts,
    ) -> Result<String> {
        let property_name = Self::extract_property_name_from_setter(method);

        match object_value {
            Value::Object(ref mut obj) => {
                obj.insert(property_name.clone(), new_value.clone());
                facts.add_value(object_name, object_value)?;
                Ok(format!(
                    "Set {} to {}",
                    property_name,
                    new_value.to_string()
                ))
            }
            _ => Err(RuleEngineError::EvaluationError {
                message: format!("Cannot call setter on non-object type: {}", object_name),
            }),
        }
    }

    /// Handle getter method calls (getXxx)
    fn handle_getter_method(
        &self,
        object_name: &str,
        method: &str,
        object_value: &Value,
    ) -> Result<String> {
        let property_name = Self::extract_property_name_from_getter(method);

        match object_value {
            Value::Object(obj) => {
                if let Some(value) = obj.get(&property_name) {
                    Ok(value.to_string())
                } else {
                    Err(RuleEngineError::EvaluationError {
                        message: format!(
                            "Property '{}' not found on object '{}'",
                            property_name, object_name
                        ),
                    })
                }
            }
            _ => Err(RuleEngineError::EvaluationError {
                message: format!("Cannot call getter on non-object type: {}", object_name),
            }),
        }
    }

    /// Handle reset method call
    fn handle_reset_method(
        &self,
        object_name: &str,
        mut object_value: Value,
        facts: &Facts,
    ) -> Result<String> {
        match object_value {
            Value::Object(ref mut obj) => {
                obj.clear();
                facts.add_value(object_name, object_value)?;
                Ok(format!("Reset {}", object_name))
            }
            _ => Err(RuleEngineError::EvaluationError {
                message: format!("Cannot reset non-object type: {}", object_name),
            }),
        }
    }

    /// Handle property access or fallback to generic method call
    fn handle_property_access_or_fallback(
        &self,
        object_name: &str,
        method: &str,
        arg_count: usize,
        object_value: &Value,
    ) -> Result<String> {
        if let Value::Object(obj) = object_value {
            // Try exact property name match
            if let Some(value) = obj.get(method) {
                return Ok(value.to_string());
            }

            // Try capitalized property name
            let capitalized_method = Self::capitalize_first_letter(method);
            if let Some(value) = obj.get(&capitalized_method) {
                return Ok(value.to_string());
            }
        }

        // Fallback to generic response
        Ok(format!(
            "Called {}.{} with {} args",
            object_name, method, arg_count
        ))
    }

    /// Extract property name from setter method (setXxx -> Xxx)
    fn extract_property_name_from_setter(method: &str) -> String {
        let property_name = &method[3..]; // Remove "set" prefix
        Self::capitalize_first_letter(property_name)
    }

    /// Extract property name from getter method (getXxx -> Xxx)
    fn extract_property_name_from_getter(method: &str) -> String {
        let property_name = &method[3..]; // Remove "get" prefix
        Self::capitalize_first_letter(property_name)
    }

    /// Helper function to capitalize first letter of a string
    fn capitalize_first_letter(s: &str) -> String {
        if s.is_empty() {
            return String::new();
        }
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
}

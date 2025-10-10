use crate::engine::rule::{Condition, ConditionGroup, Rule};
use crate::errors::{Result, RuleEngineError};
use crate::types::{ActionType, Operator, Value};
use regex::Regex;
use std::collections::HashMap;

/// GRL (Grule Rule Language) Parser
/// Parses Grule-like syntax into Rule objects
pub struct GRLParser;

impl GRLParser {
    /// Parse a single rule from GRL syntax
    ///
    /// Example GRL syntax:
    /// ```grl
    /// rule CheckAge "Age verification rule" salience 10 {
    ///     when
    ///         User.Age >= 18 && User.Country == "US"
    ///     then
    ///         User.IsAdult = true;
    ///         Retract("User");
    /// }
    /// ```
    pub fn parse_rule(grl_text: &str) -> Result<Rule> {
        let mut parser = GRLParser;
        parser.parse_single_rule(grl_text)
    }

    /// Parse multiple rules from GRL text
    pub fn parse_rules(grl_text: &str) -> Result<Vec<Rule>> {
        let mut parser = GRLParser;
        parser.parse_multiple_rules(grl_text)
    }

    fn parse_single_rule(&mut self, grl_text: &str) -> Result<Rule> {
        let cleaned = self.clean_text(grl_text);

        // Extract rule components using regex - support quoted rule names
        let rule_regex =
            Regex::new(r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*(?:salience\s+(\d+))?\s*\{(.+)\}"#)
                .map_err(|e| RuleEngineError::ParseError {
                    message: format!("Invalid rule regex: {}", e),
                })?;

        let captures =
            rule_regex
                .captures(&cleaned)
                .ok_or_else(|| RuleEngineError::ParseError {
                    message: format!("Invalid GRL rule format. Input: {}", cleaned),
                })?;

        // Rule name can be either quoted (group 1) or unquoted (group 2)
        let rule_name = if let Some(quoted_name) = captures.get(1) {
            quoted_name.as_str().to_string()
        } else if let Some(unquoted_name) = captures.get(2) {
            unquoted_name.as_str().to_string()
        } else {
            return Err(RuleEngineError::ParseError {
                message: "Could not extract rule name".to_string(),
            });
        };
        // Extract salience and rule body
        let salience = captures
            .get(3)
            .and_then(|m| m.as_str().parse::<i32>().ok())
            .unwrap_or(0);

        let rule_body = captures.get(4).unwrap().as_str();

        // Parse when and then sections
        let when_then_regex =
            Regex::new(r"when\s+(.+?)\s+then\s+(.+)").map_err(|e| RuleEngineError::ParseError {
                message: format!("Invalid when-then regex: {}", e),
            })?;

        let when_then_captures =
            when_then_regex
                .captures(rule_body)
                .ok_or_else(|| RuleEngineError::ParseError {
                    message: "Missing when or then clause".to_string(),
                })?;

        let when_clause = when_then_captures.get(1).unwrap().as_str().trim();
        let then_clause = when_then_captures.get(2).unwrap().as_str().trim();

        // Parse conditions
        let conditions = self.parse_when_clause(when_clause)?;

        // Parse actions
        let actions = self.parse_then_clause(then_clause)?;

        // Build rule
        let mut rule = Rule::new(rule_name, conditions, actions);
        rule = rule.with_priority(salience);

        Ok(rule)
    }

    fn parse_multiple_rules(&mut self, grl_text: &str) -> Result<Vec<Rule>> {
        // Split by rule boundaries - support both quoted and unquoted rule names
        // Use DOTALL flag to match newlines in rule body
        let rule_regex =
            Regex::new(r#"(?s)rule\s+(?:"[^"]+"|[a-zA-Z_]\w*).*?\}"#).map_err(|e| {
                RuleEngineError::ParseError {
                    message: format!("Rule splitting regex error: {}", e),
                }
            })?;

        let mut rules = Vec::new();

        for rule_match in rule_regex.find_iter(grl_text) {
            let rule_text = rule_match.as_str();
            let rule = self.parse_single_rule(rule_text)?;
            rules.push(rule);
        }

        Ok(rules)
    }

    fn clean_text(&self, text: &str) -> String {
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn parse_when_clause(&self, when_clause: &str) -> Result<ConditionGroup> {
        // Handle logical operators with proper parentheses support
        let trimmed = when_clause.trim();

        // Strip outer parentheses if they exist
        let clause = if trimmed.starts_with('(') && trimmed.ends_with(')') {
            // Check if these are the outermost parentheses
            let inner = &trimmed[1..trimmed.len() - 1];
            if self.is_balanced_parentheses(inner) {
                inner
            } else {
                trimmed
            }
        } else {
            trimmed
        };

        // Parse OR at the top level (lowest precedence)
        if let Some(parts) = self.split_logical_operator(clause, "||") {
            return self.parse_or_parts(parts);
        }

        // Parse AND (higher precedence)
        if let Some(parts) = self.split_logical_operator(clause, "&&") {
            return self.parse_and_parts(parts);
        }

        // Handle NOT condition
        if clause.trim_start().starts_with("!") {
            return self.parse_not_condition(clause);
        }

        // Single condition
        self.parse_single_condition(clause)
    }

    fn is_balanced_parentheses(&self, text: &str) -> bool {
        let mut count = 0;
        for ch in text.chars() {
            match ch {
                '(' => count += 1,
                ')' => {
                    count -= 1;
                    if count < 0 {
                        return false;
                    }
                }
                _ => {}
            }
        }
        count == 0
    }

    fn split_logical_operator(&self, clause: &str, operator: &str) -> Option<Vec<String>> {
        let mut parts = Vec::new();
        let mut current_part = String::new();
        let mut paren_count = 0;
        let mut chars = clause.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '(' => {
                    paren_count += 1;
                    current_part.push(ch);
                }
                ')' => {
                    paren_count -= 1;
                    current_part.push(ch);
                }
                '&' if operator == "&&" && paren_count == 0 => {
                    if chars.peek() == Some(&'&') {
                        chars.next(); // consume second &
                        parts.push(current_part.trim().to_string());
                        current_part.clear();
                    } else {
                        current_part.push(ch);
                    }
                }
                '|' if operator == "||" && paren_count == 0 => {
                    if chars.peek() == Some(&'|') {
                        chars.next(); // consume second |
                        parts.push(current_part.trim().to_string());
                        current_part.clear();
                    } else {
                        current_part.push(ch);
                    }
                }
                _ => {
                    current_part.push(ch);
                }
            }
        }

        if !current_part.trim().is_empty() {
            parts.push(current_part.trim().to_string());
        }

        if parts.len() > 1 {
            Some(parts)
        } else {
            None
        }
    }

    fn parse_or_parts(&self, parts: Vec<String>) -> Result<ConditionGroup> {
        let mut conditions = Vec::new();
        for part in parts {
            let condition = self.parse_when_clause(&part)?;
            conditions.push(condition);
        }

        if conditions.is_empty() {
            return Err(RuleEngineError::ParseError {
                message: "No conditions found in OR".to_string(),
            });
        }

        let mut iter = conditions.into_iter();
        let mut result = iter.next().unwrap();
        for condition in iter {
            result = ConditionGroup::or(result, condition);
        }

        Ok(result)
    }

    fn parse_and_parts(&self, parts: Vec<String>) -> Result<ConditionGroup> {
        let mut conditions = Vec::new();
        for part in parts {
            let condition = self.parse_when_clause(&part)?;
            conditions.push(condition);
        }

        if conditions.is_empty() {
            return Err(RuleEngineError::ParseError {
                message: "No conditions found in AND".to_string(),
            });
        }

        let mut iter = conditions.into_iter();
        let mut result = iter.next().unwrap();
        for condition in iter {
            result = ConditionGroup::and(result, condition);
        }

        Ok(result)
    }

    fn parse_not_condition(&self, clause: &str) -> Result<ConditionGroup> {
        let inner_clause = clause.strip_prefix("!").unwrap().trim();
        let inner_condition = self.parse_when_clause(inner_clause)?;
        Ok(ConditionGroup::not(inner_condition))
    }

    fn parse_single_condition(&self, clause: &str) -> Result<ConditionGroup> {
        // Remove outer parentheses if they exist (handle new syntax like "(user.age >= 18)")
        let trimmed_clause = clause.trim();
        let clause_to_parse = if trimmed_clause.starts_with('(') && trimmed_clause.ends_with(')') {
            trimmed_clause[1..trimmed_clause.len() - 1].trim()
        } else {
            trimmed_clause
        };

        // Handle typed object conditions like: $TestCar : TestCarClass( speedUp == true && speed < maxSpeed )
        let typed_object_regex =
            Regex::new(r#"\$(\w+)\s*:\s*(\w+)\s*\(\s*(.+?)\s*\)"#).map_err(|e| {
                RuleEngineError::ParseError {
                    message: format!("Typed object regex error: {}", e),
                }
            })?;

        if let Some(captures) = typed_object_regex.captures(clause_to_parse) {
            let _object_name = captures.get(1).unwrap().as_str();
            let _object_type = captures.get(2).unwrap().as_str();
            let conditions_str = captures.get(3).unwrap().as_str();

            // Parse conditions inside parentheses
            return self.parse_conditions_within_object(conditions_str);
        }

        // Parse expressions like: User.Age >= 18, Product.Price < 100.0, user.age >= 18, etc.
        // Support both PascalCase (User.Age) and lowercase (user.age) field naming
        let condition_regex = Regex::new(
            r#"([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)*)\s*(>=|<=|==|!=|>|<|contains|matches)\s*(.+)"#,
        )
        .map_err(|e| RuleEngineError::ParseError {
            message: format!("Condition regex error: {}", e),
        })?;

        let captures = condition_regex.captures(clause_to_parse).ok_or_else(|| {
            RuleEngineError::ParseError {
                message: format!("Invalid condition format: {}", clause_to_parse),
            }
        })?;

        let field = captures.get(1).unwrap().as_str().to_string();
        let operator_str = captures.get(2).unwrap().as_str();
        let value_str = captures.get(3).unwrap().as_str().trim();

        let operator =
            Operator::from_str(operator_str).ok_or_else(|| RuleEngineError::InvalidOperator {
                operator: operator_str.to_string(),
            })?;

        let value = self.parse_value(value_str)?;

        let condition = Condition::new(field, operator, value);
        Ok(ConditionGroup::single(condition))
    }

    fn parse_conditions_within_object(&self, conditions_str: &str) -> Result<ConditionGroup> {
        // Parse conditions like: speedUp == true && speed < maxSpeed
        let parts: Vec<&str> = conditions_str.split("&&").collect();

        let mut conditions = Vec::new();
        for part in parts {
            let trimmed = part.trim();
            let condition = self.parse_simple_condition(trimmed)?;
            conditions.push(condition);
        }

        // Combine with AND
        if conditions.is_empty() {
            return Err(RuleEngineError::ParseError {
                message: "No conditions found".to_string(),
            });
        }

        let mut iter = conditions.into_iter();
        let mut result = iter.next().unwrap();
        for condition in iter {
            result = ConditionGroup::and(result, condition);
        }

        Ok(result)
    }

    fn parse_simple_condition(&self, clause: &str) -> Result<ConditionGroup> {
        // Parse simple condition like: speedUp == true or speed < maxSpeed
        let condition_regex = Regex::new(r#"(\w+)\s*(>=|<=|==|!=|>|<)\s*(.+)"#).map_err(|e| {
            RuleEngineError::ParseError {
                message: format!("Simple condition regex error: {}", e),
            }
        })?;

        let captures =
            condition_regex
                .captures(clause)
                .ok_or_else(|| RuleEngineError::ParseError {
                    message: format!("Invalid simple condition format: {}", clause),
                })?;

        let field = captures.get(1).unwrap().as_str().to_string();
        let operator_str = captures.get(2).unwrap().as_str();
        let value_str = captures.get(3).unwrap().as_str().trim();

        let operator =
            Operator::from_str(operator_str).ok_or_else(|| RuleEngineError::InvalidOperator {
                operator: operator_str.to_string(),
            })?;

        let value = self.parse_value(value_str)?;

        let condition = Condition::new(field, operator, value);
        Ok(ConditionGroup::single(condition))
    }

    fn parse_value(&self, value_str: &str) -> Result<Value> {
        let trimmed = value_str.trim();

        // String literal
        if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            let unquoted = &trimmed[1..trimmed.len() - 1];
            return Ok(Value::String(unquoted.to_string()));
        }

        // Boolean
        if trimmed.eq_ignore_ascii_case("true") {
            return Ok(Value::Boolean(true));
        }
        if trimmed.eq_ignore_ascii_case("false") {
            return Ok(Value::Boolean(false));
        }

        // Null
        if trimmed.eq_ignore_ascii_case("null") {
            return Ok(Value::Null);
        }

        // Number (try integer first, then float)
        if let Ok(int_val) = trimmed.parse::<i64>() {
            return Ok(Value::Integer(int_val));
        }

        if let Ok(float_val) = trimmed.parse::<f64>() {
            return Ok(Value::Number(float_val));
        }

        // Field reference (like User.Name)
        if trimmed.contains('.') {
            return Ok(Value::String(trimmed.to_string()));
        }

        // Default to string
        Ok(Value::String(trimmed.to_string()))
    }

    fn parse_then_clause(&self, then_clause: &str) -> Result<Vec<ActionType>> {
        let statements: Vec<&str> = then_clause
            .split(';')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        let mut actions = Vec::new();

        for statement in statements {
            let action = self.parse_action_statement(statement)?;
            actions.push(action);
        }

        Ok(actions)
    }

    fn parse_action_statement(&self, statement: &str) -> Result<ActionType> {
        let trimmed = statement.trim();

        // Method call: $Object.method(args)
        let method_regex = Regex::new(r#"\$(\w+)\.(\w+)\s*\(([^)]*)\)"#).map_err(|e| {
            RuleEngineError::ParseError {
                message: format!("Method regex error: {}", e),
            }
        })?;

        if let Some(captures) = method_regex.captures(trimmed) {
            let object = captures.get(1).unwrap().as_str().to_string();
            let method = captures.get(2).unwrap().as_str().to_string();
            let args_str = captures.get(3).unwrap().as_str();

            let args = if args_str.trim().is_empty() {
                Vec::new()
            } else {
                self.parse_method_args(args_str)?
            };

            return Ok(ActionType::MethodCall {
                object,
                method,
                args,
            });
        }

        // Assignment: Field = Value
        if let Some(eq_pos) = trimmed.find('=') {
            let field = trimmed[..eq_pos].trim().to_string();
            let value_str = trimmed[eq_pos + 1..].trim();
            let value = self.parse_value(value_str)?;

            return Ok(ActionType::Set { field, value });
        }

        // Function calls: update($Object), retract($Object), etc.
        let func_regex =
            Regex::new(r#"(\w+)\s*\(\s*(.+?)?\s*\)"#).map_err(|e| RuleEngineError::ParseError {
                message: format!("Function regex error: {}", e),
            })?;

        if let Some(captures) = func_regex.captures(trimmed) {
            let function_name = captures.get(1).unwrap().as_str();
            let args_str = captures.get(2).map(|m| m.as_str()).unwrap_or("");

            match function_name.to_lowercase().as_str() {
                "update" => {
                    // Extract object name from $Object
                    let object_name = if let Some(stripped) = args_str.strip_prefix('$') {
                        stripped.to_string()
                    } else {
                        args_str.to_string()
                    };
                    Ok(ActionType::Update {
                        object: object_name,
                    })
                }
                "set" => {
                    // Handle set(field, value) format
                    let args = if args_str.is_empty() {
                        Vec::new()
                    } else {
                        args_str
                            .split(',')
                            .map(|arg| self.parse_value(arg.trim()))
                            .collect::<Result<Vec<_>>>()?
                    };

                    if args.len() >= 2 {
                        let field = args[0].to_string();
                        let value = args[1].clone();
                        Ok(ActionType::Set { field, value })
                    } else if args.len() == 1 {
                        // set(field) - set to true by default
                        Ok(ActionType::Set {
                            field: args[0].to_string(),
                            value: Value::Boolean(true),
                        })
                    } else {
                        Ok(ActionType::Custom {
                            action_type: "set".to_string(),
                            params: {
                                let mut params = HashMap::new();
                                params.insert(
                                    "args".to_string(),
                                    Value::String(args_str.to_string()),
                                );
                                params
                            },
                        })
                    }
                }
                "add" => {
                    // Handle add(value) format
                    let value = if args_str.is_empty() {
                        Value::Integer(1) // Default increment
                    } else {
                        self.parse_value(args_str.trim())?
                    };
                    Ok(ActionType::Custom {
                        action_type: "add".to_string(),
                        params: {
                            let mut params = HashMap::new();
                            params.insert("value".to_string(), value);
                            params
                        },
                    })
                }
                "log" => {
                    let message = if args_str.is_empty() {
                        "Log message".to_string()
                    } else {
                        let value = self.parse_value(args_str.trim())?;
                        value.to_string()
                    };
                    Ok(ActionType::Log { message })
                }
                _ => {
                    let args = if args_str.is_empty() {
                        Vec::new()
                    } else {
                        args_str
                            .split(',')
                            .map(|arg| self.parse_value(arg.trim()))
                            .collect::<Result<Vec<_>>>()?
                    };
                    Ok(ActionType::Call {
                        function: function_name.to_string(),
                        args,
                    })
                }
            }
        } else {
            // Custom statement
            Ok(ActionType::Custom {
                action_type: "statement".to_string(),
                params: {
                    let mut params = HashMap::new();
                    params.insert("statement".to_string(), Value::String(trimmed.to_string()));
                    params
                },
            })
        }
    }

    fn parse_method_args(&self, args_str: &str) -> Result<Vec<Value>> {
        if args_str.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Handle expressions like: $TestCar.Speed + $TestCar.SpeedIncrement
        let mut args = Vec::new();
        let parts: Vec<&str> = args_str.split(',').collect();

        for part in parts {
            let trimmed = part.trim();

            // Handle arithmetic expressions
            if trimmed.contains('+')
                || trimmed.contains('-')
                || trimmed.contains('*')
                || trimmed.contains('/')
            {
                // For now, store as string - the engine will evaluate
                args.push(Value::String(trimmed.to_string()));
            } else {
                args.push(self.parse_value(trimmed)?);
            }
        }

        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::GRLParser;

    #[test]
    fn test_parse_simple_rule() {
        let grl = r#"
        rule "CheckAge" salience 10 {
            when
                User.Age >= 18
            then
                log("User is adult");
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.name, "CheckAge");
        assert_eq!(rule.salience, 10);
        assert_eq!(rule.actions.len(), 1);
    }

    #[test]
    fn test_parse_complex_condition() {
        let grl = r#"
        rule "ComplexRule" {
            when
                User.Age >= 18 && User.Country == "US"
            then
                User.Qualified = true;
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.name, "ComplexRule");
    }

    #[test]
    fn test_parse_new_syntax_with_parentheses() {
        let grl = r#"
        rule "Default Rule" salience 10 {
            when
                (user.age >= 18)
            then
                set(user.status, "approved");
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.name, "Default Rule");
        assert_eq!(rule.salience, 10);
        assert_eq!(rule.actions.len(), 1);

        // Check that the action is parsed as a Set action
        match &rule.actions[0] {
            crate::types::ActionType::Set { field, value } => {
                assert_eq!(field, "user.status");
                assert_eq!(value, &crate::types::Value::String("approved".to_string()));
            }
            _ => panic!("Expected Set action, got: {:?}", rule.actions[0]),
        }
    }

    #[test]
    fn test_parse_complex_nested_conditions() {
        let grl = r#"
        rule "Complex Business Rule" salience 10 {
            when
                (((user.vipStatus == true) && (order.amount > 500)) || ((date.isHoliday == true) && (order.hasCoupon == true)))
            then
                apply_discount(20000);
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.name, "Complex Business Rule");
        assert_eq!(rule.salience, 10);
        assert_eq!(rule.actions.len(), 1);

        // Check that the action is parsed as a function call
        match &rule.actions[0] {
            crate::types::ActionType::Call { function, args } => {
                assert_eq!(function, "apply_discount");
                assert_eq!(args.len(), 1);
                assert_eq!(args[0], crate::types::Value::Integer(20000));
            }
            _ => panic!("Expected Call action, got: {:?}", rule.actions[0]),
        }
    }
}

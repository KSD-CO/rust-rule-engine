use crate::engine::rule::{Condition, ConditionGroup, Rule};
use crate::errors::{Result, RuleEngineError};
use crate::types::{ActionType, Operator, Value};
use chrono::{DateTime, Utc};
use regex::Regex;
use std::collections::HashMap;

/// GRL (Grule Rule Language) Parser
/// Parses Grule-like syntax into Rule objects
pub struct GRLParser;

/// Parsed rule attributes from GRL header
#[derive(Debug, Default)]
struct RuleAttributes {
    pub no_loop: bool,
    pub lock_on_active: bool,
    pub agenda_group: Option<String>,
    pub activation_group: Option<String>,
    pub date_effective: Option<DateTime<Utc>>,
    pub date_expires: Option<DateTime<Utc>>,
}

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

        // Extract rule components using regex - support various attributes
        let rule_regex = Regex::new(r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))\s*([^{]*)\{(.+)\}"#)
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

        // Attributes section (group 3)
        let attributes_section = captures.get(3).map(|m| m.as_str()).unwrap_or("");

        // Rule body (group 4)
        let rule_body = captures.get(4).unwrap().as_str();

        // Parse salience from attributes section
        let salience = self.extract_salience(attributes_section)?;

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

        // Parse conditions and actions
        let conditions = self.parse_when_clause(when_clause)?;
        let actions = self.parse_then_clause(then_clause)?;

        // Parse all attributes from rule header
        let attributes = self.parse_rule_attributes(attributes_section)?;

        // Build rule
        let mut rule = Rule::new(rule_name, conditions, actions);
        rule = rule.with_priority(salience);

        // Apply parsed attributes
        if attributes.no_loop {
            rule = rule.with_no_loop(true);
        }
        if attributes.lock_on_active {
            rule = rule.with_lock_on_active(true);
        }
        if let Some(agenda_group) = attributes.agenda_group {
            rule = rule.with_agenda_group(agenda_group);
        }
        if let Some(activation_group) = attributes.activation_group {
            rule = rule.with_activation_group(activation_group);
        }
        if let Some(date_effective) = attributes.date_effective {
            rule = rule.with_date_effective(date_effective);
        }
        if let Some(date_expires) = attributes.date_expires {
            rule = rule.with_date_expires(date_expires);
        }

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

    /// Parse rule attributes from the rule header
    fn parse_rule_attributes(&self, rule_header: &str) -> Result<RuleAttributes> {
        let mut attributes = RuleAttributes::default();

        // Check for simple boolean attributes
        if rule_header.contains("no-loop") {
            attributes.no_loop = true;
        }
        if rule_header.contains("lock-on-active") {
            attributes.lock_on_active = true;
        }

        // Parse agenda-group attribute
        if let Some(agenda_group) = self.extract_quoted_attribute(rule_header, "agenda-group")? {
            attributes.agenda_group = Some(agenda_group);
        }

        // Parse activation-group attribute
        if let Some(activation_group) =
            self.extract_quoted_attribute(rule_header, "activation-group")?
        {
            attributes.activation_group = Some(activation_group);
        }

        // Parse date-effective attribute
        if let Some(date_str) = self.extract_quoted_attribute(rule_header, "date-effective")? {
            attributes.date_effective = Some(self.parse_date_string(&date_str)?);
        }

        // Parse date-expires attribute
        if let Some(date_str) = self.extract_quoted_attribute(rule_header, "date-expires")? {
            attributes.date_expires = Some(self.parse_date_string(&date_str)?);
        }

        Ok(attributes)
    }

    /// Extract quoted attribute value from rule header
    fn extract_quoted_attribute(&self, header: &str, attribute: &str) -> Result<Option<String>> {
        let pattern = format!(r#"{}\s+"([^"]+)""#, attribute);
        let regex = Regex::new(&pattern).map_err(|e| RuleEngineError::ParseError {
            message: format!("Invalid attribute regex for {}: {}", attribute, e),
        })?;

        if let Some(captures) = regex.captures(header) {
            if let Some(value) = captures.get(1) {
                return Ok(Some(value.as_str().to_string()));
            }
        }

        Ok(None)
    }

    /// Parse date string in various formats
    fn parse_date_string(&self, date_str: &str) -> Result<DateTime<Utc>> {
        // Try ISO 8601 format first
        if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(date.with_timezone(&Utc));
        }

        // Try simple date formats
        let formats = ["%Y-%m-%d", "%Y-%m-%dT%H:%M:%S", "%d-%b-%Y", "%d-%m-%Y"];

        for format in &formats {
            if let Ok(naive_date) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
                return Ok(naive_date.and_utc());
            }
            if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, format) {
                return Ok(naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
            }
        }

        Err(RuleEngineError::ParseError {
            message: format!("Unable to parse date: {}", date_str),
        })
    }

    /// Extract salience value from attributes section
    fn extract_salience(&self, attributes_section: &str) -> Result<i32> {
        let salience_regex =
            Regex::new(r"salience\s+(\d+)").map_err(|e| RuleEngineError::ParseError {
                message: format!("Invalid salience regex: {}", e),
            })?;

        if let Some(captures) = salience_regex.captures(attributes_section) {
            if let Some(salience_match) = captures.get(1) {
                return salience_match.as_str().parse::<i32>().map_err(|e| {
                    RuleEngineError::ParseError {
                        message: format!("Invalid salience value: {}", e),
                    }
                });
            }
        }

        Ok(0) // Default salience
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

        // Handle EXISTS condition
        if clause.trim_start().starts_with("exists(") {
            return self.parse_exists_condition(clause);
        }

        // Handle FORALL condition
        if clause.trim_start().starts_with("forall(") {
            return self.parse_forall_condition(clause);
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

    fn parse_exists_condition(&self, clause: &str) -> Result<ConditionGroup> {
        let clause = clause.trim_start();
        if !clause.starts_with("exists(") || !clause.ends_with(")") {
            return Err(RuleEngineError::ParseError {
                message: "Invalid exists syntax. Expected: exists(condition)".to_string(),
            });
        }

        // Extract content between parentheses
        let inner_clause = &clause[7..clause.len() - 1]; // Remove "exists(" and ")"
        let inner_condition = self.parse_when_clause(inner_clause)?;
        Ok(ConditionGroup::exists(inner_condition))
    }

    fn parse_forall_condition(&self, clause: &str) -> Result<ConditionGroup> {
        let clause = clause.trim_start();
        if !clause.starts_with("forall(") || !clause.ends_with(")") {
            return Err(RuleEngineError::ParseError {
                message: "Invalid forall syntax. Expected: forall(condition)".to_string(),
            });
        }

        // Extract content between parentheses
        let inner_clause = &clause[7..clause.len() - 1]; // Remove "forall(" and ")"
        let inner_condition = self.parse_when_clause(inner_clause)?;
        Ok(ConditionGroup::forall(inner_condition))
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
                "log" => {
                    let message = if args_str.is_empty() {
                        "Log message".to_string()
                    } else {
                        let value = self.parse_value(args_str.trim())?;
                        value.to_string()
                    };
                    Ok(ActionType::Log { message })
                }
                "activateagendagroup" | "activate_agenda_group" => {
                    let agenda_group = if args_str.is_empty() {
                        return Err(RuleEngineError::ParseError {
                            message: "ActivateAgendaGroup requires agenda group name".to_string(),
                        });
                    } else {
                        let value = self.parse_value(args_str.trim())?;
                        match value {
                            Value::String(s) => s,
                            _ => value.to_string(),
                        }
                    };
                    Ok(ActionType::ActivateAgendaGroup { group: agenda_group })
                }
                "schedulerule" | "schedule_rule" => {
                    // Parse delay and target rule: ScheduleRule(5000, "next-rule")
                    let parts: Vec<&str> = args_str.split(',').collect();
                    if parts.len() != 2 {
                        return Err(RuleEngineError::ParseError {
                            message: "ScheduleRule requires delay_ms and rule_name".to_string(),
                        });
                    }
                    
                    let delay_ms = self.parse_value(parts[0].trim())?;
                    let rule_name = self.parse_value(parts[1].trim())?;
                    
                    let delay_ms = match delay_ms {
                        Value::Integer(i) => i as u64,
                        Value::Number(f) => f as u64,
                        _ => return Err(RuleEngineError::ParseError {
                            message: "ScheduleRule delay_ms must be a number".to_string(),
                        }),
                    };
                    
                    let rule_name = match rule_name {
                        Value::String(s) => s,
                        _ => rule_name.to_string(),
                    };
                    
                    Ok(ActionType::ScheduleRule { delay_ms, rule_name })
                }
                "completeworkflow" | "complete_workflow" => {
                    let workflow_id = if args_str.is_empty() {
                        return Err(RuleEngineError::ParseError {
                            message: "CompleteWorkflow requires workflow_id".to_string(),
                        });
                    } else {
                        let value = self.parse_value(args_str.trim())?;
                        match value {
                            Value::String(s) => s,
                            _ => value.to_string(),
                        }
                    };
                    Ok(ActionType::CompleteWorkflow { workflow_name: workflow_id })
                }
                "setworkflowdata" | "set_workflow_data" => {
                    // Parse key=value: SetWorkflowData("key=value")
                    let data_str = args_str.trim();
                    
                    // Simple key=value parsing
                    let (key, value) = if let Some(eq_pos) = data_str.find('=') {
                        let key = data_str[..eq_pos].trim().trim_matches('"');
                        let value_str = data_str[eq_pos + 1..].trim();
                        let value = self.parse_value(value_str)?;
                        (key.to_string(), value)
                    } else {
                        return Err(RuleEngineError::ParseError {
                            message: "SetWorkflowData data must be in key=value format".to_string(),
                        });
                    };
                    
                    Ok(ActionType::SetWorkflowData { key, value })
                }
                _ => {
                    // All other functions become custom actions
                    let params = if args_str.is_empty() {
                        HashMap::new()
                    } else {
                        self.parse_function_args_as_params(args_str)?
                    };
                    
                    Ok(ActionType::Custom {
                        action_type: function_name.to_string(),
                        params,
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

    /// Parse function arguments as parameters for custom actions
    fn parse_function_args_as_params(&self, args_str: &str) -> Result<HashMap<String, Value>> {
        let mut params = HashMap::new();

        if args_str.trim().is_empty() {
            return Ok(params);
        }

        // Parse positional parameters as numbered args
        let parts: Vec<&str> = args_str.split(',').collect();
        for (i, part) in parts.iter().enumerate() {
            let trimmed = part.trim();
            let value = self.parse_value(trimmed)?;
            
            // Use simple numeric indexing - engine will resolve references dynamically
            params.insert(i.to_string(), value);
        }

        Ok(params)
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

        // Check that the action is parsed as a Custom action (set is now custom)
        match &rule.actions[0] {
            crate::types::ActionType::Custom { action_type, params } => {
                assert_eq!(action_type, "set");
                assert_eq!(params.get("0"), Some(&crate::types::Value::String("user.status".to_string())));
                assert_eq!(params.get("1"), Some(&crate::types::Value::String("approved".to_string())));
            }
            _ => panic!("Expected Custom action, got: {:?}", rule.actions[0]),
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

        // Check that the action is parsed as a Custom action (apply_discount is now custom)
        match &rule.actions[0] {
            crate::types::ActionType::Custom { action_type, params } => {
                assert_eq!(action_type, "apply_discount");
                assert_eq!(params.get("0"), Some(&crate::types::Value::Integer(20000)));
            }
            _ => panic!("Expected Custom action, got: {:?}", rule.actions[0]),
        }
    }

    #[test]
    fn test_parse_no_loop_attribute() {
        let grl = r#"
        rule "NoLoopRule" no-loop salience 15 {
            when
                User.Score < 100
            then
                set(User.Score, User.Score + 10);
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.name, "NoLoopRule");
        assert_eq!(rule.salience, 15);
        assert!(rule.no_loop, "Rule should have no-loop=true");
    }

    #[test]
    fn test_parse_no_loop_different_positions() {
        // Test no-loop before salience
        let grl1 = r#"
        rule "Rule1" no-loop salience 10 {
            when User.Age >= 18
            then log("adult");
        }
        "#;

        // Test no-loop after salience
        let grl2 = r#"
        rule "Rule2" salience 10 no-loop {
            when User.Age >= 18
            then log("adult");
        }
        "#;

        let rules1 = GRLParser::parse_rules(grl1).unwrap();
        let rules2 = GRLParser::parse_rules(grl2).unwrap();

        assert_eq!(rules1.len(), 1);
        assert_eq!(rules2.len(), 1);

        assert!(rules1[0].no_loop, "Rule1 should have no-loop=true");
        assert!(rules2[0].no_loop, "Rule2 should have no-loop=true");

        assert_eq!(rules1[0].salience, 10);
        assert_eq!(rules2[0].salience, 10);
    }

    #[test]
    fn test_parse_without_no_loop() {
        let grl = r#"
        rule "RegularRule" salience 5 {
            when
                User.Active == true
            then
                log("active user");
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.name, "RegularRule");
        assert!(!rule.no_loop, "Rule should have no-loop=false by default");
    }

    #[test]
    fn test_parse_exists_pattern() {
        let grl = r#"
        rule "ExistsRule" salience 20 {
            when
                exists(Customer.tier == "VIP")
            then
                System.premiumActive = true;
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.name, "ExistsRule");
        assert_eq!(rule.salience, 20);

        // Check that condition is EXISTS pattern
        match &rule.conditions {
            crate::engine::rule::ConditionGroup::Exists(_) => {
                // Test passes
            }
            _ => panic!(
                "Expected EXISTS condition group, got: {:?}",
                rule.conditions
            ),
        }
    }

    #[test]
    fn test_parse_forall_pattern() {
        let grl = r#"
        rule "ForallRule" salience 15 {
            when
                forall(Order.status == "processed")
            then
                Shipping.enabled = true;
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.name, "ForallRule");

        // Check that condition is FORALL pattern
        match &rule.conditions {
            crate::engine::rule::ConditionGroup::Forall(_) => {
                // Test passes
            }
            _ => panic!(
                "Expected FORALL condition group, got: {:?}",
                rule.conditions
            ),
        }
    }

    #[test]
    fn test_parse_combined_patterns() {
        let grl = r#"
        rule "CombinedRule" salience 25 {
            when
                exists(Customer.tier == "VIP") && !exists(Alert.priority == "high")
            then
                System.vipMode = true;
        }
        "#;

        let rules = GRLParser::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.name, "CombinedRule");

        // Check that condition is AND with EXISTS and NOT(EXISTS) patterns
        match &rule.conditions {
            crate::engine::rule::ConditionGroup::Compound {
                left,
                operator,
                right,
            } => {
                assert_eq!(*operator, crate::types::LogicalOperator::And);

                // Left should be EXISTS
                match left.as_ref() {
                    crate::engine::rule::ConditionGroup::Exists(_) => {
                        // Expected
                    }
                    _ => panic!("Expected EXISTS in left side, got: {:?}", left),
                }

                // Right should be NOT(EXISTS)
                match right.as_ref() {
                    crate::engine::rule::ConditionGroup::Not(inner) => {
                        match inner.as_ref() {
                            crate::engine::rule::ConditionGroup::Exists(_) => {
                                // Expected
                            }
                            _ => panic!("Expected EXISTS inside NOT, got: {:?}", inner),
                        }
                    }
                    _ => panic!("Expected NOT in right side, got: {:?}", right),
                }
            }
            _ => panic!("Expected compound condition, got: {:?}", rule.conditions),
        }
    }
}

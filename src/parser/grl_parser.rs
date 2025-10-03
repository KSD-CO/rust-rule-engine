use crate::engine::rule::{Condition, ConditionGroup, Rule};
use crate::errors::{Result, RuleEngineError};
use crate::types::{ActionType, Operator, Value};

/// Simple GRL Parser for parsing rule files
pub struct GRLParser;

impl GRLParser {
    /// Parse a complete GRL file content into rules
    pub fn parse_rules(content: &str) -> Result<Vec<Rule>> {
        let mut rules = Vec::new();

        // Split content by rule blocks
        let rule_blocks = Self::extract_rule_blocks(content)?;

        for block in rule_blocks {
            let rule = Self::parse_single_rule(&block)?;
            rules.push(rule);
        }

        Ok(rules)
    }

    /// Extract individual rule blocks from content
    fn extract_rule_blocks(content: &str) -> Result<Vec<String>> {
        let mut blocks = Vec::new();
        let mut current_block = String::new();
        let mut brace_count = 0;
        let mut in_rule = false;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("rule ") {
                in_rule = true;
                current_block.clear();
                current_block.push_str(line);
                current_block.push('\n');

                // Count braces in the rule declaration line
                for ch in line.chars() {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                blocks.push(current_block.clone());
                                current_block.clear();
                                in_rule = false;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                continue;
            }

            if in_rule {
                current_block.push_str(line);
                current_block.push('\n');

                for ch in line.chars() {
                    match ch {
                        '{' => brace_count += 1,
                        '}' => {
                            brace_count -= 1;
                            if brace_count == 0 {
                                blocks.push(current_block.clone());
                                current_block.clear();
                                in_rule = false;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(blocks)
    }

    /// Parse a single rule block
    fn parse_single_rule(block: &str) -> Result<Rule> {
        let lines: Vec<&str> = block.lines().collect();

        // Parse rule header
        let (name, salience) = Self::parse_rule_header(lines[0])?;

        // Find when and then sections
        let mut when_section = Vec::new();
        let mut then_section = Vec::new();
        let mut current_section = "";

        for line in lines.iter().skip(1) {
            let line = line.trim();

            if line == "when" {
                current_section = "when";
                continue;
            } else if line == "then" {
                current_section = "then";
                continue;
            } else if line == "}" || line.is_empty() {
                continue;
            }

            match current_section {
                "when" => when_section.push(line),
                "then" => then_section.push(line),
                _ => {}
            }
        }

        // Parse conditions
        let conditions = Self::parse_conditions(&when_section)?;

        // Parse actions
        let actions = Self::parse_actions(&then_section)?;

        Ok(Rule::new(name, conditions, actions).with_salience(salience))
    }

    /// Parse rule header to extract name and salience
    fn parse_rule_header(header: &str) -> Result<(String, i32)> {
        // Example: rule "SpeedLimitCheck" salience 20 {
        let parts: Vec<&str> = header.split_whitespace().collect();

        let name = if parts.len() >= 2 {
            parts[1].trim_matches('"').to_string()
        } else {
            return Err(RuleEngineError::ParseError {
                message: "Invalid rule header".to_string(),
            });
        };

        let salience = if let Some(sal_idx) = parts.iter().position(|&x| x == "salience") {
            if sal_idx + 1 < parts.len() {
                parts[sal_idx + 1].parse::<i32>().unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        Ok((name, salience))
    }

    /// Parse when conditions
    fn parse_conditions(when_lines: &[&str]) -> Result<ConditionGroup> {
        if when_lines.is_empty() {
            return Err(RuleEngineError::ParseError {
                message: "No conditions found".to_string(),
            });
        }

        // Simple parsing - just take first condition for now
        let condition_line = when_lines.join(" ");

        // Parse simple conditions like: Car.Speed > 80.0
        if let Some((field, op, value)) = Self::parse_simple_condition(&condition_line) {
            let condition = Condition::new(field, op, value);
            return Ok(ConditionGroup::single(condition));
        }

        // Default fallback
        Ok(ConditionGroup::single(Condition::new(
            "Default.Field".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )))
    }

    /// Parse a simple condition
    fn parse_simple_condition(condition: &str) -> Option<(String, Operator, Value)> {
        let condition = condition.trim();

        // Try different operators
        if let Some(pos) = condition.find(" >= ") {
            let field = condition[..pos].trim().to_string();
            let value_str = condition[pos + 4..].trim();
            let value = Self::parse_value(value_str)?;
            return Some((field, Operator::GreaterThanOrEqual, value));
        }

        if let Some(pos) = condition.find(" > ") {
            let field = condition[..pos].trim().to_string();
            let value_str = condition[pos + 3..].trim();
            let value = Self::parse_value(value_str)?;
            return Some((field, Operator::GreaterThan, value));
        }

        if let Some(pos) = condition.find(" <= ") {
            let field = condition[..pos].trim().to_string();
            let value_str = condition[pos + 4..].trim();
            let value = Self::parse_value(value_str)?;
            return Some((field, Operator::LessThanOrEqual, value));
        }

        if let Some(pos) = condition.find(" < ") {
            let field = condition[..pos].trim().to_string();
            let value_str = condition[pos + 3..].trim();
            let value = Self::parse_value(value_str)?;
            return Some((field, Operator::LessThan, value));
        }

        if let Some(pos) = condition.find(" == ") {
            let field = condition[..pos].trim().to_string();
            let value_str = condition[pos + 4..].trim();
            let value = Self::parse_value(value_str)?;
            return Some((field, Operator::Equal, value));
        }

        if let Some(pos) = condition.find(" != ") {
            let field = condition[..pos].trim().to_string();
            let value_str = condition[pos + 4..].trim();
            let value = Self::parse_value(value_str)?;
            return Some((field, Operator::NotEqual, value));
        }

        None
    }

    /// Parse a value from string
    fn parse_value(value_str: &str) -> Option<Value> {
        let value_str = value_str.trim();

        // Boolean
        if value_str == "true" {
            return Some(Value::Boolean(true));
        }
        if value_str == "false" {
            return Some(Value::Boolean(false));
        }

        // String (quoted)
        if value_str.starts_with('"') && value_str.ends_with('"') {
            let s = value_str[1..value_str.len() - 1].to_string();
            return Some(Value::String(s));
        }

        // Number (float)
        if let Ok(f) = value_str.parse::<f64>() {
            return Some(Value::Number(f));
        }

        // Integer
        if let Ok(i) = value_str.parse::<i64>() {
            return Some(Value::Integer(i));
        }

        // Field reference
        Some(Value::String(value_str.to_string()))
    }

    /// Parse then actions
    fn parse_actions(then_lines: &[&str]) -> Result<Vec<ActionType>> {
        let mut actions = Vec::new();

        for line in then_lines {
            let line = line.trim().trim_end_matches(';');

            if line.is_empty() {
                continue;
            }

            // Parse different action types
            if let Some(action) = Self::parse_action_line(line) {
                actions.push(action);
            }
        }

        Ok(actions)
    }

    /// Parse a single action line
    fn parse_action_line(line: &str) -> Option<ActionType> {
        // Method calls: Object.setProperty(value)
        if let Some(dot_pos) = line.find('.') {
            if let Some(paren_pos) = line.find('(') {
                if dot_pos < paren_pos {
                    let object = line[..dot_pos].trim().to_string();
                    let method = line[dot_pos + 1..paren_pos].trim().to_string();

                    // Extract arguments
                    if let Some(close_paren) = line.rfind(')') {
                        let args_str = &line[paren_pos + 1..close_paren];
                        let args = Self::parse_function_args(args_str);

                        return Some(ActionType::MethodCall {
                            object,
                            method,
                            args,
                        });
                    }
                }
            }
        }

        // Function calls: functionName(args)
        if let Some(paren_pos) = line.find('(') {
            let function = line[..paren_pos].trim().to_string();
            if let Some(close_paren) = line.rfind(')') {
                let args_str = &line[paren_pos + 1..close_paren];
                let args = Self::parse_function_args(args_str);

                return Some(ActionType::Call { function, args });
            }
        }

        None
    }

    /// Parse function arguments
    fn parse_function_args(args_str: &str) -> Vec<Value> {
        if args_str.trim().is_empty() {
            return Vec::new();
        }

        args_str
            .split(',')
            .map(|arg| {
                let arg = arg.trim();
                Self::parse_value(arg).unwrap_or(Value::String(arg.to_string()))
            })
            .collect()
    }
}

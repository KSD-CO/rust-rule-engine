/// GRL Parser without regex dependency
///
/// This module provides full GRL parsing using only memchr and manual string parsing.
/// It's 4-60x faster than the regex-based GRLParser and has no regex dependency.
use crate::engine::module::{ExportItem, ExportList, ImportType, ItemType, ModuleManager};
use crate::engine::rule::{Condition, ConditionGroup, Rule};
use crate::errors::{Result, RuleEngineError};
use crate::types::{ActionType, Operator, Value};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::literal_search;

/// GRL Parser - No Regex Version
///
/// Parses Grule-like syntax into Rule objects without using regex.
/// This is the recommended parser for new code - it's faster and has fewer dependencies.
pub struct GRLParserNoRegex;

/// Result from parsing GRL with modules
#[derive(Debug, Clone)]
pub struct ParsedGRL {
    /// Parsed rules
    pub rules: Vec<Rule>,
    /// Module manager with configured modules
    pub module_manager: ModuleManager,
    /// Map of rule name to module name
    pub rule_modules: HashMap<String, String>,
}

impl Default for ParsedGRL {
    fn default() -> Self {
        Self::new()
    }
}

impl ParsedGRL {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            module_manager: ModuleManager::new(),
            rule_modules: HashMap::new(),
        }
    }
}

/// Parsed rule attributes
#[derive(Debug, Default)]
struct RuleAttributes {
    pub salience: i32,
    pub no_loop: bool,
    pub lock_on_active: bool,
    pub agenda_group: Option<String>,
    pub activation_group: Option<String>,
    pub date_effective: Option<DateTime<Utc>>,
    pub date_expires: Option<DateTime<Utc>>,
}

impl GRLParserNoRegex {
    /// Parse multiple rules from GRL text
    pub fn parse_rules(grl_text: &str) -> Result<Vec<Rule>> {
        let rule_texts = split_into_rules(grl_text);
        let mut rules = Vec::with_capacity(rule_texts.len());

        for rule_text in rule_texts {
            let rule = Self::parse_single_rule(&rule_text)?;
            rules.push(rule);
        }

        Ok(rules)
    }

    /// Parse a single rule from GRL syntax
    pub fn parse_rule(grl_text: &str) -> Result<Rule> {
        Self::parse_single_rule(grl_text)
    }

    /// Parse GRL text with module support
    pub fn parse_with_modules(grl_text: &str) -> Result<ParsedGRL> {
        let mut result = ParsedGRL::new();

        // Split modules and rules
        let (module_texts, rules_text) = split_modules_and_rules(grl_text);

        // Parse modules
        for module_text in module_texts {
            Self::parse_and_register_module(&module_text, &mut result.module_manager)?;
        }

        // Parse rules
        let rules = Self::parse_rules(&rules_text)?;

        // Assign rules to modules
        for rule in rules {
            let module_name = extract_module_from_context(grl_text, &rule.name);
            result
                .rule_modules
                .insert(rule.name.clone(), module_name.clone());

            if let Ok(module) = result.module_manager.get_module_mut(&module_name) {
                module.add_rule(&rule.name);
            }

            result.rules.push(rule);
        }

        Ok(result)
    }

    fn parse_single_rule(grl_text: &str) -> Result<Rule> {
        let cleaned = clean_text(grl_text);

        // Find "rule" keyword
        let rule_pos =
            find_keyword(&cleaned, "rule").ok_or_else(|| RuleEngineError::ParseError {
                message: "Missing 'rule' keyword".to_string(),
            })?;

        let after_rule = cleaned[rule_pos + 4..].trim_start();

        // Extract rule name (quoted or unquoted)
        let (rule_name, after_name) = extract_rule_name(after_rule)?;

        // Find opening brace
        let brace_pos = after_name
            .find('{')
            .ok_or_else(|| RuleEngineError::ParseError {
                message: "Missing opening brace".to_string(),
            })?;

        let attributes_section = &after_name[..brace_pos];
        let body_start = brace_pos + 1;

        // Find matching closing brace
        let body_with_brace = &after_name[brace_pos..];
        let close_pos =
            literal_search::find_matching_brace(body_with_brace, 0).ok_or_else(|| {
                RuleEngineError::ParseError {
                    message: "Missing closing brace".to_string(),
                }
            })?;

        let rule_body = &after_name[body_start..brace_pos + close_pos];

        // Parse attributes
        let attributes = parse_rule_attributes(attributes_section)?;

        // Parse when-then
        let (when_clause, then_clause) = parse_when_then(rule_body)?;

        // Parse conditions and actions
        let conditions = parse_when_clause(&when_clause)?;
        let actions = parse_then_clause(&then_clause)?;

        // Build rule
        let mut rule = Rule::new(rule_name, conditions, actions);
        rule = rule.with_priority(attributes.salience);

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

    fn parse_and_register_module(module_def: &str, manager: &mut ModuleManager) -> Result<()> {
        let (name, body, _) = parse_defmodule(module_def)?;

        let _ = manager.create_module(&name);
        let module = manager.get_module_mut(&name)?;

        // Parse export directive
        if let Some(export_type) = extract_directive(&body, "export:") {
            let exports = if export_type.trim() == "all" {
                ExportList::All
            } else if export_type.trim() == "none" {
                ExportList::None
            } else {
                ExportList::Specific(vec![ExportItem {
                    item_type: ItemType::All,
                    pattern: export_type.trim().to_string(),
                }])
            };
            module.set_exports(exports);
        }

        // Parse import directives
        for line in body.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import:") {
                if let Some(import_spec) = extract_directive(trimmed, "import:") {
                    Self::parse_import_spec(&name, &import_spec, manager)?;
                }
            }
        }

        Ok(())
    }

    fn parse_import_spec(
        importing_module: &str,
        spec: &str,
        manager: &mut ModuleManager,
    ) -> Result<()> {
        let parts: Vec<&str> = spec.splitn(2, '(').collect();
        if parts.is_empty() {
            return Ok(());
        }

        let source_module = parts[0].trim().to_string();
        let rest = if parts.len() > 1 { parts[1] } else { "" };

        if rest.contains("rules") {
            manager.import_from(importing_module, &source_module, ImportType::AllRules, "*")?;
        }

        if rest.contains("templates") {
            manager.import_from(
                importing_module,
                &source_module,
                ImportType::AllTemplates,
                "*",
            )?;
        }

        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Split GRL text into individual rules
fn split_into_rules(grl_text: &str) -> Vec<String> {
    let mut rules = Vec::new();
    let bytes = grl_text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if let Some(rule_pos) = memchr::memmem::find(&bytes[i..], b"rule ") {
            let abs_pos = i + rule_pos;

            // Check word boundary before "rule"
            if abs_pos > 0 && bytes[abs_pos - 1].is_ascii_alphanumeric() {
                i = abs_pos + 1;
                continue;
            }

            // Check if "rule " is inside a comment (look back for // on same line)
            if is_inside_comment(grl_text, abs_pos) {
                i = abs_pos + 5;
                continue;
            }

            if let Some(brace_pos) = memchr::memchr(b'{', &bytes[abs_pos..]) {
                let brace_abs = abs_pos + brace_pos;

                if let Some(close_pos) = literal_search::find_matching_brace(grl_text, brace_abs) {
                    let rule_text = &grl_text[abs_pos..=close_pos];
                    rules.push(rule_text.to_string());
                    i = close_pos + 1;
                    continue;
                }
            }
        }
        break;
    }

    rules
}

/// Check if a position is inside a single-line comment
fn is_inside_comment(text: &str, pos: usize) -> bool {
    // Find the start of the current line
    let bytes = text.as_bytes();
    let mut line_start = pos;
    while line_start > 0 && bytes[line_start - 1] != b'\n' {
        line_start -= 1;
    }

    // Check if there's a // between line_start and pos
    let line_prefix = &text[line_start..pos];
    line_prefix.contains("//")
}

/// Split modules and rules from GRL text
fn split_modules_and_rules(grl_text: &str) -> (Vec<String>, String) {
    let mut modules = Vec::new();
    let mut rules_text = String::new();
    let bytes = grl_text.as_bytes();
    let mut i = 0;
    let mut last_copy = 0;

    while i < bytes.len() {
        if let Some(offset) = memchr::memmem::find(&bytes[i..], b"defmodule ") {
            let abs_pos = i + offset;

            if abs_pos > last_copy {
                rules_text.push_str(&grl_text[last_copy..abs_pos]);
            }

            if let Some(brace_offset) = memchr::memchr(b'{', &bytes[abs_pos..]) {
                let brace_abs = abs_pos + brace_offset;

                if let Some(close_pos) = literal_search::find_matching_brace(grl_text, brace_abs) {
                    let module_text = &grl_text[abs_pos..=close_pos];
                    modules.push(module_text.to_string());
                    i = close_pos + 1;
                    last_copy = i;
                    continue;
                }
            }
        }
        i += 1;
    }

    if last_copy < grl_text.len() {
        rules_text.push_str(&grl_text[last_copy..]);
    }

    (modules, rules_text)
}

/// Clean text by removing comments and joining lines
fn clean_text(text: &str) -> String {
    text.lines()
        .map(|line| {
            // Remove single-line comments
            if let Some(comment_pos) = line.find("//") {
                line[..comment_pos].trim()
            } else {
                line.trim()
            }
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Find keyword at word boundary
fn find_keyword(text: &str, keyword: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let keyword_bytes = keyword.as_bytes();
    let mut pos = 0;

    while let Some(offset) = memchr::memmem::find(&bytes[pos..], keyword_bytes) {
        let abs_pos = pos + offset;

        // Check word boundaries
        let before_ok = abs_pos == 0 || !bytes[abs_pos - 1].is_ascii_alphanumeric();
        let after_pos = abs_pos + keyword_bytes.len();
        let after_ok = after_pos >= bytes.len() || !bytes[after_pos].is_ascii_alphanumeric();

        if before_ok && after_ok {
            return Some(abs_pos);
        }

        pos = abs_pos + 1;
    }

    None
}

/// Extract rule name (quoted or unquoted)
fn extract_rule_name(text: &str) -> Result<(String, &str)> {
    let trimmed = text.trim_start();

    // Try quoted name first
    if trimmed.starts_with('"') {
        if let Some(end_quote) = memchr::memchr(b'"', &trimmed.as_bytes()[1..]) {
            let name = trimmed[1..end_quote + 1].to_string();
            let remaining = &trimmed[end_quote + 2..];
            return Ok((name, remaining));
        }
        return Err(RuleEngineError::ParseError {
            message: "Unclosed quote in rule name".to_string(),
        });
    }

    // Try identifier
    let name_end = trimmed
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(trimmed.len());

    if name_end == 0 {
        return Err(RuleEngineError::ParseError {
            message: "Missing rule name".to_string(),
        });
    }

    let name = trimmed[..name_end].to_string();
    let remaining = &trimmed[name_end..];

    Ok((name, remaining))
}

/// Parse rule attributes from the attributes section
fn parse_rule_attributes(attrs: &str) -> Result<RuleAttributes> {
    let mut result = RuleAttributes::default();

    // Remove quoted strings to avoid false matches
    let cleaned = remove_quoted_strings(attrs);

    // Parse salience
    if let Some(salience_pos) = find_keyword(&cleaned, "salience") {
        let after_salience = cleaned[salience_pos + 8..].trim_start();
        let digits: String = after_salience
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        if let Ok(val) = digits.parse::<i32>() {
            result.salience = val;
        }
    }

    // Parse boolean flags
    result.no_loop = has_keyword(&cleaned, "no-loop");
    result.lock_on_active = has_keyword(&cleaned, "lock-on-active");

    // Parse quoted attributes from original (not cleaned)
    result.agenda_group = extract_quoted_attribute(attrs, "agenda-group");
    result.activation_group = extract_quoted_attribute(attrs, "activation-group");

    if let Some(date_str) = extract_quoted_attribute(attrs, "date-effective") {
        result.date_effective = parse_date_string(&date_str).ok();
    }

    if let Some(date_str) = extract_quoted_attribute(attrs, "date-expires") {
        result.date_expires = parse_date_string(&date_str).ok();
    }

    Ok(result)
}

/// Remove quoted strings from text
fn remove_quoted_strings(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_string = false;
    let mut escape_next = false;

    for ch in text.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            _ if !in_string => result.push(ch),
            _ => {}
        }
    }

    result
}

/// Check if keyword exists at word boundary
fn has_keyword(text: &str, keyword: &str) -> bool {
    find_keyword(text, keyword).is_some()
}

/// Extract quoted attribute value
fn extract_quoted_attribute(text: &str, attr_name: &str) -> Option<String> {
    let attr_pos = find_keyword(text, attr_name)?;
    let after_attr = text[attr_pos + attr_name.len()..].trim_start();

    if after_attr.starts_with('"') {
        let end_quote = memchr::memchr(b'"', &after_attr.as_bytes()[1..])?;
        Some(after_attr[1..end_quote + 1].to_string())
    } else {
        None
    }
}

/// Parse date string
fn parse_date_string(date_str: &str) -> Result<DateTime<Utc>> {
    if let Ok(date) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(date.with_timezone(&Utc));
    }

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

/// Parse when-then sections
fn parse_when_then(body: &str) -> Result<(String, String)> {
    let when_pos = find_keyword(body, "when").ok_or_else(|| RuleEngineError::ParseError {
        message: "Missing 'when' clause".to_string(),
    })?;

    let after_when = &body[when_pos + 4..];

    // Find "then" at the correct nesting level
    let then_pos = find_then_keyword(after_when).ok_or_else(|| RuleEngineError::ParseError {
        message: "Missing 'then' clause".to_string(),
    })?;

    let when_clause = after_when[..then_pos].trim().to_string();
    let then_clause = after_when[then_pos + 4..].trim().to_string();

    Ok((when_clause, then_clause))
}

/// Find "then" keyword at the correct nesting level
fn find_then_keyword(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut in_string = false;
    let mut escape_next = false;
    let mut paren_depth: i32 = 0;
    let mut brace_depth: i32 = 0;

    let mut i = 0;
    while i < bytes.len() {
        if escape_next {
            escape_next = false;
            i += 1;
            continue;
        }

        match bytes[i] {
            b'\\' if in_string => escape_next = true,
            b'"' => in_string = !in_string,
            b'(' if !in_string => paren_depth += 1,
            b')' if !in_string => paren_depth = paren_depth.saturating_sub(1),
            b'{' if !in_string => brace_depth += 1,
            b'}' if !in_string => brace_depth = brace_depth.saturating_sub(1),
            b't' if !in_string && paren_depth == 0 && brace_depth == 0 => {
                if i + 4 <= bytes.len() && &bytes[i..i + 4] == b"then" {
                    let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
                    let after_ok = i + 4 >= bytes.len() || !bytes[i + 4].is_ascii_alphanumeric();
                    if before_ok && after_ok {
                        return Some(i);
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

/// Parse defmodule declaration
fn parse_defmodule(text: &str) -> Result<(String, String, usize)> {
    let trimmed = text.trim_start();

    if !trimmed.starts_with("defmodule") {
        return Err(RuleEngineError::ParseError {
            message: "Expected 'defmodule'".to_string(),
        });
    }

    let after_defmodule = trimmed[9..].trim_start();

    let name_end = after_defmodule
        .chars()
        .position(|c| !c.is_alphanumeric() && c != '_')
        .unwrap_or(after_defmodule.len());

    if name_end == 0 {
        return Err(RuleEngineError::ParseError {
            message: "Missing module name".to_string(),
        });
    }

    let name = after_defmodule[..name_end].to_string();

    if !name
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
    {
        return Err(RuleEngineError::ParseError {
            message: "Module name must start with uppercase".to_string(),
        });
    }

    let rest = after_defmodule[name_end..].trim_start();
    if !rest.starts_with('{') {
        return Err(RuleEngineError::ParseError {
            message: "Expected '{' after module name".to_string(),
        });
    }

    let brace_pos = trimmed.len() - rest.len();
    let close_pos = literal_search::find_matching_brace(trimmed, brace_pos).ok_or_else(|| {
        RuleEngineError::ParseError {
            message: "Missing closing brace for module".to_string(),
        }
    })?;

    let body = trimmed[brace_pos + 1..close_pos].to_string();

    Ok((name, body, close_pos + 1))
}

/// Extract directive value
fn extract_directive(text: &str, directive: &str) -> Option<String> {
    let pos = text.find(directive)?;
    let after_directive = &text[pos + directive.len()..];

    let end = after_directive
        .find("import:")
        .or_else(|| after_directive.find("export:"))
        .unwrap_or(after_directive.len());

    Some(after_directive[..end].trim().to_string())
}

/// Extract module name from context
fn extract_module_from_context(grl_text: &str, rule_name: &str) -> String {
    let rule_patterns = [
        format!("rule \"{}\"", rule_name),
        format!("rule {}", rule_name),
    ];

    for pattern in &rule_patterns {
        if let Some(rule_pos) = grl_text.find(pattern) {
            let before = &grl_text[..rule_pos];
            if let Some(module_pos) = before.rfind(";; MODULE:") {
                let after_marker = &before[module_pos + 10..];
                if let Some(end_line) = after_marker.find('\n') {
                    let module_line = after_marker[..end_line].trim();
                    if let Some(first_word) = module_line.split_whitespace().next() {
                        return first_word.to_string();
                    }
                }
            }
        }
    }

    "MAIN".to_string()
}

// ============================================================================
// Condition Parsing
// ============================================================================

/// Parse the when clause into a ConditionGroup
fn parse_when_clause(when_clause: &str) -> Result<ConditionGroup> {
    let trimmed = when_clause.trim();

    // Strip outer parentheses if balanced
    let clause = strip_outer_parens(trimmed);

    // Parse OR (lowest precedence)
    if let Some(parts) = split_logical_operator(clause, "||") {
        return parse_or_parts(parts);
    }

    // Parse AND
    if let Some(parts) = split_logical_operator(clause, "&&") {
        return parse_and_parts(parts);
    }

    // Handle NOT
    if clause.trim_start().starts_with('!') {
        let inner = clause.trim_start()[1..].trim();
        let inner_condition = parse_when_clause(inner)?;
        return Ok(ConditionGroup::not(inner_condition));
    }

    // Handle EXISTS
    if clause.trim_start().starts_with("exists(") && clause.trim_end().ends_with(')') {
        let inner = &clause.trim()[7..clause.trim().len() - 1];
        let inner_condition = parse_when_clause(inner)?;
        return Ok(ConditionGroup::exists(inner_condition));
    }

    // Handle FORALL
    if clause.trim_start().starts_with("forall(") && clause.trim_end().ends_with(')') {
        let inner = &clause.trim()[7..clause.trim().len() - 1];
        let inner_condition = parse_when_clause(inner)?;
        return Ok(ConditionGroup::forall(inner_condition));
    }

    // Handle ACCUMULATE
    if clause.trim_start().starts_with("accumulate(") && clause.trim_end().ends_with(')') {
        return parse_accumulate_condition(clause);
    }

    // Handle TEST
    if clause.trim_start().starts_with("test(") && clause.trim_end().ends_with(')') {
        return parse_test_condition(clause);
    }

    // Single condition
    parse_single_condition(clause)
}

/// Strip outer parentheses if they are balanced
fn strip_outer_parens(text: &str) -> &str {
    let trimmed = text.trim();
    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        let inner = &trimmed[1..trimmed.len() - 1];
        if is_balanced_parens(inner) {
            return inner;
        }
    }
    trimmed
}

/// Check if parentheses are balanced
fn is_balanced_parens(text: &str) -> bool {
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

/// Split by logical operator at top level
fn split_logical_operator(clause: &str, operator: &str) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut paren_count = 0;
    let mut in_string = false;
    let mut chars = clause.chars().peekable();

    let op_chars: Vec<char> = operator.chars().collect();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                in_string = !in_string;
                current.push(ch);
            }
            '(' if !in_string => {
                paren_count += 1;
                current.push(ch);
            }
            ')' if !in_string => {
                paren_count -= 1;
                current.push(ch);
            }
            _ if !in_string && paren_count == 0 => {
                // Check for operator
                if op_chars.len() == 2 && ch == op_chars[0] && chars.peek() == Some(&op_chars[1]) {
                    chars.next();
                    parts.push(current.trim().to_string());
                    current.clear();
                    continue;
                }
                current.push(ch);
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    if parts.len() > 1 {
        Some(parts)
    } else {
        None
    }
}

/// Parse OR parts
fn parse_or_parts(parts: Vec<String>) -> Result<ConditionGroup> {
    let mut conditions = Vec::new();
    for part in parts {
        conditions.push(parse_when_clause(&part)?);
    }

    if conditions.is_empty() {
        return Err(RuleEngineError::ParseError {
            message: "No conditions in OR".to_string(),
        });
    }

    let mut iter = conditions.into_iter();
    let mut result = iter.next().unwrap();
    for condition in iter {
        result = ConditionGroup::or(result, condition);
    }

    Ok(result)
}

/// Parse AND parts
fn parse_and_parts(parts: Vec<String>) -> Result<ConditionGroup> {
    let mut conditions = Vec::new();
    for part in parts {
        conditions.push(parse_when_clause(&part)?);
    }

    if conditions.is_empty() {
        return Err(RuleEngineError::ParseError {
            message: "No conditions in AND".to_string(),
        });
    }

    let mut iter = conditions.into_iter();
    let mut result = iter.next().unwrap();
    for condition in iter {
        result = ConditionGroup::and(result, condition);
    }

    Ok(result)
}

/// Parse single condition like "User.Age >= 18"
fn parse_single_condition(clause: &str) -> Result<ConditionGroup> {
    let trimmed = strip_outer_parens(clause.trim());

    // Check for multifield patterns first
    if let Some(cond) = try_parse_multifield(trimmed)? {
        return Ok(ConditionGroup::single(cond));
    }

    // Check for function call pattern: func(args) op value
    if let Some(cond) = try_parse_function_call(trimmed)? {
        return Ok(ConditionGroup::single(cond));
    }

    // Parse standard condition: field op value
    let (field, op_str, value_str) = split_condition(trimmed)?;

    let operator = Operator::from_str(op_str).ok_or_else(|| RuleEngineError::InvalidOperator {
        operator: op_str.to_string(),
    })?;

    let value = parse_value(value_str)?;

    // Check if field contains arithmetic
    if contains_arithmetic(field) {
        let test_expr = format!("{} {} {}", field, op_str, value_str);
        let condition = Condition::with_test(test_expr, vec![]);
        return Ok(ConditionGroup::single(condition));
    }

    let condition = Condition::new(field.to_string(), operator, value);
    Ok(ConditionGroup::single(condition))
}

/// Try to parse multifield patterns
fn try_parse_multifield(clause: &str) -> Result<Option<Condition>> {
    // Pattern: field.array $?var (collect)
    if clause.contains(" $?") {
        let parts: Vec<&str> = clause.splitn(2, " $?").collect();
        if parts.len() == 2 {
            let field = parts[0].trim().to_string();
            let variable = format!("$?{}", parts[1].trim());
            return Ok(Some(Condition::with_multifield_collect(field, variable)));
        }
    }

    // Pattern: field.array count op value
    if clause.contains(" count ") {
        let count_pos = clause.find(" count ").unwrap();
        let field = clause[..count_pos].trim().to_string();
        let rest = clause[count_pos + 7..].trim();

        let (_, op_str, value_str) = split_condition_from_start(rest)?;
        let operator =
            Operator::from_str(op_str).ok_or_else(|| RuleEngineError::InvalidOperator {
                operator: op_str.to_string(),
            })?;
        let value = parse_value(value_str)?;

        return Ok(Some(Condition::with_multifield_count(
            field, operator, value,
        )));
    }

    // Pattern: field.array first [$var]
    if clause.contains(" first") {
        let first_pos = clause.find(" first").unwrap();
        let field = clause[..first_pos].trim().to_string();
        let rest = clause[first_pos + 6..].trim();
        let variable = if rest.starts_with('$') {
            Some(rest.split_whitespace().next().unwrap_or(rest).to_string())
        } else {
            None
        };
        return Ok(Some(Condition::with_multifield_first(field, variable)));
    }

    // Pattern: field.array last [$var]
    if clause.contains(" last") {
        let last_pos = clause.find(" last").unwrap();
        let field = clause[..last_pos].trim().to_string();
        let rest = clause[last_pos + 5..].trim();
        let variable = if rest.starts_with('$') {
            Some(rest.split_whitespace().next().unwrap_or(rest).to_string())
        } else {
            None
        };
        return Ok(Some(Condition::with_multifield_last(field, variable)));
    }

    // Pattern: field.array empty
    if let Some(stripped) = clause.strip_suffix(" empty") {
        let field = stripped.trim().to_string();
        return Ok(Some(Condition::with_multifield_empty(field)));
    }

    // Pattern: field.array not_empty
    if let Some(stripped) = clause.strip_suffix(" not_empty") {
        let field = stripped.trim().to_string();
        return Ok(Some(Condition::with_multifield_not_empty(field)));
    }

    Ok(None)
}

/// Try to parse function call condition
fn try_parse_function_call(clause: &str) -> Result<Option<Condition>> {
    // Look for pattern: identifier(args) operator value
    if let Some(paren_start) = clause.find('(') {
        if paren_start > 0 {
            let func_name = clause[..paren_start].trim();

            // Check it's a valid identifier
            if func_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                && func_name
                    .chars()
                    .next()
                    .map(|c| c.is_alphabetic())
                    .unwrap_or(false)
            {
                // Find matching close paren
                if let Some(paren_end) = find_matching_paren(clause, paren_start) {
                    let args_str = &clause[paren_start + 1..paren_end];
                    let after_paren = clause[paren_end + 1..].trim();

                    // Check if there's an operator after
                    if let Ok((_, op_str, value_str)) = split_condition_from_start(after_paren) {
                        let args: Vec<String> = if args_str.trim().is_empty() {
                            Vec::new()
                        } else {
                            args_str.split(',').map(|s| s.trim().to_string()).collect()
                        };

                        let operator = Operator::from_str(op_str).ok_or_else(|| {
                            RuleEngineError::InvalidOperator {
                                operator: op_str.to_string(),
                            }
                        })?;

                        let value = parse_value(value_str)?;

                        return Ok(Some(Condition::with_function(
                            func_name.to_string(),
                            args,
                            operator,
                            value,
                        )));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Find matching closing parenthesis
fn find_matching_paren(text: &str, open_pos: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 1;
    let mut i = open_pos + 1;
    let mut in_string = false;

    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_string = !in_string,
            b'(' if !in_string => depth += 1,
            b')' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }

    None
}

/// Split condition into field, operator, value
fn split_condition(clause: &str) -> Result<(&str, &str, &str)> {
    let operators = [">=", "<=", "==", "!=", ">", "<", "contains", "matches", "in"];

    for op in &operators {
        if let Some(op_pos) = find_operator(clause, op) {
            let field = clause[..op_pos].trim();
            let value = clause[op_pos + op.len()..].trim();
            return Ok((field, op, value));
        }
    }

    Err(RuleEngineError::ParseError {
        message: format!("Invalid condition format: {}", clause),
    })
}

/// Split condition starting from the beginning (for partial parsing)
fn split_condition_from_start(text: &str) -> Result<(&str, &str, &str)> {
    let operators = [">=", "<=", "==", "!=", ">", "<", "contains", "matches"];

    for op in &operators {
        if let Some(stripped) = text.strip_prefix(op) {
            let value = stripped.trim();
            return Ok(("", op, value));
        }
    }

    // Try to find operator in text
    split_condition(text)
}

/// Find operator position (not inside strings or brackets)
fn find_operator(text: &str, op: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let op_bytes = op.as_bytes();
    let mut in_string = false;
    let mut bracket_depth = 0;
    let mut i = 0;

    while i + op_bytes.len() <= bytes.len() {
        if bytes[i] == b'"' {
            in_string = !in_string;
            i += 1;
            continue;
        }

        if !in_string {
            if bytes[i] == b'[' {
                bracket_depth += 1;
            } else if bytes[i] == b']' {
                bracket_depth = bracket_depth.saturating_sub(1);
            }
        }

        if !in_string && bracket_depth == 0 && &bytes[i..i + op_bytes.len()] == op_bytes {
            // For keyword operators, check word boundaries
            if op.chars().next().unwrap().is_alphabetic() {
                let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
                let after_ok = i + op_bytes.len() >= bytes.len()
                    || !bytes[i + op_bytes.len()].is_ascii_alphanumeric();
                if before_ok && after_ok {
                    return Some(i);
                }
            } else {
                return Some(i);
            }
        }

        i += 1;
    }

    None
}

/// Check if string contains arithmetic operators
fn contains_arithmetic(s: &str) -> bool {
    s.contains('+') || s.contains('-') || s.contains('*') || s.contains('/') || s.contains('%')
}

/// Parse test condition
fn parse_test_condition(clause: &str) -> Result<ConditionGroup> {
    let trimmed = clause.trim();
    let inner = &trimmed[5..trimmed.len() - 1]; // Remove "test(" and ")"

    // Check if it's a function call: test(funcName(args))
    if let Some(paren_pos) = inner.find('(') {
        if let Some(close_paren) = find_matching_paren(inner, paren_pos) {
            let func_name = inner[..paren_pos].trim().to_string();
            let args_str = &inner[paren_pos + 1..close_paren];

            let args: Vec<String> = if args_str.trim().is_empty() {
                Vec::new()
            } else {
                args_str.split(',').map(|s| s.trim().to_string()).collect()
            };

            let condition = Condition::with_test(func_name, args);
            return Ok(ConditionGroup::single(condition));
        }
    }

    // Otherwise treat the whole thing as an expression
    let condition = Condition::with_test(inner.trim().to_string(), vec![]);
    Ok(ConditionGroup::single(condition))
}

/// Parse accumulate condition
fn parse_accumulate_condition(clause: &str) -> Result<ConditionGroup> {
    let trimmed = clause.trim();
    let inner = &trimmed[11..trimmed.len() - 1]; // Remove "accumulate(" and ")"

    // Split by comma at top level
    let parts = split_top_level_comma(inner)?;

    if parts.len() != 2 {
        return Err(RuleEngineError::ParseError {
            message: format!("Expected 2 parts in accumulate, got {}", parts.len()),
        });
    }

    let (source_pattern, extract_field, source_conditions) = parse_accumulate_pattern(&parts[0])?;
    let (function, function_arg) = parse_accumulate_function(&parts[1])?;

    Ok(ConditionGroup::accumulate(
        "$result".to_string(),
        source_pattern,
        extract_field,
        source_conditions,
        function,
        function_arg,
    ))
}

/// Split by comma at top level
fn split_top_level_comma(text: &str) -> Result<Vec<String>> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0;
    let mut in_string = false;

    for ch in text.chars() {
        match ch {
            '"' => {
                in_string = !in_string;
                current.push(ch);
            }
            '(' if !in_string => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' if !in_string => {
                paren_depth -= 1;
                current.push(ch);
            }
            ',' if !in_string && paren_depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    Ok(parts)
}

/// Parse accumulate pattern
fn parse_accumulate_pattern(pattern: &str) -> Result<(String, String, Vec<String>)> {
    let pattern = pattern.trim();

    let paren_pos = pattern
        .find('(')
        .ok_or_else(|| RuleEngineError::ParseError {
            message: format!("Missing '(' in accumulate pattern: {}", pattern),
        })?;

    let source_pattern = pattern[..paren_pos].trim().to_string();

    if !pattern.ends_with(')') {
        return Err(RuleEngineError::ParseError {
            message: format!("Missing ')' in accumulate pattern: {}", pattern),
        });
    }

    let inner = &pattern[paren_pos + 1..pattern.len() - 1];
    let parts = split_top_level_comma(inner)?;

    let mut extract_field = String::new();
    let mut source_conditions = Vec::new();

    for part in parts {
        let part = part.trim();

        if part.contains(':') && part.starts_with('$') {
            let colon_pos = part.find(':').unwrap();
            extract_field = part[colon_pos + 1..].trim().to_string();
        } else if part.contains("==")
            || part.contains("!=")
            || part.contains(">=")
            || part.contains("<=")
            || part.contains('>')
            || part.contains('<')
        {
            source_conditions.push(part.to_string());
        }
    }

    Ok((source_pattern, extract_field, source_conditions))
}

/// Parse accumulate function
fn parse_accumulate_function(func_str: &str) -> Result<(String, String)> {
    let func_str = func_str.trim();

    let paren_pos = func_str
        .find('(')
        .ok_or_else(|| RuleEngineError::ParseError {
            message: format!("Missing '(' in accumulate function: {}", func_str),
        })?;

    let function_name = func_str[..paren_pos].trim().to_string();

    if !func_str.ends_with(')') {
        return Err(RuleEngineError::ParseError {
            message: format!("Missing ')' in accumulate function: {}", func_str),
        });
    }

    let args = func_str[paren_pos + 1..func_str.len() - 1]
        .trim()
        .to_string();

    Ok((function_name, args))
}

// ============================================================================
// Value Parsing
// ============================================================================

/// Parse array literal: ["value1", "value2", 123, true]
fn parse_array_literal(array_str: &str) -> Result<Value> {
    let trimmed = array_str.trim();
    
    // Remove surrounding brackets
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Err(RuleEngineError::ParseError {
            message: format!("Invalid array literal: {}", array_str),
        });
    }
    
    let inner = &trimmed[1..trimmed.len() - 1].trim();
    
    // Empty array
    if inner.is_empty() {
        return Ok(Value::Array(Vec::new()));
    }
    
    // Split by comma at top level
    let elements = split_top_level_comma(inner)?;
    
    let mut array = Vec::new();
    for element in elements {
        let value = parse_value(element.trim())?;
        array.push(value);
    }
    
    Ok(Value::Array(array))
}

/// Parse a value string into a Value
fn parse_value(value_str: &str) -> Result<Value> {
    let trimmed = value_str.trim();

    // Array literal: ["value1", "value2", ...]
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        return parse_array_literal(trimmed);
    }

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

    // Integer
    if let Ok(int_val) = trimmed.parse::<i64>() {
        return Ok(Value::Integer(int_val));
    }

    // Float
    if let Ok(float_val) = trimmed.parse::<f64>() {
        return Ok(Value::Number(float_val));
    }

    // Expression (contains arithmetic or field reference)
    if is_expression(trimmed) {
        return Ok(Value::Expression(trimmed.to_string()));
    }

    // Field reference
    if trimmed.contains('.') {
        return Ok(Value::String(trimmed.to_string()));
    }

    // Variable/identifier
    if is_identifier(trimmed) {
        return Ok(Value::Expression(trimmed.to_string()));
    }

    // Default to string
    Ok(Value::String(trimmed.to_string()))
}

/// Check if string is a valid identifier
fn is_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let first = s.chars().next().unwrap();
    if !first.is_alphabetic() && first != '_' {
        return false;
    }

    s.chars().all(|c| c.is_alphanumeric() || c == '_')
}

/// Check if string is an expression
fn is_expression(s: &str) -> bool {
    let has_operator =
        s.contains('+') || s.contains('-') || s.contains('*') || s.contains('/') || s.contains('%');
    let has_field_ref = s.contains('.');
    let has_spaces = s.contains(' ');

    has_operator && (has_field_ref || has_spaces)
}

// ============================================================================
// Action Parsing
// ============================================================================

/// Parse the then clause into actions
fn parse_then_clause(then_clause: &str) -> Result<Vec<ActionType>> {
    let statements: Vec<&str> = then_clause
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut actions = Vec::new();

    for statement in statements {
        let action = parse_action_statement(statement)?;
        actions.push(action);
    }

    Ok(actions)
}

/// Parse a single action statement
fn parse_action_statement(statement: &str) -> Result<ActionType> {
    let trimmed = statement.trim();

    // Method call: $Object.method(args)
    if trimmed.starts_with('$') && trimmed.contains('.') {
        if let Some(action) = try_parse_method_call(trimmed)? {
            return Ok(action);
        }
    }

    // Compound assignment: field += value
    if let Some(pos) = trimmed.find("+=") {
        let field = trimmed[..pos].trim().to_string();
        let value_str = trimmed[pos + 2..].trim();
        let value = parse_value(value_str)?;
        return Ok(ActionType::Append { field, value });
    }

    // Assignment: field = value
    if let Some(eq_pos) = find_assignment_operator(trimmed) {
        let field = trimmed[..eq_pos].trim().to_string();
        let value_str = trimmed[eq_pos + 1..].trim();
        let value = parse_value(value_str)?;
        return Ok(ActionType::Set { field, value });
    }

    // Function call: funcName(args)
    if let Some(paren_pos) = trimmed.find('(') {
        if trimmed.ends_with(')') {
            let func_name = trimmed[..paren_pos].trim();
            let args_str = &trimmed[paren_pos + 1..trimmed.len() - 1];

            return parse_function_action(func_name, args_str);
        }
    }

    // Unknown statement
    Ok(ActionType::Custom {
        action_type: "statement".to_string(),
        params: {
            let mut params = HashMap::new();
            params.insert("statement".to_string(), Value::String(trimmed.to_string()));
            params
        },
    })
}

/// Find assignment operator (=) but not == or !=
fn find_assignment_operator(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut in_string = false;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'"' {
            in_string = !in_string;
            i += 1;
            continue;
        }

        if !in_string && bytes[i] == b'=' {
            // Check it's not == or !=
            let is_double = i + 1 < bytes.len() && bytes[i + 1] == b'=';
            let is_not_eq = i > 0 && bytes[i - 1] == b'!';
            let is_compound = i > 0
                && (bytes[i - 1] == b'+'
                    || bytes[i - 1] == b'-'
                    || bytes[i - 1] == b'*'
                    || bytes[i - 1] == b'/'
                    || bytes[i - 1] == b'%');

            if !is_double && !is_not_eq && !is_compound {
                return Some(i);
            }
        }

        i += 1;
    }

    None
}

/// Try to parse method call
fn try_parse_method_call(text: &str) -> Result<Option<ActionType>> {
    // Pattern: $Object.method(args)
    let dot_pos = match text.find('.') {
        Some(pos) => pos,
        None => return Ok(None),
    };
    let object = text[1..dot_pos].to_string(); // Skip $

    let rest = &text[dot_pos + 1..];
    let paren_pos = match rest.find('(') {
        Some(pos) => pos,
        None => return Ok(None),
    };
    let method = rest[..paren_pos].to_string();

    if !rest.ends_with(')') {
        return Ok(None);
    }

    let args_str = &rest[paren_pos + 1..rest.len() - 1];
    let args = parse_method_args(args_str)?;

    Ok(Some(ActionType::MethodCall {
        object,
        method,
        args,
    }))
}

/// Parse method arguments
fn parse_method_args(args_str: &str) -> Result<Vec<Value>> {
    if args_str.trim().is_empty() {
        return Ok(Vec::new());
    }

    let parts = split_top_level_comma(args_str)?;
    let mut args = Vec::new();

    for part in parts {
        let trimmed = part.trim();

        // Handle arithmetic expressions
        if contains_arithmetic(trimmed) {
            args.push(Value::String(trimmed.to_string()));
        } else {
            args.push(parse_value(trimmed)?);
        }
    }

    Ok(args)
}

/// Parse function-style action
fn parse_function_action(func_name: &str, args_str: &str) -> Result<ActionType> {
    match func_name.to_lowercase().as_str() {
        "retract" => {
            let object = args_str.trim().trim_start_matches('$').to_string();
            Ok(ActionType::Retract { object })
        }
        "log" => {
            let message = if args_str.is_empty() {
                "Log message".to_string()
            } else {
                let value = parse_value(args_str.trim())?;
                value.to_string()
            };
            Ok(ActionType::Log { message })
        }
        "activateagendagroup" | "activate_agenda_group" => {
            if args_str.is_empty() {
                return Err(RuleEngineError::ParseError {
                    message: "ActivateAgendaGroup requires agenda group name".to_string(),
                });
            }
            let value = parse_value(args_str.trim())?;
            let group = match value {
                Value::String(s) => s,
                _ => value.to_string(),
            };
            Ok(ActionType::ActivateAgendaGroup { group })
        }
        "schedulerule" | "schedule_rule" => {
            let parts = split_top_level_comma(args_str)?;
            if parts.len() != 2 {
                return Err(RuleEngineError::ParseError {
                    message: "ScheduleRule requires delay_ms and rule_name".to_string(),
                });
            }

            let delay_ms = parse_value(parts[0].trim())?;
            let rule_name = parse_value(parts[1].trim())?;

            let delay_ms = match delay_ms {
                Value::Integer(i) => i as u64,
                Value::Number(f) => f as u64,
                _ => {
                    return Err(RuleEngineError::ParseError {
                        message: "ScheduleRule delay_ms must be a number".to_string(),
                    })
                }
            };

            let rule_name = match rule_name {
                Value::String(s) => s,
                _ => rule_name.to_string(),
            };

            Ok(ActionType::ScheduleRule {
                delay_ms,
                rule_name,
            })
        }
        "completeworkflow" | "complete_workflow" => {
            if args_str.is_empty() {
                return Err(RuleEngineError::ParseError {
                    message: "CompleteWorkflow requires workflow_id".to_string(),
                });
            }
            let value = parse_value(args_str.trim())?;
            let workflow_name = match value {
                Value::String(s) => s,
                _ => value.to_string(),
            };
            Ok(ActionType::CompleteWorkflow { workflow_name })
        }
        "setworkflowdata" | "set_workflow_data" => {
            let data_str = args_str.trim();
            if let Some(eq_pos) = data_str.find('=') {
                let key = data_str[..eq_pos].trim().trim_matches('"').to_string();
                let value_str = data_str[eq_pos + 1..].trim();
                let value = parse_value(value_str)?;
                Ok(ActionType::SetWorkflowData { key, value })
            } else {
                Err(RuleEngineError::ParseError {
                    message: "SetWorkflowData data must be in key=value format".to_string(),
                })
            }
        }
        _ => {
            // Custom function
            let params = if args_str.is_empty() {
                HashMap::new()
            } else {
                let parts = split_top_level_comma(args_str)?;
                let mut params = HashMap::new();
                for (i, part) in parts.iter().enumerate() {
                    let value = parse_value(part.trim())?;
                    params.insert(i.to_string(), value);
                }
                params
            };

            Ok(ActionType::Custom {
                action_type: func_name.to_string(),
                params,
            })
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

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

        let rules = GRLParserNoRegex::parse_rules(grl).unwrap();
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

        let rules = GRLParserNoRegex::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "ComplexRule");
    }

    #[test]
    fn test_parse_no_loop_attribute() {
        let grl = r#"
        rule "NoLoopRule" no-loop salience 15 {
            when
                User.Score < 100
            then
                User.Score = 50;
        }
        "#;

        let rules = GRLParserNoRegex::parse_rules(grl).unwrap();
        assert!(rules[0].no_loop);
        assert_eq!(rules[0].salience, 15);
    }

    #[test]
    fn test_parse_or_condition() {
        let grl = r#"
        rule "OrRule" {
            when
                User.Status == "active" || User.Status == "premium"
            then
                User.Valid = true;
        }
        "#;

        let rules = GRLParserNoRegex::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);
    }

    #[test]
    fn test_parse_exists_pattern() {
        let grl = r#"
        rule "ExistsRule" {
            when
                exists(Customer.tier == "VIP")
            then
                System.premium = true;
        }
        "#;

        let rules = GRLParserNoRegex::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 1);

        match &rules[0].conditions {
            ConditionGroup::Exists(_) => {}
            _ => panic!("Expected EXISTS condition"),
        }
    }

    #[test]
    fn test_parse_multiple_rules() {
        let grl = r#"
        rule "Rule1" { when A > 1 then B = 2; }
        rule "Rule2" { when C < 3 then D = 4; }
        rule "Rule3" { when E == 5 then F = 6; }
        "#;

        let rules = GRLParserNoRegex::parse_rules(grl).unwrap();
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].name, "Rule1");
        assert_eq!(rules[1].name, "Rule2");
        assert_eq!(rules[2].name, "Rule3");
    }

    #[test]
    fn test_parse_assignment_action() {
        let grl = r#"
        rule "SetRule" {
            when
                X > 0
            then
                Y = 100;
                Z = "hello";
        }
        "#;

        let rules = GRLParserNoRegex::parse_rules(grl).unwrap();
        assert_eq!(rules[0].actions.len(), 2);

        match &rules[0].actions[0] {
            ActionType::Set { field, value } => {
                assert_eq!(field, "Y");
                assert_eq!(*value, Value::Integer(100));
            }
            _ => panic!("Expected Set action"),
        }
    }

    #[test]
    fn test_parse_append_action() {
        let grl = r#"
        rule "AppendRule" {
            when
                X > 0
            then
                Items += "new_item";
        }
        "#;

        let rules = GRLParserNoRegex::parse_rules(grl).unwrap();

        match &rules[0].actions[0] {
            ActionType::Append { field, value } => {
                assert_eq!(field, "Items");
                assert_eq!(*value, Value::String("new_item".to_string()));
            }
            _ => panic!("Expected Append action"),
        }
    }

    #[test]
    fn test_parse_in_operator() {
        let grl = r#"
        rule "TestInOperator" {
            when
                User.role in ["admin", "moderator", "vip"]
            then
                User.access = "granted";
        }
        "#;

        let result = GRLParserNoRegex::parse_rules(grl);
        match result {
            Ok(rules) => {
                assert_eq!(rules.len(), 1);
                assert_eq!(rules[0].name, "TestInOperator");
                
                // Check the condition
                match &rules[0].conditions {
                    ConditionGroup::Single(cond) => {
                        assert_eq!(cond.field, "User.role");
                        assert_eq!(cond.operator, crate::types::Operator::In);
                        // Value should be an array
                        match &cond.value {
                            Value::Array(arr) => {
                                assert_eq!(arr.len(), 3);
                            }
                            _ => panic!("Expected Array value, got {:?}", cond.value),
                        }
                    }
                    _ => panic!("Expected Single condition"),
                }
            }
            Err(e) => {
                panic!("Failed to parse 'in' operator: {}", e);
            }
        }
    }
}

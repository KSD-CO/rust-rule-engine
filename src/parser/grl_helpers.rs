/// GRL Parser helpers using literal search instead of regex
/// Provides fast parsing for GRL syntax without regex overhead
use super::literal_search;

/// Parse "rule Name" or "rule "Quoted Name"" and extract the name
pub fn parse_rule_header(text: &str) -> Option<(String, usize)> {
    let trimmed = text.trim_start();

    if !trimmed.starts_with("rule") {
        return None;
    }

    let skip = text.len() - trimmed.len(); // whitespace before "rule"
    let after_rule = trimmed[4..].trim_start();

    // Try quoted name first
    if after_rule.starts_with('"') {
        if let Some(end_quote) = memchr::memchr(b'"', &after_rule.as_bytes()[1..]) {
            let name = after_rule[1..end_quote + 1].to_string();
            let consumed = skip + 4 + (trimmed[4..].len() - after_rule.len()) + end_quote + 2;
            return Some((name, consumed));
        }
    }

    // Try identifier
    if let Some(ident) = literal_search::extract_identifier(after_rule) {
        let consumed = skip + 4 + (trimmed[4..].len() - after_rule.len()) + ident.len();
        return Some((ident, consumed));
    }

    None
}

/// Split GRL text into individual rules using literal "rule" and brace matching
pub fn split_into_rules(grl_text: &str) -> Vec<String> {
    let mut rules = Vec::new();
    let bytes = grl_text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        // Find "rule "
        if let Some(rule_pos) = memchr::memmem::find(&bytes[i..], b"rule ") {
            let abs_pos = i + rule_pos;

            // Find the opening brace
            if let Some(brace_pos) = memchr::memchr(b'{', &bytes[abs_pos..]) {
                let brace_abs = abs_pos + brace_pos;

                // Find matching closing brace
                if let Some(close_pos) = literal_search::find_matching_brace(grl_text, brace_abs) {
                    let rule_text = &grl_text[abs_pos..=close_pos];
                    rules.push(rule_text.to_string());
                    i = close_pos + 1;
                    continue;
                }
            }
        }
        i += 1;
    }

    rules
}

/// Parse "when ... then ..." and extract condition and action parts
pub fn parse_when_then(body: &str) -> Option<(String, String)> {
    let trimmed = body.trim();

    // Find "when"
    let when_pos = literal_search::find_literal(trimmed, "when")?;
    let after_when = trimmed[when_pos + 4..].trim_start();

    // Find "then" (need to be careful with nested structures)
    let then_pos = find_then_keyword(after_when)?;

    let condition = after_when[..then_pos].trim().to_string();
    let action = after_when[then_pos + 4..].trim().to_string();

    Some((condition, action))
}

/// Find "then" keyword at the correct nesting level
fn find_then_keyword(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut in_string = false;
    let mut escape_next = false;
    let mut paren_depth = 0;
    let mut brace_depth = 0;

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
            b')' if !in_string => paren_depth -= 1,
            b'{' if !in_string => brace_depth += 1,
            b'}' if !in_string => brace_depth -= 1,
            b't' if !in_string && paren_depth == 0 && brace_depth == 0 => {
                // Check if this is "then"
                if i + 4 <= bytes.len() && &bytes[i..i + 4] == b"then" {
                    // Make sure it's a word boundary
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

/// Extract salience value from attributes section
pub fn extract_salience(attributes: &str) -> Option<i32> {
    // Find "salience"
    let salience_pos = literal_search::find_literal(attributes, "salience")?;
    let after_salience = attributes[salience_pos + 8..].trim_start();

    // Extract digits
    let digits: String = after_salience
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();

    digits.parse().ok()
}

/// Parse defmodule declaration
pub fn parse_defmodule(text: &str) -> Option<(String, String, usize)> {
    let trimmed = text.trim_start();

    if !trimmed.starts_with("defmodule") {
        return None;
    }

    let after_defmodule = trimmed[9..].trim_start();

    // Extract module name (must start with uppercase)
    let name_end = after_defmodule
        .chars()
        .position(|c| !c.is_alphanumeric() && c != '_')?;

    let name = after_defmodule[..name_end].to_string();

    // Check if first char is uppercase
    if !name.chars().next()?.is_uppercase() {
        return None;
    }

    // Find opening brace
    let rest = after_defmodule[name_end..].trim_start();
    if !rest.starts_with('{') {
        return None;
    }

    let brace_pos = trimmed.len() - rest.len();

    // Find matching closing brace
    let close_pos = literal_search::find_matching_brace(trimmed, brace_pos)?;

    let body = trimmed[brace_pos + 1..close_pos].to_string();
    let consumed = close_pos + 1;

    Some((name, body, consumed))
}

/// Split defmodule declarations from rules
pub fn split_modules_and_rules(grl_text: &str) -> (Vec<String>, String) {
    let mut modules = Vec::new();
    let mut rules_text = String::new();
    let bytes = grl_text.as_bytes();
    let mut i = 0;
    let mut last_copy = 0;

    while i < bytes.len() {
        // Find "defmodule "
        if let Some(defmodule_pos) = memchr::memmem::find(&bytes[i..], b"defmodule ") {
            let abs_pos = i + defmodule_pos;

            // Copy text before defmodule to rules
            if abs_pos > last_copy {
                rules_text.push_str(&grl_text[last_copy..abs_pos]);
            }

            // Find the opening brace
            if let Some(brace_pos) = memchr::memchr(b'{', &bytes[abs_pos..]) {
                let brace_abs = abs_pos + brace_pos;

                // Find matching closing brace
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

    // Copy remaining text
    if last_copy < grl_text.len() {
        rules_text.push_str(&grl_text[last_copy..]);
    }

    (modules, rules_text)
}

/// Parse comparison operator from text
pub fn parse_operator(text: &str) -> Option<(&str, usize)> {
    let trimmed = text.trim_start();

    // Check two-character operators first
    if trimmed.len() >= 2 {
        match &trimmed[..2] {
            ">=" => return Some((">=", 2)),
            "<=" => return Some(("<=", 2)),
            "==" => return Some(("==", 2)),
            "!=" => return Some(("!=", 2)),
            _ => {}
        }
    }

    // Check single-character operators
    if let Some(first) = trimmed.chars().next() {
        match first {
            '>' => return Some((">", 1)),
            '<' => return Some(("<", 1)),
            _ => {}
        }
    }

    // Check keyword operators
    if trimmed.starts_with("contains") {
        return Some(("contains", 8));
    }
    if trimmed.starts_with("matches") {
        return Some(("matches", 7));
    }

    None
}

/// Check if text contains attribute keyword
pub fn has_attribute(text: &str, attr: &str) -> bool {
    // Use word boundary check
    let bytes = text.as_bytes();
    let attr_bytes = attr.as_bytes();

    if let Some(pos) = memchr::memmem::find(bytes, attr_bytes) {
        // Check word boundaries
        let before_ok = pos == 0 || !bytes[pos - 1].is_ascii_alphanumeric();
        let after_pos = pos + attr_bytes.len();
        let after_ok = after_pos >= bytes.len() || !bytes[after_pos].is_ascii_alphanumeric();

        return before_ok && after_ok;
    }

    false
}

/// Extract date-effective or date-expires value
pub fn extract_date_attribute(text: &str, attr_name: &str) -> Option<String> {
    let attr_pos = literal_search::find_literal(text, attr_name)?;
    let after_attr = text[attr_pos + attr_name.len()..].trim_start();

    // Expect format: "YYYY-MM-DD HH:MM:SS" or similar
    // Find the quoted string
    if after_attr.starts_with('"') {
        if let Some(end_quote) = memchr::memchr(b'"', &after_attr.as_bytes()[1..]) {
            return Some(after_attr[1..end_quote + 1].to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rule_header() {
        let (name, consumed) = parse_rule_header(r#"rule "MyRule" {"#).unwrap();
        assert_eq!(name, "MyRule");
        assert!(consumed > 0); // Just check it's reasonable

        let (name2, consumed2) = parse_rule_header("rule SimpleRule {").unwrap();
        assert_eq!(name2, "SimpleRule");
        assert!(consumed2 > 0);
    }

    #[test]
    fn test_parse_when_then() {
        let body = "when X > 5 then Y = 10";
        if let Some((cond, action)) = parse_when_then(body) {
            assert_eq!(cond, "X > 5");
            assert_eq!(action, "Y = 10");
        } else {
            panic!("Failed to parse when-then");
        }
    }

    #[test]
    fn test_extract_salience() {
        assert_eq!(extract_salience("salience 10"), Some(10));
        assert_eq!(extract_salience("salience  42  "), Some(42));
        assert_eq!(extract_salience("no salience here"), None);
    }

    #[test]
    fn test_parse_operator() {
        assert_eq!(parse_operator(">="), Some((">=", 2)));
        assert_eq!(parse_operator("  <= "), Some(("<=", 2)));
        assert_eq!(parse_operator("contains"), Some(("contains", 8)));
        assert_eq!(parse_operator("> 5"), Some((">", 1)));
    }

    #[test]
    fn test_has_attribute() {
        assert!(has_attribute("no-loop lock-on-active", "no-loop"));
        assert!(has_attribute("salience 10 no-loop", "no-loop"));
        assert!(!has_attribute("no-loops", "no-loop")); // Should not match partial
    }

    #[test]
    fn test_split_into_rules() {
        let grl = r#"
rule "Rule1" { when X > 5 then Y = 10 }
rule "Rule2" { when A < 3 then B = 7 }
        "#;
        let rules = split_into_rules(grl);
        assert_eq!(rules.len(), 2);
        assert!(rules[0].contains("Rule1"));
        assert!(rules[1].contains("Rule2"));
    }

    #[test]
    fn test_parse_defmodule() {
        let text = "defmodule MYMODULE { export: all }";
        if let Some((name, body, _)) = parse_defmodule(text) {
            assert_eq!(name, "MYMODULE");
            assert!(body.contains("export"));
        } else {
            panic!("Failed to parse defmodule");
        }
    }
}

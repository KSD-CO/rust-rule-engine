/// Literal search utilities using memchr and aho-corasick
/// Replaces regex for better performance on literal patterns
use aho_corasick::AhoCorasick;
use memchr::{memchr, memmem};

/// Extract text between two literal patterns
pub fn extract_between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_pos = memmem::find(text.as_bytes(), start.as_bytes())?;
    let search_from = start_pos + start.len();
    let end_pos = memmem::find(&text.as_bytes()[search_from..], end.as_bytes())?;
    Some(&text[search_from..search_from + end_pos])
}

/// Find the position of a literal string
pub fn find_literal(text: &str, pattern: &str) -> Option<usize> {
    memmem::find(text.as_bytes(), pattern.as_bytes())
}

/// Find all occurrences of multiple patterns
pub fn find_all_patterns(text: &str, patterns: &[&str]) -> Vec<(usize, usize, usize)> {
    let ac = AhoCorasick::new(patterns).unwrap();
    ac.find_iter(text)
        .map(|mat| (mat.pattern().as_usize(), mat.start(), mat.end()))
        .collect()
}

/// Check if text starts with a pattern (after trimming whitespace)
pub fn starts_with_trimmed(text: &str, pattern: &str) -> bool {
    text.trim_start().starts_with(pattern)
}

/// Extract quoted string (handles "text" format)
pub fn extract_quoted(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.starts_with('"') {
        if let Some(end_quote) = memchr(b'"', &trimmed.as_bytes()[1..]) {
            return Some(trimmed[1..end_quote + 1].to_string());
        }
    }
    None
}

/// Extract identifier (alphanumeric + underscore, starting with letter/underscore)
pub fn extract_identifier(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let mut chars = trimmed.chars();

    // First character must be letter or underscore
    let first = chars.next()?;
    if !first.is_alphabetic() && first != '_' {
        return None;
    }

    let mut result = String::new();
    result.push(first);

    // Subsequent characters can be alphanumeric or underscore
    for ch in chars {
        if ch.is_alphanumeric() || ch == '_' {
            result.push(ch);
        } else {
            break;
        }
    }

    Some(result)
}

/// Extract number from text
pub fn extract_number(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let mut result = String::new();
    let mut has_digit = false;

    for ch in trimmed.chars() {
        if ch.is_ascii_digit() {
            result.push(ch);
            has_digit = true;
        } else if ch == '.' && has_digit && !result.contains('.') {
            #[allow(clippy::if_same_then_else)]
            result.push(ch);
        } else if (ch == '-' || ch == '+') && result.is_empty() {
            result.push(ch);
        } else {
            break;
        }
    }

    if has_digit {
        Some(result)
    } else {
        None
    }
}

/// Parse rule name from "rule "Name" {" or "rule Name {"
pub fn parse_rule_name(text: &str) -> Option<String> {
    let text = text.trim();

    // Skip "rule " prefix
    if !text.starts_with("rule ") {
        return None;
    }

    let after_rule = text[5..].trim_start();

    // Try quoted name first
    if let Some(quoted) = extract_quoted(after_rule) {
        return Some(quoted);
    }

    // Try identifier
    extract_identifier(after_rule)
}

/// Find matching brace for a given opening brace position
pub fn find_matching_brace(text: &str, open_pos: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if open_pos >= bytes.len() || bytes[open_pos] != b'{' {
        return None;
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;

    #[allow(clippy::needless_range_loop)]
    for i in open_pos..bytes.len() {
        let ch = bytes[i];

        if escape_next {
            escape_next = false;
            continue;
        }

        match ch {
            b'\\' if in_string => escape_next = true,
            b'"' => in_string = !in_string,
            b'{' if !in_string => depth += 1,
            b'}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }

    None
}

/// Split text by a literal delimiter, respecting quotes and braces
pub fn split_respecting_structure(text: &str, delimiter: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut in_string = false;
    let mut escape_next = false;
    let mut brace_depth = 0;
    let bytes = text.as_bytes();
    let delim_bytes = delimiter.as_bytes();

    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i];

        if escape_next {
            current.push(ch as char);
            escape_next = false;
            i += 1;
            continue;
        }

        match ch {
            b'\\' if in_string => {
                current.push(ch as char);
                escape_next = true;
            }
            b'"' => {
                current.push(ch as char);
                in_string = !in_string;
            }
            b'{' if !in_string => {
                current.push(ch as char);
                brace_depth += 1;
            }
            b'}' if !in_string => {
                current.push(ch as char);
                brace_depth -= 1;
            }
            _ => {
                // Check for delimiter
                if !in_string
                    && brace_depth == 0
                    && i + delim_bytes.len() <= bytes.len()
                    && &bytes[i..i + delim_bytes.len()] == delim_bytes
                {
                    result.push(current.clone());
                    current.clear();
                    i += delim_bytes.len();
                    continue;
                }
                current.push(ch as char);
            }
        }
        i += 1;
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

/// Extract field path (e.g., "user.profile.name")
pub fn extract_field_path(text: &str) -> Option<String> {
    let trimmed = text.trim();
    let mut result = String::new();
    let mut chars = trimmed.chars().peekable();

    // First segment
    let first = chars.next()?;
    if !first.is_alphabetic() && first != '_' {
        return None;
    }
    result.push(first);

    loop {
        match chars.peek() {
            Some(&ch) if ch.is_alphanumeric() || ch == '_' => {
                result.push(ch);
                chars.next();
            }
            Some(&'.') => {
                result.push('.');
                chars.next();

                // Next segment must start with letter or underscore
                match chars.peek() {
                    Some(&ch) if ch.is_alphabetic() || ch == '_' => {
                        result.push(ch);
                        chars.next();
                    }
                    _ => break,
                }
            }
            _ => break,
        }
    }

    Some(result)
}

/// Simple email validation using literal checks
pub fn is_valid_email_literal(email: &str) -> bool {
    // Must contain exactly one @
    let at_count = email.bytes().filter(|&b| b == b'@').count();
    if at_count != 1 {
        return false;
    }

    let at_pos = memchr(b'@', email.as_bytes()).unwrap();

    // Local part must exist and be valid
    if at_pos == 0 {
        return false;
    }
    let local = &email[..at_pos];
    if !is_valid_email_local_part(local) {
        return false;
    }

    // Domain part must exist and be valid
    let domain = &email[at_pos + 1..];
    if domain.is_empty() {
        return false;
    }

    // Domain must have at least one dot
    if !domain.contains('.') {
        return false;
    }

    // Check TLD (last part after final dot)
    if let Some(last_dot) = domain.rfind('.') {
        let tld = &domain[last_dot + 1..];
        if tld.len() < 2 || !tld.chars().all(|c| c.is_alphabetic()) {
            return false;
        }
    } else {
        return false;
    }

    // Check domain part characters
    is_valid_email_domain(domain)
}

fn is_valid_email_local_part(local: &str) -> bool {
    for ch in local.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '_' | '%' | '+' | '-' => {}
            _ => return false,
        }
    }
    true
}

fn is_valid_email_domain(domain: &str) -> bool {
    for ch in domain.chars() {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' => {}
            _ => return false,
        }
    }
    // Domain cannot start or end with dot or hyphen
    !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.starts_with('-')
        && !domain.ends_with('-')
}

/// Simple phone number validation (digits, spaces, dashes, parentheses, plus)
pub fn is_valid_phone_literal(phone: &str) -> bool {
    let mut digit_count = 0;

    for ch in phone.chars() {
        match ch {
            '0'..='9' => digit_count += 1,
            ' ' | '-' | '(' | ')' | '+' => {}
            _ => return false,
        }
    }

    // Must have at least 7 digits
    digit_count >= 7
}

/// Simple URL validation
pub fn is_valid_url_literal(url: &str) -> bool {
    // Check for common schemes
    let schemes = ["http://", "https://", "ftp://", "ftps://"];
    let has_scheme = schemes.iter().any(|&scheme| url.starts_with(scheme));

    if !has_scheme {
        return false;
    }

    // Find scheme end
    let scheme_end = url.find("://").unwrap() + 3;
    let rest = &url[scheme_end..];

    // Must have at least a domain
    if rest.is_empty() {
        return false;
    }

    // Extract domain (before first / or end)
    let domain = if let Some(slash_pos) = rest.find('/') {
        &rest[..slash_pos]
    } else {
        rest
    };

    // Domain must contain at least one dot
    domain.contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_between() {
        let text = r#"rule "MyRule" { when X > 5 then Y = 10 }"#;
        assert_eq!(extract_between(text, "rule \"", "\""), Some("MyRule"));
        assert_eq!(
            extract_between(text, "{ ", " }"),
            Some("when X > 5 then Y = 10")
        );
    }

    #[test]
    fn test_parse_rule_name() {
        assert_eq!(
            parse_rule_name(r#"rule "MyRule" {"#),
            Some("MyRule".to_string())
        );
        assert_eq!(parse_rule_name("rule MyRule {"), Some("MyRule".to_string()));
    }

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email_literal("test@example.com"));
        assert!(is_valid_email_literal("user.name+tag@domain.co.uk"));
        assert!(!is_valid_email_literal("invalid@"));
        assert!(!is_valid_email_literal("@domain.com"));
        assert!(!is_valid_email_literal("no-at-sign.com"));
        assert!(!is_valid_email_literal("double@@at.com"));
    }

    #[test]
    fn test_phone_validation() {
        assert!(is_valid_phone_literal("+1-234-567-8900"));
        assert!(is_valid_phone_literal("(555) 123-4567"));
        assert!(is_valid_phone_literal("1234567"));
        assert!(!is_valid_phone_literal("123"));
        assert!(!is_valid_phone_literal("abc-defg"));
    }

    #[test]
    fn test_url_validation() {
        assert!(is_valid_url_literal("https://example.com"));
        assert!(is_valid_url_literal("http://www.example.com/path"));
        assert!(!is_valid_url_literal("not-a-url"));
        assert!(!is_valid_url_literal("http://"));
    }

    #[test]
    fn test_extract_field_path() {
        assert_eq!(
            extract_field_path("user.profile.name"),
            Some("user.profile.name".to_string())
        );
        assert_eq!(extract_field_path("simple"), Some("simple".to_string()));
        assert_eq!(
            extract_field_path("_private.field"),
            Some("_private.field".to_string())
        );
    }

    #[test]
    fn test_find_matching_brace() {
        let text = "rule { when { a } then { b } }";
        let open = text.find('{').unwrap();
        let close = find_matching_brace(text, open);
        assert_eq!(close, Some(text.len() - 1));
    }
}

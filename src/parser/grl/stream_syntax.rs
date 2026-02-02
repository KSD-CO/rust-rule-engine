// Stream syntax parser for GRL
//
// This module implements parsing for stream-related GRL syntax:
// - from stream("name")
// - over window(duration, type)
// - Stream patterns with variable bindings

use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, char, digit1, multispace0, multispace1},
    combinator::opt,
    sequence::delimited,
    IResult, Parser,
};
use std::time::Duration;

// Re-export WindowType from streaming module when available
#[cfg(feature = "streaming")]
pub use crate::streaming::window::WindowType;

// Fallback WindowType for when streaming feature is not enabled
#[cfg(not(feature = "streaming"))]
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum WindowType {
    Sliding,
    Tumbling,
    Session { timeout: Duration },
}

/// Stream source specification
#[derive(Debug, Clone, PartialEq)]
pub struct StreamSource {
    pub stream_name: String,
    pub window: Option<WindowSpec>,
}

/// Window specification
#[derive(Debug, Clone, PartialEq)]
pub struct WindowSpec {
    pub duration: Duration,
    pub window_type: WindowType,
}

/// Stream pattern with variable binding
#[derive(Debug, Clone, PartialEq)]
pub struct StreamPattern {
    pub var_name: String,
    pub event_type: Option<String>,
    pub source: StreamSource,
}

/// Stream join specification
#[derive(Debug, Clone, PartialEq)]
pub struct StreamJoinPattern {
    pub left: StreamPattern,
    pub right: StreamPattern,
    pub join_conditions: Vec<JoinCondition>,
}

/// Join condition between two streams
#[derive(Debug, Clone, PartialEq)]
pub enum JoinCondition {
    /// Equality condition: left.field == right.field
    Equality {
        left_field: String,
        right_field: String,
    },
    /// Custom expression condition
    Expression(String),
    /// Temporal constraint: right.time > left.time
    Temporal {
        operator: TemporalOp,
        left_field: String,
        right_field: String,
    },
}

/// Temporal operators for stream joins
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalOp {
    Before, // left.time < right.time
    After,  // left.time > right.time
    Within, // abs(left.time - right.time) < duration
}

/// Parse: from stream("stream-name")
///
/// # Example
/// ```text
/// from stream("user-events")
/// from stream("sensor-readings")
/// ```
pub fn parse_stream_source(input: &str) -> IResult<&str, StreamSource> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("from")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("stream")(input)?;
    let (input, _) = multispace0(input)?;

    // Parse stream name in parentheses: stream("name")
    let (input, stream_name) = delimited(
        (char('('), multispace0, char('"')),
        take_while1(|c: char| c != '"'),
        (char('"'), multispace0, char(')')),
    )
    .parse(input)?;

    // Optional window specification
    let (input, window) = opt(parse_window_spec).parse(input)?;

    Ok((
        input,
        StreamSource {
            stream_name: stream_name.to_string(),
            window,
        },
    ))
}

/// Parse: over window(5 min, sliding)
///
/// # Example
/// ```text
/// over window(5 min, sliding)
/// over window(1 hour, tumbling)
/// over window(30 seconds, sliding)
/// ```
pub fn parse_window_spec(input: &str) -> IResult<&str, WindowSpec> {
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("over")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("window")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char('(')(input)?;
    let (input, _) = multispace0(input)?;

    // Parse duration
    let (input, duration) = parse_duration(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = char(',')(input)?;
    let (input, _) = multispace0(input)?;

    // Parse window type
    let (input, window_type) = parse_window_type(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = char(')')(input)?;

    Ok((
        input,
        WindowSpec {
            duration,
            window_type,
        },
    ))
}

/// Parse duration: "5 min", "10 seconds", "1 hour", etc.
///
/// # Supported units
/// - ms, milliseconds, millisecond
/// - sec, second, seconds
/// - min, minute, minutes
/// - hour, hours
pub fn parse_duration(input: &str) -> IResult<&str, Duration> {
    let (input, value) = digit1(input)?;
    let (input, _) = multispace1(input)?;
    let (input, unit) = alpha1(input)?;

    let value: u64 = value.parse().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit))
    })?;

    let duration = match unit {
        "ms" | "milliseconds" | "millisecond" => Duration::from_millis(value),
        "sec" | "second" | "seconds" => Duration::from_secs(value),
        "min" | "minute" | "minutes" => Duration::from_secs(value * 60),
        "hour" | "hours" => Duration::from_secs(value * 3600),
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        }
    };

    Ok((input, duration))
}

/// Parse window type: "sliding" or "tumbling"
pub fn parse_window_type(input: &str) -> IResult<&str, WindowType> {
    let (input, type_str) = alpha1(input)?;

    let window_type = match type_str {
        "sliding" => WindowType::Sliding,
        "tumbling" => WindowType::Tumbling,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        }
    };

    Ok((input, window_type))
}

/// Parse complete stream pattern
///
/// # Example
/// ```text
/// event: EventType from stream("events") over window(5 min, sliding)
/// reading: TempReading from stream("sensors")
/// ```
pub fn parse_stream_pattern(input: &str) -> IResult<&str, StreamPattern> {
    // Parse variable binding: event:
    let (input, var_name) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = char(':')(input)?;
    let (input, _) = multispace0(input)?;

    // Optional event type (but not "from" keyword)
    let (input, event_type) = {
        let checkpoint = input;
        match take_while1::<_, _, nom::error::Error<&str>>(|c: char| {
            c.is_alphanumeric() || c == '_'
        })(input)
        {
            Ok((remaining, name)) if name != "from" => (remaining, Some(name)),
            _ => (checkpoint, None),
        }
    };

    let (input, _) = multispace0(input)?;

    // Parse stream source
    let (input, source) = parse_stream_source(input)?;

    Ok((
        input,
        StreamPattern {
            var_name: var_name.to_string(),
            event_type: event_type.map(|s| s.to_string()),
            source,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stream_source_basic() {
        let input = r#"from stream("user-events")"#;
        let result = parse_stream_source(input);

        assert!(result.is_ok());
        let (_, source) = result.unwrap();
        assert_eq!(source.stream_name, "user-events");
        assert!(source.window.is_none());
    }

    #[test]
    fn test_parse_stream_source_with_spaces() {
        let input = r#"  from   stream  (  "sensor-data"  )  "#;
        let result = parse_stream_source(input);

        assert!(result.is_ok());
        let (_, source) = result.unwrap();
        assert_eq!(source.stream_name, "sensor-data");
    }

    #[test]
    fn test_parse_duration_seconds() {
        let tests = vec![
            ("5 seconds", Duration::from_secs(5)),
            ("10 sec", Duration::from_secs(10)),
            ("1 second", Duration::from_secs(1)),
        ];

        for (input, expected) in tests {
            let result = parse_duration(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            let (_, duration) = result.unwrap();
            assert_eq!(duration, expected);
        }
    }

    #[test]
    fn test_parse_duration_minutes() {
        let tests = vec![
            ("5 min", Duration::from_secs(300)),
            ("10 minutes", Duration::from_secs(600)),
            ("1 minute", Duration::from_secs(60)),
        ];

        for (input, expected) in tests {
            let result = parse_duration(input);
            assert!(result.is_ok());
            let (_, duration) = result.unwrap();
            assert_eq!(duration, expected);
        }
    }

    #[test]
    fn test_parse_duration_hours() {
        let input = "1 hour";
        let result = parse_duration(input);

        assert!(result.is_ok());
        let (_, duration) = result.unwrap();
        assert_eq!(duration, Duration::from_secs(3600));
    }

    #[test]
    fn test_parse_duration_milliseconds() {
        let input = "500 ms";
        let result = parse_duration(input);

        assert!(result.is_ok());
        let (_, duration) = result.unwrap();
        assert_eq!(duration, Duration::from_millis(500));
    }

    #[test]
    fn test_parse_window_type() {
        let tests = vec![
            ("sliding", WindowType::Sliding),
            ("tumbling", WindowType::Tumbling),
        ];

        for (input, expected) in tests {
            let result = parse_window_type(input);
            assert!(result.is_ok());
            let (_, window_type) = result.unwrap();
            assert_eq!(window_type, expected);
        }
    }

    #[test]
    fn test_parse_window_spec() {
        let input = "over window(5 min, sliding)";
        let result = parse_window_spec(input);

        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert_eq!(spec.duration, Duration::from_secs(300));
        assert_eq!(spec.window_type, WindowType::Sliding);
    }

    #[test]
    fn test_parse_window_spec_tumbling() {
        let input = "over window(1 hour, tumbling)";
        let result = parse_window_spec(input);

        assert!(result.is_ok());
        let (_, spec) = result.unwrap();
        assert_eq!(spec.duration, Duration::from_secs(3600));
        assert_eq!(spec.window_type, WindowType::Tumbling);
    }

    #[test]
    fn test_parse_stream_pattern_simple() {
        let input = r#"event: LoginEvent from stream("logins")"#;
        let result = parse_stream_pattern(input);

        assert!(result.is_ok());
        let (_, pattern) = result.unwrap();
        assert_eq!(pattern.var_name, "event");
        assert_eq!(pattern.event_type, Some("LoginEvent".to_string()));
        assert_eq!(pattern.source.stream_name, "logins");
        assert!(pattern.source.window.is_none());
    }

    #[test]
    fn test_parse_stream_pattern_with_window() {
        let input = r#"reading: TempReading from stream("sensors") over window(10 min, sliding)"#;
        let result = parse_stream_pattern(input);

        assert!(result.is_ok());
        let (_, pattern) = result.unwrap();
        assert_eq!(pattern.var_name, "reading");
        assert_eq!(pattern.event_type, Some("TempReading".to_string()));
        assert_eq!(pattern.source.stream_name, "sensors");
        assert!(pattern.source.window.is_some());

        let window = pattern.source.window.unwrap();
        assert_eq!(window.duration, Duration::from_secs(600));
        assert_eq!(window.window_type, WindowType::Sliding);
    }

    #[test]
    fn test_parse_stream_pattern_no_type() {
        let input = r#"e: from stream("events")"#;
        let result = parse_stream_pattern(input);

        assert!(result.is_ok());
        let (_, pattern) = result.unwrap();
        assert_eq!(pattern.var_name, "e");
        assert_eq!(pattern.event_type, None);
    }

    #[test]
    fn test_invalid_window_type() {
        let input = "over window(5 min, invalid)";
        let result = parse_window_spec(input);

        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_duration_unit() {
        let input = "5 invalid_unit";
        let result = parse_duration(input);

        assert!(result.is_err());
    }
}

/// Parse stream join pattern with && operator
///
/// # Example
/// ```text
/// click: ClickEvent from stream("clicks") over window(10 min, sliding) &&
/// purchase: PurchaseEvent from stream("purchases") over window(10 min, sliding)
/// ```
pub fn parse_stream_join_pattern(input: &str) -> IResult<&str, StreamJoinPattern> {
    use nom::bytes::complete::tag;

    // Parse left stream pattern
    let (input, left) = parse_stream_pattern(input)?;

    // Expect && operator
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("&&")(input)?;
    let (input, _) = multispace0(input)?;

    // Parse right stream pattern
    let (input, right) = parse_stream_pattern(input)?;

    // Join conditions will be parsed separately from additional && clauses
    // For now, return empty conditions (to be filled by caller)
    Ok((
        input,
        StreamJoinPattern {
            left,
            right,
            join_conditions: Vec::new(),
        },
    ))
}

/// Parse a join condition like "click.user_id == purchase.user_id"
///
/// # Example
/// ```text
/// click.user_id == purchase.user_id
/// purchase.timestamp > click.timestamp
/// ```
pub fn parse_join_condition(input: &str) -> IResult<&str, JoinCondition> {
    use nom::branch::alt;
    use nom::bytes::complete::tag;

    // Parse left side: variable.field
    let (input, left_var) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, _) = char('.')(input)?;
    let (input, left_field) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;

    let (input, _) = multispace0(input)?;

    // Parse operator
    let (input, op) = alt((
        tag("=="),
        tag("!="),
        tag("<="),
        tag(">="),
        tag("<"),
        tag(">"),
    ))
    .parse(input)?;

    let (input, _) = multispace0(input)?;

    // Parse right side: variable.field
    let (input, right_var) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;
    let (input, _) = char('.')(input)?;
    let (input, right_field) = take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)?;

    // Construct condition based on operator
    let condition = match op {
        "==" => JoinCondition::Equality {
            left_field: format!("{}.{}", left_var, left_field),
            right_field: format!("{}.{}", right_var, right_field),
        },
        ">" => {
            if left_field.contains("time") || right_field.contains("time") {
                JoinCondition::Temporal {
                    operator: TemporalOp::After,
                    left_field: format!("{}.{}", left_var, left_field),
                    right_field: format!("{}.{}", right_var, right_field),
                }
            } else {
                JoinCondition::Expression(format!(
                    "{}.{} > {}.{}",
                    left_var, left_field, right_var, right_field
                ))
            }
        }
        "<" => {
            if left_field.contains("time") || right_field.contains("time") {
                JoinCondition::Temporal {
                    operator: TemporalOp::Before,
                    left_field: format!("{}.{}", left_var, left_field),
                    right_field: format!("{}.{}", right_var, right_field),
                }
            } else {
                JoinCondition::Expression(format!(
                    "{}.{} < {}.{}",
                    left_var, left_field, right_var, right_field
                ))
            }
        }
        _ => JoinCondition::Expression(format!(
            "{}.{} {} {}.{}",
            left_var, left_field, op, right_var, right_field
        )),
    };

    Ok((input, condition))
}

#[cfg(test)]
mod join_tests {
    use super::*;

    #[test]
    fn test_parse_join_condition_equality() {
        let input = "click.user_id == purchase.user_id";
        let result = parse_join_condition(input);

        assert!(result.is_ok());
        let (_, condition) = result.unwrap();
        match condition {
            JoinCondition::Equality {
                left_field,
                right_field,
            } => {
                assert_eq!(left_field, "click.user_id");
                assert_eq!(right_field, "purchase.user_id");
            }
            _ => panic!("Expected Equality condition"),
        }
    }

    #[test]
    fn test_parse_join_condition_temporal() {
        let input = "purchase.timestamp > click.timestamp";
        let result = parse_join_condition(input);

        assert!(result.is_ok());
        let (_, condition) = result.unwrap();
        match condition {
            JoinCondition::Temporal {
                operator,
                left_field,
                right_field,
            } => {
                assert_eq!(operator, TemporalOp::After);
                assert_eq!(left_field, "purchase.timestamp");
                assert_eq!(right_field, "click.timestamp");
            }
            _ => panic!("Expected Temporal condition"),
        }
    }

    #[test]
    fn test_parse_stream_join_pattern() {
        let input = r#"click: ClickEvent from stream("clicks") over window(10 min, sliding) && purchase: PurchaseEvent from stream("purchases") over window(10 min, sliding)"#;
        let result = parse_stream_join_pattern(input);

        assert!(result.is_ok());
        let (_, join_pattern) = result.unwrap();

        assert_eq!(join_pattern.left.var_name, "click");
        assert_eq!(join_pattern.left.event_type, Some("ClickEvent".to_string()));
        assert_eq!(join_pattern.left.source.stream_name, "clicks");

        assert_eq!(join_pattern.right.var_name, "purchase");
        assert_eq!(
            join_pattern.right.event_type,
            Some("PurchaseEvent".to_string())
        );
        assert_eq!(join_pattern.right.source.stream_name, "purchases");
    }
}

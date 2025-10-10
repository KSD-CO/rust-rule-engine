# Rule Examples Directory

This directory contains various GRL (Grule Rule Language) examples demonstrating different rule patterns and complexity levels.

## Files:

### `test_complex_rule.grl`
- **Purpose**: Test complex nested conditions with multiple logical operators
- **Features**: Deep parentheses nesting, OR/AND combinations
- **Use case**: Complex business rule validation

### `simple_business_rules.grl`  
- **Purpose**: Basic business rules for e-commerce
- **Features**: Simple conditions, basic actions
- **Use case**: Getting started with rule engine

### `advanced_nested_rules.grl`
- **Purpose**: Advanced rules with complex nested logic
- **Features**: Multiple condition groups, various operators
- **Use case**: Premium customer processing, seasonal campaigns

### `legacy_format_rules.grl`
- **Purpose**: Backward compatibility testing
- **Features**: PascalCase field names, traditional syntax
- **Use case**: Migration from older rule formats

## Usage:

These files can be used with the parser examples:

```bash
cargo run --example test_complex_parsing
```

Or loaded programmatically:

```rust
use rust_rule_engine::parser::grl::GRLParser;
use std::fs;

let content = fs::read_to_string("examples/rules/simple_business_rules.grl")?;
let rules = GRLParser::parse_rules(&content)?;
```

## Testing:

All rule files are tested for proper parsing and can be used to verify rule engine functionality.

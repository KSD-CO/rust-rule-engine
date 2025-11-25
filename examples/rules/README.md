# GRL Rule Files

Collection of GRL (Grule Rule Language) rule files organized by functionality.

## Directory Structure

```
rules/
├── 01-basic/          # Basic rules for beginners (6 files)
├── 02-rete/           # RETE-optimized rules (3 files)
├── 03-advanced/       # Advanced features (7 files)
└── 04-use-cases/      # Real-world use cases (5 files)
```

**Total: 21 files** (reduced from 30 files - deleted 9 duplicate/test/legacy files)

## GRL Language Overview

GRL (Grule Rule Language) is a DSL for defining business rules with syntax close to natural language:

```grl
rule "RuleName" "Description" salience 10 {
    when
        // Conditions
        Person.Age >= 18 && Person.VIP == true
    then
        // Actions
        Person.Discount = 20;
        Log("VIP customer gets 20% discount");
}
```

## Quick Start

### 1. Basic Rules (Start here)

```bash
# View the simplest rule
cat 01-basic/simple_business_rules.grl

# Run with example
cargo run --example grule_demo
```

### 2. RETE-Optimized Rules

```bash
# Rules optimized for RETE engine
cat 02-rete/rete_demo.grl

cargo run --example rete_grl_demo
```

### 3. Advanced Features

```bash
# Advanced pattern matching
cat 03-advanced/pattern_matching.grl

# Conflict resolution
cat 03-advanced/conflict_resolution_rules.grl
```

### 4. Real-World Use Cases

```bash
# Fraud detection system
cat 04-use-cases/fraud_detection.grl

# Sales analytics
cat 04-use-cases/sales_analytics.grl
```

## Main Topics

### Basics (01-basic/)
- ✅ Simple conditions and actions
- ✅ Method calls
- ✅ Arithmetic expressions
- ✅ Basic business rules

### RETE Features (02-rete/)
- ✅ Conditional Elements (exists, forall, not)
- ✅ Multifield patterns
- ✅ RETE-optimized structures

### Advanced Features (03-advanced/)
- ✅ Advanced pattern matching
- ✅ Complex boolean expressions
- ✅ Conflict resolution strategies
- ✅ Action handlers
- ✅ Fact retraction
- ✅ Truth Maintenance System (TMS)

### Use Cases (04-use-cases/)
- ✅ Purchasing workflows
- ✅ Fraud detection
- ✅ Sales analytics
- ✅ Automotive systems
- ✅ Performance optimization

## GRL Syntax Elements

### Rule Structure
```grl
rule "RuleName" "Optional description" {
    when
        // Conditions
    then
        // Actions
}
```

### Rule Attributes
- **salience**: Priority (higher = execute first)
- **no-loop**: Prevent infinite loops
- **agenda-group**: Group rules
- **activation-group**: Only one rule in group fires
- **date-effective/expires**: Time-based activation

### Conditional Elements
- **exists**: At least one fact matches
- **forall**: All facts match condition
- **not/!exists**: No facts match
- **&&, ||, !**: Boolean operators

### Actions
- Assignment: `Fact.Field = value`
- Method calls: `Fact.Method(args)`
- Logging: `Log("message")`
- Retraction: `Retract("FactName")`

## Integration with Rust Code

### Native Engine
```rust
use rust_rule_engine::RuleEngineBuilder;

let engine = RuleEngineBuilder::new()
    .add_grl_file("rules/01-basic/grule_demo.grl")?
    .build()?;
```

### RETE-UL Engine
```rust
use rust_rule_engine::rete::{IncrementalEngine, GrlReteLoader};

let mut engine = IncrementalEngine::new();
GrlReteLoader::load_from_file(
    "rules/02-rete/rete_demo.grl",
    &mut engine
)?;
```

### Load from string
```rust
let grl_content = std::fs::read_to_string("rules/fraud_detection.grl")?;
engine.add_grl_from_string(&grl_content)?;
```

## Best Practices

### 1. Naming Conventions
- Rule names: CamelCase or snake_case
- Descriptive names: "CheckVIPDiscount" > "Rule1"
- Add descriptions for complex rules

### 2. Rule Organization
- Group related rules
- Use salience to control execution order
- Comment complex conditions

### 3. Performance
- Minimize redundant conditions
- Use RETE engine for > 100 rules
- Avoid complex calculations in when clause

### 4. Maintainability
- Keep rules simple and focused
- One responsibility per rule
- Use meaningful variable names

## Migration from Deleted Files

If you need functionality from deleted files:

### Accumulate features
See: `03-advanced/pattern_matching.grl` and refer to Rust examples

### No-loop testing
See: Rule attributes in `03-advanced/conflict_resolution_rules.grl`

### Test CE features
See: `02-rete/test_ce_rules.grl`

### Legacy format
No longer supported, please use new format

## Additional Documentation

- Each subdirectory has its own README.md with details
- See corresponding Rust examples in `examples/`
- GRL syntax reference in documentation

## Statistics

| Category | Files | Complexity | Best For |
|----------|-------|------------|----------|
| 01-basic | 6 | Low | Learning GRL syntax |
| 02-rete | 3 | Medium-High | RETE engine optimization |
| 03-advanced | 7 | High | Complex rule patterns |
| 04-use-cases | 5 | Very High | Production-ready examples |

**Deleted 9 files:**
- 3 duplicates (with_no_loop_test, test_ce_rete_rules, generic_method_calls)
- 4 test files (no_loop_test, accumulate_test, rule_attributes_test, test_complex_rule)
- 2 legacy files (legacy_format_rules, function_calls)

## Contributing

When adding a new GRL file:
1. Choose the appropriate directory
2. Give the file a descriptive name (e.g., `inventory_management.grl`)
3. Add comments explaining the rules
4. Update README.md
5. Create a corresponding Rust example if needed

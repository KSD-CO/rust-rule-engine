# Rule Examples Directory

This directory contains various GRL (Grule Rule Language) examples demonstrating different rule patterns, complexity levels, and advanced features.

## Files:

### Basic Examples

#### `simple_business_rules.grl`
- **Purpose**: Basic business rules for e-commerce
- **Features**: Simple conditions, basic actions
- **Use case**: Getting started with rule engine
- **Example**: Run with `cargo run --example test_complex_parsing`

#### `test_complex_rule.grl`
- **Purpose**: Test complex nested conditions with multiple logical operators
- **Features**: Deep parentheses nesting, OR/AND combinations
- **Use case**: Complex business rule validation

#### `advanced_nested_rules.grl`
- **Purpose**: Advanced rules with complex nested logic
- **Features**: Multiple condition groups, various operators
- **Use case**: Premium customer processing, seasonal campaigns

#### `legacy_format_rules.grl`
- **Purpose**: Backward compatibility testing
- **Features**: PascalCase field names, traditional syntax
- **Use case**: Migration from older rule formats

---

### Advanced Features (v0.12.0+)

#### `test_ce_rules.grl` *(NEW in v0.12.0)*
- **Purpose**: Demonstrate Test CE (Conditional Element) with arbitrary boolean expressions
- **Features**:
  - `test()` syntax for calling arbitrary functions in WHEN clause
  - Function calls without operators: `test(isValidEmail(User.email))`
  - Negation support: `!test(hasActiveSubscription(User.id))`
  - Mixed with regular conditions: `User.age > 18 && test(checkCredit(User.id))`
- **Example**: Run with `cargo run --example test_ce_comprehensive`
- **Use case**: AI/ML integration, external API calls, custom validation functions

#### `test_ce_rete_rules.grl` *(NEW in v0.12.0)*
- **Purpose**: Test CE examples specifically for RETE-UL Engine
- **Features**:
  - RETE-compatible test() syntax
  - Variable binding with test CE: `$var`
  - Cross-pattern variable references
  - Optimized for RETE pattern matching
- **Example**: Run with `cargo run --example test_ce_rete_demo`
- **Use case**: High-performance test CE with RETE engine (50+ rules)

#### `conflict_resolution_rules.grl` *(NEW in v0.13.0)*
- **Purpose**: Demonstrate CLIPS/Drools-inspired conflict resolution strategies
- **Features**:
  - **Salience levels**: Priority-based rule execution (5 to 100)
  - **Rule complexity**: 1 to 5 conditions per rule
  - **Business scenarios**: Fraud detection, approvals, VIP discounts, risk assessment
  - **7 diverse rules**: FraudDetection, HighValueApproval, VIPDiscount, RiskAssessment, etc.
- **Strategies tested**:
  - **Salience**: Rules fire by priority (100 → 50 → 40 → ...)
  - **Complexity**: More conditions fire first (5 conds → 3 conds → 1 cond)
  - **Simplicity**: Fewer conditions fire first (1 cond → 3 conds → 5 conds)
  - **LEX/MEA**: Recency and specificity-based (engine-level strategy)
- **Example**: Run with `cargo run --example conflict_resolution_demo`
- **Use case**: Enterprise rule ordering, mission-critical rule prioritization

---

## Usage:

### Native Engine (Simple API)

```rust
use rust_rule_engine::parser::grl::GRLParser;
use std::fs;

// Load and parse rules
let content = fs::read_to_string("examples/rules/simple_business_rules.grl")?;
let rules = GRLParser::parse_rules(&content)?;

// Execute with Native Engine
let mut engine = RustRuleEngine::new();
engine.load_rules_from_file("examples/rules/simple_business_rules.grl")?;
```

### RETE-UL Engine (High Performance)

```rust
use rust_rule_engine::rete::{IncrementalEngine, GrlReteLoader};

// Load rules into RETE engine
let mut engine = IncrementalEngine::new();
GrlReteLoader::load_from_file(
    "examples/rules/conflict_resolution_rules.grl",
    &mut engine
)?;

// Set conflict resolution strategy
engine.set_conflict_resolution_strategy(
    ConflictResolutionStrategy::Salience
);

// Fire rules
engine.reset();
let fired = engine.fire_all();
```

### Test CE with Function Registry

```rust
use rust_rule_engine::rete::{IncrementalEngine, GrlReteLoader};

let mut engine = IncrementalEngine::new();

// Register custom test functions
engine.register_test_function("isValidEmail", |args| {
    // Your validation logic
    Ok(FactValue::Bool(true))
});

// Load rules with test CE
GrlReteLoader::load_from_file(
    "examples/rules/test_ce_rete_rules.grl",
    &mut engine
)?;
```

---

## Running Examples:

### Basic Parsing
```bash
cargo run --example test_complex_parsing
```

### Test CE (v0.12.0)
```bash
# Native Engine with Test CE
cargo run --example test_ce_comprehensive

# RETE Engine with Test CE
cargo run --example test_ce_rete_demo
```

### Conflict Resolution (v0.13.0)
```bash
# Full demo with 8 strategies
cargo run --example conflict_resolution_demo

# Shows different firing orders:
# - Salience: Priority-based (100 → 50 → 40...)
# - LEX: Most recent facts first
# - Complexity: More conditions first
# - Simplicity: Fewer conditions first
```

---

## Testing:

All rule files are tested for proper parsing and can be used to verify rule engine functionality:

```bash
# Run all tests
cargo test

# Test specific features
cargo test test_ce
cargo test conflict_resolution
cargo test grl_loader
```

---

## Documentation:

For more information about these features:
- **Test CE**: See [CLIPS_INSPIRED_FEATURES.md](../../CLIPS_INSPIRED_FEATURES.md)
- **Conflict Resolution**: See [examples/conflict_resolution_demo.rs](../conflict_resolution_demo.rs)
- **RETE Engine**: See [docs/RETE_GUIDE.md](../../docs/RETE_GUIDE.md)
- **GRL Syntax**: See [docs/GRL_SYNTAX.md](../../docs/GRL_SYNTAX.md)

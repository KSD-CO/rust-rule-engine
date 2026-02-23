# Agent Guidelines for rust-rule-engine

This document provides guidelines for AI coding agents working on the `rust-rule-engine` codebase.

## Project Overview

A production-ready Rust rule engine (v1.16.1) featuring:
- RETE-UL algorithm for forward chaining
- Backward chaining inference (goal-driven reasoning)
- GRL (Grule Rule Language) parser
- Streaming event processing with Redis state backend
- Plugin system with 44+ actions and 33+ functions
- Truth Maintenance System (TMS)

**Repository:** https://github.com/KSD-CO/rust-rule-engine
**License:** MIT
**Rust Edition:** 2021

## Build Commands

### Standard Build & Test
```bash
cargo build --verbose --all-features          # Build with all features
cargo test --verbose --all-features           # Run all tests
cargo clippy --all-targets --all-features -- -D warnings  # Lint
cargo fmt                                     # Format code
```

### Makefile Shortcuts
```bash
make ci                # Full CI pipeline (fmt-check, clippy, build, test, test-features, doc-test)
make check             # Quick check (fmt, clippy, test)
make test              # Run all tests with all features
make clippy            # Run clippy with -D warnings
make fmt               # Format code
make fmt-check         # Check formatting without modifying
make build             # Build project
make doc-test          # Run documentation tests
make test-features     # Test all feature combinations
```

### Running a Single Test
```bash
# Run a specific test by name
cargo test test_name --all-features

# Run tests in a specific file
cargo test --test grl_harness --all-features

# Run a single test with output
cargo test test_name --all-features -- --nocapture

# Run tests matching a pattern
cargo test backward --all-features

# Run doc tests only
cargo test --doc
```

### Feature Flags
```bash
# Test with specific features
cargo test --no-default-features --lib                          # No features
cargo test --features backward-chaining --lib                   # Backward chaining only
cargo test --features streaming --lib                           # Streaming only
cargo test --features "backward-chaining,streaming" --lib       # Multiple features
```

### Examples
```bash
# Run all examples (26 total)
make all

# Run by category
make getting-started        # 4 examples
make rete-engine           # 5 examples
make advanced-features     # 6 examples
make performance           # 3 examples (runs in --release mode)
make backward-chaining     # 4 examples (requires backward-chaining feature)
make module-system         # 2 examples

# Run individual example
cargo run --example grule_demo
cargo run --features backward-chaining --example simple_query_demo
cargo run --release --example quick_engine_comparison
```

### Benchmarks
```bash
cargo bench                                      # Run all benchmarks
cargo bench --bench engine_comparison_benchmark  # Specific benchmark
```

## Code Style Guidelines

### General Conventions

**Rust Edition:** 2021 with standard Rust formatting (no custom rustfmt.toml)

**Linting:**
- Clippy with `-D warnings` (all warnings are errors in CI)
- `#![warn(clippy::all)]` in lib.rs
- Use `#[allow(clippy::lint_name)]` sparingly and only when justified

**Documentation:**
- Use `///` for public API documentation
- Use `//!` for module-level documentation
- Include examples in doc comments where helpful
- Missing docs warning is currently disabled but documentation is encouraged

### Import Style

**Order (as used in the codebase):**
1. Crate-level imports (`crate::` or `super::`)
2. External crate imports (chrono, regex, log, etc.)
3. Standard library imports (`std::`)

**Pattern:**
```rust
// 1. Crate-level imports first
use crate::engine::{
    agenda::AgendaManager,
    facts::Facts,
    knowledge_base::KnowledgeBase,
};
use crate::errors::{Result, RuleEngineError};
use crate::types::{ActionType, Value};

// 2. External crate imports
use chrono::{DateTime, Utc};
use log::info;

// 3. Standard library imports
use std::collections::HashMap;
use std::time::{Duration, Instant};
```

**Note:** Use blank lines to separate the three groups for clarity.

### Naming Conventions

**Types:** PascalCase
```rust
struct RustRuleEngine { }
enum Value { }
type CustomFunction = Box<dyn Fn(...) -> Result<Value>>;
```

**Functions/Methods:** snake_case
```rust
pub fn execute_with_callback(&mut self) -> Result<GruleExecutionResult>
pub fn to_string(&self) -> String
```

**Constants:** SCREAMING_SNAKE_CASE
```rust
const MAX_CYCLES: usize = 100;
```

**Modules:** snake_case (single word preferred)
```rust
pub mod engine;
pub mod backward;
pub mod streaming;
```

### Type Usage

**Prefer explicit types for public APIs:**
```rust
pub fn new(name: &str) -> Self
pub fn execute(&mut self, facts: &Facts) -> Result<GruleExecutionResult>
```

**Use type aliases for complex types:**
```rust
pub type Result<T> = std::result::Result<T, RuleEngineError>;
pub type CustomFunction = Box<dyn Fn(&[Value], &Facts) -> Result<Value> + Send + Sync>;
```

**Prefer owned types in structs, borrowed in function parameters:**
```rust
pub struct Rule {
    pub name: String,              // Owned
    pub conditions: Vec<Condition>, // Owned
}

pub fn add_rule(&mut self, rule: Rule) -> Result<()>  // Owned
pub fn evaluate(&self, facts: &Facts) -> Result<bool> // Borrowed
```

### Error Handling

**Use `thiserror` for error definitions:**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RuleEngineError {
    #[error("Parse error: {message}")]
    ParseError { message: String },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
}
```

**Use `Result<T>` type alias:**
```rust
pub type Result<T> = std::result::Result<T, RuleEngineError>;

pub fn parse_rules(input: &str) -> Result<Vec<Rule>> {
    // Implementation
}
```

**Propagate errors with `?` operator:**
```rust
let rules = GRLParser::parse_rules(&content)?;
let value = facts.get("field")?;
```

### Module Organization

**Structure:**
```
src/
├── lib.rs                 # Public API re-exports
├── errors.rs              # Error types
├── types.rs               # Core type definitions
├── engine/                # Forward chaining engine
│   ├── mod.rs
│   ├── engine.rs
│   ├── facts.rs
│   └── knowledge_base.rs
├── backward/              # Backward chaining (feature-gated)
├── rete/                  # RETE algorithm
├── streaming/             # Streaming (feature-gated)
└── parser/                # GRL parser
```

**Re-exports in lib.rs:**
```rust
pub use errors::{Result, RuleEngineError};
pub use types::{Value, Operator};
pub use engine::engine::RustRuleEngine;
pub use parser::grl::GRLParser;
```

### Feature Gates

**Conditional compilation:**
```rust
#[cfg(feature = "backward-chaining")]
pub mod backward;

#[cfg(feature = "streaming")]
pub mod streaming;
```

**Optional dependencies:**
```rust
// In Cargo.toml
[dependencies]
tokio = { version = "1.42", features = ["full"], optional = true }

[features]
streaming = ["tokio"]
```

### Testing

**Test organization:**
- Unit tests: Inline with `#[cfg(test)]` modules
- Integration tests: `tests/` directory
- Doc tests: In doc comments

**Test style:**
```rust
#[test]
fn test_name() -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();
    facts.set("field", Value::Number(42.0));
    
    assert_eq!(facts.get("field")?, Value::Number(42.0));
    Ok(())
}
```

**Use `#[allow(clippy::lint)]` for test-specific patterns:**
```rust
#![allow(clippy::unnecessary_get_then_check)]
```

## Dependencies Philosophy

**Minimize dependencies:** The project recently reduced from 12 to 7 core dependencies (v1.16.1), preferring standard library over external crates:
- `std::thread::available_parallelism()` instead of `num_cpus`
- `std::sync::OnceLock` instead of `once_cell`
- `std::collections::hash_map::RandomState` instead of `fastrand`

**Core dependencies (only add if necessary):**
- serde/serde_json - serialization
- regex - pattern matching
- thiserror - error handling
- chrono - date/time
- log - logging facade
- nom - parser combinators

## Performance Considerations

- Use indexing for O(1) lookups (alpha/beta memory)
- Leverage parallel execution where appropriate
- Benchmark changes with criterion (see `benches/`)
- Profile memory usage for large-scale scenarios

## CI/CD

GitHub Actions workflow runs on:
- Push to `main`
- Pull requests to `main`
- Tags matching `v*.*.*`

**CI checks:**
1. Format check (`cargo fmt -- --check`)
2. Clippy (`-D warnings`)
3. Build (`--all-features`)
4. Tests (`--all-features`)
5. Feature combination tests (5 combinations)
6. Doc tests

**Before committing:**
```bash
make ci  # Runs full CI pipeline locally
```

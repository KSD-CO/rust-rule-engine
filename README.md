# Rust Rule Engine 🦀

[![Crates.io](https://img.shields.io/crates/v/rust-rule-engine.svg)](https://crates.io/crates/rust-rule-engine)
[![Documentation](https://docs.rs/rust-rule-engine/badge.svg)](https://docs.rs/rust-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/KSD-CO/rust-rule-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-rule-engine/actions)

A production-ready Rust rule engine with GRL syntax, forward chaining, optional backward chaining, and optional stream processing.

## Highlights

- Forward chaining with a simple native engine and RETE-UL execution
- Backward chaining for goal-driven inference (`backward-chaining` feature)
- Stream processing with time windows (`streaming` and `streaming-redis` features)
- Custom functions, pattern matching, unification, and explanation support
- Example suite covering getting started, RETE, performance, modules, and reasoning modes

## Installation

```toml
[dependencies]
rust-rule-engine = "1.21.6"
```

Enable optional features as needed:

```toml
[dependencies]
rust-rule-engine = { version = "1.21.6", features = ["backward-chaining"] }
```

Available features:

- `backward-chaining` - goal-driven inference and proof-oriented querying
- `streaming` - in-memory stream processing and time-window support
- `streaming-redis` - Redis-backed streaming state

## Quick Start

```rust
use rust_rule_engine::{Facts, RuleEngine, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = RuleEngine::new();

    engine.add_rule_from_grl(r#"
        rule "vip_discount" {
            when
                Customer.TotalSpent > 10000
            then
                Customer.Discount = 0.15;
        }
    "#)?;

    let mut facts = Facts::new();
    facts.set("Customer.TotalSpent", Value::Number(15000.0));

    engine.execute(&mut facts)?;

    assert_eq!(facts.get("Customer.Discount"), Some(&Value::Number(0.15)));
    Ok(())
}
```

## Examples

Run a few common examples:

```bash
cargo run --example grule_demo
cargo run --example rete_demo
cargo run --features backward-chaining --example simple_query_demo
cargo run --features streaming --example streaming_with_rules_demo
```

The repository includes organized example groups under `/examples`:

- `01-getting-started`
- `02-rete-engine`
- `03-advanced-features`
- `05-performance`
- `07-advanced-rete`
- `09-backward-chaining`
- `10-module-system`

## Documentation

- [Crate documentation](https://docs.rs/rust-rule-engine)
- [Project documentation index](docs/README.md)
- [Getting started guides](docs/getting-started/)
- [Core features](docs/core-features/)
- [Advanced features](docs/advanced-features/)
- [Examples documentation](docs/examples/)
- [Changelog](CHANGELOG.md)

## Development

Common local commands:

```bash
make check
make ci
cargo test --verbose --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

## License

MIT

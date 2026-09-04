# rust-rule-engine

[![Crates.io](https://img.shields.io/crates/v/rust-rule-engine.svg)](https://crates.io/crates/rust-rule-engine)
[![Documentation](https://docs.rs/rust-rule-engine/badge.svg)](https://docs.rs/rust-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

`rust-rule-engine` is a rule engine for Rust with GRL syntax, forward chaining, optional backward chaining, and optional stream processing.

It is designed for business rules, decision automation, expert systems, validation pipelines, and event-driven reasoning workloads.

## Capabilities

- Forward chaining with native execution and RETE-UL optimization
- Goal-driven reasoning via the optional `backward-chaining` feature
- Stream processing with time windows via `streaming` and `streaming-redis`
- Custom functions, pattern matching, unification, and proof/explanation support
- Example coverage for core usage, advanced rules, RETE, modules, and performance

## Installation

```toml
[dependencies]
rust-rule-engine = "1.21.6"
```

With optional features:

```toml
[dependencies]
rust-rule-engine = { version = "1.21.6", features = ["backward-chaining"] }
```

## Feature Flags

- `backward-chaining` — enables goal-driven inference and query workflows
- `streaming` — enables in-memory stream processing and time-window support
- `streaming-redis` — enables Redis-backed streaming state

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

Run common examples locally:

```bash
cargo run --example grule_demo
cargo run --example rete_demo
cargo run --features backward-chaining --example simple_query_demo
cargo run --features streaming --example streaming_with_rules_demo
```

Example groups are organized under `/examples`:

- `01-getting-started`
- `02-rete-engine`
- `03-advanced-features`
- `05-performance`
- `07-advanced-rete`
- `09-backward-chaining`
- `10-module-system`

## Documentation

- [API documentation](https://docs.rs/rust-rule-engine)
- [Project documentation index](docs/README.md)
- [Getting started guides](docs/getting-started/)
- [Core features](docs/core-features/)
- [Advanced features](docs/advanced-features/)
- [Examples documentation](docs/examples/)
- [Changelog](CHANGELOG.md)

## Development

```bash
make check
make ci
cargo test --verbose --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

## License

MIT

# Rust Rule Engine

[![Crates.io](https://img.shields.io/crates/v/rust-rule-engine.svg)](https://crates.io/crates/rust-rule-engine)
[![Documentation](https://docs.rs/rust-rule-engine/badge.svg)](https://docs.rs/rust-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/KSD-CO/rust-rule-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-rule-engine/actions)

A production-ready rule engine for Rust with **forward chaining**, **backward chaining**, and **stream processing**, using the RETE-UL algorithm with alpha/beta memory indexing and the GRL (Grule Rule Language) syntax.

🔗 [Documentation](https://docs.rs/rust-rule-engine) · [Crates.io](https://crates.io/crates/rust-rule-engine) · [Changelog](CHANGELOG.md) · [Examples](examples/README.md)

---

## Features

| Mode | Description | Use cases |
|------|-------------|-----------|
| **Forward Chaining** | Data-driven: facts change → matching rules fire. Native engine for small rule sets, RETE-UL for 100–10,000+ rules with O(1) indexing, plus parallel execution. | Business rules, validation, reactive systems |
| **Backward Chaining** | Goal-driven: given a goal, search facts/rules to prove it. Unification, DFS/BFS/iterative-deepening search, aggregation (`COUNT`/`SUM`/`AVG`/`MIN`/`MAX`), negation, disjunction, nested queries, and automatic query optimization (10–100x speedup). | Expert systems, diagnostics, planning, decision support |
| **Stream Processing** | Event-driven: sliding/tumbling/session time windows, multi-stream correlation, GRL stream syntax integrated with RETE working memory. | Fraud detection, IoT monitoring, real-time analytics, CEP |

Other highlights: proof-tree explanations (JSON/MD/HTML export), TMS-aware proof caching, custom function registration in `when` conditions, module system for organizing rules, and a minimal core dependency footprint (7 crates).

## Installation

```toml
[dependencies]
rust-rule-engine = "1.21"

# Optional features
# rust-rule-engine = { version = "1.21", features = ["backward-chaining", "streaming"] }
```

| Feature | Enables |
|---------|---------|
| `backward-chaining` | Goal-driven reasoning (`BackwardEngine`, GRL queries) |
| `streaming` | Time-windowed stream processing (requires `tokio`) |
| `streaming-redis` | Redis-backed streaming state |

## Quick Start

### Forward chaining

```rust
use rust_rule_engine::{RuleEngine, Facts, Value};

let mut engine = RuleEngine::new();

engine.add_rule_from_grl(r#"
    rule "VIP Discount" {
        when
            Customer.TotalSpent > 10000
        then
            Customer.Discount = 0.15;
    }
"#)?;

let mut facts = Facts::new();
facts.set("Customer.TotalSpent", Value::Number(15000.0));
engine.execute(&mut facts)?;
// Customer.Discount == 0.15
```

### Backward chaining

```rust
use rust_rule_engine::backward::BackwardEngine;

let mut engine = BackwardEngine::new(kb);

let result = engine.query("Order.AutoApproved == true", &mut facts)?;
if result.provable {
    println!("Proof: {:?}", result.proof_trace);
}
```

### Stream processing

```rust
use rust_rule_engine::streaming::{TimeWindow, WindowType, StreamEvent};
use std::time::Duration;

let mut window = TimeWindow::new(WindowType::Sliding, Duration::from_secs(60), start_ms, 10_000);
window.record(StreamEvent::with_timestamp("login_failure", data, "auth", event_ts_ms));

if window.count() >= 10 {
    // 10+ matching events in the trailing 60s
}
```

```grl
rule "Fraud Alert" {
    when
        login: LoginEvent from stream("logins") over window(10 min, sliding) &&
        purchase: PurchaseEvent from stream("purchases") over window(10 min, sliding) &&
        login.user_id == purchase.user_id &&
        login.ip_address != purchase.ip_address
    then
        Alert.trigger("IP mismatch detected");
}
```

More runnable examples are in [`examples/`](examples/README.md), grouped by topic (getting started, RETE engine, advanced features, performance, backward chaining, module system).

## Documentation

| Section | Description |
|---------|-------------|
| [Quick Start](docs/getting-started/QUICK_START.md) | Get running in 5 minutes |
| [Installation](docs/getting-started/INSTALLATION.md) | Setup guide |
| [Concepts](docs/getting-started/CONCEPTS.md) | Core concepts |
| [GRL Syntax](docs/core-features/GRL_SYNTAX.md) | Rule language reference |
| [Features Overview](docs/core-features/FEATURES.md) | All engine capabilities |
| [RETE Optimization](docs/advanced-features/RETE_OPTIMIZATION.md) | Indexing & memory internals |
| [Streaming & CEP](docs/advanced-features/STREAMING.md) | Stream processing guide |
| [API Reference](docs/api-reference/API_REFERENCE.md) | Full public API |
| [GRL Query Syntax](docs/api-reference/GRL_QUERY_SYNTAX.md) | Backward chaining queries |
| [Backward Chaining Quick Start](docs/BACKWARD_CHAINING_QUICK_START.md) | Goal-driven reasoning |
| [Troubleshooting](docs/guides/TROUBLESHOOTING.md) | Common issues |

**[Full documentation index →](docs/README.md)**

## What's New

See [CHANGELOG.md](CHANGELOG.md) for the complete version history. Latest: **v1.21.7** — fixed CodeQL security alerts (cleartext-logging false positives, workflow token permissions) and reorganized this README.

## Contributing

Issues and pull requests are welcome at [KSD-CO/rust-rule-engine](https://github.com/KSD-CO/rust-rule-engine). Before submitting a PR, please run:

```bash
make check   # fmt, clippy, and tests
```

## License

Licensed under the [MIT License](LICENSE).

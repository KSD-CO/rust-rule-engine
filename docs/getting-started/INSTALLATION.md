# Installation Guide

> **Version:** 1.11.0
> **Last Updated:** December 10, 2024

Complete installation guide for Rust Rule Engine.

---

## 📋 Requirements

- **Rust:** 1.70.0 or higher
- **Cargo:** Latest stable version

Check your Rust version:
```bash
rustc --version
cargo --version
```

Need to install Rust? → [https://rustup.rs/](https://rustup.rs/)

---

## 🚀 Quick Installation

### Option 1: Basic Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-rule-engine = "1.11"
```

### Option 2: With Features

```toml
[dependencies.rust-rule-engine]
version = "1.11"
features = ["backward-chaining", "streaming"]
```

### Option 3: From Git (Latest Development)

```toml
[dependencies]
rust-rule-engine = { git = "https://github.com/KSD-CO/rust-rule-engine", branch = "main" }
```

---

## 🎛️ Available Features

| Feature | Description | Size Impact | Use Case |
|---------|-------------|-------------|----------|
| **Default** | Forward chaining + RETE | Minimal | Basic rule engine |
| `backward-chaining` | Goal-driven inference | +150KB | Queries & reasoning |
| `streaming` | Complex Event Processing | +100KB | Real-time events |
| `streaming-redis` | Redis state backend | +200KB | Distributed systems |

### Feature Combinations

#### Minimal Setup (Forward Chaining Only)
```toml
[dependencies]
rust-rule-engine = "1.11"
```

**Use for:** Basic business rules, decision automation

#### Full Featured (Everything)
```toml
[dependencies.rust-rule-engine]
version = "1.11"
features = ["backward-chaining", "streaming", "streaming-redis"]
```

**Use for:** Enterprise applications, distributed systems

#### Query & Reasoning
```toml
[dependencies.rust-rule-engine]
version = "1.11"
features = ["backward-chaining"]
```

**Use for:** Diagnostic systems, expert systems

#### Stream Processing
```toml
[dependencies.rust-rule-engine]
version = "1.11"
features = ["streaming"]
```

**Use for:** IoT, real-time analytics, CEP

---

## ✅ Verify Installation

Create `src/main.rs`:

```rust
use rust_rule_engine::{Engine, Facts, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new();

    engine.add_rule_from_string(r#"
        rule "Test" {
            when
                X == 1
            then
                Y = 2;
        }
    "#)?;

    let mut facts = Facts::new();
    facts.set("X", Value::Integer(1));

    engine.run(&mut facts)?;

    assert_eq!(facts.get("Y"), Some(&Value::Integer(2)));
    println!("✅ Installation verified!");

    Ok(())
}
```

Run:
```bash
cargo run
```

**Expected output:**
```
✅ Installation verified!
```

---

## 🔧 Platform-Specific Notes

### Linux

No additional setup required. Install and go!

```bash
cargo add rust-rule-engine
cargo build
```

### macOS

No additional setup required.

```bash
cargo add rust-rule-engine
cargo build
```

### Windows

Works out of the box. If you encounter issues with Redis features:

```powershell
# Install Visual Studio Build Tools first
# Then:
cargo add rust-rule-engine --features backward-chaining,streaming
cargo build
```

### Docker

```dockerfile
FROM rust:1.75

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --features backward-chaining,streaming

CMD ["./target/release/your-app"]
```

---

## 📦 Optional Dependencies

### For Redis State Backend

```toml
[dependencies.rust-rule-engine]
version = "1.11"
features = ["streaming-redis"]

[dependencies]
redis = "0.27"
tokio = { version = "1.42", features = ["full"] }
```

**Install Redis:**
```bash
# macOS
brew install redis
brew services start redis

# Linux (Ubuntu/Debian)
sudo apt-get install redis-server
sudo systemctl start redis

# Docker
docker run -d -p 6379:6379 redis:latest
```

### For Graph Analysis (Advanced)

```toml
[dependencies]
petgraph = "0.6"  # Used by backward-chaining feature
```

This is automatically included with `backward-chaining` feature.

---

## 🔄 Upgrading from Previous Versions

### From 1.10.x → 1.11.0

**No breaking changes!** Fully backward compatible.

```bash
# Update Cargo.toml
rust-rule-engine = "1.11"

# Update dependencies
cargo update

# Rebuild
cargo build
```

**New features available:**
- Nested queries in backward chaining
- Query optimization (10-100x speedup)

### From 1.9.x → 1.10.x

```bash
cargo update
```

**New:** Disjunction (OR) support in queries

### From 1.0.x → Latest

Review [Migration Guide](../guides/MIGRATION.md) for major version changes.

---

## 🏗️ Building from Source

```bash
# Clone repository
git clone https://github.com/KSD-CO/rust-rule-engine
cd rust-rule-engine

# Build with all features
cargo build --all-features --release

# Run tests
cargo test --all-features

# Run examples
cargo run --example basic_usage
```

---

## 🧪 Development Setup

### For Contributors

```bash
# Clone and setup
git clone https://github.com/KSD-CO/rust-rule-engine
cd rust-rule-engine

# Install development dependencies
cargo install cargo-watch
cargo install cargo-tarpaulin

# Run in development mode with auto-reload
cargo watch -x "run --example basic_usage"

# Run tests with coverage
cargo tarpaulin --all-features
```

### Recommended Tools

```bash
# Code formatting
cargo install rustfmt

# Linting
cargo install clippy

# Documentation
cargo doc --open --all-features
```

---

## 📊 Binary Size Optimization

### Minimal Binary

```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
strip = true        # Strip symbols
```

**Result:** ~2MB binary (forward chaining only)

### With All Features

**Result:** ~4MB binary (includes backward-chaining + streaming)

---

## 🐛 Troubleshooting Installation

### Issue: "Can't find rust-rule-engine"

```bash
# Ensure you're using the correct version
cargo update
cargo clean
cargo build
```

### Issue: "Feature not found"

Check that you're using version 1.11+:
```toml
rust-rule-engine = "1.11"
```

### Issue: Redis compilation error

Install Redis separately or disable the feature:
```toml
features = ["backward-chaining", "streaming"]
# Remove "streaming-redis" if not needed
```

### Issue: Out of memory during compilation

```bash
# Reduce parallel jobs
cargo build -j 2
```

### Still Having Issues?

- 📖 [Troubleshooting Guide](../guides/TROUBLESHOOTING.md)
- 💬 [GitHub Discussions](https://github.com/KSD-CO/rust-rule-engine/discussions)
- 🐛 [Report Bug](https://github.com/KSD-CO/rust-rule-engine/issues)

---

## 🎯 Next Steps

**✅ Installed?** → [Quick Start Guide](QUICK_START.md)

**📚 Learn Concepts** → [Basic Concepts](CONCEPTS.md)

**🔨 Build Something** → [First Rules Tutorial](FIRST_RULES.md)

**📖 API Reference** → [API Documentation](../api-reference/API_REFERENCE.md)

---

## 📄 License & Support

- **License:** MIT
- **Repository:** [github.com/KSD-CO/rust-rule-engine](https://github.com/KSD-CO/rust-rule-engine)
- **Documentation:** [docs.rs/rust-rule-engine](https://docs.rs/rust-rule-engine)
- **Crate:** [crates.io/crates/rust-rule-engine](https://crates.io/crates/rust-rule-engine)

---

## Navigation

📚 **[Documentation Home](../README.md)** | ▶️ **Next: [Quick Start](QUICK_START.md)**

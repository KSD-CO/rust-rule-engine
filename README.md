# Rust Rule Engine v0.10.0 🦀⚡

[![Crates.io](https://img.shields.io/crates/v/rust-rule-engine.svg)](https://crates.io/crates/rust-rule-engine)
[![Documentation](https://docs.rs/rust-rule-engine/badge.svg)](https://docs.rs/rust-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/KSD-CO/rust-rule-engine/workflows/CI/badge.svg)](https://github.com/KSD-CO/rust-rule-engine/actions)

A high-performance rule engine for Rust with **RETE-UL algorithm**, **CLIPS-inspired features**, **Plugin System**, and **GRL (Grule Rule Language) support**. Designed for production use with ~97% Drools compatibility.

🔗 **[GitHub](https://github.com/KSD-CO/rust-rule-engine)** | **[Documentation](https://docs.rs/rust-rule-engine)** | **[Crates.io](https://crates.io/crates/rust-rule-engine)**

---

## ✨ What's New in v0.10.0

🎉 **CLIPS-Inspired Features + Function Calls in WHEN Clause**!

- **🔧 Function Calls in WHEN** - Call AI/custom functions directly in rule conditions *(NEW)*
- **📋 Template System** - Type-safe schema definitions for structured facts
- **🌍 Defglobal** - Global variables with thread-safe access
- **📈 Drools Compatibility** - Improved from ~95% to **~97%**
- **✅ Quality** - 85 tests passing, zero breaking changes

[**See Release Notes →**](RELEASE_v0.10.0.md) | [**CLIPS Features Guide →**](CLIPS_INSPIRED_FEATURES.md)

---

## 🚀 Key Features

### Native Engine
- **GRL Support** - Full Grule-compatible syntax
- **Function Calls in WHEN** - Call functions directly in conditions *(NEW in v0.10.0)*
- **Plugin System** - 44+ actions, 33+ functions
- **Knowledge Base** - Centralized rule management
- **Type Safety** - Rust's compile-time guarantees
- **Production Ready** - REST API, monitoring, health checks

### RETE-UL Engine (Recommended for 100+ rules)
- **🔥 RETE Algorithm** - High-performance pattern matching (~97% Drools parity)
- **📋 Template System** - Type-safe structured facts *(NEW in v0.10.0)*
- **🌍 Defglobal** - Global variables across firings *(NEW in v0.10.0)*
- **⚡ Incremental Updates** - Only re-evaluate affected rules (2x speedup)
- **🧠 Working Memory** - FactHandles with insert/update/retract
- **🎯 Advanced Agenda** - Salience, activation groups, no-loop
- **🔗 Variable Binding** - Cross-pattern $var syntax
- **💾 Memoization** - 99.99% cache hit rate

**Choose Your Engine:**
- **< 50 rules** → Native Engine (simpler API, plugin support)
- **> 100 rules** → RETE-UL Engine (better performance, advanced features)
- **Both needs** → Hybrid approach

📖 [**Engine Comparison Guide →**](ENGINE_COMPARISON.md) | [**Quick Start Guide →**](QUICK_START_ENGINES.md)

---

## 📦 Installation

```toml
[dependencies]
rust-rule-engine = "0.10.0"
```

### Optional Features
```toml
# Enable async support
rust-rule-engine = { version = "0.10.0", features = ["async"] }

# Enable all features
rust-rule-engine = { version = "0.10.0", features = ["full"] }
```

---

## 🎯 Quick Start

### Option 1: Native Engine (Simple & Plugin-rich)

```rust
use rust_rule_engine::{RustRuleEngine, Facts, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine with plugins
    let mut engine = RustRuleEngine::new();
    engine.load_default_plugins()?;

    // Load rules from GRL file
    engine.load_rules_from_file("rules/discount.grl")?;

    // Add facts
    let mut facts = Facts::new();
    facts.set("customer.tier", Value::String("gold".to_string()));
    facts.set("order.amount", Value::Float(1500.0));

    // Execute rules
    engine.execute(&mut facts)?;

    // Get result
    println!("Discount: {}", facts.get("order.discount"));

    Ok(())
}
```

**GRL Rule Example** (`rules/discount.grl`):
```grl
rule "GoldCustomerDiscount" salience 10 {
    when
        customer.tier == "gold" && order.amount > 1000
    then
        order.discount = order.amount * 0.15;
        Log("Applied 15% gold customer discount");
}
```

### Option 2: RETE-UL Engine (High Performance)

```rust
use rust_rule_engine::rete::{
    IncrementalEngine, GrlReteLoader, TypedFacts, FactValue,
    TemplateBuilder, FieldType
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = IncrementalEngine::new();

    // Optional: Define template for type safety
    let order_template = TemplateBuilder::new("Order")
        .required_string("order_id")
        .float_field("amount")
        .float_field("discount")
        .build();
    engine.templates_mut().register(order_template);

    // Load rules from GRL
    GrlReteLoader::load_from_file("rules/discount.grl", &mut engine)?;

    // Insert facts with validation
    let mut order = TypedFacts::new();
    order.set("order_id", FactValue::String("ORD-001".to_string()));
    order.set("amount", FactValue::Float(1500.0));

    let handle = engine.insert_with_template("Order", order)?;

    // Fire rules
    engine.reset();
    let fired = engine.fire_all();
    println!("Fired {} rules", fired.len());

    // Query results
    if let Some(order) = engine.working_memory().get(&handle) {
        println!("Discount: {:?}", order.data.get("discount"));
    }

    Ok(())
}
```

---

## 🔧 NEW: Function Calls in WHEN Clause

**v0.10.0 introduces the ability to call functions directly in rule conditions!**

### ✨ Before (Rule Chaining)
```grl
rule "Step1: Call AI" {
    when Customer.needsCheck == true
    then set(Customer.sentiment, aiSentiment(Customer.feedback));
}

rule "Step2: Check Result" {
    when Customer.sentiment == "negative"
    then Alert("Negative feedback detected!");
}
```

### ✨ After (Direct Function Calls)
```grl
rule "Check Sentiment" {
    when aiSentiment(Customer.feedback) == "negative"
    then Alert("Negative feedback detected!");
}
```

### 📖 Use Cases

**AI/ML Integration:**
```grl
rule "Fraud Detection" {
    when aiFraud(Transaction.amount, Transaction.userId) == true
    then set(Transaction.status, "blocked");
}
```

**Business Logic:**
```grl
rule "Credit Check" {
    when creditScore(Customer.id) > 750
    then set(Customer.tier, "premium");
}
```

**Data Validation:**
```grl
rule "Email Validation" {
    when validateEmail(User.email) == false
    then set(User.error, "Invalid email format");
}
```

**See [ai_functions_in_when.rs](examples/ai_functions_in_when.rs) for complete examples!**

---

## 📚 Documentation

### 📖 Getting Started
- [**Quick Start Guide**](QUICK_START_ENGINES.md) - Choose and use your engine
- [**Engine Comparison**](ENGINE_COMPARISON.md) - Native vs RETE-UL decision guide
- [**Examples**](examples/) - 30+ working examples

### 🔧 Core Features
- [**Features Guide**](docs/FEATURES.md) - All engine features explained
- [**Plugin System**](docs/PLUGINS.md) - Built-in plugins & custom creation
- [**Advanced Usage**](docs/ADVANCED_USAGE.md) - Complex patterns & workflows
- [**AI Integration**](docs/REAL_AI_INTEGRATION.md) - ML models & LLM integration

### 🚀 RETE-UL Engine
- [**RETE Guide**](docs/RETE_GUIDE.md) - Complete RETE-UL documentation
- [**CLIPS Features**](CLIPS_INSPIRED_FEATURES.md) - Template System & Defglobal
- [**CLIPS Analysis**](CLIPS_FEATURES_ANALYSIS.md) - Feature comparison & roadmap

### 🌐 Distributed & Production
- [**Streaming Engine**](docs/STREAMING.md) - Real-time stream processing
- [**Distributed Setup**](docs/distributed_explained.md) - Getting started with distributed mode
- [**Distributed Architecture**](docs/distributed_architecture.md) - Cluster setup & scaling
- [**Distributed Features**](docs/distributed_features_guide.md) - Complete distributed guide
- [**Performance Guide**](docs/PERFORMANCE.md) - Benchmarks & optimization

### 📋 Reference
- [**API Reference**](docs/API_REFERENCE.md) - Complete API documentation
- [**GRL Syntax**](docs/GRL_SYNTAX.md) - Rule language reference
- [**Roadmap**](docs/ROADMAP.md) - Future plans & upcoming features
- [**Release Notes**](RELEASE_v0.10.0.md) - What's new in v0.10.0
- [**Changelog**](CHANGELOG_v0.10.0.md) - Complete changelog

---

## 🖥️ VS Code Extension

Install [GRL Syntax Highlighting](https://marketplace.visualstudio.com/items?itemName=tonthatvu.grl-syntax-highlighting) for `.grl` files:

**Features:**
- Syntax highlighting for GRL
- Snippets for rules, actions, functions
- Auto-detection of `.grl` files

**Install:** Search `grl-syntax-highlighting` in VS Code Extensions

---

## 🎯 Use Cases

### 1. Business Rules Engine
```rust
// Pricing, discounts, loyalty programs
rule "VIPDiscount" {
    when customer.points > 1000
    then order.discount = 0.20;
}
```

### 2. Fraud Detection
```rust
// Real-time fraud scoring
rule "HighRiskTransaction" {
    when transaction.amount > 10000 &&
         transaction.location != customer.usual_location
    then fraud.score = 0.85;
}
```

### 3. Workflow Automation
```rust
// Multi-step approval workflows
rule "ManagerApproval" agenda-group "approvals" {
    when request.amount > 5000
    then request.requires_manager = true;
}
```

### 4. Real-Time Systems
```rust
// IoT, monitoring, alerts
rule "TemperatureAlert" {
    when sensor.temp > 80
    then Alert.send("High temperature!");
}
```

**More examples:** [examples/](examples/) directory

---

## ⚡ Performance

### RETE-UL Engine Benchmarks
- **Pattern Matching**: ~4µs per fact insertion (1000 facts)
- **Incremental Updates**: 2x speedup (only affected rules)
- **Memoization**: 99.99% cache hit rate
- **Template Validation**: 1-2µs per fact
- **Global Variables**: 120ns read, 180ns write

### Native Engine Benchmarks
- **Rule Execution**: ~10µs per rule (simple conditions)
- **Plugin Actions**: ~2-5µs per action call
- **Facts Access**: O(1) HashMap lookups

**Comparison:** [Performance Guide](docs/PERFORMANCE.md)

---

## 🗺️ Roadmap

### v0.11.0 (Next Release - 2-3 weeks)
- [ ] **Deffacts** - Initial fact definitions (CLIPS feature)
- [ ] **Test CE** - Arbitrary conditions in patterns
- [ ] **Multi-field Variables** - Array pattern matching
- **Target**: ~98-99% Drools compatibility

### v1.0.0 (Stable Release)
- [ ] Full API stability guarantee
- [ ] 99% Drools compatibility
- [ ] Performance optimizations
- [ ] Comprehensive benchmarks
- [ ] Production hardening

### Future Features
- [ ] Truth Maintenance System (TMS)
- [ ] Module System for rule organization
- [ ] Backward Chaining support
- [ ] Interactive debugger
- [ ] Visual rule builder UI

[**Full Roadmap →**](docs/ROADMAP.md)

---

## 📊 Comparison with Other Engines

| Feature | Rust Engine | Drools | CLIPS |
|---------|-------------|--------|-------|
| **Performance** | ⚡ Excellent | Good | Excellent |
| **Type Safety** | ✅ Compile-time | ⚠️ Runtime | ⚠️ Runtime |
| **Memory Safety** | ✅ Guaranteed | ❌ JVM | ❌ Manual |
| **RETE Algorithm** | ✅ RETE-UL | ✅ Full RETE | ✅ Full RETE |
| **Pattern Matching** | ✅ Advanced | ✅ Advanced | ✅ Advanced |
| **Plugin System** | ✅ Native | ⚠️ Limited | ❌ No |
| **GRL Support** | ✅ Full | N/A | N/A |
| **Template System** | ✅ v0.10.0 | ✅ | ✅ |
| **Defglobal** | ✅ v0.10.0 | ✅ | ✅ |
| **Drools Compatibility** | ~97% | 100% | ~60% |
| **Learning Curve** | Medium | High | High |
| **Ecosystem** | Growing | Mature | Mature |

---

## 🤝 Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup
```bash
# Clone repository
git clone https://github.com/KSD-CO/rust-rule-engine.git
cd rust-rule-engine

# Run tests
cargo test

# Run examples
cargo run --example rete_template_globals_demo

# Build documentation
cargo doc --open
```

---

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file.

---

## 🙏 Acknowledgments

**Inspired by:**
- [Drools](https://www.drools.org/) - JBoss Rule Engine
- [CLIPS](https://www.clipsrules.net/) - NASA C Language Integrated Production System
- [Grule](https://github.com/hyperjumptech/grule-rule-engine) - Go Rule Engine

**Special Thanks:**
- Rust community for amazing tools and libraries
- Contributors who helped improve the engine
- Users providing valuable feedback

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/KSD-CO/rust-rule-engine/issues)
- **Discussions**: [GitHub Discussions](https://github.com/KSD-CO/rust-rule-engine/discussions)
- **Email**: tonthatvu.hust@gmail.com

---

## 📈 Stats

![GitHub stars](https://img.shields.io/github/stars/KSD-CO/rust-rule-engine?style=social)
![GitHub forks](https://img.shields.io/github/forks/KSD-CO/rust-rule-engine?style=social)
![Crates.io downloads](https://img.shields.io/crates/d/rust-rule-engine)

---

<div align="center">

**Made with ❤️ by Ton That Vu**

[⭐ Star us on GitHub](https://github.com/KSD-CO/rust-rule-engine) | [📦 Download from Crates.io](https://crates.io/crates/rust-rule-engine)

</div>

# Rust Rule Engine v1.18.0 🦀⚡🚀

[![Crates.io](https://img.shields.io/crates/v/rust-rule-engine.svg)](https://crates.io/crates/rust-rule-engine)
[![Documentation](https://docs.rs/rust-rule-engine/badge.svg)](https://docs.rs/rust-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/KSD-CO/rust-rule-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-rule-engine/actions)

A blazing-fast production-ready rule engine for Rust with **SIMD/zero-copy/parallel parsing** supporting **both Forward and Backward Chaining**. Features RETE-UL algorithm with **Alpha Memory Indexing** and **Beta Memory Indexing**, parallel execution, goal-driven reasoning, and GRL (Grule Rule Language) syntax.

🔗 **[GitHub](https://github.com/KSD-CO/rust-rule-engine)** | **[Documentation](https://docs.rs/rust-rule-engine)** | **[Crates.io](https://crates.io/crates/rust-rule-engine)**

---

## ⚡ NEW in v1.18.0: Advanced Parsing Optimizations

**Phase 3 Complete:** SIMD + Zero-Copy + Parallel Parsing

- 🚀 **SIMD Search** - Vector-accelerated pattern matching (2-4x faster)
- 🧠 **Zero-Copy Parsing** - Lifetime-based parsing without allocations (90% memory reduction)
- 🔀 **Parallel Parsing** - Multi-core rule parsing (4-8x faster on quad-core)
- 📊 **4-60x Total Speedup** - Combined optimization improvements
- ✅ **193 Tests Passing** - Comprehensive validation

---

## 🎯 Reasoning Modes

### 🔄 Forward Chaining (Data-Driven)
**"When facts change, fire matching rules"**

- **Native Engine** - Simple pattern matching for small rule sets
- **RETE-UL** - Optimized network for 100-10,000 rules with O(1) indexing
- **Parallel Execution** - Multi-threaded rule evaluation

**Use Cases:** Business rules, validation, reactive systems, decision automation

### 🎯 Backward Chaining (Goal-Driven)
**"Given a goal, find facts/rules to prove it"**

- **Unification** - Pattern matching with variable bindings
- **Search Strategies** - DFS, BFS, Iterative Deepening
- **Aggregation** - COUNT, SUM, AVG, MIN, MAX
- **Negation** - NOT queries with closed-world assumption
- **Explanation** - Proof trees with JSON/MD/HTML export
- **Disjunction** - OR patterns for alternative paths
- **Nested Queries** - Subqueries with shared variables
- **Query Optimization** - Automatic goal reordering for 10-100x speedup

**Use Cases:** Expert systems, diagnostics, planning, decision support, AI reasoning

### 🌊 Stream Processing (Event-Driven) 🆕
**"Process real-time event streams with time-based windows"**

- **GRL Stream Syntax** - Declarative stream pattern definitions
- **StreamAlphaNode** - RETE-integrated event filtering & windowing
- **Time Windows** - Sliding (continuous), tumbling (non-overlapping), and **session (gap-based)** 🆕
- **Multi-Stream Correlation** - Join events from different streams
- **WorkingMemory Integration** - Stream events become facts for rule evaluation

**Use Cases:** Real-time fraud detection, IoT monitoring, financial analytics, security alerts, CEP

**Example:**
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

---

## 🚀 Quick Start

### Forward Chaining Example
```rust
use rust_rule_engine::{RuleEngine, Facts, Value};

let mut engine = RuleEngine::new();

// Define rule in GRL
engine.add_rule_from_grl(r#"
    rule "VIP Discount" {
        when
            Customer.TotalSpent > 10000
        then
            Customer.Discount = 0.15;
    }
"#)?;

// Add facts and execute
let mut facts = Facts::new();
facts.set("Customer.TotalSpent", Value::Number(15000.0));
engine.execute(&mut facts)?;

// Result: Customer.Discount = 0.15 ✓
```

### Backward Chaining Example
```rust
use rust_rule_engine::backward::BackwardEngine;

let mut engine = BackwardEngine::new(kb);

// Query: "Can this order be auto-approved?"
let result = engine.query(
    "Order.AutoApproved == true",
    &mut facts
)?;

if result.provable {
    println!("Order can be auto-approved!");
    println!("Proof: {:?}", result.proof_trace);
}
```

### Stream Processing Example 🆕
```rust
use rust_rule_engine::parser::GRLParser;
use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
use rust_rule_engine::rete::working_memory::WorkingMemory;

// Parse GRL stream pattern
let grl = r#"login: LoginEvent from stream("logins") over window(5 min, sliding)"#;
let (_, pattern) = parse_stream_pattern(grl)?;

// Create stream processor
let mut node = StreamAlphaNode::new(
    &pattern.source.stream_name,
    pattern.event_type,
    pattern.source.window.as_ref().map(|w| WindowSpec {
        duration: w.duration,
        window_type: w.window_type.clone(),
    }),
);

// Process events in real-time
let mut wm = WorkingMemory::new();
for event in event_stream {
    if node.process_event(&event) {
        // Event passed filters and is in window
        wm.insert_from_stream("logins".to_string(), event);
        // Now available for rule evaluation!
    }
}

// Run: cargo run --example streaming_fraud_detection --features streaming
```

---

## ✨ What's New in v1.17.0 🎉

### 🚀 Proof Graph Caching with TMS Integration

**Global cache for proven facts** with dependency tracking and automatic invalidation for backward chaining!

#### Key Features

**1. Proof Caching**
- Cache proven facts with their justifications (rule + premises)
- O(1) lookup by fact key (predicate + arguments)
- Multiple justifications per fact (different ways to prove)
- Thread-safe concurrent access with Arc<Mutex<>>

**2. Dependency Tracking**
- Forward edges: Track which rules used a fact as premise
- Reverse edges: Track which facts a fact depends on
- Automatic dependency graph construction during proof

**3. TMS-Aware Invalidation**
- Integrates with RETE's IncrementalEngine insert_logical
- When premise retracted → cascading invalidation through dependents
- Recursive propagation through entire dependency chain
- Statistics tracking (hits, misses, invalidations, justifications)

**4. Search Integration**
- Seamlessly integrated into DepthFirstSearch and BreadthFirstSearch
- Cache lookup before condition evaluation (early return on hit)
- Automatic cache updates via inserter closure


#### Usage Example

```rust
use rust_rule_engine::backward::{BackwardEngine, DepthFirstSearch};
use rust_rule_engine::rete::IncrementalEngine;

// Create engines
let mut rete_engine = IncrementalEngine::new();
let kb = /* load rules */;
let mut backward_engine = BackwardEngine::new(kb);

// Create search with ProofGraph enabled
let search = DepthFirstSearch::new_with_engine(
    backward_engine.kb().clone(),
    Arc::new(Mutex::new(rete_engine)),
);

// First query builds cache
let result1 = backward_engine.query_with_search(
    "eligible(?x)",
    &mut facts,
    Box::new(search.clone()),
)?;

// Subsequent queries use cache 
let result2 = backward_engine.query_with_search(
    "eligible(?x)",
    &mut facts,
    Box::new(search),
)?;
```

#### Dependency Tracking Example

```rust
// Given rules: A → B → C (chain dependency)
let result_c = engine.query("C", &mut facts)?;  // Proves A, B, C

// Retract A (premise)
facts.set("A", FactValue::Bool(false));

// Automatic cascading invalidation:
// A invalidated → B invalidated → C invalidated
// Total: 3 invalidations propagated through dependency graph
```

#### Multiple Justifications Example

```rust
// Same fact proven 3 different ways:
// Rule 1: HighSpender → eligible
// Rule 2: LoyalCustomer → eligible  
// Rule 3: Subscription → eligible

let result = engine.query("eligible(?x)", &mut facts)?;

// ProofGraph stores all 3 justifications
// If one premise fails, others still valid!
```

**Try it yourself:**
```bash
# Run comprehensive demo with 5 scenarios
cargo run --example proof_graph_cache_demo --features backward-chaining

# Run integration tests
cargo test proof_graph --features backward-chaining
```

**New Files:**
- `src/backward/proof_graph.rs` (520 lines) - Core ProofGraph implementation
- `tests/proof_graph_integration_test.rs` - 6 comprehensive tests
- `examples/09-backward-chaining/proof_graph_cache_demo.rs` - Interactive demo

**Features:**
- ✅ Global proof caching with O(1) lookup
- ✅ Dependency tracking (forward + reverse edges)
- ✅ TMS-aware cascading invalidation
- ✅ Multiple justifications per fact
- ✅ Thread-safe concurrent access
- ✅ Statistics tracking (hits/misses/invalidations)
- ✅ Zero overhead when cache miss
- ✅ Automatic integration with DFS/BFS search

---

## ✨ What's New in v1.16.1 🎉

### 🧹 Minimal Dependencies - Pure Stdlib

**Removed 5 external dependencies** - replaced with Rust stdlib or removed dead code:

**Replaced with stdlib:**
- ❌ `num_cpus` → ✅ `std::thread::available_parallelism()` (Rust 1.59+)
- ❌ `once_cell` → ✅ `std::sync::OnceLock` (Rust 1.70+)
- ❌ `fastrand` → ✅ `std::collections::hash_map::RandomState`

**Removed unused:**
- ❌ `petgraph` - Declared but never used (zero code references)
- ❌ `futures` - Declared but never used (tokio is sufficient)

**Benefits:**
- 📦 **5 fewer crates** - down from 12 to 7 core dependencies (41% reduction!)
- 🛡️ **More reliable** - 100% stdlib for threading, lazy init, randomization
- ⚡ **Zero performance regression** - all benchmarks unchanged
- 🔧 **Modern Rust** - using latest stdlib features

**Final Core Dependencies:** Only 6 essential crates (regex-free!)
```
chrono, log, nom, serde, serde_json, thiserror
```

**Note:** `regex` is now optional via `legacy-regex-parser` feature flag.

**Optional dependencies** (by feature):
- `tokio` - Async runtime for streaming
- `redis` - State backend for streaming-redis

**Code changes:**
- Thread detection: `num_cpus::get()` → `std::thread::available_parallelism()`
- Lazy regex (20 patterns): `once_cell::Lazy` → `std::sync::OnceLock`
- Random generation: `fastrand` → `RandomState::new().build_hasher()`
- Fixed flaky test in session window eviction

**Testing:**
- ✅ All 428+ tests passing
- ✅ All 14+ examples working
- ✅ All features validated (streaming, backward-chaining, etc.)

---

## ✨ What's New in v1.16.0

### 🪟 Session Windows for Stream Processing

Complete implementation of **session-based windowing** for real-time event streams! Session windows dynamically group events based on **inactivity gaps** rather than fixed time boundaries.

**What are Session Windows?**

Unlike sliding or tumbling windows, session windows adapt to natural event patterns:

```
Events: A(t=0), B(t=1), C(t=2), [gap 10s], D(t=12), E(t=13)
Timeout: 5 seconds

Result:
  Session 1: [A, B, C]  - ends when gap > 5s
  Session 2: [D, E]     - starts after gap > 5s
```

**GRL Syntax:**
```grl
rule "UserSessionAnalysis" {
    when
        activity: UserAction from stream("user-activity")
            over window(5 min, session)
    then
        AnalyzeSession(activity);
}
```

**Rust API:**
```rust
use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
use rust_rule_engine::streaming::window::WindowType;
use std::time::Duration;

let window = WindowSpec {
    duration: Duration::from_secs(60),
    window_type: WindowType::Session {
        timeout: Duration::from_secs(5),  // Gap threshold
    },
};

let mut node = StreamAlphaNode::new("user-events", None, Some(window));
```

**Perfect for:**
- 📊 **User Session Analytics** - Track natural user behavior sessions
- 🛒 **Cart Abandonment** - Detect when users don't complete checkout
- 🔒 **Fraud Detection** - Identify unusual session patterns
- 📡 **IoT Sensor Grouping** - Group burst events from sensors

**Features:**
- ✅ Automatic session boundary detection based on inactivity
- ✅ Dynamic session sizes (adapts to activity patterns)
- ✅ O(1) event processing with minimal overhead
- ✅ Full integration with RETE network
- ✅ 7 comprehensive tests (all passing)
- ✅ Interactive demo: `cargo run --example session_window_demo --features streaming`

---

## ✨ What's New in v1.15.1

### 🧹 Codebase Cleanup

Major cleanup and optimization of the project structure for better maintainability and developer experience!

**🔧 Dependencies Optimized (-75% dev-deps)**
- Removed 9 unused dev-dependencies (axum, tower, reqwest, tracing, etc.)
- Eliminated duplicate dependencies (serde, chrono already in main deps)
- Kept only essentials: criterion, tokio, serde_yaml
- Faster build times and smaller binary size

**Benefits:**
- ⚡ Faster compilation and CI runs
- 📚 Easier onboarding with clear example structure
- 🧹 Less code to maintain (-76% examples)
- ✅ Production-ready with all tests passing

---

## ✨ What's New in v1.15.0

### ➕ Array Append Operator (`+=`)

Added support for the `+=` operator to append values to arrays in GRL actions! This is particularly useful for building recommendation lists, accumulating results, and managing collections.

**GRL Syntax:**
```grl
rule "Product Recommendation" salience 100 no-loop {
    when
        ShoppingCart.items contains "Laptop" &&
        !(Recommendation.items contains "Mouse")
    then
        Recommendation.items += "Mouse";          // Append to array
        Recommendation.items += "USB-C Hub";      // Multiple appends
        Log("Added recommendations");
}
```

**Rust Usage:**
```rust
use rust_rule_engine::rete::{IncrementalEngine, TypedFacts, FactValue};
use rust_rule_engine::rete::grl_loader::GrlReteLoader;

let mut engine = IncrementalEngine::new();
GrlReteLoader::load_from_file("rules.grl", &mut engine)?;

let mut facts = TypedFacts::new();
facts.set("ShoppingCart.items", FactValue::Array(vec![
    FactValue::String("Laptop".to_string())
]));
facts.set("Recommendation.items", FactValue::Array(vec![]));

engine.insert_typed_facts("ShoppingCart", facts.clone());
engine.fire_all(&mut facts, 10);

// Result: Recommendation.items = ["Mouse", "USB-C Hub"] ✓
```

**Integration with Rule Mining:**

The `+=` operator works seamlessly with [rust-rule-miner](https://github.com/yourusername/rust-rule-miner) for automatic rule generation:

```rust
// Mine association rules from historical data
let rules = miner.mine_association_rules()?;

// Export to GRL with += syntax
let grl = GrlExporter::to_grl(&rules);
// Generates: Recommendation.items += "Phone Case";

// Load and execute in RETE engine
GrlReteLoader::load_from_string(&grl, &mut engine)?;
```

**Supported Everywhere:**
- ✅ Forward chaining (RETE engine)
- ✅ Backward chaining (goal-driven reasoning)
- ✅ Parallel execution
- ✅ All action execution contexts

---



## 📚 Documentation

Comprehensive documentation organized by topic:

### 🚀 [Getting Started](docs/getting-started/)
- **[Quick Start](docs/getting-started/QUICK_START.md)** - Get up and running in 5 minutes
- **[Installation](docs/getting-started/INSTALLATION.md)** - Installation and setup guide
- **[Basic Concepts](docs/getting-started/CONCEPTS.md)** - Core concepts explained
- **[First Rules](docs/getting-started/FIRST_RULES.md)** - Write your first rules

### 🎯 [Core Features](docs/core-features/)
- **[GRL Syntax](docs/core-features/GRL_SYNTAX.md)** - Grule Rule Language reference
- **[Features Overview](docs/core-features/FEATURES.md)** - All engine capabilities

### ⚡ [Advanced Features](docs/advanced-features/)
- **[RETE Optimization](docs/advanced-features/RETE_OPTIMIZATION.md)** - 1,235x join speedup & memory optimizations (v1.13.0+)
- **[RETE Benchmarks](docs/advanced-features/RETE_OPTIMIZATION_BENCHMARKS.md)** - Real performance data & analysis (v1.13.0+)
- **[Streaming & CEP](docs/advanced-features/STREAMING.md)** - Complex Event Processing
- **[Streaming Architecture](docs/advanced-features/STREAMING_ARCHITECTURE.md)** - Deep dive into streaming
- **[Plugins](docs/advanced-features/PLUGINS.md)** - Custom plugins and extensions
- **[Performance](docs/advanced-features/PERFORMANCE.md)** - Optimization techniques
- **[Redis State](docs/advanced-features/REDIS_STATE_BACKEND.md)** - Distributed state management

### 📖 [API Reference](docs/api-reference/)
- **[API Reference](docs/api-reference/API_REFERENCE.md)** - Complete public API
- **[GRL Query Syntax](docs/api-reference/GRL_QUERY_SYNTAX.md)** - Backward chaining queries (v1.11.0+)
- **[Parser Cheat Sheet](docs/api-reference/PARSER_CHEAT_SHEET.md)** - Quick syntax reference

### 📝 [Guides](docs/guides/)
- **[Backward Chaining Quick Start](docs/BACKWARD_CHAINING_QUICK_START.md)** - Goal-driven reasoning
- **[RETE Integration](docs/guides/BACKWARD_CHAINING_RETE_INTEGRATION.md)** - Combine forward + backward
- **[Module Management](docs/guides/MODULE_PARSING_GUIDE.md)** - Organize rules into modules
- **[Troubleshooting](docs/guides/TROUBLESHOOTING.md)** - Common issues and solutions

### 💡 [Examples](docs/examples/)
- **[AI Integration](docs/examples/AI_INTEGRATION.md)** - Combine with ML models

**[📚 Full Documentation Index →](docs/README.md)**


---

## 📜 Older Releases

See [CHANGELOG.md](CHANGELOG.md) for full version history (v0.1.0 - v0.19.0).


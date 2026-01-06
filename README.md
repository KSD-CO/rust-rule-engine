# Rust Rule Engine v1.15.1 🦀⚡🚀

[![Crates.io](https://img.shields.io/crates/v/rust-rule-engine.svg)](https://crates.io/crates/rust-rule-engine)
[![Documentation](https://docs.rs/rust-rule-engine/badge.svg)](https://docs.rs/rust-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/KSD-CO/rust-rule-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-rule-engine/actions)

A blazing-fast production-ready rule engine for Rust supporting **both Forward and Backward Chaining**. Features RETE-UL algorithm with **Alpha Memory Indexing** and **Beta Memory Indexing**, parallel execution, goal-driven reasoning, and GRL (Grule Rule Language) syntax.

🔗 **[GitHub](https://github.com/KSD-CO/rust-rule-engine)** | **[Documentation](https://docs.rs/rust-rule-engine)** | **[Crates.io](https://crates.io/crates/rust-rule-engine)**

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
- **Time Windows** - Sliding (continuous) and tumbling (non-overlapping)
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
use rust_rule_engine::parser::grl::stream_syntax::parse_stream_pattern;
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

## ✨ What's New in v1.15.1 🎉

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

## ✨ What's New in v1.14.0 🎉

⚡ **Alpha Memory Indexing - Up to 800x Faster Queries!**

New hash-based indexing for alpha node fact filtering, complementing Beta Memory Indexing for complete RETE optimization!

### 🔍 Alpha Memory Indexing

**Problem:** Alpha nodes scan all facts linearly to find matches - O(n) complexity becomes slow with large datasets.

**Solution:** Hash-based indexing provides O(1) fact lookups - **up to 800x speedup** for filtered queries!

```rust
use rust_rule_engine::rete::{AlphaMemoryIndex, FactValue, TypedFacts};

// Create alpha memory with indexing
let mut mem = AlphaMemoryIndex::new();

// Create index on frequently-queried field
mem.create_index("status".to_string());

// Insert facts (index updated automatically)
for i in 0..10_000 {
    let mut fact = TypedFacts::new();
    fact.set("id", i as i64);
    fact.set("status", if i % 100 == 0 { "active" } else { "pending" });
    mem.insert(fact);
}

// Query using index - O(1) lookup!
let active = mem.filter("status", &FactValue::String("active".to_string()));
println!("Found {} active facts", active.len());
// Without index: 10,000 comparisons (O(n))
// With index: 1 hash lookup (O(1)) → ~800x faster!
```

**Real Benchmark Results:**

| Dataset Size | Linear Scan | Indexed Lookup | Speedup |
|--------------|-------------|----------------|---------|
| 1,000 facts  | 310 µs      | 396 ns         | **782x** |
| 10,000 facts | 3.18 ms     | 396 ns         | **8,030x** |
| 50,000 facts | 15.9 ms     | 396 ns         | **40,151x** 🚀 |

**Key Features:**
- ✅ **Auto-tuning** - Automatically creates indexes after 50+ queries on a field
- ✅ **Multiple indexes** - Index different fields independently
- ✅ **Statistics tracking** - Monitor index hit rates and effectiveness
- ✅ **Low overhead** - ~7-9% memory per index

**When to Use:**
```rust
// ✅ Use when:
// - Dataset > 10K facts
// - Read-heavy workload (query > insert)
// - High selectivity queries (<10% match rate)
// - Same queries repeated multiple times

// ❌ Skip when:
// - Dataset < 1K facts (overhead > benefit)
// - Write-heavy workload (insert > query)
// - Query each field only once

// 🤖 Auto-tuning mode (recommended):
let mut mem = AlphaMemoryIndex::new();

// Query many times...
for _ in 0..100 {
    mem.filter_tracked("status", &FactValue::String("active".to_string()));
}

// Auto-create index when query count > 50
mem.auto_tune();  // Indexes "status" automatically!
```

**Memory Overhead:**

| Index Count | Memory Usage | Overhead |
|-------------|--------------|----------|
| 0 indexes   | 59.31 MB     | Baseline |
| 1 index     | 60.32 MB     | +1.7%    |
| 3 indexes   | 72.15 MB     | +21.6%   |
| 5 indexes   | 85.67 MB     | +44.4%   |

**Recommendation:** Use 1-3 indexes max (~20% overhead) for best ROI.

---

## ✨ What's New in v1.13.0

⚡ **Beta Memory Indexing - Up to 1,235x Faster Joins!**

Comprehensive RETE optimization system with **Beta Memory Indexing** providing exponential speedup for multi-pattern rules!

### 🚀 Beta Memory Indexing

**Problem:** Join operations use nested loops (O(n²)) which becomes a bottleneck with large fact sets.

**Solution:** Hash-based indexing changes O(n²) to O(n) - providing **11x to 1,235x speedup!**

```rust
use rust_rule_engine::rete::optimization::BetaMemoryIndex;
use rust_rule_engine::rete::TypedFacts;

// Create sample facts (e.g., orders with customer IDs)
let mut orders = Vec::new();
for i in 0..1000 {
    let mut order = TypedFacts::new();
    order.set("OrderId", format!("O{}", i));
    order.set("CustomerId", format!("C{}", i % 100));  // 100 unique customers
    order.set("Amount", (i * 50) as i64);
    orders.push(order);
}

// Build index on join key (CustomerId)
let mut index = BetaMemoryIndex::new("CustomerId".to_string());
for (idx, order) in orders.iter().enumerate() {
    index.add(order, idx);  // O(1) insertion
}

// Perform O(1) lookup instead of O(n) scan
// Note: Key format is the Debug representation of FactValue
let key = "String(\"C50\")";  // Looking for customer C50's orders
let matches = index.lookup(key);  // O(1) hash lookup!

println!("Found {} orders for customer C50", matches.len());
// Without indexing: 1,000 comparisons (O(n))
// With indexing: 1 hash lookup (O(1)) → 1,000x faster!
```

**Real Benchmark Results:**

| Dataset Size | Nested Loop (O(n²)) | Indexed (O(n)) | Speedup |
|--------------|---------------------|----------------|---------|
| 100 facts    | 1.00 ms             | 92 µs          | **11x** |
| 1,000 facts  | 113.79 ms           | 672.76 µs      | **169x** |
| 5,000 facts  | **2.63 seconds**    | **2.13 ms**    | **1,235x** 🚀 |

**Key Insight:** At 5,000 facts, the difference between 2.6 SECONDS and 2ms is production-critical!

### 🔧 Memory Optimizations

Three additional optimizations focus on reducing memory footprint:

**1. Node Sharing** - Deduplicate identical alpha nodes
```rust
use rust_rule_engine::rete::optimization::NodeSharingRegistry;

let mut registry = NodeSharingRegistry::new();

// Register 10,000 nodes with 100 unique patterns
for (idx, node) in nodes.iter().enumerate() {
    registry.register(node, idx);
}

// Result: 98.1% memory reduction (689.84 KB saved)
let stats = registry.stats();
println!("Memory saved: {:.1}%", stats.memory_saved_percent);
```

**2. Alpha Memory Compaction** - Eliminate duplicate facts
```rust
use rust_rule_engine::rete::optimization::CompactAlphaMemory;

let mut memory = CompactAlphaMemory::new();

// Insert 10,000 facts with duplicates
for fact in facts {
    memory.add(&fact);
}

// Result: 98.7% memory reduction (925.00 KB saved)
println!("Unique facts: {} (saved {:.1}%)",
    memory.len(), memory.memory_savings());
```

**3. Token Pooling** - Reduce allocations
```rust
use rust_rule_engine::rete::optimization::TokenPool;

let mut pool = TokenPool::new(100);

// Process 10,000 events with token reuse
for event in events {
    let mut token = pool.acquire();
    token.set_fact(event);
    // ... process ...
    pool.release(token);
}

// Result: 99% fewer allocations
let stats = pool.stats();
println!("Reuse rate: {:.1}%", stats.reuse_rate);
```

### 📊 When to Use Each Optimization

| Optimization | Always Use? | Use When | Skip When |
|---|---|---|---|
| **Beta Indexing** ⚡ | **YES** | Any join operations | Never (always beneficial) |
| **Alpha Indexing** 🆕 | No | Read-heavy + >10K facts | Write-heavy or <1K facts |
| **Node Sharing** | No | Memory-constrained + 10K+ rules | Speed is priority |
| **Alpha Memory Compaction** | No | Many duplicate facts expected | Few duplicates |
| **Token Pooling** | No | 100K+ events/sec continuous | Batch/low-volume processing |

### 💡 Recommended Usage

**Default (Most Production Systems):**
```rust
// Use Beta + Alpha Indexing for maximum performance
use rust_rule_engine::rete::{AlphaMemoryIndex, BetaMemoryIndex};

// Alpha indexing: for filtering (auto-tune recommended)
let mut alpha_mem = AlphaMemoryIndex::new();
// Will auto-create indexes for frequently-queried fields

// Beta indexing: for joins (always use)
let mut beta_index = BetaMemoryIndex::new("user_id".to_string());
// 150-1,235x faster joins - no downsides!
```

**Memory-Constrained + Large Rule Sets:**
```rust
use rust_rule_engine::rete::optimization::{
    BetaMemoryIndex,      // For speed (always)
    NodeSharingRegistry,  // For memory (if 10K+ rules)
};
```

**High-Duplicate Workloads:**
```rust
use rust_rule_engine::rete::optimization::{
    BetaMemoryIndex,      // For speed (always)
    CompactAlphaMemory,   // For deduplication (if >50% duplicates)
};
```

### 🔬 Try It Yourself

```bash
# Run interactive demos
cargo run --example alpha_indexing_demo          # Alpha Memory Indexing
cargo run --example rete_optimization_demo       # Beta Memory Indexing
cargo run --example grl_optimization_demo        # GRL rules + indexing

# Run benchmarks
cargo bench --bench engine_comparison_benchmark  # Compare all optimizations
cargo bench --bench alpha_indexing_benchmark     # Alpha indexing details
cargo run --bin memory_usage_benchmark --release # Memory analysis


# View detailed HTML reports
open target/criterion/report/index.html
```

### 📚 Complete Documentation

- **[RETE Optimization Guide](docs/advanced-features/RETE_OPTIMIZATION.md)** - Comprehensive optimization guide
- **[Benchmark Results](docs/advanced-features/RETE_OPTIMIZATION_BENCHMARKS.md)** - Real benchmark data & analysis
- **[Optimization Demo](examples/05-performance/rete_optimization_demo.rs)** - Interactive demonstration
- **[GRL + Optimization Demo](examples/05-performance/grl_optimization_demo.rs)** - Real GRL rules with Beta Indexing
- **[Memory Analysis](examples/05-performance/memory_usage_comparison.rs)** - Actual KB/MB measurements with RETE engine

**New in v1.13.0:**
- ✅ Beta Memory Indexing (11x to 1,235x speedup)
- ✅ Node Sharing (98.1% memory reduction)
- ✅ Alpha Memory Compaction (98.7% memory reduction)
- ✅ Token Pooling (99% fewer allocations)
- ✅ Comprehensive benchmarks with scaled datasets
- ✅ Real memory measurements (KB/MB)
- ✅ Production-ready optimization manager
- ✅ 30+ optimization tests

---

## ✨ Previous Update - v1.12.1

🌊 **Stream Processing Foundation!**

**GRL Stream Syntax** - Parse and process real-time event streams with time-based windows!

### 🆕 Stream Processing Features

**GRL Stream Pattern Syntax:**
```rust
// Stream with sliding window
login: LoginEvent from stream("logins") over window(10 min, sliding)

// Stream with tumbling window
metric: MetricEvent from stream("metrics") over window(5 sec, tumbling)

// Simple stream without window
event: Event from stream("events")
```

**StreamAlphaNode - RETE Integration:**
```rust
use rust_rule_engine::parser::grl::stream_syntax::parse_stream_pattern;
use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};

// Parse GRL pattern
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

// Process events
if node.process_event(&event) {
    let handle = working_memory.insert_from_stream("logins".to_string(), event);
    // Event now in RETE network for rule evaluation!
}
```

**Real-World Example - Fraud Detection:**
```rust
// 4 fraud detection rules implemented:
// 1. Suspicious IP changes (multiple IPs in 15 min)
// 2. High velocity purchases (>3 purchases in 15 min)
// 3. Impossible travel (location change too fast)
// 4. IP mismatch (login IP != purchase IP)

// Result: 7 alerts triggered from 16 events
cargo run --example streaming_fraud_detection --features streaming
```

**Features Implemented:**
- ✅ GRL stream syntax parser (nom-based, 15 tests)
- ✅ StreamAlphaNode for event filtering & windowing (10 tests)
- ✅ Sliding windows (continuous rolling)
- ✅ Tumbling windows (non-overlapping)
- ✅ WorkingMemory integration (stream → facts)
- ✅ Duration units: ms, sec, min, hour
- ✅ Optional event type filtering
- ✅ Multi-stream correlation

**Test Coverage:**
- 58 streaming tests (100% pass)
- 8 integration tests (fraud, IoT, trading, security)
- 3 end-to-end tests (GRL → RETE → WorkingMemory)
- 2 comprehensive examples

---

## ✨ Previous Update - v1.11.0

🎯 **Nested Queries & Query Optimization!**

Complete **Phase 1.1** with nested queries (subqueries) and intelligent query optimization for 10-100x performance improvements!

### 🆕 Nested Queries

```rust
use rust_rule_engine::backward::*;

// Find grandparents using nested queries
let results = engine.query(
    "grandparent(?x, ?z) WHERE
        parent(?x, ?y) AND
        (parent(?y, ?z) WHERE child(?z, ?y))",
    &mut facts
)?;

// Complex eligibility with nested OR
query "CheckEligibility" {
    goal: (eligible(?x) WHERE (vip(?x) OR premium(?x))) AND active(?x)
    on-success: { LogMessage("Eligible!"); }
}
```

### ⚡ Query Optimization

```rust
// Enable optimization in GRL
query "OptimizedSearch" {
    goal: item(?x) AND expensive(?x) AND in_stock(?x)
    enable-optimization: true  // Automatically reorders goals!
}

// Manual optimization
let mut optimizer = QueryOptimizer::new();
optimizer.set_selectivity("in_stock(?x)".to_string(), 0.1);   // 10% in stock
optimizer.set_selectivity("expensive(?x)".to_string(), 0.3);  // 30% expensive
optimizer.set_selectivity("item(?x)".to_string(), 0.9);       // 90% items

let optimized = optimizer.optimize_goals(goals);
// Result: in_stock → expensive → item (10-100x faster!)
```

**Performance Benefits:**
- **Before**: 1000 items → 900 expensive → 270 in_stock = 2170 evaluations
- **After**: 10 in_stock → 8 expensive → 8 items = 26 evaluations
- **Speedup**: ~83x faster! 🚀

**New Features:**
- Nested queries with WHERE clauses
- Query optimizer with goal reordering
- Selectivity estimation (heuristic & custom)
- Join order optimization
- `enable-optimization` flag in GRL
- 19 new tests + 9 integration tests

**Testing:** 485/485 tests pass (368 unit + 117 integration) • Zero regressions

📖 **[Nested Query Demo](examples/09-backward-chaining/nested_query_demo.rs)** • **[Optimizer Demo](examples/09-backward-chaining/optimizer_demo.rs)** • **[GRL Integration](examples/09-backward-chaining/grl_optimizer_demo.rs)**

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


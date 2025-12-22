# Rust Rule Engine v1.12.1 🦀⚡🚀

[![Crates.io](https://img.shields.io/crates/v/rust-rule-engine.svg)](https://crates.io/crates/rust-rule-engine)
[![Documentation](https://docs.rs/rust-rule-engine/badge.svg)](https://docs.rs/rust-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/KSD-CO/rust-rule-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-rule-engine/actions)

A blazing-fast production-ready rule engine for Rust supporting **both Forward and Backward Chaining**. Features RETE-UL algorithm, parallel execution, goal-driven reasoning, and GRL (Grule Rule Language) syntax.

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

## ✨ What's New in v1.12.1 🎉

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

## ✨ Previous Updates - v0.19.1

🐛 **Bug Fixes & Improvements**

- **Fixed**: GRL parser attribute matching for `no-loop` and `lock-on-active` keywords
- **Updated**: Example files now use reorganized GRL file paths structure
- **Added**: Missing test files for examples

---

## ✨ What's New in v0.19.0

🚀 **Parallel Rule Engine - Production Ready!** - Multi-threaded execution with full feature parity!

- **🎯 Full Feature Support** - ALL advanced features now work in parallel mode:
  - ✅ Custom function calls (thread-safe with Arc/RwLock)
  - ✅ Pattern matching (exists/forall via PatternMatcher)
  - ✅ Accumulate operations (sum/avg/min/max/count/collect)
  - ✅ MultiField operations (all 7 operations)
  - ✅ Expression evaluation with variable resolution
  - ✅ Nested field access
  - ✅ AND/OR/NOT compound conditions
- **🔄 Smart Parallelization** - Auto-detects when to parallelize based on rule count
- **📊 Benchmarked** - Extensively tested with simple & complex conditions
- **🎯 Zero Limitations** - No restrictions on rule complexity or features
- **🔒 Thread-Safe** - Proper synchronization with Arc/Mutex/RwLock
- **📈 Linear Scaling** - Performance improves with more CPU cores

**When to Use Each Engine:**
- **Native Engine**: Simple rules, low latency requirements, single-threaded environments
- **Parallel Engine**: High-throughput, many rules (100+), multi-core systems, batch processing
- **RETE Engine**: Incremental updates, fact changes, complex pattern matching, state tracking

---

## ✨ What's New in v0.18.1

🔄 **Workflow Orchestration Support** - Build complex multi-stage workflows with rules!

- **🎯 CompleteWorkflow** - Mark workflows as completed with automatic timestamping
- **📊 SetWorkflowData** - Store and track workflow context data as facts
- **🔍 Queryable State** - All workflow state stored in facts, accessible in conditions
- **⏱️ Timestamp Tracking** - Automatic completion time recording (ISO8601 format)

## ✨ What's New in v0.18.0

🧠 **Truth Maintenance System (TMS)** - Intelligent fact dependency tracking!

- **🔗 Justification Tracking** - Track why each fact exists (explicit or derived)
- **⚡ Auto-Retraction** - Derived facts automatically retracted when premises become invalid
- **🌲 Cascade Delete** - Transitively retract all dependent facts
- **💾 Logical Assertions** - Facts derived by rules (vs explicit user assertions)
- **🎯 Production Ready** - Full integration with RETE-UL engine
- **📊 Statistics API** - Monitor TMS state and dependencies

**TMS Example:**
```rust
// Insert explicit fact (user-provided)
let customer_handle = engine.insert_explicit("Customer".to_string(), customer_data);

// Insert logical fact (rule-derived)
let gold_handle = engine.insert_logical(
    "GoldStatus".to_string(),
    gold_data,
    "PromoteToGold".to_string(),
    vec![customer_handle], // Premise: depends on Customer fact
);

// When Customer is retracted, GoldStatus is automatically retracted too!
engine.retract(customer_handle)?; // Cascade: GoldStatus removed automatically
```

**Technical Improvements:**
- **Per-Fact Evaluation**: Rules check each fact separately instead of flattening all facts together
- **Matched Handle Storage**: `Activation` struct now tracks which specific fact matched
- **Handle Injection**: Actions receive the exact handle of the matched fact
- **Validation Check**: Before executing action, verify matched fact still exists
- **ActionResult Architecture**: Proper queuing and processing of action side effects
- **TMS Integration**: Full justification tracking and cascade retraction support

---

## ✨ What's New in v0.17.2

⚡ **30x Parser Optimization** - GRL parsing is now lightning-fast!

- **🚀 30x Speedup** - Parse 15 rules in 5.7ms instead of 171ms
- **💾 Regex Caching** - 15 critical regexes cached with `once_cell::sync::Lazy`
- **🔥 Hot Path Optimized** - All core parsing patterns pre-compiled
- **📊 Consistent Performance** - 176-207 parses/sec (5-6ms per parse)
- **✅ Zero Overhead** - Lazy initialization, no runtime cost after first use
- **🔄 Fully Backward Compatible** - 100% API compatibility, no breaking changes
- **📝 All Tests Pass** - 134 unit tests + 47+ examples verified
- **🎯 Production Ready** - Engine startup time dramatically reduced

**Performance Comparison:**

```
Before v0.17.2:  171,535 µs per parse (5.83 parses/sec) ❌
After v0.17.2:     5,679 µs per parse (176 parses/sec) ✅
Improvement:       30x faster 🚀
```

**Impact on Real Scenarios:**
- **File with 15 rules**: 171ms → 5.7ms ✅
- **File with 100 rules**: ~1.1 sec → ~38ms ✅
- **File with 1000 rules**: ~11 sec → ~380ms ✅
- **Rule hotloading**: Now practical and responsive ✅

**Technical Details:**

The parser was creating fresh regex objects on every parse operation. v0.18.0 implements compile-once, reuse-many pattern:

```rust
// Before: Regex compiled 18+ times per parse ❌
let regex = Regex::new(r#"pattern"#)?;

// After: Regex compiled once, cached forever ✅
static CACHED_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"pattern"#).expect("valid pattern")
});
```

**Coverage:**
- ✅ Core parsing: RULE, RULE_SPLIT, WHEN_THEN, SALIENCE regexes
- ✅ Conditions: TEST, TYPED_TEST, FUNCTION_CALL, CONDITION, SIMPLE_CONDITION
- ✅ Multifields: COLLECT, COUNT, FIRST, LAST, EMPTY, NOT_EMPTY
- ✅ Actions: METHOD_CALL, FUNCTION_BINDING
- ✅ Validation: EMAIL_REGEX caching in plugins

**Benchmark Results:**
```
Test: Quick Parse (100 iterations)
  Average: 5.7 ms per parse
  Throughput: 176 parses/sec ✅

Test: Batch Parsing (5000 iterations)
  Average: 5.0 ms per parse
  Throughput: 200 parses/sec ✅

Test: Memory Stress (10,000 parses)
  Average: 5.3 ms per parse
  Throughput: 188 parses/sec ✅
```

[**📊 Optimization Details →**](OPTIMIZATION_SUMMARY.md) | [**🔬 Technical Analysis →**](PARSER_OPTIMIZATION_REPORT.md)

## ✨ What's New in v0.17.0

🎉 **Multi-field Variables (CLIPS-style Multislot)** - Complete array/collection pattern matching!

- **🔢 9 Operations** - Collect, Contains, Count, First, Last, Index, Slice, IsEmpty, NotEmpty
- **📦 CLIPS Parity** - 90-95% feature compatibility (up from 85-90%)
- **⚡ Both Engines** - Full support in Native Engine and RETE-UL!
- **🛒 E-commerce Ready** - Perfect for shopping carts, bulk orders, inventory
- **🎯 Pattern Matching** - 100% complete (10/10 core features)
- **📝 GRL Syntax** - Natural array operations in rules
- **🚀 Production Ready** - Comprehensive tests and examples

**Example - Multi-field Operations in GRL:**
```grl
rule "BulkDiscount" salience 100 no-loop {
    when
        Order.items count >= 5
    then
        Log("Bulk order detected!");
        Order.discount = 0.15;
}

rule "CategorizeElectronics" salience 90 no-loop {
    when
        Product.tags contains "electronics"
    then
        Log("Electronics product found");
        Product.category = "tech";
}

rule "EmptyCart" salience 80 no-loop {
    when
        ShoppingCart.items empty
    then
        Log("Cart is empty");
        ShoppingCart.status = "empty";
}

rule "ProcessFirstTask" salience 70 no-loop {
    when
        Queue.tasks not_empty &&
        Queue.tasks first $task
    then
        Log("Processing first task...");
        Queue.current = $task;
}
```

**Template Definition (CLIPS-style):**
```rust
use rust_rule_engine::rete::{TemplateBuilder, FieldType};

let order_template = TemplateBuilder::new("Order")
    .multislot_field("items", FieldType::String)  // CLIPS naming
    .float_field("discount")
    .build();
```

**All 9 Multifield Operations:**

| Operation | GRL Syntax | CLIPS Equivalent | Use Case |
|-----------|-----------|------------------|----------|
| **Collect** | `Order.items $?all` | `$?var` | Collect all values |
| **Contains** | `Product.tags contains "sale"` | `(member$ x $?list)` | Check membership |
| **Count** | `Order.items count > 5` | `(length$ $?list)` | Count elements |
| **First** | `Queue.tasks first $task` | `(nth$ 1 $?list)` | Get first element |
| **Last** | `Order.items last $item` | `(nth$ -1 $?list)` | Get last element |
| **Index** | `items[2]` | `(nth$ 3 $?list)` | Access by index |
| **Slice** | `items[1:3]` | `(subseq$ $?list 2 4)` | Extract range |
| **IsEmpty** | `Cart.items empty` | `(= (length$ $?list) 0)` | Check if empty |
| **NotEmpty** | `Queue.tasks not_empty` | `(> (length$ $?list) 0)` | Check if not empty |

[**🎉 Multifield Demo →**](examples/multifield_demo.rs) | [**⚡ RETE Demo →**](examples/rete_multifield_demo.rs) | [**📝 GRL Examples →**](examples/rules/multifield_patterns.grl)

### Previous Updates

## ✨ What's New in v0.16.0

🧮 **CLIPS-Style Expression Evaluation** - Runtime arithmetic expressions in GRL rules!

- **➕ Arithmetic Operations** - Full support for +, -, *, /, % operators
- **📊 Field References** - Use fact fields in expressions (Order.quantity * Order.price)
- **🔗 Chained Expressions** - Values set by one action available to subsequent rules
- **🎯 Type Preservation** - Integer × Integer = Integer; mixed types = Float
- **⚡ Both Engines** - Works perfectly with Native Engine and RETE-UL!
- **🚀 Runtime Evaluation** - Expressions evaluated when rule fires
- **📝 CLIPS Syntax** - Similar to CLIPS (bind ?total (* ?quantity ?price))
- **✅ Production Ready** - Battle-tested with order processing and calculations

**Example - Expression Evaluation in GRL:**
```grl
rule "CalculateOrderTotal" salience 100 no-loop {
    when
        Order.quantity > 0 && Order.price > 0
    then
        Log("Calculating order total...");
        Order.total = Order.quantity * Order.price;
        Order.discount = Order.total * 0.1;
        Order.final = Order.total - Order.discount;
}

rule "CalculateTax" salience 90 no-loop {
    when
        Order.final > 0
    then
        Log("Calculating tax...");
        Order.tax = Order.final * 0.08;
        Order.grandTotal = Order.final + Order.tax;
}
```

**How it Works:**
```rust
// Native Engine
let mut facts = Facts::new();
facts.set("Order.quantity", Value::Integer(10));
facts.set("Order.price", Value::Integer(100));

engine.execute(&mut facts)?;

// Results:
// Order.total = 1000 (10 * 100)
// Order.discount = 100.0 (1000 * 0.1)
// Order.final = 900.0 (1000 - 100)
// Order.tax = 72.0 (900 * 0.08)
// Order.grandTotal = 972.0 (900 + 72)
```

**Similar to Drools DRL:**
- Drools: `$o.total = $o.quantity * $o.price`
- Rust Rule Engine: `Order.total = Order.quantity * Order.price`

[**🧮 Expression Demo →**](examples/expression_demo.rs) | [**📝 GRL Examples →**](examples/rules/expression_demo.grl)

### Previous Updates

## ✨ What's New in v0.15.0

🚀 **Thread-Safe RETE Engine** - Multi-threaded support for Axum & async web services!

- **🔥 Send + Sync** - IncrementalEngine is now Send + Sync for multi-threaded use
- **⚡ Axum Compatible** - Use with `Arc<Mutex<IncrementalEngine>>` in web services
- **🎯 Breaking Change** - Action closures changed from `Box<FnMut>` to `Arc<Fn + Send + Sync>`
- **📝 Migration** - Replace `Box::new(move |facts| ...)` with `Arc::new(move |facts| ...)`

🗑️ **Retract Actions** - CLIPS-style fact retraction!

- **🔥 Retract Facts** - Remove facts from working memory in GRL rules
- **📝 CLIPS Syntax** - `retract($Object)` just like CLIPS
- **🎯 GRL Parser Support** - Parse retract syntax from .grl files
- **🧠 Working Memory** - Mark facts as retracted to prevent future matches
- **🔄 Engine Integration** - Full support in Native, RETE, and Parallel engines
- **✅ Production Ready** - Session cleanup, workflow completion, resource management

**Example - Retract in GRL:**
```grl
rule "CleanupExpiredSession" {
    when
        Session.expired == true
    then
        Log("Session expired, cleaning up...");
        retract($Session);
}

rule "RemoveInvalidUser" {
    when
        User.verified == false
    then
        retract($User);
}
```

**Similar to CLIPS:**
- CLIPS: `(retract ?f)`
- Rust Rule Engine: `retract($Object)`

[**🗑️ Native Engine Demo →**](examples/retract_demo.rs) | [**⚡ RETE Engine Demo →**](examples/retract_demo_rete.rs) | [**📝 GRL Examples →**](examples/rules/retract_demo.grl)

### 🔄 Migration Guide: v0.14.x → v0.15.0

**Breaking Change:** Action closures in RETE engine are now `Arc<Fn + Send + Sync>` instead of `Box<FnMut>`.

**Before (v0.14.x):**
```rust
let rule = TypedReteUlRule {
    name: "MyRule".to_string(),
    node: my_node,
    priority: 0,
    no_loop: true,
    action: Box::new(move |facts: &mut TypedFacts| {
        facts.set("result", true);
    }),
};
```

**After (v0.15.0):**
```rust
let rule = TypedReteUlRule {
    name: "MyRule".to_string(),
    node: my_node,
    priority: 0,
    no_loop: true,
    action: Arc::new(move |facts: &mut TypedFacts| {
        facts.set("result", true);
    }),
};
```

**Why this change?**
- Makes `IncrementalEngine` Send + Sync for use with Axum and async web frameworks
- Enables sharing the engine across threads safely with `Arc<Mutex<IncrementalEngine>>`
- No mutable state needed in actions (facts are passed as `&mut`)

**Note:** If you use `add_rule_with_action()`, no changes needed - the function accepts closures directly.

### Previous Updates

## ✨ What's New in v0.14.1

🗑️ **Retract Actions** - CLIPS-style fact retraction added!

- Retract facts from working memory with `retract($Object)` syntax
- Full GRL parser support for retract in .grl files
- Integration with Native, RETE, and Parallel engines
- Production-ready for session cleanup and workflow completion

## ✨ What's New in v0.14.0

🎉 **MAJOR UPDATE: Fully Automatic Accumulate Functions!**

This release completes the accumulate feature with 100% automatic evaluation across all engine paths!

🧮 **AUTO Accumulate Functions** - Fully automated aggregation in rule conditions!

- **🚀 FULLY AUTOMATIC** - No manual calculation needed!
- **📊 5 Built-in Functions** - sum, count, average, min, max
- **🎯 GRL Parser Support** - Parse `accumulate()` syntax from .grl files
- **⚡ Auto Collection** - Engine automatically collects matching facts
- **🔄 Auto Calculation** - Engine automatically runs aggregate functions
- **💉 Auto Injection** - Engine automatically injects results into facts
- **🎯 RETE Integration** - Efficient aggregation with pattern matching
- **📈 Real-time Analytics** - Calculate metrics across multiple facts
- **💼 Business Rules** - Revenue totals, order counts, averages
- **✅ Production Ready** - Battle-tested with e-commerce analytics

**Example - Just Write This in GRL:**
```grl
rule "HighRevenue" {
    when
        accumulate(Order($amt: amount, status == "completed"), sum($amt))
    then
        Alert.send("High revenue!");
}
```

**Engine does ALL of this automatically:**
1. ✅ Collects all Order facts
2. ✅ Filters by `status == "completed"`
3. ✅ Extracts `amount` field
4. ✅ Runs `sum()` function
5. ✅ Injects result into facts
6. ✅ Evaluates rule condition

[**🚀 AUTO Accumulate (RECOMMENDED) →**](examples/test_auto_accumulate.rs) | [**⚡ Native & RETE-UL Demo →**](examples/test_accumulate_rete_ul.rs) | [**📚 Manual API Demo →**](examples/accumulate_demo.rs) | [**📖 Parser Demo →**](examples/test_accumulate_parser.rs)

⚡ **Variable-to-Variable Comparison** - Dynamic threshold comparisons!

- **🔄 Compare Variables** - Direct comparison between fact fields (e.g., `Facts.L1 > Facts.L1Min`)
- **📊 Dynamic Thresholds** - No hardcoded values, change thresholds on-the-fly
- **🎯 RETE-UL Support** - Full integration with incremental engine
- **📝 GRL Syntax** - Natural syntax: `when (Facts.value > Facts.threshold)`
- **⚡ Efficient Evaluation** - Leverages RETE's pattern matching
- **🔧 Flexible Rules** - Same rule adapts to different threshold configurations
- **✅ Production Ready** - Battle-tested with complex eligibility rules

[**See Variable Comparison Demo →**](examples/famicanxi_rete_test.rs) | [**Test Variable Comparison →**](examples/test_variable_comparison.rs)

### Previous Updates

### v0.13.4
🧮 **Accumulate Functions (Initial Release)** - Aggregation in rule conditions!

- **📊 5 Built-in Functions** - sum, count, average, min, max
- **🎯 GRL Parser Support** - Parse `accumulate()` syntax from .grl files
- **📈 Real-time Analytics** - Calculate metrics across multiple facts
- **⚠️ Note:** Required manual injection in v0.13.4 - now fully automatic in v0.14.0!

⚡ **Variable-to-Variable Comparison** - Dynamic threshold comparisons!

- **🔄 Compare Variables** - Direct comparison between fact fields
- **📊 Dynamic Thresholds** - Change thresholds on-the-fly
- **✅ Production Ready** - Battle-tested

### v0.13.0 (Earlier)
⚡ **Conflict Resolution Strategies** - CLIPS/Drools-inspired rule ordering!

- **🎯 8 Strategies** - Salience, LEX, MEA, Depth, Breadth, Simplicity, Complexity, Random
- **📊 Priority-Based** - Control rule execution order with salience
- **🕐 Recency-Based** - Most recent facts fire first (LEX)
- **🔍 Specificity** - More specific rules fire first (Complexity, MEA)
- **⚙️ Performance** - Simple rules before complex (Simplicity)
- **🔄 Dynamic Switching** - Change strategies at runtime
- **✅ CLIPS Compatible** - Industry-standard conflict resolution
- **📈 ~98% Drools Parity** - Enhanced compatibility

[**See Conflict Resolution Demo →**](examples/conflict_resolution_demo.rs) | [**CLIPS Features Guide →**](CLIPS_INSPIRED_FEATURES.md)

### Previous Updates

### v0.12.0
🧪 **Test CE (Conditional Element)** - CLIPS-inspired arbitrary boolean expressions!

- **🔬 Test CE Syntax** - Call arbitrary functions in rule conditions without operators
- **📝 GRL Support** - Parse `test(function(args))` directly from .grl files
- **🎯 Native Engine** - Fully implemented with function registry
- **⚡ Truthy Evaluation** - Automatic boolean conversion for all value types
- **🔗 Negation Support** - Use `!test()` for negated conditions
- **🤝 Combined Conditions** - Mix test() with regular conditions using AND/OR
- **📚 Multiple Arguments** - Support functions with any number of arguments

[**See Test CE Demo →**](examples/test_ce_comprehensive.rs)

### v0.11.0
🎯 **Deffacts System** - Initial fact definitions (CLIPS feature)!

- **📦 Deffacts** - Pre-defined fact sets for initial state
- **🔄 Reset Support** - Restore original facts with `reset_with_deffacts()`
- **📋 Multiple Sets** - Organize initial facts by category
- **✅ Template Integration** - Type-safe initial facts
- **🏗️ Builder API** - Fluent interface for defining deffacts

[**See Deffacts Demo →**](examples/rete_deffacts_demo.rs)

### v0.10.2
📧 **Metadata Update** - Corrected author email contact information

### v0.10.1

🚀 **RETE Performance Optimization + Comprehensive Benchmarks**!

- **⚡ RETE Fixed** - Eliminated infinite loop issue, now blazing fast
- **📊 Benchmarked** - Comprehensive comparison: Traditional vs RETE
- **🔥 2-24x Faster** - RETE shows 2x speedup at 10 rules, 24x at 50+ rules
- **✅ Production Ready** - Max iterations guard, optimized agenda management
- **📈 Scalability Proven** - ~5µs per rule, scales linearly

[**See Benchmark Results →**](BENCHMARK_RESULTS.md)

### v0.10.0
- **🔧 Function Calls in WHEN** - Call AI/custom functions directly in rule conditions
- **📋 Template System** - Type-safe schema definitions for structured facts
- **🌍 Defglobal** - Global variables with thread-safe access
- **📈 Drools Compatibility** - ~97% Drools parity

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

### Backward Chaining Engine ✅ PRODUCTION READY (v1.1.0)
- **🚀 100-1000x Performance** - O(1) conclusion index vs O(n) linear search
- **🎯 Goal-Driven Reasoning** - Work backwards from goals to prove them (88% complete)
- **🔍 Expression Parser** - Full AST-based boolean logic (<20µs parsing)
- **🧩 Variable Unification** - Pattern matching with conflict detection
- **🔄 Search Strategies** - Depth-first, breadth-first, iterative deepening
- **📊 Proof Traces** - Track reasoning chains and statistics
- **✅ Comprehensive Testing** - 39 unit tests + 15 examples + 9 benchmarks
- **📚 Complete Documentation** - 5 comprehensive guides

### Stream Processing Engine ✅ PRODUCTION READY (v1.4.0)
- **🌊 20+ Stream Operators** - Fluent API for real-time data processing
- **🔑 State Management** - Memory, File, and Redis backends for distributed deployments
- **⏱️ Watermark Support** - Event-time processing with out-of-order handling
- **🪟 Windowing** - Sliding, Tumbling, Session windows for time-based aggregations
- **🚀 High Performance** - 1M+ events/sec (Memory), 100k+ ops/sec (Redis)
- **📊 Built-in Aggregators** - Count, Sum, Average, Min, Max with custom support
- **🔄 Redis Integration** - Distributed state with connection pooling and TTL
- **🎯 Late Data Handling** - Drop, AllowedLateness, SideOutput, RecomputeWindows
- **✅ Comprehensive Testing** - 21 unit tests + 5 comprehensive demos
- **📚 Complete Documentation** - Architecture diagrams and production guides

### RETE-UL Engine (Recommended for 50+ rules)
- **🚀 High Performance** - Efficient RETE algorithm with incremental updates
- **🔥 RETE Algorithm** - Advanced pattern matching with good Drools compatibility
- **🎉 Multi-field Variables** - Array/collection pattern matching with 9 operations *(v0.17.0)*
- **🧮 Expression Evaluation** - Runtime arithmetic expressions (+, -, *, /, %) *(v0.16.0)*
- **🔗 Chained Expressions** - Values from previous rules available to subsequent rules *(v0.16.0)*
- **🧮 Accumulate Functions** - sum, count, average, min, max aggregations *(v0.13.4)*
- **🔄 Variable Comparison** - Compare fact fields dynamically (L1 > L1Min) *(v0.13.4)*
- **🗑️ Retract** - Remove facts from working memory *(v0.14.1)*
- **🔒 Thread-Safe** - Send + Sync for multi-threaded use *(v0.15.0)*
- **📋 Template System** - Type-safe structured facts *(v0.10.0)*
- **🌍 Defglobal** - Global variables across firings *(v0.10.0)*
- **📦 Deffacts** - Initial fact definitions *(v0.11.0)*
- **🧪 Test CE** - Arbitrary boolean expressions in rules *(v0.12.0)*
- **⚡ Conflict Resolution** - 8 CLIPS strategies (Salience, LEX, MEA, etc.) *(v0.13.0)*
- **🧠 Truth Maintenance System (TMS)** - Automatic fact retraction and dependency tracking *(v0.16.0)*
  - **Logical Assertions** - Facts derived by rules are auto-retracted when premises become invalid
  - **Justifications** - Track why facts exist (explicit user input vs. derived by rules)
  - **Cascade Retraction** - Automatically retract dependent facts when base facts are removed
  - **CLIPS-Compatible** - `logicalAssert()` API for derived facts
- **🎯 Incremental Updates** - Only re-evaluate affected rules
- **🧠 Working Memory** - FactHandles with insert/update/retract
- **🔗 Variable Binding** - Cross-pattern $var syntax
- **💾 Memoization** - Efficient caching for repeated evaluations

**Choose Your Engine:**
- **Forward Chaining (data-driven)**:
  - **< 10 rules** → Native Engine (simpler API, plugin support)
  - **10-50 rules** → Either (RETE ~2x faster)
  - **50+ rules** → RETE-UL Engine (2-24x faster, highly recommended)
- **Backward Chaining (goal-driven)** 🆕:
  - **Any rule count** → Backward Engine (100-1000x faster with O(1) index)
  - **Ideal for**: Diagnostics, expert systems, decision trees
  - **Scales to**: 10,000+ rules efficiently
- **Stream Processing (real-time)** 🆕:
  - **Event streams** → Stream Processing Engine (1M+ events/sec)
  - **Ideal for**: IoT monitoring, financial analytics, user behavior tracking
  - **Distributed**: Redis backend for horizontal scaling
  - **Features**: Windowing, watermarking, late data handling
- **Both needs** → Hybrid approach (combine forward + backward + streaming)

📊 **Performance**: RETE shows 2-24x improvement; Backward shows 100-1000x improvement; Streaming handles 1M+ events/sec!

📖 [**Engine Comparison Guide →**](ENGINE_COMPARISON.md) | [**Quick Start Guide →**](QUICK_START_ENGINES.md)

---

## 📦 Installation

```toml
[dependencies]
rust-rule-engine = "1.12.0"
```

### Optional Features
```toml
# Enable backward chaining with negation support (Production Ready! 🚀)
rust-rule-engine = { version = "1.12.0", features = ["backward-chaining"] }

# Enable streaming support (NEW in v1.12.1! 🌊)
rust-rule-engine = { version = "1.12.0", features = ["streaming"] }

# Enable streaming with Redis backend (for distributed deployments)
rust-rule-engine = { version = "1.12.0", features = ["streaming", "streaming-redis"] }

# Enable all features
rust-rule-engine = { version = "1.12.0", features = ["backward-chaining", "streaming", "streaming-redis"] }
```

---

## 🔄 Migrating to v0.18.0

### Breaking Change: Action Closure Signature

v0.18.0 introduces a **breaking change** to fix critical bugs in action execution.

#### Who is Affected?

✅ **GRL Files** - **NOT AFFECTED** - No changes needed!  
❌ **Programmatic Rules** - If you create rules with `TypedReteUlRule`, update your closures.

#### Migration Steps

**Step 1: Add Import**
```rust
use rust_rule_engine::rete::action_result::ActionResults;
```

**Step 2: Update Closure Signature**
```rust
// ❌ Before v0.18.0
let action = Arc::new(|facts: &mut TypedFacts| {
    println!("Rule fired!");
    facts.set("status", "processed");
});

// ✅ After v0.18.0
let action = Arc::new(|facts: &mut TypedFacts, _results: &mut ActionResults| {
    println!("Rule fired!");
    facts.set("status", "processed");
});
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

## 🧮 NEW: Accumulate Functions (v0.13.4)

**Powerful aggregation capabilities for calculating metrics across multiple facts!**

This feature enables you to perform aggregations (sum, count, average, min, max) directly in your rule conditions, making it easy to build analytics and reporting rules.

### ✨ Built-in Accumulate Functions

```rust
// 5 Ready-to-Use Functions
sum()      // Add up numeric values
count()    // Count matching facts
average()  // Calculate mean
min()      // Find minimum value
max()      // Find maximum value
```

### 📖 Real-World Example: Sales Analytics

**Business Scenario:**
E-commerce platform needs to automatically detect high-value sales periods and trigger inventory allocation.

**Rust Implementation:**
```rust
use rust_rule_engine::rete::accumulate::*;
use rust_rule_engine::rete::FactValue;

// Sample order amounts
let orders = vec![
    FactValue::Float(1500.0),
    FactValue::Float(2500.0),
    FactValue::Float(3200.0),
    FactValue::Float(1800.0),
];

// Calculate total revenue
let sum_fn = SumFunction;
let mut state = sum_fn.init();
for amount in &orders {
    state.accumulate(amount);
}

let total = state.get_result(); // Float(9000.0)

// Business rule: If total > $8000, trigger alert
if let FactValue::Float(revenue) = total {
    if revenue > 8000.0 {
        println!("✅ High-value sales period detected!");
        println!("   Recommendation: Allocate extra inventory");
    }
}
```

### 🎯 Future GRL Syntax (Coming Soon)

When integrated with GRL parser, you'll be able to write:

```grl
rule "HighSalesAlert" {
    when
        $total: accumulate(
            Order($amount: amount, status == "completed"),
            sum($amount)
        )
        $total > 8000
    then
        Alert.send("High-value sales period!");
        Inventory.allocate_extra();
}

rule "AverageOrderValue" {
    when
        $avg: accumulate(
            Order($amount: amount),
            average($amount)
        )
        $avg > 1000
    then
        Customer.offerPremiumMembership();
}
```

### 📊 All Accumulate Functions

**1. SUM - Total Revenue**
```rust
let mut sum_state = SumFunction.init();
for order in orders {
    sum_state.accumulate(&order.amount);
}
// Result: Float(total_revenue)
```

**2. COUNT - Number of Orders**
```rust
let mut count_state = CountFunction.init();
for order in orders {
    count_state.accumulate(&order.amount);
}
// Result: Integer(order_count)
```

**3. AVERAGE - Mean Order Value**
```rust
let mut avg_state = AverageFunction.init();
for order in orders {
    avg_state.accumulate(&order.amount);
}
// Result: Float(average_value)
```

**4. MIN - Smallest Order**
```rust
let mut min_state = MinFunction.init();
for order in orders {
    min_state.accumulate(&order.amount);
}
// Result: Float(minimum_value)
```

**5. MAX - Largest Order**
```rust
let mut max_state = MaxFunction.init();
for order in orders {
    max_state.accumulate(&order.amount);
}
// Result: Float(maximum_value)
```

### 🔧 Custom Accumulate Functions

Create your own accumulate functions by implementing the trait:

```rust
use rust_rule_engine::rete::accumulate::*;

// Custom function: Collect all values
pub struct CollectFunction;

impl AccumulateFunction for CollectFunction {
    fn init(&self) -> Box<dyn AccumulateState> {
        Box::new(CollectState { values: Vec::new() })
    }

    fn name(&self) -> &str {
        "collect"
    }

    fn clone_box(&self) -> Box<dyn AccumulateFunction> {
        Box::new(self.clone())
    }
}
```

### 🧪 Complete Examples

See working examples:
- [accumulate_demo.rs](examples/accumulate_demo.rs) - Basic accumulate functions
- [accumulate_rete_integration.rs](examples/accumulate_rete_integration.rs) - E-commerce analytics

---

## 🔄 Variable-to-Variable Comparison (v0.13.4)

**The RETE-UL engine now supports comparing variables directly with each other!**

This powerful feature enables dynamic threshold comparisons without hardcoding values in rules, making your rule logic more flexible and reusable.

### ✨ Why Variable Comparison?

**Traditional Approach (Hardcoded):**
```grl
rule "CheckAge" {
    when customer.age > 18  // Hardcoded threshold
    then customer.eligible = true;
}
```

**New Approach (Dynamic):**
```grl
rule "CheckAge" {
    when customer.age > settings.minAge  // Dynamic threshold
    then customer.eligible = true;
}
```

### 📖 Real-World Example: Product Eligibility

**Business Scenario:**
FamiCanxi product requires customers to meet dynamic thresholds for L1 and CM2 scores that can vary based on market conditions.

**GRL Rule** ([famicanxi_rules.grl](examples/famicanxi_rules.grl)):
```grl
rule "FamiCanxi Product Eligibility Rule" salience 50 {
  when
    (Facts.L1 > Facts.L1Min) &&
    (Facts.CM2 > Facts.Cm2Min) &&
    (Facts.productCode == 1)
  then
    Facts.levelApprove = 1;
}
```

**RETE-UL Implementation:**
```rust
use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine, TypedFacts};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = IncrementalEngine::new();

    // Load rule with variable comparisons
    GrlReteLoader::load_from_file("examples/famicanxi_rules.grl", &mut engine)?;

    // Insert facts with dynamic thresholds
    let mut facts = TypedFacts::new();
    facts.set("L1", 100i64);        // Customer score
    facts.set("L1Min", 50i64);      // Dynamic threshold (can change per request)
    facts.set("CM2", 80i64);        // Customer CM2 score
    facts.set("Cm2Min", 60i64);     // Dynamic threshold
    facts.set("productCode", 1i64);

    engine.insert("Facts".to_string(), facts);
    engine.reset();

    let fired = engine.fire_all();
    println!("Rules fired: {}", fired.len()); // Output: Rules fired: 1

    Ok(())
}
```

### 🎯 Key Benefits

1. **Dynamic Business Rules** - Change thresholds without modifying rule code
2. **A/B Testing** - Test different threshold configurations easily
3. **Multi-Tenant Support** - Different thresholds per customer/region
4. **Configuration-Driven** - Rules adapt to configuration changes
5. **Reduced Code Duplication** - One rule handles multiple scenarios

### 📊 Supported Comparisons

```grl
// Numeric comparisons
Facts.value > Facts.threshold
Facts.value >= Facts.minimum
Facts.value < Facts.maximum
Facts.value <= Facts.limit
Facts.value == Facts.target
Facts.value != Facts.excluded

// Mixed: variable with constant
Facts.value > Facts.threshold && Facts.status == "active"

// Multiple variable comparisons
(Facts.minValue < Facts.value) && (Facts.value < Facts.maxValue)
```

### 🧪 Test Examples

See complete working examples:
- [famicanxi_rete_test.rs](examples/famicanxi_rete_test.rs) - RETE-UL engine with variable comparison
- [famicanxi_grl_test.rs](examples/famicanxi_grl_test.rs) - Standard engine with GRL
- [test_variable_comparison.rs](examples/test_variable_comparison.rs) - Comprehensive test suite

---

## 🧠 Truth Maintenance System (TMS)

**v0.16.0 introduces automatic dependency tracking and cascade retraction!**

The Truth Maintenance System (TMS) automatically tracks why facts exist and removes derived facts when their premises become invalid. This is similar to CLIPS' logical assertions.

### ✨ Why TMS?

**Problem Without TMS:**
```rust
// Rule derives Gold status from high spending
rule "Upgrade to Gold" {
    when Customer.totalSpent > 10000
    then insert(GoldStatus { customerId: Customer.id });
}

// Later, spending drops below threshold
customer.totalSpent = 5000;

// ❌ GoldStatus fact still exists! Manual cleanup needed.
```

**Solution With TMS:**
```rust
// Rule uses logical assertion
rule "Upgrade to Gold" {
    when Customer.totalSpent > 10000
    then logicalAssert(GoldStatus { customerId: Customer.id });
}

// Later, spending drops below threshold
customer.totalSpent = 5000;

// ✅ GoldStatus automatically retracted by TMS!
```

### 🎯 Key Concepts

#### 1. Explicit vs Logical Facts

- **Explicit Facts**: Inserted by user code, persist until manually retracted
  ```rust
  engine.insert("Customer", customer_data);  // Explicit
  ```

- **Logical Facts**: Derived by rules, auto-retracted when premises invalid
  ```rust
  engine.insert_logical("GoldStatus", status, "UpgradeRule", vec![customer_handle]);
  ```

#### 2. Justifications

Each fact has one or more justifications explaining why it exists:
- **Explicit Justification**: "User inserted this fact"
- **Logical Justification**: "Rule X derived this from facts Y and Z"

#### 3. Cascade Retraction

When a premise fact is retracted, all facts logically derived from it are automatically retracted:

```
Customer(id=1, spent=15000) ──┐
                               ├──> GoldStatus(customer=1) ──> FreeShipping(customer=1)
                               │
Rule: "Upgrade to Gold"  ──────┘

// Retract Customer
engine.retract(customer_handle);

// ✅ Automatically retracts:
//    - GoldStatus (derived from Customer)
//    - FreeShipping (derived from GoldStatus)
```

### 📖 Real-World Example: Customer Tier Management

**Business Scenario:**
E-commerce platform automatically manages customer tiers based on spending. When spending changes, tier status should update automatically.

**Implementation:**

```rust
use rust_rule_engine::rete::{IncrementalEngine, GrlReteLoader, TypedFacts, FactValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = IncrementalEngine::new();
    
    // Load rules with logical assertions
    let rules = r#"
        rule "GoldTier" salience 100 {
            when
                Customer.totalSpent > 10000
            then
                Log("Customer qualifies for Gold tier");
                logicalAssert("GoldStatus", Customer.id);
        }
        
        rule "FreeShipping" salience 50 {
            when
                GoldStatus.customerId == Customer.id
            then
                Log("Gold customer gets free shipping");
                logicalAssert("FreeShipping", Customer.id);
        }
    "#;
    
    GrlReteLoader::load_from_string(rules, &mut engine)?;
    
    // Insert customer (explicit fact)
    let mut customer = TypedFacts::new();
    customer.set("id", FactValue::String("CUST-001".to_string()));
    customer.set("totalSpent", FactValue::Float(15000.0));
    let customer_handle = engine.insert("Customer".to_string(), customer);
    
    engine.fire_all();
    
    // ✅ TMS now tracking:
    //    - Customer (explicit)
    //    - GoldStatus (logical, depends on Customer)
    //    - FreeShipping (logical, depends on GoldStatus)
    
    println!("Gold customers: {}", 
        engine.working_memory().get_by_type("GoldStatus").len());  // 1
    
    // Update customer spending below threshold
    let mut updated = TypedFacts::new();
    updated.set("totalSpent", FactValue::Float(5000.0));
    engine.update(customer_handle, updated)?;
    
    engine.fire_all();
    
    // ✅ TMS automatically retracted:
    //    - GoldStatus (premise invalid)
    //    - FreeShipping (cascade from GoldStatus)
    
    println!("Gold customers: {}", 
        engine.working_memory().get_by_type("GoldStatus").len());  // 0
    
    Ok(())
}
```

### 🔍 TMS API

```rust
// Logical assertion (auto-retract when premises invalid)
let handle = engine.insert_logical(
    "GoldStatus".to_string(),
    status_data,
    "UpgradeRule".to_string(),
    vec![customer_handle]  // Premise fact handles
);

// Explicit assertion (manual lifecycle)
let handle = engine.insert_explicit(
    "Customer".to_string(),
    customer_data
);

// Get TMS statistics
let stats = engine.tms().stats();
println!("Logical facts: {}", stats.logical_facts);
println!("Justifications: {}", stats.total_justifications);

// Query justifications for a fact
if let Some(justs) = engine.tms().get_justifications(&handle) {
    for just in justs {
        println!("Justified by rule: {}", just.source_rule);
    }
}
```

### 🎯 Best Practices

1. **Use Logical Assertions for Derived Facts**
   - Facts calculated from other facts should be logical
   - E.g., tier status, discount eligibility, recommendations

2. **Use Explicit Assertions for Base Facts**
   - User input, external data should be explicit
   - E.g., customer profiles, orders, transactions

3. **Track Premises Correctly**
   - Pass all fact handles used in rule's WHEN clause
   - Ensures proper cascade retraction

4. **Monitor TMS Statistics**
   - Check for memory leaks (orphaned justifications)
   - Verify cascade behavior in tests

---

## 🔧 Function Calls in WHEN Clause

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

### 2. Dynamic Eligibility & Thresholds (NEW!)
```grl
// Product eligibility with dynamic thresholds
rule "ProductEligibility" {
    when (customer.score > settings.minScore) &&
         (customer.income > settings.minIncome) &&
         (customer.age >= settings.minAge)
    then customer.eligible = true;
}

// Credit limit based on dynamic risk assessment
rule "CreditLimit" {
    when (customer.creditScore > risk.threshold) &&
         (customer.debtRatio < risk.maxDebtRatio)
    then customer.creditLimit = customer.income * risk.multiplier;
}
```

### 3. Fraud Detection
```rust
// Real-time fraud scoring
rule "HighRiskTransaction" {
    when transaction.amount > 10000 &&
         transaction.location != customer.usual_location
    then fraud.score = 0.85;
}
```

### 4. Workflow Automation
```rust
// Multi-step approval workflows
rule "ManagerApproval" agenda-group "approvals" {
    when request.amount > 5000
    then request.requires_manager = true;
}
```

### 5. Real-Time Systems
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

## Automated GRL Test Harness

This repository includes a lightweight, data-driven test harness used to exercise the GRL examples in `examples/rules` and verify they still parse and run against the engine.

Purpose:

- Provide end-to-end coverage for `.grl` example files without requiring full production action implementations.
- Detect regressions in the parser, engine, and example rules.

Where to find it:

- `tests/grl_harness_data.rs` — the primary data-driven harness. It reads `tests/grl_cases.yml`, constructs `Facts`, loads the `.grl` file(s), builds a `KnowledgeBase`, registers lightweight action handlers and functions, executes the engine, and performs simple assertions.
- `tests/grl_harness.rs` — smaller smoke tests used by the harness and examples.
- `tests/grl_cases.yml` — YAML-driven cases. Each case points at a `.grl` file and provides `initial_facts` and optional `expect` checks.

Why it uses minimal action handlers:

Many GRL samples call custom actions (e.g., `apply_discount`, `sendAlert`, `setEcoMode`, etc.). To exercise the rules end-to-end without requiring external systems, the harness registers small, no-op or fact-mutating action handlers. These handlers are only for testing and live in `tests/grl_harness_data.rs`.

How to run the harness (local development / CI):

```bash
# from repository root (zsh)
cargo test --tests -- --nocapture
```

What to look for:

- The harness prints a per-case log (e.g., "=== Running case: fraud_detection ===") and a small set of logs generated by the registered handlers and functions.
- Each case prints the number of rules fired. The harness currently performs lightweight assertions (e.g., rules fired, and simple fact field checks) — see `tests/grl_harness_data.rs` for details.

How to add or update cases:

1. Add a new case to `tests/grl_cases.yml` with fields: `name`, `grl`, `initial_facts`, and optional `expect`.
2. If the `.grl` uses custom actions not yet covered, either:
    - Add a small test handler in `tests/grl_harness_data.rs` (follow the existing pattern), or
    - Add sufficient `initial_facts` so rules can be exercised without that action being mandatory.
3. Run the harness and verify the new case behaves as expected.

Notes & next improvements:

- The harness currently registers many minimal handlers to unblock rule execution; a future iteration should replace no-ops with tighter, case-specific assertions so the tests verify meaningful behavior instead of only successful execution.
- There are some compiler warnings in the codebase (missing docs, unused-variable warnings). These do not block tests but can be cleaned up to keep CI logs tidy.

Questions or contributions: If you'd like, I can (a) strengthen per-case assertions, (b) consolidate test handlers into helpers, or (c) add a GitHub Actions workflow to run the harness in CI.

---

## 📄 License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file.

---

## 🙏 Acknowledgments



---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/KSD-CO/rust-rule-engine/issues)
- **Discussions**: [GitHub Discussions](https://github.com/KSD-CO/rust-rule-engine/discussions)
- **Email**: ttvuhm@gmail.com

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



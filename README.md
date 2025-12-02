# Rust Rule Engine v1.5.0 🦀⚡🚀

[![Crates.io](https://img.shields.io/crates/v/rust-rule-engine.svg)](https://crates.io/crates/rust-rule-engine)
[![Documentation](https://docs.rs/rust-rule-engine/badge.svg)](https://docs.rs/rust-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/KSD-CO/rust-rule-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-rule-engine/actions)

A high-performance rule engine for Rust with **RETE-UL algorithm**, **Parallel Execution**, **Real-Time Stream Processing**, **Distributed State with Redis**, **CLIPS-inspired Module System with Cyclic Import Detection**, **Production-Ready Backward Chaining**, **Enhanced Null Handling**, **Plugin System**, and **GRL (Grule Rule Language) support**. Designed for production use with excellent performance and Drools compatibility.

🔗 **[GitHub](https://github.com/KSD-CO/rust-rule-engine)** | **[Documentation](https://docs.rs/rust-rule-engine)** | **[Crates.io](https://crates.io/crates/rust-rule-engine)**

---

## ✨ What's New in v1.5.0 🎉

🚀 **Enhanced Null Handling & Business Logic Integration!**

This release adds **robust null checking support** and demonstrates production-ready integration patterns with real business logic. Process complex business rules with confidence using inline GRL strings and comprehensive null handling.

### 🎉 Major Features:

✅ **Null Value Handling** (NEW!)
- ✅ Missing fields treated as `Value::Null` (not false)
- ✅ Support for `field == null` and `field != null` conditions
- ✅ Special null handling in Equal/NotEqual operators
- ✅ Consistent null behavior across HashMap-based evaluation
- ✅ Default fallback rule patterns with null checks
- ✅ 3 files enhanced: engine.rs, types.rs, rule.rs
- ✅ 100% backward compatible with existing rules

✅ **Business Logic Examples** (NEW!)
- ✅ Invoice creation with 5 GRL rule sets (24 rules total)
- ✅ Complex discount strategy selection via rules
- ✅ 17 comprehensive tests (100% passing)
- ✅ Real-world patterns: eligibility, FPT push, discount calculation
- ✅ Inline GRL strings for maintainable business logic
- ✅ Demonstrates production-ready integration patterns

✅ **Streaming Examples Enhanced** (NEW!)
- ✅ State management with rule engine integration
- ✅ Watermark processing with business rules
- ✅ Basic streaming with GRL rule evaluation
- ✅ Clear demonstration of rule engine usage in streams
- ✅ Production-ready patterns for real-time processing

### 📊 Quality & Testing:
- **Test Coverage**: 155+ tests passing (100% success rate)
- **Library Tests**: 133/133 ✅ (core engine, GRL parser, RETE)
- **Integration Tests**: 5/5 ✅ (end-to-end scenarios)
- **Business Logic Tests**: 17/17 ✅ (invoice rules, discount strategy)
- **Examples**: 52+ ✅ (all categories validated)
- **Regressions**: 0 (comprehensive validation completed)
- **Deployment Status**: 🟢 Production Ready

### 🔧 Key Improvements:
- **Null Checking**: Robust handling of undefined/null fields in conditions
- **Default Fallbacks**: Reliable fallback rule patterns when primary conditions don't match
- **Business Integration**: Production-ready patterns for complex business logic
- **Test Infrastructure**: Comprehensive test suite with 17 business logic tests
- **Examples**: Clear demonstrations of rule engine integration in real scenarios

---

## 📋 Version History

### v1.5.0 (Current) - Null Handling & Business Integration Release
- ✅ **Null Value Handling** - Missing fields treated as Value::Null
- ✅ **Null Checking Conditions** - Support for `field == null` patterns
- ✅ **Default Fallback Rules** - Reliable fallback patterns with null checks
- ✅ **Business Logic Examples** - 24 production-ready GRL rules
- ✅ **Discount Strategy Rules** - Complex strategy selection via rules
- ✅ **Streaming Examples** - 3 demos with clear rule engine usage
- ✅ **17 business tests** - 100% passing with comprehensive coverage
- ✅ **155+ total tests** - Zero regressions, production ready

### v1.4.0 - Stream Processing Release
- ✅ **Stream Operators** - 20+ fluent operators with aggregations
- ✅ **Watermarking** - Out-of-order event handling
- ✅ **State Management** - Distributed state with Redis backend
- ✅ **Windowing** - Sliding, Tumbling, Session windows
- ✅ **5 comprehensive demos** - IoT monitoring, user analytics, etc.
- ✅ **Full documentation** - Architecture diagrams and guides
- ✅ **21 unit tests** - All passing



### v1.3.0
- ✅ **Cyclic Import Detection** - Prevents circular module dependencies
- ✅ **BFS-based cycle detection** - O(V + E) performance
- ✅ **Self-import prevention** - Detects A → A patterns
- ✅ **Clear error messages** - Shows cycle paths
- ✅ **100% backward compatible** - No breaking changes
- ✅ **13 comprehensive tests** - All passing

### 🎉 Major Milestones:

✅ **Complete Backward Chaining System** (88% → Production Ready!)
- ✅ All Phase 1 tasks 100% complete
- ✅ Phase 2 testing & docs 92% complete
- ✅ Phase 3 optimization 65% complete
- ✅ **100-1000x proven speedup** with O(1) Conclusion Index
- ✅ Scales to 10,000+ rules efficiently

✅ **Module System with Cyclic Detection** (NEW in v1.3.0!)
- ✅ Cyclic import detection with BFS algorithm
- ✅ Prevents self-imports and circular dependencies
- ✅ Clear error messages with cycle paths
- ✅ <1ms performance for 100 modules
- ✅ CLIPS-inspired module system with full support
- ✅ GRL parser with defmodule directives
- ✅ Import/export control with visibility rules
- ✅ Module-aware rule focusing
- ✅ 100% backward compatible (all 85 examples work unchanged)
- ✅ Automatic module assignment via backward search
- ✅ Performance: <1ms parsing for typical files

✅ **Comprehensive Testing** (52 unit tests + 20 examples)
- ✅ 21 expression parser tests
- ✅ 10 conclusion index tests
- ✅ 8 unification tests
- ✅ 13 cyclic detection tests (NEW!)
- ✅ 20 working examples (15 demos + 5 test suites)
- ✅ **All tests passing**

✅ **Performance Benchmarks** (9 Criterion groups)
- ✅ Expression parsing: <20µs
- ✅ Index lookup: ~200ns (O(1) constant time)
- ✅ Cycle detection: <1ms for 100 modules
- ✅ Query execution: <10ms for 100 rules
- ✅ **Proven 100-1000x speedup** 🔥

✅ **Complete Documentation** (6 comprehensive guides)
- ✅ Quick Start Guide (5-minute getting started)
- ✅ Troubleshooting Guide (comprehensive FAQ)
- ✅ Performance Analysis (detailed benchmarks)
- ✅ Cyclic Import Detection (NEW! cycle prevention guide)
- ✅ Beta Release Summary (migration guide)
- ✅ Implementation Plan (technical details)

### 🔧 What's Ready for Production:

✅ **Module System** - Organize large rule bases (NEW!)
✅ **RETE-style conclusion index** - O(1) rule lookup
✅ **Unification system** - Variable bindings & pattern matching
✅ **Core backward chaining engine** - Goal-driven reasoning
✅ **All 3 search strategies** - DFS, BFS, Iterative Deepening
✅ **Complex condition evaluation** - AND, OR, NOT, EXISTS, FORALL
✅ **Safety mechanisms** - Cycle detection, depth limits
✅ **GRL query syntax** - Declarative queries with actions
✅ **TMS integration** - Logical facts with cascade retraction
✅ **Rollback system** - Speculative changes with undo
✅ **Missing facts analysis** - What's needed to prove goals
✅ **Proof traces** - Explanation of reasoning chains
✅ **Performance benchmarks** - Comprehensive benchmark suite


### 📋 Production Recommendations:

**Safe configurations:**
```rust
let config = BackwardConfig {
    max_depth: 20,                         // Set reasonable limit
    generate_proof_trace: true,            // Enable explanations
    search_strategy: SearchStrategy::DepthFirst,
    ..Default::default()
};
```

**Supported use cases:**
- ✅ Diagnostic systems (medical, technical troubleshooting)
- ✅ Access control & approval flows
- ✅ Compliance checking & validation
- ✅ Question answering (yes/no queries)
- ✅ Missing facts detection
- ✅ Expert systems with goal-driven reasoning
- ✅ Financial decision making (loan approvals, credit checks)
- ✅ Product recommendations & AI systems

**Documentation:**
- **[Module System Guide](docs/GRL_SYNTAX.md#module-system)** - Module organization & best practices 🆕
- **[Module Parsing Guide](docs/MODULE_PARSING_GUIDE.md)** - Parser internals & algorithms 🆕
- **[Parser Examples](docs/MODULE_PARSING_EXAMPLES.md)** - Real-world module examples 🆕
- **[Quick Start Guide](docs/BACKWARD_CHAINING_QUICK_START.md)** - 5-minute getting started
- **[Troubleshooting Guide](docs/BACKWARD_CHAINING_TROUBLESHOOTING.md)** - Common issues & FAQ
- **[Performance Analysis](.planning/BACKWARD_CHAINING_PERFORMANCE.md)** - Benchmark results
- **[Architecture Overview](BACKWARD_CHAINING_ARCHITECTURE.md)** - Technical details
- **[Full Changelog](.planning/CHANGELOG_v1.1.0.md)** - Complete v1.1.0 changes

**Module System Example:**

```grl
; Define modules with export/import control
defmodule SENSORS {
  export: all
}

defmodule CONTROL {
  import: SENSORS (rules * (templates *))
  export: all
}

; Module context for SENSORS rules
;; MODULE: SENSORS
rule "CheckHighTemperature" {
    when: Temperature > 30
    then: ActivateCooling();
}

; Module context for CONTROL rules  
;; MODULE: CONTROL
rule "ActivateCooling" {
    when: CoolingNeeded == true
    then: System.Cooling = "ON";
}
```

**Module System Usage:**

```rust
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::engine::module::ModuleManager;

// Parse GRL with module support
let grl_content = std::fs::read_to_string("smart_home.grl")?;
let parsed = GRLParser::parse_with_modules(&grl_content)?;

// Access parsed rules and module information
println!("Modules: {:?}", parsed.module_manager.list_modules());
println!("Rule -> Module mapping: {:?}", parsed.rule_modules);

// Load rules into engine with module context
let mut engine = RuleEngine::new();
engine.load_rules(parsed.rules);
engine.execute();
```

**Backward Chaining Example:**

```rust
use rust_rule_engine::backward::{BackwardEngine, GRLQueryParser, GRLQueryExecutor};

// Load rules from GRL file
let rules = load_rules_from_file("approval_rules.grl");
let mut kb = KnowledgeBase::new("Approval");
for rule in GRLParser::parse_rules(&rules)? {
    kb.add_rule(rule)?;
}

// Load query definition
let query_str = load_query_from_file("queries.grl", "CheckAutoApproval");

// Set up facts
let mut facts = Facts::new();
facts.set("Customer.LoyaltyPoints", Value::Number(150.0));
facts.set("Order.Amount", Value::Number(5000.0));

// Execute backward chaining query
let query = GRLQueryParser::parse(&query_str)?;
let mut bc_engine = BackwardEngine::new(kb);
let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts)?;

if result.provable {
    println!("✅ Goal proven! Order can be auto-approved");
} else {
    println!("⏳ Manual review required");
}
```

**GRL Query Syntax:**

```grl
query "CheckAutoApproval" {
    goal: Order.AutoApproved == true && Order.RequiresManualReview != true
    strategy: depth-first
    max-depth: 10
    
    on-success: {
        Order.Status = "APPROVED";
        Order.ProcessingTime = "Instant";
        LogMessage("✅ Order auto-approved");
    }
    
    on-failure: {
        Order.Status = "PENDING_REVIEW";
        Order.ProcessingTime = "1-2 business days";
        LogMessage("⏳ Manual review needed");
    }
}
```

**Use Cases:**
- **Medical Diagnosis** - Work backwards from symptoms to identify diseases
- **E-commerce Approval** - Determine if orders should be auto-approved
- **Detective Systems** - Solve crimes by proving hypotheses from evidence
- **Decision Trees** - Classification and recommendation engines
- **Expert Systems** - Knowledge-based reasoning and inference

---

**Stream Processing Example:**

```rust
use rust_rule_engine::streaming::{DataStream, StateBackend};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SensorReading {
    sensor_id: String,
    temperature: f64,
    timestamp: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create stream with Redis backend for distributed state
    let stream = DataStream::new(StateBackend::redis("redis://localhost:6379").await?);
    
    // Process high-temperature alerts with windowing
    stream
        .filter(|reading: &SensorReading| reading.temperature > 80.0)
        .key_by(|reading| reading.sensor_id.clone())
        .window_tumbling(Duration::from_secs(60))
        .aggregate(|readings: Vec<SensorReading>| {
            readings.iter().map(|r| r.temperature).sum::<f64>() / readings.len() as f64
        })
        .for_each(|(sensor, avg_temp)| {
            println!("⚠️ Sensor {} avg temp: {:.1}°C", sensor, avg_temp);
        })
        .await?;
    
    Ok(())
}
```

**Watermark & Late Data Example:**

```rust
use rust_rule_engine::streaming::{
    WatermarkGenerator, WatermarkStrategy, LateDataHandler, LateDataStrategy
};

// Create watermark generator for out-of-order events
let watermark_gen = WatermarkGenerator::bounded_out_of_order(
    Duration::from_secs(10), // Max 10 seconds out of order
    |event: &SensorReading| event.timestamp,
);

// Handle late data with 30-second grace period
let late_handler = LateDataHandler::new(
    LateDataStrategy::AllowedLateness(Duration::from_secs(30)),
);

// Process with event-time semantics
let watermarked = stream.with_watermark(watermark_gen, late_handler);
```

**Redis State Backend Example:**

```rust
use rust_rule_engine::streaming::StateBackend;

// Single instance with local state
let memory_backend = StateBackend::memory();

// Distributed deployment with Redis
let redis_backend = StateBackend::redis("redis://localhost:6379").await?;

// File-based persistence
let file_backend = StateBackend::file("./state_checkpoints")?;

// Switch backends without code changes
let stream = DataStream::new(redis_backend);
```

**Stream Processing Use Cases:**
- **IoT Monitoring** - Real-time sensor data processing with alerting
- **Financial Analytics** - Trade monitoring, fraud detection, risk scoring
- **User Behavior Tracking** - Session analysis, engagement metrics
- **System Monitoring** - Log aggregation, performance metrics, anomaly detection
- **E-commerce** - Real-time inventory, shopping cart analytics

**Performance:**
- **Memory Backend**: 1M+ events/sec, <1μs latency
- **File Backend**: 100k+ events/sec, <10μs latency
- **Redis Backend**: 100k+ ops/sec, <1ms latency
- **Horizontal Scaling**: Linear scalability with Redis Cluster

**Documentation:**
- **[Architecture Guide](docs/STREAMING_ARCHITECTURE.md)** - Complete architecture with diagrams 🆕
- **[Redis Backend Guide](docs/REDIS_STATE_BACKEND.md)** - Distributed state setup 🆕
- **[Streaming Guide](docs/STREAMING.md)** - Comprehensive streaming features

**Examples (5 comprehensive demos):**
- [Stream Operators Demo](examples/03-advanced-features/stream_operators_demo.rs) - 7 operator scenarios
- [State Management Demo](examples/03-advanced-features/state_management_demo.rs) - 6 state scenarios
- [Watermark Demo](examples/03-advanced-features/watermark_demo.rs) - 4 watermark scenarios
- [Redis State Demo](examples/03-advanced-features/redis_state_demo.rs) - 4 distributed scenarios
- [IoT Monitoring Demo](examples/06-use-cases/iot_monitoring_demo.rs) - Production IoT example

---

**Examples (16 working examples):**

*Demo Applications (12):*
- [Simple Query Demo](examples/09-backward-chaining/simple_query_demo.rs) - Basic backward chaining
- [RETE Index Demo](examples/09-backward-chaining/rete_index_demo.rs) - O(1) performance showcase 🔥
- [Multiple Solutions Demo](examples/09-backward-chaining/multiple_solutions_demo.rs) - Find all proof paths (GRL-based) 🆕
- [Medical Diagnosis](examples/09-backward-chaining/medical_diagnosis_demo.rs) - Disease diagnosis system
- [E-commerce Approval](examples/09-backward-chaining/ecommerce_approval_demo.rs) - Order approval workflow
- [Detective System](examples/09-backward-chaining/detective_system_demo.rs) - Crime-solving inference
- [Loan Approval](examples/09-backward-chaining/loan_approval_demo.rs) - Financial decisions (29 rules)
- [Family Relations](examples/09-backward-chaining/family_relations_demo.rs) - Relationship inference (21 rules)
- [Access Control](examples/09-backward-chaining/access_control_demo.rs) - RBAC permissions (26 rules)
- [Product Recommendations](examples/09-backward-chaining/product_recommendation_demo.rs) - AI recommendations (30 rules)
- [GRL Query Demo](examples/09-backward-chaining/grl_query_demo.rs) - Query language features
- [Unification Demo](examples/09-backward-chaining/unification_demo.rs) - Variable bindings & pattern matching

*Test Suites (4):*
- [Comprehensive Test](examples/09-backward-chaining/comprehensive_backward_test.rs) - 12 feature tests
- [Edge Cases Test](examples/09-backward-chaining/backward_edge_cases_test.rs) - 8 correctness tests
- [Critical Tests](examples/09-backward-chaining/backward_critical_missing_tests.rs) - 10 safety tests
- [Unit Tests](tests/backward_comprehensive_tests.rs) - 44 unit tests (21 parser + 10 index + 8 unification + 5 multiple solutions) 🆕

**Technical Features:**
- **O(1) Conclusion Index** - HashMap-based rule lookup (100-1000x speedup) ✅ 🆕
- **Expression AST** - Full boolean logic parsing (&&, ||, !, ==, !=, <, >, <=, >=) ✅
- **Unification System** - Variable bindings & pattern matching ✅ 🆕
- **Search Strategies** - DFS, BFS, Iterative Deepening ✅
- **Memoization** - Automatic caching of proven goals ✅
- **Cycle Detection** - Prevent infinite loops in recursive proofs ✅
- **Proof Traces** - Full explanation of reasoning chains ✅
- **Query Statistics** - Goals explored, rules evaluated, execution time ✅
- **Rule Executor** - Shared condition/action evaluation ✅
- **Rollback System** - Undo frames for speculative changes ✅

**Production Ready Status (88% Complete):**
- ✅ **Phase 1 (100%)**: Core features complete
  - Expression parser (21 tests)
  - Conclusion index (10 tests)
  - Unification (8 tests)
  - Rule execution
- ✅ **Phase 2 (92%)**: Quality & testing
  - 44 unit tests + 15 examples
  - 9 Criterion benchmark groups
  - 5 comprehensive documentation guides
- ✅ **Phase 3 (65%)**: Optimization
  - O(1) indexing proven (100-1000x speedup)
  - Performance profiling complete

**Production Recommendations:**
- ✅ **PRODUCTION READY**: Single-threaded use with all core features
- ✅ **PRODUCTION READY**: Diagnostic systems, decision making, expert systems
- ✅ **PRODUCTION READY**: Up to 10,000+ rules with excellent performance
- ✅ **STABLE API**: All core APIs finalized and documented

[**🚀 Quick Start Guide →**](docs/BACKWARD_CHAINING_QUICK_START.md) | [**📊 Performance Analysis →**](.planning/BACKWARD_CHAINING_PERFORMANCE.md) | [**📝 Examples →**](examples/09-backward-chaining/)

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
rust-rule-engine = "1.4.0"
```

### Optional Features
```toml
# Enable backward chaining (Production Ready! 🚀)
rust-rule-engine = { version = "1.4.0", features = ["backward-chaining"] }

# Enable streaming support
rust-rule-engine = { version = "1.4.0", features = ["streaming"] }

# Enable streaming with Redis backend (for distributed deployments)
rust-rule-engine = { version = "1.4.0", features = ["streaming", "streaming-redis"] }

# Enable all features
rust-rule-engine = { version = "1.4.0", features = ["backward-chaining", "streaming", "streaming-redis"] }
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



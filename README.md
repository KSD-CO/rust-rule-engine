# Rust Rule Engine v0.14.1 🦀⚡

[![Crates.io](https://img.shields.io/crates/v/rust-rule-engine.svg)](https://crates.io/crates/rust-rule-engine)
[![Documentation](https://docs.rs/rust-rule-engine/badge.svg)](https://docs.rs/rust-rule-engine)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/KSD-CO/rust-rule-engine/actions/workflows/rust.yml/badge.svg)](https://github.com/KSD-CO/rust-rule-engine/actions)

A high-performance rule engine for Rust with **RETE-UL algorithm**, **CLIPS-inspired features**, **Plugin System**, and **GRL (Grule Rule Language) support**. Designed for production use with good Drools compatibility.

🔗 **[GitHub](https://github.com/KSD-CO/rust-rule-engine)** | **[Documentation](https://docs.rs/rust-rule-engine)** | **[Crates.io](https://crates.io/crates/rust-rule-engine)**

---

## ✨ What's New in v0.14.1

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

### Previous Updates

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

### RETE-UL Engine (Recommended for 50+ rules)
- **🚀 High Performance** - Efficient RETE algorithm with incremental updates
- **🔥 RETE Algorithm** - Advanced pattern matching with good Drools compatibility
- **🧮 Accumulate Functions** - sum, count, average, min, max aggregations *(v0.13.4)*
- **🔄 Variable Comparison** - Compare fact fields dynamically (L1 > L1Min) *(v0.13.4)*
- **🗑️ Retract** - Remove facts from working memory *(v0.14.1)*
- **📋 Template System** - Type-safe structured facts *(v0.10.0)*
- **🌍 Defglobal** - Global variables across firings *(v0.10.0)*
- **📦 Deffacts** - Initial fact definitions *(v0.11.0)*
- **🧪 Test CE** - Arbitrary boolean expressions in rules *(v0.12.0)*
- **⚡ Conflict Resolution** - 8 CLIPS strategies (Salience, LEX, MEA, etc.) *(v0.13.0)*
- **🎯 Incremental Updates** - Only re-evaluate affected rules
- **🧠 Working Memory** - FactHandles with insert/update/retract
- **🔗 Variable Binding** - Cross-pattern $var syntax
- **💾 Memoization** - Efficient caching for repeated evaluations

**Choose Your Engine:**
- **< 10 rules** → Native Engine (simpler API, plugin support)
- **10-50 rules** → Either (RETE ~2x faster)
- **50+ rules** → RETE-UL Engine (2-24x faster, highly recommended)
- **Both needs** → Hybrid approach

📊 **Performance at 50 rules**: RETE shows good performance improvements over traditional engine!

📖 [**Engine Comparison Guide →**](ENGINE_COMPARISON.md) | [**Quick Start Guide →**](QUICK_START_ENGINES.md)

---

## 📦 Installation

```toml
[dependencies]
rust-rule-engine = "0.14.1"
```

### Optional Features
```toml
# Enable streaming support
rust-rule-engine = { version = "0.14.1", features = ["streaming"] }
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



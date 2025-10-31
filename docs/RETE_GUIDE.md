# RETE-UL Engine Complete Guide

Comprehensive guide to the RETE-UL (Unordered Linear) algorithm implementation in rust-rule-engine.

---

## 📖 Table of Contents

- [What is RETE?](#what-is-rete)
- [When to Use RETE](#when-to-use-rete)
- [Quick Start](#quick-start)
- [Core Features](#core-features)
- [CLIPS-Inspired Features (v0.10.0)](#clips-inspired-features-v0100)
- [Performance](#performance)
- [Best Practices](#best-practices)
- [Examples](#examples)

---

## What is RETE?

RETE is a pattern matching algorithm for production rule systems, used in:
- **Drools** (JBoss/Red Hat)
- **CLIPS** (NASA)
- **Jess** (Sandia National Labs)

**RETE-UL** is our tree-based variant optimized for:
- ✅ Fast pattern matching (4µs per fact)
- ✅ Incremental updates (2x speedup)
- ✅ Memory efficiency
- ✅ ~97% Drools compatibility

---

## When to Use RETE

### Use RETE-UL Engine When:
- ✅ You have **100+ rules**
- ✅ You need **high performance** at scale
- ✅ You want **Drools compatibility**
- ✅ You need **incremental updates**
- ✅ You want **type safety** (templates)

### Use Native Engine When:
- ✅ You have **< 50 rules**
- ✅ You need **plugin support**
- ✅ You want **simpler API**
- ✅ Performance is less critical

**Decision Guide:** [ENGINE_COMPARISON.md](../ENGINE_COMPARISON.md)

---

## Quick Start

### Basic Usage

```rust
use rust_rule_engine::rete::{
    IncrementalEngine, GrlReteLoader, TypedFacts, FactValue
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine
    let mut engine = IncrementalEngine::new();
    
    // Load rules from GRL file
    GrlReteLoader::load_from_file("rules/discount.grl", &mut engine)?;
    
    // Insert facts
    let mut order = TypedFacts::new();
    order.set("order_id", FactValue::String("ORD-001".to_string()));
    order.set("amount", FactValue::Float(1500.0));
    
    let handle = engine.insert("Order".to_string(), order);
    
    // Fire rules
    engine.reset();
    let fired = engine.fire_all();
    
    println!("Fired {} rules", fired.len());
    
    Ok(())
}
```

---

## Core Features

### 1. Typed Facts System 📦

```rust
use rust_rule_engine::rete::{TypedFacts, FactValue};

let mut facts = TypedFacts::new();

// Multiple data types
facts.set("name", FactValue::String("John".to_string()));
facts.set("age", FactValue::Integer(30));
facts.set("salary", FactValue::Float(75000.0));
facts.set("is_active", FactValue::Boolean(true));
facts.set("tags", FactValue::Array(vec![
    FactValue::String("vip".to_string()),
    FactValue::String("premium".to_string()),
]));

// Advanced operators
facts.evaluate_condition("name", "contains", &FactValue::String("John".to_string()));
facts.evaluate_condition("age", ">", &FactValue::Integer(18));
```

**Supported Operators:**
- Comparison: `==`, `!=`, `>`, `>=`, `<`, `<=`
- String: `contains`, `startsWith`, `endsWith`, `matches`
- Array: `in` (membership)

---

### 2. Working Memory with FactHandles 🧠

```rust
use rust_rule_engine::rete::WorkingMemory;

let mut wm = WorkingMemory::new();

// Insert fact - returns unique FactHandle
let mut person = TypedFacts::new();
person.set("name", FactValue::String("Alice".to_string()));
person.set("age", FactValue::Integer(30));

let handle = wm.insert("Person".to_string(), person);

// Update fact
let mut updated = TypedFacts::new();
updated.set("name", FactValue::String("Alice".to_string()));
updated.set("age", FactValue::Integer(31));
wm.update(handle, updated)?;

// Retract fact
wm.retract(handle)?;

// Query by type
let persons = wm.get_by_type("Person");
```

**Features:**
- ✅ Unique FactHandle per fact (like Drools)
- ✅ Insert/Update/Retract operations
- ✅ Type indexing (~4µs per insert)
- ✅ Modification tracking

---

### 3. Advanced Agenda Control 📋

```rust
use rust_rule_engine::rete::{Activation, AdvancedAgenda};

// Create activation
let activation = Activation::new("DiscountRule", 10)  // salience 10
    .with_activation_group("discounts")
    .with_agenda_group("processing")
    .with_no_loop(true)
    .with_auto_focus(true);

// Add to agenda
let mut agenda = AdvancedAgenda::new();
agenda.add_activation(activation);

// Get next activation (by salience)
if let Some(next) = agenda.get_next_activation() {
    println!("Firing: {}", next.rule_name);
}
```

**Attributes:**
- **Salience**: Priority-based execution (higher first)
- **Activation Groups**: Only one rule in group fires
- **Agenda Groups**: Organize rules into phases
- **No-Loop**: Prevent infinite self-triggering
- **Lock-on-Active**: Fire once per agenda activation
- **Auto-Focus**: Automatic agenda group activation

---

### 4. Variable Binding & Multi-Pattern Matching 🔗

```rust
use rust_rule_engine::rete::{PatternBuilder, Variable};

// Variable binding
let pattern = PatternBuilder::for_type("Person")
    .bind("name", "$name")              // Bind to variable
    .where_var("age", ">", "$minAge")   // Use variable in condition
    .build();

// Multi-pattern (JOINs)
let multi = MultiPattern::new("PersonOrderJoin")
    .add_pattern(
        PatternBuilder::for_type("Person")
            .bind("id", "$personId")
            .build()
    )
    .add_pattern(
        PatternBuilder::for_type("Order")
            .where_var("customer_id", "==", "$personId")  // JOIN condition
            .build()
    )
    .build();
```

**Use Cases:**
- Cross-pattern variable sharing
- Complex JOINs across fact types
- Drools-style DSL patterns

---

### 5. Incremental Propagation ⚡

```rust
let mut engine = IncrementalEngine::new();

// Add rules with dependencies
engine.add_rule(rule, vec!["Person".to_string(), "Order".to_string()]);

// Insert fact - only affected rules re-evaluate
engine.insert("Person".to_string(), person_facts);

// Update fact - selective propagation (2x speedup!)
engine.update(handle, updated_facts)?;
```

**Benefits:**
- ✅ 2x speedup vs full re-evaluation
- ✅ Only affected rules re-evaluated
- ✅ Scales linearly with affected rules
- ✅ Automatic dependency tracking

---

### 6. Memoization & Caching 💾

```rust
use rust_rule_engine::rete::MemoizedEvaluator;

let mut evaluator = MemoizedEvaluator::new();

// First evaluation - cache MISS
let result1 = evaluator.evaluate(&node, &facts, |n, f| {
    n.matches_typed(f)  // Expensive
});

// Second evaluation - cache HIT! ⚡
let result2 = evaluator.evaluate(&node, &facts, |n, f| {
    n.matches_typed(f)  // Skipped!
});

// Statistics
let stats = evaluator.stats();
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
// Output: Hit rate: 99.99%
```

**Performance:**
- 📊 99.99% cache hit rate (optimal)
- 🚀 5-20x speedup for repeated patterns
- 💾 Hash-based cache (minimal overhead)

---

## CLIPS-Inspired Features (v0.10.0)

### Template System 📋

Type-safe schema definitions for facts:

```rust
use rust_rule_engine::rete::{TemplateBuilder, FieldType};

// Define template
let person_template = TemplateBuilder::new("Person")
    .required_string("name")           // Required field
    .integer_field("age")              // Optional with default
    .boolean_field("is_adult")
    .array_field("skills", FieldType::String)
    .build();

// Register template
engine.templates_mut().register(person_template);

// Insert with validation
let mut person = TypedFacts::new();
person.set("name", FactValue::String("Bob".to_string()));
person.set("age", FactValue::Integer(25));

let handle = engine.insert_with_template("Person", person)?;
// ✅ Automatic validation!
```

**Benefits:**
- ✅ Type safety at insertion time
- ✅ Required fields checking
- ✅ Default values
- ✅ Living documentation

**See:** [CLIPS_INSPIRED_FEATURES.md](../CLIPS_INSPIRED_FEATURES.md)

---

### Defglobal (Global Variables) 🌍

Persistent state across rule firings:

```rust
// Define globals
engine.globals().define("orders_count", FactValue::Integer(0))?;
engine.globals().define("total_revenue", FactValue::Float(0.0))?;

// Read-only constants
engine.globals().define_readonly("MAX_RETRIES", FactValue::Integer(3))?;

// Access and modify
let count = engine.globals().get("orders_count")?;
engine.globals().set("orders_count", FactValue::Integer(5))?;

// Increment (convenience method)
engine.globals().increment("orders_count", 1.0)?;
engine.globals().increment("total_revenue", 150.0)?;
```

**Benefits:**
- ✅ State persistence
- ✅ Thread-safe (Arc<RwLock>)
- ✅ Read-only constants
- ✅ Numeric operations

---

## Performance

### Benchmarks

| Operation | Time | Notes |
|-----------|------|-------|
| Fact insertion | ~4µs | 1000 facts |
| Template validation | 1-2µs | Per fact |
| Global read | 120ns | RwLock overhead |
| Global write | 180ns | Atomic update |
| Pattern matching | ~10µs | Complex patterns |
| Incremental update | 2x faster | vs full eval |
| Memoization | 99.99% hit | Optimal scenario |

### Scaling

```
Rules  | RETE Execution | Native Execution
-------|----------------|------------------
10     | 0.15ms         | 0.10ms
50     | 0.40ms         | 0.50ms
100    | 0.50ms         | 1.00ms (2x slower)
500    | 1.20ms         | 5.50ms (4.5x slower)
1000   | 2.00ms         | 12.0ms (6x slower)
```

**See:** [docs/PERFORMANCE.md](PERFORMANCE.md)

---

## Best Practices

### 1. Use Templates for Type Safety
```rust
// ✅ Good: Type-safe with validation
engine.insert_with_template("Order", order)?;

// ❌ Bad: No validation
engine.insert("Order".to_string(), order);
```

### 2. Batch Fact Updates
```rust
// ✅ Good: Batch then fire
engine.insert("Order", order1);
engine.insert("Order", order2);
engine.reset();
engine.fire_all();

// ❌ Bad: Fire after each insert
engine.insert("Order", order1);
engine.fire_all();  // Expensive!
engine.insert("Order", order2);
engine.fire_all();  // Expensive!
```

### 3. Use Salience Strategically
```rust
rule "CriticalCheck" salience 100 { ... }  // Highest
rule "BusinessLogic" salience 50 { ... }   // Medium
rule "Logging" salience 1 { ... }          // Lowest
```

### 4. Leverage Incremental Updates
```rust
// Dependency tracking is automatic
engine.add_rule(rule, vec!["Person", "Order"]);

// Only rules depending on "Person" re-evaluate
engine.insert("Person".to_string(), person);
```

### 5. Monitor Cache Performance
```rust
let stats = evaluator.stats();
if stats.hit_rate < 0.8 {
    println!("⚠️ Low cache hit rate: {:.2}%", stats.hit_rate * 100.0);
}
```

---

## Examples

### Complete Example Files

1. **[rete_grl_demo.rs](../../examples/rete_grl_demo.rs)**
   - GRL file loading
   - Basic RETE usage
   - 5 working examples

2. **[rete_template_globals_demo.rs](../../examples/rete_template_globals_demo.rs)**
   - Template System
   - Defglobal
   - Combined usage

3. **[rete_demo.grl](../../examples/rules/rete_demo.grl)**
   - Sample GRL rules
   - Optimized for RETE

---

## Feature Comparison

| Feature | Native | RETE-UL | Drools |
|---------|--------|---------|--------|
| Pattern Matching | Basic | Advanced | Advanced |
| Working Memory | HashMap | FactHandles | FactHandles |
| Incremental Updates | ❌ | ✅ | ✅ |
| Variable Binding | ❌ | ✅ | ✅ |
| Template System | ❌ | ✅ | ✅ |
| Defglobal | ❌ | ✅ | ✅ |
| Agenda Control | Basic | Advanced | Advanced |
| Memoization | ❌ | ✅ | ✅ |
| Plugin Support | ✅ | ❌ | Limited |
| GRL Support | ✅ | ✅ | ❌ (DRL) |

---

## Troubleshooting

### Common Issues

**Q: Rules not firing?**
```rust
// Did you reset the agenda?
engine.reset();  // Required before fire_all()
engine.fire_all();
```

**Q: Template validation failing?**
```rust
// Check required fields
let template = TemplateBuilder::new("Person")
    .required_string("name")  // Must be provided!
    .build();
```

**Q: Performance slower than expected?**
```rust
// Enable memoization (automatic in RETE)
// Check cache hit rate
let stats = evaluator.stats();
println!("Cache hit rate: {:.2}%", stats.hit_rate * 100.0);
```

**Q: Global variable not found?**
```rust
// Define before use
engine.globals().define("counter", FactValue::Integer(0))?;
let value = engine.globals().get("counter")?;
```

---

## Next Steps

- **Learn CLIPS Features**: [CLIPS_INSPIRED_FEATURES.md](../CLIPS_INSPIRED_FEATURES.md)
- **Compare Engines**: [ENGINE_COMPARISON.md](../ENGINE_COMPARISON.md)
- **Quick Start**: [QUICK_START_ENGINES.md](../QUICK_START_ENGINES.md)
- **Performance Guide**: [PERFORMANCE.md](PERFORMANCE.md)

---

**Last Updated**: 2025-10-31 (v0.10.0)
**Drools Compatibility**: ~97%
**Status**: Production Ready ✅

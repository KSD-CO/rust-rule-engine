# Quick Start: Choosing Your Engine

## TL;DR

```rust
// Small/Medium projects with plugins → Native Engine
let mut engine = RustRuleEngine::new(KnowledgeBase::new("MyApp"));

// Large/High-performance projects → RETE-UL Engine
let mut engine = IncrementalEngine::new();
```

---

## Quick Comparison

| Criteria | Native Engine | RETE-UL Engine |
|----------|--------------|----------------|
| **Best For** | Feature-rich apps | High-performance |
| **Rule Count** | < 100 | > 100 |
| **Performance** | Good | Excellent (2-16x) |
| **Plugins** | ✅ Yes | ❌ No |
| **Pattern Matching** | Basic | Advanced |
| **Learning Curve** | Easy | Medium |

---

## Option 1: Native Engine (Recommended for Most Users)

### When to Use
- Getting started with rule engines
- Need plugins (string/math/date utilities)
- Need action handlers (SendEmail, LogToDatabase)
- Want analytics and monitoring
- Have < 100 rules

### Quick Start

```rust
use rust_rule_engine::*;

fn main() -> Result<()> {
    // Create engine with knowledge base
    let kb = KnowledgeBase::new("MyApp");
    let mut engine = RustRuleEngine::new(kb);

    // Load rules from GRL file
    let rules = std::fs::read_to_string("rules/business_logic.grl")?;
    let parsed_rules = GRLParser::parse_rules(&rules)?;

    for rule in parsed_rules {
        engine.knowledge_base().add_rule(rule)?;
    }

    // Create facts
    let mut facts = Facts::new();
    facts.set("customer.tier", Value::String("premium".to_string()));
    facts.set("order.amount", Value::Number(1500.0));

    // Execute rules
    let result = engine.execute(&facts)?;
    println!("Fired {} rules", result.rules_fired);

    Ok(())
}
```

### Example GRL File

```grl
// rules/business_logic.grl
rule "PremiumDiscount" salience 10 no-loop {
    when
        customer.tier == "premium" && order.amount > 1000
    then
        order.discount = 0.15;
        order.free_shipping = true;
}

rule "VIPUpgrade" salience 15 no-loop {
    when
        customer.total_spent > 10000
    then
        customer.tier = "VIP";
}
```

---

## Option 2: RETE-UL Engine (High Performance)

### When to Use
- Need maximum performance
- Have > 100 rules
- Need pattern matching (variable binding, JOINs)
- Want incremental updates
- Migrating from Drools

### Quick Start

```rust
use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine, TypedFacts};

fn main() -> Result<()> {
    // Create RETE engine
    let mut engine = IncrementalEngine::new();

    // Load rules from GRL file (same format!)
    GrlReteLoader::load_from_file("rules/business_logic.grl", &mut engine)?;

    // Or load from string
    let rules = r#"
    rule "HighValueOrder" salience 20 no-loop {
        when
            Order.amount > 1000
        then
            Order.priority = "high";
    }
    "#;
    GrlReteLoader::load_from_string(rules, &mut engine)?;

    // Insert facts with types
    let mut order = TypedFacts::new();
    order.set("amount", 1500.0);
    order.set("customer_id", "C123");
    let handle = engine.insert("Order".to_string(), order);

    // Fire rules
    let fired = engine.fire_all();
    println!("Fired {} rules: {:?}", fired.len(), fired);

    // Incremental update - only affected rules re-evaluated! ⚡
    let mut updated = TypedFacts::new();
    updated.set("amount", 2000.0);
    engine.update(handle, updated)?;

    Ok(())
}
```

### Advanced Pattern Matching

```rust
use rust_rule_engine::rete::{PatternBuilder, MultiPattern, WorkingMemory, FactValue};

let mut wm = WorkingMemory::new();

// Insert facts
let mut customer = TypedFacts::new();
customer.set("name", "John");
customer.set("tier", "VIP");
wm.insert("Customer".to_string(), customer);

let mut order = TypedFacts::new();
order.set("customer_name", "John");
order.set("amount", 1500.0);
wm.insert("Order".to_string(), order);

// Pattern: Match customer with their orders
let customer_pattern = PatternBuilder::for_type("Customer")
    .bind("name", "$customerName")
    .where_field("tier", "==", FactValue::String("VIP".to_string()))
    .build();

let order_pattern = PatternBuilder::for_type("Order")
    .where_var("customer_name", "==", "$customerName")
    .where_field("amount", ">", FactValue::Float(1000.0))
    .build();

let multi = MultiPattern::new("VIPOrders".to_string())
    .with_pattern(customer_pattern)
    .with_pattern(order_pattern);

// Find matches
let matches = multi.match_all(&wm);
for (_handles, bindings) in matches {
    println!("VIP Customer: {}", bindings.get("$customerName").unwrap());
}
```

---

## Option 3: Hybrid Approach (Best of Both Worlds)

Use **both engines** for different purposes:

```rust
use rust_rule_engine::*;
use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine};

fn main() -> Result<()> {
    // Native Engine for plugin-rich business logic
    let mut native = RustRuleEngine::new(KnowledgeBase::new("Native"));
    native.load_rules_from_file("rules/business_logic.grl")?;

    // RETE Engine for high-performance pattern matching
    let mut rete = IncrementalEngine::new();
    GrlReteLoader::load_from_file("rules/streaming_analytics.grl", &mut rete)?;

    // Execute both
    let mut facts = Facts::new();
    facts.set("user.action", Value::String("purchase".to_string()));

    // Native handles business logic + plugins
    native.execute(&facts)?;

    // RETE handles high-frequency updates
    let mut stream_fact = TypedFacts::new();
    stream_fact.set("action", "purchase");
    rete.insert("Event".to_string(), stream_fact);
    rete.fire_all();

    Ok(())
}
```

---

## Performance Comparison

### Native Engine Performance

```rust
// ~50µs per execution (< 50 rules)
let start = Instant::now();
engine.execute(&facts)?;
println!("Execution time: {:?}", start.elapsed());
// Output: ~50µs
```

### RETE-UL Engine Performance

```rust
// ~30µs per execution (100+ rules) - 1.7x faster
let start = Instant::now();
engine.fire_all();
println!("Execution time: {:?}", start.elapsed());
// Output: ~30µs

// Incremental update: ~35µs (4x faster than full re-eval)
let start = Instant::now();
engine.update(handle, updated_fact)?;
engine.fire_all();
println!("Update time: {:?}", start.elapsed());
// Output: ~35µs
```

---

## Migration Guide

### From Native to RETE-UL

**Step 1**: Keep your GRL files (same format!)
```grl
// No changes needed - same syntax!
rule "MyRule" salience 10 no-loop {
    when
        Customer.tier == "VIP"
    then
        Customer.discount = 0.2;
}
```

**Step 2**: Change engine initialization
```rust
// Before: Native
let mut engine = RustRuleEngine::new(kb);
engine.load_rules_from_file("rules.grl")?;

// After: RETE-UL
let mut engine = IncrementalEngine::new();
GrlReteLoader::load_from_file("rules.grl", &mut engine)?;
```

**Step 3**: Update fact creation
```rust
// Before: Native
let mut facts = Facts::new();
facts.set("customer.tier", Value::String("VIP".to_string()));

// After: RETE-UL
let mut facts = TypedFacts::new();
facts.set("tier", "VIP");
let handle = engine.insert("Customer".to_string(), facts);
```

**Step 4**: Benchmark and compare
```rust
// Measure performance improvement
let start = Instant::now();
// ... execute rules
let duration = start.elapsed();
println!("Performance: {:?}", duration);
```

---

## Choosing the Right Engine: Decision Tree

```
START
  |
  ├─ Do you have > 100 rules?
  │  └─ YES → Use RETE-UL Engine ✅
  │  └─ NO  → Continue
  |
  ├─ Do you need plugins (string/math/date utils)?
  │  └─ YES → Use Native Engine ✅
  │  └─ NO  → Continue
  |
  ├─ Do you need pattern matching (variable binding, JOINs)?
  │  └─ YES → Use RETE-UL Engine ✅
  │  └─ NO  → Continue
  |
  ├─ Is performance critical (real-time processing)?
  │  └─ YES → Use RETE-UL Engine ✅
  │  └─ NO  → Continue
  |
  └─ Default → Use Native Engine ✅ (easier to start)
```

---

## Complete Examples

### Native Engine: E-commerce System

```rust
use rust_rule_engine::*;

fn main() -> Result<()> {
    let kb = KnowledgeBase::new("ECommerce");
    let mut engine = RustRuleEngine::new(kb);

    // Load business rules
    let rules = r#"
    rule "FreeShipping" salience 15 {
        when
            order.amount > 50
        then
            order.shipping = 0;
    }

    rule "LoyaltyPoints" salience 10 {
        when
            customer.tier == "gold" && order.amount > 100
        then
            customer.points = customer.points + 500;
    }
    "#;

    for rule in GRLParser::parse_rules(rules)? {
        engine.knowledge_base().add_rule(rule)?;
    }

    // Execute
    let mut facts = Facts::new();
    facts.set("order.amount", Value::Number(150.0));
    facts.set("customer.tier", Value::String("gold".to_string()));
    facts.set("customer.points", Value::Number(1000.0));

    let result = engine.execute(&facts)?;
    println!("Rules fired: {}", result.rules_fired);
    println!("Shipping: {:?}", facts.get("order.shipping"));
    println!("Points: {:?}", facts.get("customer.points"));

    Ok(())
}
```

### RETE-UL Engine: Real-Time Trading

```rust
use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine, TypedFacts};

fn main() -> Result<()> {
    let mut engine = IncrementalEngine::new();

    // Load trading rules
    let rules = r#"
    rule "BuySignal" salience 25 no-loop {
        when
            Stock.price > 100 && Stock.change_pct > 5
        then
            Alert.signal = "BUY";
    }

    rule "SellSignal" salience 25 no-loop {
        when
            Stock.price < 90 && Stock.change_pct < -5
        then
            Alert.signal = "SELL";
    }
    "#;

    GrlReteLoader::load_from_string(rules, &mut engine)?;

    // Stream stock updates
    for tick in stock_stream() {
        let mut stock = TypedFacts::new();
        stock.set("price", tick.price);
        stock.set("change_pct", tick.change);

        // Incremental update - super fast! ⚡
        let handle = engine.insert("Stock".to_string(), stock);

        // Check for signals
        engine.reset();
        let fired = engine.fire_all();

        if !fired.is_empty() {
            println!("Trading signal: {:?}", fired);
        }
    }

    Ok(())
}
```

---

## FAQ

**Q: Can I use the same GRL files for both engines?**
A: Yes! Both engines support the same GRL syntax.

**Q: Which engine should beginners use?**
A: Start with Native Engine - it's simpler and has more features.

**Q: Can I switch engines later?**
A: Yes! Your GRL files work with both engines.

**Q: Do I need to choose just one?**
A: No! Use both engines in the same application (hybrid approach).

**Q: Which is faster?**
A: RETE-UL is 2-16x faster for large rule sets (> 100 rules).

**Q: Which has more features?**
A: Native has plugins, analytics, action handlers. RETE-UL has pattern matching.

---

## Next Steps

- 📖 Read [ENGINE_COMPARISON.md](ENGINE_COMPARISON.md) for detailed comparison
- 🎯 Try examples: `cargo run --example rete_grl_demo`
- 📚 See [RETE_DROOLS_IMPROVEMENTS.md](RETE_DROOLS_IMPROVEMENTS.md) for RETE details
- 🚀 Check [README.md](README.md) for full documentation

---

**Ready to start? Choose your engine and build amazing rule-based systems!** 🚀

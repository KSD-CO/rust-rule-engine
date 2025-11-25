# RETE-Optimized GRL Rules

Rules optimized for the RETE-UL engine with incremental pattern matching.

## Files

### 1. rete_demo.grl
**10 rules** - RETE-UL optimized patterns

Features:
- Conditional elements (exists, forall)
- Complex pattern matching
- Optimized for incremental evaluation
- Multiple fact patterns

**Example:**
```grl
rule "CheckAllAdults" {
    when
        exists(Person.Age >= 18)
        forall(Person.IsVerified == true)
    then
        System.AllAdultsVerified = true;
}
```

**Best for:**
- Large rulesets (100+ rules)
- Frequent fact updates
- Complex pattern matching
- Performance-critical applications

### 2. test_ce_rules.grl
**5 rules** - Conditional Elements testing

Conditional Elements:
- `exists()` - At least one match
- `forall()` - All must match
- `!exists()` - No matches
- Combined CE patterns

**Example:**
```grl
rule "NoHighRiskUsers" {
    when
        !exists(User.RiskLevel == "HIGH")
    then
        System.SafeToProcess = true;
}
```

**Learn:**
- CE syntax and semantics
- Quantified patterns
- Negation patterns

### 3. multifield_patterns.grl
**10 rules** - CLIPS-inspired multifield variables

Features:
- Multifield slot matching
- Variable binding patterns
- List operations
- Complex data structures

**Example:**
```grl
rule "ProcessOrders" {
    when
        exists($order: Order && $order.Items.Count > 0)
    then
        ProcessOrderItems($order);
}
```

**Learn:**
- Working with collections
- Multifield slots
- Variable binding ($var syntax)
- CLIPS-style patterns

## RETE vs Native Engine

### When to use RETE?

**✅ Use RETE when:**
- Number of rules > 100
- Facts change frequently
- Complex patterns, many joins
- Need high performance
- Have many conditional elements

**❌ Don't need RETE when:**
- Few rules (< 50)
- Static facts, rarely change
- Simple patterns
- Simplicity is priority

### Performance Characteristics

| Feature | Native | RETE-UL |
|---------|--------|---------|
| Initial load | Fast | Slower (build network) |
| First execution | Fast | Medium |
| Subsequent runs | Medium | Very Fast |
| Memory usage | Low | Higher (stores partial matches) |
| Best for | Simple rules | Complex patterns |

## Conditional Elements Deep Dive

### exists()
"At least one fact satisfies"
```grl
rule "HasVIPCustomer" {
    when
        exists(Customer.Status == "VIP")
    then
        EnableVIPFeatures();
}
```

### forall()
"All facts must satisfy"
```grl
rule "AllOrdersShipped" {
    when
        forall(Order.Status == "SHIPPED")
    then
        CloseBusinessDay();
}
```

### !exists() / not()
"No facts satisfy"
```grl
rule "NoPendingOrders" {
    when
        !exists(Order.Status == "PENDING")
    then
        System.ReadyForNewOrders = true;
}
```

### Combined Patterns
```grl
rule "ComplexCheck" {
    when
        exists(Customer.Age >= 18) &&
        forall(Order.IsPaid == true) &&
        !exists(Customer.IsBlocked == true)
    then
        ProcessOrders();
}
```

## Multifield Variables

### Syntax
```grl
rule "ProcessItems" {
    when
        exists($item: CartItem && $item.Price > 100)
    then
        ApplyDiscount($item);
}
```

### Variable Binding
- `$var` - Bind matched fact to variable
- Use `$var` in then clause
- Can bind multiple variables

### Collections
```grl
rule "BulkOrder" {
    when
        exists($order: Order && $order.Items.Length > 10)
    then
        $order.ApplyBulkDiscount();
}
```

## Optimization Tips

### 1. Pattern Order
Put selective patterns first:
```grl
// Good - specific first
when
    User.Role == "ADMIN" && User.IsActive == true

// Less optimal - broad first
when
    User.IsActive == true && User.Role == "ADMIN"
```

### 2. Avoid Redundancy
```grl
// Good - single exists
when
    exists(Order.Total > 1000)

// Bad - redundant check
when
    exists(Order.Total > 1000) && Order.Total > 1000
```

### 3. Use Salience Wisely
```grl
// High priority rules first
rule "CriticalCheck" salience 100 {
    when exists(Alert.Severity == "CRITICAL")
    then HandleCritical();
}

rule "NormalCheck" salience 10 {
    when exists(Alert.Severity == "INFO")
    then HandleInfo();
}
```

## Integration with Rust

```rust
use rust_rule_engine::rete::IncrementalEngine;

// Load RETE-optimized rules
let engine = IncrementalEngine::new();
engine.load_grl("rules/02-rete/rete_demo.grl")?;

// Add facts
engine.assert_fact("Person", person1)?;
engine.assert_fact("Person", person2)?;

// Run (incremental matching)
engine.run()?;

// Update fact (only affected rules re-evaluate)
engine.modify_fact("Person", updated_person)?;
engine.run()?;
```

## Run Examples

```bash
# RETE basic demo
cargo run --example rete_demo

# RETE with GRL
cargo run --example rete_grl_demo

# Multifield demo
cargo run --example rete_multifield_demo
```

## Next Steps

After understanding RETE basics:
- `03-advanced/pattern_matching.grl` - Advanced patterns
- `07-advanced-rete/` - Advanced RETE features
- Performance examples in `05-performance/`

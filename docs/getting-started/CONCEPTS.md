# Core Concepts

> **Version:** 1.11.0
> **Prerequisite:** [Quick Start Guide](QUICK_START.md)

Understanding the fundamental concepts of the Rust Rule Engine.

---

## 📚 Table of Contents

1. [Facts & Working Memory](#facts--working-memory)
2. [Rules](#rules)
3. [Pattern Matching](#pattern-matching)
4. [Forward vs Backward Chaining](#forward-vs-backward-chaining)
5. [RETE Algorithm](#rete-algorithm)
6. [GRL Syntax](#grl-syntax)

---

## Facts & Working Memory

### What are Facts?

Facts are pieces of data that represent the current state of your system. Think of them as a key-value store:

```rust
use rust_rule_engine::{Facts, Value};

let mut facts = Facts::new();

// Setting facts
facts.set("Customer.Name", Value::String("Alice".to_string()));
facts.set("Customer.Age", Value::Integer(30));
facts.set("Customer.TotalSpent", Value::Number(1500.0));
facts.set("Customer.IsVIP", Value::Boolean(false));
```

### Fact Types

| Type | Rust Value | Example |
|------|------------|---------|
| **String** | `Value::String(String)` | `"Alice"`, `"Premium"` |
| **Integer** | `Value::Integer(i64)` | `42`, `-10`, `1000` |
| **Number** | `Value::Number(f64)` | `3.14`, `99.99`, `1500.0` |
| **Boolean** | `Value::Boolean(bool)` | `true`, `false` |

### Hierarchical Facts

Use dot notation for structured data:

```rust
// Customer facts
facts.set("Customer.Name", Value::String("Alice".to_string()));
facts.set("Customer.Address.City", Value::String("NYC".to_string()));
facts.set("Customer.Address.Zip", Value::String("10001".to_string()));

// Order facts
facts.set("Order.ID", Value::String("ORD-123".to_string()));
facts.set("Order.Total", Value::Number(299.99));
facts.set("Order.Items.Count", Value::Integer(5));
```

### Working Memory

Working memory is the **current state** of all facts. The rule engine:
1. Reads facts from working memory
2. Evaluates rules against these facts
3. Updates working memory with new facts

```rust
let mut facts = Facts::new();          // Empty working memory
facts.set("X", Value::Integer(10));     // Add fact to working memory
engine.run(&mut facts)?;                 // Engine processes working memory
// Facts may be updated by rules
```

---

## Rules

### Rule Structure

A rule has three parts:

```grl
rule "Rule Name" {
    when
        <conditions>     // Pattern to match
    then
        <actions>        // What to do when matched
}
```

### Example Rule

```grl
rule "VIP Discount" {
    when
        Customer.TotalSpent > 1000 &&
        Customer.Membership == "Gold"
    then
        Customer.DiscountRate = 0.2;
        Customer.FreeShipping = true;
        LogMessage("VIP discount applied");
}
```

**Components:**
- **Name**: `"VIP Discount"` - Describes what the rule does
- **When** (Condition): Checks if `TotalSpent > 1000` AND `Membership == "Gold"`
- **Then** (Action): Sets discount rate and enables free shipping

### Rule Execution

Rules are evaluated in the **Recognize-Act Cycle**:

```
1. MATCH: Find all rules whose conditions match current facts
2. SELECT: Choose which rule to fire (conflict resolution)
3. FIRE: Execute the actions of the selected rule
4. REPEAT: Go back to step 1 with updated facts
```

### Rule Syntax

```grl
rule "Name" {
    when
        // Conditions (AND with &&, OR with ||)
        Field1 == "value" &&
        Field2 > 100
    then
        // Actions
        ResultField = "computed value";
        AnotherField = Field2 * 2;
}
```

---

## Pattern Matching

### Simple Patterns

```grl
// Equality
Customer.Type == "VIP"

// Comparison
Order.Total > 100
Product.Stock < 10

// Boolean
Customer.IsActive == true
Item.InStock == false
```

### Complex Patterns

```grl
// Multiple conditions (AND)
Customer.Age > 18 &&
Customer.Income > 50000 &&
Customer.CreditScore > 700

// Disjunction (OR)
(Customer.Type == "VIP" || Customer.TotalSpent > 10000) &&
Customer.IsActive == true

// Negation (NOT)
NOT Customer.IsBanned == true
```

### Arithmetic in Patterns

```grl
// In conditions
Order.Total > Order.SubTotal * 1.1

// In actions
Order.Tax = Order.SubTotal * 0.08;
Order.Final = Order.SubTotal + Order.Tax;
```

---

## Forward vs Backward Chaining

### Forward Chaining (Data-Driven)

**Start with facts → Apply rules → Derive conclusions**

```rust
// Forward chaining example
let mut engine = Engine::new();
engine.add_rule_from_string(r#"
    rule "Infer High Risk" {
        when
            Applicant.CreditScore < 600 &&
            Applicant.Income < 30000
        then
            Applicant.RiskLevel = "high";
    }
"#)?;

let mut facts = Facts::new();
facts.set("Applicant.CreditScore", Value::Integer(550));
facts.set("Applicant.Income", Value::Number(25000.0));

engine.run(&mut facts)?;
// Result: Applicant.RiskLevel = "high" is derived
```

**Use When:**
- You have data and want to find all applicable conclusions
- Real-time event processing
- Business rule automation
- System monitoring and alerts

### Backward Chaining (Goal-Driven)

**Start with goal → Find rules → Request needed facts**

```rust
// Backward chaining example
use rust_rule_engine::backward::BackwardEngine;

let mut bc_engine = BackwardEngine::new(kb);

// Ask: "Is applicant high risk?"
let result = bc_engine.query(
    "Applicant.RiskLevel == \"high\"",
    &mut facts
)?;

if result.provable {
    println!("Applicant is high risk");
}
```

**Use When:**
- You have a question and want to find if it's true
- Diagnostic systems
- Decision support
- Complex queries and reasoning

### Comparison

| Aspect | Forward Chaining | Backward Chaining |
|--------|------------------|-------------------|
| **Direction** | Facts → Conclusions | Goal → Facts |
| **Trigger** | New data arrives | Question asked |
| **Efficiency** | All applicable rules | Only relevant rules |
| **Best For** | Event processing | Queries & diagnosis |
| **Example** | "What can I conclude?" | "Is X true?" |

---

## RETE Algorithm

### What is RETE?

RETE (Latin for "net") is a pattern-matching algorithm that makes forward chaining extremely fast.

**Key Idea:** Don't re-evaluate everything when facts change - only check what's affected.

### How RETE Works

```
1. BUILD NETWORK
   Rules → Compiled into a discrimination network

2. MATCH FACTS
   Facts → Flow through network
   Network → Remembers partial matches

3. UPDATE EFFICIENTLY
   Fact changes → Only affected nodes re-evaluate
   Result: O(1) to O(n) instead of O(rules × facts)
```

### RETE Network Structure

```
         [Root]
           |
    [Type Node: Customer]
           |
    [Alpha Node: Customer.Type == "VIP"]
           |
    [Beta Node: Join with Order]
           |
    [Terminal: Fire Rule]
```

### Performance Benefits

**Without RETE:**
```
10,000 facts × 1,000 rules = 10,000,000 checks
Every fact change: Full re-evaluation
```

**With RETE:**
```
Initial: Build network once
Fact change: Check only affected paths (typically < 100)
Result: 100-1000x faster
```

### Example

```rust
// RETE automatically optimizes this:
engine.add_rule_from_string(r#"
    rule "Complex Pattern" {
        when
            Customer.Type == "VIP" &&
            Order.Total > 1000 &&
            Inventory.Stock > 0
        then
            Process();
    }
"#)?;

// Network built once
// Subsequent fact updates are O(1)
facts.set("Order.Total", Value::Number(1500.0)); // Fast!
```

---

## GRL Syntax

### GRL = Grule Rule Language

A domain-specific language for writing rules in a clear, readable format.

### Basic GRL Structure

```grl
rule "Rule Name" "Optional description" salience 10 {
    when
        <conditions>
    then
        <actions>
}
```

### GRL Features

#### 1. Salience (Priority)

```grl
rule "High Priority" salience 100 {
    when Customer.Type == "VIP"
    then ProcessFirst();
}

rule "Low Priority" salience 10 {
    when Customer.Type == "Regular"
    then ProcessLater();
}
```

Higher salience = Higher priority (fires first)

#### 2. String Functions

```grl
rule "Uppercase Check" {
    when
        Customer.Name.ToUpper() == "ALICE"
    then
        Match = true;
}
```

#### 3. Mathematical Operations

```grl
rule "Calculate Discount" {
    when
        Order.Total > 100
    then
        Order.Discount = Order.Total * 0.1;
        Order.Final = Order.Total - Order.Discount;
}
```

#### 4. Logical Operators

```grl
rule "Complex Logic" {
    when
        (A == 1 || B == 2) &&
        (C > 3 && D < 4) &&
        NOT E == true
    then
        Result = "matched";
}
```

---

## Key Takeaways

✅ **Facts** = Current state (key-value store)
✅ **Rules** = If-then logic (when X then Y)
✅ **Pattern Matching** = Finding facts that match conditions
✅ **Forward Chaining** = Data-driven (facts → conclusions)
✅ **Backward Chaining** = Goal-driven (question → proof)
✅ **RETE** = Fast pattern matching algorithm
✅ **GRL** = Human-readable rule syntax

---

## Next Steps

**📖 Learn More:**
- [Forward Chaining Guide](../core-features/FORWARD_CHAINING.md)
- [Backward Chaining Guide](../BACKWARD_CHAINING_QUICK_START.md)
- [GRL Syntax Reference](../core-features/GRL_SYNTAX.md)

**🔨 Build Something:**
- [First Rules Tutorial](FIRST_RULES.md)
- [Real-World Examples](../examples/)

**📚 Go Deeper:**
- [API Reference](../api-reference/API_REFERENCE.md)
- [Performance Tuning](../advanced-features/PERFORMANCE.md)

---

## Navigation

◀️ **Previous: [Quick Start](QUICK_START.md)** | 📚 **[Documentation Home](../README.md)** | ▶️ **Next: [First Rules](FIRST_RULES.md)**

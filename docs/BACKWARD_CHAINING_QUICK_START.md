# Backward Chaining Quick Start Guide

> **Version**: 1.1.0-beta
> **5-Minute Getting Started Guide**

---

## 🚀 Quick Start (3 Steps)

### Step 1: Add Dependency

```toml
[dependencies]
rust-rule-engine = { version = "1.1.0-beta", features = ["backward-chaining"] }
```

### Step 2: Create Engine

```rust
use rust_rule_engine::backward::BackwardEngine;
use rust_rule_engine::KnowledgeBase;
use rust_rule_engine::Facts;
use rust_rule_engine::types::Value;

// Create knowledge base with rules
let mut kb = KnowledgeBase::new("my_kb");
// ... add rules (see below) ...

// Create backward chaining engine
let mut bc_engine = BackwardEngine::new(kb);
```

### Step 3: Query

```rust
// Set initial facts
let mut facts = Facts::new();
facts.set("User.Age", Value::Number(25.0));

// Query for a goal
let result = bc_engine.query("User.IsAdult == true", &mut facts)?;

// Check result
if result.is_provable() {
    println!("✅ Goal proven!");
} else {
    println!("❌ Goal not provable");
}
```

---

## 📝 Complete Example

```rust
use rust_rule_engine::backward::BackwardEngine;
use rust_rule_engine::{KnowledgeBase, Facts, Rule, Condition, ConditionGroup};
use rust_rule_engine::types::{Value, ActionType, Operator};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create knowledge base
    let mut kb = KnowledgeBase::new("vip_checker");

    // 2. Add rule: "If User.Age >= 18 then User.IsAdult = true"
    kb.add_rule(Rule::new(
        "CheckAdult".to_string(),
        ConditionGroup::single(Condition::new(
            "User.Age".to_string(),
            Operator::GreaterOrEqual,
            Value::Number(18.0),
        )),
        vec![ActionType::Set {
            field: "User.IsAdult".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // 3. Add another rule: "If User.IsAdult and User.Points > 1000 then User.IsVIP = true"
    kb.add_rule(Rule::new(
        "CheckVIP".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(Condition::new(
                "User.IsAdult".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            ConditionGroup::single(Condition::new(
                "User.Points".to_string(),
                Operator::Greater,
                Value::Number(1000.0),
            )),
        ),
        vec![ActionType::Set {
            field: "User.IsVIP".to_string(),
            value: Value::Boolean(true),
        }],
    ))?;

    // 4. Create backward engine
    let mut bc_engine = BackwardEngine::new(kb);

    // 5. Set initial facts
    let mut facts = Facts::new();
    facts.set("User.Age", Value::Number(25.0));
    facts.set("User.Points", Value::Number(1500.0));

    // 6. Query: Can we prove User.IsVIP?
    let result = bc_engine.query("User.IsVIP == true", &mut facts)?;

    // 7. Check result
    if result.is_provable() {
        println!("✅ User is VIP!");

        // Show how it was proven
        if let Some(trace) = result.proof_trace() {
            println!("\n📋 Proof:");
            for step in trace.steps() {
                println!("  - Used rule: {}", step.rule_name());
            }
        }
    } else {
        println!("❌ User is not VIP");
    }

    Ok(())
}
```

**Output**:
```
✅ User is VIP!

📋 Proof:
  - Used rule: CheckVIP
  - Used rule: CheckAdult
```

---

## 🎯 Common Patterns

### Pattern 1: Simple Fact Checking

```rust
// Query if a fact is true
facts.set("Order.Status", Value::String("Completed".to_string()));
let result = bc_engine.query("Order.Status == \"Completed\"", &mut facts)?;
```

### Pattern 2: Numeric Comparisons

```rust
// Check if number meets threshold
facts.set("Order.Total", Value::Number(150.0));
let result = bc_engine.query("Order.Total > 100", &mut facts)?;
```

### Pattern 3: Logical AND

```rust
// Check multiple conditions
let result = bc_engine.query(
    "User.IsVIP == true && Order.Total > 1000",
    &mut facts
)?;
```

### Pattern 4: Logical OR

```rust
// Check any condition is true
let result = bc_engine.query(
    "User.IsVIP == true || User.IsPremium == true",
    &mut facts
)?;
```

### Pattern 5: Negation

```rust
// Check condition is false
let result = bc_engine.query("!User.IsBanned", &mut facts)?;
```

### Pattern 6: Complex Expression

```rust
// Nested logic
let result = bc_engine.query(
    "(User.IsVIP == true && Order.Total > 1000) || User.IsPremium == true",
    &mut facts
)?;
```

---

## 🔧 Configuration

### Basic Configuration

```rust
use rust_rule_engine::backward::BackwardConfig;

let config = BackwardConfig {
    max_depth: 20,                    // Max rule chaining depth
    generate_proof_trace: true,       // Enable proof traces
    search_strategy: SearchStrategy::DepthFirst,
    ..Default::default()
};

let mut bc_engine = BackwardEngine::with_config(kb, config);
```

### Performance Tuning

```rust
// For production - disable proof traces to save memory
let config = BackwardConfig {
    generate_proof_trace: false,  // Faster, less memory
    max_depth: 50,
    ..Default::default()
};
```

---

## 📊 Working with Results

### Check if Goal is Provable

```rust
let result = bc_engine.query(goal, &mut facts)?;

if result.is_provable() {
    println!("✅ Goal proven");
}
```

### Get Proof Trace

```rust
if let Some(trace) = result.proof_trace() {
    println!("Rules used:");
    for step in trace.steps() {
        println!("  - {}", step.rule_name());
    }
}
```

### Get Statistics

```rust
println!("Goals explored: {}", result.goals_explored());
println!("Rules evaluated: {}", result.rules_evaluated());
println!("Query time: {:?}", result.query_time());
```

### Get Derived Facts

```rust
// Facts are modified in-place
println!("All facts after query: {:?}", facts.all());
```

---

## 🎨 Rule Creation Patterns

### Simple Rule

```rust
Rule::new(
    "rule_name".to_string(),
    ConditionGroup::single(Condition::new(
        "Field".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    )),
    vec![ActionType::Set {
        field: "Output".to_string(),
        value: Value::Boolean(true),
    }],
)
```

### Rule with AND Conditions

```rust
Rule::new(
    "rule_name".to_string(),
    ConditionGroup::and(
        ConditionGroup::single(Condition::new("A".to_string(), ...)),
        ConditionGroup::single(Condition::new("B".to_string(), ...)),
    ),
    vec![ActionType::Set { ... }],
)
```

### Rule with OR Conditions

```rust
Rule::new(
    "rule_name".to_string(),
    ConditionGroup::or(
        ConditionGroup::single(Condition::new("A".to_string(), ...)),
        ConditionGroup::single(Condition::new("B".to_string(), ...)),
    ),
    vec![ActionType::Set { ... }],
)
```

### Rule with NOT Condition

```rust
Rule::new(
    "rule_name".to_string(),
    ConditionGroup::not(
        ConditionGroup::single(Condition::new("A".to_string(), ...))
    ),
    vec![ActionType::Set { ... }],
)
```

---

## 🚨 Common Mistakes

### ❌ Mistake 1: Field Name Mismatch

```rust
// Wrong
facts.set("UserAge", Value::Number(25.0));
bc_engine.query("User.Age == 25", &mut facts)?;  // Won't match!

// Correct
facts.set("User.Age", Value::Number(25.0));
bc_engine.query("User.Age == 25", &mut facts)?;  // ✅
```

### ❌ Mistake 2: Immutable Facts

```rust
// Wrong
let facts = Facts::new();  // Immutable!
bc_engine.query(goal, &facts)?;  // Error!

// Correct
let mut facts = Facts::new();  // Mutable!
bc_engine.query(goal, &mut facts)?;  // ✅
```

### ❌ Mistake 3: Wrong Operator

```rust
// Wrong - single '='
"User.IsVIP = true"  // Parse error!

// Correct - double '=='
"User.IsVIP == true"  // ✅
```

### ❌ Mistake 4: Creating New Engine Each Query

```rust
// Slow - loses memoization
for query in queries {
    let bc_engine = BackwardEngine::new(kb.clone());  // ❌
    bc_engine.query(query, &mut facts)?;
}

// Fast - reuses engine
let mut bc_engine = BackwardEngine::new(kb);  // ✅
for query in queries {
    bc_engine.query(query, &mut facts)?;
}
```

---

## 📖 Next Steps

### Learn More

1. **Examples**: Check `examples/09-backward-chaining/` for real-world demos
2. **Troubleshooting**: See [BACKWARD_CHAINING_TROUBLESHOOTING.md](./BACKWARD_CHAINING_TROUBLESHOOTING.md)
3. **Performance**: Read [BACKWARD_CHAINING_PERFORMANCE.md](../.planning/BACKWARD_CHAINING_PERFORMANCE.md)
4. **API Docs**: https://docs.rs/rust-rule-engine

### Performance Tips

1. ✅ Reuse `BackwardEngine` instances
2. ✅ Disable proof traces in production
3. ✅ Put cheap conditions first in queries
4. ✅ Set facts directly when possible instead of deriving
5. ✅ Use appropriate max_depth setting

### Best Practices

1. ✅ Use consistent field naming (e.g., `Object.Field`)
2. ✅ Add error handling for queries
3. ✅ Validate rules before adding to KB
4. ✅ Test with minimal examples first
5. ✅ Monitor performance with statistics

---

## 🏃 Run Examples

```bash
# Simple query demo
cargo run --example simple_query_demo --features backward-chaining

# Medical diagnosis
cargo run --example medical_diagnosis_demo --features backward-chaining

# E-commerce approval
cargo run --example ecommerce_approval_demo --features backward-chaining

# Performance showcase
cargo run --example rete_index_demo --features backward-chaining

# See all examples
ls examples/09-backward-chaining/
```

---

## 🧪 Run Tests

```bash
# All tests
cargo test --features backward-chaining

# Specific test file
cargo test --features backward-chaining --test backward_comprehensive_tests

# With output
cargo test --features backward-chaining -- --nocapture
```

---

## 📊 Run Benchmarks

```bash
# All benchmarks
cargo bench --features backward-chaining --bench backward_chaining_benchmarks

# Specific group
cargo bench --features backward-chaining --bench backward_chaining_benchmarks expression_parsing

# Generate HTML report
cargo bench --features backward-chaining --bench backward_chaining_benchmarks -- --save-baseline main
```

---

## 💡 Quick Tips

### Tip 1: Debug Queries

```rust
let result = bc_engine.query(goal, &mut facts)?;
println!("Provable: {}", result.is_provable());
println!("Goals explored: {}", result.goals_explored());
if let Some(trace) = result.proof_trace() {
    println!("Proof: {:#?}", trace);
}
```

### Tip 2: Test Index Performance

```rust
use std::time::Instant;

let start = Instant::now();
let result = bc_engine.query(goal, &mut facts)?;
println!("Query time: {:?}", start.elapsed());
// Should be <10ms for most queries
```

### Tip 3: Validate Rules

```rust
for rule in kb.get_rules() {
    if rule.actions.is_empty() {
        eprintln!("Warning: Rule '{}' has no actions", rule.name);
    }
}
```

---

**Happy Backward Chaining!** 🚀

For issues or questions: https://github.com/KSD-CO/rust-rule-engine/issues

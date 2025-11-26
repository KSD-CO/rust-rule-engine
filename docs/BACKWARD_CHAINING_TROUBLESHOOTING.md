# Backward Chaining Troubleshooting Guide

> **Version**: 1.1.0-beta
> **Last Updated**: 2025-11-27
> **For**: rust-rule-engine backward chaining feature

---

## 📋 Table of Contents

1. [Common Issues](#common-issues)
2. [Performance Problems](#performance-problems)
3. [Query Errors](#query-errors)
4. [Rule Execution Issues](#rule-execution-issues)
5. [Memory & Resource Issues](#memory--resource-issues)
6. [Integration Problems](#integration-problems)
7. [Debugging Tips](#debugging-tips)
8. [FAQ](#faq)

---

## 🔧 Common Issues

### Issue 1: Feature Flag Not Enabled

**Symptoms**:
```rust
error[E0433]: failed to resolve: could not find `backward` in `rust_rule_engine`
  --> src/main.rs:1:29
   |
1  | use rust_rule_engine::backward::BackwardEngine;
   |                       ^^^^^^^^ could not find `backward` in `rust_rule_engine`
```

**Cause**: The `backward-chaining` feature flag is not enabled.

**Solution**:

Add the feature flag to `Cargo.toml`:

```toml
[dependencies]
rust-rule-engine = { version = "1.1.0-beta", features = ["backward-chaining"] }
```

Or use on command line:
```bash
cargo build --features backward-chaining
cargo test --features backward-chaining
cargo run --features backward-chaining
```

**Verification**:
```bash
cargo tree --features backward-chaining | grep petgraph
# Should show: petgraph v0.6.x
```

---

### Issue 2: Query Returns "Not Provable" When It Should Succeed

**Symptoms**:
```rust
let result = bc_engine.query("User.IsVIP == true", &mut facts)?;
assert!(result.is_provable()); // FAILS
```

**Common Causes**:

#### Cause 2.1: Facts Not Set Correctly

**Problem**: Field name mismatch
```rust
// ❌ Wrong - field name doesn't match query
facts.set("UserIsVIP", Value::Boolean(true));
let result = bc_engine.query("User.IsVIP == true", &mut facts)?;
// Returns: not provable
```

**Solution**: Match field names exactly
```rust
// ✅ Correct - exact match
facts.set("User.IsVIP", Value::Boolean(true));
let result = bc_engine.query("User.IsVIP == true", &mut facts)?;
// Returns: provable
```

#### Cause 2.2: No Rules Conclude the Goal

**Problem**: No rules set the field you're querying
```rust
// Rule sets "Order.Total"
kb.add_rule(Rule::new(
    "CalculateTotal".to_string(),
    conditions,
    vec![ActionType::Set {
        field: "Order.Total".to_string(),
        value: Value::Number(100.0),
    }],
))?;

// ❌ Query different field
let result = bc_engine.query("Order.Amount > 50", &mut facts)?;
// Returns: not provable (no rule sets Order.Amount)
```

**Solution**: Ensure rules conclude what you're querying
```rust
// ✅ Correct - query matches rule conclusion
let result = bc_engine.query("Order.Total > 50", &mut facts)?;
// Returns: provable
```

#### Cause 2.3: Rule Conditions Not Satisfied

**Problem**: Rule exists but its conditions aren't met
```rust
// Rule requires User.Age >= 18
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

// ❌ Age not set
facts.set("User.Name", Value::String("John".to_string()));
let result = bc_engine.query("User.IsAdult == true", &mut facts)?;
// Returns: not provable
```

**Solution**: Set required facts
```rust
// ✅ Correct - all required facts set
facts.set("User.Age", Value::Number(25.0));
let result = bc_engine.query("User.IsAdult == true", &mut facts)?;
// Returns: provable
```

**Debugging**:
```rust
// Enable debug logging
let result = bc_engine.query("User.IsAdult == true", &mut facts)?;

// Check proof trace
if let Some(trace) = result.proof_trace() {
    println!("Proof trace: {:#?}", trace);
    // Shows which rules were tried and why they failed
}

// Check explored goals
println!("Goals explored: {}", result.goals_explored());
```

---

### Issue 3: Query Parsing Errors

**Symptoms**:
```rust
let result = bc_engine.query("User.IsVIP = true", &mut facts);
// Error: Failed to parse query expression
```

**Common Causes**:

#### Cause 3.1: Invalid Operator

**Problem**: Using wrong comparison operator
```rust
// ❌ Wrong - single '=' is not valid
"User.IsVIP = true"

// ❌ Wrong - triple '=' is not valid
"User.Age === 25"
```

**Solution**: Use correct operators
```rust
// ✅ Correct operators
"User.IsVIP == true"   // Equality
"User.Age != 25"       // Not equal
"Score > 50"           // Greater than
"Score < 100"          // Less than
"Points >= 100"        // Greater or equal
"Temperature <= 32"    // Less or equal
```

#### Cause 3.2: Missing Quotes for Strings

**Problem**: String values without quotes
```rust
// ❌ Wrong - string needs quotes
"Status == Active"
```

**Solution**: Quote string literals
```rust
// ✅ Correct
"Status == \"Active\""
```

#### Cause 3.3: Invalid Field Names

**Problem**: Field names with invalid characters
```rust
// ❌ Wrong - spaces in field name
"User Name == \"John\""

// ❌ Wrong - special characters
"User@Email == \"test\""
```

**Solution**: Use valid identifiers
```rust
// ✅ Correct - use dots for nested fields
"User.Name == \"John\""
"User.Email == \"test@example.com\""
```

---

### Issue 4: Performance Degradation

**Symptoms**:
- Queries taking longer than expected
- CPU usage high
- Memory growing over time

**Diagnosis**:
```rust
use std::time::Instant;

let start = Instant::now();
let result = bc_engine.query("Complex.Goal", &mut facts)?;
let elapsed = start.elapsed();

println!("Query time: {:?}", elapsed);
println!("Goals explored: {}", result.goals_explored());

// If elapsed > 100ms for simple queries, investigate
```

**Common Causes & Solutions**: See [Performance Problems](#performance-problems) section.

---

## 🚀 Performance Problems

### Problem 1: Slow Query Execution

**Symptoms**: Queries taking >100ms for <1000 rules

**Diagnosis**:
```rust
let result = bc_engine.query(goal, &mut facts)?;
println!("Goals explored: {}", result.goals_explored());
println!("Rules evaluated: {}", result.rules_evaluated());

// If goals_explored > 1000, you have a deep search tree
```

**Solutions**:

#### Solution 1.1: Verify Conclusion Index is Enabled

```rust
// Check if index is working
use rust_rule_engine::backward::conclusion_index::ConclusionIndex;

let index = bc_engine.conclusion_index(); // If available
let stats = index.stats();
println!("Index stats: {:?}", stats);

// Should show:
// - total_rules > 0
// - indexed_fields > 0
```

If index is empty, rebuild the engine:
```rust
// Force rebuild
let bc_engine = BackwardEngine::new(kb.clone());
```

#### Solution 1.2: Optimize Query Order

Put cheaper conditions first:
```rust
// ❌ Slow - expensive check first
"ExpensiveFunction() && User.IsVIP == true"

// ✅ Fast - cheap check first (short-circuit)
"User.IsVIP == true && ExpensiveFunction()"
```

#### Solution 1.3: Reduce Search Depth

Limit chaining depth if getting too deep:
```rust
let config = BackwardConfig {
    max_depth: 10,  // Limit depth
    ..Default::default()
};

let mut bc_engine = BackwardEngine::with_config(kb, config);
```

#### Solution 1.4: Add Memoization

Reuse engine instance for multiple queries:
```rust
// ❌ Slow - creates new engine each time
for query in queries {
    let bc_engine = BackwardEngine::new(kb.clone());
    bc_engine.query(query, &mut facts)?;
}

// ✅ Fast - reuse engine (memoization works)
let mut bc_engine = BackwardEngine::new(kb.clone());
for query in queries {
    bc_engine.query(query, &mut facts)?;
}
```

---

### Problem 2: High Memory Usage

**Symptoms**: Memory growing continuously, OOM errors

**Diagnosis**:
```rust
// Check proof trace size
let result = bc_engine.query(goal, &mut facts)?;
if let Some(trace) = result.proof_trace() {
    println!("Trace size: {} bytes",
             std::mem::size_of_val(trace));
}
```

**Solutions**:

#### Solution 2.1: Disable Proof Traces for Production

```rust
let config = BackwardConfig {
    generate_proof_trace: false,  // Saves memory
    ..Default::default()
};

let mut bc_engine = BackwardEngine::with_config(kb, config);
```

#### Solution 2.2: Clear Facts After Queries

```rust
for query in queries {
    let mut facts = Facts::new();
    // ... set facts ...
    bc_engine.query(query, &mut facts)?;
    // facts dropped here, memory freed
}
```

#### Solution 2.3: Limit Search Depth

```rust
let config = BackwardConfig {
    max_depth: 20,  // Prevent infinite recursion
    max_goals: 1000,  // Limit goal exploration
    ..Default::default()
};
```

---

### Problem 3: Conclusion Index Not Working

**Symptoms**: Lookups still O(n) slow

**Diagnosis**:
```rust
// Time a lookup
use std::time::Instant;

let start = Instant::now();
let candidates = bc_engine.find_candidates("Field == value");
let elapsed = start.elapsed();

println!("Lookup time: {:?}", elapsed);
// Should be <1µs for O(1) performance
// If >10µs for <1000 rules, index may not be working
```

**Solutions**:

#### Solution 3.1: Check Index Build

```rust
// Verify index was built
let stats = bc_engine.conclusion_index().stats();
assert!(stats.total_rules > 0, "Index not built!");
```

#### Solution 3.2: Rebuild Index After Rule Changes

```rust
// If you modify rules after engine creation:
kb.add_rule(new_rule)?;

// ❌ Index is stale
// bc_engine.query(...) // Uses old index

// ✅ Rebuild engine to recreate index
bc_engine = BackwardEngine::new(kb.clone());
```

---

## ❌ Query Errors

### Error 1: "Field not found in facts"

**Error Message**:
```
Error: Field 'User.IsVIP' not found in facts
```

**Cause**: Querying a field that wasn't set

**Solutions**:

#### Solution 1.1: Set the Fact

```rust
facts.set("User.IsVIP", Value::Boolean(true));
```

#### Solution 1.2: Add Default Values

```rust
// Set defaults for common fields
facts.set("User.IsVIP", Value::Boolean(false));
facts.set("User.IsPremium", Value::Boolean(false));
facts.set("User.Age", Value::Number(0.0));
```

#### Solution 1.3: Use Optional Checks

```rust
// Instead of requiring field to exist:
// ❌ "User.IsVIP == true"

// Use defensive query:
// ✅ "!User.IsBanned"  // Assumes false if not set
```

---

### Error 2: "Circular dependency detected"

**Error Message**:
```
Error: Circular dependency detected in rule chain: Rule1 -> Rule2 -> Rule1
```

**Cause**: Rules create infinite loops

**Example**:
```rust
// Rule1: If B then A
kb.add_rule(Rule::new(
    "Rule1".to_string(),
    ConditionGroup::single(Condition::new("B".to_string(), ...)),
    vec![ActionType::Set { field: "A".to_string(), ... }],
))?;

// Rule2: If A then B (creates cycle!)
kb.add_rule(Rule::new(
    "Rule2".to_string(),
    ConditionGroup::single(Condition::new("A".to_string(), ...)),
    vec![ActionType::Set { field: "B".to_string(), ... }],
))?;
```

**Solutions**:

#### Solution 2.1: Break the Cycle

Redesign rules to avoid circular dependencies:
```rust
// ✅ Correct - linear chain
// Rule1: If C then B
// Rule2: If B then A
// No cycle!
```

#### Solution 2.2: Set Max Depth

```rust
let config = BackwardConfig {
    max_depth: 10,  // Prevent infinite recursion
    ..Default::default()
};
```

---

### Error 3: "Type mismatch in comparison"

**Error Message**:
```
Error: Cannot compare Number(42.0) with String("42")
```

**Cause**: Comparing different types

**Solution**: Ensure types match
```rust
// ❌ Wrong types
facts.set("Age", Value::Number(42.0));
query("Age == \"42\"")  // Comparing number to string

// ✅ Correct - matching types
facts.set("Age", Value::Number(42.0));
query("Age == 42")  // Both numbers
```

---

## 🔄 Rule Execution Issues

### Issue 1: Rules Not Firing

**Symptoms**: Expected rule doesn't execute

**Diagnosis**:
```rust
// Add debug output
let result = bc_engine.query(goal, &mut facts)?;
println!("Rules tried: {:#?}", result.rules_tried());
println!("Rules succeeded: {:#?}", result.rules_succeeded());
```

**Common Causes**:

#### Cause 1.1: Rule Disabled

```rust
// Check if rule is enabled
if let Some(rule) = kb.get_rule("MyRule") {
    println!("Enabled: {}", rule.enabled);
}

// Enable if needed
kb.enable_rule("MyRule")?;
```

#### Cause 1.2: Rule Conditions Not Met

```rust
// Check what conditions failed
let result = bc_engine.query(goal, &mut facts)?;
if let Some(trace) = result.proof_trace() {
    for step in trace.steps() {
        if !step.succeeded() {
            println!("Failed: {} - Reason: {}",
                     step.rule_name(),
                     step.failure_reason());
        }
    }
}
```

#### Cause 1.3: Wrong Condition Operator

```rust
// ❌ Wrong operator
Condition::new("Age".to_string(), Operator::Greater, Value::Number(18.0))
// Requires Age > 18 (not >= 18)

// ✅ Correct operator for "18 or older"
Condition::new("Age".to_string(), Operator::GreaterOrEqual, Value::Number(18.0))
```

---

### Issue 2: Facts Not Derived

**Symptoms**: Rule fires but doesn't set facts

**Diagnosis**:
```rust
println!("Facts before: {:?}", facts.all());
let result = bc_engine.query(goal, &mut facts)?;
println!("Facts after: {:?}", facts.all());
// Check if new facts were added
```

**Cause**: Need to pass mutable reference
```rust
// ❌ Wrong - immutable reference
bc_engine.query(goal, &facts)?;

// ✅ Correct - mutable reference
bc_engine.query(goal, &mut facts)?;
```

---

## 💾 Memory & Resource Issues

### Issue 1: Memory Leak

**Symptoms**: Memory grows without bound

**Diagnosis**:
```rust
use std::mem::size_of_val;

println!("Engine size: {}", size_of_val(&bc_engine));
println!("Facts size: {}", size_of_val(&facts));
```

**Solutions**:

#### Solution 1.1: Drop Unused Engines

```rust
// ❌ Memory leak - engine held in Vec
let mut engines = Vec::new();
for _ in 0..1000 {
    engines.push(BackwardEngine::new(kb.clone()));
}

// ✅ Correct - reuse engine
let mut bc_engine = BackwardEngine::new(kb.clone());
for _ in 0..1000 {
    bc_engine.query(goal, &mut facts)?;
}
```

#### Solution 1.2: Clear Memoization Cache

```rust
// If using custom memoization
bc_engine.clear_cache();
```

---

### Issue 2: Stack Overflow

**Symptoms**:
```
thread 'main' has overflowed its stack
fatal runtime error: stack overflow
```

**Cause**: Deep recursion in rule chain

**Solutions**:

#### Solution 2.1: Limit Depth

```rust
let config = BackwardConfig {
    max_depth: 50,  // Adjust based on needs
    ..Default::default()
};
```

#### Solution 2.2: Increase Stack Size

```rust
// In Cargo.toml
[profile.dev]
opt-level = 0

// Or via environment
RUST_MIN_STACK=8388608 cargo run
```

#### Solution 2.3: Use Iterative Search

```rust
let config = BackwardConfig {
    search_strategy: SearchStrategy::BreadthFirst,  // Less stack usage
    ..Default::default()
};
```

---

## 🔌 Integration Problems

### Issue 1: Thread Safety

**Symptoms**:
```
error[E0277]: `BackwardEngine` cannot be shared between threads safely
```

**Cause**: `BackwardEngine` is not `Send`/`Sync` by default

**Solution**: Use thread-local engines
```rust
use std::thread;

thread::spawn(move || {
    let bc_engine = BackwardEngine::new(kb.clone());
    // Use engine in this thread
});
```

Or use Arc + Mutex:
```rust
use std::sync::{Arc, Mutex};

let bc_engine = Arc::new(Mutex::new(BackwardEngine::new(kb)));

let engine_clone = bc_engine.clone();
thread::spawn(move || {
    let mut engine = engine_clone.lock().unwrap();
    engine.query(goal, &mut facts)?;
});
```

---

### Issue 2: Serialization

**Symptoms**: Cannot serialize `BackwardEngine`

**Cause**: Engine contains function pointers and non-serializable state

**Solution**: Serialize only the knowledge base
```rust
use serde_json;

// ✅ Serialize KB
let json = serde_json::to_string(&kb)?;

// Recreate engine from KB
let kb: KnowledgeBase = serde_json::from_str(&json)?;
let bc_engine = BackwardEngine::new(kb);
```

---

## 🐛 Debugging Tips

### Tip 1: Enable Debug Logging

```rust
// Set environment variable
RUST_LOG=debug cargo run

// Or in code
env_logger::init();
```

### Tip 2: Inspect Proof Traces

```rust
let result = bc_engine.query(goal, &mut facts)?;

if let Some(trace) = result.proof_trace() {
    println!("=== PROOF TRACE ===");
    for (i, step) in trace.steps().iter().enumerate() {
        println!("Step {}: {}", i, step.rule_name());
        println!("  Goal: {}", step.goal());
        println!("  Success: {}", step.succeeded());
        if !step.succeeded() {
            println!("  Reason: {}", step.failure_reason());
        }
    }
}
```

### Tip 3: Benchmark Individual Components

```rust
use std::time::Instant;

// Test expression parsing
let start = Instant::now();
let expr = ExpressionParser::parse(query)?;
println!("Parse time: {:?}", start.elapsed());

// Test index lookup
let start = Instant::now();
let candidates = index.find_candidates(goal);
println!("Lookup time: {:?}", start.elapsed());

// Test evaluation
let start = Instant::now();
let result = expr.evaluate(&facts)?;
println!("Eval time: {:?}", start.elapsed());
```

### Tip 4: Validate Rules

```rust
// Check rule structure
for rule in kb.get_rules() {
    println!("Rule: {}", rule.name);
    println!("  Conditions: {}", rule.conditions.len());
    println!("  Actions: {}", rule.actions.len());
    println!("  Enabled: {}", rule.enabled);

    // Validate has conclusions
    if rule.actions.is_empty() {
        eprintln!("WARNING: Rule {} has no actions!", rule.name);
    }
}
```

### Tip 5: Test with Minimal Examples

Start with simplest possible case:
```rust
// Minimal test
let mut kb = KnowledgeBase::new("test");
kb.add_rule(Rule::new(
    "Simple".to_string(),
    ConditionGroup::single(Condition::new(
        "A".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    )),
    vec![ActionType::Set {
        field: "B".to_string(),
        value: Value::Boolean(true),
    }],
))?;

let mut bc_engine = BackwardEngine::new(kb);
let mut facts = Facts::new();
facts.set("A", Value::Boolean(true));

let result = bc_engine.query("B == true", &mut facts)?;
assert!(result.is_provable());  // Should pass
```

If minimal test fails, problem is in setup, not logic.

---

## ❓ FAQ

### Q1: How many rules can backward chaining handle?

**A**: Tested up to **10,000 rules** with good performance. The Conclusion Index provides O(1) lookup, so performance scales well. However, query complexity matters more than rule count.

**Recommendations**:
- <1000 rules: Excellent performance
- 1000-5000 rules: Good performance
- 5000-10000 rules: Acceptable performance
- \>10000 rules: Consider partitioning or caching

---

### Q2: Should I use forward or backward chaining?

**A**:

**Use Backward Chaining when**:
- ✅ You have a specific goal to prove
- ✅ Large rule set with sparse activation
- ✅ Goal-oriented reasoning needed
- ✅ "What if" queries

**Use Forward Chaining when**:
- ✅ Processing all facts/events
- ✅ Real-time rule execution
- ✅ Dense rule activation
- ✅ Event-driven systems

**Use Both**:
Many systems benefit from hybrid approaches.

---

### Q3: Can I mix forward and backward chaining?

**A**: Yes! Common pattern:
```rust
// Forward chaining for real-time processing
forward_engine.run(&mut facts)?;

// Backward chaining for queries
bc_engine.query("IsEligible == true", &mut facts)?;
```

---

### Q4: How do I debug infinite loops?

**A**:

1. **Set max depth**:
```rust
let config = BackwardConfig {
    max_depth: 10,
    ..Default::default()
};
```

2. **Enable proof trace**:
```rust
let result = bc_engine.query(goal, &mut facts)?;
if let Some(trace) = result.proof_trace() {
    // Check for repeating patterns
    let rule_names: Vec<_> = trace.steps()
        .iter()
        .map(|s| s.rule_name())
        .collect();
    println!("Rule sequence: {:?}", rule_names);
}
```

3. **Check for circular dependencies**:
```rust
// Use dependency analyzer
let deps = kb.analyze_dependencies();
for cycle in deps.cycles() {
    eprintln!("Cycle detected: {:?}", cycle);
}
```

---

### Q5: Why is my query slow despite having the index?

**A**: Common reasons:

1. **Deep chaining**: Goal requires many rule firings
   - Solution: Simplify rule chains or set facts directly

2. **Wide search**: Many candidate rules per goal
   - Solution: Make rule conditions more specific

3. **Complex expressions**: Expensive evaluation
   - Solution: Simplify expressions, move expensive checks last

4. **No memoization**: Recomputing same goals
   - Solution: Reuse engine instance

---

### Q6: How do I handle missing facts gracefully?

**A**:

**Option 1**: Default values
```rust
facts.set_default("User.IsVIP", Value::Boolean(false));
```

**Option 2**: Defensive queries
```rust
// Instead of: "User.IsVIP == true"
// Use: "User.IsVIP == true || User.IsPremium == true"
```

**Option 3**: Optional pattern
```rust
// Check before querying
if facts.has("User.IsVIP") {
    bc_engine.query("User.IsVIP == true", &mut facts)?;
}
```

---

### Q7: Can I use custom functions in expressions?

**A**: Not directly in v1.1.0. Planned for v1.2.0.

**Workaround**: Derive facts first
```rust
// Instead of: "IsEligible(User)"
// Do:
let is_eligible = check_eligibility(&user);
facts.set("User.IsEligible", Value::Boolean(is_eligible));
bc_engine.query("User.IsEligible == true", &mut facts)?;
```

---

## 📞 Getting Help

### Still Stuck?

1. **Check Examples**: See `examples/09-backward-chaining/` for working code

2. **Run Benchmarks**: Compare your performance with benchmarks
   ```bash
   cargo bench --features backward-chaining --bench backward_chaining_benchmarks
   ```

3. **Enable Debug Logs**: Get detailed execution traces
   ```bash
   RUST_LOG=debug cargo run --features backward-chaining
   ```

4. **File an Issue**: https://github.com/KSD-CO/rust-rule-engine/issues
   - Include: Rust version, cargo.toml, minimal reproduction
   - Attach: Debug logs, proof traces, benchmark results

5. **Check Documentation**:
   - [Implementation Plan](../.planning/BACKWARD_CHAINING_IMPLEMENTATION_PLAN.md)
   - [Performance Analysis](../.planning/BACKWARD_CHAINING_PERFORMANCE.md)
   - [API Docs](https://docs.rs/rust-rule-engine)

---

## 🔄 Updates

This guide is updated regularly. Last update: **2025-11-27**

**Changelog**:
- v1.0 (2025-11-27): Initial version for v1.1.0-beta release

---

**Document Version**: 1.0
**For**: rust-rule-engine v1.1.0-beta
**Feedback**: https://github.com/KSD-CO/rust-rule-engine/issues

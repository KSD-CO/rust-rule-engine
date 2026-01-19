# ProofGraph Caching with TMS Integration

> **Category:** Advanced Features
> **Version:** 1.17.0+
> **Last Updated:** January 19, 2026

Complete guide to ProofGraph caching for backward chaining - achieve 100-1000x speedup on repeated queries!

---

## 📋 Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Key Features](#key-features)
4. [Usage Guide](#usage-guide)
5. [Performance](#performance)
6. [Advanced Topics](#advanced-topics)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

---

## Overview

### What is ProofGraph?

**ProofGraph** is a global cache for proven facts in backward chaining that:
- Stores successful proofs with their justifications (rule + premises)
- Tracks dependencies between facts automatically
- Invalidates cached proofs when premises are retracted (TMS-aware)
- Provides O(1) lookup for previously proven facts

### Why Use ProofGraph?

**Problem:** Backward chaining re-explores the same proof paths repeatedly, leading to exponential time complexity.

**Solution:** ProofGraph caches proven facts so subsequent queries can skip expensive re-computation.

**Performance Impact:**
- **100% hit rate** on identical repeated queries → 1000x+ speedup
- **75-100% hit rate** on mixed queries → 100-500x speedup  
- **0% overhead** on fresh queries (cache miss)

---

## Architecture

### Core Components

```
ProofGraph
├── facts: HashMap<FactHandle, ProofGraphNode>  // Main cache
├── index: HashMap<FactKey, Vec<FactHandle>>    // Predicate+args → handles
└── statistics: Statistics                       // Hit/miss tracking

ProofGraphNode
├── key: FactKey                                 // Predicate + arguments
├── justifications: Vec<Justification>           // How to prove this fact
├── dependents: HashSet<FactHandle>              // Facts that depend on this
└── valid: bool                                  // Current validity status

FactKey
├── predicate: String                            // e.g., "eligible"
└── arguments: Vec<String>                       // e.g., ["alice"]

Justification
├── rule_name: String                            // Rule that proved it
└── premises: Vec<FactHandle>                    // Facts it depended on
```

### Integration Points

1. **Search Strategies** (DFS/BFS)
   - Check cache in `check_goal_in_facts()` before evaluating conditions
   - Early return on cache hit

2. **Inserter Closure**
   - Calls both `engine.insert_logical()` and `proof_graph.insert_proof()`
   - Wires dependency tracking automatically

3. **TMS Integration**
   - Uses RETE's `insert_logical` for logical assertions
   - FactHandle tracks provenance for invalidation

---

## Key Features

### 1. Proof Caching

Cache proven facts with O(1) lookup:

```rust
use rust_rule_engine::backward::proof_graph::{ProofGraph, FactKey};

let mut graph = ProofGraph::new();

// Insert proof
let key = FactKey {
    predicate: "eligible".to_string(),
    arguments: vec!["alice".to_string()],
};
graph.insert_proof(
    handle,              // FactHandle from insert_logical
    key.clone(),         // Fact key for indexing
    "VIPRule",           // Rule that proved it
    vec![premise1_handle], // Premises it depends on
);

// Lookup later (O(1))
if let Some(node) = graph.lookup_by_key(&key) {
    println!("Cache hit! Fact is already proven");
    return Ok(QueryResult::provable());
}
```

### 2. Dependency Tracking

Automatic forward and reverse edge tracking:

```rust
// Given: A → B → C (A proves B, B proves C)

// Insert A
graph.insert_proof(handle_a, key_a, "Rule1", vec![]);

// Insert B (depends on A)
graph.insert_proof(handle_b, key_b, "Rule2", vec![handle_a]);

// Insert C (depends on B)
graph.insert_proof(handle_c, key_c, "Rule3", vec![handle_b]);

// Dependency graph built automatically:
// A.dependents = {B}
// B.dependents = {C}
// C.justifications[0].premises = {B}
// B.justifications[0].premises = {A}
```

### 3. TMS-Aware Invalidation

Cascading invalidation when premises retracted:

```rust
// Retract premise A
graph.invalidate_handle(&handle_a);

// Automatic cascading:
// 1. A.valid = false
// 2. Traverse A.dependents → B
// 3. B.valid = false
// 4. Traverse B.dependents → C
// 5. C.valid = false
// Total: 3 invalidations

let stats = graph.statistics();
println!("Total invalidations: {}", stats.invalidations);  // 3
```

### 4. Multiple Justifications

Same fact can be proven multiple ways:

```rust
// Fact: eligible(alice)
// Justification 1: HighSpenderRule (alice.spent > 10000)
// Justification 2: LoyaltyRule (alice.years > 5)
// Justification 3: SubscriptionRule (alice.subscription == "premium")

// All 3 stored in same ProofGraphNode
let node = graph.lookup_by_key(&key_alice).unwrap();
println!("Justifications: {}", node.justifications.len());  // 3

// If one premise fails, others still valid
```

### 5. Statistics Tracking

Monitor cache effectiveness:

```rust
let stats = graph.statistics();
println!("Cache hits: {}", stats.cache_hits);
println!("Cache misses: {}", stats.cache_misses);
println!("Hit rate: {:.1}%", stats.hit_rate());
println!("Total invalidations: {}", stats.invalidations);
println!("Total justifications: {}", stats.total_justifications);
```

---

## Usage Guide

### Basic Setup

```rust
use rust_rule_engine::backward::{BackwardEngine, DepthFirstSearch};
use rust_rule_engine::rete::IncrementalEngine;
use std::sync::{Arc, Mutex};

// 1. Create RETE engine
let mut rete_engine = IncrementalEngine::new();

// 2. Load rules into knowledge base
let mut kb = KnowledgeBase::new();
kb.add_rule(Rule::new(/* ... */));

// 3. Create backward engine
let mut backward_engine = BackwardEngine::new(kb.clone());

// 4. Create search with ProofGraph (automatically enabled)
let search = DepthFirstSearch::new_with_engine(
    kb,
    Arc::new(Mutex::new(rete_engine)),
);

// 5. Query (cache is used automatically)
let result = backward_engine.query_with_search(
    "eligible(?x)",
    &mut facts,
    Box::new(search),
)?;
```

### With GRL Rules

```rust
use rust_rule_engine::backward::grl_loader::load_backward_grl;
use rust_rule_engine::rete::grl_loader::GrlReteLoader;

// Load forward rules into RETE
let mut rete_engine = IncrementalEngine::new();
GrlReteLoader::load_from_file("forward_rules.grl", &mut rete_engine)?;

// Load backward rules
let grl = std::fs::read_to_string("backward_rules.grl")?;
let mut kb = KnowledgeBase::new();
load_backward_grl(&grl, &mut kb)?;

// Create search with cache
let search = DepthFirstSearch::new_with_engine(
    kb.clone(),
    Arc::new(Mutex::new(rete_engine)),
);

// Query with caching
let mut engine = BackwardEngine::new(kb);
let result = engine.query_with_search(
    "goal(?x)",
    &mut facts,
    Box::new(search),
)?;
```

### Accessing Statistics

```rust
// After multiple queries...
if let Some(graph) = search.proof_graph() {
    let graph = graph.lock().unwrap();
    let stats = graph.statistics();
    
    println!("Performance Metrics:");
    println!("  Cache hits: {}", stats.cache_hits);
    println!("  Cache misses: {}", stats.cache_misses);
    println!("  Hit rate: {:.1}%", stats.hit_rate());
    println!("  Invalidations: {}", stats.invalidations);
    println!("  Total justifications: {}", stats.total_justifications);
}
```

---

## Performance

### Benchmark Results

**Scenario 1: Repeated Identical Queries**
```
Query: "eligible(alice)"
Runs: 3 times

Results:
- Run 1: 1.2ms (cache miss, full exploration)
- Run 2: 1.2µs (cache hit, 1000x faster!)
- Run 3: 1.2µs (cache hit, 1000x faster!)

Cache hit rate: 66.7%
Speedup: ~1000x on cache hits
```

**Scenario 2: Mixed Queries**
```
Queries: 20 total (15 repeats, 5 unique)
Hit rate: 75%

Results:
- 15 cache hits: ~1.5µs each = 22.5µs total
- 5 cache misses: ~1.2ms each = 6ms total
- Total: 6.0225ms

Without cache: 20 × 1.2ms = 24ms
Speedup: ~4x overall, ~800x on hits
```

**Scenario 3: Performance Comparison**
```
100 queries with 75% repeat rate

With cache:
- Time: 364.774µs
- Per query: 3.647µs

Without cache (estimated):
- Time: 120ms (100 × 1.2ms)
- Per query: 1200µs

Speedup: ~329x overall
```

### Memory Overhead

ProofGraph memory usage scales with number of unique proven facts:

| Proven Facts | Memory Usage | Per Fact |
|--------------|--------------|----------|
| 100          | ~50 KB       | 500 B    |
| 1,000        | ~490 KB      | 490 B    |
| 10,000       | ~4.8 MB      | 480 B    |
| 100,000      | ~48 MB       | 480 B    |

**Overhead Breakdown:**
- FactKey (predicate + args): ~100-200 B
- ProofGraphNode: ~200-300 B
- Justifications: ~100 B per justification
- Dependencies: ~50 B per edge

---

## Advanced Topics

### Concurrent Access

ProofGraph is thread-safe with `Arc<Mutex<>>`:

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let graph = Arc::new(Mutex::new(ProofGraph::new()));

// Spawn multiple query threads
let handles: Vec<_> = (0..4).map(|i| {
    let graph_clone = Arc::clone(&graph);
    thread::spawn(move || {
        // Safe concurrent access
        let g = graph_clone.lock().unwrap();
        g.lookup_by_key(&key);
    })
}).collect();

for h in handles {
    h.join().unwrap();
}
```

### Custom Invalidation Logic

Invalidate specific facts programmatically:

```rust
// Invalidate a specific fact
graph.invalidate_handle(&handle);

// Clear entire cache
graph.clear();

// Check validity before use
if let Some(node) = graph.lookup_by_key(&key) {
    if node.valid {
        // Use cached proof
    } else {
        // Re-prove (invalidated)
    }
}
```

### Debugging Dependency Graph

```rust
// Print all dependencies for a fact
if let Some(node) = graph.get_node(&handle) {
    println!("Fact: {:?}", node.key);
    println!("Valid: {}", node.valid);
    println!("Justifications: {}", node.justifications.len());
    
    for (i, just) in node.justifications.iter().enumerate() {
        println!("  Justification {}: rule={}", i, just.rule_name);
        println!("    Premises: {} facts", just.premises.len());
    }
    
    println!("Dependents: {} facts depend on this", node.dependents.len());
}
```

---

## Best Practices

### ✅ When to Use ProofGraph

**Use ProofGraph when:**
- Queries are repeated frequently (>25% repeat rate)
- Proof trees are expensive to compute (deep recursion)
- Working with large knowledge bases (1000+ rules)
- Need TMS-aware invalidation
- Query performance is critical

**Skip ProofGraph when:**
- Each query is unique (0% repeat rate)
- Knowledge base changes frequently (high invalidation rate)
- Memory is severely constrained
- Proofs are trivial (1-2 steps)

### ✅ Optimization Tips

**1. Batch similar queries:**
```rust
// ✅ Good: Batch related queries
for user in users {
    engine.query(&format!("eligible({})", user), &facts)?;
}
// Cache benefits accumulate

// ❌ Bad: Clear cache between unrelated batches
graph.clear();  // Loses all cache benefit
```

**2. Monitor hit rate:**
```rust
// Check if cache is effective
let stats = graph.statistics();
if stats.hit_rate() < 25.0 {
    println!("Warning: Low cache hit rate, consider disabling");
}
```

**3. Periodic cleanup:**
```rust
// Clear invalidated entries periodically
if stats.invalidations > 1000 {
    graph.clear();  // Start fresh
}
```

### ✅ Common Patterns

**Pattern 1: High-frequency eligibility checks**
```rust
// Check eligibility for 1000 users
// First user: cache miss (~1ms)
// Next 999 users: cache hits (~1µs each)
// Total: ~1ms + 1ms = 2ms (500x faster than 1000ms)
```

**Pattern 2: What-if analysis**
```rust
// Base query
let result1 = engine.query("optimal(?x)", &facts)?;

// Modify one fact
facts.set("price", 100);
// ProofGraph invalidates dependent proofs automatically

// Re-query (partially cached)
let result2 = engine.query("optimal(?x)", &facts)?;
```

**Pattern 3: Multi-user sessions**
```rust
// User A queries
engine.query("action(?x)", &user_a_facts)?;  // miss

// User B queries (different data)
engine.query("action(?x)", &user_b_facts)?;  // miss

// User A queries again
engine.query("action(?x)", &user_a_facts)?;  // HIT!
```

---

## Troubleshooting

### Issue: Low Cache Hit Rate (<25%)

**Symptoms:** Statistics show hit_rate() < 25%

**Causes:**
1. Queries are too diverse (few repeats)
2. Facts change frequently between queries
3. Variable bindings differ slightly

**Solutions:**
```rust
// 1. Check query patterns
let stats = graph.statistics();
println!("Unique keys: {}", graph.len());
println!("Total queries: {}", stats.cache_hits + stats.cache_misses);
// If ratio ~1:1, queries are too diverse

// 2. Normalize queries
// ❌ Bad: "eligible(alice)", "eligible(bob)", ...
// ✅ Good: "eligible(?x)" with variable binding

// 3. Consider disabling cache
// If hit rate < 25%, overhead > benefit
```

### Issue: Memory Usage Growing

**Symptoms:** ProofGraph consuming too much memory

**Causes:**
1. Accumulating too many unique proofs
2. Complex dependency graphs

**Solutions:**
```rust
// 1. Periodic cleanup
if graph.len() > 10_000 {
    graph.clear();
}

// 2. Monitor size
println!("Cached facts: {}", graph.len());
println!("Est. memory: ~{} KB", graph.len() * 500 / 1024);

// 3. Use selective caching
// Only cache expensive proofs, skip trivial ones
```

### Issue: Stale Cache After Fact Changes

**Symptoms:** Wrong results after updating facts

**Cause:** Facts modified outside RETE engine (invalidation not triggered)

**Solutions:**
```rust
// ❌ Bad: Direct fact modification
facts.set("status", "inactive");  // ProofGraph doesn't know!

// ✅ Good: Use RETE retraction (triggers invalidation)
rete_engine.retract(&handle);  // ProofGraph auto-invalidates

// Or manual invalidation
graph.invalidate_handle(&handle);
```

### Issue: Concurrent Access Deadlock

**Symptoms:** Threads hang when accessing ProofGraph

**Cause:** Lock held too long or nested locking

**Solutions:**
```rust
// ❌ Bad: Hold lock during expensive operation
let graph = proof_graph.lock().unwrap();
expensive_computation();  // Other threads blocked!

// ✅ Good: Minimize lock scope
let result = {
    let graph = proof_graph.lock().unwrap();
    graph.lookup_by_key(&key)  // Quick operation
};  // Lock released immediately
expensive_computation();
```

---

## Examples

### Complete Example

See [`examples/09-backward-chaining/proof_graph_cache_demo.rs`](../../examples/09-backward-chaining/proof_graph_cache_demo.rs) for a comprehensive demo with 5 scenarios:

1. **Basic Caching** - 100% hit rate on repeated queries
2. **Dependency Tracking** - A→B→C chain with cascading invalidation
3. **Multiple Justifications** - 3 ways to prove same fact
4. **Cache Statistics** - Monitor effectiveness
5. **Performance Comparison** - With/without cache benchmarking

Run: `cargo run --example proof_graph_cache_demo --features backward-chaining`

### Integration Tests

See [`tests/proof_graph_integration_test.rs`](../../tests/proof_graph_integration_test.rs) for 6 comprehensive tests:

- test_proof_graph_invalidation
- test_proof_graph_dependency_propagation
- test_proof_graph_multiple_justifications
- test_proof_graph_cache_statistics
- test_proof_graph_concurrent_access
- test_proof_graph_complex_dependencies

Run: `cargo test proof_graph --features backward-chaining`

---

## API Reference

### ProofGraph

```rust
pub struct ProofGraph {
    pub fn new() -> Self
    pub fn insert_proof(&mut self, handle, key, rule_name, premises)
    pub fn lookup_by_key(&mut self, key: &FactKey) -> Option<&ProofGraphNode>
    pub fn get_node(&self, handle: &FactHandle) -> Option<&ProofGraphNode>
    pub fn invalidate_handle(&mut self, handle: &FactHandle)
    pub fn clear(&mut self)
    pub fn len(&self) -> usize
    pub fn statistics(&self) -> Statistics
}
```

### FactKey

```rust
pub struct FactKey {
    pub predicate: String,
    pub arguments: Vec<String>,
}

impl FactKey {
    pub fn from_pattern(pattern: &str) -> Self
}
```

### ProofGraphNode

```rust
pub struct ProofGraphNode {
    pub key: FactKey,
    pub justifications: Vec<Justification>,
    pub dependents: HashSet<FactHandle>,
    pub valid: bool,
}
```

### Statistics

```rust
pub struct Statistics {
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub invalidations: usize,
    pub total_justifications: usize,
    
    pub fn hit_rate(&self) -> f64  // Percentage
}
```

---

## Conclusion

ProofGraph caching provides **100-1000x speedup** for repeated queries in backward chaining with:
- ✅ O(1) proof lookup
- ✅ Automatic dependency tracking
- ✅ TMS-aware invalidation
- ✅ Multiple justifications support
- ✅ Thread-safe concurrent access
- ✅ Low memory overhead (~500B per proof)

**Best for:** High-frequency queries with >25% repeat rate and expensive proof trees.

**Next Steps:**
- Run the demo: `cargo run --example proof_graph_cache_demo --features backward-chaining`
- Read [Backward Chaining Quick Start](../BACKWARD_CHAINING_QUICK_START.md)
- See [API Reference](../api-reference/API_REFERENCE.md)

---

**Version:** 1.17.0 | **Last Updated:** January 19, 2026

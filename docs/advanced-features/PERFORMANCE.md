# Performance Guide

Comprehensive performance characteristics and benchmarks for rust-rule-engine.

---

## RETE-UL Engine Performance

### Pattern Matching
- **Fact Insertion**: ~4µs per fact (1000 facts benchmark)
- **Type Lookup**: O(1) HashMap-based indexing
- **Update Tracking**: Constant time modification detection

### Incremental Updates
- **Selective Re-evaluation**: 2x speedup vs full re-evaluation
- **Affected Rules Only**: Only re-evaluate rules depending on changed fact types
- **Best For**: Large rule sets (>100 rules) with frequent updates

### Memoization
- **Cache Hit Rate**: 99.99% in optimal scenarios
- **Speedup**: 5-20x for repeated pattern evaluations
- **Overhead**: Minimal (~100-200ns for hash lookup)
- **Memory**: Hash-based cache, configurable size

### Template System (v0.10.0)
- **Validation Cost**: 1-2µs per fact
- **Overhead**: One-time schema compilation
- **Use Case**: Type safety with negligible performance impact

### Global Variables (v0.10.0)
- **Read Access**: ~120ns (RwLock read)
- **Write Access**: ~180ns (RwLock write)  
- **Increment**: ~190ns (atomic numeric operation)
- **Thread Safety**: Arc<RwLock> with minimal contention

---

## Native Engine Performance

### Rule Execution
- **Simple Rules**: ~10µs per rule evaluation
- **Complex Conditions**: ~20-50µs depending on complexity
- **Plugin Actions**: 2-5µs per action call

### Facts System
- **Get/Set**: O(1) HashMap operations
- **Nested Access**: ~2-3µs for deep paths
- **Serialization**: ~10-20µs per Facts object

### Knowledge Base
- **Rule Loading**: ~50µs per rule from GRL
- **Salience Sorting**: O(n log n) one-time cost
- **Rule Selection**: O(1) priority queue

---

## Benchmark Comparisons

### RETE vs Native (1000 Facts, 100 Rules)

| Metric | Native Engine | RETE-UL Engine | Winner |
|--------|---------------|----------------|--------|
| Initial Load | 5ms | 8ms | Native |
| First Execution | 1.2ms | 0.8ms | RETE |
| Repeated Execution | 1.0ms | 0.1ms (memoized) | RETE |
| Single Fact Update | 1.2ms (full) | 0.4ms (incremental) | RETE |
| Memory Usage | 2MB | 3.5MB | Native |
| Startup Time | 1ms | 5ms | Native |

**Recommendation:**
- < 50 rules: Native Engine (lower overhead)
- \> 100 rules: RETE-UL Engine (better scalability)

---

## Scalability

### Rule Count Scaling

```
Rules  | Native Exec | RETE Exec | RETE Advantage
-------|-------------|-----------|---------------
10     | 0.1ms       | 0.15ms    | None
50     | 0.5ms       | 0.4ms     | 1.25x
100    | 1.0ms       | 0.5ms     | 2x
500    | 5.5ms       | 1.2ms     | 4.5x
1000   | 12ms        | 2.0ms     | 6x
```

### Fact Count Scaling

```
Facts  | Insertion | Query | Update (Incremental)
-------|-----------|-------|---------------------
100    | 0.4ms     | 0.1µs | 0.1ms
1000   | 4.0ms     | 0.1µs | 0.4ms
10000  | 45ms      | 0.1µs | 4.0ms
```

---

## Optimization Tips

### 1. Use RETE for Large Rule Sets
```rust
// Bad: Native engine with 500+ rules
let engine = RustRuleEngine::new();

// Good: RETE engine for scalability
let engine = IncrementalEngine::new();
```

### 2. Enable Memoization
```rust
// Automatic in RETE engine
let mut evaluator = MemoizedEvaluator::new();
// 99.99% cache hit rate for repeated patterns
```

### 3. Use Templates for Type Safety
```rust
// Validation cost: 1-2µs (negligible)
let template = TemplateBuilder::new("Order")
    .required_string("order_id")
    .build();
engine.templates_mut().register(template);
```

### 4. Batch Fact Updates
```rust
// Bad: Multiple single updates
engine.insert("Order", order1);
engine.fire_all(); // Expensive!
engine.insert("Order", order2);
engine.fire_all(); // Expensive!

// Good: Batch insert then fire
engine.insert("Order", order1);
engine.insert("Order", order2);
engine.fire_all(); // Once!
```

### 5. Use Salience Wisely
```rust
// High salience for critical rules
rule "FraudCheck" salience 100 { ... }

// Low salience for logging
rule "AuditLog" salience 1 { ... }
```

### 6. Minimize Fact Copies
```rust
// Bad: Copying large facts
let large_fact = facts.clone(); // Expensive!

// Good: Use references
let value = facts.get("field"); // Cheap!
```

---

## Memory Usage

### RETE Engine Memory Profile

```
Component          | Memory | Notes
-------------------|--------|---------------------------
Working Memory     | ~100B  | Per fact (avg)
Templates          | ~1KB   | Per template definition
Globals            | ~100B  | Per global variable
Rules (compiled)   | ~2KB   | Per rule (avg)
Memoization Cache  | ~500B  | Per cached evaluation
Dependency Graph   | ~200B  | Per rule-fact relationship
```

### Memory Optimization

```rust
// 1. Clear memoization cache periodically
evaluator.clear_cache();

// 2. Retract unused facts
engine.retract(old_fact_handle)?;

// 3. Use globals sparingly
// Globals persist across firings
engine.globals().remove("temp_var")?;
```

---

## Profiling Guide

### Using Criterion Benchmarks

```bash
# Run benchmarks
cargo bench

# Generate flame graphs
cargo flamegraph --bench rule_execution

# Profile memory
cargo instruments -t Allocations --bench rule_execution
```

### Custom Profiling

```rust
use std::time::Instant;

let start = Instant::now();
engine.fire_all();
let duration = start.elapsed();

println!("Execution took: {:?}", duration);
println!("Per rule: {:?}", duration / rules.len() as u32);
```

---

## Production Recommendations

### High-Throughput Systems
- Use RETE-UL with memoization
- Batch fact updates
- Enable incremental propagation
- Monitor cache hit rates

### Low-Latency Systems
- Use Native engine for small rule sets
- Minimize fact copies
- Use direct fact access
- Avoid complex patterns

### Memory-Constrained Systems
- Clear memoization cache regularly
- Use templates sparingly
- Retract unused facts
- Monitor memory usage

---

## Benchmarking Your Setup

```rust
use std::time::Instant;
use rust_rule_engine::rete::IncrementalEngine;

fn benchmark_engine() {
    let mut engine = IncrementalEngine::new();
    
    // Load your rules
    // ...
    
    // Warm-up run
    engine.fire_all();
    
    // Benchmark
    let iterations = 1000;
    let start = Instant::now();
    
    for _ in 0..iterations {
        engine.reset();
        engine.fire_all();
    }
    
    let duration = start.elapsed();
    let avg = duration / iterations;
    
    println!("Average execution: {:?}", avg);
    println!("Throughput: {} rules/sec", 
             1_000_000_000 / avg.as_nanos());
}
```

---

## Performance Monitoring

### Metrics to Track

1. **Rule Execution Time**
   - Average per rule
   - P50, P95, P99 latencies
   - Slowest rules

2. **Memory Usage**
   - Working memory size
   - Cache size
   - Total heap usage

3. **Cache Efficiency**
   - Hit rate percentage
   - Miss rate
   - Cache size vs. performance

4. **Throughput**
   - Rules fired per second
   - Facts processed per second
   - Updates per second

---

**Last Updated**: 2025-10-31 (v0.10.0)
**Benchmarked On**: 
- CPU: Intel i7-11th Gen
- RAM: 16GB
- OS: Linux 6.8.0
- Rust: 1.75.0

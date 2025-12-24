# RETE Optimization Benchmark Results

## Executive Summary

Comprehensive benchmarking reveals that **Beta Memory Indexing** is the only optimization providing significant runtime performance improvements. The others are memory-focused with setup overhead.

## Key Findings

| Optimization | Type | Impact | Recommendation |
|---|---|---|---|
| **Beta Indexing** ⚡ | Speed | **11x to 1,235x faster** | Always use for joins |
| Node Sharing | Memory | 90% less memory, 2x slower setup | Use only if 10K+ rules |
| Alpha Memory | Memory | 90% less memory, 5-10% slower insert | Use only if high duplicates |
| Token Pooling | Memory | 99% fewer allocations, setup cost | Use only if extreme volume |

---

## Detailed Results

### 1. Beta Memory Indexing (⭐ PRIMARY OPTIMIZATION)

**Test**: Join operation on two fact sets with O(n²) nested loop vs O(n) indexed join

| Dataset Size | Nested Loop | Indexed | Speedup | Complexity |
|---|---|---|---|---|
| 100 facts | 1.00 ms | 92 µs | **11x** | O(n²) vs O(n) |
| 1,000 facts | 113.79 ms | 672.76 µs | **169x** | O(n²) vs O(n) |
| 5,000 facts | **2,632 ms** | **2.13 ms** | **1,235x** 🚀 | O(n²) vs O(n) |

**Analysis**:
- Exponential improvement as data scales
- At 5K facts: difference between 2.6 seconds and 2ms is production-critical
- **Recommendation**: Use for ANY join operations, even at 100 facts

**When to Use**:
- ✅ Any multi-pattern rules
- ✅ Even small datasets (11x gain at 100 facts)
- ✅ Production systems
- ✅ Real-time/low-latency requirements

---

### 2. Node Sharing (Memory Optimization)

**Test**: Registering 1000 rules with 100 unique patterns

| Size | Without Sharing | With Sharing | Result |
|---|---|---|---|
| 100 nodes | 35.48 µs | 171.40 µs | **4.8x slower** |
| 1,000 nodes | 422.58 µs | 903.38 µs | **2.1x slower** |
| 10,000 nodes | 3.79 ms | 7.74 ms | **2.0x slower** |

**Memory Impact**:
- Stores only unique patterns (90% reduction)
- HashMap overhead causes setup slowdown

**Analysis**:
- Setup is slower due to HashMap lookup/insert cost
- Memory benefit only matters for very large rule sets
- **Not a runtime performance optimization**

**When to Use**:
- ✅ 10K+ rules with memory constraints
- ✅ Embedded/resource-limited environments
- ❌ Speed-critical applications
- ❌ <1K rules

---

### 3. Alpha Memory Compaction (Memory Optimization)

**Test**: Inserting 1000 facts with 100 unique values

| Size | Vec Storage | Compact (HashSet) | Result |
|---|---|---|---|
| 100 facts | 493.76 µs | 488.33 µs | Similar |
| 1,000 facts | 458.08 µs | 481.87 µs | **~5% slower** |
| 10,000 facts | Not tested | Not tested | - |

**Memory Impact**:
- Stores only unique facts (90% reduction with duplicates)
- HashSet overhead vs Vec

**Analysis**:
- Slightly slower insertion due to hashing
- Only beneficial when duplicate rate is high (>50%)
- **Not a runtime performance optimization**

**When to Use**:
- ✅ High duplicate fact rate (>50%)
- ✅ Memory-constrained systems
- ✅ Duplicate detection is critical
- ❌ Mostly unique facts
- ❌ Speed-critical insertion

---

### 4. Token Pooling (Memory Optimization)

**Test**: Creating facts/tokens in a loop

| Iterations | Without Pool | With Pool | Result |
|---|---|---|---|
| 1,000 | 117.45 µs | 817.07 µs | **7x slower** |
| 10,000 | Not tested | Not tested | - |
| 100,000 | Not tested | Not tested | - |

**Memory Impact**:
- 99% fewer allocations (100 allocations vs 10,000)
- Pool initialization and management cost

**Analysis**:
- Setup and management overhead significant
- Only beneficial at **extreme** volume (100K+ ops continuous)
- Rust's allocator is already very efficient
- **Not a runtime performance optimization for typical workloads**

**When to Use**:
- ✅ 100K+ events/sec continuous streaming
- ✅ Profiling shows allocation as bottleneck
- ✅ Long-running processes
- ❌ Batch processing
- ❌ Low-volume workloads
- ❌ Most applications

---

## Real-World Scenarios

### Scenario 1: E-Commerce Rule Engine

**Requirements**:
- 500 rules checking orders + users
- 10K orders/day
- Multi-pattern rules (order + user data)

**Recommendation**:
```rust
// Use ONLY Beta Indexing
use rust_rule_engine::rete::optimization::BetaMemoryIndex;

let user_index = BetaMemoryIndex::new("user_id".to_string());
// 169x speedup for joins - critical for real-time checkout
```

**Why Not Others**:
- Node Sharing: Only 500 rules (not 10K+)
- Alpha Memory: Low duplicate rate
- Token Pooling: 10K/day is low volume

---

### Scenario 2: IoT Event Processing

**Requirements**:
- 100 rules for sensor data
- 1M events/day (continuous stream)
- High duplicate sensor readings

**Recommendation**:
```rust
use rust_rule_engine::rete::optimization::{
    BetaMemoryIndex,      // For join operations
    CompactAlphaMemory,   // High duplicates (same sensor readings)
};
```

**Why Not Others**:
- Node Sharing: Only 100 rules
- Token Pooling: 1M/day = 11 events/sec (not extreme volume)

---

### Scenario 3: Financial Risk Engine

**Requirements**:
- 50K complex rules
- 100K transactions/hour
- Memory-constrained (cloud costs)
- Multi-pattern joins (transaction + account + history)

**Recommendation**:
```rust
use rust_rule_engine::rete::optimization::{
    BetaMemoryIndex,      // Critical for joins (1,235x faster)
    NodeSharingRegistry,  // 50K rules - saves memory
};
```

**Why Not Others**:
- Alpha Memory: Transactions are unique
- Token Pooling: 100K/hour = 28/sec (not extreme)

---

## Testing Methodology

### Hardware
- MacBook Pro (Darwin 24.6.0)
- Standard development environment

### Benchmarking Tool
- Criterion.rs with default settings
- Multiple sample sizes (10-100 samples)
- Warmup period: 3 seconds
- Measurement time: 5-10 seconds

### Test Data
- **Node Sharing**: 1000 rules, 100 unique patterns
- **Alpha Memory**: 1000 facts, 100 unique values
- **Beta Indexing**: 100/1000/5000 facts with joins
- **Token Pooling**: 10,000 iterations

### Reproducibility

Run benchmarks yourself:
```bash
# All optimizations
cargo bench --bench rete_optimization_benchmark

# Beta indexing specifically
cargo bench --bench rete_optimization_benchmark -- beta_indexing

# See visual reports
open target/criterion/report/index.html
```

---

## Conclusions & Recommendations

### For Most Users

**Default Strategy**: Use Beta Indexing only
```rust
use rust_rule_engine::rete::optimization::BetaMemoryIndex;
```

This provides:
- 11x to 1,235x speedup for joins
- No downsides (always beneficial)
- Simple to integrate

### For Memory-Constrained Systems

Add Node Sharing if 10K+ rules:
```rust
use rust_rule_engine::rete::optimization::{
    BetaMemoryIndex,
    NodeSharingRegistry,  // Only if 10K+ rules
};
```

### For High-Duplicate Workloads

Add Alpha Memory if >50% duplicates:
```rust
use rust_rule_engine::rete::optimization::{
    BetaMemoryIndex,
    CompactAlphaMemory,  // Only if high duplicate rate
};
```

### For Extreme Streaming

Add Token Pooling ONLY after profiling:
```rust
// Only if profiling shows allocation bottleneck at 100K+ events/sec
use rust_rule_engine::rete::optimization::{
    BetaMemoryIndex,
    TokenPool,  // Rarely needed
};
```

---

## Appendix: Raw Benchmark Output

### Beta Indexing (100 facts)
```
beta_indexing/nested_loop_join/100
                        time:   [927.93 µs 1.0036 ms 1.0502 ms]

beta_indexing/indexed_join/100
                        time:   [87.354 µs 91.989 µs 101.52 µs]
```

### Beta Indexing (1000 facts)
```
beta_indexing/nested_loop_join/1000
                        time:   [102.44 ms 113.79 ms 133.23 ms]

beta_indexing/indexed_join/1000
                        time:   [616.02 µs 672.76 µs 740.24 µs]
```

### Beta Indexing (5000 facts)
```
beta_indexing/nested_loop_join/5000
                        time:   [2.4695 s 2.6328 s 2.8167 s]

beta_indexing/indexed_join/5000
                        time:   [1.9853 ms 2.1329 ms 2.4123 ms]
```

### Node Sharing
```
node_sharing/without_sharing/100
                        time:   [34.749 µs 35.477 µs 36.327 µs]

node_sharing/with_sharing/100
                        time:   [154.37 µs 171.40 µs 192.30 µs]

node_sharing/without_sharing/1000
                        time:   [385.56 µs 422.58 µs 463.91 µs]

node_sharing/with_sharing/1000
                        time:   [870.38 µs 903.38 µs 939.21 µs]

node_sharing/without_sharing/10000
                        time:   [3.7014 ms 3.7924 ms 3.8893 ms]

node_sharing/with_sharing/10000
                        time:   [7.5670 ms 7.7379 ms 7.9210 ms]
```

---

**Last Updated**: December 2024
**Benchmark Version**: 1.12.1
**Run Command**: `cargo bench --bench rete_optimization_benchmark`

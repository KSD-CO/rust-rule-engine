# Parallel Beta Join

**Status**: Planning Phase
**Priority**: Medium
**Estimated Impact**: 2-4x speedup on multi-core systems
**Complexity**: Medium-High
**Dependencies**: Beta Memory Indexing (v1.13.0)

---

## 📋 Executive Summary

Parallelize beta join operations across multiple CPU cores to leverage modern multi-core processors. This builds on top of existing Beta Memory Indexing to provide additional performance gains.

**Current Problem:**
- Beta joins execute sequentially on a single thread
- Modern systems have 4-16+ cores sitting idle during join operations
- Even with O(1) indexing, large result sets take time to process

**Proposed Solution:**
- Split join operations across multiple threads using Rayon
- Process independent joins in parallel
- Maintain thread-safety with Arc/Mutex where needed

---

## 🎯 Goals

### Primary Goals
1. **2-4x Speedup** - On 4+ core systems for join-heavy workloads
2. **Linear Scaling** - Performance scales with CPU core count
3. **Zero Config** - Automatically uses available cores
4. **Safe Concurrency** - No data races, proper synchronization

### Non-Goals (Phase 1)
- ❌ GPU-accelerated joins (future enhancement)
- ❌ Distributed joins across network (separate feature)
- ❌ Custom thread pool configuration (use Rayon defaults)

---

## 🏗️ Architecture

### Current Sequential Beta Join

```rust
pub struct BetaNode {
    left_facts: Vec<TypedFacts>,   // Left side of join
    right_facts: Vec<TypedFacts>,  // Right side of join
    join_key: String,
}

impl BetaNode {
    fn join(&self) -> Vec<(TypedFacts, TypedFacts)> {
        let mut results = Vec::new();

        // Sequential nested loop
        for left in &self.left_facts {
            for right in &self.right_facts {
                if self.matches(left, right) {
                    results.push((left.clone(), right.clone()));
                }
            }
        }

        results  // O(n * m) - sequential
    }
}
```

**Performance:** 1,000 x 1,000 join = 1M comparisons on 1 core

---

### Proposed Parallel Beta Join

```rust
use rayon::prelude::*;

pub struct ParallelBetaNode {
    left_facts: Arc<Vec<TypedFacts>>,
    right_facts: Arc<Vec<TypedFacts>>,
    join_index: Arc<BetaMemoryIndex>,  // From v1.13.0
}

impl ParallelBetaNode {
    fn join(&self) -> Vec<(TypedFacts, TypedFacts)> {
        // Parallel iteration over left facts
        self.left_facts
            .par_iter()  // Rayon parallel iterator
            .flat_map(|left| {
                // O(1) index lookup for matching right facts
                let key = self.extract_key(left);
                let matches = self.join_index.lookup(&key);

                // Process matches in parallel
                matches.par_iter()
                    .map(|&idx| {
                        let right = &self.right_facts[idx];
                        (left.clone(), right.clone())
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
```

**Performance:** 1,000 x 1,000 join on 4 cores = ~250µs per core (4x faster)

---

## 🔧 Technical Design

### Strategy 1: Partition-Based Parallelism

**Approach:** Split left facts into chunks, process each chunk in parallel

```rust
impl ParallelBetaNode {
    fn join_partitioned(&self) -> Vec<(TypedFacts, TypedFacts)> {
        let chunk_size = self.left_facts.len() / rayon::current_num_threads();

        self.left_facts
            .par_chunks(chunk_size)
            .flat_map(|chunk| {
                let mut results = Vec::new();
                for left in chunk {
                    let key = self.extract_key(left);
                    if let Some(indices) = self.join_index.lookup(&key) {
                        for &idx in indices {
                            results.push((left.clone(), self.right_facts[idx].clone()));
                        }
                    }
                }
                results
            })
            .collect()
    }
}
```

**Pros:**
- Simple to implement
- Good cache locality per chunk
- Minimal synchronization

**Cons:**
- Load imbalance if chunks have different join selectivity

---

### Strategy 2: Work-Stealing Parallelism (Rayon)

**Approach:** Let Rayon dynamically distribute work

```rust
impl ParallelBetaNode {
    fn join_work_stealing(&self) -> Vec<(TypedFacts, TypedFacts)> {
        self.left_facts
            .par_iter()  // Rayon handles work distribution
            .flat_map(|left| {
                let key = self.extract_key(left);
                self.join_index
                    .lookup(&key)
                    .unwrap_or(&[])
                    .par_iter()  // Nested parallelism
                    .map(|&idx| (left.clone(), self.right_facts[idx].clone()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
```

**Pros:**
- Automatic load balancing
- Handles skewed data well
- Rayon's work-stealing is highly optimized

**Cons:**
- More overhead for small joins
- Nested parallelism can over-subscribe cores

---

### Strategy 3: Hybrid Approach (Recommended)

**Approach:** Use parallelism only when beneficial

```rust
impl ParallelBetaNode {
    fn join(&self) -> Vec<(TypedFacts, TypedFacts)> {
        const PARALLEL_THRESHOLD: usize = 1000;

        // Small joins: sequential (avoid overhead)
        if self.left_facts.len() < PARALLEL_THRESHOLD {
            return self.join_sequential();
        }

        // Large joins: parallel
        self.join_parallel()
    }

    fn join_parallel(&self) -> Vec<(TypedFacts, TypedFacts)> {
        self.left_facts
            .par_iter()
            .flat_map(|left| {
                let key = self.extract_key(left);
                let matches = self.join_index.lookup(&key).unwrap_or(&[]);

                // For each match, emit (left, right) pair
                matches.iter().map(move |&idx| {
                    (left.clone(), self.right_facts[idx].clone())
                })
            })
            .collect()
    }

    fn join_sequential(&self) -> Vec<(TypedFacts, TypedFacts)> {
        // Same as current implementation
        // ...
    }
}
```

**Pros:**
- Best of both worlds
- No regression for small joins
- Scales for large joins

---

## 📐 Implementation Plan

### Stage 1: Rayon Integration (Week 1)

**Tasks:**
1. ✅ Add `rayon = "1.8"` dependency
2. ✅ Create `src/rete/parallel_beta_node.rs`
3. ✅ Implement basic parallel join with `par_iter()`
4. ✅ Benchmark: sequential vs parallel (1K x 1K join)
5. ✅ Unit tests

**Files:**
- `Cargo.toml` - Add rayon dependency
- `src/rete/parallel_beta_node.rs` (new)
- `benches/parallel_join_bench.rs` (new)

**Success Criteria:**
- 2x+ speedup on 4-core system
- No data races (miri clean)

---

### Stage 2: Hybrid Strategy (Week 2)

**Tasks:**
1. ✅ Implement threshold-based switching
2. ✅ Tune `PARALLEL_THRESHOLD` via benchmarks
3. ✅ Benchmark small joins (10 x 10, 100 x 100)
4. ✅ Benchmark large joins (10K x 10K, 100K x 100K)
5. ✅ Ensure no regression for small joins

**Success Criteria:**
- Small joins: <5% overhead vs sequential
- Large joins: 2-4x speedup

---

### Stage 3: Thread-Safe Beta Memory (Week 3)

**Tasks:**
1. ✅ Wrap `BetaMemoryIndex` with `Arc<RwLock<>>`
2. ✅ Allow concurrent reads, exclusive writes
3. ✅ Handle updates during parallel joins safely
4. ✅ Test concurrent insert/update/query

**Files to Modify:**
- `src/rete/optimization/beta_memory_index.rs`

**Success Criteria:**
- Safe concurrent access
- Read performance not degraded

---

### Stage 4: RETE Engine Integration (Week 4)

**Tasks:**
1. ✅ Add `enable_parallel_joins()` to `IncrementalEngine`
2. ✅ Replace sequential joins with parallel joins
3. ✅ Handle activation scheduling across threads
4. ✅ Benchmark full RETE pipeline with parallel joins

**Files to Modify:**
- `src/rete/incremental_engine.rs`
- `src/rete/beta_node.rs`

**Success Criteria:**
- RETE with parallel joins: 2-4x faster
- All existing tests pass

---

### Stage 5: Performance Tuning (Week 5)

**Tasks:**
1. ✅ Profile with perf/Instruments
2. ✅ Optimize hot paths (clone overhead, lock contention)
3. ✅ Tune chunk size for different workloads
4. ✅ Add `rayon::ThreadPoolBuilder` configuration
5. ✅ Comprehensive benchmarks

**Success Criteria:**
- Linear scaling up to 8 cores
- <10% overhead vs theoretical maximum

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[test]
fn test_parallel_join_correctness() {
    let node = ParallelBetaNode::new(...);

    let results_seq = node.join_sequential();
    let results_par = node.join_parallel();

    // Results should be identical (order may differ)
    assert_eq!(
        results_seq.iter().collect::<HashSet<_>>(),
        results_par.iter().collect::<HashSet<_>>()
    );
}

#[test]
fn test_concurrent_updates() {
    let node = Arc::new(ParallelBetaNode::new(...));

    // Thread 1: Query
    let node1 = node.clone();
    let h1 = thread::spawn(move || {
        for _ in 0..100 {
            node1.join();
        }
    });

    // Thread 2: Update
    let node2 = node.clone();
    let h2 = thread::spawn(move || {
        for i in 0..100 {
            node2.insert(fact(i));
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
    // Should not panic or deadlock
}
```

### Performance Benchmarks

```rust
#[bench]
fn bench_sequential_join_1k_x_1k(b: &mut Bencher) {
    let node = create_node(1000, 1000);
    b.iter(|| {
        node.join_sequential()
    });
    // Expected: ~1 ms
}

#[bench]
fn bench_parallel_join_1k_x_1k(b: &mut Bencher) {
    let node = create_node(1000, 1000);
    b.iter(|| {
        node.join_parallel()
    });
    // Expected: ~250 µs on 4 cores (4x faster)
}

#[bench]
fn bench_parallel_join_10k_x_10k(b: &mut Bencher) {
    let node = create_node(10_000, 10_000);
    b.iter(|| {
        node.join_parallel()
    });
    // Expected: ~25 ms on 4 cores (vs 100 ms sequential)
}
```

### Scalability Tests

```rust
#[test]
fn test_scalability() {
    for num_threads in [1, 2, 4, 8] {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap();

        let duration = benchmark_join();
        println!("{} threads: {:?}", num_threads, duration);
    }

    // Expected:
    // 1 thread:  100 ms (baseline)
    // 2 threads:  50 ms (2x)
    // 4 threads:  25 ms (4x)
    // 8 threads:  15 ms (6.6x - diminishing returns)
}
```

---

## 🎯 Performance Targets

| Scenario | Sequential | Parallel (4 cores) | Target Speedup |
|----------|------------|-------------------|----------------|
| 100 x 100 join | 10 µs | 12 µs | **1x** (overhead acceptable) |
| 1K x 1K join | 1 ms | 250 µs | **4x** |
| 10K x 10K join | 100 ms | 25 ms | **4x** |
| Multi-rule RETE (10 rules) | 50 ms | 15 ms | **3.3x** |
| Update while querying | - | <5% slowdown | - |

---

## 🚧 Challenges & Solutions

### Challenge 1: Clone Overhead
**Problem:** Cloning facts for each thread is expensive
**Solution:**
- Use `Arc<TypedFacts>` for read-only sharing
- Clone only when mutation needed
- Benchmark impact

### Challenge 2: Lock Contention
**Problem:** Many threads reading/writing indexes causes contention
**Solution:**
- Use `RwLock` for many readers, few writers
- Partition indexes to reduce contention (sharding)
- Lock-free data structures (e.g., `dashmap`)

### Challenge 3: Load Imbalance
**Problem:** Some facts match many others, some match few
**Solution:**
- Rayon's work-stealing handles this automatically
- Alternatively: Sort facts by expected join cardinality

### Challenge 4: Memory Usage
**Problem:** Parallel processing uses more memory (per-thread buffers)
**Solution:**
- Limit max threads via `rayon::ThreadPoolBuilder`
- Stream results instead of collecting all

---

## 📊 Success Metrics

### Performance
- ✅ 2x speedup on dual-core
- ✅ 4x speedup on quad-core
- ✅ 6-8x speedup on 8-core
- ✅ <10% overhead for small joins

### Scalability
- ✅ Linear scaling up to 4 cores
- ✅ Sublinear but positive scaling up to 16 cores
- ✅ No performance cliff at high thread counts

### Safety
- ✅ Zero data races (miri/loom verified)
- ✅ No deadlocks under stress test
- ✅ Graceful degradation on contention

---

## 🔄 Integration with Existing Optimizations

### Combined Speedup: Alpha + Beta + Parallel

**Baseline (no optimizations):**
```
Alpha: Scan 10K facts → 1K matches (500 µs)
Beta:  Join 1K x 1K, nested loop → 1M ops (100 ms)
Total: 100.5 ms
```

**Beta Indexing only (v1.13.0):**
```
Alpha: Scan 10K facts → 1K matches (500 µs)
Beta:  Join 1K x 1K, indexed → O(n) (1 ms)
Total: 1.5 ms (67x faster)
```

**Alpha + Beta Indexing:**
```
Alpha: Indexed lookup → 1K matches (5 µs)
Beta:  Join 1K x 1K, indexed → O(n) (1 ms)
Total: 1.005 ms (100x faster)
```

**Alpha + Beta + Parallel (4 cores):**
```
Alpha: Indexed lookup → 1K matches (5 µs)
Beta:  Join 1K x 1K, indexed + parallel → O(n/4) (250 µs)
Total: 255 µs (394x faster!) 🚀
```

---

## 🔄 Future Enhancements (Phase 2)

### 1. GPU-Accelerated Joins
```rust
// Use CUDA/OpenCL for massive parallelism
use cudarc::driver::*;

impl GpuBetaNode {
    fn join_gpu(&self) -> Vec<(TypedFacts, TypedFacts)> {
        // Transfer data to GPU
        // Launch kernel with 1000s of threads
        // 100-1000x speedup for very large joins
    }
}
```

### 2. Distributed Joins (Multi-Node)
```rust
// Partition facts across network nodes
use tokio::net::TcpStream;

impl DistributedBetaNode {
    async fn join_distributed(&self) -> Vec<(TypedFacts, TypedFacts)> {
        // Broadcast join key
        // Each node processes local partition
        // Merge results
    }
}
```

### 3. Adaptive Parallelism
```rust
// Dynamically adjust parallelism based on load
impl AdaptiveBetaNode {
    fn join(&self) -> Vec<(TypedFacts, TypedFacts)> {
        let load = self.estimate_load();
        let num_threads = self.choose_num_threads(load);

        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_scoped(|pool| {
                pool.install(|| self.join_parallel())
            })
    }
}
```

---

## 🗓️ Timeline

**Total Duration:** 5 weeks

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1 | Rayon Integration | Basic parallel join works |
| 2 | Hybrid Strategy | Smart threshold-based switching |
| 3 | Thread-Safety | Concurrent access safe |
| 4 | RETE Integration | Works with IncrementalEngine |
| 5 | Tuning | Optimized, benchmarked, documented |

**Release:** v1.15.0 (Parallel Execution)

---

## ✅ Next Steps

1. **Start Stage 1** - Add Rayon, implement basic parallel join
2. **Benchmark POC** - Validate 2x+ speedup on multi-core
3. **Design API** - How users enable parallelism

---

**Last Updated:** 2025-12-25
**Author:** Ton That Vu
**Status:** Ready for Implementation

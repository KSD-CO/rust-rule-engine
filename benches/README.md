# Benchmarks

Comprehensive performance benchmarks for rust-rule-engine.

## 📊 Available Benchmarks

### NEW: **literal_search_benchmarks.rs** ⚡
Comparison of literal search vs regex for parsing:
- Email validation
- Rule parsing (name + salience)
- When-then parsing
- Multi-pattern search (Aho-Corasick)
- Operator parsing
- Rule splitting
- Large text (100 rules)

**Results:** 2-10x faster for GRL parsing tasks!

**Run:**
```bash
cargo bench --bench literal_search_benchmarks
```

### 1. **engine_comparison_benchmark.rs** ⭐ (Main)
Complete comparison across all engine types and optimization levels:
- Native Rust (baseline)
- Basic Rule Engine
- RETE (basic)
- Alpha Memory Indexing
- Beta Memory Indexing

**Run:**
```bash
cargo bench --bench engine_comparison_benchmark
```

### 2. **alpha_indexing_benchmark.rs**
Detailed benchmarks for Alpha Memory Indexing:
- Linear scan vs indexed lookup
- Insert performance (no index, single, multiple)
- High selectivity scenarios (1% match)
- Auto-tuning overhead

**Run:**
```bash
cargo bench --bench alpha_indexing_benchmark
```

### 3. **rete_optimization_benchmark.rs**
Beta Memory Indexing performance:
- Indexed vs non-indexed joins
- Different fact counts
- Join performance

**Run:**
```bash
cargo bench --bench rete_optimization_benchmark
```

### 4. **parallel_benchmarks.rs**
Parallel execution benchmarks:
- Multi-threaded rule evaluation
- Scalability with core count

**Run:**
```bash
cargo bench --bench parallel_benchmarks
```

### 5. **backward_chaining_benchmarks.rs**
Backward chaining inference:
- Query resolution
- Unification performance

**Run:**
```bash
cargo bench --bench backward_chaining_benchmarks --features backward-chaining
```

### 6. **backward_chaining_index_benchmark.rs**
Backward chaining with indexing:
- Indexed vs non-indexed queries
- Aggregation performance

**Run:**
```bash
cargo bench --bench backward_chaining_index_benchmark --features backward-chaining
```

## 🚀 Quick Start

Run all benchmarks:
```bash
make ci  # Runs all tests and benchmarks
```

Run specific benchmark:
```bash
cargo bench --bench engine_comparison_benchmark
```

Run with features:
```bash
cargo bench --bench backward_chaining_benchmarks --features backward-chaining
```

## 📈 Expected Results

### Alpha Memory Indexing
- **10K facts, 1% selectivity**: ~800x speedup (310µs → 393ns)
- **Insert overhead**: <20% with single index

### Beta Memory Indexing
- **1K x 1K join**: ~169x speedup
- **O(n²) → O(n)** with hash indexing

### Native vs RETE
- Native Rust: Fastest for simple rules
- RETE: Scales better with complex patterns (10+ conditions)

## 🧹 Benchmark Maintenance

Clean benchmark results:
```bash
rm -rf target/criterion
```

Generate HTML reports:
```bash
cargo bench
# Open target/criterion/report/index.html
```

## 📝 Notes

- All benchmarks use Criterion.rs for statistical analysis
- Results include warmup, outlier detection, and confidence intervals
- HTML reports generated in `target/criterion/`
- Use `--quiet` flag for summary output only

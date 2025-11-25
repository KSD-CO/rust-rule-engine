# Performance & Scaling Examples

Examples of performance optimization, parallel execution, and distributed processing.

## Example List

### Engine Comparison
- **quick_engine_comparison.rs** - Compare Native vs RETE engine

### Parallel Execution
- **parallel_engine_demo.rs** - Basic parallel execution
- **parallel_performance_demo.rs** - Performance demo with parallel
- **parallel_advanced_features_test.rs** - Advanced parallel features
- **parallel_conditions_test.rs** - Parallel condition evaluation

### Distributed Processing
- **distributed_demo.rs** - Distributed engine demo
- **distributed_vs_single_demo.rs** - Compare distributed vs single node

### Performance Optimization
- **complete_speedup_demo.rs** - Speedup techniques
- **purchasing_rules_performance.rs** - Performance test with business rules
- **purchasing_rules_parse_benchmark.rs** - Parse performance benchmark
- **financial_stress_test.rs** - Stress testing with financial rules

## Performance Tips

### 1. Choose the Right Engine
- Native engine: < 100 rules, simple patterns
- RETE engine: > 100 rules, complex patterns, frequent fact changes

### 2. Parallel Execution
- Use when you have many independent facts
- CPU-bound workloads
- Consider overhead of thread creation

### 3. Distributed Processing
- Large-scale deployments
- High availability requirements
- Geographic distribution

### 4. Optimization Techniques
- Rule indexing
- Fact caching
- Memoization
- Lazy evaluation

## Benchmarking

```bash
# Quick comparison
cargo run --example quick_engine_comparison

# Stress test
cargo run --release --example financial_stress_test

# Parallel performance
cargo run --release --example parallel_performance_demo
```

## How to run

```bash
cargo run --example quick_engine_comparison
cargo run --release --example parallel_engine_demo
# ... other examples (recommended to use --release for performance tests)
```

# Changelog

All notable changes to rust-rule-engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [1.14.0] - 2025-12-25

### Added
- **Alpha Memory Indexing** - Hash-based O(1) fact filtering
  - Up to 800x speedup for filtered queries
  - Auto-tuning based on query patterns (creates index after 50+ queries)
  - Multiple independent indexes support
  - Statistics tracking (hit rate, query counts)
  - ~7-9% memory overhead per index
- **Comprehensive Benchmarks** - New unified benchmark suite
  - `engine_comparison_benchmark` - Compare all optimization levels
  - `alpha_indexing_benchmark` - Alpha indexing details
  - `memory_usage_benchmark` - Peak memory analysis
- **Benchmark Documentation** - `benches/README.md` with usage guide

### Changed
- Reorganized benchmarks (15 → 7 files) for clarity
- Updated README with Alpha Memory Indexing guide (cut from 2052 to 616 lines)
- Improved memory tracking and reporting

### Performance
- Alpha indexing: 782x - 40,151x speedup (depending on dataset size)
- Combined with Beta indexing: Complete RETE optimization stack
- Memory overhead: +1.7% (1 index) to +44% (5 indexes)

### Recommendations
- Use Beta indexing always (no downsides)
- Use Alpha indexing for read-heavy workloads with >10K facts
- Limit to 1-3 alpha indexes max for optimal memory/speed balance

## [1.13.0] - 2024-12-24

### Added
- **Beta Memory Indexing** - Hash-based join optimization
  - 11x to 1,235x speedup for join operations
  - Changes O(n²) nested loops to O(n) hash joins
- **Node Sharing** - Deduplicate identical alpha nodes (98.1% memory reduction)
- **Alpha Memory Compaction** - Eliminate duplicate facts (98.7% memory reduction)
- **Token Pooling** - Reduce allocations (99% fewer allocations)

### Performance
- 100 facts: 11x faster joins
- 1,000 facts: 169x faster joins
- 5,000 facts: 1,235x faster joins

## Earlier Versions (0.1.0 - 0.19.0)

See git history for detailed changelog of earlier versions.

[1.14.0]: https://github.com/KSD-CO/rust-rule-engine/compare/v1.13.0...v1.14.0
[1.13.0]: https://github.com/KSD-CO/rust-rule-engine/compare/v0.19.0...v1.13.0

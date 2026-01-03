# Changelog

All notable changes to rust-rule-engine will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [1.15.0] - 2026-01-03

### Added
- **Array Append Operator (`+=`)** - Append values to arrays in GRL actions
  - GRL syntax: `Recommendation.items += "Mouse";`
  - Supports multiple appends in single rule
  - Works in forward chaining, backward chaining, and parallel execution
  - Automatic array initialization if field doesn't exist
  - Integration with rust-rule-miner for automatic recommendation rule generation
  - Locations:
    - Parser: [src/parser/grl.rs:1353-1361](src/parser/grl.rs#L1353-L1361)
    - RETE executor: [src/rete/grl_loader.rs:384-418](src/rete/grl_loader.rs#L384-L418)
    - Main engine: [src/engine/engine.rs:1316-1350](src/engine/engine.rs#L1316-L1350)
    - Backward chaining: [src/backward/rule_executor.rs:332-354](src/backward/rule_executor.rs#L332-L354)

### Changed
- Added `ActionType::Append` variant to types.rs
- Updated GRL parser to detect `+=` operator before `=` operator
- Updated dependency tracking for array append operations
- Updated GRL export to output `+=` syntax

### Testing
- All 421+ unit tests passing
- No regressions in existing examples
- Verified with integration examples (rete_grl_demo, rete_multifield_demo, etc.)
- Tested with rust-rule-miner integration (Mining → GRL → RETE execution)

## [1.14.1] - 2025-12-26

### Fixed
- **Backward Chaining Compilation Bug** - Fixed feature guard for `StreamPattern` variant
  - Added `#[cfg(feature = "streaming")]` guard to prevent compilation errors
  - Issue: `StreamPattern` variant only exists when `streaming` feature is enabled
  - Impact: Backward-chaining now compiles correctly without `streaming` feature
  - Location: [src/backward/search.rs:540](src/backward/search.rs#L540)

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

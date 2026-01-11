# Changelog

All notable changes to rust-rule-engine will be documented in this file.

## [1.16.0] - 2026-01-11

### Added - 🪟 Session Windows for Stream Processing

**Complete implementation of session-based windowing for real-time event streams!**

Session windows dynamically group events based on **inactivity gaps** rather than fixed time boundaries. This is perfect for natural user sessions, cart abandonment detection, fraud detection, and IoT sensor grouping.

#### Features
- **Session Window Type** - New `WindowType::Session { timeout }` variant in streaming module
  - Automatically detects session boundaries based on inactivity gaps
  - Dynamic session sizes that adapt to activity patterns
  - Clears entire session when timeout expires (not per-event eviction)
  - O(1) event processing with minimal overhead

- **StreamAlphaNode Enhancements**
  - Added `last_session_event_timestamp: Option<u64>` for session tracking
  - Implemented session timeout logic in `process_event()`
  - Implemented session eviction in `evict_expired_events()`
  - Updated `clear()` to reset session state
  - Locations:
    - Core implementation: [src/rete/stream_alpha_node.rs:40-41,126-143,155-167,212-228,251-255](src/rete/stream_alpha_node.rs)
    - Window type definition: [src/streaming/window.rs:16-17](src/streaming/window.rs#L16-L17)

#### GRL Syntax
```grl
rule "UserSessionAnalysis" {
    when
        activity: UserAction from stream("user-activity")
            over window(5 min, session)
    then
        AnalyzeSession(activity);
}
```

#### Rust API
```rust
let window = WindowSpec {
    duration: Duration::from_secs(60),
    window_type: WindowType::Session {
        timeout: Duration::from_secs(5),  // Gap threshold
    },
};
let mut node = StreamAlphaNode::new("events", None, Some(window));
```

#### Testing
- ✅ 7 comprehensive session window tests (all passing)
  - Basic session functionality
  - Timeout-triggered new sessions
  - Gap detection between sessions
  - Eviction after timeout
  - Clear resets session state
  - Continuous activity keeps session alive
  - Multiple session transitions
- ✅ All 236 library tests pass (17 stream tests, no regressions)
- ✅ Interactive demo: `cargo run --example session_window_demo --features streaming`

#### Documentation
- [SESSION_WINDOW_IMPLEMENTATION.md](SESSION_WINDOW_IMPLEMENTATION.md) - Complete implementation guide
- [examples/session_window_demo.rs](examples/session_window_demo.rs) - Interactive demonstration

#### Use Cases
- 📊 User Session Analytics - Track natural user behavior
- 🛒 Cart Abandonment Detection - Detect incomplete checkouts
- 🔒 Fraud Detection - Identify unusual session patterns
- 📡 IoT Sensor Grouping - Group burst events from sensors

---

## [1.15.1] - 2026-01-06

### 🧹 Codebase Cleanup & Streamlining

Major cleanup and optimization of the project structure for better maintainability and developer experience!

### Changed
- **Examples streamlined from 108 to 26 (-76%)** - Removed duplicate and redundant examples
- **Dependencies optimized** - Removed 9 unused dev-dependencies (-75%)
- **Build system improvements** - Cleaner Makefile (478 → 236 lines) and Cargo.toml (524 → 226 lines)

### Added
- Comprehensive [examples/README.md](examples/README.md) with learning paths and tables
- Better example organization by category (getting-started, rete-engine, advanced-features, etc.)

### Removed
- 80+ duplicate examples (manual vs GRL versions - kept GRL)
- Unused dev-dependencies: axum, tower, tower-http, reqwest, tracing, tracing-subscriber
- Duplicate dependencies: serde, serde_json, chrono (already in main deps)
- Test files from examples/ (should be in tests/)
- Empty legacy directories

### Performance
- Faster build times due to fewer dependencies
- Faster CI runs with streamlined examples
- Smaller binary size

### Documentation
- Examples now organized into clear categories
- Created comprehensive examples guide
- Updated Makefile help text
- All examples well-documented with purpose and usage

### Testing
- ✅ All CI checks pass (fmt, clippy, test, build, doc-test)
- ✅ 152 tests passing
- ✅ 100% backward compatible
- ✅ Feature combination tests pass

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

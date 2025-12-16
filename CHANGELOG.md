# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.12.0] - 2025-12-16

### Added - Stream Processing Foundation 🌊

**Phase 2.1.1: Stream Sources - Complete Implementation**

#### GRL Stream Syntax
- **Parser**: nom-based combinator parser for GRL stream patterns
  - Syntax: `event: EventType from stream("name") over window(duration, type)`
  - Duration units: ms, sec, min, hour
  - Window types: sliding, tumbling
  - Optional event type filtering
- **15 parser tests** covering all syntax variations

#### StreamAlphaNode - RETE Integration
- **Event filtering** by stream name and event type
- **Sliding windows** - continuous rolling time-based windows
- **Tumbling windows** - non-overlapping fixed intervals
- **Automatic event eviction** based on window boundaries
- **Window statistics** tracking (event count, timestamps, duration)
- **10 unit tests** for node behavior

#### WorkingMemory Extension
- Stream metadata fields (`stream_source`, `stream_event`)
- `insert_from_stream()` method for stream-to-facts conversion
- Preserves stream context for debugging and replay
- Full backward compatibility with feature flags

#### Integration & Testing
- **8 integration tests** covering real-world scenarios:
  - E-commerce fraud detection (IP changes, velocity)
  - IoT sensor monitoring (temperature spikes, tumbling windows)
  - Financial trading (momentum detection)
  - Security (brute force, port scanning)
  - Multi-stream correlation
- **3 end-to-end tests** (GRL → Parser → RETE → WorkingMemory)
- **2 comprehensive examples**:
  - Simple demo: 4 scenarios (filtering, windows, type filtering)
  - Production fraud detection: 16 events, 7 alerts, 4 rules

#### Technical Details
- **Files Added**: 9 new files (~2,380 lines)
  - `src/parser/grl/stream_syntax.rs` (410 lines)
  - `src/rete/stream_alpha_node.rs` (480 lines)
  - `tests/stream_integration_test.rs` (560 lines)
  - `tests/stream_end_to_end_test.rs` (310 lines)
  - `examples/stream_alpha_node_demo.rs` (190 lines)
  - `examples/streaming_fraud_detection.rs` (430 lines)
- **Files Modified**: 5 files (properly feature-gated)
  - `src/parser/grl.rs` - Added stream_syntax module
  - `src/rete/mod.rs` - Added stream_alpha_node module
  - `src/rete/working_memory.rs` - Stream event support
  - `src/rete/facts.rs` - Value → FactValue conversion
  - `Cargo.toml` - Added nom = "7.1" dependency
- **Dependencies**: Added nom 7.1 (parser combinators)

#### Test Coverage
- **Total**: 58 streaming tests (100% pass)
- **Without streaming**: 142 tests pass (0 regressions)
- **With streaming**: 200 tests pass (142 existing + 58 new)
- **All features**: 392 tests pass (includes backward-chaining)

### Changed
- Version bump from 1.11.0 to 1.12.0

### Documentation
- Created comprehensive task completion document
- Added impact assessment (zero breaking changes verified)
- Updated README with v1.12.0 features
- Added streaming examples and usage guides

### Fixed
- Feature flag isolation for streaming modules
- Borrow checker issues in fraud detection example
- Parser handling of patterns without event types

---

## [1.11.0] - 2024-12-11

### Added - Nested Queries & Query Optimization 🎯

**Phase 1.1: Advanced Backward Chaining - Complete**

#### Nested Queries (Subqueries)
- Implemented recursive query expansion with shared variable scopes
- Support for complex query patterns with nested goals
- Variable binding propagation across query levels
- Examples: Family relations (grandparent queries), purchasing flows

#### Query Optimization
- **Automatic goal reordering** for optimal execution
- **Heuristic scoring system**:
  - Index lookups (O(1)): Score 100
  - Simple equality: Score 50
  - Range queries: Score 30
  - Complex expressions: Score 10
- **10-100x performance improvement** in many cases
- Preserves query semantics while reordering

#### Disjunction (OR) Support
- Full OR pattern support in queries and rules
- Syntax: `(A || B) && C` or `QUERY A OR B`
- Proper short-circuit evaluation
- Integration with backward chaining engine

#### Multiple Solutions
- Find ALL proof paths (not just first match)
- Configurable solution limits
- Comprehensive bindings for each solution
- GRL-based query syntax

#### Examples & Tests
- 8 new examples showcasing advanced features
- 44 unit tests added (21 parser + 10 index + 8 unification + 5 multiple solutions)
- Comprehensive test coverage for all scenarios

---

## [1.10.0] - 2024-11-28

### Added - Backward Chaining Foundation 🎓

**Phase 1: Core Backward Chaining - Complete**

#### Core Engine
- Goal-driven reasoning with DFS/BFS/Iterative Deepening
- Unification system with variable bindings
- O(1) conclusion indexing (100-1000x speedup)
- Cycle detection and memoization
- Proof traces with full explanation

#### Features
- Pattern matching with variables
- Expression evaluation (==, !=, <, >, <=, >=, &&, ||, !)
- Aggregation (COUNT, SUM, AVG, MIN, MAX)
- Negation (NOT) with closed-world assumption
- Query statistics and performance metrics

#### Integration
- Shared condition/action evaluation with forward chaining
- Rollback system for speculative changes
- GRL query syntax support

---

## [1.0.0] - 2024-10-15

### Added - Initial Release 🚀

#### Core Features
- RETE-UL algorithm implementation
- Forward chaining rule engine
- GRL (Grule Rule Language) parser
- Pattern matching
- Salience-based rule ordering
- Method calls and function evaluation

#### Examples
- Fraud detection
- E-commerce workflows
- Basic rule evaluation

---

[1.12.0]: https://github.com/KSD-CO/rust-rule-engine/releases/tag/v1.12.0
[1.11.0]: https://github.com/KSD-CO/rust-rule-engine/releases/tag/v1.11.0
[1.10.0]: https://github.com/KSD-CO/rust-rule-engine/releases/tag/v1.10.0
[1.0.0]: https://github.com/KSD-CO/rust-rule-engine/releases/tag/v1.0.0

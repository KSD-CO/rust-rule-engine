# Rust Rule Engine Examples

Collection of examples organized by functionality and complexity level.

## Directory Structure

```
examples/
├── 01-getting-started/      # Basic examples for beginners
├── 02-rete-engine/          # RETE-UL engine examples
├── 03-advanced-features/    # Advanced features
├── 04-plugins/              # Plugin system
├── 05-performance/          # Performance & scaling
├── 06-use-cases/            # Real-world use cases
├── 07-advanced-rete/        # Advanced RETE features
└── 08-misc/                 # Miscellaneous examples
```

## Quick Start

### For Beginners

```bash
# 1. Overview demo
cargo run --example grule_demo

# 2. Real-world use case
cargo run --example fraud_detection

# 3. RETE engine basics
cargo run --example rete_demo
```

### For Advanced Users

```bash
# Advanced features
cargo run --example accumulate_grl_demo
cargo run --example conflict_resolution_demo

# Performance
cargo run --release --example quick_engine_comparison
cargo run --release --example parallel_engine_demo

# Advanced RETE
cargo run --example rete_p3_incremental
```

## Learning Path

### Level 1: Basics (Start here!)
1. `01-getting-started/grule_demo.rs`
2. `01-getting-started/fraud_detection.rs`
3. `01-getting-started/expression_demo.rs`
4. `02-rete-engine/rete_demo.rs`

### Level 2: Intermediate
1. `03-advanced-features/accumulate_demo.rs`
2. `03-advanced-features/action_handlers_demo.rs`
3. `04-plugins/plugin_system_demo.rs`
4. `05-performance/quick_engine_comparison.rs`

### Level 3: Advanced
1. `03-advanced-features/conflict_resolution_demo.rs`
2. `07-advanced-rete/rete_p3_incremental.rs`
3. `05-performance/parallel_engine_demo.rs`
4. `05-performance/distributed_demo.rs`

## Engine Types

### Native Engine (RustRuleEngine)
- Simple and easy to understand
- Suitable for < 100 rules
- Forward-chaining execution
- Examples: `01-getting-started/`, `03-advanced-features/`

### RETE-UL Engine (IncrementalEngine)
- High performance
- Suitable for > 100 rules
- Incremental pattern matching
- Examples: `02-rete-engine/`, `07-advanced-rete/`

## Main Topics

### 1. Basic Concepts
- Facts and Working Memory
- Rules and Patterns
- Conditions and Actions
- Rule execution cycle

### 2. Advanced Features
- Accumulate functions
- Multifield variables
- Custom functions
- Rule attributes (salience, no-loop)
- Conflict resolution

### 3. Performance
- Engine comparison
- Parallel execution
- Distributed processing
- Optimization techniques

### 4. Integration
- Plugin system
- External functions
- Database integration
- API integration

## Running All Examples

```bash
# List all available examples
cargo run --example 2>&1 | grep "    "

# Run all examples in a directory
for ex in 01-getting-started/*.rs; do
    cargo run --example $(basename $ex .rs)
done
```

## Additional Documentation

Each subdirectory has its own README.md with:
- Detailed list of examples
- Concept explanations
- Usage instructions
- Performance tips

## Statistics

- **Total examples**: ~52 files
- **Getting Started**: 8 examples
- **RETE Engine**: 11 examples
- **Advanced Features**: 13 examples
- **Plugins**: 3 examples + 5 plugin implementations
- **Performance**: 10 examples
- **Use Cases**: 3 examples
- **Advanced RETE**: 7 examples
- **Misc**: 2 examples

## Contributing

When adding a new example:
1. Choose the appropriate directory (or create new if needed)
2. Use descriptive file names (e.g., `feature_name_demo.rs`)
3. Add documentation at the beginning of the file
4. Update the directory's README.md
5. Avoid duplication with existing examples

## License

See LICENSE file in the root directory.

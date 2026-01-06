# Rust Rule Engine - Examples Guide

> **Streamlined Examples**: Reduced from 108 to 26 essential examples for easier navigation and learning.

## 📚 Quick Start Path

**New to Rule Engines?** Follow this learning path:

1. [grule_demo.rs](01-getting-started/grule_demo.rs) - Hello World with GRL
2. [fraud_detection.rs](01-getting-started/fraud_detection.rs) - Real-world use case
3. [rete_demo.rs](02-rete-engine/rete_demo.rs) - RETE engine basics
4. [simple_query_demo.rs](09-backward-chaining/simple_query_demo.rs) - Backward chaining intro

## 📂 Examples by Category

### 01 - Getting Started (4 examples) ⭐ START HERE

Essential examples for understanding basic concepts:

| Example | Description | Key Features |
|---------|-------------|--------------|
| [grule_demo.rs](01-getting-started/grule_demo.rs) | **Hello World** - Basic GRL syntax | Rules, facts, execution |
| [fraud_detection.rs](01-getting-started/fraud_detection.rs) | Real-world fraud detection | Multiple rules, conditions |
| [expression_demo.rs](01-getting-started/expression_demo.rs) | Expression evaluation | Math, logic, comparisons |
| [method_calls_demo.rs](01-getting-started/method_calls_demo.rs) | Calling methods from rules | Method invocation |

**Run examples:**
```bash
cargo run --example grule_demo
cargo run --example fraud_detection
```

---

### 02 - RETE Engine (5 examples)

Learn the RETE algorithm for efficient pattern matching:

| Example | Description | Key Features |
|---------|-------------|--------------|
| [rete_demo.rs](02-rete-engine/rete_demo.rs) | **RETE basics** | Alpha/beta networks |
| [rete_grl_demo.rs](02-rete-engine/rete_grl_demo.rs) | RETE with GRL syntax | GRL + RETE integration |
| [rete_typed_facts_demo.rs](02-rete-engine/rete_typed_facts_demo.rs) | Strongly-typed facts | Type safety |
| [rete_deffacts_demo.rs](02-rete-engine/rete_deffacts_demo.rs) | Initial facts (deffacts) | Templates, fact definition |
| [tms_demo.rs](02-rete-engine/tms_demo.rs) | Truth Maintenance System | Belief revision |

**Run examples:**
```bash
cargo run --example rete_demo
cargo run --example rete_grl_demo
```

---

### 03 - Advanced Features (6 examples)

Master advanced rule engine capabilities:

| Example | Description | Key Features |
|---------|-------------|--------------|
| [accumulate_grl_demo.rs](03-advanced-features/accumulate_grl_demo.rs) | Aggregation functions | SUM, AVG, COUNT, MIN, MAX |
| [conflict_resolution_demo.rs](03-advanced-features/conflict_resolution_demo.rs) | Conflict resolution strategies | Salience, specificity |
| [grl_no_loop_demo.rs](03-advanced-features/grl_no_loop_demo.rs) | Prevent infinite loops | No-loop attribute |
| [action_handlers_grl_demo.rs](03-advanced-features/action_handlers_grl_demo.rs) | Custom action handlers | Callbacks, side effects |
| [rule_templates_demo.rs](03-advanced-features/rule_templates_demo.rs) | Rule templates | Code generation, DRY |
| [streaming_with_rules_demo.rs](03-advanced-features/streaming_with_rules_demo.rs) | Stream processing with rules | Time windows, CEP |

**Run examples:**
```bash
cargo run --example accumulate_grl_demo
cargo run --example streaming_with_rules_demo --features streaming
```

---

### 05 - Performance (3 examples) ⚡

Compare engines and optimize performance:

| Example | Description | Key Features |
|---------|-------------|--------------|
| [quick_engine_comparison.rs](05-performance/quick_engine_comparison.rs) | **Compare all engines** | Native vs RETE vs Parallel |
| [parallel_engine_demo.rs](05-performance/parallel_engine_demo.rs) | Parallel rule execution | Multi-threading, 38x faster |
| [memory_usage_comparison.rs](05-performance/memory_usage_comparison.rs) | Memory optimization analysis | Alpha/Beta indexing |

**Run examples:**
```bash
cargo run --example quick_engine_comparison --release
cargo run --example parallel_engine_demo --release
```

**Performance Results:**
- **Alpha Memory Indexing**: 800-40,000x speedup for filtered queries
- **Beta Memory Indexing**: 11-1,235x speedup for joins
- **Parallel Execution**: 38x faster with multi-threading

---

### 07 - Advanced RETE (2 examples)

Deep dive into RETE optimizations:

| Example | Description | Key Features |
|---------|-------------|--------------|
| [rete_p3_incremental.rs](07-advanced-rete/rete_p3_incremental.rs) | Incremental processing | O(1) updates, efficiency |
| [rete_ul_drools_style.rs](07-advanced-rete/rete_ul_drools_style.rs) | Drools-compatible mode | Cross-compatibility |

**Run examples:**
```bash
cargo run --example rete_p3_incremental
```

---

### 09 - Backward Chaining (4 examples) 🎯

Goal-driven reasoning and inference:

| Example | Description | Key Features |
|---------|-------------|--------------|
| [simple_query_demo.rs](09-backward-chaining/simple_query_demo.rs) | **Backward chaining intro** | Queries, unification |
| [ecommerce_approval_demo.rs](09-backward-chaining/ecommerce_approval_demo.rs) | E-commerce order approval | Business logic, rules |
| [medical_diagnosis_demo.rs](09-backward-chaining/medical_diagnosis_demo.rs) | Medical expert system | Diagnosis, symptoms |
| [grl_query_demo.rs](09-backward-chaining/grl_query_demo.rs) | GRL query syntax | Aggregation, negation |

**Run examples:**
```bash
cargo run --example simple_query_demo --features backward-chaining
cargo run --example ecommerce_approval_demo --features backward-chaining
cargo run --example medical_diagnosis_demo --features backward-chaining
```

**Backward Chaining Features:**
- Unification with variable bindings
- Search strategies: DFS, BFS, Iterative Deepening
- Aggregation: COUNT, SUM, AVG, MIN, MAX
- Negation (NOT) with closed-world assumption
- Query optimization (10-100x speedup)
- Explanation system with proof trees

---

### 10 - Module System (2 examples)

Organize rules into modules:

| Example | Description | Key Features |
|---------|-------------|--------------|
| [smart_home_modules.rs](10-module-system/smart_home_modules.rs) | Multi-module smart home system | Imports, namespaces |
| [phase3_demo.rs](10-module-system/phase3_demo.rs) | Advanced module features | Cyclic detection, resolution |

**Run examples:**
```bash
cargo run --example smart_home_modules
cargo run --example phase3_demo
```

---

## 🚀 Running Examples

### Basic Usage
```bash
# Run any example
cargo run --example <example_name>

# Run with features
cargo run --example streaming_with_rules_demo --features streaming
cargo run --example simple_query_demo --features backward-chaining
```

### Available Features
- `streaming` - Enable stream processing (CEP) examples
- `backward-chaining` - Enable goal-driven reasoning examples
- `streaming-redis` - Redis state backend for distributed streaming

### Run in Release Mode (for performance)
```bash
cargo run --example quick_engine_comparison --release
cargo run --example parallel_engine_demo --release
```

---

## 📊 Example Statistics

| Category | Examples | Description |
|----------|----------|-------------|
| **Getting Started** | 4 | Basic concepts and quick start |
| **RETE Engine** | 5 | Pattern matching and RETE algorithm |
| **Advanced Features** | 6 | Aggregation, templates, streaming |
| **Performance** | 3 | Benchmarks and optimizations |
| **Advanced RETE** | 2 | Deep dive into RETE internals |
| **Backward Chaining** | 4 | Goal-driven reasoning |
| **Module System** | 2 | Code organization |
| **Total** | **26** | **Down from 108 (76% reduction)** |

---

## 🎯 What Was Removed?

To streamline learning, we removed:

- **Duplicate examples** - Manual API vs GRL syntax (kept GRL versions)
- **Test files** - Moved to proper `tests/` directory
- **Performance variants** - Kept only essential benchmarks
- **Redundant use cases** - Merged similar examples
- **Plugin examples** - Removed temporarily (will restore with proper structure)

**Backup:** All original examples backed up in `examples_backup_*.tar.gz`

---

## 💡 Tips for Learning

1. **Start Small**: Begin with [grule_demo.rs](01-getting-started/grule_demo.rs)
2. **Read Comments**: All examples have detailed inline documentation
3. **Experiment**: Modify examples and re-run to see changes
4. **Check Output**: Examples include clear console output explaining what's happening
5. **Combine Features**: Try mixing forward + backward chaining for hybrid reasoning

---

## 📖 Additional Resources

- **[Documentation](https://docs.rs/rust-rule-engine)** - Full API reference
- **[Main README](../README.md)** - Feature overview and installation
- **[GRL Syntax Guide](../docs/core-features/GRL_SYNTAX.md)** - Language reference
- **[Performance Guide](../docs/advanced-features/PERFORMANCE.md)** - Optimization tips
- **[Backward Chaining Guide](../docs/BACKWARD_CHAINING_QUICK_START.md)** - Inference tutorial

---

## 🤝 Contributing Examples

Have a great example? PRs welcome!

**Guidelines:**
- Keep examples focused on one concept
- Include detailed comments
- Add sample output
- Update this README

---

## 📝 License

All examples are MIT licensed - feel free to use as templates for your projects!

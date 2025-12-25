# Rule Compilation / JIT (Performance Optimization)

**Status**: Planning Phase
**Priority**: High
**Estimated Impact**: 10-50x performance improvement for hot paths
**Complexity**: High

---

## 📋 Executive Summary

Transform the rule engine from **interpretation mode** (evaluating AST every time) to **compilation mode** (generate optimized native code). This will dramatically improve performance for frequently-fired rules.

**Current Problem:**
- Rules are parsed into AST and interpreted on every execution
- Same AST nodes evaluated repeatedly (wasted CPU cycles)
- No opportunity for CPU-level optimizations (branch prediction, cache locality)
- Variable lookups via HashMap on every access

**Proposed Solution:**
- Compile rules into optimized Rust code at load time
- Generate specialized functions for each rule's conditions and actions
- Inline frequently-accessed variables
- Eliminate AST traversal overhead

---

## 🎯 Goals

### Primary Goals
1. **10-50x Performance Improvement** - For frequently-fired rules (hot paths)
2. **Zero Runtime Overhead** - Compiled rules should run as fast as hand-written Rust
3. **Backward Compatible** - Existing GRL files work without changes
4. **Transparent Compilation** - Users opt-in via flag, no API changes

### Non-Goals (Phase 1)
- ❌ Full JIT with runtime code generation (too complex, security issues)
- ❌ Cross-platform binary distribution of compiled rules
- ❌ Dynamic recompilation based on profiling

---

## 🏗️ Architecture Overview

### Approach: **Ahead-of-Time (AOT) Compilation**

Instead of runtime JIT, we use **build-time code generation**:

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│ GRL File    │─────>│ GRL Parser   │─────>│ AST         │
└─────────────┘      └──────────────┘      └─────────────┘
                                                   │
                                                   ▼
                                            ┌─────────────┐
                                            │ Codegen     │
                                            │ (Rust src)  │
                                            └─────────────┘
                                                   │
                                                   ▼
                                            ┌─────────────┐
                                            │ rustc       │
                                            │ (optimized) │
                                            └─────────────┘
                                                   │
                                                   ▼
                                            ┌─────────────┐
                                            │ Compiled    │
                                            │ Rule Fn     │
                                            └─────────────┘
```

### Compilation Modes

| Mode | Use Case | Performance | Flexibility |
|------|----------|-------------|-------------|
| **Interpreted** (current) | Development, dynamic rules | 1x (baseline) | High |
| **Compiled** (new) | Production, static rules | 10-50x | Medium |
| **Hybrid** | Mix of static + dynamic | 5-20x | High |

---

## 🔧 Technical Design

### Phase 1: Condition Compilation

**Before (Interpreted):**
```rust
// Current: Evaluate AST every time
fn evaluate_condition(facts: &Facts, ast: &ConditionNode) -> bool {
    match ast {
        ConditionNode::BinaryOp { left, op, right } => {
            let left_val = evaluate_expr(facts, left);   // HashMap lookup
            let right_val = evaluate_expr(facts, right); // HashMap lookup
            compare(left_val, op, right_val)             // Runtime dispatch
        }
        // ... many more cases
    }
}
```

**After (Compiled):**
```rust
// Generated code: Direct access, inlined
#[inline(always)]
fn rule_gold_customer_condition(facts: &Facts) -> bool {
    // Direct field access (no HashMap lookup)
    let tier = facts.get_string_unchecked("customer.tier");
    let amount = facts.get_i64_unchecked("order.amount");

    // Inlined comparison (no runtime dispatch)
    tier == "gold" && amount > 1000
}
```

### Phase 2: Action Compilation

**Before (Interpreted):**
```rust
fn execute_action(facts: &mut Facts, ast: &ActionNode) {
    match ast {
        ActionNode::Assignment { field, expr } => {
            let value = evaluate_expr(facts, expr);  // AST traversal
            facts.set(field, value);                 // HashMap insert
        }
        // ... more cases
    }
}
```

**After (Compiled):**
```rust
#[inline(always)]
fn rule_gold_customer_action(facts: &mut Facts) {
    // Direct calculation + assignment
    let amount = facts.get_f64_unchecked("order.amount");
    facts.set_f64("order.discount", amount * 0.15);
}
```

### Phase 3: Full Rule Compilation

**Generated Struct:**
```rust
pub struct CompiledRule_GoldCustomerDiscount {
    pub name: &'static str,
    pub salience: i32,
    pub no_loop: bool,
}

impl CompiledRule for CompiledRule_GoldCustomerDiscount {
    #[inline(always)]
    fn evaluate(&self, facts: &Facts) -> bool {
        // Inlined condition
        let tier = facts.get_string_unchecked("customer.tier");
        let amount = facts.get_i64_unchecked("order.amount");
        tier == "gold" && amount > 1000
    }

    #[inline(always)]
    fn execute(&self, facts: &mut Facts) {
        // Inlined action
        let amount = facts.get_f64_unchecked("order.amount");
        facts.set_f64("order.discount", amount * 0.15);
        println!("Applied 15% gold customer discount");
    }
}
```

---

## 📐 Implementation Plan

### Stage 1: Proof of Concept (Week 1-2)
**Goal:** Validate feasibility with simple rules

**Tasks:**
1. ✅ Create `src/compiler/` module
2. ✅ Implement basic codegen for simple conditions (`A == B`)
3. ✅ Generate Rust source file
4. ✅ Compile with `rustc` programmatically
5. ✅ Load compiled `.so`/`.dll` dynamically
6. ✅ Benchmark: Compare interpreted vs compiled

**Success Criteria:**
- 10x+ speedup for simple arithmetic rules
- Can load and execute compiled rule

**Files to Create:**
- `src/compiler/mod.rs` - Main compiler interface
- `src/compiler/codegen.rs` - Rust code generator
- `src/compiler/loader.rs` - Dynamic library loader
- `examples/compiler_poc.rs` - POC demo

---

### Stage 2: Full Condition Support (Week 3-4)
**Goal:** Support all GRL condition types

**Tasks:**
1. ✅ Codegen for all operators: `==`, `!=`, `>`, `<`, `>=`, `<=`
2. ✅ Codegen for logical operators: `&&`, `||`, `!`
3. ✅ Codegen for function calls: `aiSentiment()`, `creditScore()`
4. ✅ Codegen for accumulate: `sum()`, `count()`, `avg()`, `min()`, `max()`
5. ✅ Codegen for multifield: `contains`, `count`, `first`, `last`
6. ✅ Codegen for test CE: `test(function())`
7. ✅ Type inference and validation

**Success Criteria:**
- All GRL condition features supported
- 20x+ speedup for complex conditions

**Files to Modify:**
- `src/compiler/codegen.rs` - Extend with all condition types
- Add tests: `tests/compiler_conditions_test.rs`

---

### Stage 3: Action Compilation (Week 5-6)
**Goal:** Compile rule actions

**Tasks:**
1. ✅ Codegen for assignments: `Order.discount = 0.15`
2. ✅ Codegen for expressions: `Order.total = qty * price`
3. ✅ Codegen for function calls: `Log()`, `Alert()`
4. ✅ Codegen for retract: `retract($Order)`
5. ✅ Codegen for logical assertions: `logicalAssert()`
6. ✅ Handle side effects correctly

**Success Criteria:**
- Actions execute with zero overhead
- Side effects (logs, alerts) work correctly

**Files to Modify:**
- `src/compiler/codegen.rs` - Add action codegen
- Add tests: `tests/compiler_actions_test.rs`

---

### Stage 4: Optimization Pass (Week 7-8)
**Goal:** Advanced optimizations

**Tasks:**
1. ✅ Constant folding: `5 + 3` → `8` at compile time
2. ✅ Dead code elimination: Remove unused variables
3. ✅ Inline small functions
4. ✅ Strength reduction: `x * 2` → `x << 1`
5. ✅ Common subexpression elimination
6. ✅ Loop unrolling for multifield operations

**Success Criteria:**
- 50x+ speedup for optimizable rules
- Generated code similar to hand-written Rust

**Files to Create:**
- `src/compiler/optimizer.rs` - Optimization passes

---

### Stage 5: RETE Integration (Week 9-10)
**Goal:** Use compiled rules in RETE engine

**Tasks:**
1. ✅ Modify `IncrementalEngine` to support compiled rules
2. ✅ Compile alpha node tests
3. ✅ Compile beta join conditions
4. ✅ Hybrid mode: Some rules compiled, some interpreted
5. ✅ Benchmark RETE with compiled rules

**Success Criteria:**
- RETE + Compiled rules: 100x+ speedup
- Seamless integration with existing RETE features

**Files to Modify:**
- `src/rete/incremental_engine.rs` - Support compiled rules
- `src/rete/alpha_node.rs` - Use compiled tests
- `src/rete/beta_node.rs` - Use compiled joins

---

### Stage 6: Build System Integration (Week 11-12)
**Goal:** Smooth developer experience

**Tasks:**
1. ✅ Cargo build script: Auto-compile `.grl` files
2. ✅ Cache compiled rules (avoid recompilation)
3. ✅ Hot reload in development mode
4. ✅ Production build optimization flags
5. ✅ Error messages with GRL source locations
6. ✅ Documentation and examples

**Success Criteria:**
- Zero-config compilation for users
- Clear error messages
- Fast incremental builds

**Files to Create:**
- `build.rs` - Build script
- `examples/compiled_rules_demo.rs` - Full demo
- `docs/advanced-features/RULE_COMPILATION.md` - Documentation

---

## 🧪 Testing Strategy

### Unit Tests
- Test each codegen component independently
- Verify generated Rust code compiles
- Compare output: interpreted vs compiled

### Integration Tests
- End-to-end: GRL → Compile → Execute
- All GRL features (conditions, actions, multifield, etc.)
- Error handling (invalid rules, compilation errors)

### Performance Benchmarks
```rust
// Benchmark suite
#[bench]
fn bench_interpreted_simple_rule(b: &mut Bencher) { ... }

#[bench]
fn bench_compiled_simple_rule(b: &mut Bencher) { ... }

// Expected results:
// interpreted: 1,000 ns/iter
// compiled:       50 ns/iter (20x faster)
```

### Stress Tests
- 10,000 compiled rules
- Hot reload under load
- Memory leaks check

---

## 🎯 Performance Targets

| Scenario | Interpreted | Compiled | Target Speedup |
|----------|-------------|----------|----------------|
| Simple condition (`A == B`) | 1,000 ns | 50 ns | **20x** |
| Complex condition (5+ ops) | 5,000 ns | 100 ns | **50x** |
| Arithmetic expression | 2,000 ns | 40 ns | **50x** |
| Function call | 3,000 ns | 200 ns | **15x** |
| Full rule (condition + action) | 10,000 ns | 300 ns | **33x** |
| RETE with 1,000 rules | 10 ms | 200 µs | **50x** |

---

## 🚧 Challenges & Solutions

### Challenge 1: Dynamic Loading
**Problem:** Rust doesn't support runtime code loading easily
**Solution:** Use `libloading` crate for dynamic library loading

### Challenge 2: Type Safety
**Problem:** Compiled code bypasses Rust's type checker
**Solution:** Generate type-safe code, validate at compile time

### Challenge 3: Incremental Compilation
**Problem:** Recompiling all rules on every change is slow
**Solution:** Track dependencies, only recompile changed rules

### Challenge 4: Debugging
**Problem:** Compiled code harder to debug than interpreted
**Solution:** Preserve source maps, add debug symbols, support both modes

### Challenge 5: Cross-Platform
**Problem:** Compiled `.so`/`.dll` not portable
**Solution:** Distribute source + compile on target, or use WASM

---

## 📊 Success Metrics

### Performance
- ✅ 10x minimum speedup for simple rules
- ✅ 50x maximum speedup for optimizable rules
- ✅ No regression for interpreted mode

### Compatibility
- ✅ 100% GRL syntax compatibility
- ✅ All existing examples work
- ✅ Zero API breaking changes

### Usability
- ✅ One-line opt-in: `engine.enable_compilation()`
- ✅ Clear error messages
- ✅ Fast incremental builds (<1s)

---

## 🔄 Future Enhancements (Phase 2)

### 1. Profile-Guided Optimization (PGO)
- Collect runtime statistics
- Recompile hot rules with aggressive optimization
- 2-5x additional speedup

### 2. LLVM Backend
- Generate LLVM IR instead of Rust source
- More optimization opportunities
- Potential 2x additional speedup

### 3. GPU Compilation
- Compile rules to GPU kernels (CUDA/OpenCL)
- Parallel rule evaluation
- 100-1000x speedup for data-parallel rules

### 4. WebAssembly Target
- Compile to WASM for browser/edge deployment
- Portable compiled rules
- Near-native performance

---

## 📚 References

### Similar Projects
- **Drools** (Java) - Has JIT compilation via JVM
- **CLIPS** (C) - Compiled rules support
- **Jess** (Java) - Rete + JIT

### Technical Resources
- [libloading](https://docs.rs/libloading/) - Dynamic library loading
- [syn](https://docs.rs/syn/) - Rust code generation
- [quote](https://docs.rs/quote/) - Rust macro helpers
- [LLVM Rust Bindings](https://docs.rs/inkwell/) - For Phase 2

---

## 🗓️ Timeline

**Total Duration:** 12 weeks (3 months)

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1-2 | POC | Simple rule compilation works |
| 3-4 | Conditions | All condition types supported |
| 5-6 | Actions | All action types supported |
| 7-8 | Optimization | Advanced optimizations implemented |
| 9-10 | RETE | Integration with RETE engine |
| 11-12 | Polish | Build system, docs, examples |

**Release:** v1.14.0 (Rule Compilation Support)

---

## ✅ Next Steps

1. **Review this plan** - Get feedback from team/users
2. **Create GitHub milestone** - Track progress
3. **Start POC** - Validate approach with simple example
4. **Write RFC** - Formal proposal for API design

---

**Last Updated:** 2025-12-25
**Author:** Ton That Vu
**Status:** Ready for Review

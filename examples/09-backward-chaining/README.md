# Backward Chaining Examples

> **Note:** These examples require the `backward-chaining` feature to be enabled:
> ```toml
> rust-rule-engine = { version = "0.19", features = ["backward-chaining"] }
> ```

Backward chaining (goal-driven reasoning) examples demonstrating how to query "Can goal X be proven?" by working backwards through rules.

## Examples

### 1. **ecommerce_approval_demo.rs** ⭐ **START HERE**
Real-world e-commerce order approval system
- Scenario 1: VIP customer với đơn 5 triệu
- Scenario 2: Khách mới mua đơn nhỏ 500k
- Scenario 3: Đơn lớn 50 triệu - tài khoản mới (cần review)
- Scenario 4: Batch processing 100 đơn hàng

**This is the BEST example to understand backward chaining in practice!**

**Run:**
```bash
cargo run --example ecommerce_approval_demo --features backward-chaining
```

### 2. **simple_query_demo.rs**
Basic backward chaining queries
- Simple goal queries
- Query results and proof traces
- Missing facts detection
- Memoization benefits

**Run:**
```bash
cargo run --example simple_query_demo --features backward-chaining
```

### 2. **medical_diagnosis_demo.rs**
Medical diagnostic reasoning system
- Diagnose flu from symptoms
- Metabolic syndrome detection
- Differential diagnosis (heart attack vs GERD)
- Pneumonia diagnosis with reasoning explanation

**Run:**
```bash
cargo run --example medical_diagnosis_demo --features backward-chaining
```

### 3. **detective_system_demo.rs**
Crime investigation system (Sherlock Holmes style)
- Murder mystery solving
- Alibi verification
- Motive analysis
- Deductive reasoning chains

**Run:**
```bash
cargo run --example detective_system_demo --features backward-chaining
```

### 4. **grl_query_demo.rs**
GRL Query Syntax demonstration
- Declarative query definitions
- Query with on-success/failure actions
- Multiple query execution
- Medical diagnosis using GRL queries

**Run:**
```bash
cargo run --example grl_query_demo --features backward-chaining
```

### 5. **proof_graph_cache_demo.rs** 🆕
ProofGraph caching and TMS integration demonstration
- Cache hits on repeated queries (100% hit rate demo)
- Dependency tracking and cascading invalidation
- Multiple justifications for same fact (alternative proofs)
- Cache statistics and performance metrics
- Expected 100-1000x speedup for cached queries

**Run:**
```bash
cargo run --example proof_graph_cache_demo --features backward-chaining
```

## Use Cases

**Backward chaining is ideal for:**

1. **Diagnostic Systems**
   - Medical diagnosis: "Does the patient have disease X?"
   - Technical troubleshooting: "Why is the system failing?"
   - Root cause analysis: "What caused this error?"

2. **Question Answering**
   - "Can user X perform action Y?"
   - "Is customer eligible for VIP status?"
   - "What's preventing goal Z from being achieved?"

3. **Compliance Checking**
   - "Does this satisfy regulation X?"
   - "What's missing for loan approval?"
   - "Is the application complete?"

4. **Investigation & Reasoning**
   - Detective work: "Who could have committed the crime?"
   - Security analysis: "How was the system breached?"
   - Debugging: "Why did this rule not fire?"

**When to use Forward vs Backward:**

| Feature | Forward Chaining | Backward Chaining |
|---------|-----------------|-------------------|
| **Approach** | Data-driven | Goal-driven |
| **Best for** | Event processing, monitoring | Queries, diagnostics |
| **Execution** | All matching rules fire | Only rules needed for goal |
| **Performance** | Fast for many rules | Efficient for specific queries |
| **Use case** | Real-time systems | On-demand analysis |

**Hybrid mode (Future):**
- Forward for continuous monitoring
- Backward for on-demand queries
- Best of both worlds

## Testing & Verification

This directory includes comprehensive test suites that verify all backward chaining features:

### **comprehensive_backward_test.rs** - All Features Test
12 comprehensive tests covering all major features:
- Basic goal proving, search strategies (DFS, BFS, Iterative)
- Complex conditions (AND, OR, NOT), multi-level chaining
- Built-in functions, GRL query syntax, action handlers
- Memoization, TMS integration, missing facts detection, proof traces

**Run:**
```bash
cargo run --example comprehensive_backward_test --features backward-chaining
```

### **backward_edge_cases_test.rs** - Correctness Tests
8 critical edge case tests for correctness verification:
- Rollback on failure, NOT condition evaluation
- Backtracking, false positive prevention
- Speculative changes rollback, EXISTS/FORALL conditions
- Nested rollback

**Run:**
```bash
cargo run --example backward_edge_cases_test --features backward-chaining
```

### **backward_critical_missing_tests.rs** - Critical Tests
10 previously untested critical cases:
- OR edge cases, cycle detection, max depth limit
- Complex nested conditions, string operators
- Function edge cases, action types, diamond dependency
- Empty knowledge base, large rule chains

**Run:**
```bash
cargo run --example backward_critical_missing_tests --features backward-chaining
```

### Test Coverage Summary
See **[BACKWARD_CHAINING_TEST_SUMMARY.md](BACKWARD_CHAINING_TEST_SUMMARY.md)** for complete test coverage analysis:
- 109 total tests (73 unit + 5 doc + 1 integration + 30 example)
- 95% feature coverage
- All critical bugs documented and fixed
- Production readiness assessment

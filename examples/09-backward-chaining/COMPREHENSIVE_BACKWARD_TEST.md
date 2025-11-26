# Comprehensive Backward Chaining Feature Test

## Overview

This example (`comprehensive_backward_test.rs`) demonstrates **ALL features** of the backward chaining engine in a single, runnable test suite. It serves as both a functional test and a feature showcase.

## Running the Test

```bash
cargo run --example comprehensive_backward_test --features backward-chaining
```

Expected output: All 12 tests should pass with ✅ ALL TESTS PASSED!

---

## Features Tested

### ✅ Test 1: Basic Goal Proving
**Features:** Simple backward chaining query

- **What it tests:** Basic rule execution and goal proving
- **Rules:** If User.Points > 1000, then User.IsVIP = true
- **Query:** User.IsVIP == true
- **Demonstrates:**
  - Simple condition evaluation
  - Action execution (Set)
  - Fact derivation

---

### ✅ Test 2: Search Strategies (DFS, BFS, Iterative Deepening)
**Features:** All three search strategies

- **What it tests:** Different search algorithms produce correct results
- **Strategies:**
  1. **Depth-First Search** - Goes deep into one branch before backtracking
  2. **Breadth-First Search** - Explores all goals at one level before going deeper
  3. **Iterative Deepening** - Combines benefits of DFS and BFS
- **Rules:** 3-level rule chain (A → B → C → D)
- **Demonstrates:**
  - BackwardConfig customization
  - Multiple search strategies
  - Performance differences between strategies

**Results:**
- DFS: 3 goals explored
- BFS: 1 goal explored
- IDS: 1 goal explored

---

### ✅ Test 3: Complex Conditions (AND, OR, NOT)
**Features:** Logical operators in rule conditions

- **What it tests:** Compound condition evaluation
- **Conditions tested:**
  - **AND:** (Age > 18) AND (HasLicense == true)
  - **OR:** (Card == true) OR (Cash == true)
- **Demonstrates:**
  - ConditionGroup::Compound
  - LogicalOperator (And, Or, Not)
  - Complex condition satisfaction

---

### ✅ Test 4: Multi-Level Rule Chaining
**Features:** Recursive backward reasoning

- **What it tests:** Rules that depend on conclusions of other rules
- **Chain:** Points(150) → Bronze → Discount(10) → SpecialOffer
- **Levels:** 3 levels of rule dependencies
- **Demonstrates:**
  - Recursive goal proving
  - Rule chaining depth
  - Intermediate fact derivation

---

### ✅ Test 5: Built-in Function Calls
**Features:** Function evaluation in conditions

- **Functions tested:**
  - `len(field)` - String/array length
  - `isEmpty(field)` - Empty check
- **Examples:**
  - `len(User.Name) > 5` → "Alexander".len() = 9 > 5 ✓
  - `isEmpty(Product.Description) == false` → "Great product" is not empty ✓
- **Demonstrates:**
  - Condition::with_function()
  - Built-in function support
  - Function result comparison

---

### ✅ Test 6: GRL Query Syntax
**Features:** Goal-driven Rule Language queries

- **What it tests:** Parsing and execution of GRL queries
- **GRL Features:**
  ```grl
  query "CheckVIP" {
      goal: User.IsVIP == true
      strategy: depth-first
      max-depth: 5
      on-success: {
          User.DiscountRate = 0.15;
          LogMessage("VIP status confirmed");
      }
  }
  ```
- **Demonstrates:**
  - GRLQueryParser
  - GRLQueryExecutor
  - Action execution on success

---

### ✅ Test 7: Action Handlers (on-success, on-failure, on-missing)
**Features:** Conditional action execution

- **Handlers tested:**
  - `on-success` - Execute when goal is proven
  - `on-failure` - Execute when goal fails
  - `on-missing` - Execute when facts are missing
- **Demonstrates:**
  - QueryAction execution
  - Fact assignments in actions
  - Handler selection based on result

---

### ✅ Test 8: Conditional Execution (when clauses)
**Features:** Query guards

- **What it tests:** `when` clause prevents query execution
- **Example:**
  ```grl
  when: Environment.Mode == "Production"
  ```
- **Test cases:**
  - Development mode: Query does NOT execute ✓
  - Production mode: Query executes ✓
- **Demonstrates:**
  - Conditional query execution
  - Expression evaluation in when clauses
  - Guard conditions

---

### ✅ Test 9: Memoization (Caching)
**Features:** Query result caching

- **What it tests:** Second identical query returns cached result
- **Configuration:**
  ```rust
  BackwardConfig {
      enable_memoization: true,
      ...
  }
  ```
- **Results:**
  - First query: 1 goal explored
  - Second query: 0 goals explored (cached!) ✓
- **Demonstrates:**
  - GoalManager caching
  - Performance optimization
  - Result reuse

---

### ✅ Test 10: TMS Integration (Logical Facts)
**Features:** Truth Maintenance System integration

- **What it tests:** Backward chaining with RETE TMS
- **Components:**
  - IncrementalEngine (RETE)
  - Logical fact insertion
  - Justification tracking
- **Demonstrates:**
  - query_with_rete_engine()
  - TMS inserter callback
  - Logical fact derivation with justifications

**How it works:**
```rust
let rete_engine = Arc::new(Mutex::new(IncrementalEngine::new()));
engine.query_with_rete_engine(goal, facts, Some(rete_engine))
```

---

### ✅ Test 11: Missing Facts Detection
**Features:** Unprovable goal analysis

- **What it tests:** System reports which facts are missing
- **Rule:** Requires both Input.A AND Input.B
- **Scenario:** Only Input.A provided, Input.B missing
- **Result:** Reports "Input.B == true" as missing ✓
- **Demonstrates:**
  - Missing fact detection
  - Unprovable goal analysis
  - QueryResult.missing_facts

---

### ✅ Test 12: Proof Traces
**Features:** Reasoning explanation

- **What it tests:** System generates proof of goal
- **Chain:** Start → Middle → End
- **Demonstrates:**
  - ProofTrace generation
  - Reasoning step tracking
  - Proof explanation

**Output:**
```
Goal: End == true
Steps: 2 reasoning steps
  1. ProofStep { ... }
  2. ProofStep { ... }
```

---

## Code Statistics

- **Total Tests:** 12
- **Lines of Code:** ~670
- **Rules Created:** ~15
- **Queries Executed:** ~20
- **Coverage:** ~95% of backward chaining features

---

## Feature Matrix

| Feature | Tested | Lines | Notes |
|---------|--------|-------|-------|
| Basic queries | ✅ | 49-77 | Simple goal proving |
| DFS search | ✅ | 88-175 | Depth-first strategy |
| BFS search | ✅ | 88-175 | Breadth-first strategy |
| Iterative deepening | ✅ | 88-175 | IDS strategy |
| AND conditions | ✅ | 177-250 | Compound conditions |
| OR conditions | ✅ | 177-250 | Logical OR |
| NOT conditions | ✅ | 177-250 | Negation |
| Rule chaining | ✅ | 252-312 | Multi-level |
| len() function | ✅ | 314-367 | String length |
| isEmpty() function | ✅ | 314-367 | Empty check |
| exists() function | ❌ | - | Not in test |
| count() function | ❌ | - | Not in test |
| GRL parsing | ✅ | 369-420 | Query syntax |
| Action handlers | ✅ | 422-468 | on-* actions |
| when clauses | ✅ | 470-497 | Conditional exec |
| Memoization | ✅ | 499-537 | Caching |
| TMS integration | ✅ | 539-578 | RETE + TMS |
| Missing facts | ✅ | 580-625 | Detection |
| Proof traces | ✅ | 627-671 | Explanation |

---

## Success Criteria

All tests must:
1. ✅ Compile without errors
2. ✅ Execute without panics
3. ✅ Assert all expected behaviors
4. ✅ Print clear success messages

**Current Status:** ✅ ALL TESTS PASSED

---

## Usage as Integration Test

This example can also serve as an integration test:

```bash
# Run and check exit code
cargo run --example comprehensive_backward_test --features backward-chaining
if [ $? -eq 0 ]; then
    echo "✅ All backward chaining features working"
else
    echo "❌ Some features failed"
    exit 1
fi
```

---

## Extending the Tests

To add a new feature test:

1. Create a new function `test_N_feature_name()`
2. Add comprehensive assertions
3. Print clear success/failure messages
4. Call from `main()`
5. Update this README

---

## See Also

- [Backward Chaining Architecture](../BACKWARD_CHAINING_ARCHITECTURE.md)
- [Examples README](09-backward-chaining/README.md)
- [Main README](../README.md)

# Backward Chaining Design Document

## Overview

Backward chaining (goal-driven reasoning) implementation for rust-rule-engine. Tách riêng thành optional feature để:
- Không làm tăng binary size khi không dùng
- Dễ maintain và test độc lập
- Cho phép users chọn forward-only hoặc hybrid mode

---

## Architecture Design

### 1. Feature Flag Structure

```toml
[features]
default = []
streaming = ["tokio", "futures"]
backward-chaining = ["petgraph"]  # NEW FEATURE

# Hybrid mode (both forward + backward)
full-reasoning = ["backward-chaining"]
```

**Rationale:**
- `petgraph` cho dependency graph analysis
- Feature tách biệt không ảnh hưởng existing code
- Users có thể opt-in khi cần

### 2. Module Structure

```
src/
  backward/           # NEW MODULE (cfg-gated)
    mod.rs           # Public API
    goal.rs          # Goal system
    search.rs        # Search strategies
    dependency.rs    # Rule dependency graph (backward)
    query.rs         # Query interface
    backward_engine.rs  # Backward engine (renamed to avoid confusion)
    hybrid.rs        # Forward + Backward integration
```

### 3. Core Components

#### 3.1 Goal System

```rust
/// Represents a goal to prove/achieve
#[cfg(feature = "backward-chaining")]
pub struct Goal {
    /// Target fact pattern to prove
    pub pattern: Pattern,
    /// Current status
    pub status: GoalStatus,
    /// Sub-goals required
    pub sub_goals: Vec<Goal>,
    /// Rules that can derive this goal
    pub candidate_rules: Vec<String>,
}

#[cfg(feature = "backward-chaining")]
pub enum GoalStatus {
    Pending,
    InProgress,
    Proven,
    Unprovable,
}
```

#### 3.2 Query Interface

```rust
#[cfg(feature = "backward-chaining")]
pub trait BackwardReasoning {
    /// Query if a goal can be proven
    fn query(&self, goal: &str, facts: &Facts) -> Result<QueryResult>;
    
    /// Prove a goal and return derivation trace
    fn prove(&self, goal: Goal, facts: &Facts) -> Result<ProofTrace>;
    
    /// Find all solutions for a goal
    fn find_all(&self, goal: &str, facts: &Facts) -> Result<Vec<QueryResult>>;
}
```

#### 3.3 Search Strategies

```rust
#[cfg(feature = "backward-chaining")]
pub enum SearchStrategy {
    DepthFirst,     // Prolog-style
    BreadthFirst,   // Level-by-level
    Iterative,      // Iterative deepening
}
```

---

## Implementation Plan

### Phase 1: Foundation (v0.20.0)

**Goals:**
- Setup feature flag infrastructure
- Implement basic goal system
- Create simple query interface

**Deliverables:**
```rust
// Basic usage
#[cfg(feature = "backward-chaining")]
use rust_rule_engine::backward::*;

let engine = BackwardEngine::new(kb);
let result = engine.query("User.IsVIP == true", &facts)?;
```

### Phase 2: Integration (v0.21.0)

**Goals:**
- Integrate with RETE pattern matching
- Implement dependency graph analysis
- Add search strategies

**Deliverables:**
```rust
// Hybrid mode
let hybrid = HybridEngine::new(forward_engine, backward_engine);
hybrid.configure(HybridConfig {
    forward_first: true,
    backward_fallback: true,
});
```

### Phase 3: Advanced Features (v0.22.0)

**Goals:**
- GRL syntax extensions for queries
- Proof explanation system
- Performance optimizations

**Deliverables:**
```grl
// Query syntax in GRL
query "CheckVIPStatus" {
    goal: User.IsVIP == true
    strategy: depth-first
    max-depth: 5
}
```

---

## Technical Considerations

### 1. Performance

**Optimizations:**
- Memoization của proven goals (avoid re-proof)
- RETE integration để tận dụng pattern matching
- Parallel goal proving (independent branches)

**Benchmarking:**
- Compare vs pure forward chaining
- Measure overhead của backward search
- Identify best use cases

### 2. Memory Management

**Strategies:**
- Limit search depth (avoid infinite loops)
- Prune unprovable branches early
- Cache intermediate results

### 3. Compatibility

**Backward compatibility:**
```rust
// Existing code works unchanged
let engine = RustRuleEngine::new(kb);
engine.execute(&facts)?;  // Forward-only

// Opt-in to backward chaining
#[cfg(feature = "backward-chaining")]
{
    use rust_rule_engine::backward::BackwardEngine;
    let bc_engine = BackwardEngine::from_forward(engine);
}
```

---

## Use Cases

### 1. Diagnostic Systems

```rust
// "Why is User.IsVIP false?"
let trace = engine.prove_why("User.IsVIP == false", &facts)?;
trace.print_explanation();
// Output: User.SpendingTotal < 1000 → Score < 80 → IsVIP = false
```

### 2. Planning Systems

```rust
// "What needs to be true to apply discount?"
let requirements = engine.find_preconditions("Order.DiscountApplied", &facts)?;
// Returns: ["User.IsVIP == true", "Order.Amount > 500"]
```

### 3. Question Answering

```rust
// "Can this user get VIP status?"
let answer = engine.query("User.IsVIP == true", &facts)?;
if answer.is_provable {
    println!("Yes! By: {:?}", answer.proof_path);
} else {
    println!("No. Missing: {:?}", answer.missing_facts);
}
```

---

## API Design

### Basic API

```rust
#[cfg(feature = "backward-chaining")]
pub struct BackwardEngine {
    knowledge_base: Arc<KnowledgeBase>,
    config: BackwardConfig,
}

#[cfg(feature = "backward-chaining")]
impl BackwardEngine {
    pub fn new(kb: KnowledgeBase) -> Self;
    
    pub fn query(&self, goal: &str, facts: &Facts) -> Result<QueryResult>;
    
    pub fn prove(&self, goal: Goal, facts: &Facts) -> Result<ProofTrace>;
    
    pub fn explain_why(&self, fact: &str, facts: &Facts) -> Result<Explanation>;
    
    pub fn find_preconditions(&self, goal: &str) -> Result<Vec<String>>;
}

#[cfg(feature = "backward-chaining")]
pub struct QueryResult {
    pub provable: bool,
    pub bindings: HashMap<String, Value>,
    pub proof_trace: ProofTrace,
    pub missing_facts: Vec<String>,
}
```

### Advanced API

```rust
#[cfg(feature = "backward-chaining")]
pub struct HybridEngine {
    forward: RustRuleEngine,
    backward: BackwardEngine,
    config: HybridConfig,
}

#[cfg(feature = "backward-chaining")]
impl HybridEngine {
    /// Execute forward chaining, then backward for queries
    pub fn execute_hybrid(&mut self, facts: &Facts) -> Result<HybridResult>;
    
    /// Query with fallback to forward chaining
    pub fn smart_query(&self, query: &str, facts: &Facts) -> Result<QueryResult>;
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(all(test, feature = "backward-chaining"))]
mod tests {
    #[test]
    fn test_simple_goal_proving() { }
    
    #[test]
    fn test_recursive_goals() { }
    
    #[test]
    fn test_circular_dependency_detection() { }
    
    #[test]
    fn test_search_strategies() { }
}
```

### Integration Tests

```rust
// tests/backward_chaining_test.rs
#![cfg(feature = "backward-chaining")]

#[test]
fn test_hybrid_execution() {
    // Forward chaining sets up base facts
    // Backward chaining proves queries
}
```

### Benchmarks

```rust
// benches/backward_chaining_bench.rs
#![cfg(feature = "backward-chaining")]

fn bench_query_performance(c: &mut Criterion) {
    // Compare forward vs backward vs hybrid
}
```

---

## Documentation Plan

### 1. User Guide

- `docs/BACKWARD_CHAINING.md` - Comprehensive guide
- `docs/HYBRID_MODE.md` - Forward + Backward together
- `docs/QUERY_SYNTAX.md` - GRL query extensions

### 2. Examples

```
examples/09-backward-chaining/
  README.md
  simple_query_demo.rs
  diagnostic_system_demo.rs
  planning_demo.rs
  hybrid_mode_demo.rs
  proof_explanation_demo.rs
```

### 3. API Reference

Auto-generated từ rustdoc với extensive examples.

---

## Migration Path

### For Users

**Opt-in process:**
```toml
# Step 1: Enable feature
[dependencies]
rust-rule-engine = { version = "0.20", features = ["backward-chaining"] }
```

```rust
// Step 2: Use new APIs
#[cfg(feature = "backward-chaining")]
use rust_rule_engine::backward::BackwardEngine;

// Step 3: Gradual adoption
let forward = RustRuleEngine::new(kb);
#[cfg(feature = "backward-chaining")]
let backward = BackwardEngine::from_forward(&forward);
```

### Breaking Changes

**None!** Feature là opt-in, existing code không bị ảnh hưởng.

---

## Timeline Estimate

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| Phase 1 | 2-3 weeks | Basic goal system, simple queries |
| Phase 2 | 3-4 weeks | RETE integration, search strategies |
| Phase 3 | 2-3 weeks | GRL extensions, optimizations |
| Testing | 1-2 weeks | Comprehensive tests, benchmarks |
| Docs | 1 week | User guide, examples, API docs |
| **Total** | **9-13 weeks** | Feature-complete backward chaining |

---

## Success Metrics

1. **Performance:**
   - Query resolution < 10ms for simple goals
   - Hybrid mode overhead < 5%

2. **Coverage:**
   - 90%+ test coverage
   - Examples for all major use cases

3. **Usability:**
   - Clear documentation
   - Easy opt-in/opt-out
   - No breaking changes

4. **Adoption:**
   - Feedback from early users
   - Use cases validated

---

## Open Questions

1. **Negation Handling:** How to handle negation-as-failure?
2. **Variable Binding:** Should we support logic variables?
3. **Cut Operator:** Do we need Prolog-style cut?
4. **Module System:** How to organize large backward rules?

---

## Next Steps

1. ✅ Review design with stakeholders
2. ⬜ Prototype goal system
3. ⬜ Implement basic query engine
4. ⬜ Create first examples
5. ⬜ Gather feedback and iterate

---

## References

- [Prolog Backward Chaining](https://en.wikipedia.org/wiki/Backward_chaining)
- [CLIPS Manual - Backward Chaining](https://clipsrules.net/)
- [Drools Backward Chaining](https://docs.drools.org/)

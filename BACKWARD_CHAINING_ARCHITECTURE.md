# Backward Chaining Architecture

**Last Updated:** 2025-11-25 (After Unification Implementation)
**Status:** Alpha - 48% Complete

---

## 📐 System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     BACKWARD CHAINING SYSTEM                            │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
            ┌───────────────────────┼───────────────────────┐
            │                       │                       │
            ▼                       ▼                       ▼
    ┌───────────────┐      ┌───────────────┐     ┌───────────────┐
    │ Query Layer   │      │ Execution     │     │ Knowledge     │
    │               │      │ Layer         │     │ Layer         │
    └───────────────┘      └───────────────┘     └───────────────┘
```

---

## 🏗️ Module Structure

```
src/backward/
├── mod.rs                  # Module exports
├── backward_engine.rs      # Main engine (591 lines)
├── expression.rs           # AST parser (734 lines) ✅
├── unification.rs          # Variable bindings (600+ lines) ✨ NEW
├── goal.rs                 # Goal management (242 lines)
├── search.rs               # Search strategies (538 lines)
├── query.rs                # Query interface (288 lines)
├── grl_query.rs            # GRL integration (701 lines)
└── rule_executor.rs        # Rule execution (243 lines)

Total: ~4,000 lines of code
```

---

## 🔄 Data Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            QUERY FLOW                                   │
└─────────────────────────────────────────────────────────────────────────┘

User Query String
      │
      ▼
┌──────────────────┐
│ ExpressionParser │  Parse query string to AST
│                  │  "User.IsVIP == ?X"
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Goal             │  Create goal with expression
│ - pattern        │  - Variable bindings (Bindings)
│ - expression     │  - Status tracking
│ - bindings       │  - Sub-goals
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ BackwardEngine   │  Main reasoning engine
│ - query()        │  - Find candidate rules
│ - prove_goal()   │  - Execute search strategy
└────────┬─────────┘
         │
         ├─────────────────┐
         │                 │
         ▼                 ▼
┌──────────────┐   ┌──────────────┐
│ SearchEngine │   │ Unifier      │
│ - DFS/BFS    │   │ - unify()    │
│ - Iterative  │   │ - match()    │
└──────┬───────┘   │ - evaluate() │
       │           └──────┬───────┘
       │                  │
       ▼                  ▼
┌──────────────────────────────┐
│ RuleExecutor                 │
│ - evaluate_condition()       │
│ - execute_actions()          │
│ - derive_facts()             │
└──────────────┬───────────────┘
               │
               ▼
┌──────────────────────────────┐
│ Facts (Updated)              │
│ - New derived facts          │
│ - Variable bindings          │
└──────────────────────────────┘
               │
               ▼
┌──────────────────────────────┐
│ QueryResult                  │
│ - provable: bool             │
│ - bindings: HashMap          │
│ - proof_trace                │
│ - stats                      │
└──────────────────────────────┘
```

---

## 🧩 Core Components

### 1. Expression Parser ✅ 95% Complete

```rust
// AST-based expression parsing
pub enum Expression {
    Field(String),              // "User.IsVIP"
    Literal(Value),            // true, 42, "hello"
    Variable(String),          // "?X", "?Customer" ✨
    Comparison { ... },        // "X == Y"
    And { ... },              // "A && B"
    Or { ... },               // "A || B"
    Not(Box<Expression>),     // "!X"
}

impl ExpressionParser {
    pub fn parse(input: &str) -> Result<Expression>
}
```

**Features:**
- ✅ Recursive descent parsing
- ✅ All operators (==, !=, >, <, >=, <=, &&, ||, !)
- ✅ Parentheses support
- ✅ Variable parsing (?X syntax)
- ⚠️ Needs more comprehensive tests

---

### 2. Unification System ✨ NEW - 90% Complete

```rust
// Variable bindings with conflict detection
pub struct Bindings {
    bindings: HashMap<String, Value>,
}

impl Bindings {
    pub fn bind(&mut self, var: String, value: Value) -> Result<()>
    pub fn get(&self, var: &str) -> Option<&Value>
    pub fn merge(&mut self, other: &Bindings) -> Result<()>
    pub fn to_map(&self) -> HashMap<String, Value>
    // ... 9 more methods
}

// Pattern matching & unification
pub struct Unifier;

impl Unifier {
    // Unify two expressions
    pub fn unify(
        left: &Expression,
        right: &Expression,
        bindings: &mut Bindings,
    ) -> Result<bool>

    // Match expression against facts
    pub fn match_expression(
        expr: &Expression,
        facts: &Facts,
        bindings: &mut Bindings,
    ) -> Result<bool>

    // Evaluate with variable substitution
    pub fn evaluate_with_bindings(
        expr: &Expression,
        facts: &Facts,
        bindings: &Bindings,
    ) -> Result<Value>
}
```

**Features:**
- ✅ Variable binding with conflict detection
- ✅ Full unification algorithm
- ✅ Pattern matching
- ✅ Binding propagation
- ✅ 10 comprehensive unit tests
- ✅ Integration example working

**Use Cases:**
```rust
// Variable binding
let mut bindings = Bindings::new();
bindings.bind("Customer", Value::String("Alice"))?;

// Pattern matching
Unifier::match_expression(&expr, &facts, &mut bindings)?;

// Unification
Unifier::unify(&var_expr, &literal_expr, &mut bindings)?;

// Evaluation with substitution
let result = Unifier::evaluate_with_bindings(&expr, &facts, &bindings)?;
```

---

### 3. Goal Management

```rust
pub struct Goal {
    pub pattern: String,
    pub expression: Option<Expression>,
    pub status: GoalStatus,
    pub sub_goals: Vec<Goal>,
    pub candidate_rules: Vec<String>,
    pub bindings: Bindings,  // ✨ Now uses Bindings type
    pub depth: usize,
}

pub enum GoalStatus {
    Pending,
    InProgress,
    Proven,
    Unprovable,
}
```

**Bindings Integration:** ✨
- Goals now maintain variable bindings during proof search
- Bindings propagate through sub-goals
- Conflict detection prevents invalid proofs

---

### 4. Search Strategies

```rust
pub enum SearchStrategy {
    DepthFirst,   // ✅ Implemented
    BreadthFirst, // ✅ Implemented
    Iterative,    // ⚠️ Planned
}

pub struct SearchResult {
    pub success: bool,
    pub path: Vec<String>,
    pub goals_explored: usize,
    pub max_depth_reached: usize,
    pub bindings: HashMap<String, Value>,
}
```

---

### 5. Backward Engine

```rust
pub struct BackwardEngine {
    knowledge_base: KnowledgeBase,
    goal_manager: GoalManager,
    config: BackwardConfig,
}

impl BackwardEngine {
    pub fn query(
        &mut self,
        query_str: &str,
        facts: &mut Facts,
    ) -> Result<QueryResult>
}
```

**Flow:**
1. Parse query string → Expression AST
2. Create Goal with expression
3. Find candidate rules (⚠️ Currently O(n), needs RETE)
4. Execute search strategy
5. Unify variables ✨
6. Execute matching rules
7. Return QueryResult with bindings ✨

---

## 🔗 Integration Points

### With Forward Chaining (Hybrid Mode)

```
Forward Chaining              Backward Chaining
(Data-driven)                 (Goal-driven)
      │                             │
      │    1. Derive facts          │
      ├────────────────────────────►│
      │                             │
      │    2. Query goal            │
      │◄────────────────────────────┤
      │                             │
      │    3. Return bindings       │
      │◄────────────────────────────┤
      │                             │
```

### With RETE Network (Planned)

```
RETE Network
      │
      │ Conclusion Index
      │ (field → rules)
      │
      ▼
Backward Engine
      │ Fast candidate
      │ finding (O(1))
      ▼
Search Strategy
```

---

## 📊 Performance Characteristics

| Operation | Current | Target | Status |
|-----------|---------|--------|--------|
| Query Parsing | O(n) | O(n) | ✅ Optimal |
| Candidate Finding | **O(n)** | **O(1)** | ⚠️ **Needs RETE** |
| Unification | O(1) | O(1) | ✅ Optimal |
| Pattern Matching | O(m) | O(m) | ✅ Optimal |
| Proof Search (DFS) | O(b^d) | O(b^d) | ✅ Expected |
| Proof Search (BFS) | O(b^d) | O(b^d) | ✅ Expected |

Where:
- n = number of rules
- m = expression complexity
- b = branching factor
- d = proof depth

**Critical Bottleneck:** O(n) candidate finding needs RETE integration!

---

## 🎯 Example Usage

### Basic Query with Variables

```rust
use rust_rule_engine::backward::{BackwardEngine, Bindings, Unifier};

// Setup
let mut engine = BackwardEngine::new(kb);
let mut facts = Facts::new();
facts.set("User.Points", Value::Number(1500.0));

// Query with variable
let result = engine.query("User.Status == ?Status", &mut facts)?;

if result.provable {
    // Access variable bindings
    if let Some(status) = result.bindings.get("Status") {
        println!("User status: {:?}", status);
    }
}
```

### Pattern Matching

```rust
let mut bindings = Bindings::new();
let expr = ExpressionParser::parse("User.Age > ?MinAge")?;

// Bind variable
bindings.bind("MinAge", Value::Number(18.0))?;

// Match against facts
if Unifier::match_expression(&expr, &facts, &mut bindings)? {
    println!("User is adult!");
}
```

### Unification

```rust
let mut bindings = Bindings::new();

let var = Expression::Variable("X".to_string());
let lit = Expression::Literal(Value::Number(42.0));

// Unify variable with value
if Unifier::unify(&var, &lit, &mut bindings)? {
    println!("X = {:?}", bindings.get("X")); // X = 42
}
```

---

## 🚀 Future Enhancements

### Phase 1 Remaining (Critical)
1. **RETE Integration** - O(1) candidate finding
2. **Rule Execution Testing** - Verify chained reasoning
3. **Expression Parser Tests** - Edge cases

### Phase 2 (Quality)
1. **Comprehensive Test Suite** - 90%+ coverage
2. **Error Handling** - Custom error types
3. **Documentation** - API docs, examples

### Phase 3 (Optimization)
1. **Advanced Memoization** - Cache proven sub-goals
2. **Lazy Evaluation** - Only evaluate needed branches
3. **Parallel Search** - Multiple proof paths simultaneously

---

## 📝 Notes

**Completed in this session:** ✨
- Full unification system (600+ lines)
- 10 unit tests
- Integration example
- Bindings propagation
- Conflict detection

**Key Achievement:**
Task 1.4 went from 40% → 90% complete in one session!

**Next Priority:**
Focus on testing (Task 1.1, 1.2) or RETE integration (Task 1.3) for performance.

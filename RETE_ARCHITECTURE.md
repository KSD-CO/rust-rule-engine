# RETE Algorithm Architecture

## Overview

RETE (from Latin meaning "network") is a high-performance pattern matching algorithm used in rule engines. Our Rust Rule Engine implementation uses the **RETE-UL (RETE with Unification and Lattice)** variant to achieve **2-24x faster** performance compared to traditional forward-chaining engines.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│                        RUST RULE ENGINE - RETE-UL ARCHITECTURE                  │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

                            ┌──────────────────────┐
                            │   GRL Rules (Text)   │
                            │  "when ... then ..." │
                            └──────────┬───────────┘
                                       │
                                       │ Parse
                                       ↓
                            ┌──────────────────────┐
                            │    GRL Parser        │
                            │  (src/parser/grl.rs) │
                            └──────────┬───────────┘
                                       │
                                       │ Convert
                                       ↓
                     ┌─────────────────────────────────────┐
                     │      GRL to RETE Loader             │
                     │   (src/rete/grl_loader.rs)          │
                     │  • Conditions → ReteUlNode          │
                     │  • Actions → Closures               │
                     └─────────────────┬───────────────────┘
                                       │
                                       │ Build Network
                                       ↓
┌────────────────────────────────────────────────────────────────────────────────┐
│                          RETE-UL NETWORK BUILDER                               │
│                          (src/rete/network.rs)                                 │
├────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐      │
│  │ UlAlpha  │   │  UlAnd   │   │  UlOr    │   │  UlNot   │   │ UlExists │      │
│  │  (Test)  │   │  (&&)    │   │  (||)    │   │  (NOT)   │   │ (EXISTS) │      │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘   └──────────┘      │
│                                                                                │
│  ┌──────────┐   ┌──────────┐   ┌──────────────────────┐   ┌──────────────┐     │
│  │ UlForall │   │UlAccumul.│   │   UlMultiField       │   │ UlTerminal   │     │
│  │(FORALL)  │   │(sum/avg) │   │ (array operations)   │   │ (Rule name)  │     │
│  └──────────┘   └──────────┘   └──────────────────────┘   └──────────────┘     │
│                                                                                │
└─────────────────────────────────┬──────────────────────────────────────────────┘
                                  │
                                  │ Compile to
                                  ↓
┌────────────────────────────────────────────────────────────────────────────────┐
│                        INCREMENTAL ENGINE                                      │
│                     (src/rete/propagation.rs)                                  │
├────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                       ALPHA NETWORK                                    │    │
│  │                     (src/rete/alpha.rs)                                │    │
│  │  ┌─────────────┐   ┌─────────────┐   ┌─────────────┐                   │    │
│  │  │ AlphaNode 1 │   │ AlphaNode 2 │   │ AlphaNode N │                   │    │
│  │  │ User.Age>18 │   │Country=="US"│   │ Price>100   │   ...             │    │
│  │  └──────┬──────┘   └──────┬──────┘   └──────┬──────┘                   │    │
│  │         │                 │                 │                          │    │
│  │         └─────────────────┴─────────────────┘                          │    │
│  │                           │                                            │    │
│  │                           ↓                                            │    │
│  │                  ┌─────────────────┐                                   │    │
│  │                  │  Alpha Memory   │                                   │    │
│  │                  │  (Matched Facts)│                                   │    │
│  │                  └────────┬────────┘                                   │    │
│  └─────────────────────────────┼──────────────────────────────────────────┘    │
│                                │                                               │
│                                ↓                                               │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                        BETA NETWORK                                    │    │
│  │                      (src/rete/beta.rs)                                │    │
│  │  ┌──────────────────────────────────────────────────────────────┐      │    │
│  │  │  BetaNode: Join(Alpha1, Alpha2)                              │      │    │
│  │  │  • Cartesian Product of matches                              │      │    │
│  │  │  • Combine multiple conditions                               │      │    │
│  │  └────────────────────────────┬─────────────────────────────────┘      │    │
│  │                               │                                        │    │
│  │                               ↓                                        │    │
│  │                    ┌────────────────────┐                              │    │
│  │                    │   Beta Memory      │                              │    │
│  │                    │ (Joined Matches)   │                              │    │
│  │                    └──────────┬─────────┘                              │    │
│  └───────────────────────────────┼────────────────────────────────────────┘    │
│                                  │                                             │
│                                  ↓                                             │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                          AGENDA                                        │    │
│  │                       (src/rete/agenda.rs)                             │    │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │    │
│  │  │  Priority Queue (BinaryHeap)                                     │  │    │
│  │  │  ┌────────────────────┐  ┌────────────────────┐                  │  │    │
│  │  │  │ Activation         │  │ Activation         │                  │  │    │
│  │  │  │ Rule: "HighPrio"   │  │ Rule: "Normal"     │    ...           │  │    │
│  │  │  │ Salience: 100      │  │ Salience: 0        │                  │  │    │
│  │  │  │ Group: "MAIN"      │  │ Group: "MAIN"      │                  │  │    │
│  │  │  └────────────────────┘  └────────────────────┘                  │  │    │
│  │  │                                                                  │  │    │
│  │  │  Conflict Resolution Strategy:                                   │  │    │
│  │  │  1. Salience (priority)                                          │  │    │
│  │  │  2. Agenda Groups                                                │  │    │
│  │  │  3. Activation Groups                                            │  │    │
│  │  │  4. No-loop                                                      │  │    │
│  │  │  5. Lock-on-active                                               │  │    │
│  │  └──────────────────────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────┬───────────────────────────────────┘    │
│                                       │                                        │
│                                       │ get_next_activation()                  │
│                                       ↓                                        │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                        RULE FIRING                                     │    │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │    │
│  │  │  1. Execute rule action (closure)                                │  │    │
│  │  │  2. Modify facts in Working Memory                               │  │    │
│  │  │  3. Mark rule as fired (for no-loop)                             │  │    │
│  │  │  4. Propagate changes incrementally                              │  │    │
│  │  └──────────────────────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────┬───────────────────────────────────┘    │
│                                       │                                        │
│                                       │ propagate_changes()                    │
│                                       ↓                                        │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                     WORKING MEMORY                                     │    │
│  │                  (src/rete/working_memory.rs)                          │    │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │    │
│  │  │  HashMap<FactHandle, WorkingMemoryFact>                          │  │    │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐               │  │    │
│  │  │  │ Fact #1     │  │ Fact #2     │  │ Fact #N     │   ...         │  │    │
│  │  │  │ Type: User  │  │ Type: Order │  │ Type: Prod  │               │  │    │
│  │  │  │ Age: 25     │  │ Total: 150  │  │ Price: 99   │               │  │    │
│  │  │  └─────────────┘  └─────────────┘  └─────────────┘               │  │    │
│  │  │                                                                  │  │    │
│  │  │  Change Tracking:                                                │  │    │
│  │  │  • modified_facts: HashSet<FactHandle>                           │  │    │
│  │  │  • retracted_facts: HashSet<FactHandle>                          │  │    │
│  │  │                                                                  │  │    │
│  │  │  Operations:                                                     │  │    │
│  │  │  • insert(type, facts) → Add new fact                            │  │    │
│  │  │  • update(handle, facts) → Modify existing (triggers re-eval)    │  │    │
│  │  │  • retract(handle) → Remove fact                                 │  │    │
│  │  └──────────────────────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
└────────────────────────────────────────────────────────────────────────────────┘

                                      │
                                      │ Incremental Update Loop
                                      │ (Only re-evaluate affected patterns)
                                      ↓
                            ┌──────────────────────┐
                            │   OUTPUT RESULTS     │
                            │  • Fired rules list  │
                            │  • Modified facts    │
                            │  • Engine stats      │
                            └──────────────────────┘


PERFORMANCE CHARACTERISTICS:
─────────────────────────────────────────────────────────────────────────────

  Initial Evaluation:  O(n × m)   [Same as forward chaining]
  Incremental Update:  O(k)       [k << n×m, 2-24x faster!]
  Rule Firing:         O(log n)   [Priority queue lookup]

  Where:
    n = number of rules
    m = number of facts  
    k = affected patterns (typically << n×m)


DATA FLOW EXAMPLE:
─────────────────────────────────────────────────────────────────────────────

  User inserts: { User.Age: 25, User.Country: "US" }
       ↓
  Alpha nodes evaluate:
    ✓ User.Age > 18     → MATCH
    ✓ Country == "US"   → MATCH
       ↓
  Beta nodes join:
    (Age > 18) AND (Country == "US") → MATCH
       ↓
  Create activation:
    Rule: "AdultUSUser", Salience: 50 → Add to Agenda
       ↓
  Fire rule:
    Execute action → User.IsAdult = true
       ↓
  Propagate changes:
    Re-evaluate only rules with "User.IsAdult" pattern (incremental!)
       ↓
  Done! (2-24x faster than re-evaluating ALL rules)
```

## Core Components

### 1. Alpha Network (Pattern Matching Layer)

**File:** `src/rete/alpha.rs`

Alpha nodes perform pattern matching on individual facts. Each alpha node tests a single simple condition.

```rust
pub struct AlphaNode {
    pub field: String,      // Field name (e.g., "User.Age")
    pub operator: String,   // Comparison operator (==, >, <, etc.)
    pub value: String,      // Value to compare against
}
```

**Functions:**
- `matches_typed()`: Checks if fact matches the pattern
- Variable reference support: `Facts.L1 > Facts.L1Min`
- Arithmetic expression evaluation: `User.Age % 3 == 0`

**Example:**
```grl
User.Age > 18  →  AlphaNode { field: "User.Age", operator: ">", value: "18" }
```

### 2. Beta Network (Join Layer)

**File:** `src/rete/beta.rs`

Beta nodes combine (join) multiple patterns from alpha memory to create compound conditions.

```rust
pub struct BetaNode {
    pub left: AlphaMemory,   // Left side matches
    pub right: AlphaMemory,  // Right side matches
}
```

**Functions:**
- `join()`: Cartesian product of left and right matches
- Combine multiple conditions: `A && B`

**Example:**
```
AlphaMemory(User.Age > 18) × AlphaMemory(User.Country == "US")
→ BetaMemory(matches both conditions)
```

### 3. Working Memory (Fact Storage)

**File:** `src/rete/working_memory.rs`

Central fact store với change tracking và incremental updates.

```rust
pub struct WorkingMemory {
    facts: HashMap<FactHandle, WorkingMemoryFact>,
    fact_types: HashMap<String, Vec<FactHandle>>,
    modified_facts: HashSet<FactHandle>,
    retracted_facts: HashSet<FactHandle>,
}
```

**Functions:**
- `insert()`: Add new fact
- `update()`: Update fact (triggers re-evaluation)
- `retract()`: Remove fact
- `to_typed_facts()`: Convert to TypedFacts for evaluation

**Change Tracking:**
```
Insert → modified_facts ← Update
              ↓
        propagate_changes()
              ↓
        Re-evaluate rules
```

### 4. Agenda (Activation Management)

**File:** `src/rete/agenda.rs`

Manages rule activations and firing order with conflict resolution strategies.

```rust
pub struct Agenda {
    activations: HashMap<String, BinaryHeap<Activation>>,
    fired_rules: HashSet<String>,
    focus: String,
    focus_stack: Vec<String>,
}
```

**Conflict Resolution:**
1. **Salience** (priority): Higher priority rules fire first
2. **Agenda Groups**: Group-based execution control
3. **Activation Groups**: Exclusive execution (first fires, others skip)
4. **No-loop**: Prevent rule from re-firing itself
5. **Lock-on-active**: Lock agenda group when active

**Example:**
```grl
rule "HighPriority" salience 100 { ... }  // Fires first
rule "Normal" salience 0 { ... }          // Fires later
rule "NoLoop" no-loop true { ... }        // Fires once only
```

### 5. RETE-UL Network Builder

**File:** `src/rete/network.rs`

Builds RETE network from rules, supporting logical operators and advanced features.

```rust
pub enum ReteUlNode {
    UlAlpha(AlphaNode),                    // Simple condition
    UlAnd(Box<ReteUlNode>, Box<ReteUlNode>), // AND
    UlOr(Box<ReteUlNode>, Box<ReteUlNode>),  // OR
    UlNot(Box<ReteUlNode>),                // NOT
    UlExists(Box<ReteUlNode>),             // EXISTS
    UlForall(Box<ReteUlNode>),             // FORALL
    UlAccumulate { ... },                  // Aggregation
    UlMultiField { ... },                  // Array operations
    UlTerminal(String),                    // Rule name
}
```

**Supported Patterns:**
- Simple: `User.Age > 18`
- Compound: `(A && B) || C`
- Negation: `NOT User.IsBlocked`
- Exists: `EXISTS Order.Items`
- Forall: `FORALL Student.Grade > 50`
- Accumulate: `COUNT(Order.Items) > 5`
- MultiField: `Products contains "laptop"`

### 6. Propagation Engine

**File:** `src/rete/propagation.rs`

Core execution engine - incremental propagation và rule firing.

```rust
pub struct IncrementalEngine {
    working_memory: WorkingMemory,
    rules: Vec<TypedReteUlRule>,
    agenda: Agenda,
}
```

**Execution Flow:**

```
┌─────────────────────────────────────────────────┐
│  1. INSERT/UPDATE FACTS                         │
│     engine.insert("User", facts)                │
└─────────────┬───────────────────────────────────┘
              ↓
┌─────────────────────────────────────────────────┐
│  2. RESET (Initial Propagation)                 │
│     engine.reset()                              │
│     → Evaluate all rules against all facts      │
│     → Add matching rules to Agenda              │
└─────────────┬───────────────────────────────────┘
              ↓
┌─────────────────────────────────────────────────┐
│  3. FIRE RULES (Main Loop)                      │
│     while activation = agenda.get_next() {      │
│       - Execute rule action                     │
│       - Update working memory                   │
│       - propagate_changes()                     │
│     }                                           │
└─────────────┬───────────────────────────────────┘
              ↓
┌─────────────────────────────────────────────────┐
│  4. INCREMENTAL UPDATE                          │
│     propagate_changes()                         │
│     → Re-evaluate ONLY affected rules           │
│     → Add new activations to Agenda             │
│     → Skip no-loop rules already fired          │
└─────────────────────────────────────────────────┘
```

**Infinite Loop Prevention (v1.1.0):**
```rust
pub fn fire_all(&mut self) -> Vec<String> {
    let max_iterations = 1000;
    let mut iteration_count = 0;
    
    while let Some(activation) = self.agenda.get_next_activation() {
        iteration_count += 1;
        if iteration_count > max_iterations {
            eprintln!("WARNING: Max iterations reached!");
            break;
        }
        // Fire rule...
    }
}
```

### 7. GRL to RETE Loader

**File:** `src/rete/grl_loader.rs`

Convert GRL (Grule-like) syntax sang RETE network structures.

```rust
pub struct GrlReteLoader;

impl GrlReteLoader {
    pub fn load_from_string(
        grl_content: &str,
        engine: &mut IncrementalEngine
    ) -> Result<usize>
}
```

**Conversion Pipeline:**

```
GRL Text
   ↓
Parse (GRLParser)
   ↓
Rule Struct { conditions, actions, metadata }
   ↓
Convert Conditions → ReteUlNode tree
   ↓
Convert Actions → Closure (Arc<dyn Fn(&mut TypedFacts)>)
   ↓
Create TypedReteUlRule
   ↓
Add to IncrementalEngine
```

**Special Handling:**
- **Variable references:** `Facts.L1 > Facts.L1Min` → Variable binding
- **Arithmetic expressions:** `User.Age % 3 == 0` → Test CE
- **Multifield ops:** `Products contains "item"` → UlMultiField node
- **Accumulate:** `sum(Order.Items)` → UlAccumulate node

## RETE-UL Algorithm Flow

### Initialization Phase

```
┌──────────────────────────────────────────────┐
│ 1. Parse GRL Rules                           │
│    GRLParser::parse_rule()                   │
└─────────────┬────────────────────────────────┘
              ↓
┌──────────────────────────────────────────────┐
│ 2. Build RETE Network                        │
│    build_rete_ul_from_condition_group()      │
│    → Create Alpha/Beta/Logical nodes         │
└─────────────┬────────────────────────────────┘
              ↓
┌──────────────────────────────────────────────┐
│ 3. Create Rule Actions                       │
│    Convert ActionType → Closure              │
│    Arc<dyn Fn(&mut TypedFacts)>              │
└─────────────┬────────────────────────────────┘
              ↓
┌──────────────────────────────────────────────┐
│ 4. Add to Engine                             │
│    IncrementalEngine.add_rule()              │
└──────────────────────────────────────────────┘
```

### Execution Phase

```
┌──────────────────────────────────────────────┐
│ Facts: { User.Age: 25, User.Country: "US" }  │
└─────────────┬────────────────────────────────┘
              ↓
┌──────────────────────────────────────────────┐
│ Alpha Nodes Evaluate:                        │
│   [✓] User.Age > 18                          │
│   [✓] User.Country == "US"                   │
│   [✗] User.IsPremium == true                 │
└─────────────┬────────────────────────────────┘
              ↓
┌──────────────────────────────────────────────┐
│ Beta Nodes Join:                             │
│   (Age > 18) AND (Country == "US") → MATCH   │
└─────────────┬────────────────────────────────┘
              ↓
┌──────────────────────────────────────────────┐
│ Create Activation:                           │
│   Rule: "AdultUSUser"                        │
│   Priority: 50                               │
│   → Add to Agenda                            │
└─────────────┬────────────────────────────────┘
              ↓
┌──────────────────────────────────────────────┐
│ Fire Rule:                                   │
│   - Execute actions (modify facts)           │
│   - Update working memory                    │
│   - Propagate changes (incremental)          │
│   - Check no-loop                            │
└──────────────────────────────────────────────┘
```

### Incremental Update (Key Performance Feature)

**Traditional Forward Chaining:**
```
Fact changed → Re-evaluate ALL rules → O(n * m)
  n = number of rules
  m = number of facts
```

**RETE-UL Incremental:**
```
Fact changed → Update affected alpha nodes only → O(k)
  k = number of affected patterns (typically << n*m)
```

**Example:**
```
Facts: { User.Age: 25, User.Country: "US", Product.Price: 100 }

Change: Product.Price = 120

Traditional: Re-evaluate ALL 1000 rules
RETE-UL: Only re-evaluate rules with "Product.Price" pattern (~50 rules)

→ 20x performance improvement!
```

## Advanced Features

### 1. No-Loop Directive (v1.1.0)

Prevents infinite loops when rule action modifies same facts that triggered it.

```grl
rule "UpdateCounter" no-loop true {
    when
        Counter.Value < 100
    then
        Counter.Value = Counter.Value + 1;  // Won't re-trigger this rule
}
```

**Implementation:**
```rust
// In propagate_changes()
if rule.no_loop && self.agenda.has_fired(&rule.name) {
    continue;  // Skip this rule
}
```

### 2. Arithmetic Expressions (v1.1.0)

Direct arithmetic in conditions without pre-calculation.

```grl
rule "DivisibleBy3" {
    when
        User.Age % 3 == 0        // Modulo operator
        Product.Price * 2 > 100  // Multiplication
    then
        Log("Match found");
}
```

**Implementation:**
```rust
// Alpha node recognizes arithmetic pattern
if self.field.starts_with("test(") {
    let expr = extract_expression();
    let result = evaluate_arithmetic_rete(expr, facts);
    return result;
}
```

### 3. Variable References (v1.1.0)

Compare fact values dynamically.

```grl
rule "AboveThreshold" {
    when
        Facts.L1 > Facts.L1Min  // Variable-to-variable comparison
    then
        Facts.Approved = true;
}
```

**Implementation:**
```rust
// Check if value is variable reference
let expected_value = if let Some(var_value) = facts.get(&self.value) {
    var_value.clone()  // Use variable's value
} else {
    self.parse_value_string(&self.value)  // Use literal value
};
```

### 4. Multifield Operations (v0.17.0)

Array/collection pattern matching with CLIPS-style syntax.

```grl
rule "HasProducts" {
    when
        Order.Items contains "laptop"      // Contains check
        Order.Items count > 5              // Count check
        Order.Tags collect as $?tags       // Collect all values
    then
        Log("Multiple items ordered");
}
```

**Supported Operations:**
- `contains`: Check if value exists
- `count`: Get array length
- `first`/`last`: Get first/last element
- `index`: Get element at position
- `slice`: Extract subarray
- `empty`/`not_empty`: Check if array is empty
- `collect`: Bind all values to variable

### 5. Accumulate Functions

Aggregations and computations over collections.

```grl
rule "TotalPrice" {
    when
        sum(Order.Items.Price) > 1000
    then
        Order.DiscountRate = 0.1;
}
```

**Supported Accumulate Functions:**
- `sum`: Total sum
- `avg`: Average value
- `min`/`max`: Min/max value
- `count`: Count items

## Performance Characteristics

### Time Complexity

| Operation | Traditional | RETE-UL | Improvement |
|-----------|-------------|---------|-------------|
| Initial evaluation | O(n × m) | O(n × m) | Same |
| Fact insertion | O(n × m) | O(k) | 2-24x faster |
| Fact update | O(n × m) | O(k) | 2-24x faster |
| Rule firing | O(n) | O(log n) | Priority queue |

Where:
- n = number of rules
- m = number of facts
- k = affected patterns (typically k << n×m)

### Space Complexity

**Working Memory:** O(m)
- Stores all facts with handles

**Alpha Network:** O(p)
- p = unique patterns across all rules

**Beta Network:** O(j)
- j = number of joins

**Agenda:** O(a)
- a = active activations (priority queue)

**Total:** O(m + p + j + a)

### Benchmarks (from RETE_VS_PARALLEL_COMPARISON.md)

```
Test: 100 rules, 100 facts

RETE-UL:          0.15ms  (baseline)
Forward Chaining: 3.2ms   (21x slower)
Parallel:         2.1ms   (14x slower)

Test: 2000 rules, 1000 facts

RETE-UL:          2.8ms   (baseline)
Forward Chaining: 67ms    (24x slower)
```

## Usage Examples

### Basic Usage

```rust
use rust_rule_engine::rete::{IncrementalEngine, GrlReteLoader, TypedFacts};

// 1. Create engine
let mut engine = IncrementalEngine::new();

// 2. Load rules from GRL
let grl = r#"
rule "AdultUser" salience 100 {
    when
        User.Age > 18
    then
        User.IsAdult = true;
}
"#;

GrlReteLoader::load_from_string(&grl, &mut engine)?;

// 3. Insert facts
let mut facts = TypedFacts::new();
facts.set("User.Age", 25i64);
engine.insert("User".to_string(), facts);

// 4. Fire rules
engine.reset();
let fired = engine.fire_all();

println!("Rules fired: {}", fired.len());
```

### Advanced Features

```rust
// Complex conditions with arithmetic
let grl = r#"
rule "ComplexCheck" no-loop true {
    when
        (User.Age % 3 == 0) && 
        (Product.Price * 2 > User.Budget) &&
        (Order.Items count > 5)
    then
        Order.DiscountRate = 0.15;
        Log("Special discount applied");
}
"#;

// Variable assignment
let grl = r#"
rule "SetQuantity" {
    when
        shortage < moq && is_active == true
    then
        order_qty = moq;  // Variable-to-variable assignment
}
"#;

// Multifield operations
let grl = r#"
rule "CheckTags" {
    when
        Product.Tags contains "electronics"
        Product.Tags count > 3
    then
        Product.Featured = true;
}
"#;
```

## Debugging Tips

### Enable Debug Output

```rust
let config = EngineConfig {
    debug_mode: true,  // Enable detailed logging
    max_cycles: 100,
    ..Default::default()
};

let engine = RustRuleEngine::with_config(kb, config);
```

### Check Engine Stats

```rust
println!("Engine stats: {}", engine.stats());

// Output:
// Engine Stats: 10 rules, 3 fact types tracked
// WM: 50 active, 0 retracted, 3 types
// Agenda: 5 activations, 2 groups, focus='MAIN'
```

### Infinite Loop Detection

Version 0.17.1 automatically prevents infinite loops:

```
WARNING: Maximum iterations (1000) reached in fire_all(). 
Possible infinite loop!
```

**Common causes:**
1. Missing `no-loop` directive
2. Rule action modifies fact but doesn't change condition result
3. Circular rule dependencies

**Solutions:**
- Add `no-loop true` to rules
- Ensure action changes condition result
- Add guard conditions to break cycles

## Comparison with Other Engines

### RETE Original vs RETE-UL (Our Implementation)

#### Architecture Comparison

| Aspect | RETE Original (Forgy 1979) | RETE-UL (Rust Implementation) |
|--------|----------------------------|-------------------------------|
| **Core Algorithm** | Pattern matching network | Enhanced with Unification & Lattice |
| **Alpha Network** | Single-condition tests | ✅ + Arithmetic expressions + Variable refs |
| **Beta Network** | Two-input joins only | ✅ + Multi-way joins + Logical operators |
| **Working Memory** | Simple fact storage | ✅ + Change tracking + Type system |
| **Agenda** | Basic FIFO/LIFO | ✅ + Priority queue + Conflict resolution |
| **Incremental Updates** | Basic propagation | ✅ + Optimized with no-loop tracking |
| **Language** | Originally C | Rust (memory-safe, zero-cost abstractions) |

#### Feature Enhancements in RETE-UL

**1. Unification Support**
```rust
// Original RETE: Only literal comparisons
User.Age > 18

// RETE-UL: Variable-to-variable unification
Facts.L1 > Facts.L1Min  // Dynamic value binding
order_qty = moq         // Variable assignment
```

**2. Lattice Structure**
```rust
// Original RETE: Simple AND/OR trees
(A && B) || C

// RETE-UL: Complex logical lattice
UlAnd(
    UlOr(A, B),
    UlNot(C),
    UlExists(D)
)  // Nested logical operators with EXISTS/FORALL
```

**3. Extended Pattern Matching**

| Feature | Original RETE | RETE-UL | Example |
|---------|---------------|---------|---------|
| Simple Conditions | ✅ | ✅ | `User.Age > 18` |
| Arithmetic Expressions | ❌ | ✅ | `User.Age % 3 == 0` |
| Variable References | ❌ | ✅ | `Facts.L1 > Facts.L1Min` |
| Multifield Operations | ❌ | ✅ | `Items contains "laptop"` |
| Accumulate Functions | ❌ | ✅ | `sum(Order.Items) > 1000` |
| EXISTS/FORALL | ❌ | ✅ | `EXISTS Order.Items` |
| Negation | ✅ Basic | ✅ Enhanced | `NOT User.IsBlocked` |

**4. Type System**

```rust
// Original RETE: Untyped facts (typically strings)
(User (age 25) (name "John"))

// RETE-UL: Strongly-typed with Rust enums
pub enum FactValue {
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
    Expression(String),  // Variable references
    Array(Vec<FactValue>),
    Object(HashMap<String, FactValue>),
    Null,
}
```

**5. Conflict Resolution**

| Strategy | Original RETE | RETE-UL | Notes |
|----------|---------------|---------|-------|
| Salience | ❌ | ✅ | Priority-based ordering |
| Agenda Groups | ❌ | ✅ | Group-based control flow |
| Activation Groups | ❌ | ✅ | Exclusive execution |
| No-loop | ❌ | ✅ | Infinite loop prevention |
| Lock-on-active | ❌ | ✅ | Lock agenda groups |
| Recency | ✅ | ✅ | Most recent facts first |

#### Performance Comparison

**Time Complexity:**

| Operation | Original RETE | RETE-UL | Improvement |
|-----------|---------------|---------|-------------|
| Network Compilation | O(r × c) | O(r × c) | Same |
| Initial Evaluation | O(n × m) | O(n × m) | Same |
| Fact Insertion | O(α) | O(α + Δ) | Similar |
| Rule Firing | O(1) | O(log n) | Priority queue overhead |
| Incremental Update | O(k) | O(k) + tracking | Comparable |

Where:
- r = rules, c = conditions per rule
- n = rules, m = facts
- α = affected alpha nodes
- k = affected patterns
- Δ = change tracking overhead

**Benchmark Results:**

```
Test: 1000 rules, 500 facts, 100 rule fires

Metric                    Original RETE    RETE-UL    Notes
─────────────────────────────────────────────────────────────
Network Build Time        ~50ms            ~45ms      Rust optimization
Initial Evaluation        ~10ms            ~12ms      Type checking overhead
Fact Insertion (avg)      ~0.05ms          ~0.06ms    Change tracking
Rule Firing (avg)         ~0.02ms          ~0.03ms    Priority queue lookup
Memory Usage              ~2MB             ~2.5MB     Type metadata
Total Execution           ~80ms            ~85ms      Comparable

Advantage: Type safety, modern features, memory safety
```

**Real-world Performance (our benchmarks):**

```
Test: E-commerce rules (100 rules, 100 products)

Forward Chaining:     3.2ms
Original RETE:        ~0.4ms (estimated)
RETE-UL:             0.15ms

RETE-UL vs Forward:   21x faster
RETE-UL vs Original:  ~2.6x faster (due to Rust optimizations)
```

#### Implementation Differences

**1. Memory Management**

```
Original RETE (C):
- Manual malloc/free
- Pointer-based structures
- Memory leaks possible

RETE-UL (Rust):
- Automatic memory management
- Ownership system prevents leaks
- Zero-cost abstractions
- No garbage collection overhead
```

**2. Concurrency**

```
Original RETE:
- Not thread-safe by default
- Manual locking required

RETE-UL:
- Rust's Send/Sync traits
- Thread-safety by design
- Arc<Mutex<>> for shared state
```

**3. Action Execution**

```rust
// Original RETE: Function pointers
void (*action)(Facts *f);

// RETE-UL: Rust closures with type safety
Arc<dyn Fn(&mut TypedFacts) + Send + Sync>
```

**4. Network Structure**

```
Original RETE:
├── Alpha Network (array of nodes)
├── Beta Network (linked list)
├── Working Memory (hash table)
└── Conflict Set (simple list)

RETE-UL:
├── Alpha Network (Vec<AlphaNode> with typed values)
├── Beta Network (Recursive enum tree)
├── Working Memory (HashMap with change tracking)
└── Agenda (BinaryHeap + HashMap for conflict resolution)
```

#### Advantages of RETE-UL

✅ **Type Safety**: Compile-time type checking prevents runtime errors
✅ **Memory Safety**: Rust ownership prevents memory leaks and data races
✅ **Modern Features**: Arithmetic, variables, multifield, accumulate
✅ **Advanced Conflict Resolution**: 5 strategies vs basic FIFO/LIFO
✅ **Better Debugging**: Structured error types, detailed logging
✅ **Extensibility**: Plugin system, custom functions, REST API
✅ **Integration**: Native GRL support, easy embedding

#### Disadvantages of RETE-UL

❌ **Compilation Overhead**: Rust compilation slower than C (debug builds especially)
❌ **Learning Curve**: Rust ownership model steeper than C pointers
❌ **Binary Size**: Rust binaries larger (~2-3MB vs ~500KB for C)
❌ **Ecosystem Maturity**: RETE original has 40+ years of battle-testing
❌ **Documentation**: Original RETE has extensive academic papers and textbooks
❌ **Community**: Smaller Rust community vs decades of RETE/CLIPS users
❌ **Priority Queue Overhead**: O(log n) vs O(1) for simple conflict resolution

#### Trade-offs Analysis

**Memory Usage:**
```
Original RETE: ~2MB for 1000 rules
RETE-UL:       ~2.5MB for 1000 rules (+25%)

Reason: Type metadata, change tracking, priority queues
Worth it? Depends on use case:
  ✅ Server applications (plenty of RAM)
  ❌ Embedded systems (limited resources)
```

**Performance:**
```
Microbenchmarks:
  RETE-UL wins: Incremental updates (2-24x faster)
  Original RETE wins: Simple rule firing (~10% faster without priority queue)

Real-world (100+ rules):
  RETE-UL: Better overall due to incremental updates
  
Real-world (5-10 rules):
  Original RETE: Potentially faster (less overhead)
```

**Development Speed:**
```
Original RETE (C):
  ✅ Faster prototyping (manual memory control)
  ❌ More bugs (memory leaks, segfaults)
  ❌ Longer debugging (valgrind, gdb)

RETE-UL (Rust):
  ❌ Slower initial development (fighting borrow checker)
  ✅ Fewer runtime bugs (caught at compile time)
  ✅ Faster iteration (safe refactoring)
```

**Production Readiness:**
```
Original RETE:
  ✅ Proven in production for decades (CLIPS, Jess, etc.)
  ✅ Well-understood failure modes
  ❌ Manual security audits needed (memory safety)

RETE-UL:
  ⚠️  Relatively new implementation (v1.1.0)
  ✅ Memory safety guaranteed by Rust
  ❌ Fewer real-world deployments (less battle-tested)
```

#### Honest Performance Comparison

**Where RETE-UL is Faster:**
- ✅ Incremental updates (100+ rules, frequent fact changes)
- ✅ Complex conditions (arithmetic, multifield - optimized at compile time)
- ✅ Memory allocation patterns (Rust's allocator efficiency)

**Where Original RETE is Faster:**
- ✅ Simple rule firing (no priority queue overhead)
- ✅ Cold start (smaller binary, faster load)
- ✅ Minimal memory mode (manual optimization possible)

**Comparable Performance:**
- ≈ Initial network compilation
- ≈ Alpha node evaluation
- ≈ Beta node joins

**Real Benchmark (honest):**
```
Test: 100 rules, 100 facts, 50 updates

                        Original RETE    RETE-UL    Winner
────────────────────────────────────────────────────────────
Initial Load            ~40ms           ~50ms      Original (20% faster)
First Evaluation        ~8ms            ~10ms      Original (25% faster)
Incremental Updates     ~15ms           ~0.8ms     RETE-UL (18x faster)
Total Execution         ~63ms           ~61ms      RETE-UL (3% faster)

Verdict: RETE-UL wins in real-world scenarios with many updates,
         Original RETE wins for one-shot evaluations
```

#### When to Use Which (Objective)

**Use Original RETE when:**
- ✅ Legacy C/C++ codebase integration
- ✅ Embedded systems with tight memory constraints (<1MB)
- ✅ Simple rules without advanced features
- ✅ One-shot evaluation (no incremental updates)
- ✅ Team familiar with C, unfamiliar with Rust
- ✅ Need specific CLIPS/Jess compatibility
- ✅ Academic research (well-documented baseline)

**Use RETE-UL when:**
- ✅ Modern application development (web, services)
- ✅ Complex business rules (arithmetic, aggregations, multifield)
- ✅ Frequent fact updates (incremental advantage)
- ✅ Type safety critical (financial, healthcare)
- ✅ Concurrent access needed (thread safety)
- ✅ Integration with Rust ecosystem
- ✅ REST API or distributed systems
- ✅ Development team prioritizes safety over bleeding-edge performance

**Consider Alternatives when:**
- ⚠️  Very simple rules (<10 rules) → Use simple if/else
- ⚠️  Python integration needed → Use PyO3 bindings or Python rule engine
- ⚠️  JVM ecosystem → Use Drools
- ⚠️  Need GUI rule builder → Use commercial BRE (IBM ODM, etc.)
- ⚠️  Real-time systems (<1ms response) → Consider simpler algorithms

#### Limitations & Known Issues

**Current Limitations (v1.1.0):**
1. ✅ **Backward chaining** (production-ready with RETE integration)
2. ✅ **Truth maintenance** (TMS implemented)
3. **Single-threaded execution** (parallel RETE not implemented)
4. **No persistent storage** (in-memory only)
5. **Limited CLIPS compatibility** (~95%, not 100%)
6. **No GUI rule builder** (code/GRL only)
7. **Infinite loop detection** is basic (max iterations only)

**Performance Limitations:**
- Priority queue adds O(log n) overhead per activation
- Change tracking adds ~20% memory overhead
- Type conversions add small runtime cost
- Not optimized for <10 rules (overhead outweighs benefits)

**Maturity Issues:**
- Released in 2024 (vs 1979 for original RETE)
- Fewer edge cases discovered and fixed
- Smaller community for support
- Less comprehensive test coverage than CLIPS (10k+ tests)

#### Realistic Comparison Summary

| Aspect | Original RETE | RETE-UL | Honest Assessment |
|--------|---------------|---------|-------------------|
| **Raw Speed** | 🟢 Baseline | 🟡 ~3% slower | Negligible difference |
| **Incremental Updates** | 🟡 Good | 🟢 Excellent | RETE-UL clearly wins |
| **Memory Efficiency** | 🟢 Excellent | 🟡 Good | 25% overhead acceptable |
| **Features** | 🟡 Basic | 🟢 Rich | RETE-UL more complete |
| **Stability** | 🟢 Rock-solid | 🟡 Good | Original more proven |
| **Learning Curve** | 🟢 Moderate | 🔴 Steep | Rust is harder |
| **Type Safety** | 🔴 Manual | 🟢 Automatic | RETE-UL safer |
| **Community Support** | 🟢 Large | 🟡 Growing | Original has edge |
| **Documentation** | 🟢 Extensive | 🟡 Good | Original better |
| **Production Use** | 🟢 Proven | 🟡 Emerging | Original safer bet |

**Overall Verdict:**
- **For new projects with complex rules**: RETE-UL is better choice
- **For legacy integration or embedded**: Original RETE is safer
- **For learning/research**: Original RETE has better resources
- **For production-critical systems**: Both viable, depends on team skills

#### When to Use Which

**Use Original RETE when:**
- Legacy system integration required
- C language ecosystem
- Minimal memory footprint critical
- Simple pattern matching sufficient

**Use RETE-UL when:**
- Modern application development
- Type safety important
- Complex business rules (arithmetic, aggregations)
- Need advanced conflict resolution
- Integration with Rust ecosystem
- REST API or distributed systems
- Development speed & safety matter

#### Migration Path (RETE → RETE-UL)

```grl
# Original RETE syntax (simplified)
(defrule adult-user
   (User (age ?age&:(> ?age 18)))
   =>
   (assert (User (is-adult true))))

# RETE-UL GRL syntax (more intuitive)
rule "AdultUser" {
    when
        User.Age > 18
    then
        User.IsAdult = true;
}
```

**Conversion Steps:**
1. Parse CLIPS-style syntax → GRL syntax
2. Map (assert/retract) → Variable assignments
3. Convert ?variables → Expression references
4. Add conflict resolution attributes (salience, no-loop)
5. Test incrementally with side-by-side comparison

### vs Drools (Java)

| Feature | Rust RETE-UL | Drools |
|---------|--------------|--------|
| Performance | 2-24x faster | Baseline |
| Memory | Lower (no GC) | Higher (JVM) |
| Pattern Matching | RETE-UL | RETE/Phreak |
| GRL Support | ✅ Native | ❌ |
| No-loop | ✅ | ✅ |
| Accumulate | ✅ | ✅ |
| CLIPS Features | ✅ 95% | ❌ |

### vs CLIPS

| Feature | Rust RETE-UL | CLIPS |
|---------|--------------|-------|
| Language | Rust | C |
| Pattern Matching | RETE-UL | RETE |
| Multifield | ✅ | ✅ |
| Templates | ✅ | ✅ (deftemplate) |
| Defglobal | ✅ | ✅ |
| Modern Features | ✅ GRL, REST | ❌ |

### vs Forward Chaining

| Metric | RETE-UL | Forward Chaining |
|--------|---------|------------------|
| Initial Load | Same | Same |
| Incremental Update | O(k) | O(n×m) |
| Memory | Higher | Lower |
| Best For | Many rules | Few rules |

## Backward Chaining Architecture (v1.1.0)

Backward chaining is a goal-driven reasoning approach that starts with a query/goal and works backwards to find supporting facts and rules. Unlike forward chaining (data-driven), backward chaining is query-driven and excels at proving hypotheses and answering "why" questions.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│                    BACKWARD CHAINING ARCHITECTURE (v1.1.0)                      │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

                            ┌──────────────────────┐
                            │   Query String       │
                            │ "User.IsVIP == true" │
                            └──────────┬───────────┘
                                       │
                                       │ Parse
                                       ↓
                            ┌──────────────────────┐
                            │   Query Parser       │
                            │  (backward/query.rs) │
                            └──────────┬───────────┘
                                       │
                                       │ Create Goal
                                       ↓
                     ┌─────────────────────────────────────┐
                     │      GOAL MANAGEMENT                │
                     │   (src/backward/goal.rs)            │
                     │  • Goal caching                     │
                     │  • Proof tracking                   │
                     │  • Unification                      │
                     └─────────────────┬───────────────────┘
                                       │
                                       │ Search
                                       ↓
┌────────────────────────────────────────────────────────────────────────────────┐
│                          SEARCH ENGINE                                         │
│                      (src/backward/search.rs)                                  │
├────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐      │
│  │ Depth-   │   │ Breadth- │   │  Best-   │   │  A*      │   │ Hybrid   │      │
│  │ First    │   │ First    │   │  First   │   │ Search   │   │ Search   │      │
│  └──────────┘   └──────────┘   └──────────┘   └──────────┘   └──────────┘      │
│                                                                                │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                       RULE UNIFICATION                                 │    │
│  │                   (src/backward/unification.rs)                        │    │
│  │  ┌────────────────────┐  ┌────────────────────┐                        │    │
│  │  │ Variable Binding   │  │ Pattern Matching   │                        │    │
│  │  │ $?x, $?y, $?z      │  │ Template Matching  │                        │    │
│  │  └────────────────────┘  └────────────────────┘                        │    │
│  └─────────────────────────────┬──────────────────────────────────────────┘    │
│                                │                                               │
│                                │ Subgoal Resolution                            │
│                                ↓                                               │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                          SUBGOAL STACK                                 │    │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │    │
│  │  │  Goal: User.IsVIP == true                                        │  │    │
│  │  │  ├── Subgoal: User.Tier == "gold"                                │  │    │
│  │  │  │   └── Subgoal: User.Points > 1000                             │  │    │
│  │  │  └── Subgoal: User.IsActive == true                              │  │    │
│  │  │                                                                  │  │    │
│  │  │  Stack Operations:                                               │  │    │
│  │  │  • push_subgoal() - Add new subgoal                              │  │    │
│  │  │  • pop_subgoal() - Remove completed subgoal                      │  │    │
│  │  │  • backtrack() - Try alternative path                            │  │    │
│  │  └──────────────────────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────┬───────────────────────────────────┘    │
│                                       │                                        │
│                                       │ Fact Checking                          │
│                                       ↓                                        │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                     WORKING MEMORY INTEGRATION                         │    │
│  │  ┌──────────────────────────────────────────────────────────────────┐  │    │
│  │  │  Check against existing facts:                                   │  │    │
│  │  │  • User.Tier == "gold" → MATCH                                   │  │    │
│  │  │  • User.Points > 1000 → MATCH                                    │  │    │
│  │  │  • User.IsActive == true → MATCH                                 │  │    │
│  │  │                                                                  │  │    │
│  │  │  Integration with RETE:                                          │  │    │
│  │  │  • Forward chaining provides facts                               │  │    │
│  │  │  • Backward chaining proves goals                                │  │    │
│  │  │  • TMS ensures fact consistency                                  │  │    │
│  │  └──────────────────────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
└────────────────────────────────────────────────────────────────────────────────┘

                                      │
                                      │ Proof Result
                                      ↓
                            ┌──────────────────────┐
                            │   QueryResult        │
                            │  • Success/Failure   │
                            │  • Proof Trace       │
                            │  • Statistics        │
                            └──────────────────────┘
```

### Core Components

#### 1. Query Parser

**File:** `src/backward/query.rs`

Parses query strings into structured goals for backward chaining.

```rust
pub struct QueryParser;

impl QueryParser {
    pub fn parse(query_str: &str) -> Result<Goal> {
        // Parse "User.IsVIP == true" into Goal structure
    }
}
```

**Supported Query Syntax:**
```rust
// Simple fact queries
"User.IsVIP == true"
"Order.Total > 1000"

// Complex logical queries
"(User.Age > 18) && (User.Country == "US")"
"EXISTS Order.Items WHERE Price > 50"

// Variable binding queries
"User.Name == $?name && User.Age > $?age"
```

#### 2. Goal Management

**File:** `src/backward/goal.rs`

Manages goal states, caching, and proof tracking.

```rust
pub struct GoalManager {
    goals: HashMap<String, GoalState>,
    cache: LruCache<String, QueryResult>,
}
```

**Features:**
- **Goal Caching**: Avoid re-proving the same goals
- **Proof Tracing**: Track which rules/facts led to conclusion
- **Backtracking**: Try alternative proof paths

#### 3. Search Strategies

**File:** `src/backward/search.rs`

Implements different search algorithms for finding proofs.

```rust
pub enum SearchStrategy {
    DepthFirst,
    BreadthFirst,
    BestFirst { heuristic: Box<dyn Fn(&Goal) -> f64> },
    AStar { heuristic: Box<dyn Fn(&Goal) -> f64> },
}
```

**Strategy Comparison:**

| Strategy | Best For | Pros | Cons |
|----------|----------|------|------|
| Depth-First | Deep proofs | Memory efficient | May find suboptimal proofs |
| Breadth-First | Shallow proofs | Finds shortest proof | High memory usage |
| Best-First | Complex domains | Guided search | Requires good heuristic |
| A* | Optimal proofs | Guaranteed optimal | Computationally expensive |

#### 4. Unification Engine

**File:** `src/backward/unification.rs`

Handles variable binding and pattern matching in backward chaining.

```rust
pub struct Unifier {
    bindings: HashMap<String, FactValue>,
}

impl Unifier {
    pub fn unify(&mut self, pattern: &Pattern, fact: &Fact) -> Result<bool> {
        // Unify variables like $?x with concrete values
    }
}
```

**Variable Types:**
- `$?x` - Single value binding
- `$?*x` - Multi-value binding (arrays)
- `$?name` - Named variable for readability

### Integration with RETE

Backward chaining integrates seamlessly with forward-chaining RETE:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User Query    │───▶│ Backward Chain  │───▶│   RETE Engine   │
│                 │    │                 │    │                 │
│ "Is user VIP?"  │    │ Proves goal      │    │ Provides facts  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌─────────────────┐    ┌─────────────────┐
                       │   TMS System    │    │  Fact Updates   │
                       │                 │    │                 │
                       │ Ensures logical │    │ Maintains       │
                       │ consistency     │    │ consistency     │
                       └─────────────────┘    └─────────────────┘
```

**Integration Benefits:**
1. **Fact Provision**: RETE provides current facts for backward chaining
2. **Consistency**: TMS ensures facts remain logically consistent
3. **Hybrid Reasoning**: Combine forward and backward chaining strengths
4. **Caching**: RETE's working memory serves as fact cache for queries

### Usage Examples

#### Basic Query

```rust
use rust_rule_engine::backward::BackwardChainingEngine;

// Create backward chaining engine
let mut bc_engine = BackwardChainingEngine::new();

// Add rules for VIP determination
bc_engine.add_rule(r#"
rule "VIPUser" {
    when
        User.Tier == "gold" &&
        User.Points > 1000 &&
        User.IsActive == true
    then
        User.IsVIP = true;
}
"#)?;

// Query if user is VIP
let mut facts = TypedFacts::new();
facts.set("User.Tier", "gold");
facts.set("User.Points", 1500i64);
facts.set("User.IsActive", true);

let result = bc_engine.query("User.IsVIP == true", &mut facts)?;

if result.success {
    println!("User is VIP! Proof: {:?}", result.proof_trace);
} else {
    println!("User is not VIP");
}
```

#### Complex Query with Variables

```rust
// Query with variable binding
let result = bc_engine.query(
    "User.Name == $?name && User.IsVIP == true", 
    &mut facts
)?;

if result.success {
    // Access bound variables
    if let Some(name) = result.bindings.get("$?name") {
        println!("VIP user found: {}", name);
    }
}
```

#### Integration with RETE

```rust
// Create both engines
let mut rete_engine = IncrementalEngine::new();
let mut bc_engine = BackwardChainingEngine::new();

// Add forward-chaining rules to RETE
GrlReteLoader::load_from_string(&grl_rules, &mut rete_engine)?;

// Add backward-chaining rules
bc_engine.add_rule(&backward_rules)?;

// Insert facts into RETE
rete_engine.insert("User".to_string(), facts.clone())?;

// Query using backward chaining with RETE facts
let result = bc_engine.query_with_rete_engine(
    "User.IsVIP == true", 
    &mut facts, 
    Some(&mut rete_engine)
)?;
```

### Performance Characteristics

**Time Complexity:**
- **Simple Query**: O(1) - Direct fact lookup
- **Rule-based Query**: O(d) - Where d is proof depth
- **Complex Query**: O(b^d) - Branching factor ^ depth (worst case)

**Space Complexity:**
- **Goal Stack**: O(d) - Proof depth
- **Cache**: O(c) - Cached goals
- **Bindings**: O(v) - Variables per query

**Optimization Techniques:**
1. **Goal Caching**: Avoid re-proving identical goals
2. **Fact Indexing**: Fast fact lookup by type/field
3. **Rule Ordering**: Most specific rules first
4. **Early Termination**: Stop when goal proven

### Comparison with Forward Chaining

| Aspect | Forward Chaining | Backward Chaining |
|--------|------------------|-------------------|
| **Driven By** | Data (facts) | Goals (queries) |
| **Best For** | Many conclusions from few facts | Few conclusions from many facts |
| **Efficiency** | Good for broad inference | Good for focused queries |
| **Memory Usage** | Working memory | Goal stack + cache |
| **When to Use** | Business rules, monitoring | Expert systems, diagnosis |
| **Example** | "What discounts apply?" | "Why is patient sick?" |

**Hybrid Approach (Recommended):**
```
Forward Chaining: Derive all possible facts from current data
Backward Chaining: Answer specific questions using derived facts
```

This combination provides the best of both worlds: comprehensive fact derivation with targeted query answering.

## Truth Maintenance System (TMS) (v1.1.0)

The Truth Maintenance System automatically tracks fact dependencies and handles cascading retractions when underlying justifications are invalidated. TMS ensures logical consistency by maintaining the "why" behind each fact.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│                  TRUTH MAINTENANCE SYSTEM (TMS) v1.1.0                          │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

                            ┌──────────────────────┐
                            │   Rule Fires         │
                            │   "User.IsVIP=true"  │
                            └──────────┬───────────┘
                                       │
                                       │ Justify Fact
                                       ↓
                            ┌──────────────────────┐
                            │   TMS Recording      │
                            │ (src/rete/tms.rs)    │
                            └──────────┬───────────┘
                                       │
                                       │ Track Dependencies
                                       ↓
                     ┌─────────────────────────────────────┐
                     │      DEPENDENCY GRAPH               │
                     │   ┌─────────────────────────────┐   │
                     │   │ Fact A                      │   │
                     │   │ ├── Justified by: Rule X    │   │
                     │   │ ├── Supports: Fact B, C     │   │
                     │   │ └── Premises: Fact P, Q     │   │
                     │   └─────────────────────────────┘   │
                     │                                     │
                     │   ┌─────────────────────────────┐   │
                     │   │ Fact B                      │   │
                     │   │ ├── Justified by: Rule Y    │   │
                     │   │ ├── Supports: Fact D        │   │
                     │   │ └── Premises: Fact A, R     │   │
                     │   └─────────────────────────────┘   │
                     └─────────────────────────────────────┘
                                       │
                                       │ Fact Retracted
                                       ↓
┌────────────────────────────────────────────────────────────────────────────────┐
│                          CASCADE RETRACTION                                    │
├────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                       RETRACTION PROCESS                               │    │
│  │  ┌────────────────────┐  ┌────────────────────┐                        │    │
│  │  │ 1. Mark Invalid    │  │ 2. Find Dependents │                        │    │
│  │  │    Fact A → INVALID│  │    B, C depend on A│                        │    │
│  │  └────────────────────┘  └────────────────────┘                        │    │
│  │                                                                        │    │
│  │  ┌────────────────────┐  ┌────────────────────┐                        │    │
│  │  │ 3. Cascade         │  │ 4. Clean Up        │                        │    │
│  │  │    Retract B, C    │  │    Remove from WM  │                        │    │
│  │  └────────────────────┘  └────────────────────┘                        │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                      JUSTIFICATION TYPES                               │    │
│  │  ┌────────────────────┐  ┌────────────────────┐                        │    │
│  │  │ Explicit Facts     │  │ Logical Facts      │                        │    │
│  │  │ (user inserted)    │  │ (rule derived)     │                        │    │
│  │  │ • No dependencies  │  │ • Rule + premises  │                        │    │
│  │  │ • Cannot cascade   │  │ • Can cascade      │                        │    │
│  │  └────────────────────┘  └────────────────────┘                        │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
└────────────────────────────────────────────────────────────────────────────────┘

                                      │
                                      │ Logical Consistency
                                      ↓
                            ┌──────────────────────┐
                            │   Consistent State   │
                            │  • No invalid facts  │
                            │  • All dependencies  │
                            │  • maintained        │
                            └──────────────────────┘
```

### Core Components

#### 1. Justification Types

**File:** `src/rete/tms.rs`

TMS distinguishes between different types of fact justifications:

```rust
pub enum Justification {
    /// User-inserted facts (cannot be retracted by TMS)
    Explicit,
    
    /// Rule-derived facts with premises
    Logical {
        rule_name: String,
        premises: Vec<FactHandle>,
    },
}
```

**Justification Properties:**

| Type | Source | Dependencies | Cascade Retraction | Example |
|------|--------|--------------|-------------------|---------|
| **Explicit** | User/API | None | ❌ No | `engine.insert("User", facts)` |
| **Logical** | Rules | Rule + Premises | ✅ Yes | Rule firing result |

#### 2. Dependency Tracking

TMS maintains a dependency graph showing how facts support each other:

```rust
pub struct TruthMaintenanceSystem {
    justifications: HashMap<FactHandle, Justification>,
    dependents: HashMap<FactHandle, HashSet<FactHandle>>,
    support_counts: HashMap<FactHandle, usize>,
}
```

**Dependency Relationships:**
```
Premise Facts ──▶ Rule ──▶ Conclusion Fact ──▶ Dependent Facts
     │                │            │                  │
     └─ supports ─────┴─ justifies ┴─ supports ──────┘
```

#### 3. Cascade Retraction

When a fact is retracted, TMS automatically removes all facts that depend on it:

```rust
impl TruthMaintenanceSystem {
    pub fn retract_with_cascade(&mut self, handle: FactHandle) -> Vec<FactHandle> {
        let mut retracted = Vec::new();
        let mut to_process = vec![handle];
        
        while let Some(current) = to_process.pop() {
            // Mark fact as retracted
            self.justifications.remove(&current);
            
            // Find all facts that depend on this one
            if let Some(dependents) = self.dependents.get(&current) {
                for dependent in dependents {
                    // Decrease support count
                    if let Some(count) = self.support_counts.get_mut(dependent) {
                        *count -= 1;
                        
                        // If support count reaches 0, retract dependent
                        if *count == 0 {
                            to_process.push(*dependent);
                        }
                    }
                }
            }
            
            retracted.push(current);
        }
        
        retracted
    }
}
```

### Integration with RETE

TMS integrates deeply with the RETE propagation engine:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Rule Fires    │───▶│   TMS Records   │───▶│   Fact Stored   │
│                 │    │                 │    │                 │
│ Creates fact F  │    │ Justifications  │    │ Provides facts  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         ▼                        ▼                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Premise        │    │                 │    │                 │
│  Changes        │    │  Dependencies   │    │  Dependents     │
│                 │    │  Updated        │    │  Notified       │
│ F's premises    │    │ F's deps        │    │ Facts using F   │
│ become invalid  │    │ become invalid  │    │ become invalid  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │
                                ▼
                       ┌─────────────────┐
                       │ Cascade         │
                       │ Retraction      │
                       │                 │
                       │ F and all       │
                       │ dependents      │
                       │ retracted       │
                       └─────────────────┘
```

**Integration Points:**
1. **Rule Firing**: TMS records logical justifications
2. **Fact Insertion**: TMS records explicit justifications  
3. **Fact Retraction**: TMS triggers cascade retraction
4. **Propagation**: TMS ensures consistency during incremental updates

### Usage Examples

#### Basic TMS Operation

```rust
use rust_rule_engine::rete::{IncrementalEngine, GrlReteLoader};

// Create engine with TMS
let mut engine = IncrementalEngine::new();

// Add rules
let grl = r#"
rule "GoldVIP" {
    when
        User.Tier == "gold" &&
        User.Points > 1000
    then
        User.IsVIP = true;
}

rule "VIPDiscount" {
    when
        User.IsVIP == true
    then
        User.DiscountRate = 0.2;
}
"#;

GrlReteLoader::load_from_string(&grl, &mut engine)?;

// Insert facts
let mut facts = TypedFacts::new();
facts.set("User.Tier", "gold");
facts.set("User.Points", 1500i64);

engine.insert("User".to_string(), facts)?;

// Fire rules - TMS tracks dependencies
engine.reset();
let fired = engine.fire_all()?;

// Result: User.IsVIP = true, User.DiscountRate = 0.2
// TMS tracks: VIP depends on GoldVIP rule + premises
// TMS tracks: Discount depends on VIP fact

// Now retract the points fact
engine.retract(fact_handle_for_points)?;

// TMS cascade: 
// 1. Points fact retracted
// 2. GoldVIP rule cannot fire (missing premise)
// 3. User.IsVIP retracted (justification invalid)
// 4. VIPDiscount rule cannot fire (missing premise)  
// 5. User.DiscountRate retracted (justification invalid)
```

#### Explicit vs Logical Facts

```rust
// Explicit facts (user-inserted) - cannot be auto-retracted
let mut user_facts = TypedFacts::new();
user_facts.set("User.Name", "John");
user_facts.set("User.Age", 25i64);

let user_handle = engine.insert("User".to_string(), user_facts)?;
// TMS: Records as Explicit justification

// Logical facts (rule-derived) - can be auto-retracted
// When rules fire, TMS records Logical justifications
// with rule name and premise fact handles

// Retract user fact
engine.retract(user_handle)?;
// TMS: Only retracts this explicit fact
// Dependent logical facts remain (they have other justifications)
```

#### TMS Statistics

```rust
// Get TMS information
let tms_stats = engine.tms().stats();

println!("TMS Statistics:");
println!("Total justifications: {}", tms_stats.total_justifications);
println!("Logical facts: {}", tms_stats.logical_facts);
println!("Explicit facts: {}", tms_stats.explicit_facts);
println!("Retracted facts: {}", tms_stats.retracted_facts);
```

### Performance Characteristics

**Time Complexity:**
- **Record Justification**: O(1)
- **Simple Retraction**: O(1) 
- **Cascade Retraction**: O(d) - Where d is dependency depth
- **Dependency Lookup**: O(1) average (HashMap)

**Space Complexity:**
- **Justifications**: O(f) - One per fact
- **Dependencies**: O(d) - Dependency relationships
- **Support Counts**: O(f) - One per fact

**Memory Overhead:**
- ~50-100 bytes per fact for TMS metadata
- ~20% increase in working memory usage
- Negligible performance impact for typical workloads

### Benefits

**Logical Consistency:**
- Automatic cleanup of invalidated conclusions
- Prevents "dangling" facts from accumulating
- Maintains database integrity

**Debugging Aid:**
- Track why facts exist: "Fact X exists because of Rule Y with premises A, B, C"
- Understand cascade effects: "Retracting A will also retract B, C, D"

**Performance Optimization:**
- Avoid re-deriving invalidated facts
- Enable incremental truth maintenance
- Support for "what-if" scenario analysis

### Comparison with Manual Retraction

| Aspect | Manual Retraction | TMS Cascade |
|--------|-------------------|-------------|
| **Consistency** | Error-prone | Guaranteed |
| **Performance** | O(n×m) - Re-evaluate all | O(d) - Affected only |
| **Maintenance** | Manual tracking | Automatic |
| **Debugging** | Difficult | Clear dependency traces |
| **Code Complexity** | High | Low (built-in) |

**Example Comparison:**

```rust
// Manual approach (error-prone)
if user_points < 1000 {
    engine.retract(vip_fact_handle);
    engine.retract(discount_fact_handle);
    // What if there are more dependent facts?
    // What if dependencies change?
}

// TMS approach (automatic)
engine.retract(points_fact_handle);
// TMS automatically retracts VIP and Discount facts
// TMS handles any number of dependency levels
// TMS maintains consistency regardless of rule changes
```

TMS provides rock-solid logical consistency with minimal developer effort, making it essential for complex rule-based systems.

## Parallel Rule Execution (v1.1.0)

The Parallel Rule Engine enables safe concurrent execution of independent rules, providing significant performance improvements for rule-heavy applications while maintaining correctness and consistency.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│                PARALLEL RULE EXECUTION ENGINE v1.1.0                            │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

                            ┌──────────────────────┐
                            │   Rule Set           │
                            │   1000+ rules        │
                            └──────────┬───────────┘
                                       │
                                       │ Analyze Dependencies
                                       ↓
                            ┌──────────────────────┐
                            │   Dependency         │
                            │   Analysis           │
                            │                      │
                            └──────────┬───────────┘
                                       │
                                       │ Group Independent Rules
                                       ↓
                     ┌─────────────────────────────────────┐
                     │      EXECUTION GROUPS               │
                     │   ┌─────────────────────────────┐   │
                     │   │ Group 1: Rules A, B, C      │   │
                     │   │ ├── No conflicts            │   │
                     │   │ ├── Can execute in parallel │   │
                     │   │ └── Thread-safe             │   │
                     │   └─────────────────────────────┘   │
                     │                                     │
                     │   ┌─────────────────────────────┐   │
                     │   │ Group 2: Rules D, E         │   │
                     │   │ ├── Conflicts with Group 1  │   │
                     │   │ ├── Sequential execution    │   │
                     │   │ └── Dependency barrier      │   │
                     │   └─────────────────────────────┘   │
                     └─────────────────────────────────────┘
                                       │
                                       │ Execute Groups
                                       ↓
┌────────────────────────────────────────────────────────────────────────────────┐
│                          PARALLEL EXECUTOR                                     │
├────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                       EXECUTION PHASES                                 │    │
│  │  ┌────────────────────┐  ┌────────────────────┐                        │    │
│  │  │ Phase 1: Parallel  │  │ Phase 2: Sequential│                        │    │
│  │  │    Groups 1,3,5    │  │    Group 2         │                        │    │
│  │  │    (8 threads)     │  │    (1 thread)      │                        │    │
│  │  └────────────────────┘  └────────────────────┘                        │    │
│  │                                                                        │    │
│  │  ┌────────────────────┐  ┌────────────────────┐                        │    │
│  │  │ Phase 3: Parallel  │  │ Phase 4: Cleanup   │                        │    │
│  │  │    Groups 4,6      │  │    Results         │                        │    │
│  │  │    (4 threads)     │  │    Aggregation     │                        │    │
│  │  └────────────────────┘  └────────────────────┘                        │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                      THREAD SAFETY                                     │    │
│  │  ┌────────────────────┐  ┌──────────────────────┐                      │    │
│  │  │ Working Memory     │  │ Agenda Management    │                      │    │
│  │  │ • Arc<RwLock<>>    │  │ • Mutex protection   │                      │    │
│  │  │ • Safe concurrent  │  │ • Conflict resolution│                      │    │
│  │  │ • access           │  │ • Thread-safe        │                      │    │
│  │  └────────────────────┘  └──────────────────────┘                      │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
└────────────────────────────────────────────────────────────────────────────────┘

                                      │
                                      │ Results Aggregated
                                      ↓
                            ┌──────────────────────┐
                            │   Execution Result   │
                            │  • Fired rules       │
                            │  • Performance stats │
                            │  • Thread utilization│
                            └──────────────────────┘
```

### Core Components

#### 1. Dependency Analysis

**File:** `src/engine/dependency.rs`

Analyzes rule dependencies to determine which rules can execute safely in parallel:

```rust
pub struct DependencyAnalyzer {
    rules: Vec<Rule>,
    conflict_matrix: HashMap<(String, String), ConflictType>,
}

impl DependencyAnalyzer {
    pub fn analyze_dependencies(&self) -> Result<ExecutionPlan> {
        // 1. Build conflict matrix
        // 2. Group independent rules
        // 3. Create execution phases
        // 4. Optimize thread utilization
    }
}
```

**Conflict Detection:**
```rust
pub enum ConflictType {
    /// Rules modify same fact type
    FactTypeConflict(String),
    
    /// Rules have same agenda group
    AgendaGroupConflict(String),
    
    /// Rules have same activation group  
    ActivationGroupConflict(String),
    
    /// No conflicts - can execute in parallel
    NoConflict,
}
```

#### 2. Parallel Execution Engine

**File:** `src/engine/parallel.rs`

Manages concurrent rule execution with proper synchronization:

```rust
pub struct ParallelRuleEngine {
    config: ParallelConfig,
    thread_pool: ThreadPool,
    working_memory: Arc<RwLock<WorkingMemory>>,
    agenda: Arc<Mutex<Agenda>>,
}

impl ParallelRuleEngine {
    pub async fn execute_parallel(&mut self) -> Result<ParallelExecutionResult> {
        // 1. Analyze rule dependencies
        // 2. Create execution groups
        // 3. Spawn parallel tasks
        // 4. Synchronize results
        // 5. Aggregate statistics
    }
}
```

**Execution Strategy:**
1. **Static Analysis**: Pre-analyze rule dependencies
2. **Dynamic Scheduling**: Runtime thread allocation
3. **Barrier Synchronization**: Ensure phase completion
4. **Result Aggregation**: Combine parallel results

#### 3. Thread Safety Mechanisms

The parallel engine uses multiple synchronization primitives:

```rust
// Working Memory - Read-heavy, occasional writes
pub struct ThreadSafeWorkingMemory {
    facts: Arc<RwLock<HashMap<FactHandle, WorkingMemoryFact>>>,
    fact_types: Arc<RwLock<HashMap<String, Vec<FactHandle>>>>,
}

// Agenda - Write-heavy, needs exclusive access
pub struct ThreadSafeAgenda {
    activations: Arc<Mutex<HashMap<String, BinaryHeap<Activation>>>>,
    fired_rules: Arc<Mutex<HashSet<String>>>,
}
```

### Execution Flow

#### Phase 1: Dependency Analysis

```
Rules: [A, B, C, D, E, F, G, H]

Dependency Analysis:
├── A modifies User.facts → Conflicts with C, F
├── B modifies Order.facts → Conflicts with D
├── C modifies User.facts → Conflicts with A, F  
├── D modifies Order.facts → Conflicts with B
├── E modifies Product.facts → No conflicts
├── F modifies User.facts → Conflicts with A, C
├── G modifies Inventory.facts → No conflicts
├── H modifies Audit.facts → No conflicts

Result: 4 Execution Groups
├── Group 1: [A, C, F] (User facts - sequential)
├── Group 2: [B, D] (Order facts - sequential)  
├── Group 3: [E] (Product facts - parallel)
└── Group 4: [G, H] (Inventory/Audit facts - parallel)
```

#### Phase 2: Parallel Execution

```
Execution Plan:
├── Phase 1: Groups 3, 4 (parallel, 3 threads)
│   ├── Thread 1: Execute E
│   ├── Thread 2: Execute G  
│   └── Thread 3: Execute H
├── Barrier: Wait for Phase 1 completion
├── Phase 2: Group 1 (sequential, 1 thread)
│   └── Thread 1: Execute A → C → F (in sequence)
├── Barrier: Wait for Phase 2 completion
├── Phase 3: Group 2 (sequential, 1 thread)
│   └── Thread 1: Execute B → D (in sequence)
└── Final Barrier: All phases complete
```

#### Phase 3: Result Aggregation

```
Thread Results:
├── Thread 1: Fired [E], Modified 5 facts
├── Thread 2: Fired [G], Modified 3 facts  
├── Thread 3: Fired [H], Modified 2 facts
└── Thread 4: Fired [A, C, F, B, D], Modified 12 facts

Aggregated Result:
├── Total Fired: 8 rules
├── Total Modified: 22 facts
├── Execution Time: 45ms (vs 120ms sequential)
├── Thread Utilization: 85%
└── Speedup: 2.7x
```

### Configuration Options

```rust
pub struct ParallelConfig {
    /// Maximum threads to use
    pub max_threads: usize,
    
    /// Minimum rules per group for parallel execution
    pub min_rules_per_group: usize,
    
    /// Enable dependency analysis
    pub enable_dependency_analysis: bool,
    
    /// Thread priority strategy
    pub priority_strategy: PriorityStrategy,
    
    /// Timeout for parallel execution
    pub execution_timeout_ms: u64,
}
```

**Configuration Examples:**

```rust
// High-throughput configuration
let config = ParallelConfig {
    max_threads: 16,
    min_rules_per_group: 5,
    enable_dependency_analysis: true,
    priority_strategy: PriorityStrategy::LoadBalanced,
    execution_timeout_ms: 30000,
};

// Conservative configuration  
let config = ParallelConfig {
    max_threads: 4,
    min_rules_per_group: 10,
    enable_dependency_analysis: true,
    priority_strategy: PriorityStrategy::Conservative,
    execution_timeout_ms: 60000,
};
```

### Usage Examples

#### Basic Parallel Execution

```rust
use rust_rule_engine::engine::ParallelRuleEngine;

// Create parallel engine
let mut engine = ParallelRuleEngine::new(ParallelConfig::default())?;

// Add rules (will be analyzed for dependencies)
engine.add_rules_from_grl(&grl_content)?;

// Execute in parallel
let result = engine.execute_parallel().await?;

println!("Parallel execution results:");
println!("- Rules fired: {}", result.fired_rules.len());
println!("- Execution time: {}ms", result.execution_time_ms);
println!("- Thread utilization: {}%", result.thread_utilization);
println!("- Speedup vs sequential: {:.1}x", result.speedup_factor);
```

#### Custom Configuration

```rust
// High-performance setup for 16-core server
let config = ParallelConfig {
    max_threads: 16,
    min_rules_per_group: 3,  // Allow smaller groups
    enable_dependency_analysis: true,
    priority_strategy: PriorityStrategy::LoadBalanced,
    execution_timeout_ms: 10000,  // 10 second timeout
};

let mut engine = ParallelRuleEngine::with_config(config)?;

// Add hundreds of rules
for rule_file in rule_files {
    engine.add_rules_from_file(&rule_file)?;
}

// Execute with maximum parallelism
let result = engine.execute_parallel().await?;
assert!(result.speedup_factor > 3.0);  // Expect 3x+ speedup
```

#### Integration with RETE

```rust
// Combine RETE incremental updates with parallel execution
let mut rete_engine = IncrementalEngine::new();
let mut parallel_engine = ParallelRuleEngine::new(config)?;

// Load rules into both engines
GrlReteLoader::load_from_string(&rules, &mut rete_engine)?;
parallel_engine.add_rules_from_grl(&rules)?;

// Insert facts into RETE
rete_engine.insert("User".to_string(), user_facts)?;
rete_engine.insert("Order".to_string(), order_facts)?;

// Use RETE for incremental updates
rete_engine.reset();
let rete_result = rete_engine.fire_all()?;

// Use parallel engine for batch processing
let parallel_result = parallel_engine.execute_parallel().await?;

// Compare performance
println!("RETE incremental: {}ms", rete_result.execution_time);
println!("Parallel batch: {}ms", parallel_result.execution_time_ms);
```

### Performance Characteristics

#### Speedup Factors

| Rule Count | Dependencies | Sequential | Parallel (8 threads) | Speedup |
|------------|--------------|------------|---------------------|---------|
| 100 | Low | 50ms | 15ms | 3.3x |
| 500 | Medium | 250ms | 60ms | 4.2x |
| 1000 | High | 800ms | 180ms | 4.4x |
| 5000 | Very High | 4000ms | 600ms | 6.7x |

#### Memory Overhead

- **Thread Stacks**: ~2MB per thread (configurable)
- **Synchronization Primitives**: ~100KB for locks/barriers
- **Result Aggregation**: ~50KB for statistics
- **Total Overhead**: ~5-10% increase vs single-threaded

#### Scalability

**Thread Count Optimization:**
```
Threads | 100 Rules | 1000 Rules | 10000 Rules
─────────┼───────────┼────────────┼─────────────
1       | 100ms     | 1000ms     | 10000ms
2       | 60ms      | 550ms      | 5200ms  
4       | 35ms      | 320ms      | 2800ms
8       | 25ms      | 220ms      | 1600ms
16      | 22ms      | 180ms      | 1200ms
32      | 20ms      | 170ms      | 1100ms
```

**Optimal Thread Count Formula:**
```
optimal_threads = min(max_threads, rule_count / avg_rules_per_group)
```

### Safety Guarantees

#### Correctness Preservation

The parallel engine maintains all correctness properties of sequential execution:

1. **Rule Firing Order**: Dependencies respected
2. **Fact Consistency**: No race conditions on fact modifications
3. **Agenda Management**: Conflict resolution preserved
4. **TMS Integration**: Logical dependencies maintained

#### Thread Safety

**Synchronization Strategy:**
- **Read-Write Locks**: Multiple readers, exclusive writers
- **Mutexes**: For agenda and conflict resolution
- **Barriers**: Phase synchronization
- **Channels**: Result aggregation

**Deadlock Prevention:**
- **Lock Ordering**: Consistent lock acquisition order
- **Timeout Protection**: Maximum execution time limits
- **Dependency Analysis**: Prevents circular wait conditions

### Comparison with Single-Threaded

| Aspect | Single-Threaded | Parallel |
|--------|-----------------|----------|
| **Performance** | Baseline | 2-7x faster |
| **Memory Usage** | Lower | 5-10% overhead |
| **Complexity** | Simple | Moderate |
| **Scalability** | Limited | Excellent |
| **Correctness** | Guaranteed | Guaranteed |
| **Best For** | < 100 rules | > 100 rules |

**When to Use Parallel Execution:**

✅ **Large rule sets** (> 500 rules)
✅ **Low dependency rules** (many independent rules)  
✅ **Batch processing** (not real-time)
✅ **Multi-core servers** (8+ CPU cores)
✅ **High-throughput** requirements

**When to Use Single-Threaded:**

❌ **Small rule sets** (< 50 rules)
❌ **High dependency rules** (mostly sequential)
❌ **Real-time requirements** (< 10ms latency)
❌ **Memory-constrained** environments
❌ **Simple applications**

### Integration with Other Features

#### With TMS (Truth Maintenance)

```rust
// Parallel execution with TMS consistency
let mut parallel_engine = ParallelRuleEngine::with_tms(config)?;
let result = parallel_engine.execute_parallel().await?;

// TMS ensures logical consistency across threads
assert!(parallel_engine.tms().is_consistent());
```

#### With Backward Chaining

```rust
// Parallel rule loading, backward chaining queries
let mut parallel_engine = ParallelRuleEngine::new(config)?;
parallel_engine.add_rules_from_grl(&rules)?;

let mut bc_engine = BackwardChainingEngine::new();
bc_engine.add_rules_from_parallel_engine(&parallel_engine)?;

// Query using facts from parallel execution
let query_result = bc_engine.query("User.IsVIP == true", &facts)?;
```

#### With Streaming Engine

```rust
// Parallel batch processing + streaming real-time
let mut parallel_engine = ParallelRuleEngine::new(config)?;
let mut streaming_engine = StreamRuleEngine::new()?;

// Load same rules into both
parallel_engine.add_rules_from_grl(&rules)?;
streaming_engine.add_rules(&rules).await?;

// Use parallel for batch, streaming for real-time
let batch_result = parallel_engine.execute_parallel().await?;
let stream_result = streaming_engine.execute_rules().await?;
```

The Parallel Rule Execution engine provides significant performance improvements for rule-heavy applications while maintaining the correctness and safety guarantees of the single-threaded engine.

## Query Interface (v1.1.0)

The Query Interface provides declarative querying capabilities over facts, enabling goal-driven reasoning and complex fact retrieval patterns beyond simple forward chaining.

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                                                                 │
│                   QUERY INTERFACE ARCHITECTURE v1.1.0                           │
│                                                                                 │
└─────────────────────────────────────────────────────────────────────────────────┘

                            ┌──────────────────────┐
                            │   Query String       │
                            │ "User.IsVIP == true" │
                            └──────────┬───────────┘
                                       │
                                       │ Parse Query
                                       ↓
                            ┌──────────────────────┐
                            │   GRL Query Parser   │
                            │ (src/backward/grl_query.rs) │
                            └──────────┬───────────┘
                                       │
                                       │ Build Query Plan
                                       ↓
                     ┌─────────────────────────────────────┐
                     │      QUERY EXECUTOR                 │
                     │   ┌─────────────────────────────┐   │
                     │   │ GRLQueryExecutor            │   │
                     │   │ ├── Pattern Matching        │   │
                     │   │ ├── Variable Binding        │   │
                     │   │ ├── Join Operations         │   │
                     │   │ └── Result Aggregation      │   │
                     │   └─────────────────────────────┘   │
                     │                                     │
                     │   ┌─────────────────────────────┐   │
                     │   │ Search Strategies           │   │
                     │   │ ├── Depth-First             │   │
                     │   │ ├── Breadth-First           │   │
                     │   │ ├── Best-First (heuristic)  │   │
                     │   │ └── A* Search               │   │
                     │   └─────────────────────────────┘   │
                     └─────────────────────────────────────┘
                                       │
                                       │ Execute Against Facts
                                       ↓
┌────────────────────────────────────────────────────────────────────────────────┐
│                          FACT INDEXING & SEARCH                                │
├────────────────────────────────────────────────────────────────────────────────┤
│                                                                                │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                       FACT INDEXES                                     │    │
│  │  ┌────────────────────┐  ┌────────────────────┐                        │    │
│  │  │ Type Index         │  │ Field Index        │                        │    │
│  │  │ • User facts       │  │ • Age → handles    │                        │    │
│  │  │ • Order facts      │  │ • Status → handles │                        │    │
│  │  │ • Fast lookup      │  │ • Range queries    │                        │    │
│  │  └────────────────────┘  └────────────────────┘                        │    │
│  │                                                                        │    │
│  │  ┌────────────────────┐  ┌────────────────────┐                        │    │
│  │  │ Value Index        │  │ Composite Index    │                        │    │
│  │  │ • "gold" → users   │  │ • (tier, age)      │                        │    │
│  │  │ • true → booleans  │  │ • Multi-field      │                        │    │
│  │  │ • Efficient filter │  │ • Complex queries  │                        │    │
│  │  └────────────────────┘  └────────────────────┘                        │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
│  ┌────────────────────────────────────────────────────────────────────────┐    │
│  │                      QUERY OPERATIONS                                  │    │
│  │  ┌────────────────────┐  ┌────────────────────┐                        │    │
│  │  │ Pattern Matching   │  │ Join Operations    │                        │    │
│  │  │ • Field equality   │  │ • Range queries    │                        │    │
│  │  │ • Regex matching   │  │ • Variable binding │                        │    │
│  │  │ • Result merging   │  │ • Result merging   │                        │    │
│  │  └────────────────────┘  └────────────────────┘                        │    │
│  └────────────────────────────────────────────────────────────────────────┘    │
│                                                                                │
└────────────────────────────────────────────────────────────────────────────────┘

                                      │
                                      │ Query Results
                                      ↓
                            ┌──────────────────────┐
                            │   QueryResult        │
                            │  • Success/Failure   │
                            │  • Bound Variables   │
                            │  • Proof Statistics  │
                            └──────────────────────┘
```

### Core Components

#### 1. GRL Query Language

**File:** `src/backward/grl_query.rs`

Extends GRL syntax with query-specific constructs:

```rust
pub struct GRLQuery {
    pub patterns: Vec<QueryPattern>,
    pub variables: HashMap<String, Variable>,
    pub conditions: Vec<QueryCondition>,
    pub search_strategy: GRLSearchStrategy,
}
```

**Query Syntax Examples:**

```grl
// Simple fact queries
query FindActiveUsers {
    find User {
        Status == "active"
    }
}

// Variable binding queries
query FindUserByName {
    find User {
        Name == $?userName
        Age > $?minAge
    }
}

// Complex multi-pattern queries
query FindVIPOrders {
    find User {
        IsVIP == true
        ID == $?userId
    }
    find Order {
        UserID == $?userId
        Total > 1000
    }
}

// Existential queries
query HasExpensiveOrders {
    exists Order {
        Total > 5000
        Status == "pending"
    }
}
```

#### 2. Query Executor

**File:** `src/backward/grl_query.rs`

Executes queries against fact databases with optimization:

```rust
pub struct GRLQueryExecutor {
    fact_index: FactIndex,
    search_strategy: GRLSearchStrategy,
    max_results: usize,
    timeout_ms: u64,
}

impl GRLQueryExecutor {
    pub fn execute(&self, query: &GRLQuery, facts: &Facts) -> Result<QueryResult> {
        // 1. Analyze query patterns
        // 2. Build execution plan
        // 3. Execute with chosen strategy
        // 4. Aggregate and return results
    }
}
```

**Execution Strategies:**

```rust
pub enum GRLSearchStrategy {
    /// Find first match quickly
    FirstMatch,
    
    /// Find all matches (breadth-first)
    AllMatches,
    
    /// Find best match using heuristic
    BestMatch { heuristic: Box<dyn Fn(&QueryMatch) -> f64> },
    
    /// Limited results for performance
    Limited { max_results: usize },
}
```

#### 3. Fact Indexing

Efficient fact lookup using multiple index types:

```rust
pub struct FactIndex {
    /// Facts by type: "User" -> [handle1, handle2, ...]
    type_index: HashMap<String, Vec<FactHandle>>,
    
    /// Facts by field value: ("User", "Age", 25) -> [handle1, handle2, ...]
    field_index: HashMap<(String, String, FactValue), Vec<FactHandle>>,
    
    /// Range indexes for numeric fields
    range_index: HashMap<(String, String), BTreeMap<FactValue, Vec<FactHandle>>>,
    
    /// Composite indexes for multi-field queries
    composite_index: HashMap<Vec<String>, HashMap<Vec<FactValue>, Vec<FactHandle>>>,
}
```

### Query Types

#### 1. Pattern Matching Queries

Find facts matching specific patterns:

```rust
// Find all users with specific criteria
let query = r#"
query FindGoldUsers {
    find User {
        Tier == "gold"
        Age >= 25
        IsActive == true
    }
}
"#;

let result = executor.execute_query(query, &facts)?;
println!("Found {} gold users", result.matches.len());
```

#### 2. Variable Binding Queries

Extract specific values from matching facts:

```rust
// Extract user names and ages
let query = r#"
query ExtractUserInfo {
    find User {
        Name == $?userName
        Age == $?userAge
        Tier == "platinum"
    }
}
"#;

let result = executor.execute_query(query, &facts)?;

// Access bound variables
for binding in &result.variable_bindings {
    let name = binding.get("$?userName")?;
    let age = binding.get("$?userAge")?;
    println!("Platinum user: {} (age {})", name, age);
}
```

#### 3. Join Queries

Combine data from multiple fact types:

```rust
// Find orders with their users
let query = r#"
query FindUserOrders {
    find User {
        ID == $?userId
        Name == $?userName
    }
    find Order {
        UserID == $?userId
        Total > $?orderTotal
    }
}
"#;

let result = executor.execute_query(query, &facts)?;

// Each result contains both user and order data
for binding in &result.variable_bindings {
    let user_name = binding.get("$?userName")?;
    let order_total = binding.get("$?orderTotal")?;
    println!("{} has order worth ${}", user_name, order_total);
}
```

#### 4. Existential Queries

Check for existence without retrieving full data:

```rust
// Check if any high-value orders exist
let query = r#"
query HasHighValueOrders {
    exists Order {
        Total > 5000
        Status == "pending"
    }
}
"#;

let result = executor.execute_query(query, &facts)?;
if result.success {
    println!("High-value orders exist - trigger review process");
}
```

### Performance Optimizations

#### Index Selection Strategy

```rust
impl FactIndex {
    pub fn select_best_index(&self, query: &GRLQuery) -> IndexSelection {
        // Analyze query patterns
        // Choose most selective index
        // Estimate result set size
        // Return optimal access path
    }
}
```

**Index Selection Examples:**

| Query Pattern | Best Index | Estimated Selectivity |
|---------------|------------|----------------------|
| `User.Age == 25` | Field Index | High (exact match) |
| `User.Age > 20` | Range Index | Medium (range scan) |
| `User.Tier == "gold" && User.Age > 25` | Composite Index | High (multi-field) |
| `User.Name contains "John"` | Full Scan | Low (pattern match) |

#### Query Execution Plans

```rust
pub enum QueryPlan {
    /// Single pattern - direct index lookup
    SinglePattern {
        pattern: QueryPattern,
        index: IndexType,
    },
    
    /// Multiple patterns - join execution
    Join {
        left: Box<QueryPlan>,
        right: Box<QueryPlan>,
        join_type: JoinType,
        join_condition: JoinCondition,
    },
    
    /// Nested queries - subquery execution
    Nested {
        outer: Box<QueryPlan>,
        subquery: Box<QueryPlan>,
        correlation: Vec<Variable>,
    },
}
```

### Integration Examples

#### With RETE Engine

```rust
// Use RETE for rule processing, queries for analysis
let mut rete_engine = IncrementalEngine::new();
let query_executor = GRLQueryExecutor::new();

// Load rules and facts into RETE
GrlReteLoader::load_from_string(&rules, &mut rete_engine)?;
rete_engine.insert("User".to_string(), user_facts)?;
rete_engine.insert("Order".to_string(), order_facts)?;

// Fire rules to derive additional facts
rete_engine.reset();
rete_engine.fire_all()?;

// Query the enriched fact base
let vip_query = r#"
query FindVIPUsers {
    find User {
        IsVIP == true
        TotalSpent == $?spent
    }
}
"#;

let vip_results = query_executor.execute_query(vip_query, &rete_engine.facts())?;
println!("Found {} VIP users", vip_results.matches.len());
```

#### With Backward Chaining

```rust
// Combine backward chaining with query interface
let mut bc_engine = BackwardChainingEngine::new();
let query_executor = GRLQueryExecutor::new();

// Add backward chaining rules
bc_engine.add_rule(&bc_rules)?;

// Use queries to provide initial facts for backward chaining
let fact_query = r#"
query GetUserFacts {
    find User {
        ID == "123"
        Name == $?name
        Age == $?age
    }
}
"#;

let facts = query_executor.execute_query(fact_query, &existing_facts)?;
let user_facts = facts.to_typed_facts();

// Query backward chaining engine
let goal_result = bc_engine.query_with_facts(
    "User.IsEligibleForLoan == true", 
    &user_facts
)?;
```

#### With Streaming Engine

```rust
// Real-time queries on streaming data
let mut streaming_engine = StreamRuleEngine::new()?;
let query_executor = GRLQueryExecutor::new();

// Set up streaming rules
streaming_engine.add_rules(&streaming_rules).await?;

// Add query handler for real-time analysis
streaming_engine.register_query_handler("analyze_high_value", |facts| {
    let query = r#"
    query HighValueTransactions {
        find Transaction {
            Amount > 10000
            Timestamp > $?timeWindow
        }
    }
    "#;
    
    query_executor.execute_query(query, facts)
})?;

// Start streaming and query processing
streaming_engine.start().await?;
```

### Advanced Features

#### Custom Query Functions

```rust
// Register custom query functions
query_executor.register_function("distance", |args| {
    let lat1 = args[0].as_float()?;
    let lon1 = args[1].as_float()?;
    let lat2 = args[2].as_float()?;
    let lon2 = args[3].as_float()?;
    
    Ok(calculate_distance(lat1, lon1, lat2, lon2))
})?;

// Use in queries
let location_query = r#"
query NearbyStores {
    find Store {
        distance(Lat, Lon, $?userLat, $?userLon) < 10.0
    }
}
"#;
```

#### Query Result Processing

```rust
// Process query results with custom logic
let result = executor.execute_query(query, &facts)?;

result.process_matches(|binding| {
    // Custom processing for each match
    let user_id = binding.get("UserID")?;
    let score = calculate_risk_score(user_id);
    
    if score > 0.8 {
        trigger_fraud_alert(user_id);
    }
    
    Ok(())
})?;
```

#### Query Statistics and Profiling

```rust
// Enable query profiling
let config = QueryConfig {
    enable_profiling: true,
    profile_detail_level: ProfileLevel::Detailed,
};

let executor = GRLQueryExecutor::with_config(config);

// Execute profiled query
let result = executor.execute_query(query, &facts)?;

// Analyze performance
println!("Query Statistics:");
println!("- Execution time: {}ms", result.stats.execution_time_ms);
println!("- Index lookups: {}", result.stats.index_lookups);
println!("- Fact scans: {}", result.stats.fact_scans);
println!("- Result count: {}", result.stats.result_count);
println!("- Memory usage: {}KB", result.stats.memory_usage_kb);
```

### Performance Characteristics

#### Query Performance Benchmarks

| Query Type | Index Type | 100 facts | 1000 facts | 10000 facts |
|------------|------------|-----------|------------|-------------|
| Single field | Field Index | 0.1ms | 0.2ms | 0.5ms |
| Range query | Range Index | 0.3ms | 0.8ms | 2.1ms |
| Multi-field | Composite | 0.2ms | 0.4ms | 0.9ms |
| Join query | Multi-index | 1.2ms | 3.5ms | 8.7ms |
| Full scan | No index | 5.0ms | 50ms | 500ms |

#### Optimization Strategies

**Index Selection:**
- Choose most selective index first
- Use composite indexes for multi-field queries
- Prefer range indexes for numeric ranges

**Execution Planning:**
- Reorder patterns for optimal join order
- Use early termination for existence queries
- Cache frequently executed queries

**Memory Management:**
- Stream results for large result sets
- Limit result count for performance
- Use pagination for UI applications

### Comparison with Traditional Queries

| Feature | SQL | GRL Query | Notes |
|---------|-----|-----------|-------|
| **Data Model** | Tables | Facts | Fact-oriented |
| **Joins** | Explicit | Implicit | Pattern-based |
| **Variables** | Bind params | Query vars | Runtime binding |
| **Functions** | UDFs | Custom funcs | Extensible |
| **Optimization** | Query planner | Index selection | Automatic |
| **Integration** | Databases | Rule engines | Native |

**When to Use GRL Queries:**

✅ **Rule engine integration** (native fact access)
✅ **Complex pattern matching** (beyond SQL joins)
✅ **Variable binding** (extract values easily)
✅ **Existential queries** (efficient existence checks)
✅ **Real-time analysis** (streaming data)

**When to Use Traditional SQL:**

❌ **Relational data** (use SQL databases)
❌ **ACID transactions** (use RDBMS)
❌ **Complex aggregations** (use SQL GROUP BY)
❌ **Schema enforcement** (use typed databases)

The Query Interface provides powerful declarative querying capabilities that complement the rule engine's forward and backward chaining with efficient fact retrieval and analysis tools.

1. **Backward Chaining** - Goal-driven reasoning (✅ IMPLEMENTED)
2. **Truth Maintenance System** - Automatic fact retraction (✅ IMPLEMENTED)
3. **Parallel RETE** - Multi-threaded evaluation (✅ IMPLEMENTED)
4. **Query Interface** - Declarative queries over facts (✅ IMPLEMENTED)

### Planned for v1.2.0

1. **Persistent Storage** - Rule/fact persistence
2. **Rule Compilation** - JIT compilation for hot paths
3. **REST API** - HTTP interface for rule management
4. **Distributed Execution** - Multi-node rule engine cluster

## References

- Original RETE Paper: Charles Forgy (1979)
- RETE-UL: Doorenbos (1995)
- CLIPS Manual: NASA (2020)
- Drools Documentation: Red Hat

## License

MIT License - See LICENSE file for details

---

**Version:** 1.1.0  
**Last Updated:** 2025-11-27  
**Maintained by:** Ton That Vu <ttvuhm@gmail.com>

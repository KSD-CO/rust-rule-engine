# Backward Chaining Quick Start Guide

> **Category:** Guides
> **Version:** 1.11.0+
> **Last Updated:** December 10, 2024
> **Estimated Time:** 10 minutes

Goal-driven inference with backward chaining - from zero to queries in 10 minutes!

---

## 🎯 What is Backward Chaining?

**Backward chaining** is goal-driven reasoning: Start with a question, work backwards to find if it's provable.

```
Question: "Is this customer VIP?"
    ↓
Search for rules that conclude VIP status
    ↓
Check conditions needed for those rules
    ↓
Use facts or recursively prove sub-goals
    ↓
Answer: "Yes, provable!" or "No, not provable"
```

**Use Cases:**
- Expert systems & diagnostics
- Decision support systems
- Query answering
- AI reasoning
- Complex eligibility checks

---

## 🚀 Quick Start (3 Steps)

### Step 1: Add Dependency

```toml
[dependencies.rust-rule-engine]
version = "1.11"
features = ["backward-chaining"]
```

### Step 2: Create Engine & Add Rules

```rust
use rust_rule_engine::backward::BackwardEngine;
use rust_rule_engine::{KnowledgeBase, Facts, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create knowledge base
    let mut kb = KnowledgeBase::new("demo");

    // Add rules using GRL syntax
    kb.add_rule_from_string(r#"
        rule "VIP Status" {
            when
                Customer.TotalSpent > 5000 &&
                Customer.YearsMember > 2
            then
                Customer.IsVIP = true;
        }
    "#)?;

    kb.add_rule_from_string(r#"
        rule "Adult Check" {
            when
                User.Age >= 18
            then
                User.IsAdult = true;
        }
    "#)?;

    // Create backward chaining engine
    let mut bc_engine = BackwardEngine::new(kb);

    Ok(())
}
```

### Step 3: Query & Get Results

```rust
// Set up facts
let mut facts = Facts::new();
facts.set("Customer.TotalSpent", Value::Number(6000.0));
facts.set("Customer.YearsMember", Value::Integer(3));

// Ask a question
let result = bc_engine.query("Customer.IsVIP == true", &mut facts)?;

// Check result
if result.provable {
    println!("✅ Customer IS VIP!");
    println!("Solutions found: {}", result.solutions.len());
} else {
    println!("❌ Customer is NOT VIP");
}
```

**Output:**
```
✅ Customer IS VIP!
Solutions found: 1
```

---

## ✨ New in v1.11.0

### 1. Nested Queries (Subqueries)

Ask complex questions with WHERE clauses:

```rust
use rust_rule_engine::backward::{GRLQueryParser, GRLQueryExecutor};

let query_str = r#"
query "Find High-Value Active Customers" {
    goal: qualified(?customer) WHERE
        (high_value(?customer) WHERE total_spent(?customer, ?amt) AND ?amt > 10000)
        AND active(?customer)
    enable-optimization: true
    max-depth: 20

    on-success: {
        Customer.Tier = "platinum";
        Print("Found qualified customer");
    }
}
"#;

let query = GRLQueryParser::parse(query_str)?;
let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts)?;
```

**Features:**
- Multi-level nesting
- Shared variables between queries
- OR/AND combinations
- NOT negation support

### 2. Query Optimization (10-100x Speedup!)

Enable automatic goal reordering for massive performance gains:

```rust
let query_str = r#"
query "Optimized Search" {
    goal: item(?x) AND expensive(?x) AND in_stock(?x) AND category(?x, "Premium")
    enable-optimization: true

    on-success: {
        Results.Add = x;
    }
}
"#;
```

**Without Optimization:**
```
item(?x)        → 10,000 items checked
expensive(?x)   → 3,000 remaining
in_stock(?x)    → 300 remaining
category(?x)    → 50 final results
Total: 13,350 evaluations
```

**With Optimization:**
```
category(?x)    → 500 items (most selective)
in_stock(?x)    → 50 remaining
expensive(?x)   → 30 remaining
item(?x)        → 30 final
Total: 610 evaluations (~22x faster!)
```

---

## 📚 Complete Example: Medical Diagnosis

```rust
use rust_rule_engine::backward::{BackwardEngine, GRLQueryParser, GRLQueryExecutor};
use rust_rule_engine::{KnowledgeBase, Facts, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create knowledge base with medical rules
    let mut kb = KnowledgeBase::new("medical");

    kb.add_rule_from_string(r#"
        rule "Flu Diagnosis" {
            when
                Patient.HasFever == true &&
                Patient.HasCough == true &&
                Patient.HasFatigue == true
            then
                Diagnosis.Disease = "Influenza";
                Diagnosis.Confidence = "high";
        }
    "#)?;

    kb.add_rule_from_string(r#"
        rule "Fever from High WBC" {
            when
                Patient.WhiteBloodCellCount > 11000
            then
                Patient.HasFever = true;
        }
    "#)?;

    kb.add_rule_from_string(r#"
        rule "Fatigue from Fever" {
            when
                Patient.HasFever == true &&
                Patient.DaysSick > 2
            then
                Patient.HasFatigue = true;
        }
    "#)?;

    // Create engine
    let mut bc_engine = BackwardEngine::new(kb);

    // Set patient facts
    let mut facts = Facts::new();
    facts.set("Patient.WhiteBloodCellCount", Value::Number(12000.0));
    facts.set("Patient.HasCough", Value::Boolean(true));
    facts.set("Patient.DaysSick", Value::Integer(3));

    // Query: Does patient have flu?
    let result = bc_engine.query(
        "Diagnosis.Disease == \"Influenza\"",
        &mut facts
    )?;

    if result.provable {
        println!("✅ Diagnosis: Flu confirmed!");
        println!("Reasoning chain:");
        println!("  1. High WBC (12000) → HasFever = true");
        println!("  2. HasFever + DaysSick > 2 → HasFatigue = true");
        println!("  3. HasFever + HasCough + HasFatigue → Influenza");
    } else {
        println!("❌ Cannot confirm flu diagnosis");
    }

    Ok(())
}
```

**Output:**
```
✅ Diagnosis: Flu confirmed!
Reasoning chain:
  1. High WBC (12000) → HasFever = true
  2. HasFever + DaysSick > 2 → HasFatigue = true
  3. HasFever + HasCough + HasFatigue → Influenza
```

---

## 🎯 Key Features

### Aggregation Functions (v1.7.0+)

```rust
let query = r#"
query "Total Sales" {
    goal: sum(?amount) WHERE sale(?id, ?amount) AND ?amount > 100
    on-success: {
        Report.TotalSales = result;
    }
}
"#;
```

**Supported:** COUNT, SUM, AVG, MIN, MAX, FIRST, LAST

### Negation (v1.8.0+)

```rust
let result = bc_engine.query(
    "NOT Customer.IsBanned == true",
    &mut facts
)?;
// Succeeds if customer is NOT banned
```

### Disjunction - OR (v1.10.0+)

```rust
let query = r#"
query "Priority Customer" {
    goal: (Customer.IsVIP == true OR Customer.TotalSpent > 10000)
          AND Customer.IsActive == true
}
"#;
```

### Explanation System (v1.9.0+)

Get proof trees showing reasoning:

```rust
use rust_rule_engine::backward::explanation::ProofTree;

let result = bc_engine.query("goal", &mut facts)?;

// Generate explanation
let tree = result.explanation;
tree.print();  // Console output

// Export formats
let json = tree.to_json()?;
let markdown = tree.to_markdown();
let html = tree.to_html();
```

---

## 🔧 Search Strategies

Choose the right strategy for your use case:

### Depth-First (Default)
```rust
let mut bc_engine = BackwardEngine::new(kb);
bc_engine.set_strategy(SearchStrategy::DepthFirst);
```

**Best for:** Most queries, memory-efficient

### Breadth-First
```rust
bc_engine.set_strategy(SearchStrategy::BreadthFirst);
```

**Best for:** Finding shortest proof path

### Iterative Deepening
```rust
bc_engine.set_strategy(SearchStrategy::IterativeDeepening);
```

**Best for:** Unknown depth queries, optimal solutions

---

## 📝 GRL Query Syntax

Write queries in GRL files for better organization:

**queries/eligibility.grl:**
```grl
query "VIP Eligibility" {
    goal: eligible(?customer) WHERE
        (vip(?customer) OR (premium(?customer) AND loyalty(?customer, ?years) AND ?years > 3))
        AND active(?customer)
        AND NOT suspended(?customer)

    strategy: depth-first
    max-depth: 20
    max-solutions: 10
    enable-optimization: true
    enable-memoization: true

    on-success: {
        Customer.Eligible = true;
        Customer.Benefits = "full_access";
        Print("Customer is eligible");
    }

    on-failure: {
        Customer.Eligible = false;
        Print("Customer not eligible");
    }

    on-missing: {
        Print("Missing required customer data");
        Request.AdditionalInfo = true;
    }
}
```

**Load and execute:**
```rust
use std::fs;

let grl_content = fs::read_to_string("queries/eligibility.grl")?;
let query = GRLQueryParser::parse(&grl_content)?;
let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts)?;
```

---

## 🚀 Performance Tips

### 1. Enable Optimization for Multi-Goal Queries
```grl
enable-optimization: true  // 10-100x speedup!
```

### 2. Use Memoization
```grl
enable-memoization: true  // Cache proven goals
```

### 3. Set Appropriate Depth Limits
```grl
max-depth: 20  // Prevent infinite loops
```

### 4. Limit Solutions When Appropriate
```grl
max-solutions: 10  // Stop after finding 10
```

### 5. Choose Right Strategy
- **Depth-first**: Most queries (default)
- **Breadth-first**: Shortest path needed
- **Iterative**: Unknown complexity

---

## 🔄 Combine with Forward Chaining

Use both for hybrid reasoning:

```rust
use rust_rule_engine::Engine;

// Forward chaining for reactive rules
let mut fc_engine = Engine::new();
fc_engine.add_rule_from_string(r#"
    rule "Update Status" {
        when Order.Total > 1000
        then Order.Status = "high_value";
    }
"#)?;

// Backward chaining for queries
let mut bc_engine = BackwardEngine::new(kb);

// Run forward chaining first
fc_engine.run(&mut facts)?;

// Then query with backward chaining
let result = bc_engine.query("Order.Status == \"high_value\"", &mut facts)?;
```

See [Integration Guide](guides/BACKWARD_CHAINING_RETE_INTEGRATION.md) for details.

---

## 📖 Next Steps

### Learn More
- **[GRL Query Syntax](api-reference/GRL_QUERY_SYNTAX.md)** - Complete query language reference
- **[API Reference](api-reference/API_REFERENCE.md)** - Full API documentation
- **[Troubleshooting](guides/TROUBLESHOOTING.md)** - Common issues and solutions

### Examples
- **[Nested Queries Demo](../examples/09-backward-chaining/nested_query_demo.rs)**
- **[Query Optimization Demo](../examples/09-backward-chaining/optimizer_demo.rs)**
- **[GRL File Demo](../examples/09-backward-chaining/nested_grl_file_demo.rs)**

### Run Examples
```bash
# Nested queries
cargo run --example nested_query_demo --features backward-chaining

# Query optimization
cargo run --example optimizer_demo --features backward-chaining

# GRL integration
cargo run --example grl_optimizer_demo --features backward-chaining
```

---

## 💡 Common Patterns

### Pattern 1: Eligibility Check
```rust
query "Check Eligibility" {
    goal: eligible(?person) WHERE
        age_requirement(?person) AND
        income_requirement(?person) AND
        NOT disqualified(?person)
}
```

### Pattern 2: Hierarchical Relationships
```rust
query "Find Ancestors" {
    goal: ancestor(?a, ?d) WHERE
        parent(?a, ?p) AND
        (parent(?p, ?d) OR ancestor(?p, ?d))
}
```

### Pattern 3: Complex Business Rules
```rust
query "Approve Transaction" {
    goal: approved(?txn) WHERE
        (low_risk(?txn) OR (medium_risk(?txn) AND manual_review(?txn)))
        AND NOT fraud_detected(?txn)
        AND within_limits(?txn)
}
```

---

## 🐛 Troubleshooting

### Query Not Provable?
1. Check facts are set correctly
2. Verify rule conditions match fact names
3. Check for typos in field names
4. Ensure rules are added to knowledge base

### Performance Issues?
1. Enable optimization: `enable-optimization: true`
2. Reduce max-depth if too high
3. Use memoization: `enable-memoization: true`
4. Profile with statistics

### Need Help?
- 📖 [Troubleshooting Guide](guides/TROUBLESHOOTING.md)
- 💬 [GitHub Discussions](https://github.com/KSD-CO/rust-rule-engine/discussions)
- 🐛 [Report Issue](https://github.com/KSD-CO/rust-rule-engine/issues)

---

## Navigation

📚 **[Documentation Home](README.md)** | 📖 **[Getting Started](getting-started/QUICK_START.md)** | 🔍 **[GRL Query Syntax](api-reference/GRL_QUERY_SYNTAX.md)**

**Related:**
- [Forward Chaining](core-features/FORWARD_CHAINING.md)
- [RETE Integration](guides/BACKWARD_CHAINING_RETE_INTEGRATION.md)
- [API Reference](api-reference/API_REFERENCE.md)

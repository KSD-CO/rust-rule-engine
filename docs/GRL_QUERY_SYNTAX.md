# GRL Query Syntax for Backward Chaining

> **Feature:** `backward-chaining`  
> **Version:** 0.20.0+

GRL Query Syntax extends the Grule Rule Language to support backward chaining queries.

---

## Basic Query Syntax

### Simple Query
```grl
query "CheckVIPStatus" {
    goal: User.IsVIP == true
}
```

### Query with Strategy
```grl
query "DiagnoseFlu" {
    goal: Diagnosis.Disease == "Influenza"
    strategy: depth-first
    max-depth: 10
}
```

### Query with Actions
```grl
query "ApprovalCheck" {
    goal: Application.Approved == true
    on-success: LogMessage("Application approved")
    on-failure: LogMessage("Application rejected")
}
```

---

## Query Attributes

### `goal` (Required)
The target pattern to prove.

```grl
query "Example" {
    goal: Order.Total > 1000
}
```

Supports:
- Equality: `==`, `!=`
- Comparison: `>`, `>=`, `<`, `<=`
- Logical: `&&`, `||`
- Complex expressions

### `strategy` (Optional)
Search strategy to use.

**Options:**
- `depth-first` (default) - Prolog-style, goes deep first
- `breadth-first` - Level-by-level exploration
- `iterative` - Iterative deepening

```grl
query "FastSearch" {
    goal: Result.Found == true
    strategy: breadth-first
}
```

### `max-depth` (Optional)
Maximum depth for goal search (prevents infinite loops).

```grl
query "SafeQuery" {
    goal: Complex.Chain == true
    max-depth: 5  // Stop after 5 levels
}
```

**Default:** 10

### `max-solutions` (Optional)
Maximum number of solutions to find.

```grl
query "FindAll" {
    goal: Product.InStock == true
    max-solutions: 100  // Find up to 100 products
}
```

**Default:** 1 (stop after first solution)

### `enable-memoization` (Optional)
Enable caching of proven goals.

```grl
query "Cached" {
    goal: Expensive.Computation == true
    enable-memoization: true
}
```

**Default:** true

---

## Action Handlers

### `on-success`
Execute when query is provable.

```grl
query "VIPCheck" {
    goal: User.IsVIP == true
    on-success: {
        LogMessage("VIP status confirmed");
        User.DiscountRate = 0.2;
    }
}
```

### `on-failure`
Execute when query is not provable.

```grl
query "CreditCheck" {
    goal: Applicant.CreditScore > 700
    on-failure: {
        LogMessage("Credit check failed");
        Application.Status = "Rejected";
    }
}
```

### `on-missing`
Handle missing facts.

```grl
query "DataCheck" {
    goal: Document.Verified == true
    on-missing: {
        LogMessage("Missing required data");
        RequestAdditionalInfo();
    }
}
```

---

## Advanced Features

### Parameterized Queries

```grl
query "CheckThreshold" {
    goal: Value >= $threshold
    params: {
        threshold: Number
    }
}
```

Usage:
```rust
let result = bc_engine.query_with_params(
    "CheckThreshold",
    &facts,
    hashmap!{ "threshold" => Value::Number(100.0) }
)?;
```

### Query Templates

```grl
query-template "DiagnoseDisease" {
    goal: Diagnosis.Disease == $disease_name
    strategy: depth-first
    max-depth: $search_depth
}

// Instantiate
query "CheckFlu" from "DiagnoseDisease" {
    disease_name: "Influenza"
    search_depth: 8
}
```

### Conditional Queries

```grl
query "ConditionalCheck" {
    goal: Result.Valid == true
    when: Environment.Mode == "Production"
    strategy: depth-first
}
```

---

## Complete Examples

### Medical Diagnosis Query

```grl
rule "DiagnoseFlu" {
    when
        Patient.HasFever == true && 
        Patient.HasCough == true && 
        Patient.HasFatigue == true
    then
        Diagnosis.Disease = "Influenza";
}

rule "FeverFromInfection" {
    when
        Patient.WhiteBloodCellCount > 11000
    then
        Patient.HasFever = true;
}

// Query to check diagnosis
query "CheckFluDiagnosis" {
    goal: Diagnosis.Disease == "Influenza"
    strategy: depth-first
    max-depth: 10
    enable-memoization: true
    
    on-success: {
        LogMessage("Flu diagnosis confirmed");
        Treatment.Recommended = "Rest and fluids";
    }
    
    on-failure: {
        LogMessage("Flu not confirmed");
        Action.Next = "Consider other diagnoses";
    }
}
```

### Business Logic Query

```grl
query "CheckVIPEligibility" {
    goal: Customer.IsVIP == true
    strategy: breadth-first
    max-depth: 5
    
    on-success: {
        Customer.DiscountRate = 0.2;
        Customer.ShippingFree = true;
        LogMessage("VIP benefits applied");
    }
    
    on-failure: {
        LogMessage("Customer not eligible for VIP");
        Action.Recommend = "Suggest VIP upgrade";
    }
    
    on-missing: {
        LogMessage("Missing customer data");
        Request.AdditionalInfo = ["PurchaseHistory", "LoyaltyPoints"];
    }
}
```

### Detective Investigation Query

```grl
query "CheckSuspectGuilty" {
    goal: Investigation.Guilty == true
    strategy: depth-first
    max-depth: 15
    
    on-success: {
        Investigation.Verdict = "Guilty";
        Action.RecommendedAction = "Issue arrest warrant";
        LogProofTrace();
    }
    
    on-failure: {
        Investigation.Verdict = "Insufficient Evidence";
        Investigation.MissingEvidence = GetMissingFacts();
        Action.RecommendedAction = "Continue investigation";
    }
}
```

---

## Query Files

Save queries in separate `.grlq` files:

**queries/medical_queries.grlq:**
```grl
query "DiagnoseFlu" {
    goal: Diagnosis.Disease == "Influenza"
    strategy: depth-first
}

query "CheckDiabetes" {
    goal: Diagnosis.Disease == "Type 2 Diabetes"
    strategy: breadth-first
}

query "AssessRisk" {
    goal: Risk.Level == "High"
    max-depth: 8
}
```

Load and execute:
```rust
let queries = GRLParser::parse_queries_from_file("queries/medical_queries.grlq")?;
let bc_engine = BackwardEngine::new(kb);

for query in queries {
    let result = bc_engine.execute_query(&query, &facts)?;
    println!("{}: {}", query.name, result.provable);
}
```

---

## API Integration

### Parse Query from String

```rust
use rust_rule_engine::backward::GRLQuery;

let query_str = r#"
query "Example" {
    goal: User.IsVIP == true
    strategy: depth-first
}
"#;

let query = GRLQuery::parse(query_str)?;
```

### Execute Query

```rust
let mut bc_engine = BackwardEngine::new(kb);
let result = bc_engine.execute_query(&query, &facts)?;

if result.provable {
    println!("Query succeeded!");
    // Execute on-success actions
    query.execute_success_actions(&mut facts)?;
}
```

### Query with Callbacks

```rust
let config = QueryConfig {
    on_goal_explored: |goal| println!("Exploring: {}", goal),
    on_rule_evaluated: |rule| println!("Evaluating: {}", rule),
    on_proof_found: |trace| println!("Proof: {:?}", trace),
};

let result = bc_engine.query_with_config(&query, &facts, config)?;
```

---

## Best Practices

### 1. Use Descriptive Query Names
```grl
// ❌ Bad
query "Q1" { ... }

// ✅ Good
query "CheckCustomerVIPStatus" { ... }
```

### 2. Set Reasonable Depth Limits
```grl
// For simple queries
query "Simple" {
    goal: X == true
    max-depth: 5
}

// For complex chains
query "Complex" {
    goal: Y == true
    max-depth: 20
}
```

### 3. Enable Memoization for Expensive Queries
```grl
query "ExpensiveComputation" {
    goal: Result.Computed == true
    enable-memoization: true  // Cache results
}
```

### 4. Provide Meaningful Actions
```grl
query "Diagnosis" {
    goal: Disease.Identified == true
    
    on-success: {
        // Clear next steps
        LogMessage("Diagnosis complete");
        Treatment.Recommended = GetTreatmentPlan();
    }
    
    on-failure: {
        // Actionable feedback
        LogMessage("Diagnosis inconclusive");
        Tests.Additional = GetRequiredTests();
    }
}
```

---

## Migration from Code

### Before (Code-based)
```rust
let mut bc_engine = BackwardEngine::new(kb);
let result = bc_engine.query("User.IsVIP == true", &facts)?;

if result.provable {
    println!("VIP confirmed");
    facts.set("User.DiscountRate", Value::Number(0.2));
}
```

### After (GRL-based)
```grl
query "CheckVIPStatus" {
    goal: User.IsVIP == true
    on-success: {
        LogMessage("VIP confirmed");
        User.DiscountRate = 0.2;
    }
}
```

```rust
let queries = GRLParser::parse_queries_from_file("queries.grlq")?;
let mut bc_engine = BackwardEngine::new(kb);
bc_engine.execute_queries(&queries, &facts)?;
```

---

## See Also

- [GRL Syntax Reference](GRL_SYNTAX.md)
- [Backward Chaining Design](BACKWARD_CHAINING_DESIGN.md)
- [API Reference](API_REFERENCE.md)

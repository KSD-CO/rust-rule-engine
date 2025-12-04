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
- Negation: `NOT` (v1.8.0+)
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

## Negation (NOT Keyword)

**Version:** 1.8.0+

The `NOT` keyword enables negated goals with closed-world assumption - a goal succeeds if it CANNOT be proven.

### Basic Negation

```grl
query "NotBannedUsers" {
    goal: NOT User.IsBanned == true
    on-success: {
        User.Allowed = true;
        LogMessage("User is not banned");
    }
}
```

### Closed-World Assumption

If a fact is not explicitly stated (or derivable), it's assumed FALSE:

```grl
// User has no "is_banned" field → assumed NOT banned
query "CheckAccess" {
    goal: NOT User.IsBanned == true
    on-success: {
        // This succeeds because User.IsBanned is not in facts
        AllowAccess();
    }
}
```

### Real-World Examples

#### E-commerce Auto-Approval
```grl
query "AutoApprovedOrders" {
    goal: NOT Order.RequiresApproval == true
    on-success: {
        Order.Status = "auto_approved";
        ProcessOrder();
    }
    on-failure: {
        // Order DOES require approval
        SendToManualReview();
    }
}
```

#### User Access Control
```grl
query "AllowAccess" {
    goal: NOT User.IsSuspended == true
    on-success: {
        // User is NOT suspended → allow access
        GrantAccess();
    }
    on-failure: {
        // User IS suspended → deny access
        DenyAccess();
    }
}
```

#### Inventory Availability
```grl
query "AvailableItems" {
    goal: NOT Item.Reserved == true
    on-success: {
        Item.Available = true;
        AddToSalesInventory();
    }
}
```

### Combining NOT with Other Conditions

**Note:** Currently, NOT must be at the beginning of the goal. For complex conditions, use separate queries:

```grl
// Check if user is Active AND NOT Banned
query "EligibleUser" {
    goal: User.IsActive == true
    on-success: {
        // Then check NOT banned separately
        CheckNotBanned();
    }
}

query "CheckNotBanned" {
    goal: NOT User.IsBanned == true
    on-success: {
        // Both conditions met
        GrantFullAccess();
    }
}
```

### Negation Semantics

1. **Explicit FALSE:** `User.IsBanned = false` → `NOT User.IsBanned == true` succeeds
2. **Missing Field:** No `User.IsBanned` field → `NOT User.IsBanned == true` succeeds (closed-world)
3. **Explicit TRUE:** `User.IsBanned = true` → `NOT User.IsBanned == true` fails

### Use Cases

- **Access Control:** Check users are NOT banned/suspended
- **Inventory:** Find items NOT sold/reserved
- **Approval Workflows:** Process orders that do NOT require approval
- **Feature Flags:** Check features are NOT disabled
- **Membership:** Find users who are NOT VIP/premium

---

## Aggregation Functions

**Version:** 1.7.0+

Aggregation functions enable powerful data analysis by computing metrics across multiple facts that match a pattern.

### Supported Functions

- **COUNT** - Count matching facts
- **SUM** - Sum numeric values
- **AVG** - Calculate average
- **MIN** - Find minimum value
- **MAX** - Find maximum value
- **FIRST** - Get first matching value
- **LAST** - Get last matching value

### Basic Aggregation Syntax

```grl
query "TotalSalary" {
    goal: sum(?salary) WHERE employee(?name, ?salary)
    on-success: {
        Payroll.Total = result;
        LogMessage("Total payroll calculated");
    }
}
```

### Query Syntax

**Format:**
```
aggregate_function(?variable) WHERE pattern [AND filter_conditions]
```

**Components:**
- `aggregate_function` - One of: count, sum, avg, min, max, first, last
- `?variable` - Variable to aggregate (e.g., `?salary`, `?amount`)
- `pattern` - Fact pattern to match (e.g., `employee(?name, ?salary)`)
- `filter_conditions` - Optional AND conditions to filter facts

### Examples

#### Count Employees

```grl
query "CountEmployees" {
    goal: count(?x) WHERE employee(?x)
    on-success: {
        Stats.EmployeeCount = result;
        LogMessage("Employee count: " + result);
    }
}
```

```rust
// Facts: employee(alice), employee(bob), employee(charlie)
// Result: Integer(3)
```

#### Sum High Salaries

```grl
query "HighEarnerPayroll" {
    goal: sum(?salary) WHERE salary(?name, ?salary) AND ?salary > 80000
    on-success: {
        Payroll.HighEarners = result;
        LogMessage("High earner total: $" + result);
    }
}
```

```rust
// Facts: salary(alice, 90000), salary(bob, 75000), salary(charlie, 95000)
// Filter: Only salaries > 80000
// Result: Float(185000.0) - alice (90000) + charlie (95000)
```

#### Average Product Price

```grl
query "AveragePrice" {
    goal: avg(?price) WHERE product(?name, ?price)
    on-success: {
        Analytics.AvgPrice = result;
    }
}
```

```rust
// Facts: product(laptop, 999.99), product(mouse, 29.99), product(keyboard, 79.99)
// Result: Float(369.99) - (999.99 + 29.99 + 79.99) / 3
```

#### Min/Max Scores

```grl
query "ScoreRange" {
    goal: min(?score) WHERE student(?name, ?score)
    on-success: {
        Stats.MinScore = result;
    }
}

query "MaxScore" {
    goal: max(?score) WHERE student(?name, ?score)
    on-success: {
        Stats.MaxScore = result;
    }
}
```

```rust
// Facts: student(alice, 85), student(bob, 92), student(charlie, 78)
// Min Result: Integer(78)
// Max Result: Integer(92)
```

#### First/Last Values

```grl
query "FirstEmployee" {
    goal: first(?name) WHERE employee(?name)
    on-success: {
        Report.FirstHire = result;
    }
}

query "LastEmployee" {
    goal: last(?name) WHERE employee(?name)
    on-success: {
        Report.LatestHire = result;
    }
}
```

### Real-World Use Cases

#### E-commerce Analytics

```grl
// Total revenue from completed orders
query "TotalRevenue" {
    goal: sum(?amount) WHERE purchase(?item, ?amount) AND ?amount > 0
    on-success: {
        Sales.TotalRevenue = result;
        LogMessage("Revenue: $" + result);
    }
}

// Average order value
query "AverageOrderValue" {
    goal: avg(?total) WHERE order(?id, ?total)
    on-success: {
        Analytics.AOV = result;
        if result > 100 {
            Marketing.Strategy = "premium";
        }
    }
}

// Count high-value customers
query "VIPCustomers" {
    goal: count(?customer) WHERE purchase(?customer, ?amount) AND ?amount > 1000
    on-success: {
        Stats.VIPCount = result;
    }
}
```

#### Salary & Payroll Management

```grl
// Total payroll
query "TotalPayroll" {
    goal: sum(?salary) WHERE salary(?name, ?salary)
    on-success: {
        Payroll.Total = result;
        Budget.Allocated = result;
    }
}

// Department payroll
query "EngineeringPayroll" {
    goal: sum(?salary) WHERE employee(?name, "engineering", ?salary)
    on-success: {
        Department.Engineering.Budget = result;
    }
}

// Salary statistics
query "SalaryRange" {
    goal: max(?salary) WHERE salary(?name, ?salary)
    on-success: {
        Stats.MaxSalary = result;
    }
}

query "MinSalary" {
    goal: min(?salary) WHERE salary(?name, ?salary)
    on-success: {
        Stats.MinSalary = result;
    }
}
```

#### Inventory Management

```grl
// Total inventory value
query "InventoryValue" {
    goal: sum(?value) WHERE item(?name, ?quantity, ?price, ?value)
    on-success: {
        Inventory.TotalValue = result;
        LogMessage("Inventory worth: $" + result);
    }
}

// Count low-stock items
query "LowStockCount" {
    goal: count(?item) WHERE item(?item, ?qty) AND ?qty < 10
    on-success: {
        Alerts.LowStockItems = result;
        if result > 5 {
            SendAlert("Low stock alert!");
        }
    }
}

// Average item price
query "AvgItemPrice" {
    goal: avg(?price) WHERE item(?name, ?price)
    on-success: {
        Pricing.Average = result;
    }
}
```

### Filter Conditions

Use AND to filter facts before aggregation:

```grl
// Sum only completed orders
query "CompletedOrdersTotal" {
    goal: sum(?amount) WHERE order(?id, ?amount, ?status) AND ?status == "completed"
    on-success: {
        Revenue.Completed = result;
    }
}

// Count active premium customers
query "PremiumActiveCount" {
    goal: count(?id) WHERE customer(?id, ?tier, ?active) AND ?tier == "premium" AND ?active == true
    on-success: {
        Stats.PremiumActive = result;
    }
}

// Average salary for senior engineers
query "SeniorEngAvgSalary" {
    goal: avg(?salary) WHERE employee(?name, ?level, ?dept, ?salary) AND ?level == "senior" AND ?dept == "engineering"
    on-success: {
        Benchmarks.SeniorEngSalary = result;
    }
}
```

### Type Safety

Aggregation functions handle type conversions automatically:

```rust
// Integer values → Integer result
count(?x) WHERE employee(?x)  // Integer(5)
sum(?qty) WHERE item(?name, ?qty)  // If all Integer → Integer

// Mixed Integer/Float → Float result
sum(?amount) WHERE purchase(?item, ?amount)  // If any Float → Float

// Averages always return Float
avg(?score) WHERE student(?name, ?score)  // Always Float
```

**Type Rules:**
- **COUNT**: Always returns `Integer`
- **SUM**: Returns `Integer` if all values are Integer, otherwise `Float`
- **AVG**: Always returns `Float`
- **MIN/MAX**: Returns same type as values (Integer or Float)
- **FIRST/LAST**: Returns value's original type (can be String, Integer, Float, Boolean)

### Error Handling

```grl
query "SafeAggregation" {
    goal: sum(?amount) WHERE order(?id, ?amount)
    on-success: {
        // Aggregation succeeded
        Results.Total = result;
    }
    on-failure: {
        // No matching facts found
        LogMessage("No orders to aggregate");
        Results.Total = 0;
    }
}
```

**Common Scenarios:**
- **No matching facts**: Query fails (returns `provable: false`)
- **Empty result set**: Query fails
- **Non-numeric values for numeric aggregations**: Values ignored or converted
- **Mixed types**: Automatic type promotion (Integer → Float)

### Programmatic Usage

```rust
use rust_rule_engine::backward::BackwardEngine;

let mut engine = BackwardEngine::new(kb);

// Execute aggregation query
let result = engine.query_aggregate(
    "sum(?salary) WHERE salary(?name, ?salary) AND ?salary > 80000",
    &mut facts
)?;

// Get numeric result
match result {
    Value::Integer(n) => println!("Total: {}", n),
    Value::Float(f) => println!("Total: {:.2}", f),
    _ => println!("Unexpected result type"),
}
```

### Performance Considerations

1. **Filter Early**: Use AND conditions to reduce facts before aggregation
   ```grl
   // ✅ Good - filter first
   sum(?amt) WHERE order(?id, ?amt) AND ?amt > 100

   // ❌ Less efficient - aggregates all, filters later
   sum(?amt) WHERE order(?id, ?amt)
   ```

2. **Index Variables**: Ensure pattern variables are indexed
   ```grl
   // ✅ Good - uses indexed field
   sum(?salary) WHERE employee(?name, ?salary)

   // ❌ Slow - no index
   sum(?value) WHERE complex_pattern(?a, ?b, ?c, ?value)
   ```

3. **Limit Result Size**: Use max-solutions for large datasets
   ```grl
   query "TopProducts" {
       goal: sum(?sales) WHERE product(?name, ?sales)
       max-solutions: 1000  // Limit processing
   }
   ```

### Limitations (v1.7.0)

**Current Limitations:**
1. **Single variable aggregation**: Cannot aggregate multiple variables simultaneously
2. **No GROUP BY**: Cannot group by categories (coming in future release)
3. **Filter placement**: AND conditions must be in WHERE clause
4. **No nested aggregations**: Cannot nest aggregate functions

**Future Enhancements:**
- GROUP BY support: `sum(?salary) WHERE employee(?dept, ?name, ?salary) GROUP BY ?dept`
- Multiple aggregations: `count(?x), sum(?y) WHERE pattern`
- HAVING clause: `sum(?amt) HAVING result > 1000`
- Nested aggregations: `avg(sum(?x)) WHERE pattern`

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

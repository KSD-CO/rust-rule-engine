# GRL Query Syntax for Backward Chaining

> **Category:** API Reference
> **Feature:** `backward-chaining`
> **Version:** 1.11.0+
> **Last Updated:** December 10, 2024

GRL Query Syntax extends the Grule Rule Language to support backward chaining queries with:
- ✅ Aggregation (COUNT, SUM, AVG, MIN, MAX, FIRST, LAST)
- ✅ Negation (NOT keyword with closed-world assumption)
- ✅ Explanation (Proof trees with JSON/MD/HTML export)
- ✅ Disjunction (OR patterns with parentheses)
- ✅ **Nested Queries** (Subqueries with WHERE clauses) - **NEW in v1.11.0**
- ✅ **Query Optimization** (Automatic goal reordering) - **NEW in v1.11.0**

---

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Basic Query Syntax](#basic-query-syntax)
3. [Query Attributes](#query-attributes)
4. [Action Handlers](#action-handlers)
5. [Negation (NOT Keyword)](#negation-not-keyword)
6. [Aggregation Functions](#aggregation-functions)
7. [Nested Queries (Subqueries)](#nested-queries-subqueries) ⭐ **NEW v1.11.0**
8. [Query Optimization](#query-optimization) ⭐ **NEW v1.11.0**
9. [Advanced Features](#advanced-features)
10. [Complete Examples](#complete-examples)
11. [Query Files](#query-files)
12. [API Integration](#api-integration)
13. [Best Practices](#best-practices)
14. [Migration from Code](#migration-from-code)
15. [Explanation System](#explanation-system-v190)
16. [See Also](#see-also)

---

## Quick Reference

### What's New in v1.11.0

| Feature | Syntax | Use Case |
|---------|--------|----------|
| **Nested Queries** | `goal: pattern WHERE subquery` | Multi-level relationships (grandparent, eligibility) |
| **Query Optimization** | `enable-optimization: true` | 10-100x speedup on multi-goal queries |
| **Combined Features** | `goal: x WHERE (a OR b) AND NOT c` | Complex business logic with all operators |

**Example - All v1.11.0 Features:**
```grl
query "ComplexEligibility" {
    goal: eligible(?customer) WHERE
        (vip(?customer) OR (premium(?customer) AND loyalty(?customer, ?years) AND ?years > 3))
        AND active(?customer)
        AND NOT suspended(?customer)
    enable-optimization: true
    enable-memoization: true
    max-depth: 20
    on-success: {
        Customer.Eligible = true;
    }
}
```

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
- Disjunction: `OR` with parentheses (v1.10.0+)
- Nested queries: `WHERE` subqueries (v1.11.0+)
- Aggregation: `count`, `sum`, `avg`, `min`, `max`, `first`, `last` (v1.7.0+)
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

### `enable-optimization` (Optional)

**Version:** 1.11.0+

Enable automatic query optimization with goal reordering.

```grl
query "OptimizedQuery" {
    goal: item(?x) AND expensive(?x) AND in_stock(?x)
    enable-optimization: true
    max-solutions: 10
}
```

**Default:** false

**Benefits:**
- 10-100x speedup on multi-goal queries
- Automatic selectivity-based goal reordering
- No code changes required
- Especially effective with 3+ goals

**See:** [Query Optimization](#query-optimization) section for detailed usage

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

## Nested Queries (Subqueries)

**Version:** 1.11.0+

Nested queries enable complex multi-level reasoning by embedding subqueries within the main query. This allows you to express hierarchical relationships, conditional logic, and multi-step inference patterns.

### Basic Nested Query Syntax

**Format:**
```
goal_pattern WHERE subquery_pattern [AND additional_conditions]
```

**Components:**
- `goal_pattern` - Main goal to prove
- `WHERE` - Introduces nested subquery
- `subquery_pattern` - Pattern that must be proven first
- Variable sharing between main and subquery automatically detected

### Simple Nested Example

```grl
query "FindGrandparents" {
    goal: grandparent(?gp, ?gc) WHERE parent(?gp, ?p) AND parent(?p, ?gc)
    strategy: depth-first
    max-depth: 15
    enable-optimization: true

    on-success: {
        Print("Found grandparent relationship");
        Relationship.Type = "grandparent";
    }
}
```

**Explanation:**
- Main goal: `grandparent(?gp, ?gc)` - Find grandparent-grandchild pairs
- Subquery: `parent(?gp, ?p) AND parent(?p, ?gc)` - A grandparent is someone whose child is also a parent
- Shared variables: `?gp` (grandparent), `?gc` (grandchild), `?p` (intermediate parent)

### Variable Sharing

Variables prefixed with `?` are automatically shared between nested queries:

```grl
query "EligibleCustomers" {
    goal: eligible(?customer) WHERE (vip(?customer) OR premium(?customer)) AND active(?customer)
    strategy: breadth-first
    max-depth: 20

    on-success: {
        Customer.Eligible = true;
        Print("Customer is eligible");
    }
}
```

**Variable Flow:**
1. `?customer` appears in main goal `eligible(?customer)`
2. `?customer` is used in subquery `vip(?customer)` and `premium(?customer)`
3. Engine automatically binds values from subquery to main query

### Complex Multi-Level Nesting

You can nest queries multiple levels deep:

```grl
query "HighValueActive" {
    goal: qualified(?c) WHERE
        (high_value(?c) WHERE total_spent(?c, ?amt) AND ?amt > 10000)
        AND active(?c)
    strategy: depth-first
    max-solutions: 5
    enable-optimization: true

    on-success: {
        Print("Found qualified high-value active customer");
        Customer.Tier = "platinum";
    }
}
```

**Execution Flow:**
1. Innermost: `total_spent(?c, ?amt) AND ?amt > 10000` - Find customers who spent >$10k
2. Middle: `high_value(?c)` - Prove customer is high-value using above condition
3. Outer: Check `active(?c)` - Customer must also be active
4. Final: `qualified(?c)` - Customer meets all criteria

### Combining OR with Nested Queries

```grl
query "PriorityCustomers" {
    goal: priority(?customer) WHERE
        (vip(?customer) OR (premium(?customer) AND loyalty_years(?customer, ?years) AND ?years > 3))
        AND active(?customer)
    max-depth: 20
    enable-optimization: true

    on-success: {
        Customer.Priority = "high";
        Customer.SupportTier = "premium";
    }
}
```

**Logic:**
- VIP customers automatically get priority
- OR premium customers with >3 years loyalty
- AND they must be active

### Negation in Nested Queries

```grl
query "AvailableNotSold" {
    goal: available(?item) WHERE
        item(?item) AND
        NOT sold(?item) AND
        in_stock(?item)
    strategy: depth-first
    enable-optimization: true

    on-success: {
        Print("Item is available for purchase");
        Item.Status = "available";
    }
}
```

**Conditions:**
1. `item(?item)` - Is a valid item
2. `NOT sold(?item)` - Has NOT been sold
3. `in_stock(?item)` - Is currently in stock

### Real-World Examples

#### E-commerce: Customer Eligibility

```grl
query "FreeShippingEligible" {
    goal: free_shipping(?customer) WHERE
        (vip(?customer) OR (order_total(?customer, ?total) AND ?total > 50))
        AND shipping_address(?customer, ?addr)
        AND valid_address(?addr)
    max-depth: 15
    enable-optimization: true

    on-success: {
        Order.ShippingCost = 0;
        Order.ShippingType = "free_standard";
        Print("Free shipping applied");
    }

    on-failure: {
        Order.ShippingCost = 9.99;
        Print("Standard shipping: $9.99");
    }
}
```

#### HR: Promotion Eligibility

```grl
query "PromotionEligible" {
    goal: eligible_for_promotion(?employee) WHERE
        (tenure(?employee, ?years) AND ?years >= 2)
        AND (performance(?employee, ?rating) AND ?rating >= 4.0)
        AND NOT under_review(?employee)
        AND manager_approval(?employee, ?manager)
    max-depth: 20
    enable-optimization: true

    on-success: {
        Employee.EligibleForPromotion = true;
        HR.ReviewQueue.Add = employee_id;
        Print("Employee eligible for promotion review");
    }
}
```

#### Finance: Loan Approval

```grl
query "LoanApproval" {
    goal: approve_loan(?applicant, ?amount) WHERE
        (credit_score(?applicant, ?score) AND ?score >= 650)
        AND (income(?applicant, ?income) AND ?income >= ?amount * 0.3)
        AND (debt_ratio(?applicant, ?ratio) AND ?ratio < 0.43)
        AND NOT bankruptcy(?applicant)
    max-depth: 15
    enable-optimization: true

    on-success: {
        Loan.Status = "approved";
        Loan.InterestRate = calculate_rate(credit_score);
        Print("Loan approved");
    }

    on-failure: {
        Loan.Status = "rejected";
        Print("Loan application rejected");
    }
}
```

#### Healthcare: Treatment Authorization

```grl
query "AuthorizeTreatment" {
    goal: authorized(?patient, ?treatment) WHERE
        (diagnosis(?patient, ?condition) AND requires_treatment(?condition, ?treatment))
        AND (insurance_coverage(?patient, ?treatment) OR emergency(?patient))
        AND NOT contraindication(?patient, ?treatment)
    max-depth: 20
    enable-optimization: true

    on-success: {
        Treatment.Authorized = true;
        Treatment.StartDate = today();
        Print("Treatment authorized");
    }
}
```

### Performance Considerations

1. **Order Matters**: Put most selective conditions first
   ```grl
   // ✅ Good - check rare condition first
   goal: result(?x) WHERE rare_condition(?x) AND common_condition(?x)

   // ❌ Less efficient - checks common condition first
   goal: result(?x) WHERE common_condition(?x) AND rare_condition(?x)
   ```

2. **Use Optimization**: Enable query optimization for automatic reordering
   ```grl
   query "Optimized" {
       goal: result(?x) WHERE condition_a(?x) AND condition_b(?x) AND condition_c(?x)
       enable-optimization: true  // Automatically reorders for best performance
   }
   ```

3. **Limit Depth**: Set appropriate max-depth for nested queries
   ```grl
   query "DeepNesting" {
       goal: complex(?x) WHERE nested(?x) WHERE deeply_nested(?x)
       max-depth: 25  // Increase for deeper nesting
   }
   ```

### Programmatic Usage

```rust
use rust_rule_engine::backward::{NestedQueryParser, NestedQueryEvaluator};

// Parse nested query
let query_str = "grandparent(?x, ?z) WHERE parent(?x, ?y) AND parent(?y, ?z)";
let query = NestedQueryParser::parse(query_str)?;

// Evaluate
let mut evaluator = NestedQueryEvaluator::new();
let result = evaluator.evaluate(&query, &mut engine, &mut facts)?;

// Check results
println!("Provable: {}", result.provable);
println!("Solutions: {:?}", result.solutions);
println!("Stats: goals={}, rules={}",
    result.stats.goals_explored,
    result.stats.rules_evaluated
);
```

### Limitations (v1.11.0)

**Current Limitations:**
1. Variables must be bound before use in conditions (no free variables in comparisons)
2. Aggregation within nested queries not yet supported
3. Maximum practical nesting depth: ~5 levels

**Future Enhancements:**
- Aggregation in subqueries: `sum(?amt) WHERE (transaction(?id, ?amt) WHERE date(?id, ?d))`
- Correlated subqueries with outer scope references
- Performance optimizations for deep nesting

---

## Query Optimization

**Version:** 1.11.0+

The query optimizer automatically reorders goals to minimize the number of evaluations needed, providing 10-100x speedup on multi-goal queries.

### Enabling Optimization

Add `enable-optimization: true` to your query:

```grl
query "OptimizedQuery" {
    goal: item(?x) AND expensive(?x) AND in_stock(?x)
    enable-optimization: true
    strategy: depth-first
}
```

### How Optimization Works

The optimizer uses **selectivity estimation** to reorder goals from most selective (fewest matches) to least selective:

**Without Optimization:**
```
item(?x)        → 1000 candidates
expensive(?x)   → 900 candidates (from 1000)
in_stock(?x)    → 270 candidates (from 900)
Total evaluations: 1000 + 900 + 270 = 2170
```

**With Optimization:**
```
in_stock(?x)    → 10 candidates
expensive(?x)   → 8 candidates (from 10)
item(?x)        → 8 candidates (from 8)
Total evaluations: 10 + 8 + 8 = 26
Result: ~83x faster! 🚀
```

### Selectivity Estimation

The optimizer estimates selectivity using heuristics:

| Pattern Type | Estimated Selectivity | Reason |
|-------------|----------------------|---------|
| `?x == constant` | 0.05 (5%) | Equality is highly selective |
| `?x > constant` | 0.3 (30%) | Range queries moderately selective |
| `?x < constant` | 0.3 (30%) | Range queries moderately selective |
| `NOT pattern(?x)` | 0.2 (20%) | Negation is fairly selective |
| `rare_predicate(?x)` | 0.1 (10%) | Uncommon predicates |
| `common_predicate(?x)` | 0.7 (70%) | Common predicates |
| `item(?x)` | 0.9 (90%) | Very general patterns |

### Optimization Examples

#### Example 1: E-commerce Product Search

```grl
query "FindProducts" {
    goal: product(?item) AND category(?item, "Electronics") AND price(?item, ?p) AND ?p < 100
    enable-optimization: true
    max-solutions: 50

    on-success: {
        Results.Add = item;
    }
}
```

**Optimization Result:**
```
Original order:
  product(?item)             [90% selectivity] → 10,000 items
  category(?item, "Electronics") [30%]        → 3,000 items
  price(?item, ?p)          [70%]            → 2,100 items
  ?p < 100                  [30%]            → 630 items

Optimized order:
  category(?item, "Electronics") [30%]        → 3,000 items
  ?p < 100                  [30%]            → 900 items
  price(?item, ?p)          [70%]            → 630 items
  product(?item)            [90%]            → 630 items

Result: 67% fewer evaluations
```

#### Example 2: Customer Eligibility

```grl
query "EligibleForDiscount" {
    goal: customer(?c) AND active(?c) AND total_purchases(?c, ?total) AND ?total > 1000 AND NOT banned(?c)
    enable-optimization: true
    enable-memoization: true

    on-success: {
        Customer.DiscountEligible = true;
        Customer.DiscountRate = 0.15;
    }
}
```

**Optimization Result:**
```
Original order: customer → active → total_purchases → ?total > 1000 → NOT banned
Optimized order: NOT banned → ?total > 1000 → active → total_purchases → customer

~95% reduction in evaluations
```

#### Example 3: Multi-Goal Inventory Query

```grl
query "AvailableHighValueItems" {
    goal: item(?x) AND in_stock(?x) AND price(?x, ?p) AND ?p > 500 AND category(?x, "Premium")
    enable-optimization: true
    max-depth: 15

    on-success: {
        Inventory.HighValueItems.Add = x;
        Alert.LowStock = check_threshold(x);
    }
}
```

**Optimization Strategy:**
1. **Most selective first**: `category(?x, "Premium")` - Only premium items
2. **Then price filter**: `?p > 500` - High-value items
3. **Then stock check**: `in_stock(?x)` - Available now
4. **Finally general**: `item(?x)` and `price(?x, ?p)` - Basic info

### Custom Selectivity Hints

For programmatic usage, you can override selectivity estimates:

```rust
use rust_rule_engine::backward::QueryOptimizer;

let mut optimizer = QueryOptimizer::new();

// Set custom selectivity values (0.0 = most selective, 1.0 = least selective)
optimizer.set_selectivity("in_stock(?x)".to_string(), 0.01);  // 1% in stock
optimizer.set_selectivity("expensive(?x)".to_string(), 0.15); // 15% expensive
optimizer.set_selectivity("item(?x)".to_string(), 0.95);      // 95% are items

// Optimize goals
let optimized = optimizer.optimize_goals(goals);
```

### Optimization Configuration

```rust
use rust_rule_engine::backward::OptimizerConfig;

let config = OptimizerConfig {
    enable_reordering: true,
    enable_index_selection: true,
    enable_memoization: true,
    selectivity_threshold: 0.5,  // Only reorder if selectivity < 50%
};

let optimizer = QueryOptimizer::with_config(config);
```

### Optimization Statistics

Track optimization effectiveness:

```rust
let stats = optimizer.get_stats();

println!("Goals reordered: {}", stats.goals_reordered);
println!("Estimated speedup: {}x", stats.estimated_speedup);
println!("Actual evaluations: {}", stats.actual_evaluations);
println!("Without optimization: {}", stats.estimated_without_optimization);
```

### Real-World Performance Gains

#### Case Study 1: Medical Diagnosis
```grl
query "DiagnoseCondition" {
    goal: patient(?p) AND symptom(?p, "fever") AND symptom(?p, "cough") AND test_result(?p, ?t) AND ?t == "positive"
    enable-optimization: true
}
```

**Results:**
- Before: 5,000 patients → 1,200 with fever → 300 with fever+cough → 50 with positive test
- After: 50 with positive test → 45 with cough → 12 with fever+cough (recheck patients)
- **Performance**: 92% reduction in evaluations

#### Case Study 2: Financial Fraud Detection
```grl
query "FraudulentTransaction" {
    goal: transaction(?t) AND amount(?t, ?a) AND ?a > 10000 AND suspicious_pattern(?t) AND NOT verified(?t)
    enable-optimization: true
}
```

**Results:**
- Before: 1M transactions checked
- After: 2K unverified → 500 suspicious → 50 high-amount
- **Performance**: 99.995% reduction, ~10,000x faster

#### Case Study 3: Inventory Management
```grl
query "RestockNeeded" {
    goal: item(?i) AND in_stock(?i) AND quantity(?i, ?q) AND ?q < 10 AND high_demand(?i)
    enable-optimization: true
}
```

**Results:**
- Before: 100K items → 30K in stock → 5K low quantity → 500 high demand
- After: 2K high demand → 200 low quantity → 50 in stock
- **Performance**: 98% reduction

### Best Practices

#### 1. Always Enable for Multi-Goal Queries
```grl
// ✅ Good - optimization for 3+ goals
query "Complex" {
    goal: a(?x) AND b(?x) AND c(?x) AND d(?x)
    enable-optimization: true
}

// ⚠️  Unnecessary - single goal
query "Simple" {
    goal: single_condition(?x)
    enable-optimization: true  // No benefit, but no harm
}
```

#### 2. Combine with Memoization
```grl
query "HighPerformance" {
    goal: complex(?x) AND nested(?x) AND deep(?x)
    enable-optimization: true   // Reorder goals
    enable-memoization: true    // Cache results
}
```

#### 3. Profile Before Optimizing
```rust
// Test without optimization first
let result_baseline = engine.query(query_str, &mut facts)?;
println!("Baseline: {} evaluations", result_baseline.stats.rules_evaluated);

// Then enable optimization
query.enable_optimization = true;
let result_optimized = engine.query(query_str, &mut facts)?;
println!("Optimized: {} evaluations", result_optimized.stats.rules_evaluated);
println!("Speedup: {}x", result_baseline.stats.rules_evaluated / result_optimized.stats.rules_evaluated);
```

#### 4. Use Appropriate Strategies
```grl
// Depth-first + optimization = best for most queries
query "Recommended" {
    goal: complex_pattern(?x)
    strategy: depth-first
    enable-optimization: true
}

// Breadth-first for finding all solutions quickly
query "FindAll" {
    goal: pattern(?x)
    strategy: breadth-first
    enable-optimization: true
    max-solutions: 100
}
```

### When Optimization Helps Most

**High Impact:**
- Queries with 4+ goals
- Goals with vastly different selectivity (some rare, some common)
- Large fact databases (1000+ facts)
- Complex nested queries

**Medium Impact:**
- Queries with 2-3 goals
- Moderate selectivity differences
- Medium fact databases (100-1000 facts)

**Low Impact:**
- Single-goal queries
- All goals have similar selectivity
- Small fact databases (<100 facts)
- Already well-ordered goals

### Limitations (v1.11.0)

**Current Limitations:**
1. Selectivity estimates are heuristic-based (not statistics-based)
2. Cannot account for correlations between goals
3. Optimization overhead for very simple queries

**Future Enhancements:**
- Statistical selectivity from actual data
- Cost-based optimization
- Adaptive optimization based on execution history
- Index selection hints

### Programmatic Usage Example

```rust
use rust_rule_engine::backward::{BackwardEngine, GRLQueryParser, GRLQueryExecutor};

// Parse query with optimization
let query_str = r#"
query "Optimized" {
    goal: item(?x) AND expensive(?x) AND in_stock(?x)
    enable-optimization: true
    max-solutions: 10
}
"#;

let query = GRLQueryParser::parse(query_str)?;
let mut engine = BackwardEngine::new(kb);

// Execute with optimization
let result = GRLQueryExecutor::execute(&query, &mut engine, &mut facts)?;

// Check performance
println!("Provable: {}", result.provable);
println!("Solutions: {}", result.solutions.len());
println!("Goals explored: {}", result.stats.goals_explored);
println!("Rules evaluated: {}", result.stats.rules_evaluated);
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

## Explanation System (v1.9.0+)

### Proof Tree Generation

The explanation system captures the reasoning process in a hierarchical proof tree structure.

```rust
use rust_rule_engine::backward::*;

// Create proof tree manually
let mut root = ProofNode::rule(
    "loan_approved == true".to_string(),
    "loan_approval_rule".to_string(),
    0,
);

let credit_node = ProofNode::fact("credit_score = 750".to_string(), 1);
root.add_child(credit_node);

let tree = ProofTree::new(root, "Check loan approval".to_string());

// Print to console
tree.print();
```

**Output:**
```
Query: Check loan approval
Result: ✓ Proven

Proof Tree:
================================================================================
✓ loan_approved == true [Rule: loan_approval_rule]
  ✓ credit_score = 750 [FACT]
================================================================================
```

### ProofNode Types

1. **Fact** - Goal proven by existing facts
2. **Rule** - Goal proven by rule application
3. **Negation** - Negated goals (NOT operator)
4. **Failed** - Goals that could not be proven

```rust
// Create different node types
let fact_node = ProofNode::fact("user.age = 25".to_string(), 1);
let rule_node = ProofNode::rule("user.is_adult == true".to_string(), "age_check".to_string(), 0);
let negation_node = ProofNode::negation("NOT user.is_banned == true".to_string(), 1, true);
```

### Export Formats

#### JSON Export
```rust
let json = tree.to_json()?;
std::fs::write("proof.json", json)?;
```

**Output:**
```json
{
  "root": {
    "goal": "loan_approved == true",
    "rule_name": "loan_approval_rule",
    "proven": true,
    "node_type": "Rule",
    "children": [...]
  },
  "success": true,
  "stats": {
    "goals_explored": 7,
    "rules_evaluated": 4,
    "facts_checked": 3,
    "max_depth": 3,
    "total_nodes": 7
  }
}
```

#### Markdown Export
```rust
let markdown = tree.to_markdown();
std::fs::write("proof.md", markdown)?;
```

**Output:**
```markdown
# Proof Explanation

**Query:** `loan_approved == true`
**Result:** ✓ Proven

## Proof Tree
* ✓ `loan_approved == true` **[Rule: loan_approval_rule]**
  * ✓ `credit_score = 750` *[FACT]*

## Statistics
- **Goals explored:** 7
- **Rules evaluated:** 4
- **Facts checked:** 3
- **Max depth:** 3
- **Total nodes:** 7
```

#### HTML Export
```rust
let html = tree.to_html();
std::fs::write("proof.html", html)?;
```

Generates an interactive HTML page with:
- CSS styling
- Color-coded success/failure indicators
- Hierarchical tree visualization
- Statistics summary

### Full Explanation with Steps

```rust
let explanation = Explanation::new("Is loan approved?".to_string(), tree);
explanation.print();
```

**Output:**
```
================================================================================
EXPLANATION
================================================================================

Query: Is loan approved?
Result: ✓ Proven

Query 'loan_approved == true' was successfully proven using 4 rules and 3 facts.

Step-by-Step Reasoning:
--------------------------------------------------------------------------------
Step 1: loan_approved == true
  Rule: loan_approval_rule
  Condition: loan_approved == true
  Result: Success
Step 2: credit_score = 750 [FACT]
  Result: Success
...
================================================================================
```

### ExplanationBuilder (Future Integration)

The `ExplanationBuilder` tracks query execution in real-time:

```rust
let mut builder = ExplanationBuilder::new();
builder.enable();

// During query execution (future integration):
// builder.start_goal(&goal);
// builder.goal_proven_by_fact(&goal, &bindings);
// builder.goal_proven_by_rule(&goal, "rule_name", &bindings);
// builder.finish_goal();

// Build final proof tree
let tree = builder.build("query string".to_string());
```

### Use Cases

1. **Debugging** - Understand why queries succeed or fail
2. **Auditing** - Generate compliance reports showing decision logic
3. **Transparency** - Explain AI decisions to end users
4. **Education** - Teach logical reasoning and rule-based systems
5. **Documentation** - Auto-generate examples from actual queries

### Example Demo

Run the complete explanation demo:

```bash
cargo run --features backward-chaining --example explanation_demo
```

Or with Make:

```bash
make explanation_demo
```

The demo includes:
1. Simple proof tree with basic facts
2. Complex multi-level reasoning (loan approval)
3. Negation in reasoning (access control)
4. Export to JSON, Markdown, and HTML

---

## See Also

### Documentation
- [GRL Syntax Reference](GRL_SYNTAX.md)
- [Backward Chaining Design](BACKWARD_CHAINING_DESIGN.md)
- [API Reference](API_REFERENCE.md)

### Example Demos (v1.11.0)
- [Nested Queries Demo](../examples/09-backward-chaining/nested_query_demo.rs) - Nested query examples
- [Nested GRL File Demo](../examples/09-backward-chaining/nested_grl_file_demo.rs) - Parse and execute .grl files
- [Optimizer Demo](../examples/09-backward-chaining/optimizer_demo.rs) - Query optimization showcase
- [GRL Optimizer Demo](../examples/09-backward-chaining/grl_optimizer_demo.rs) - GRL + optimization
- [Explanation Demo](../examples/09-backward-chaining/explanation_demo.rs) - Proof tree generation
- [Aggregation Demo](../examples/09-backward-chaining/aggregation_demo.rs) - COUNT, SUM, AVG, etc.

### Example GRL Files
- [Nested Queries](../examples/rules/09-backward-chaining/nested_queries.grl) - Nested query examples

### Running Examples
```bash
# Nested queries
cargo run --example nested_query_demo --features backward-chaining
cargo run --example nested_grl_file_demo --features backward-chaining

# Query optimization
cargo run --example optimizer_demo --features backward-chaining
cargo run --example grl_optimizer_demo --features backward-chaining

# Other features
cargo run --example explanation_demo --features backward-chaining
cargo run --example aggregation_demo --features backward-chaining
```

Or with Make:
```bash
make nested_query_demo
make optimizer_demo
make explanation_demo
make aggregation_demo
```

---

## Navigation

◀️ **Previous: [GRL Syntax](../core-features/GRL_SYNTAX.md)** | 📚 **[Documentation Home](../README.md)** | ▶️ **Next: [API Reference](API_REFERENCE.md)**

**Related:**
- [Backward Chaining Quick Start](../BACKWARD_CHAINING_QUICK_START.md)
- [Backward Chaining Integration](../guides/BACKWARD_CHAINING_RETE_INTEGRATION.md)
- [Troubleshooting](../guides/TROUBLESHOOTING.md)

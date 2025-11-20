# GRL Syntax Reference

Complete reference for Grule Rule Language (GRL) syntax supported by rust-rule-engine v0.17.1.

---

## Table of Contents
1. [Rule Structure](#rule-structure)
2. [Rule Attributes](#rule-attributes)
3. [Conditions](#conditions)
4. [Actions](#actions)
5. [Advanced Features (v0.17.x)](#advanced-features)
6. [Built-in Functions](#built-in-functions)
7. [Best Practices](#best-practices)
8. [Common Patterns](#common-patterns)

---

## Rule Structure

```grl
rule "RuleName" [attributes] {
    when
        <conditions>
    then
        <actions>
}
```

### Basic Example
```grl
rule "SimpleDiscount" {
    when
        Order.amount > 1000
    then
        Order.discount = 0.10;
}
```

---

## Rule Attributes

### Salience (Priority)
Controls execution order. Higher salience = higher priority (executes first).

```grl
rule "HighPriority" salience 100 {
    when User.tier == "platinum"
    then User.discount = 0.20;
}

rule "NormalPriority" salience 50 {
    when User.tier == "gold"
    then User.discount = 0.15;
}
```

**Default**: 0 (if not specified)

### No-Loop (v0.17.1)
Prevents infinite loops when rule modifies facts that triggered it.

```grl
rule "UpdateCounter" no-loop true {
    when
        Counter.value < 100
    then
        Counter.value = Counter.value + 1;  // Won't re-trigger
}
```

**Important**: Always use `no-loop true` for rules that modify their trigger conditions.

### Agenda Groups
Organize rules into execution phases for workflow control.

```grl
rule "ValidateOrder" agenda-group "validation" {
    when Order.validated == false
    then Order.validated = true;
}

rule "ProcessOrder" agenda-group "processing" {
    when Order.validated == true
    then Order.status = "processed";
}
```

**Usage**: Set focus to control which group executes.

### Activation Groups
Only one rule in group fires (highest salience wins).

```grl
rule "PlatinumDiscount" activation-group "discounts" salience 30 {
    when Customer.tier == "platinum"
    then Order.discount = 0.20;
}

rule "GoldDiscount" activation-group "discounts" salience 20 {
    when Customer.tier == "gold"
    then Order.discount = 0.15;
}
```

**Use Case**: Mutually exclusive rules (pricing tiers, shipping methods).

### Lock-on-Active
Prevents rule from firing again while its agenda group is active.

```grl
rule "OncePerActivation" lock-on-active true {
    when Order.needsValidation == true
    then Order.needsValidation = false;
}
```

### Date Effective/Expires
Time-bound rules (optional feature).

```grl
rule "HolidaySale" 
    date-effective "2025-12-01" 
    date-expires "2025-12-31" 
    salience 10 {
    when Order.amount > 100
    then Order.discount = 0.20;
}
```

---

## Conditions

### Comparison Operators
```grl
when
    age > 18              // Greater than
    age >= 18             // Greater than or equal
    age < 65              // Less than
    age <= 65             // Less than or equal
    status == "active"    // Equal
    status != "banned"    // Not equal
```

### Logical Operators
```grl
when
    age > 18 && status == "active"      // AND
    tier == "gold" || tier == "platinum" // OR
    !(status == "banned")               // NOT
    (A && B) || (C && D)                // Grouped expressions
```

### Arithmetic Expressions (v0.17.1) ⭐ NEW
Direct arithmetic in conditions without pre-calculation.

```grl
rule "ModuloCheck" {
    when
        User.Age % 3 == 0        // Modulo operator
    then
        User.divisibleBy3 = true;
}

rule "PriceDoubleCheck" {
    when
        Product.Price * 2 > User.Budget  // Multiplication
    then
        Product.affordable = false;
}

rule "ComplexMath" {
    when
        (Order.total - Order.discount) * 1.1 > 1000  // Combined operations
    then
        Order.needsApproval = true;
}
```

**Supported Operators**: `+`, `-`, `*`, `/`, `%` (modulo)

### Variable References (v0.17.1) ⭐ NEW
Compare fact values dynamically (variable-to-variable).

```grl
rule "AboveThreshold" {
    when
        Facts.L1 > Facts.L1Min  // Dynamic comparison
    then
        Facts.Approved = true;
}

rule "SetQuantity" {
    when
        shortage < moq && is_active == true
    then
        order_qty = moq;  // Variable assignment
}
```

**Usage**: Use `Facts.` prefix for variable references in RETE engine.

### String Operations
```grl
when
    name.contains("John")
    email.startsWith("admin")
    email.endsWith("@example.com")
    code.matches("ABC*")     // Wildcard pattern (if supported by plugin)
```

### Array/Multifield Operations (v0.17.0)
CLIPS-style collection pattern matching.

```grl
when
    Order.Items contains "laptop"      // Contains check
    Order.Items count > 5              // Count elements
    Order.Tags first == "priority"     // First element
    Order.Tags last == "verified"      // Last element
    Basket.Items empty                 // Check if empty
    Cart.Products not_empty            // Check not empty
```

**Supported Operations**:
- `contains <value>`: Check if value exists
- `count`: Get array length
- `first`/`last`: Get first/last element
- `index <n>`: Get element at position
- `slice <start> <end>`: Extract subarray
- `empty`/`not_empty`: Check if array is empty
- `collect as $?var`: Bind all values to variable

### Nested Field Access
```grl
when
    Customer.address.city == "New York"
    Order.items[0].price > 100
    User.profile.settings.notifications == true
```

---

## Advanced Features

### Test CE (Custom Expressions) (v0.17.1)
Execute arbitrary boolean expressions for complex logic.

```grl
rule "ComplexTest" {
    when
        test(User.Age % 3 == 0 && Product.Price * 2 > 100)
    then
        Order.specialOffer = true;
}
```

**Use Case**: When standard patterns don't suffice.

### EXISTS Pattern
True if at least one fact matches the condition.

```grl
rule "HasPendingOrders" {
    when
        exists(Order.status == "pending")
    then
        Alert.hasPending = true;
}
```

### NOT EXISTS / NOT Pattern
True if no facts match the condition.

```grl
rule "NoFailedPayments" {
    when
        !exists(Payment.status == "failed")
    then
        Order.paymentOk = true;
}

// Alternative syntax
rule "NoComplaints" {
    when
        NOT Complaint.status == "open"
    then
        Customer.goodStanding = true;
}
```

### FORALL Pattern
True if ALL facts of a type match the condition.

```grl
rule "AllItemsValidated" {
    when
        forall(Item.validated == true)
    then
        Order.readyToShip = true;
}
```

### Accumulate Functions (v0.17.0)
Aggregations and computations over collections.

```grl
rule "BulkDiscount" {
    when
        sum(Order.Items.Price) > 1000
    then
        Order.discount = 0.15;
}

rule "AverageScore" {
    when
        avg(Review.rating) > 4.5
    then
        Product.featured = true;
}

rule "CountCheck" {
    when
        count(Order.Items) > 10
    then
        Order.bulkOrder = true;
}
```

**Supported Functions**:
- `sum(field)`: Total sum
- `avg(field)`: Average value
- `min(field)`/`max(field)`: Min/max value
- `count(field)`: Count items

### Complex Pattern Combinations
```grl
rule "ComplexEligibility" {
    when
        Customer.tier == "gold" &&
        exists(Order.amount > 1000) &&
        !exists(Complaint.status == "open") &&
        forall(Payment.status == "completed")
    then
        Customer.priority = "high";
        Customer.autoApprove = true;
}
```

---

## Actions

### Simple Assignment
```grl
then
    Order.discount = 0.15;
    Order.status = "approved";
    User.lastLogin = "2025-11-20";
```

### Arithmetic Operations
```grl
then
    Order.total = Order.subtotal * (1 - Order.discount);
    Counter.value = Counter.value + 1;
    Product.finalPrice = Product.price * 0.9;
```

### String Concatenation
```grl
then
    User.fullName = User.firstName + " " + User.lastName;
    Log.message = "Order " + Order.id + " processed at " + Now();
```

### Variable-to-Variable Assignment (v0.17.1)
```grl
then
    order_qty = moq;              // Copy value
    Facts.Result = Facts.Input;   // Transfer between fields
```

### Multiple Actions
```grl
then
    Order.status = "processed";
    Order.processedAt = Now();
    Order.discount = 0.10;
    Log("Order " + Order.id + " completed");
```

---

## Built-in Functions

**Note**: Most built-in functions require the Plugin system. Enable plugins to use these.

### Logging Functions
```grl
Log("Processing order")
LogInfo("Order validated")
LogWarn("Low inventory detected")
LogError("Payment failed")
```

### String Functions
```grl
ToUpper("hello")         // "HELLO"
ToLower("WORLD")         // "world"
Trim("  text  ")         // "text"
Replace(text, "old", "new")
Substring(text, 0, 5)
Length(text)
```

### Math Functions
```grl
Abs(-5)                  // 5
Round(3.7, 0)            // 4.0
Ceil(3.2)                // 4.0
Floor(3.8)               // 3.0
Max(10, 20)              // 20
Min(10, 20)              // 10
Pow(2, 3)                // 8
Sqrt(16)                 // 4
```

### Date/Time Functions
```grl
Now()                    // Current timestamp
AddDays(date, 7)         // Add 7 days
AddHours(date, 24)       // Add 24 hours
FormatDate(date, "YYYY-MM-DD")
ParseDate("2025-11-20", "YYYY-MM-DD")
DaysBetween(date1, date2)
```

### Validation Functions
```grl
IsEmail("user@example.com")    // true/false
IsURL("https://example.com")
IsNumeric("123")
IsAlpha("abc")
InRange(value, 0, 100)
Matches(text, pattern)
```

### Custom Functions
Define custom functions via Plugin API:

```rust
// Register custom function
engine.add_function("CalculateTax", |args| {
    let amount: f64 = args[0].as_f64();
    let rate: f64 = args[1].as_f64();
    amount * rate
});
```

```grl
// Use in GRL
then
    Order.tax = CalculateTax(Order.subtotal, 0.08);
```

---

## Comments

```grl
// Single line comment

/* 
   Multi-line
   comment
*/

rule "Documented" {
    when
        // Check customer tier
        Customer.tier == "gold"
    then
        /* 
         * Apply gold tier discount
         * with special pricing
         */
        Order.discount = 0.15;
}
```

---

## Variable Types

### In Conditions
- **String**: `"text"`, `'text'`
- **Integer**: `42`, `-10`, `0`
- **Float**: `3.14`, `-0.5`, `1.0`
- **Boolean**: `true`, `false`
- **Arrays**: `["a", "b"]`, `[1, 2, 3]`
- **Objects**: `{key: "value"}`
- **Null**: `null`

### Type Coercion
```grl
when
    "42" == 42          // String to number (if supported)
    1 == true           // Number to boolean (1 = true, 0 = false)
```

---

## Best Practices

### 1. Use Descriptive Rule Names
```grl
// ✅ Good
rule "ApplyGoldCustomerDiscount" { ... }
rule "ValidateEmailFormat" { ... }

// ❌ Bad
rule "Rule1" { ... }
rule "Discount" { ... }
```

### 2. Organize with Salience
```grl
// Critical checks first (90-100)
rule "FraudDetection" salience 100 { ... }
rule "SecurityValidation" salience 95 { ... }

// Business logic (40-60)
rule "ApplyDiscount" salience 50 { ... }
rule "CalculateShipping" salience 45 { ... }

// Logging/cleanup last (1-10)
rule "AuditLog" salience 5 { ... }
rule "Cleanup" salience 1 { ... }
```

### 3. Prevent Infinite Loops
```grl
// ✅ Always use no-loop when modifying trigger conditions
rule "UpdateStatus" no-loop true {
    when Order.status == "pending"
    then Order.status = "processed";
}

// ❌ Dangerous without no-loop
rule "InfiniteLoop" {
    when Counter.value < 100
    then Counter.value = Counter.value + 1;  // Will loop forever!
}
```

### 4. Group Related Rules
```grl
// Use agenda groups for workflow stages
rule "StageValidation" agenda-group "validation" { ... }
rule "StageProcessing" agenda-group "processing" { ... }
rule "StageFinalization" agenda-group "finalization" { ... }
```

### 5. Use Activation Groups for Mutual Exclusion
```grl
// Only highest salience in group fires
rule "PlatinumDiscount" activation-group "discounts" salience 30 { ... }
rule "GoldDiscount" activation-group "discounts" salience 20 { ... }
rule "SilverDiscount" activation-group "discounts" salience 10 { ... }
rule "DefaultDiscount" activation-group "discounts" salience 0 { ... }
```

### 6. Add Comments for Complex Logic
```grl
rule "ComplexPricing" {
    when
        // Check if customer qualifies for volume discount
        // AND has been active for 6+ months
        // AND has no payment issues
        (Order.quantity > 100 || Order.total > 5000) &&
        Customer.activeMonths >= 6 &&
        !exists(Payment.status == "failed")
    then
        // Apply compound discount:
        // Base 10% + 1% per year of membership (max 20%)
        Order.discount = 0.10 + Math.min(Customer.yearsActive * 0.01, 0.10);
}
```

---

## Common Patterns

### Tiered Discount System
```grl
rule "PlatinumDiscount" salience 30 activation-group "discounts" {
    when Customer.tier == "platinum"
    then Order.discount = 0.20;
}

rule "GoldDiscount" salience 20 activation-group "discounts" {
    when Customer.tier == "gold"
    then Order.discount = 0.15;
}

rule "SilverDiscount" salience 10 activation-group "discounts" {
    when Customer.tier == "silver"
    then Order.discount = 0.10;
}

rule "DefaultDiscount" salience 0 activation-group "discounts" {
    when true  // Always matches
    then Order.discount = 0.05;
}
```

### Workflow State Machine
```grl
rule "InitiateOrder" agenda-group "intake" auto-focus salience 100 {
    when Order.status == "new"
    then Order.status = "validating";
}

rule "ValidateOrder" agenda-group "validation" salience 50 {
    when Order.status == "validating" && Order.amount > 0
    then Order.status = "validated";
}

rule "ProcessPayment" agenda-group "processing" salience 40 {
    when Order.status == "validated"
    then Order.status = "processing";
}

rule "FinalizeOrder" agenda-group "completion" salience 30 {
    when Order.status == "processing"
    then Order.status = "completed";
}
```

### Validation Chain with Error Accumulation
```grl
rule "ValidateEmail" salience 100 no-loop true {
    when
        User.email != null &&
        !IsEmail(User.email)
    then
        User.errors = User.errors + "Invalid email format; ";
}

rule "ValidateAge" salience 100 no-loop true {
    when User.age < 18
    then User.errors = User.errors + "Must be 18 or older; ";
}

rule "ValidatePhone" salience 100 no-loop true {
    when
        User.phone != null &&
        Length(User.phone) < 10
    then
        User.errors = User.errors + "Invalid phone number; ";
}

rule "ApproveIfValid" salience 50 {
    when User.errors == ""
    then User.approved = true;
}

rule "RejectIfInvalid" salience 50 {
    when User.errors != ""
    then User.approved = false;
}
```

### Dynamic Threshold Comparison (v0.17.1)
```grl
rule "CheckL1Level" salience 50 no-loop true {
    when
        Facts.L1 > Facts.L1Min &&
        Facts.CM2 > Facts.Cm2Min &&
        Facts.productCode == 1
    then
        Facts.levelApprove = 1;
}

rule "DynamicPricingRule" no-loop true {
    when
        Product.currentPrice > Product.basePrice * 1.5
    then
        Product.needsReview = true;
}
```

### Fraud Detection Scoring
```grl
rule "HighAmountAlert" salience 10 {
    when Transaction.amount > 10000
    then Transaction.riskScore = Transaction.riskScore + 30;
}

rule "ForeignLocationAlert" salience 8 {
    when
        Transaction.country != User.homeCountry
    then
        Transaction.riskScore = Transaction.riskScore + 20;
}

rule "LateNightTransaction" salience 6 {
    when
        Transaction.hour >= 23 || Transaction.hour < 6
    then
        Transaction.riskScore = Transaction.riskScore + 15;
}

rule "FlagHighRisk" salience 5 {
    when Transaction.riskScore >= 50
    then
        Transaction.flagged = true;
        Alert("High risk transaction detected");
}
```

---

## Engine-Specific Features

### RETE-UL Engine
For RETE engine (IncrementalEngine), use `Facts.` prefix for dynamic field access:

```grl
rule "RETE Example" salience 50 no-loop true {
    when
        Facts.L1 > Facts.L1Min
    then
        Facts.Approved = true;
}
```

### Native Engine
Standard field access without prefix:

```grl
rule "Native Example" {
    when
        Order.amount > 1000
    then
        Order.discount = 0.10;
}
```

---

## Debugging Tips

### Enable Debug Mode
```rust
let config = EngineConfig {
    debug_mode: true,
    max_cycles: 100,
    ..Default::default()
};
```

### Add Logging to Rules
```grl
rule "DebugRule" {
    when Customer.tier == "gold"
    then
        Log("Gold customer detected: " + Customer.id);
        Order.discount = 0.15;
}
```

### Check Execution Stats
```rust
println!("Rules fired: {}", engine.fired_rules().len());
println!("Engine stats: {}", engine.stats());
```

---

## VS Code Extension

Install [GRL Syntax Highlighting](https://marketplace.visualstudio.com/items?itemName=tonthatvu.grl-syntax-highlighting) for better editing:

**Features:**
- Syntax highlighting
- Code snippets (type `rule`, `when`, `then`)
- Auto-completion
- Bracket matching
- Error detection

**Snippets:**
- `rule` → Full rule template
- `when` → When clause
- `then` → Then clause
- `exist` → EXISTS pattern
- `forall` → FORALL pattern

---

## Example Files

See [examples/](../examples/) directory for complete working examples:

### Basic Examples
- `examples/ecommerce.rs` - E-commerce discount rules
- `examples/expression_demo.rs` - Expression evaluation
- `examples/custom_functions_demo.rs` - Custom function usage

### Advanced Examples (v0.17.x)
- `examples/assignment_test.rs` - Variable assignment (v0.17.1)
- `examples/assignment_test_rete.rs` - RETE variable assignment (v0.17.1)
- `examples/test_modulo_execution.rs` - Arithmetic expressions (v0.17.1)
- `examples/famicanxi_rete_test.rs` - Dynamic thresholds (v0.17.1)

### RETE Examples
- `examples/famicanxi_rules.grl` - Production RETE rules with no-loop
- `examples/rules/no_loop_test.grl` - No-loop directive testing
- `examples/rules/fraud_detection.grl` - Fraud scoring system

---

## Migration Guide

### From v0.16.x to v0.17.x

**New Features:**
1. **Arithmetic in conditions**: `User.Age % 3 == 0`
2. **Variable references**: `Facts.L1 > Facts.L1Min`
3. **No-loop directive**: `no-loop true`
4. **Improved RETE**: Infinite loop prevention

**Breaking Changes:**
- None (fully backward compatible)

**Recommended Updates:**
```grl
// Old (still works)
rule "OldStyle" {
    when Counter.value < 100
    then Counter.value = Counter.value + 1;
}

// New (recommended)
rule "NewStyle" no-loop true {
    when Counter.value < 100
    then Counter.value = Counter.value + 1;
}
```

---

## Performance Tips

### 1. Use Salience Strategically
Higher salience rules execute first. Group by priority:
- **90-100**: Critical validation/security
- **50-80**: Core business logic
- **20-40**: Secondary processing
- **1-10**: Logging/cleanup

### 2. Minimize Rule Re-evaluation
```grl
// ✅ Good: Specific condition
rule "FastRule" {
    when Order.status == "pending" && Order.amount > 1000
    then ...
}

// ❌ Slow: Broad condition
rule "SlowRule" {
    when Order.status != null
    then ...
}
```

### 3. Use RETE for Large Rule Sets
For 100+ rules, RETE-UL engine provides 2-24x performance improvement.

### 4. Leverage Activation Groups
Avoid evaluating multiple mutually exclusive rules.

---

## Troubleshooting

### Infinite Loops
**Problem**: Rule keeps firing infinitely.

**Solution**: Add `no-loop true`:
```grl
rule "FixedRule" no-loop true {
    when Counter.value < 100
    then Counter.value = Counter.value + 1;
}
```

### Rules Not Firing
**Checklist**:
1. Check fact field names (case-sensitive)
2. Verify condition logic
3. Check salience (higher fires first)
4. Enable debug mode
5. Check agenda group focus

### Type Mismatches
**Problem**: `Order.amount > "1000"` doesn't match.

**Solution**: Ensure type consistency:
```grl
when Order.amount > 1000  // Integer comparison
```

### Performance Issues
**Problem**: Slow rule execution.

**Solutions**:
1. Use RETE-UL engine for 50+ rules
2. Optimize conditions (specific > general)
3. Use activation groups for mutual exclusion
4. Profile with benchmarks

---

**Version**: 0.17.1  
**Last Updated**: 2025-11-20  
**Documentation**: [https://github.com/yourusername/rust-rule-engine](https://github.com/yourusername/rust-rule-engine)  
**Issues**: [GitHub Issues](https://github.com/yourusername/rust-rule-engine/issues)

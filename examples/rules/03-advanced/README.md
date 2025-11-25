# Advanced GRL Features

Advanced features of GRL and the rule engine.

## Files

### 1. pattern_matching.grl
**5 rules** - Advanced pattern matching

Features:
- exists, forall, combined patterns
- Complex boolean expressions
- Nested conditions
- Pattern composition

**Example:**
```grl
rule "ComplexPattern" {
    when
        exists(User.Status == "ACTIVE") &&
        forall(Order.IsPaid == true) &&
        !exists(Alert.Level == "CRITICAL")
    then
        ProcessWorkflow();
}
```

### 2. advanced_nested_rules.grl
**3 rules** - Complex boolean operators

Features:
- Deeply nested conditions
- Multiple AND/OR combinations
- Parentheses grouping
- Complex logic trees

**Example:**
```grl
rule "NestedConditions" {
    when
        (User.Age >= 18 && User.Country == "US") ||
        (User.Age >= 21 && User.Country == "JP") ||
        (User.IsVerified == true && User.Premium == true)
    then
        GrantAccess();
}
```

### 3. advanced_method_calls.grl
**4 rules** - Advanced method chaining

Features:
- Method chaining
- Complex method calls
- Return value handling
- Multiple method invocations

**Example:**
```grl
rule "MethodChaining" {
    when
        User.GetProfile().GetPreferences().IsEmailEnabled() == true
    then
        User.GetNotificationService().SendEmail();
}
```

### 4. conflict_resolution_rules.grl
**7 rules** - Conflict resolution strategies

Features:
- Salience (priority) values
- Different complexity levels
- Conflict resolution demo
- Execution order control

**Rule priorities:**
```grl
rule "HighPriority" salience 100 { ... }
rule "MediumPriority" salience 50 { ... }
rule "LowPriority" salience 10 { ... }
```

**Strategies:**
- Salience-based (explicit priority)
- Complexity-based (implicit)
- FIFO (first in, first out)
- LIFO (last in, first out)

### 5. action_handlers.grl
**6 rules** - Action handlers with function calls

Features:
- Custom action handlers
- Function calls in actions
- Side effects management
- External integrations

**Example:**
```grl
rule "NotifyUser" {
    when
        Order.Status == "COMPLETED"
    then
        SendEmailNotification(Order.CustomerEmail, "Order completed");
        LogEvent("ORDER_COMPLETED", Order.Id);
        UpdateInventory(Order.Items);
}
```

### 6. retract_demo.grl
**4 rules** - Fact retraction

Features:
- Retract facts from working memory
- Dynamic fact management
- Clean up after processing
- Memory management

**Example:**
```grl
rule "ProcessAndRetract" {
    when
        TempData.IsProcessed == false
    then
        ProcessData(TempData);
        TempData.IsProcessed = true;
        Retract("TempData");  // Remove from memory
}
```

**Use cases:**
- Temporary data cleanup
- State machine transitions
- Memory optimization
- Event processing

### 7. tms_demo.grl
**8 rules** - Truth Maintenance System

Features:
- Logical dependencies tracking
- Belief revision
- Automatic retraction
- Dependency management

**Example:**
```grl
rule "DerivedFact" {
    when
        Person.Age >= 18
    then
        Assert("Adult", Person);  // Logical assertion
}

rule "Cleanup" {
    when
        Person.Age < 18
    then
        Retract("Adult");  // TMS auto-retracts dependent facts
}
```

**TMS benefits:**
- Automatic consistency maintenance
- Dependency tracking
- No manual cleanup needed
- Prevents inconsistencies

## Advanced Concepts

### 1. Salience (Priority)

Controls execution order when multiple rules are activated:

```grl
// Highest priority - runs first
rule "CriticalValidation" salience 100 {
    when
        Data.IsValid == false
    then
        StopProcessing();
}

// Medium priority
rule "NormalProcessing" salience 50 {
    when
        Data.IsReady == true
    then
        Process();
}

// Lowest priority - cleanup
rule "Cleanup" salience 1 {
    when
        Processing.IsComplete == true
    then
        CleanupResources();
}
```

**Default salience**: 0
**Range**: -1000 to 1000 (typically)

### 2. No-Loop Attribute

Prevent infinite loops:

```grl
rule "UpdateAge" no-loop {
    when
        Person.BirthYear > 0
    then
        Person.Age = CurrentYear - Person.BirthYear;
        // Without no-loop, this might trigger the rule again
}
```

### 3. Agenda Groups

Organize rules into groups:

```grl
rule "Validation" agenda-group "validate" {
    when
        Data.Status == "NEW"
    then
        ValidateData();
}

rule "Processing" agenda-group "process" {
    when
        Data.Status == "VALIDATED"
    then
        ProcessData();
}
```

Execute by group:
```rust
engine.set_focus("validate");
engine.run();
engine.set_focus("process");
engine.run();
```

### 4. Activation Groups

Only one rule in group can fire:

```grl
rule "LargeDiscount" activation-group "discount" {
    when
        Order.Total > 1000
    then
        Order.Discount = 20;
}

rule "MediumDiscount" activation-group "discount" {
    when
        Order.Total > 500
    then
        Order.Discount = 10;
}
// Only the first matching rule fires
```

### 5. Date Effective/Expires

Time-based activation:

```grl
rule "HolidaySale"
    date-effective "2024-12-01"
    date-expires "2024-12-31" {
    when
        Order.Total > 0
    then
        Order.Discount = 15;
}
```

## Pattern Matching Patterns

### Complex Combinations
```grl
rule "ComplexMatch" {
    when
        // At least one premium user
        exists(User.IsPremium == true) &&

        // All orders are paid
        forall(Order.Status == "PAID") &&

        // No pending issues
        !exists(Issue.Status == "OPEN") &&

        // System ready
        System.IsOnline == true
    then
        StartProcessing();
}
```

### Nested Quantifiers
```grl
rule "NestedQuantifiers" {
    when
        exists(
            User.Role == "ADMIN" &&
            forall(User.Permissions.Contains("WRITE"))
        )
    then
        EnableAdminFeatures();
}
```

## Performance Tips

### 1. Specific Patterns First
```grl
// Good - most selective first
when
    User.Id == 12345 && User.IsActive == true

// Less optimal
when
    User.IsActive == true && User.Id == 12345
```

### 2. Minimize Computation in When
```grl
// Good - simple comparison
when
    Order.Total > 1000

// Bad - complex calculation
when
    (Order.ItemCount * Order.AveragePrice * 1.1) > 1000
```

### 3. Use Salience Wisely
Don't overuse - let engine optimize when possible.

### 4. Retract When Done
Clean up temporary facts to reduce memory usage.

## Integration with Rust

```rust
// Load advanced rules
engine.add_grl_file("rules/03-advanced/conflict_resolution_rules.grl")?;

// Set conflict resolution strategy
engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::Salience)?;

// Enable TMS
engine.enable_truth_maintenance(true)?;

// Run with advanced features
engine.run()?;
```

## Run Examples

```bash
# Conflict resolution
cargo run --example conflict_resolution_demo

# Action handlers
cargo run --example action_handlers_demo

# Pattern matching
cargo run --example pattern_matching_from_grl

# TMS demo
cargo run --example tms_demo
```

## Common Pitfalls

### 1. Infinite Loops
```grl
// BAD - will loop forever
rule "UpdateValue" {
    when
        Data.Value < 100
    then
        Data.Value = Data.Value + 1;
}

// GOOD - use no-loop or better condition
rule "UpdateValue" no-loop {
    when
        Data.Value < 100 && Data.Updated == false
    then
        Data.Value = 100;
        Data.Updated = true;
}
```

### 2. Salience Overuse
```grl
// Don't do this for every rule
rule "Rule1" salience 100 { ... }
rule "Rule2" salience 99 { ... }
rule "Rule3" salience 98 { ... }

// Better - only when necessary
rule "CriticalRule" salience 100 { ... }
rule "NormalRule1" { ... }  // default salience
rule "NormalRule2" { ... }
```

### 3. Missing Retraction
```grl
// Memory leak - temporary facts accumulate
rule "CreateTemp" {
    when
        Data.NeedsProcessing == true
    then
        Assert("TempData", new TempData());
}

// Better - retract when done
rule "ProcessTemp" {
    when
        TempData.IsProcessed == true
    then
        Retract("TempData");
}
```

## Next Steps

- `04-use-cases/` - Apply advanced features in production
- `07-advanced-rete/` - RETE-specific advanced features
- Performance tuning in `05-performance/`

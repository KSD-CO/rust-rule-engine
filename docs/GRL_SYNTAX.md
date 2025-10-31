# GRL Syntax Reference

Complete reference for Grule Rule Language (GRL) syntax supported by rust-rule-engine.

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

---

## Basic Rule

```grl
rule "SimpleDiscount" {
    when
        order.amount > 1000
    then
        order.discount = 0.10;
}
```

---

## Rule Attributes

### Salience (Priority)
```grl
rule "HighPriority" salience 100 {
    when condition
    then action;
}
```
Higher salience = higher priority (executes first)

### No-Loop
```grl
rule "PreventLoop" no-loop {
    when order.status == "pending"
    then order.status = "processed";
}
```
Prevents rule from firing again after modifying its own conditions

### Agenda Groups
```grl
rule "ValidationPhase" agenda-group "validation" {
    when order.validated == false
    then order.validated = true;
}
```
Organize rules into execution phases

### Activation Groups
```grl
rule "GoldDiscount" activation-group "discounts" salience 10 {
    when customer.tier == "gold"
    then order.discount = 0.15;
}

rule "SilverDiscount" activation-group "discounts" salience 5 {
    when customer.tier == "silver"
    then order.discount = 0.10;
}
```
Only one rule in group fires (highest salience wins)

### Lock-on-Active
```grl
rule "OncePerActivation" lock-on-active {
    when condition
    then action;
}
```
Fires only once per agenda group activation

### Date Effective/Expires
```grl
rule "HolidaySale" 
    date-effective "2025-12-01" 
    date-expires "2025-12-31" {
    when order.amount > 100
    then order.discount = 0.20;
}
```

---

## Conditions

### Comparison Operators
```grl
when
    age > 18          // Greater than
    age >= 18         // Greater than or equal
    age < 65          // Less than
    age <= 65         // Less than or equal
    status == "active" // Equal
    status != "banned" // Not equal
```

### Logical Operators
```grl
when
    age > 18 && status == "active"  // AND
    tier == "gold" || tier == "platinum" // OR
    !(status == "banned")           // NOT
```

### String Operations
```grl
when
    name.contains("John")
    email.startsWith("admin")
    email.endsWith("@example.com")
    code.matches("ABC*")  // Wildcard pattern
```

### Array Operations
```grl
when
    "premium" in tags  // Membership check
```

### Nested Field Access
```grl
when
    customer.address.city == "New York"
    order.items[0].price > 100
```

---

## Advanced Patterns

### EXISTS Pattern
```grl
when
    exists(Order.status == "pending")
then
    Alert.send("Pending orders detected");
```
True if at least one fact matches

### NOT EXISTS Pattern
```grl
when
    !exists(Payment.status == "failed")
then
    order.payment_ok = true;
```
True if no facts match

### FORALL Pattern
```grl
when
    forall(Item.validated == true)
then
    order.ready_to_ship = true;
```
True if all facts of type match

### Complex Patterns
```grl
when
    customer.tier == "gold" &&
    exists(Order.amount > 1000) &&
    !exists(Complaint.status == "open")
then
    customer.priority = "high";
```

---

## Actions

### Assignment
```grl
then
    order.discount = 0.15;
    order.status = "approved";
```

### Arithmetic
```grl
then
    order.total = order.subtotal * (1 - order.discount);
    counter = counter + 1;
```

### String Operations
```grl
then
    user.full_name = user.first_name + " " + user.last_name;
```

### Function Calls (with Plugins)
```grl
then
    Log("Order processed");
    SendEmail(customer.email, "Order Confirmed");
    CallAPI("https://api.example.com/notify");
```

### Multiple Actions
```grl
then
    order.status = "processed";
    order.processed_at = Now();
    Log("Order " + order.id + " processed");
    SendEmail(customer.email, "Confirmation");
```

---

## Built-in Functions (with Plugins)

### Logging
```grl
Log("message")
LogInfo("info message")
LogWarn("warning")
LogError("error")
```

### String Functions
```grl
ToUpper(text)
ToLower(text)
Trim(text)
Replace(text, old, new)
Substring(text, start, length)
```

### Math Functions
```grl
Abs(number)
Round(number, decimals)
Ceil(number)
Floor(number)
Max(a, b)
Min(a, b)
```

### Date/Time Functions
```grl
Now()
AddDays(date, days)
AddHours(date, hours)
FormatDate(date, format)
ParseDate(string, format)
```

### Validation Functions
```grl
IsEmail(string)
IsURL(string)
IsNumeric(string)
IsAlpha(string)
InRange(value, min, max)
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
        customer.tier == "gold"
    then
        /* Apply gold tier discount */
        order.discount = 0.15;
}
```

---

## Variable Types

### Supported in Conditions
- **String**: `"text"`
- **Integer**: `42`, `-10`
- **Float**: `3.14`, `-0.5`
- **Boolean**: `true`, `false`
- **Arrays**: `[1, 2, 3]` (limited support)

---

## Best Practices

### 1. Descriptive Rule Names
```grl
// Good
rule "ApplyGoldCustomerDiscount" { ... }

// Bad
rule "Rule1" { ... }
```

### 2. Use Salience Wisely
```grl
// High priority for critical checks
rule "FraudDetection" salience 100 { ... }

// Normal priority for business logic
rule "ApplyDiscount" salience 50 { ... }

// Low priority for logging
rule "AuditLog" salience 1 { ... }
```

### 3. Avoid Infinite Loops
```grl
// Use no-loop to prevent self-triggering
rule "UpdateStatus" no-loop {
    when order.status == "pending"
    then order.status = "processed";
}
```

### 4. Group Related Rules
```grl
// Use agenda groups for workflow phases
rule "Validate" agenda-group "validation" { ... }
rule "Process" agenda-group "processing" { ... }
rule "Finalize" agenda-group "finalization" { ... }
```

### 5. Use Activation Groups for Mutually Exclusive Rules
```grl
// Only highest salience fires
rule "PlatinumDiscount" activation-group "discounts" salience 30 { ... }
rule "GoldDiscount" activation-group "discounts" salience 20 { ... }
rule "SilverDiscount" activation-group "discounts" salience 10 { ... }
```

---

## Common Patterns

### Tiered Discount System
```grl
rule "PlatinumDiscount" salience 30 {
    when customer.tier == "platinum"
    then order.discount = 0.20;
}

rule "GoldDiscount" salience 20 {
    when customer.tier == "gold"
    then order.discount = 0.15;
}

rule "SilverDiscount" salience 10 {
    when customer.tier == "silver"
    then order.discount = 0.10;
}
```

### Workflow Stages
```grl
rule "Stage1" agenda-group "intake" auto-focus {
    when request.status == "new"
    then request.status = "validated";
}

rule "Stage2" agenda-group "processing" {
    when request.status == "validated"
    then request.status = "processed";
}

rule "Stage3" agenda-group "completion" {
    when request.status == "processed"
    then request.status = "completed";
}
```

### Validation Chain
```grl
rule "ValidateEmail" salience 100 {
    when !IsEmail(user.email)
    then user.errors.add("Invalid email");
}

rule "ValidateAge" salience 100 {
    when user.age < 18
    then user.errors.add("Must be 18+");
}

rule "ApproveIfValid" salience 50 {
    when user.errors.isEmpty()
    then user.approved = true;
}
```

---

## VS Code Extension

Install [GRL Syntax Highlighting](https://marketplace.visualstudio.com/items?itemName=tonthatvu.grl-syntax-highlighting) for better editing experience:

**Features:**
- Syntax highlighting
- Snippets (type `rule`, `when`, `then`)
- Auto-completion
- Error detection

---

## Examples

See [examples/rules/](../../examples/rules/) directory for complete working examples:
- `discount.grl` - Discount rules
- `validation.grl` - Input validation
- `workflow.grl` - Multi-stage workflows
- `fraud_detection.grl` - Fraud scoring
- `rete_demo.grl` - RETE engine examples

---

**Last Updated**: 2025-10-31 (v0.10.0)

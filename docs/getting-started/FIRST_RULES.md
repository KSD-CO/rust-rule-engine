# Writing Your First Rules

> **Version:** 1.11.0
> **Prerequisite:** [Quick Start](QUICK_START.md), [Basic Concepts](CONCEPTS.md)

Step-by-step guide to writing effective rules.

---

## 📚 Table of Contents

1. [Simple Rule](#simple-rule)
2. [Multiple Conditions](#multiple-conditions)
3. [Complex Logic](#complex-logic)
4. [Working with Numbers](#working-with-numbers)
5. [String Operations](#string-operations)
6. [Best Practices](#best-practices)

---

## Simple Rule

### Your First Rule

Let's start with the simplest possible rule:

```rust
use rust_rule_engine::{Engine, Facts, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new();

    // Simple rule: If X is 1, set Y to 2
    engine.add_rule_from_string(r#"
        rule "SetY" {
            when
                X == 1
            then
                Y = 2;
        }
    "#)?;

    let mut facts = Facts::new();
    facts.set("X", Value::Integer(1));

    engine.run(&mut facts)?;

    println!("Y = {:?}", facts.get("Y"));
    // Output: Y = Some(Integer(2))

    Ok(())
}
```

**Breakdown:**
- `when X == 1` - Condition: Check if X equals 1
- `then Y = 2;` - Action: Set Y to 2
- Rule fires only when condition is true

### Real-World Example: Welcome Bonus

```rust
engine.add_rule_from_string(r#"
    rule "Welcome Bonus" {
        when
            Customer.IsNew == true
        then
            Customer.BonusPoints = 100;
            Customer.WelcomeEmailSent = true;
    }
"#)?;

facts.set("Customer.IsNew", Value::Boolean(true));
engine.run(&mut facts)?;

assert_eq!(facts.get("Customer.BonusPoints"), Some(&Value::Integer(100)));
```

---

## Multiple Conditions

### AND Logic

All conditions must be true:

```rust
engine.add_rule_from_string(r#"
    rule "VIP Discount" {
        when
            Customer.TotalSpent > 1000 &&
            Customer.YearsMember > 2
        then
            Customer.DiscountRate = 0.15;
            Customer.Tier = "VIP";
    }
"#)?;

facts.set("Customer.TotalSpent", Value::Number(1500.0));
facts.set("Customer.YearsMember", Value::Integer(3));

engine.run(&mut facts)?;
```

**Result:** Both conditions true → Rule fires

### Real-World: Loan Approval

```rust
engine.add_rule_from_string(r#"
    rule "Approve Loan" {
        when
            Applicant.CreditScore > 700 &&
            Applicant.Income > 50000 &&
            Applicant.DebtRatio < 0.4 &&
            Applicant.EmploymentYears >= 2
        then
            Loan.Status = "approved";
            Loan.InterestRate = 3.5;
    }
"#)?;
```

### OR Logic (v1.10.0+)

At least one condition must be true:

```rust
engine.add_rule_from_string(r#"
    rule "Priority Customer" {
        when
            (Customer.Type == "VIP" || Customer.TotalSpent > 5000) &&
            Customer.IsActive == true
        then
            Customer.Priority = "high";
            Customer.SupportLine = "premium";
    }
"#)?;
```

**Result:** VIP *OR* high spending → Priority

---

## Complex Logic

### Nested Conditions

```rust
engine.add_rule_from_string(r#"
    rule "Special Offer" {
        when
            (Customer.Age >= 18 && Customer.Age <= 25) &&
            (Student.IsEnrolled == true || Student.GraduatedRecently == true) &&
            Purchase.Category == "Tech"
        then
            Purchase.Discount = 0.2;
            Marketing.TargetGroup = "student-tech";
    }
"#)?;
```

**Logic:**
- Must be 18-25 years old
- AND (currently student OR recent graduate)
- AND buying tech products
- → 20% discount

### Negation

```rust
engine.add_rule_from_string(r#"
    rule "Standard Shipping" {
        when
            Order.Total >= 50 &&
            NOT Customer.IsVIP == true
        then
            Order.ShippingCost = 9.99;
    }
"#)?;
```

**Logic:** Order ≥ $50 but NOT VIP → Charge shipping

---

## Working with Numbers

### Arithmetic in Conditions

```rust
engine.add_rule_from_string(r#"
    rule "Calculate Tax" {
        when
            Order.SubTotal > 0
        then
            Order.Tax = Order.SubTotal * 0.08;
            Order.Total = Order.SubTotal + Order.Tax;
    }
"#)?;

facts.set("Order.SubTotal", Value::Number(100.0));
engine.run(&mut facts)?;

// Order.Tax = 8.0
// Order.Total = 108.0
```

### Comparisons

```rust
engine.add_rule_from_string(r#"
    rule "Bulk Discount" {
        when
            Order.Quantity >= 10 &&
            Order.ItemPrice * Order.Quantity > 500
        then
            Order.BulkDiscount = (Order.ItemPrice * Order.Quantity) * 0.1;
            Order.FinalPrice = (Order.ItemPrice * Order.Quantity) - Order.BulkDiscount;
    }
"#)?;
```

### Integer vs Float

```rust
// Integer comparison
facts.set("Count", Value::Integer(10));
// when Count > 5  ✅ Works

// Float comparison
facts.set("Price", Value::Number(19.99));
// when Price > 15.0  ✅ Works

// Mixed (auto-conversion)
// when Count * Price > 100  ✅ Works
```

---

## String Operations

### String Matching

```rust
engine.add_rule_from_string(r#"
    rule "Premium Member" {
        when
            Customer.Tier == "Platinum" ||
            Customer.Tier == "Gold"
        then
            Customer.Benefits = "premium_lounge_access";
    }
"#)?;
```

### Case-Insensitive Matching

```rust
engine.add_rule_from_string(r#"
    rule "Check Email Domain" {
        when
            Customer.Email.Contains("@company.com")
        then
            Customer.IsEmployee = true;
            Customer.EmployeeDiscount = 0.3;
    }
"#)?;
```

### String Functions

```rust
engine.add_rule_from_string(r#"
    rule "Normalize Name" {
        when
            Customer.Name.Length() > 0
        then
            Customer.DisplayName = Customer.Name.ToUpper();
    }
"#)?;
```

---

## Best Practices

### ✅ DO: Clear Rule Names

```rust
// ✅ Good
rule "Apply VIP Discount for High-Value Customers" { ... }

// ❌ Bad
rule "Rule1" { ... }
```

### ✅ DO: One Purpose Per Rule

```rust
// ✅ Good - Single responsibility
rule "Calculate Tax" {
    when Order.SubTotal > 0
    then Order.Tax = Order.SubTotal * 0.08;
}

rule "Apply Shipping" {
    when Order.Total > 0
    then Order.Shipping = 9.99;
}

// ❌ Bad - Multiple responsibilities
rule "Process Order" {
    when Order.SubTotal > 0
    then
        Order.Tax = Order.SubTotal * 0.08;
        Order.Shipping = 9.99;
        Order.ProcessDate = Today();
        SendEmail();
}
```

### ✅ DO: Use Meaningful Fact Names

```rust
// ✅ Good
Customer.TotalSpent
Order.SubTotal
Product.InStock

// ❌ Bad
X, Y, Z
Val1, Val2
Temp
```

### ✅ DO: Handle Edge Cases

```rust
rule "Safe Division" {
    when
        Order.Quantity > 0 &&  // Prevent division by zero
        Order.Total > 0
    then
        Order.AverageItemPrice = Order.Total / Order.Quantity;
}
```

### ✅ DO: Use Salience for Priority

```rust
// High priority rule
rule "Emergency Override" salience 100 {
    when System.Emergency == true
    then ProcessImmediately();
}

// Normal priority
rule "Standard Process" salience 10 {
    when System.Ready == true
    then ProcessNormally();
}
```

### ❌ DON'T: Circular Dependencies

```rust
// ❌ Bad - Infinite loop
rule "A" {
    when X == 1
    then Y = 2;
}

rule "B" {
    when Y == 2
    then X = 1;  // Sets X back to 1, triggering rule A again!
}
```

### ❌ DON'T: Overly Complex Conditions

```rust
// ❌ Bad - Too complex
rule "Complex" {
    when
        ((A == 1 && B == 2) || (C == 3 && D == 4)) &&
        ((E > 5 && F < 6) || (G != 7 && H >= 8)) &&
        ((I <= 9 || J > 10) && (K == 11 || L < 12))
    then Result = true;
}

// ✅ Good - Break into smaller rules
rule "CheckA" {
    when A == 1 && B == 2
    then Condition1 = true;
}

rule "CheckB" {
    when C == 3 && D == 4
    then Condition2 = true;
}

rule "Final" {
    when Condition1 == true || Condition2 == true
    then Result = true;
}
```

---

## Complete Example: E-commerce Rules

```rust
use rust_rule_engine::{Engine, Facts, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = Engine::new();

    // Rule 1: VIP Status
    engine.add_rule_from_string(r#"
        rule "Grant VIP Status" salience 100 {
            when
                Customer.TotalSpent > 5000 &&
                Customer.OrderCount > 10
            then
                Customer.IsVIP = true;
                Customer.Tier = "VIP";
        }
    "#)?;

    // Rule 2: VIP Discount
    engine.add_rule_from_string(r#"
        rule "VIP Discount" salience 90 {
            when
                Customer.IsVIP == true &&
                Order.SubTotal > 0
            then
                Order.VIPDiscount = Order.SubTotal * 0.15;
        }
    "#)?;

    // Rule 3: Free Shipping
    engine.add_rule_from_string(r#"
        rule "Free Shipping for VIP" salience 80 {
            when
                Customer.IsVIP == true ||
                Order.SubTotal > 100
            then
                Order.ShippingCost = 0;
                Order.FreeShipping = true;
        }
    "#)?;

    // Rule 4: Calculate Total
    engine.add_rule_from_string(r#"
        rule "Calculate Final Total" salience 70 {
            when
                Order.SubTotal > 0
            then
                Order.Tax = Order.SubTotal * 0.08;
                Order.FinalTotal = Order.SubTotal - Order.VIPDiscount + Order.Tax + Order.ShippingCost;
        }
    "#)?;

    // Set up facts
    let mut facts = Facts::new();
    facts.set("Customer.TotalSpent", Value::Number(6000.0));
    facts.set("Customer.OrderCount", Value::Integer(15));
    facts.set("Order.SubTotal", Value::Number(200.0));
    facts.set("Order.VIPDiscount", Value::Number(0.0));
    facts.set("Order.ShippingCost", Value::Number(9.99));

    // Run engine
    engine.run(&mut facts)?;

    // Print results
    println!("=== Order Summary ===");
    println!("VIP Status: {:?}", facts.get("Customer.IsVIP"));
    println!("SubTotal: ${:?}", facts.get("Order.SubTotal"));
    println!("VIP Discount: -${:?}", facts.get("Order.VIPDiscount"));
    println!("Tax: +${:?}", facts.get("Order.Tax"));
    println!("Shipping: ${:?}", facts.get("Order.ShippingCost"));
    println!("Final Total: ${:?}", facts.get("Order.FinalTotal"));

    Ok(())
}
```

**Output:**
```
=== Order Summary ===
VIP Status: Some(Boolean(true))
SubTotal: $Some(Number(200.0))
VIP Discount: -$Some(Number(30.0))
Tax: +$Some(Number(16.0))
Shipping: $Some(Number(0.0))
Final Total: $Some(Number(186.0))
```

---

## Next Steps

**📖 Learn More:**
- [Forward Chaining Guide](../core-features/FORWARD_CHAINING.md)
- [GRL Syntax Reference](../core-features/GRL_SYNTAX.md)
- [Pattern Matching](../core-features/PATTERN_MATCHING.md)

**🔨 Try Advanced Features:**
- [Backward Chaining](../BACKWARD_CHAINING_QUICK_START.md)
- [Streaming & CEP](../advanced-features/STREAMING.md)
- [Modules & Organization](../advanced-features/MODULES.md)

**📚 API Reference:**
- [Complete API](../api-reference/API_REFERENCE.md)
- [Error Handling](../api-reference/ERROR_HANDLING.md)

---

## Navigation

◀️ **Previous: [Concepts](CONCEPTS.md)** | 📚 **[Documentation Home](../README.md)** | ▶️ **Next: [Forward Chaining](../core-features/FORWARD_CHAINING.md)**

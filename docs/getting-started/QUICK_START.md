# Quick Start Guide

> **Version:** 1.11.0
> **Estimated Time:** 5 minutes

Get up and running with Rust Rule Engine in just a few minutes!

---

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-rule-engine = "1.11"

# Optional features
[dependencies.rust-rule-engine]
version = "1.11"
features = ["backward-chaining", "streaming", "streaming-redis"]
```

### Features

| Feature | Description | Use When |
|---------|-------------|----------|
| `backward-chaining` | Goal-driven inference | You need queries and logical reasoning |
| `streaming` | Complex Event Processing | You process real-time event streams |
| `streaming-redis` | Redis state backend | You need distributed state management |

---

## ⚡ Your First Rule (30 seconds)

```rust
use rust_rule_engine::{Engine, Facts, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create an engine
    let mut engine = Engine::new();

    // 2. Add a simple rule
    engine.add_rule_from_string(r#"
        rule "VIP Discount" {
            when
                Customer.TotalSpent > 1000
            then
                Customer.DiscountRate = 0.2;
                Customer.IsVIP = true;
        }
    "#)?;

    // 3. Set up facts
    let mut facts = Facts::new();
    facts.set("Customer.TotalSpent", Value::Number(1500.0));

    // 4. Run the engine
    engine.run(&mut facts)?;

    // 5. Check results
    assert_eq!(facts.get("Customer.IsVIP"), Some(&Value::Boolean(true)));
    assert_eq!(facts.get("Customer.DiscountRate"), Some(&Value::Number(0.2)));

    println!("✅ VIP status granted with 20% discount!");
    Ok(())
}
```

**Output:**
```
✅ VIP status granted with 20% discount!
```

---

## 🎯 Next Steps (2 minutes each)

### 1. Try Multiple Rules

```rust
engine.add_rule_from_string(r#"
    rule "Free Shipping" {
        when
            Customer.IsVIP == true &&
            Order.Total > 50
        then
            Order.ShippingCost = 0;
    }
"#)?;

facts.set("Order.Total", Value::Number(75.0));
engine.run(&mut facts)?;
```

### 2. Use GRL Files

Create `rules/customer.grl`:
```grl
rule "VIP Discount" {
    when
        Customer.TotalSpent > 1000
    then
        Customer.DiscountRate = 0.2;
        Customer.IsVIP = true;
}

rule "Free Shipping" {
    when
        Customer.IsVIP == true && Order.Total > 50
    then
        Order.ShippingCost = 0;
}
```

Load it:
```rust
use std::fs;

let grl_content = fs::read_to_string("rules/customer.grl")?;
engine.add_rules_from_grl(&grl_content)?;
```

### 3. Try Backward Chaining (Queries)

```rust
use rust_rule_engine::backward::BackwardEngine;

let mut bc_engine = BackwardEngine::new(kb);

// Ask a question
let result = bc_engine.query("Customer.IsVIP == true", &facts)?;

if result.provable {
    println!("✅ Customer is VIP!");
} else {
    println!("❌ Customer is not VIP");
}
```

---

## 📚 What to Learn Next

### Choose Your Path:

**🎓 Learn Core Concepts** → [Basic Concepts](CONCEPTS.md)
Understand facts, rules, and pattern matching

**⚡ Forward Chaining Deep Dive** → [Forward Chaining](../core-features/FORWARD_CHAINING.md)
Master the RETE algorithm

**🔍 Backward Chaining & Queries** → [Backward Chaining Quick Start](../BACKWARD_CHAINING_QUICK_START.md)
Goal-driven reasoning and queries

**🌊 Stream Processing** → [Streaming Guide](../advanced-features/STREAMING.md)
Real-time complex event processing

**📖 Full API Reference** → [API Reference](../api-reference/API_REFERENCE.md)
Complete API documentation

---

## 🎯 Common Use Cases

### Business Rules
```rust
// Loan approval
rule "Approve Loan" {
    when
        Applicant.CreditScore > 700 &&
        Applicant.Income > 50000 &&
        Applicant.DebtRatio < 0.4
    then
        Loan.Status = "approved";
}
```

### E-commerce
```rust
// Dynamic pricing
rule "Flash Sale" {
    when
        Product.Category == "Electronics" &&
        Inventory.Stock > 100
    then
        Product.Price = Product.Price * 0.8;
        Product.Label = "Flash Sale!";
}
```

### Healthcare
```rust
// Treatment authorization
rule "Authorize Treatment" {
    when
        Patient.InsuranceCoverage == "Premium" &&
        Treatment.Cost < 10000
    then
        Treatment.Authorized = true;
}
```

---

## 💡 Pro Tips

1. **Use meaningful fact names** - `Customer.TotalSpent` instead of `x.y`
2. **Keep rules simple** - One rule, one purpose
3. **Test incrementally** - Add rules one at a time
4. **Use GRL files** - Better organization for complex systems
5. **Enable features as needed** - Don't load what you don't use

---

## 🐛 Troubleshooting

### "Rule not firing"
✅ Check that facts match the condition exactly
✅ Verify fact types (Number vs Integer)
✅ Run `engine.run()` after setting facts

### "Parser error"
✅ Check GRL syntax (semicolons, braces)
✅ Ensure field names don't have typos
✅ Use double quotes for strings

### Need Help?
- 📖 [Troubleshooting Guide](../guides/TROUBLESHOOTING.md)
- 💬 [GitHub Discussions](https://github.com/KSD-CO/rust-rule-engine/discussions)
- 🐛 [Report an Issue](https://github.com/KSD-CO/rust-rule-engine/issues)

---

## 📊 Performance Tips

```rust
// ✅ Good - specific patterns
when Customer.Type == "VIP"

// ❌ Avoid - too broad
when Customer.TotalSpent > 0

// ✅ Use indexing for large rule sets
engine.with_index_on("Customer.Type");
```

---

## 🚀 Ready for More?

- **[Write Your First Rules](FIRST_RULES.md)** - Detailed rule writing guide
- **[Core Concepts](CONCEPTS.md)** - Deep dive into architecture
- **[API Reference](../api-reference/API_REFERENCE.md)** - Complete API docs
- **[Examples](../examples/)** - Real-world use cases

---

## Navigation

📚 **[Documentation Home](../README.md)** | ▶️ **Next: [Basic Concepts](CONCEPTS.md)**

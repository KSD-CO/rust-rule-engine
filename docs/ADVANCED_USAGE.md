# Advanced Usage Guide

Advanced patterns and techniques for rust-rule-engine.

---

## Complex Pattern Matching

### Combining Patterns

```grl
rule "ComplexCheck" {
    when
        customer.tier == "gold" &&
        exists(Order.amount > 1000) &&
        !exists(Complaint.status == "open") &&
        forall(Payment.status == "cleared")
    then
        customer.approved = true;
}
```

### Nested Conditions

```grl
rule "NestedLogic" {
    when
        (customer.age >= 18 && customer.verified == true) ||
        (customer.guardian != null && customer.guardian.verified == true)
    then
        customer.can_purchase = true;
}
```

---

## Knowledge Base Advanced Usage

### Dynamic Rule Loading

```rust
// Load rules dynamically
engine.load_rules_from_string(rules_text)?;

// Add rules programmatically
engine.add_rule(Rule {
    name: "DynamicRule".to_string(),
    salience: 10,
    conditions: vec![/* ... */],
    actions: vec![/* ... */],
})?;
```

### Rule Management

```rust
// List all rules
let rules = engine.list_rules();

// Get specific rule
let rule = engine.get_rule("RuleName")?;

// Remove rule
engine.remove_rule("OldRule")?;

// Update rule priority
engine.update_rule_salience("RuleName", 100)?;
```

---

## Workflow Patterns

### Sequential Workflow

```grl
rule "Stage1" agenda-group "intake" auto-focus {
    when request.status == "new"
    then 
        request.status = "validated";
        SetAgendaFocus("processing");
}

rule "Stage2" agenda-group "processing" {
    when request.status == "validated"
    then 
        request.status = "processed";
        SetAgendaFocus("completion");
}

rule "Stage3" agenda-group "completion" {
    when request.status == "processed"
    then 
        request.status = "completed";
}
```

### Parallel Workflow

```grl
rule "CheckInventory" activation-group "checks" salience 10 {
    when order.items.count > 0
    then CheckInventory(order.items);
}

rule "CheckPayment" activation-group "checks" salience 10 {
    when order.payment_method != ""
    then ValidatePayment(order.payment_method);
}

rule "CheckShipping" activation-group "checks" salience 10 {
    when order.shipping_address != ""
    then ValidateAddress(order.shipping_address);
}
```

---

## Error Handling

### Graceful Error Handling

```rust
match engine.execute(&mut facts) {
    Ok(fired) => {
        println!("Successfully fired {} rules", fired);
    }
    Err(e) => {
        eprintln!("Rule execution error: {:?}", e);
        // Fallback logic
    }
}
```

### Validation Before Execution

```rust
// Validate facts before execution
if !facts.has("required_field") {
    return Err("Missing required field".into());
}

// Validate rules syntax
match engine.validate_rules() {
    Ok(_) => println!("✅ All rules valid"),
    Err(e) => println!("❌ Invalid rules: {:?}", e),
}
```

---

## Performance Optimization

### Batch Processing

```rust
// Collect all facts first
let mut facts_batch = vec![];
for data in dataset {
    let mut facts = Facts::new();
    // Populate facts...
    facts_batch.push(facts);
}

// Process batch
for facts in facts_batch {
    engine.execute(&mut facts)?;
}
```

### Selective Rule Execution

```rust
// Only execute high-priority rules
engine.set_min_salience(50)?;
engine.execute(&mut facts)?;

// Reset for all rules
engine.set_min_salience(0)?;
```

### Caching

```rust
// Cache frequently used facts
let fact_cache = HashMap::new();
fact_cache.insert("customer_tier", facts.get("customer.tier"));

// Reuse cached values
if let Some(tier) = fact_cache.get("customer_tier") {
    // Use cached tier
}
```

---

## Integration Patterns

### REST API Integration

```rust
rule "NotifyAPI" {
    when order.status == "completed"
    then CallAPI("https://api.example.com/notify", order.id);
}
```

### Database Integration

```rust
use rust_rule_engine::RustRuleEngine;

fn process_with_db(db: &Database, order_id: i64) -> Result<()> {
    let mut engine = RustRuleEngine::new();
    engine.load_default_plugins()?;
    
    // Load order from DB
    let order = db.get_order(order_id)?;
    
    // Convert to facts
    let mut facts = Facts::new();
    facts.set("order.id", Value::Integer(order.id));
    facts.set("order.amount", Value::Float(order.amount));
    
    // Execute rules
    engine.execute(&mut facts)?;
    
    // Save back to DB
    db.update_order(order_id, facts)?;
    
    Ok(())
}
```

### Message Queue Integration

```rust
use tokio;

#[tokio::main]
async fn main() {
    let mut engine = RustRuleEngine::new();
    
    // Listen to message queue
    while let Some(message) = queue.recv().await {
        let mut facts = parse_message(message)?;
        engine.execute(&mut facts)?;
        queue.ack(message).await?;
    }
}
```

---

## Testing Strategies

### Unit Testing Rules

```rust
#[test]
fn test_discount_rule() {
    let mut engine = RustRuleEngine::new();
    engine.load_rules_from_file("rules/discount.grl").unwrap();
    
    let mut facts = Facts::new();
    facts.set("customer.tier", Value::String("gold".to_string()));
    facts.set("order.amount", Value::Float(1500.0));
    
    engine.execute(&mut facts).unwrap();
    
    assert_eq!(facts.get("order.discount"), Some(&Value::Float(225.0)));
}
```

### Coverage Testing

```rust
use rust_rule_engine::Coverage;

let mut coverage = Coverage::new();
coverage.start_tracking(&engine);

// Execute test cases
for test_case in test_cases {
    engine.execute(&mut test_case)?;
}

// Generate report
let report = coverage.generate_report();
println!("Coverage: {:.2}%", report.percentage());
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_discount_always_positive(amount in 0.0..10000.0) {
        let mut engine = RustRuleEngine::new();
        let mut facts = Facts::new();
        facts.set("order.amount", Value::Float(amount));
        
        engine.execute(&mut facts)?;
        
        let discount = facts.get("order.discount")
            .and_then(|v| v.as_float())
            .unwrap_or(0.0);
            
        prop_assert!(discount >= 0.0);
    }
}
```

---

## Debugging Techniques

### Rule Tracing

```rust
// Enable tracing
engine.enable_trace();

// Execute rules
engine.execute(&mut facts)?;

// Get trace
let trace = engine.get_trace();
for entry in trace {
    println!("{}: {} - {}", entry.timestamp, entry.rule, entry.action);
}
```

### Fact Inspection

```rust
// Before execution
println!("Initial facts: {:#?}", facts);

// After execution
println!("Final facts: {:#?}", facts);

// Diff
let changed = facts.get_changed_fields();
println!("Changed: {:?}", changed);
```

### Rule Debugging

```rust
// Add debug logging to rules
rule "Debug" {
    when condition
    then
        Log("Rule fired with: " + field);
        LogInfo("Details: " + details);
}
```

---

## Security Best Practices

### Input Validation

```rust
// Validate user input before adding to facts
fn sanitize_input(input: &str) -> Result<String> {
    if input.contains(';') || input.contains('--') {
        return Err("Invalid input".into());
    }
    Ok(input.to_string())
}
```

### Sandboxing

```rust
// Limit rule capabilities
engine.disable_external_calls()?;
engine.set_max_iterations(1000)?;
engine.set_timeout(Duration::from_secs(5))?;
```

### Access Control

```rust
// Rule-level permissions
if !user.has_permission("execute_critical_rules") {
    engine.disable_rules_with_tag("critical")?;
}
```

---

## Monitoring & Observability

### Metrics Collection

```rust
use prometheus::{Counter, Histogram};

let rules_fired = Counter::new("rules_fired_total", "Total rules fired").unwrap();
let execution_time = Histogram::new("execution_duration_seconds", "Execution time").unwrap();

// Instrument execution
let start = Instant::now();
let fired = engine.execute(&mut facts)?;
execution_time.observe(start.elapsed().as_secs_f64());
rules_fired.inc_by(fired as f64);
```

### Logging

```rust
use tracing::{info, warn, error};

info!("Executing rules for order {}", order_id);

match engine.execute(&mut facts) {
    Ok(fired) => info!("Fired {} rules successfully", fired),
    Err(e) => error!("Rule execution failed: {:?}", e),
}
```

---

## Migration Patterns

### From Other Rule Engines

```rust
// Drools DRL → GRL conversion helper
fn convert_drools_to_grl(drl: &str) -> Result<String> {
    // Conversion logic
    // ...
}

// CLIPS rules → GRL conversion
fn convert_clips_to_grl(clips: &str) -> Result<String> {
    // Conversion logic
    // ...
}
```

---

## Best Practices Summary

1. **Rule Organization**
   - Group related rules with agenda groups
   - Use meaningful rule names
   - Document complex conditions

2. **Performance**
   - Batch fact updates
   - Use RETE for large rule sets
   - Enable memoization

3. **Error Handling**
   - Validate inputs
   - Handle errors gracefully
   - Provide meaningful error messages

4. **Testing**
   - Unit test individual rules
   - Integration test workflows
   - Use coverage analysis

5. **Security**
   - Validate all inputs
   - Sandbox rule execution
   - Implement access control

---

**See Also:**
- [FEATURES.md](FEATURES.md) - Core features
- [RETE_GUIDE.md](RETE_GUIDE.md) - RETE engine guide
- [PERFORMANCE.md](PERFORMANCE.md) - Performance tips

**Last Updated**: 2025-10-31 (v0.10.0)

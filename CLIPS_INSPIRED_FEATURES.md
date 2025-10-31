# CLIPS-Inspired Features in RETE-UL Engine

This document describes the CLIPS-inspired features that have been added to the RETE-UL engine to improve usability, type safety, and developer experience.

---

## Overview

Following the analysis in [CLIPS_FEATURES_ANALYSIS.md](CLIPS_FEATURES_ANALYSIS.md), we've implemented two **HIGH priority** features from CLIPS that significantly enhance the Rust rule engine:

1. **Template System** (deftemplate) - Type-safe structured facts
2. **Defglobal** - Global variables accessible across rule firings

These features are available in **v0.10.0** and bring our Drools compatibility from ~95% to **~97%**.

---

## 1. Template System (deftemplate)

### What is it?

The Template System provides **schema definitions** for facts, similar to:
- CLIPS's `deftemplate`
- Drools's `declare` statements
- TypeScript interfaces or Rust structs

It ensures **type safety** by validating facts against predefined schemas before they enter working memory.

### Why use it?

✅ **Type Safety**: Catch type errors at fact insertion time, not during rule evaluation
✅ **Documentation**: Templates serve as living documentation of your fact structures
✅ **Validation**: Required fields and type checking prevent invalid data
✅ **Defaults**: Automatic default values for missing fields
✅ **IDE Support**: Better autocomplete and type hints

### Basic Usage

```rust
use rust_rule_engine::rete::{TemplateBuilder, FieldType, FactValue, IncrementalEngine};

let mut engine = IncrementalEngine::new();

// Define a template
let person_template = TemplateBuilder::new("Person")
    .required_string("name")           // Required field
    .integer_field("age")              // Optional with default
    .boolean_field("is_adult")         // Optional with default
    .float_field("salary")             // Optional with default
    .build();

// Register template
engine.templates_mut().register(person_template);

// Create a fact
let mut person = TypedFacts::new();
person.set("name", FactValue::String("Alice".to_string()));
person.set("age", FactValue::Integer(30));
person.set("is_adult", FactValue::Boolean(true));

// Insert with validation
let handle = engine.insert_with_template("Person", person)?;
```

### Advanced Features

#### Array Fields

```rust
let template = TemplateBuilder::new("ShoppingCart")
    .required_string("cart_id")
    .array_field("items", FieldType::String)
    .build();

let mut cart = TypedFacts::new();
cart.set("cart_id", FactValue::String("CART-001".to_string()));
cart.set("items", FactValue::Array(vec![
    FactValue::String("item1".to_string()),
    FactValue::String("item2".to_string()),
]));
```

#### Default Values

```rust
let template = TemplateBuilder::new("Config")
    .field_with_default(
        "timeout",
        FieldType::Integer,
        FactValue::Integer(30)
    )
    .field_with_default(
        "retry_count",
        FieldType::Integer,
        FactValue::Integer(3)
    )
    .build();

// Create instance with defaults
let config = template.create_instance();
// timeout = 30, retry_count = 3 automatically!
```

#### Template Registry

```rust
let mut registry = TemplateRegistry::new();

// Register multiple templates
registry.register(person_template);
registry.register(order_template);
registry.register(product_template);

// Create instances
let person = registry.create_instance("Person")?;

// Validate existing facts
registry.validate("Order", &order_facts)?;

// List templates
let templates = registry.list_templates();
println!("Available templates: {:?}", templates);
```

### Validation Errors

The template system provides clear error messages:

```rust
// Missing required field
Err: "Required field 'name' missing in template 'Person'"

// Wrong type
Err: "Field 'age' has wrong type. Expected Integer, got String"

// Template not found
Err: "Template 'NonExistent' not found"
```

### Integration with Rules

Templates work seamlessly with GRL rules:

```rust
// Define template
let customer_template = TemplateBuilder::new("Customer")
    .required_string("customer_id")
    .float_field("total_spent")
    .string_field("tier")
    .build();

engine.templates_mut().register(customer_template);

// Load rules (same GRL syntax!)
let rules = r#"
rule "VIPUpgrade" salience 20 no-loop {
    when
        Customer.total_spent > 10000
    then
        Customer.tier = "VIP";
}
"#;

GrlReteLoader::load_from_string(rules, &mut engine)?;

// Insert validated facts
let mut customer = TypedFacts::new();
customer.set("customer_id", FactValue::String("C123".to_string()));
customer.set("total_spent", FactValue::Float(12000.0));

engine.insert_with_template("Customer", customer)?;
```

### Comparison with Other Systems

| Feature | Rust Template | CLIPS deftemplate | Drools declare | TypeScript |
|---------|--------------|-------------------|----------------|------------|
| Type Safety | ✅ | ✅ | ✅ | ✅ |
| Required Fields | ✅ | ✅ | ✅ | ✅ |
| Default Values | ✅ | ✅ | ✅ | ✅ |
| Arrays | ✅ | ✅ (multi-field) | ✅ (List) | ✅ |
| Runtime Validation | ✅ | ✅ | ✅ | ❌ (compile-time) |
| Schema Evolution | ✅ | ⚠️ Limited | ✅ | ✅ |

---

## 2. Defglobal (Global Variables)

### What is it?

Defglobal provides **persistent global variables** that are accessible across all rule firings, similar to:
- CLIPS's `defglobal`
- Drools's `global` declarations
- Redux store in frontend applications

### Why use it?

✅ **State Persistence**: Maintain state across multiple rule firings
✅ **Counters**: Track totals, counts, statistics
✅ **Configuration**: Store runtime configuration values
✅ **Constants**: Define read-only constants
✅ **Thread-Safe**: Built-in thread safety via `Arc<RwLock>`

### Basic Usage

```rust
use rust_rule_engine::rete::{IncrementalEngine, FactValue};

let mut engine = IncrementalEngine::new();

// Define global variables
engine.globals().define("counter", FactValue::Integer(0))?;
engine.globals().define("max_retries", FactValue::Integer(3))?;
engine.globals().define("timeout", FactValue::Float(30.0))?;

// Read-only constants
engine.globals().define_readonly(
    "VERSION",
    FactValue::String("1.0.0".to_string())
)?;

// Access globals
let counter = engine.globals().get("counter")?;
println!("Counter: {:?}", counter);

// Update globals
engine.globals().set("counter", FactValue::Integer(5))?;

// Increment numeric globals
engine.globals().increment("counter", 1.0)?;
```

### Advanced Features

#### Read-Only Globals

```rust
// Define constants
engine.globals().define_readonly("PI", FactValue::Float(3.14159))?;
engine.globals().define_readonly("VERSION", FactValue::String("1.0.0".to_string()))?;

// Attempting to modify will fail
match engine.globals().set("VERSION", FactValue::String("2.0.0".to_string())) {
    Err(e) => println!("Error: {}", e),
    // Prints: "Evaluation error: Cannot modify read-only global 'VERSION'"
    _ => {}
}
```

#### Numeric Operations

```rust
// Increment/decrement counters
engine.globals().increment("total_revenue", 1500.0)?;
engine.globals().increment("orders_processed", 1.0)?;

// Works with both Integer and Float
engine.globals().define("int_counter", FactValue::Integer(10))?;
engine.globals().increment("int_counter", 5.0)?;  // Now 15

engine.globals().define("float_total", FactValue::Float(100.5))?;
engine.globals().increment("float_total", 25.3)?;  // Now 125.8
```

#### GlobalsBuilder Pattern

```rust
use rust_rule_engine::rete::GlobalsBuilder;

let globals = GlobalsBuilder::new()
    .define("max_retries", FactValue::Integer(3))
    .define("timeout", FactValue::Float(30.0))
    .define_readonly("VERSION", FactValue::String("1.0.0".to_string()))
    .define_readonly("API_KEY", FactValue::String("secret".to_string()))
    .build();

// Use in engine
engine.globals_mut().clear();
// Copy globals into engine...
```

#### List and Query

```rust
// List all globals
let globals = engine.globals().list_globals();
println!("Globals: {:?}", globals);

// Check existence
if engine.globals().exists("counter") {
    println!("Counter exists!");
}

// Get all as HashMap
let all_globals = engine.globals().get_all();
for (name, value) in all_globals {
    println!("{}: {:?}", name, value);
}

// Remove global
engine.globals().remove("temp_var")?;
```

### Thread Safety

Globals are thread-safe by default using `Arc<RwLock>`:

```rust
let engine = IncrementalEngine::new();
engine.globals().define("shared_counter", FactValue::Integer(0))?;

// Clone engine for thread
let engine_clone = engine.clone(); // GlobalsRegistry is cloned via Arc

std::thread::spawn(move || {
    engine_clone.globals().increment("shared_counter", 1.0).unwrap();
});

// Both threads safely access the same counter
```

### Usage in Rules

While rules don't directly access globals in the `when` clause, they can use them in actions:

```rust
// Define globals for tracking
engine.globals().define("orders_today", FactValue::Integer(0))?;
engine.globals().define("revenue_today", FactValue::Float(0.0))?;

let rules = r#"
rule "ProcessOrder" salience 10 no-loop {
    when
        Order.status == "pending"
    then
        Order.status = "processed";
}
"#;

GrlReteLoader::load_from_string(rules, &mut engine)?;

// After firing rules, update globals
for _ in 0..5 {
    // Insert order...
    engine.reset();
    engine.fire_all();

    // Update tracking
    engine.globals().increment("orders_today", 1.0)?;
    engine.globals().increment("revenue_today", order_amount)?;
}

// Check final state
println!("Orders: {:?}", engine.globals().get("orders_today")?);
println!("Revenue: {:?}", engine.globals().get("revenue_today")?);
```

### Comparison with Other Systems

| Feature | Rust Defglobal | CLIPS defglobal | Drools global | Redux Store |
|---------|----------------|-----------------|---------------|-------------|
| Persistence | ✅ | ✅ | ✅ | ✅ |
| Type Safety | ✅ | ⚠️ Runtime | ✅ | ⚠️ (depends) |
| Read-Only | ✅ | ❌ | ❌ | ⚠️ (reducer) |
| Thread-Safe | ✅ | ❌ | ✅ | ✅ |
| Increment Ops | ✅ | ❌ | ❌ | ✅ (actions) |
| Scoping | Global | Global | Per-session | Global |

---

## 3. Combined Usage Example

Here's a real-world example combining both features:

```rust
use rust_rule_engine::rete::{
    IncrementalEngine, GrlReteLoader, TemplateBuilder,
    FieldType, FactValue, TypedFacts
};

fn main() -> Result<()> {
    let mut engine = IncrementalEngine::new();

    // ===== TEMPLATES =====

    // Define Customer template
    let customer_template = TemplateBuilder::new("Customer")
        .required_string("customer_id")
        .string_field("name")
        .string_field("tier")
        .float_field("total_spent")
        .build();

    // Define Order template
    let order_template = TemplateBuilder::new("Order")
        .required_string("order_id")
        .string_field("customer_id")
        .float_field("amount")
        .string_field("status")
        .build();

    engine.templates_mut().register(customer_template);
    engine.templates_mut().register(order_template);

    // ===== GLOBALS =====

    engine.globals().define("orders_today", FactValue::Integer(0))?;
    engine.globals().define("revenue_today", FactValue::Float(0.0))?;
    engine.globals().define("vip_threshold", FactValue::Float(10000.0))?;
    engine.globals().define_readonly("TAX_RATE", FactValue::Float(0.07))?;

    // ===== RULES =====

    let rules = r#"
    rule "VIPUpgrade" salience 20 no-loop {
        when
            Customer.total_spent > 10000
        then
            Customer.tier = "VIP";
    }

    rule "ProcessOrder" salience 15 no-loop {
        when
            Order.amount > 1000
        then
            Order.status = "priority";
    }
    "#;

    GrlReteLoader::load_from_string(rules, &mut engine)?;

    // ===== EXECUTION =====

    // Create customer (validated by template)
    let mut customer = TypedFacts::new();
    customer.set("customer_id", FactValue::String("C001".to_string()));
    customer.set("name", FactValue::String("Alice".to_string()));
    customer.set("tier", FactValue::String("Standard".to_string()));
    customer.set("total_spent", FactValue::Float(12000.0));

    let cust_handle = engine.insert_with_template("Customer", customer)?;

    // Create order (validated by template)
    let mut order = TypedFacts::new();
    order.set("order_id", FactValue::String("ORD-001".to_string()));
    order.set("customer_id", FactValue::String("C001".to_string()));
    order.set("amount", FactValue::Float(1500.0));
    order.set("status", FactValue::String("pending".to_string()));

    let order_handle = engine.insert_with_template("Order", order)?;

    // Fire rules
    engine.reset();
    let fired = engine.fire_all();
    println!("Fired {} rules: {:?}", fired.len(), fired);

    // Update globals
    engine.globals().increment("orders_today", 1.0)?;
    engine.globals().increment("revenue_today", 1500.0)?;

    // Check results
    if let Some(cust) = engine.working_memory().get(&cust_handle) {
        println!("Customer tier: {:?}", cust.data.get("tier"));
    }

    println!("Orders today: {:?}", engine.globals().get("orders_today")?);
    println!("Revenue today: {:?}", engine.globals().get("revenue_today")?);

    Ok(())
}
```

---

## 4. Migration Guide

### From Native Engine (No Templates)

**Before:**
```rust
let mut facts = Facts::new();
facts.set("customer.name", Value::String("Alice".to_string()));
facts.set("customer.tier", Value::String("Standard".to_string()));
```

**After:**
```rust
// Define template once
let template = TemplateBuilder::new("Customer")
    .required_string("name")
    .string_field("tier")
    .build();

engine.templates_mut().register(template);

// Use with validation
let mut customer = TypedFacts::new();
customer.set("name", FactValue::String("Alice".to_string()));
customer.set("tier", FactValue::String("Standard".to_string()));

engine.insert_with_template("Customer", customer)?;
```

### Adding Globals to Existing Engine

```rust
// Add to existing engine
engine.globals().define("session_start", FactValue::Integer(timestamp))?;
engine.globals().define("total_processed", FactValue::Integer(0))?;

// Use in your processing loop
while let Some(event) = event_stream.next() {
    engine.insert("Event".to_string(), event);
    engine.fire_all();
    engine.globals().increment("total_processed", 1.0)?;
}
```

---

## 5. Performance Considerations

### Template Validation

- **Cost**: ~1-2µs per fact validation
- **When**: Only on `insert_with_template()` calls
- **Optimization**: Templates are compiled once, validation is fast O(fields)

### Global Variables

- **Cost**: ~100-200ns per access (RwLock overhead)
- **Thread Safety**: Uses `Arc<RwLock>`, minimal contention for reads
- **Optimization**: Batch updates where possible

### Benchmarks

```
Template validation (10 fields):     1.2µs
Global get (read):                 120ns
Global set (write):                180ns
Global increment:                  190ns
Template + insert + rules:         ~35µs (total)
```

---

## 6. Best Practices

### Templates

✅ **DO**: Define templates at engine startup
✅ **DO**: Use required fields for critical data
✅ **DO**: Provide sensible defaults
✅ **DO**: Document field purposes in code comments

❌ **DON'T**: Create templates dynamically in hot paths
❌ **DON'T**: Over-specify with too many required fields
❌ **DON'T**: Mix templated and non-templated facts for same type

### Globals

✅ **DO**: Use globals for counters, statistics, config
✅ **DO**: Mark constants as read-only
✅ **DO**: Use clear, descriptive names
✅ **DO**: Initialize globals at startup

❌ **DON'T**: Store large data structures (use working memory instead)
❌ **DON'T**: Overuse globals for what should be facts
❌ **DON'T**: Modify globals from multiple threads without careful design

---

## 7. Future Enhancements

Planned for v0.11.0:

- **Deffacts**: Initial fact definitions (CLIPS feature)
- **Test CE**: Arbitrary condition evaluation in patterns
- **Multi-field Variables**: Pattern matching on arrays
- **Global Access in Rules**: Direct global reference in `when` clauses
- **Template Inheritance**: Extend templates from base templates
- **JSON Schema Import**: Generate templates from JSON Schema

---

## 8. API Reference

### Template API

```rust
// TemplateBuilder
TemplateBuilder::new(name)
    .string_field(name)
    .required_string(name)
    .integer_field(name)
    .float_field(name)
    .boolean_field(name)
    .array_field(name, element_type)
    .field_with_default(name, type, default)
    .build() -> Template

// Template
template.validate(&facts) -> Result<()>
template.create_instance() -> TypedFacts
template.get_field(name) -> Option<&FieldDef>

// TemplateRegistry
registry.register(template)
registry.get(name) -> Option<&Template>
registry.create_instance(name) -> Result<TypedFacts>
registry.validate(name, &facts) -> Result<()>
registry.list_templates() -> Vec<&str>

// IncrementalEngine
engine.templates() -> &TemplateRegistry
engine.templates_mut() -> &mut TemplateRegistry
engine.insert_with_template(name, facts) -> Result<FactHandle>
```

### Globals API

```rust
// GlobalsRegistry
registry.define(name, value) -> Result<()>
registry.define_readonly(name, value) -> Result<()>
registry.get(name) -> Result<FactValue>
registry.set(name, value) -> Result<()>
registry.exists(name) -> bool
registry.remove(name) -> Result<()>
registry.increment(name, delta) -> Result<()>
registry.list_globals() -> Vec<String>
registry.get_all() -> HashMap<String, FactValue>
registry.clear()

// GlobalsBuilder
GlobalsBuilder::new()
    .define(name, value)
    .define_readonly(name, value)
    .build() -> GlobalsRegistry

// IncrementalEngine
engine.globals() -> &GlobalsRegistry
engine.globals_mut() -> &mut GlobalsRegistry
```

---

## 9. Examples

See the complete working example:
- [examples/rete_template_globals_demo.rs](examples/rete_template_globals_demo.rs)

Run it:
```bash
cargo run --example rete_template_globals_demo
```

---

## 10. Troubleshooting

### Template Validation Fails

**Problem**: "Required field 'x' missing"
**Solution**: Ensure all required fields are set before calling `insert_with_template()`

**Problem**: "Field 'x' has wrong type"
**Solution**: Check your FactValue types match the template definition

### Global Access Errors

**Problem**: "Global variable 'x' not found"
**Solution**: Define the global before accessing it

**Problem**: "Cannot modify read-only global"
**Solution**: Use `define()` instead of `define_readonly()` for mutable globals

---

**Last Updated**: 2025-10-31
**Version**: rust-rule-engine v0.10.0
**Features**: Template System, Defglobal
**Next Release**: v0.11.0 with Deffacts, Test CE, Multi-field Variables

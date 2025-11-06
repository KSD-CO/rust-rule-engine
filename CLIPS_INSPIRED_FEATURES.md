# CLIPS-Inspired Features in RETE-UL Engine

This document describes the CLIPS-inspired features that have been added to the RETE-UL engine to improve usability, type safety, and developer experience.

---

## Overview

Following the analysis in [CLIPS_FEATURES_ANALYSIS.md](CLIPS_FEATURES_ANALYSIS.md), we've implemented **HIGH priority** features from CLIPS that significantly enhance the Rust rule engine:

1. **Template System** (deftemplate) - Type-safe structured facts *(v0.10.0)*
2. **Defglobal** - Global variables accessible across rule firings *(v0.10.0)*
3. **Deffacts** - Initial fact definitions *(v0.11.0)*
4. **Test CE** (test conditional element) - Arbitrary boolean expressions *(v0.12.0)*
5. **Conflict Resolution Strategies** - CLIPS/Drools-inspired rule ordering *(v0.13.0)*

These features bring our Drools compatibility from ~95% to **~98%** and provide powerful CLIPS-style condition evaluation.

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

## 3. Deffacts (Initial Facts)

### What is it?

Deffacts provides **initial fact definitions** that are automatically loaded into working memory, similar to:
- CLIPS's `deffacts`
- Drools's declared facts
- Database seed data

### Why use it?

✅ **Initial State**: Define starting state for your system
✅ **Seed Data**: Pre-populate working memory with default entities
✅ **Testing**: Consistent initial state for test scenarios
✅ **Reset Support**: Restore original state with `reset_with_deffacts()`
✅ **Organization**: Group related initial facts together

### Basic Usage

```rust
use rust_rule_engine::rete::{IncrementalEngine, DeffactsBuilder, FactValue, TypedFacts};

let mut engine = IncrementalEngine::new();

// Create initial person facts
let mut person1 = TypedFacts::new();
person1.set("name", FactValue::String("Alice".to_string()));
person1.set("age", FactValue::Integer(30));

let mut person2 = TypedFacts::new();
person2.set("name", FactValue::String("Bob".to_string()));
person2.set("age", FactValue::Integer(25));

// Create deffacts using builder
let initial_people = DeffactsBuilder::new("initial-people")
    .add_fact("Person", person1)
    .add_fact("Person", person2)
    .with_description("Initial person data")
    .build();

// Register deffacts
engine.deffacts_mut().register(initial_people)?;

// Load all deffacts into working memory
let handles = engine.load_deffacts();
println!("Loaded {} facts", handles.len());
```

### Advanced Features

#### Multiple Deffacts Sets

```rust
// First deffacts set - Users
let users_deffacts = DeffactsBuilder::new("system-users")
    .add_fact("User", admin_data)
    .add_fact("User", guest_data)
    .with_description("System user accounts")
    .build();

// Second deffacts set - Configuration
let config_deffacts = DeffactsBuilder::new("system-config")
    .add_fact("Config", config_data)
    .with_description("System configuration")
    .build();

// Register both
engine.deffacts_mut().register(users_deffacts)?;
engine.deffacts_mut().register(config_deffacts)?;

// Load all at once
let handles = engine.load_deffacts();
```

#### Batch Add Facts

```rust
let mut person1 = TypedFacts::new();
person1.set("name", FactValue::String("Alice".to_string()));

let mut person2 = TypedFacts::new();
person2.set("name", FactValue::String("Bob".to_string()));

let people = vec![person1, person2];

let deffacts = DeffactsBuilder::new("people")
    .add_facts("Person", people)  // Add multiple facts of same type
    .build();
```

#### Deffacts Registry

```rust
let registry = engine.deffacts();

// List all deffacts
let names = registry.list_deffacts();
println!("Deffacts: {:?}", names);

// Check existence
if registry.exists("initial-data") {
    println!("Found initial-data");
}

// Get total fact count
let total = registry.total_fact_count();
println!("Total facts: {}", total);

// Load specific deffacts by name
let handles = engine.load_deffacts_by_name("initial-data")?;
```

### Integration with Templates

Deffacts works seamlessly with templates for type-safe initial facts:

```rust
// Define template
let customer_template = TemplateBuilder::new("Customer")
    .required_string("customer_id")
    .string_field("name")
    .string_field("tier")
    .float_field("total_spent")
    .build();

engine.templates_mut().register(customer_template);

// Create initial customers
let mut customer1 = TypedFacts::new();
customer1.set("customer_id", FactValue::String("C001".to_string()));
customer1.set("name", FactValue::String("VIP Corp".to_string()));
customer1.set("tier", FactValue::String("VIP".to_string()));
customer1.set("total_spent", FactValue::Float(50000.0));

let customers_deffacts = DeffactsBuilder::new("initial-customers")
    .add_fact("Customer", customer1)
    .with_description("VIP customers")
    .build();

engine.deffacts_mut().register(customers_deffacts)?;

// Load deffacts - will validate against template!
let handles = engine.load_deffacts();
```

### Reset and Reload

Similar to CLIPS reset functionality:

```rust
// Initial load
let handles = engine.load_deffacts();

// Modify facts...
engine.update(handles[0], modified_data)?;

// Reset - clears working memory and reloads all deffacts
let new_handles = engine.reset_with_deffacts();
// All facts restored to original deffacts values!
```

### Usage with Rules

```rust
// Create initial pending orders
let orders_deffacts = DeffactsBuilder::new("pending-orders")
    .add_fact("Order", order1)
    .add_fact("Order", order2)
    .add_fact("Order", order3)
    .with_description("Orders waiting to be processed")
    .build();

engine.deffacts_mut().register(orders_deffacts)?;

// Load business rules
let rules = r#"
rule "HighValueOrder" salience 20 no-loop {
    when
        Order.amount > 1000
    then
        Order.priority = "high";
}

rule "ProcessOrder" salience 10 no-loop {
    when
        Order.status == "pending"
    then
        Order.status = "processing";
}
"#;

GrlReteLoader::load_from_string(rules, &mut engine)?;

// Load deffacts
engine.load_deffacts();

// Fire rules on initial facts
engine.reset();
let fired = engine.fire_all();
println!("Processed {} rules", fired.len());
```

### Comparison with Other Systems

| Feature | Rust Deffacts | CLIPS deffacts | Drools declared | SQL seed data |
|---------|---------------|----------------|-----------------|---------------|
| Initial Facts | ✅ | ✅ | ✅ | ✅ |
| Type Safety | ✅ (with templates) | ⚠️ Runtime | ✅ | ⚠️ (depends) |
| Reset Support | ✅ | ✅ | ❌ | ❌ |
| Multiple Sets | ✅ | ✅ | ❌ | ✅ (migrations) |
| Builder API | ✅ | ❌ | ❌ | ⚠️ (ORMs) |
| Runtime Load | ✅ | ❌ (parse-time) | ✅ | ✅ |

---

## 4. Test CE (Test Conditional Element)

### What is it?

The Test CE allows you to call **arbitrary boolean functions** directly in rule conditions without comparison operators. This is a powerful CLIPS feature that enables:

- **Custom validation logic** - Email validation, regex matching, range checks
- **Complex computations** - Call any function that returns a boolean
- **Multiple arguments** - Pass multiple fact fields or literals to test functions
- **Negation support** - Use `!test()` for inverted logic
- **Combined conditions** - Mix test() with regular conditions using AND/OR

### Why use it?

✅ **Arbitrary Logic**: Call any function, not limited to comparison operators
✅ **Cleaner Rules**: More readable than complex AND/OR chains
✅ **Function Reuse**: Share validation logic across multiple rules
✅ **Type Flexibility**: Automatic truthy evaluation for all value types
✅ **CLIPS Compatible**: Direct port of CLIPS test CE feature

### Basic Usage

```rust
use rust_rule_engine::{RustRuleEngine, Facts, Value};

let mut engine = RustRuleEngine::new();

// Register a test function
engine.register_function(
    "is_valid_email",
    |args: &[Value], _facts: &Facts| {
        if let Some(Value::String(email)) = args.first() {
            Ok(Value::Boolean(email.contains('@') && email.contains('.')))
        } else {
            Ok(Value::Boolean(false))
        }
    },
);

// Load GRL rule with test()
let grl = r#"
rule "ValidateEmail" {
    when
        test(is_valid_email(User.email))
    then
        User.status = "valid";
}
"#;
engine.load_rules_from_string(grl)?;
```

### GRL Syntax

```grl
// Simple test CE
rule "EmailCheck" {
    when
        test(is_valid_email(User.email))
    then
        User.verified = true;
}

// Test with multiple arguments
rule "PriceRangeCheck" {
    when
        test(in_range(Product.price, 100, 1000))
    then
        Product.category = "mid-range";
}

// Combined with regular conditions
rule "ApproveOrder" {
    when
        Order.amount > 100 &&
        test(is_valid_email(Customer.email))
    then
        Order.status = "approved";
}

// Negated test CE
rule "BlockInvalidEmail" {
    when
        User.checkEmail == true &&
        !test(is_valid_email(User.email))
    then
        User.status = "blocked";
}
```

### Advanced Features

#### Multiple Test Functions

```rust
// Range validation
engine.register_function(
    "in_range",
    |args: &[Value], _facts: &Facts| {
        if args.len() >= 3 {
            if let (Some(Value::Number(val)), Some(Value::Number(min)), Some(Value::Number(max)))
                = (args.get(0), args.get(1), args.get(2)) {
                return Ok(Value::Boolean(*val >= *min && *val <= *max));
            }
        }
        Ok(Value::Boolean(false))
    },
);

// Age validation
engine.register_function(
    "is_adult",
    |args: &[Value], _facts: &Facts| {
        if let Some(Value::Integer(age)) = args.first() {
            Ok(Value::Boolean(*age >= 18))
        } else {
            Ok(Value::Boolean(false))
        }
    },
);
```

#### Truthy Evaluation

Test functions can return any value type, which is automatically converted to boolean:

```rust
engine.register_function(
    "get_count",
    |args: &[Value], _facts: &Facts| {
        // Return integer - will be truthy if != 0
        Ok(Value::Integer(5))
    },
);

// In GRL:
// test(get_count(items)) - fires if count != 0
```

**Truthy conversion rules:**
- `Boolean(b)` → `b`
- `Integer(i)` → `i != 0`
- `Number(f)` → `f != 0.0`
- `String(s)` → `!s.is_empty()`
- Other types → `false`

#### Combined Logic Example

```rust
let grl = r#"
rule "PremiumCustomer" {
    when
        test(is_valid_email(Customer.email)) &&
        test(in_range(Customer.age, 25, 65)) &&
        Customer.spending > 1000
    then
        Customer.tier = "premium";
}
"#;
```

### Comparison Table

| Feature | Test CE | Regular Condition | Function Call in WHEN |
|---------|---------|-------------------|----------------------|
| Syntax | `test(func(args))` | `field == value` | `func(args) == value` |
| Operator Required | ❌ | ✅ | ✅ |
| Returns Boolean Directly | ✅ | ❌ | ❌ |
| Multiple Arguments | ✅ | ❌ | ✅ |
| Truthy Evaluation | ✅ | ❌ | ❌ |
| Negation | `!test()` | `field != value` | `func() != value` |
| Use Case | Arbitrary logic | Simple comparison | Return value comparison |

### Examples

See [test_ce_comprehensive.rs](examples/test_ce_comprehensive.rs) for complete examples including:
- Email validation
- Range checking with multiple arguments
- Combined conditions (regular + test CE)
- Negated test CE
- GRL file parsing

### Current Implementation Status

**Native Engine**: ✅ Fully implemented
- Function registry
- GRL parsing support
- Truthy evaluation
- Negation support
- Combined conditions

**RETE Engine**: 🚧 Partial
- Function registry added
- GRL parsing support
- Evaluation logic pending

### Common Use Cases

#### Email Validation
```grl
rule "ValidateEmail" {
    when
        test(is_valid_email(User.email))
    then
        User.status = "verified";
}
```

#### Business Hours Check
```rust
engine.register_function(
    "is_business_hours",
    |args: &[Value], _facts: &Facts| {
        let now = chrono::Local::now();
        let hour = now.hour();
        Ok(Value::Boolean(hour >= 9 && hour < 17))
    },
);
```

#### Complex Validation
```grl
rule "ApproveTransaction" {
    when
        test(is_valid_card(Payment.card_number)) &&
        test(has_sufficient_balance(Account.balance, Payment.amount)) &&
        test(is_within_limit(Payment.amount, Account.daily_limit))
    then
        Payment.status = "approved";
}
```

---

## 5. Conflict Resolution Strategies

### What is it?

Conflict Resolution Strategies determine **which rule fires first** when multiple rules are activated simultaneously. This is a core CLIPS/Drools feature inspired by:
- **CLIPS** conflict resolution strategies (Depth, Breadth, LEX, MEA, Complexity, Simplicity, Random)
- **Drools** salience and agenda groups
- **Production system theory** for conflict resolution

When multiple rules match the same facts, the engine needs a systematic way to decide the firing order. Different strategies are optimal for different scenarios.

### Why use it?

✅ **Deterministic Behavior**: Control exact rule execution order
✅ **Priority-Based**: Higher priority rules fire first (Salience)
✅ **Recency-Based**: Most recent facts trigger first (LEX)
✅ **Specificity-Based**: More specific rules fire first (MEA, Complexity)
✅ **Simplicity-First**: Simpler rules fire before complex ones (Simplicity)
✅ **CLIPS/Drools Compatible**: Industry-standard conflict resolution

### Available Strategies

| Strategy | Description | Best For |
|----------|-------------|----------|
| **Salience** | Priority-based (higher values first) | Business rules with explicit priorities |
| **LEX** | Recency (most recent facts first) | Event processing, reactive systems |
| **MEA** | Recency + Specificity (recent + complex) | Balanced performance |
| **Depth** | Depth-first execution | Workflow chains |
| **Breadth** | Breadth-first execution | Parallel processing |
| **Simplicity** | Fewer conditions first | Quick checks before complex logic |
| **Complexity** | More conditions first | Specific rules before general ones |
| **Random** | Random ordering | Testing, fuzzing |

### Basic Usage

```rust
use rust_rule_engine::rete::{
    IncrementalEngine, ConflictResolutionStrategy, Activation
};

let mut engine = IncrementalEngine::new();

// Set strategy (default is Salience)
engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::Salience);

// Rules fire in priority order: HighPriority → MediumPriority → LowPriority
```

### Examples

See [conflict_resolution_demo.rs](examples/conflict_resolution_demo.rs) for comprehensive examples of all 8 strategies.

```bash
cargo run --example conflict_resolution_demo
```

---

## 6. Combined Usage Example

Here's a real-world example combining multiple features:

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

## 7. Migration Guide

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

## 8. Performance Considerations

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

## 9. Best Practices

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

## 10. Future Enhancements

Completed in v0.11.0:
- ✅ **Deffacts**: Initial fact definitions (CLIPS feature)

Planned for future releases:
- **Test CE**: Arbitrary condition evaluation in patterns
- **Multi-field Variables**: Pattern matching on arrays
- **Global Access in Rules**: Direct global reference in `when` clauses
- **Template Inheritance**: Extend templates from base templates
- **JSON Schema Import**: Generate templates from JSON Schema

---

## 11. API Reference

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

### Deffacts API

```rust
// DeffactsBuilder
DeffactsBuilder::new(name)
    .add_fact(fact_type, data)
    .add_facts(fact_type, vec![data1, data2])
    .with_description(description)
    .build() -> Deffacts

// Deffacts
deffacts.name -> String
deffacts.facts -> Vec<FactInstance>
deffacts.description -> Option<String>
deffacts.fact_count() -> usize
deffacts.is_empty() -> bool

// DeffactsRegistry
registry.register(deffacts) -> Result<()>
registry.register_or_replace(deffacts)
registry.get(name) -> Option<&Deffacts>
registry.get_mut(name) -> Option<&mut Deffacts>
registry.exists(name) -> bool
registry.remove(name) -> Result<Deffacts>
registry.list_deffacts() -> Vec<String>
registry.get_all_facts() -> Vec<(String, FactInstance)>
registry.total_fact_count() -> usize
registry.clear()

// IncrementalEngine
engine.deffacts() -> &DeffactsRegistry
engine.deffacts_mut() -> &mut DeffactsRegistry
engine.load_deffacts() -> Vec<FactHandle>
engine.load_deffacts_by_name(name) -> Result<Vec<FactHandle>>
engine.reset_with_deffacts() -> Vec<FactHandle>
```

---

## 12. Examples

See the complete working examples:
- [examples/rete_template_globals_demo.rs](examples/rete_template_globals_demo.rs) - Templates & Globals
- [examples/rete_deffacts_demo.rs](examples/rete_deffacts_demo.rs) - Deffacts System

Run them:
```bash
cargo run --example rete_template_globals_demo
cargo run --example rete_deffacts_demo
```

---

## 13. Troubleshooting

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

### Deffacts Errors

**Problem**: "Deffacts 'x' not found"
**Solution**: Register the deffacts before calling `load_deffacts_by_name()`

**Problem**: "Deffacts 'x' already exists"
**Solution**: Use `register_or_replace()` instead of `register()` to overwrite

---

**Last Updated**: 2025-11-06
**Version**: rust-rule-engine v0.13.0
**Features**: Template System, Defglobal, Deffacts, Test CE, Conflict Resolution Strategies
**Next Release**: v0.14.0 with Multi-field Variables, RETE Test CE integration

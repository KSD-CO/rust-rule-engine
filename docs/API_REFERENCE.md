# API Reference

Complete API reference for rust-rule-engine.

---

## Core Types

### RustRuleEngine

Main engine for Native mode.

```rust
pub struct RustRuleEngine { /* ... */ }

impl RustRuleEngine {
    // Creation
    pub fn new() -> Self
    
    // Rule Loading
    pub fn load_rules_from_file(&mut self, path: &str) -> Result<()>
    pub fn load_rules_from_string(&mut self, content: &str) -> Result<()>
    pub fn add_rule(&mut self, rule: Rule) -> Result<()>
    
    // Execution
    pub fn execute(&mut self, facts: &mut Facts) -> Result<usize>
    pub fn execute_with_limit(&mut self, facts: &mut Facts, max: usize) -> Result<usize>
    
    // Plugin Management
    pub fn load_plugin(&mut self, plugin: Box<dyn RulePlugin>) -> Result<()>
    pub fn load_default_plugins(&mut self) -> Result<()>
    pub fn unload_plugin(&mut self, name: &str) -> Result<()>
    
    // Query
    pub fn list_rules(&self) -> Vec<&Rule>
    pub fn get_rule(&self, name: &str) -> Option<&Rule>
    pub fn remove_rule(&mut self, name: &str) -> Result<()>
}
```

---

### Facts

Key-value storage for rule data.

```rust
pub struct Facts { /* ... */ }

impl Facts {
    pub fn new() -> Self
    pub fn set(&mut self, key: &str, value: Value)
    pub fn get(&self, key: &str) -> Option<&Value>
    pub fn has(&self, key: &str) -> bool
    pub fn remove(&mut self, key: &str) -> Option<Value>
    pub fn clear(&mut self)
    pub fn keys(&self) -> Vec<&String>
    pub fn len(&self) -> usize
    pub fn is_empty(&self) -> bool
}
```

---

### Value

Supported data types.

```rust
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}

impl Value {
    pub fn as_string(&self) -> Option<&String>
    pub fn as_integer(&self) -> Option<i64>
    pub fn as_float(&self) -> Option<f64>
    pub fn as_boolean(&self) -> Option<bool>
    pub fn as_array(&self) -> Option<&Vec<Value>>
    pub fn as_object(&self) -> Option<&HashMap<String, Value>>
}
```

---

## RETE Types

### IncrementalEngine

Main RETE-UL engine.

```rust
pub struct IncrementalEngine { /* ... */ }

impl IncrementalEngine {
    pub fn new() -> Self
    
    // Rule Management
    pub fn add_rule(&mut self, rule: TypedReteUlRule, depends_on: Vec<String>)
    
    // Fact Operations
    pub fn insert(&mut self, fact_type: String, data: TypedFacts) -> FactHandle
    pub fn update(&mut self, handle: FactHandle, data: TypedFacts) -> Result<(), String>
    pub fn retract(&mut self, handle: FactHandle) -> Result<(), String>
    
    // Template Operations (v0.10.0)
    pub fn templates(&self) -> &TemplateRegistry
    pub fn templates_mut(&mut self) -> &mut TemplateRegistry
    pub fn insert_with_template(&mut self, name: &str, data: TypedFacts) -> Result<FactHandle>
    
    // Global Variables (v0.10.0)
    pub fn globals(&self) -> &GlobalsRegistry
    pub fn globals_mut(&mut self) -> &mut GlobalsRegistry
    
    // Execution
    pub fn fire_all(&mut self) -> Vec<String>
    pub fn reset(&mut self)
    
    // Query
    pub fn working_memory(&self) -> &WorkingMemory
    pub fn working_memory_mut(&mut self) -> &mut WorkingMemory
    pub fn agenda(&self) -> &AdvancedAgenda
    pub fn stats(&self) -> IncrementalEngineStats
}
```

---

### TypedFacts

Strongly-typed facts for RETE.

```rust
pub struct TypedFacts { /* ... */ }

impl TypedFacts {
    pub fn new() -> Self
    pub fn set(&mut self, key: &str, value: FactValue)
    pub fn get(&self, key: &str) -> Option<&FactValue>
    pub fn has(&self, key: &str) -> bool
    pub fn remove(&mut self, key: &str) -> Option<FactValue>
}
```

---

### FactValue

RETE fact value types.

```rust
pub enum FactValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<FactValue>),
    Null,
}
```

---

### TemplateBuilder (v0.10.0)

Build type-safe templates.

```rust
pub struct TemplateBuilder { /* ... */ }

impl TemplateBuilder {
    pub fn new(name: impl Into<String>) -> Self
    
    // Field Definitions
    pub fn string_field(self, name: impl Into<String>) -> Self
    pub fn required_string(self, name: impl Into<String>) -> Self
    pub fn integer_field(self, name: impl Into<String>) -> Self
    pub fn float_field(self, name: impl Into<String>) -> Self
    pub fn boolean_field(self, name: impl Into<String>) -> Self
    pub fn array_field(self, name: impl Into<String>, element_type: FieldType) -> Self
    pub fn field_with_default(self, name: impl Into<String>, field_type: FieldType, default: FactValue) -> Self
    
    pub fn build(self) -> Template
}
```

---

### GlobalsRegistry (v0.10.0)

Global variables management.

```rust
pub struct GlobalsRegistry { /* ... */ }

impl GlobalsRegistry {
    pub fn new() -> Self
    
    // Define
    pub fn define(&self, name: impl Into<String>, value: FactValue) -> Result<()>
    pub fn define_readonly(&self, name: impl Into<String>, value: FactValue) -> Result<()>
    
    // Access
    pub fn get(&self, name: &str) -> Result<FactValue>
    pub fn set(&self, name: &str, value: FactValue) -> Result<()>
    pub fn exists(&self, name: &str) -> bool
    
    // Operations
    pub fn increment(&self, name: &str, delta: f64) -> Result<()>
    pub fn remove(&self, name: &str) -> Result<()>
    pub fn list_globals(&self) -> Vec<String>
    pub fn get_all(&self) -> HashMap<String, FactValue>
    pub fn clear(&self)
}
```

---

### GrlReteLoader

Load GRL files into RETE engine.

```rust
pub struct GrlReteLoader;

impl GrlReteLoader {
    pub fn load_from_file<P: AsRef<Path>>(path: P, engine: &mut IncrementalEngine) -> Result<usize>
    pub fn load_from_string(grl_text: &str, engine: &mut IncrementalEngine) -> Result<usize>
}
```

---

## Plugin Trait

### RulePlugin

Custom plugin interface.

```rust
pub trait RulePlugin: Send + Sync {
    // Required
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn description(&self) -> &str;
    
    // Optional
    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        Ok(())
    }
    
    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        Ok(())
    }
    
    fn on_load(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn on_unload(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn health_check(&self) -> Result<()> {
        Ok(())
    }
}
```

---

## Error Types

### RuleEngineError

Main error type.

```rust
pub enum RuleEngineError {
    ParseError { message: String },
    EvaluationError { message: String },
    FieldNotFound { field: String },
    IoError(std::io::Error),
    TypeMismatch { expected: String, actual: String },
    InvalidOperator { operator: String },
    InvalidLogicalOperator { operator: String },
    RegexError { message: String },
    ActionError { message: String },
    ExecutionError(String),
    SerializationError { message: String },
    PluginError { message: String },
}

pub type Result<T> = std::result::Result<T, RuleEngineError>;
```

---

## Feature Flags

Enable optional features in `Cargo.toml`:

```toml
[dependencies]
rust-rule-engine = { version = "0.10.0", features = ["async"] }
```

### Available Features
- `async` - Async support for rule execution
- `full` - All features enabled

---

## Examples

### Basic Usage

```rust
use rust_rule_engine::{RustRuleEngine, Facts, Value};

let mut engine = RustRuleEngine::new();
engine.load_rules_from_file("rules.grl")?;

let mut facts = Facts::new();
facts.set("amount", Value::Float(1000.0));

engine.execute(&mut facts)?;
```

### RETE Usage

```rust
use rust_rule_engine::rete::{
    IncrementalEngine, GrlReteLoader, TypedFacts, FactValue
};

let mut engine = IncrementalEngine::new();
GrlReteLoader::load_from_file("rules.grl", &mut engine)?;

let mut facts = TypedFacts::new();
facts.set("amount", FactValue::Float(1000.0));

engine.insert("Order".to_string(), facts);
engine.reset();
engine.fire_all();
```

### Template Usage (v0.10.0)

```rust
use rust_rule_engine::rete::{TemplateBuilder, FieldType};

let template = TemplateBuilder::new("Person")
    .required_string("name")
    .integer_field("age")
    .build();

engine.templates_mut().register(template);

let handle = engine.insert_with_template("Person", person_facts)?;
```

### Global Variables (v0.10.0)

```rust
engine.globals().define("counter", FactValue::Integer(0))?;
engine.globals().increment("counter", 1.0)?;
let count = engine.globals().get("counter")?;
```

---

## Constants

```rust
// Maximum iterations for cycle detection
pub const MAX_ITERATIONS: usize = 1000;

// Default salience
pub const DEFAULT_SALIENCE: i32 = 0;

// Default agenda group
pub const DEFAULT_AGENDA_GROUP: &str = "MAIN";
```

---

## Macros

```rust
// Create facts quickly
facts! {
    "customer.tier" => "gold",
    "order.amount" => 1500.0,
}

// Create typed facts
typed_facts! {
    "name" => FactValue::String("Alice".into()),
    "age" => FactValue::Integer(30),
}
```

---

**Full Documentation**: https://docs.rs/rust-rule-engine

**Examples**: [examples/](../../examples/) directory

**Last Updated**: 2025-10-31 (v0.10.0)

# Plugin System Guide

Complete guide to the plugin system in rust-rule-engine.

---

## Overview

The plugin system provides modular, extensible functionality through:
- **44+ built-in actions**
- **33+ built-in functions**
- **Easy custom plugin creation**
- **Hot-reload support** (future)
- **Health monitoring**

---

## Loading Plugins

### Load All Default Plugins
```rust
use rust_rule_engine::RustRuleEngine;

let mut engine = RustRuleEngine::new();
engine.load_default_plugins()?;
```

### Load Specific Plugins
```rust
engine.load_plugin(StringUtilitiesPlugin::new())?;
engine.load_plugin(MathOperationsPlugin::new())?;
```

---

## Built-in Plugins

### 1. String Utilities Plugin 📝

**Actions (13):**
- `ToUpper(field)` - Convert to uppercase
- `ToLower(field)` - Convert to lowercase
- `Trim(field)` - Remove whitespace
- `TrimLeft(field)` - Remove leading whitespace
- `TrimRight(field)` - Remove trailing whitespace
- `Replace(field, old, new)` - Replace substring
- `ReplaceAll(field, old, new)` - Replace all occurrences
- `Substring(field, start, length)` - Extract substring
- `Concat(field, ...values)` - Concatenate strings
- `Split(field, delimiter, output)` - Split string into array
- `PadLeft(field, length, char)` - Pad left
- `PadRight(field, length, char)` - Pad right
- `Reverse(field)` - Reverse string

**Functions (7):**
- `Length(string)` - String length
- `Contains(string, substring)` - Check substring
- `StartsWith(string, prefix)` - Check prefix
- `EndsWith(string, suffix)` - Check suffix
- `IndexOf(string, substring)` - Find position
- `IsEmpty(string)` - Check if empty
- `CharAt(string, index)` - Get character at position

**Example:**
```grl
rule "NormalizeEmail" {
    when user.email != ""
    then 
        ToLower(user.email);
        Trim(user.email);
}
```

---

### 2. Math Operations Plugin 🔢

**Actions (12):**
- `Increment(field, amount)` - Add to field
- `Decrement(field, amount)` - Subtract from field
- `Multiply(field, factor)` - Multiply field
- `Divide(field, divisor)` - Divide field
- `Mod(field, divisor)` - Modulo operation
- `Abs(field)` - Absolute value
- `Round(field, decimals)` - Round number
- `Ceil(field)` - Round up
- `Floor(field)` - Round down
- `Clamp(field, min, max)` - Constrain value
- `Pow(field, exponent)` - Power
- `Sqrt(field)` - Square root

**Functions (8):**
- `Max(a, b)` - Maximum value
- `Min(a, b)` - Minimum value
- `Avg(...values)` - Average
- `Sum(...values)` - Sum of values
- `Random()` - Random 0-1
- `RandomInt(min, max)` - Random integer
- `IsEven(number)` - Check if even
- `IsOdd(number)` - Check if odd

**Example:**
```grl
rule "ApplyDiscount" {
    when order.amount > 1000
    then
        Multiply(order.amount, 0.9);  // 10% discount
        Round(order.amount, 2);       // 2 decimals
}
```

---

### 3. Date/Time Plugin 📅

**Actions (8):**
- `SetNow(field)` - Set current timestamp
- `AddDays(field, days)` - Add days
- `AddHours(field, hours)` - Add hours
- `AddMinutes(field, minutes)` - Add minutes
- `FormatDate(field, format)` - Format date
- `ParseDate(field, format)` - Parse date string
- `SetDate(field, year, month, day)` - Set specific date
- `Truncate(field, unit)` - Truncate to unit

**Functions (6):**
- `Now()` - Current timestamp
- `DayOfWeek(date)` - Get day of week (1-7)
- `DayOfMonth(date)` - Get day of month
- `Month(date)` - Get month
- `Year(date)` - Get year
- `IsWeekend(date)` - Check if weekend

**Example:**
```grl
rule "SetExpiration" {
    when order.created_at != ""
    then
        SetNow(order.expires_at);
        AddDays(order.expires_at, 30);
}
```

---

### 4. Validation Plugin ✅

**Actions (6):**
- `ValidateEmail(field, error_field)` - Email validation
- `ValidateURL(field, error_field)` - URL validation
- `ValidateNumeric(field, error_field)` - Number validation
- `ValidateAlpha(field, error_field)` - Alphabetic validation
- `ValidateRange(field, min, max, error_field)` - Range check
- `ValidatePattern(field, pattern, error_field)` - Regex match

**Functions (6):**
- `IsEmail(string)` - Check if valid email
- `IsURL(string)` - Check if valid URL
- `IsNumeric(string)` - Check if numeric
- `IsAlpha(string)` - Check if alphabetic
- `InRange(value, min, max)` - Check range
- `MatchesPattern(string, pattern)` - Regex match

**Example:**
```grl
rule "ValidateUser" {
    when user.email != ""
    then
        ValidateEmail(user.email, user.errors);
        ValidateRange(user.age, 18, 120, user.errors);
}
```

---

### 5. Collection Operations Plugin 📋

**Actions (7):**
- `Append(array, value)` - Add to array
- `Remove(array, index)` - Remove from array
- `Sort(array)` - Sort array
- `Reverse(array)` - Reverse array
- `Filter(array, condition, output)` - Filter array
- `Map(array, operation, output)` - Transform array
- `Clear(array)` - Empty array

**Functions (6):**
- `Length(array)` - Array length
- `Contains(array, value)` - Check membership
- `IndexOf(array, value)` - Find index
- `First(array)` - Get first element
- `Last(array)` - Get last element
- `IsEmpty(array)` - Check if empty

**Example:**
```grl
rule "ProcessItems" {
    when order.items != []
    then
        Sort(order.items);
        Log("Processing " + Length(order.items) + " items");
}
```

---

## Creating Custom Plugins

### 1. Implement RulePlugin Trait

```rust
use rust_rule_engine::RulePlugin;

pub struct MyCustomPlugin {
    name: String,
}

impl MyCustomPlugin {
    pub fn new() -> Self {
        Self {
            name: "MyCustomPlugin".to_string(),
        }
    }
}

impl RulePlugin for MyCustomPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn description(&self) -> &str {
        "My custom plugin description"
    }

    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // Register your custom actions
        engine.register_action("MyAction", |facts, params| {
            // Action implementation
            Ok(())
        });
        Ok(())
    }

    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // Register your custom functions
        engine.register_function("MyFunction", |facts, params| {
            // Function implementation
            Ok(Value::String("result".to_string()))
        });
        Ok(())
    }

    fn on_load(&mut self) -> Result<()> {
        println!("Plugin loaded!");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<()> {
        println!("Plugin unloaded!");
        Ok(())
    }

    fn health_check(&self) -> Result<()> {
        // Health check logic
        Ok(())
    }
}
```

### 2. Load Custom Plugin

```rust
let mut engine = RustRuleEngine::new();
engine.load_plugin(MyCustomPlugin::new())?;
```

### 3. Use in Rules

```grl
rule "UseCustom" {
    when condition
    then MyAction(field, param);
}
```

---

## Plugin Lifecycle

```
Create → Load → Register → Active → Health Check → Unload
  ↓       ↓        ↓          ↓           ↓           ↓
 new()  on_load() register_*() (usage)  health_check() on_unload()
```

---

## Plugin Health Monitoring

```rust
// Check plugin health
let health = engine.plugin_health("StringUtilities")?;
match health {
    PluginHealth::Healthy => println!("✅ Plugin healthy"),
    PluginHealth::Warning(msg) => println!("⚠️ Warning: {}", msg),
    PluginHealth::Error(msg) => println!("❌ Error: {}", msg),
}

// Get all plugin stats
let stats = engine.plugin_stats();
for stat in stats {
    println!("{}: {} actions, {} functions", 
             stat.name, stat.action_count, stat.function_count);
}
```

---

## Best Practices

### 1. Keep Plugins Focused
```rust
// ✅ Good: Single responsibility
StringUtilitiesPlugin
MathOperationsPlugin

// ❌ Bad: Too broad
EverythingPlugin
```

### 2. Handle Errors Gracefully
```rust
impl RulePlugin for MyPlugin {
    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        engine.register_action("MyAction", |facts, params| {
            // ✅ Proper error handling
            let value = params.get(0)
                .ok_or_else(|| "Missing parameter")?;
            Ok(())
        });
        Ok(())
    }
}
```

### 3. Document Plugin Functions
```rust
impl RulePlugin for MyPlugin {
    fn description(&self) -> &str {
        "MyPlugin provides:
         - MyAction(field, param): Description
         - MyFunction(value): Description"
    }
}
```

### 4. Version Your Plugins
```rust
fn version(&self) -> &str {
    "1.0.0"  // Semantic versioning
}
```

---

## Plugin Examples

See [examples/](../../examples/) directory for:
- Custom plugin implementations
- Advanced plugin patterns
- Integration examples

---

**See Also:**
- [FEATURES.md](FEATURES.md) - Core features
- [API_REFERENCE.md](API_REFERENCE.md) - API documentation

**Last Updated**: 2025-10-31 (v0.10.0)

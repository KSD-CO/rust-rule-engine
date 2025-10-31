# Core Features Guide

Complete guide to all features in rust-rule-engine.

---

## Core Engine Features

### GRL Support
- Full Grule-compatible syntax
- File and string loading
- Automatic rule parsing
- Syntax validation

### Knowledge Base Management
- Centralized rule storage
- Salience-based priority
- Rule lifecycle management
- Dynamic rule updates

### Type Safety
- Rust's compile-time guarantees
- Strong typing for facts
- Type-safe rule execution
- Error handling at compile time

### Facts System
- Flexible key-value storage
- Nested object support
- Array handling
- Type coercion

---

## Advanced Pattern Matching

### EXISTS Pattern
Check if at least one fact matches:
```grl
when exists(Order.status == "pending")
then Alert.send("Pending orders found");
```

### NOT Pattern
Check if no facts match:
```grl
when !exists(Error.critical == true)
then System.status = "healthy";
```

### FORALL Pattern
Check if all facts match:
```grl
when forall(Item.validated == true)
then Order.ready = true;
```

### Complex Patterns
Combine with logical operators:
```grl
when 
    customer.tier == "gold" &&
    exists(Order.amount > 1000) &&
    !exists(Complaint.status == "open")
then
    customer.priority = "high";
```

---

## Rule Attributes

### Salience (Priority)
Control execution order:
```grl
rule "HighPriority" salience 100 { ... }
rule "Normal" salience 50 { ... }
rule "LowPriority" salience 1 { ... }
```

### No-Loop
Prevent infinite self-triggering:
```grl
rule "UpdateStatus" no-loop {
    when order.status == "pending"
    then order.status = "processed";
}
```

### Agenda Groups
Organize rules into phases:
```grl
rule "Validation" agenda-group "validate" { ... }
rule "Processing" agenda-group "process" { ... }
```

### Activation Groups
Mutually exclusive execution:
```grl
rule "Premium" activation-group "discount" salience 10 { ... }
rule "Standard" activation-group "discount" salience 5 { ... }
```

### Date Effective/Expires
Time-based activation:
```grl
rule "Holiday" 
    date-effective "2025-12-01" 
    date-expires "2025-12-31" { ... }
```

---

## Workflow Engine

### Scheduled Tasks
```rust
engine.schedule_task("cleanup", Duration::from_secs(3600));
```

### Workflow State Tracking
```rust
let state = engine.get_workflow_state("order_processing");
println!("Progress: {:.2}%", state.progress);
```

### Dynamic Rule Activation
```rust
engine.set_workflow_data("stage", "approval");
// Rules in "approval" agenda-group auto-activate
```

---

## Built-in Functions

See [PLUGINS.md](PLUGINS.md) for complete list.

### String Operations
- `ToUpper()`, `ToLower()`, `Trim()`
- `Replace()`, `Substring()`
- `Contains()`, `StartsWith()`, `EndsWith()`

### Math Operations
- `Abs()`, `Round()`, `Ceil()`, `Floor()`
- `Max()`, `Min()`, `Pow()`, `Sqrt()`
- `Random()`, `RandomRange()`

### Date/Time
- `Now()`, `AddDays()`, `AddHours()`
- `FormatDate()`, `ParseDate()`
- `DayOfWeek()`, `IsWeekend()`

### Validation
- `IsEmail()`, `IsURL()`, `IsNumeric()`
- `IsAlpha()`, `InRange()`
- `MatchesPattern()`

---

## Production Features

### REST API
Full web API with endpoints for:
- Rule management (CRUD)
- Fact insertion
- Rule execution
- Analytics queries

### Real-time Analytics
- Rule execution metrics
- Performance monitoring
- Coverage analysis
- Live dashboards

### Health Checks
- System health endpoints
- Plugin health monitoring
- Resource usage tracking

### Memory Management
- Automatic fact cleanup
- Configurable retention
- Memory usage limits

---

## Performance Features

### Cycle Detection
- Automatic infinite loop detection
- Configurable max iterations
- Clear error messages

### Parallel Processing
- Multi-threaded rule execution (optional)
- Thread-safe fact access
- Concurrent rule evaluation

### Optimization
- Lazy evaluation
- Short-circuit logic
- Efficient pattern matching

---

**See Also:**
- [PLUGINS.md](PLUGINS.md) - Plugin system details
- [ADVANCED_USAGE.md](ADVANCED_USAGE.md) - Complex patterns
- [RETE_GUIDE.md](RETE_GUIDE.md) - RETE engine features

**Last Updated**: 2025-10-31 (v0.10.0)

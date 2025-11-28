# Cyclic Import Detection

## Overview

The module system includes **automatic cyclic import detection** to prevent circular dependencies between modules. This feature ensures your module architecture remains clean and prevents infinite loops during module resolution.

## Why Cyclic Detection Matters

Circular dependencies cause several problems:

1. **Infinite Loops**: During module loading and visibility resolution
2. **Memory Issues**: Recursive traversal can exhaust memory
3. **Unpredictable Behavior**: Rules might fire in unexpected order
4. **Configuration Errors**: Hard to debug module configuration issues

The engine detects cycles **before** they become problems by validating imports at the time they're added.

## How It Works

The cyclic detection uses **Breadth-First Search (BFS)** over the import graph:

```
When you attempt: MOD_C imports from MOD_A
Current graph:   MOD_A -> MOD_B -> MOD_C

1. Check if path exists from MOD_A to MOD_C
2. BFS from MOD_A:
   - Visit MOD_B (imported by MOD_A)
   - Visit MOD_C (imported by MOD_B) ← FOUND!
3. Creating MOD_C -> MOD_A would complete cycle
4. ✗ Reject import with clear error message
```

## Detection Scenarios

### 1. Self-Import (Immediate Cycle)

```rust
let mut manager = ModuleManager::new();
manager.create_module("DATABASE").unwrap();

// ✗ Always rejected - a module cannot import from itself
manager.import_from("DATABASE", "DATABASE", ImportType::AllRules, "*")?;
// Error: "Cyclic import detected: DATABASE cannot import from itself"
```

### 2. Simple Cycle (A → B → A)

```rust
let mut manager = ModuleManager::new();
manager.create_module("PARSER").unwrap();
manager.create_module("TOKENIZER").unwrap();

// ✓ First import is allowed
manager.import_from("PARSER", "TOKENIZER", ImportType::AllRules, "*")?;

// ✗ Second import creates cycle
manager.import_from("TOKENIZER", "PARSER", ImportType::AllRules, "*")?;
// Error: "Cyclic import detected: PARSER -> TOKENIZER"
```

### 3. Complex Cycle (A → B → C → A)

```rust
let mut manager = ModuleManager::new();
manager.create_module("COMPILER").unwrap();
manager.create_module("OPTIMIZER").unwrap();
manager.create_module("CODEGEN").unwrap();

// Build chain: COMPILER -> OPTIMIZER -> CODEGEN
manager.import_from("COMPILER", "OPTIMIZER", ImportType::AllRules, "*")?;
manager.import_from("OPTIMIZER", "CODEGEN", ImportType::AllRules, "*")?;

// ✗ Completing the cycle is rejected
manager.import_from("CODEGEN", "COMPILER", ImportType::AllRules, "*")?;
// Error: "Cyclic import detected: COMPILER -> OPTIMIZER -> CODEGEN"
```

### 4. Valid Chains (No Cycles)

```rust
let mut manager = ModuleManager::new();
manager.create_module("SENSORS").unwrap();
manager.create_module("PROCESSOR").unwrap();
manager.create_module("ACTUATORS").unwrap();

// ✓ All these are allowed (no cycles created)
manager.import_from("SENSORS", "PROCESSOR", ImportType::AllRules, "*")?;
manager.import_from("PROCESSOR", "ACTUATORS", ImportType::AllRules, "*")?;
```

### 5. Diamond Dependency (Multiple Paths, No Cycle)

```rust
//     SERVICE_A
//        /  \
//    CORE    SERVICE_B
//        \  /
//      (connects but no cycle)

let mut manager = ModuleManager::new();
manager.create_module("CORE").unwrap();
manager.create_module("SERVICE_A").unwrap();
manager.create_module("SERVICE_B").unwrap();

// ✓ All allowed - no cycle exists
manager.import_from("SERVICE_A", "CORE", ImportType::AllRules, "*")?;
manager.import_from("SERVICE_B", "CORE", ImportType::AllRules, "*")?;
manager.import_from("SERVICE_A", "SERVICE_B", ImportType::AllRules, "*")?;

// ✗ This would create cycle: SERVICE_B -> SERVICE_A -> SERVICE_B
manager.import_from("SERVICE_B", "SERVICE_A", ImportType::AllRules, "*")?;
// Error: "Cyclic import detected: SERVICE_A -> SERVICE_B"
```

## API Reference

### Main Detection Method

The `import_from` method now validates cycles automatically:

```rust
pub fn import_from(
    &mut self,
    to_module: &str,      // Module that wants to import
    from_module: &str,    // Module to import from
    import_type: ImportType,
    pattern: &str
) -> Result<()>
```

**Returns:**
- `Ok(())` - Import added successfully, no cycle
- `Err(RuleEngineError::ModuleError { message })` - Cycle detected

**Error Message Format:**
```
"Cyclic import detected: MOD_A -> MOD_B -> MOD_C"
```

### Graph Inspection

#### Get Import Graph

```rust
pub fn get_import_graph(&self) -> &HashMap<String, HashSet<String>>
```

Returns the raw import graph where each module maps to the modules it imports from.

**Example:**
```rust
let graph = manager.get_import_graph();
// graph["MOD_A"] contains {"MOD_B", "MOD_C"} - A imports from B and C
```

#### Debug Representation

```rust
pub fn get_import_graph_debug(&self) -> Vec<(String, Vec<String>)>
```

Returns a human-readable representation of the import graph, useful for debugging and visualization.

**Example:**
```rust
let debug = manager.get_import_graph_debug();
// Output:
// [
//   ("SENSORS", ["HARDWARE_DRIVERS"]),
//   ("CONTROL", ["SENSORS", "UTILS"]),
// ]
```

## Best Practices

### 1. Design Layered Architecture

```
┌─────────────────────┐
│  APPLICATION        │  Layer 3: High-level logic
├─────────────────────┤
│  CONTROL_LOGIC      │  Layer 2: Core processing
├─────────────────────┤
│  SENSORS, DRIVERS   │  Layer 1: Low-level I/O
└─────────────────────┘

✓ Imports flow downward
✗ No upward imports
```

### 2. Use Utility Modules

```rust
// Create UTILS module for shared functionality
manager.create_module("UTILS")?;

// Multiple modules can import from UTILS
manager.import_from("CONTROL", "UTILS", ...)?;
manager.import_from("AUTOMATION", "UTILS", ...)?;
// ✓ No cycle created by sharing utilities
```

### 3. Check Graph Before Complex Operations

```rust
fn validate_architecture(manager: &ModuleManager) -> Result<()> {
    let graph = manager.get_import_graph_debug();
    
    for (module, imports) in graph {
        println!("Module {} imports from: {:?}", module, imports);
    }
    
    Ok(())
}
```

### 4. Plan Imports Carefully

Before adding imports, mentally trace the dependency graph:

```rust
// Good: Clear layer structure
manager.import_from("APP", "CONTROL", ...)?;        // OK
manager.import_from("CONTROL", "SENSORS", ...)?;    // OK
// Would fail: manager.import_from("SENSORS", "APP", ...)?;

// Bad: Confusing cross-layer imports
manager.import_from("APP", "CONTROL", ...)?;
manager.import_from("APP", "SENSORS", ...)?;        // Why both?
manager.import_from("SENSORS", "CONTROL", ...)?;    // Now confusing
```

## Error Messages

### Self-Import Error

```
Module error: Cyclic import detected: DATABASE cannot import from itself
```

**Fix:** Don't import a module from itself. Rules within a module are always visible to that module.

### Direct Cycle Error

```
Module error: Cyclic import detected: PARSER -> TOKENIZER
```

**Fix:** Remove one of the circular imports. Usually, you need to reverse one direction.

### Indirect Cycle Error

```
Module error: Cyclic import detected: COMPILER -> OPTIMIZER -> CODEGEN
```

**Fix:** Review the chain and break the cycle at the most appropriate point.

## Implementation Details

### Algorithm: BFS Cycle Detection

1. When `import_from("A", "B", ...)` is called:
2. Do BFS from module B
3. Check if we can reach module A
4. If reachable: Adding A→B would complete cycle ✗
5. If not reachable: Safe to add A→B ✓

**Time Complexity:** O(V + E) where V = modules, E = imports
**Space Complexity:** O(V) for visited set and queue

### Graph Maintenance

The import graph is maintained in the `ModuleManager`:

```rust
pub struct ModuleManager {
    modules: HashMap<String, Module>,
    import_graph: HashMap<String, HashSet<String>>,
    // ...
}
```

When a module is deleted, all references are cleaned up automatically.

## Testing

The feature is thoroughly tested with 13+ test cases:

- ✓ Self-import detection
- ✓ Simple cycles (A→B→A)
- ✓ Complex cycles (A→B→C→A)
- ✓ Longer chains (A→B→C→D)
- ✓ Valid chains allowed
- ✓ Multiple independent chains
- ✓ Diamond dependencies
- ✓ Graph tracking
- ✓ Module deletion cleanup
- ✓ Error message clarity

Run tests:
```bash
cargo test test_cycle_detection
```

## Performance

Cycle detection adds minimal overhead:

- **Per import:** O(V + E) BFS traversal
- **Typical case:** < 1ms for 100 modules
- **Memory:** One import graph entry per module pair

For most applications with < 100 modules, the overhead is negligible.

## Real-World Example

See `examples/10-module-system/cyclic_import_detection.rs` for:

1. Self-import detection demo
2. Simple cycle examples
3. Complex cycle scenarios
4. Valid chain demonstrations
5. Diamond dependency patterns
6. Smart home system architecture

Run it:
```bash
cargo run --example cyclic_import_detection
```

## Migration Guide

If you have existing code that might have cycles:

### Step 1: Enable Cycle Checking

```rust
let mut manager = ModuleManager::new();
// All existing code now validates for cycles
```

### Step 2: Handle Errors

```rust
match manager.import_from("A", "B", ImportType::AllRules, "*") {
    Ok(_) => println!("Import successful"),
    Err(e) => {
        eprintln!("Cycle detected: {}", e);
        // Fix: redesign module architecture
    }
}
```

### Step 3: Redesign if Needed

If cycles are detected:
1. Review module dependencies
2. Identify circular dependencies
3. Refactor into layers
4. Test with cycle detection enabled

## See Also

- [Module System Guide](MODULE_PARSING_GUIDE.md)
- [Module Refactoring](MODULE_REFACTORING.md)
- [API Reference](API_REFERENCE.md)
- [Examples](../examples/10-module-system/)

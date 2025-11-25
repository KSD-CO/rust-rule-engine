# Advanced Features Examples

Advanced features of the Rust Rule Engine.

## Example List

### Accumulate Functions
- **accumulate_demo.rs** - Manual accumulate functions
- **accumulate_grl_demo.rs** - Accumulate with GRL syntax

### Multifield Operations
- **multifield_operations_demo.rs** - Operations with multifield variables
- **pattern_matching_from_grl.rs** - Advanced pattern matching

### Action Handlers
- **action_handlers_demo.rs** - Custom action handlers
- **action_handlers_grl_demo.rs** - Action handlers with GRL

### Functions & Templates
- **custom_functions_demo.rs** - Define custom functions
- **rule_templates_demo.rs** - Using rule templates

### Rule Control
- **conflict_resolution_demo.rs** - Conflict resolution strategies
- **rule_attributes_demo.rs** - Rule attributes (salience, no-loop, etc.)
- **no_loop_demo.rs** - Using no-loop attribute
- **grl_no_loop_demo.rs** - No-loop with GRL syntax

### Fact Management
- **retract_demo.rs** - Retract facts in native engine
- **retract_demo_rete.rs** - Retract facts in RETE engine

## How to run

```bash
cargo run --example accumulate_demo
cargo run --example conflict_resolution_demo
# ... other examples
```

## Note

These examples require basic knowledge of rule engines.
Please see the `01-getting-started` directory first if you're not familiar.

# Advanced Features Examples

Advanced features of the Rust Rule Engine.

## Example List

### Streaming + Rule Engine (NEW!)
- **streaming_with_rules_demo.rs** - Streaming operators + Rule Engine integration
  - Fraud detection with rules from GRL files
  - Dynamic pricing with rule evaluation
  - Compliance checking with streaming events
  
- **streaming_state_management_demo.rs** - Stateful stream processing
  - Session tracking with StateStore
  - Aggregation with threshold alerts
  - Stateful fraud detection
  
- **streaming_watermark_demo.rs** - Watermark-based processing
  - Late data detection and handling
  - Out-of-order event processing
  - Time window aggregation with alerts

Run with: `cargo run --example streaming_with_rules_demo --features streaming`

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
# Streaming examples (require --features streaming)
cargo run --example streaming_with_rules_demo --features streaming
cargo run --example streaming_state_management_demo --features streaming
cargo run --example streaming_watermark_demo --features streaming

# Other advanced examples
cargo run --example accumulate_demo
cargo run --example conflict_resolution_demo
# ... other examples
```

## Note

These examples require basic knowledge of rule engines.
Please see the `01-getting-started` directory first if you're not familiar.

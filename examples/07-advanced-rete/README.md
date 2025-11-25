# Advanced RETE Examples

Advanced features of the RETE-UL engine.

## Example List

### Working Memory & Agenda
- **rete_p2_working_memory.rs** - Working memory management
- **rete_p2_advanced_agenda.rs** - Advanced agenda control

### Incremental Processing
- **rete_p3_incremental.rs** - Incremental fact propagation
- **rete_p3_variable_binding.rs** - Advanced variable binding

### Optimization
- **rete_engine_cached.rs** - Cached RETE engine
- **rete_memoization_demo.rs** - Memoization techniques

### Integration
- **rete_ul_drools_style.rs** - Drools-style syntax compatibility
- **accumulate_rete_integration.rs** - Accumulate functions in RETE

## RETE Architecture

```
Facts → Alpha Network → Beta Network → Agenda → Actions
         (Filter)       (Join)         (Sort)    (Execute)
```

### Components

1. **Alpha Network**: Filters facts based on conditions
2. **Beta Network**: Joins facts according to patterns
3. **Working Memory**: Stores facts and partial matches
4. **Agenda**: Sorts activated rules by priority
5. **Conflict Resolution**: Selects rule to execute

## Advanced Features

### Incremental Processing
- Only re-evaluate affected rules when facts change
- Efficient with large rulesets
- Maintains partial matches

### Memoization
- Cache computation results
- Avoid redundant calculations
- Configurable cache strategies

### Advanced Agenda Control
- Custom conflict resolution
- Dynamic salience
- Rule flow control

## Performance Considerations

1. **Network Size**: Optimize number of nodes in alpha/beta network
2. **Memory Usage**: Balance between speed and memory
3. **Cache Strategy**: Choose appropriate memoization strategy
4. **Fact Indexing**: Index facts according to access patterns

## How to run

```bash
cargo run --example rete_p2_working_memory
cargo run --example rete_p3_incremental
cargo run --release --example rete_engine_cached
```

## When to use Advanced RETE?

- Very large rulesets (1000+ rules)
- Complex pattern matching
- High-frequency fact updates
- Need maximum performance
- Complex conflict resolution requirements

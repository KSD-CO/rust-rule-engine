# RETE-UL Engine Examples

Examples using the RETE-UL algorithm (IncrementalEngine) - high-performance engine with incremental pattern matching.

## Example List

- **rete_demo.rs** - Basic RETE engine demo
- **rete_grl_demo.rs** - Load and run GRL rules with RETE
- **rete_parse_demo.rs** - RETE rules parsing demo
- **rete_call_function_demo.rs** - Calling functions in RETE rules
- **rete_multifield_demo.rs** - Using multifield variables
- **multifield_demo.rs** - Multifield operations in RETE
- **rete_typed_facts_demo.rs** - Typed facts with RETE
- **rete_deffacts_demo.rs** - Using deffacts in RETE
- **rete_template_globals_demo.rs** - Templates and global variables
- **rete_memoization_demo.rs** - Memoization for performance optimization
- **tms_demo.rs** - Truth Maintenance System

## What is RETE?

RETE is a pattern matching algorithm optimized for rule engines:
- Incremental matching: only re-evaluate rules affected by changes
- High performance with large numbers of rules
- Shares computation between similar rules

## When to use RETE?

- Have many rules (>100)
- Facts change frequently
- Need high performance
- Have many similar patterns

## How to run

```bash
cargo run --example rete_demo
cargo run --example rete_grl_demo
# ... other examples
```

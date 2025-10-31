# RETE Rule DSL & Auto-Network Design

## Goal
- Allow users to define rules in a simple DSL or structured format (struct, JSON, YAML, etc.)
- Engine automatically parses and converts rules to RETE node network
- No manual node construction required

## Design

### 1. Rule Definition Format
- Example struct:
  ```rust
  struct Rule {
      name: String,
      conditions: ConditionGroup,
      action: String,
  }
  
  enum ConditionGroup {
      Single(Condition),
      Compound { left: Box<ConditionGroup>, operator: String, right: Box<ConditionGroup> },
      Not(Box<ConditionGroup>),
      Exists(Box<ConditionGroup>),
      Forall(Box<ConditionGroup>),
  }
  
  struct Condition {
      field: String,
      operator: String,
      value: String,
  }
  ```
- Or DSL/JSON/YAML for external config

### 2. Parser/Converter
- Function: `build_rete_from_rule(rule: &Rule) -> ReteNode`
- Recursively convert `ConditionGroup` to RETE node network
- Attach rule name/action to Terminal node

### 3. Usage Flow
1. User defines rules in DSL/struct
2. Engine parses rules, builds RETE network
3. Facts are propagated, matching rules fire

### 4. Example
```rust
let rule = Rule {
    name: "ActiveUser",
    conditions: ConditionGroup::Compound {
        left: Box::new(ConditionGroup::Single(Condition { field: "status", operator: "==", value: "active" })),
        operator: "AND".to_string(),
        right: Box::new(ConditionGroup::Single(Condition { field: "age", operator: ">", value: "18" })),
    },
    action: "notify".to_string(),
};
let rete_node = build_rete_from_rule(&rule);
```

## Implementation Steps
1. Define Rule/Condition structs and enums
2. Implement parser: `build_rete_from_rule`
3. Update engine to accept rules, build network automatically
4. Add example/test for auto-conversion

## Benefits
- User-friendly, no manual node creation
- Flexible: support for DSL, struct, JSON, YAML
- Easy to extend for new logic

---

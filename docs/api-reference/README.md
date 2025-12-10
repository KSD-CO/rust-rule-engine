# API Reference

Complete API documentation for Rust Rule Engine.

---

## 📚 Available Documentation

### [API Reference](API_REFERENCE.md)
Complete public API reference.

**Coverage:**
- Engine API
- Facts API
- Rules API
- Knowledge Base API
- All public types and methods

**Use when:** Looking up specific API methods

### [GRL Query Syntax](GRL_QUERY_SYNTAX.md)
Complete reference for backward chaining query language.

**Coverage:**
- Query syntax
- Nested queries (v1.11.0+)
- Query optimization (v1.11.0+)
- Aggregation functions
- Negation and disjunction
- Action handlers
- All query features

**Use when:** Writing backward chaining queries

### [Parser Cheat Sheet](PARSER_CHEAT_SHEET.md)
Quick reference for GRL parser.

**Coverage:**
- Common syntax patterns
- Parser rules
- Quick examples
- Troubleshooting

**Use when:** Need quick syntax lookup

---

## 🎯 Find What You Need

### By Feature

**Forward Chaining**
→ [API Reference](API_REFERENCE.md) - Engine, Facts, Rules

**Backward Chaining**
→ [GRL Query Syntax](GRL_QUERY_SYNTAX.md) - Queries, Goals, Proofs

**Parsing**
→ [Parser Cheat Sheet](PARSER_CHEAT_SHEET.md) - GRL syntax

### By Task

**Creating an Engine**
```rust
use rust_rule_engine::Engine;
let engine = Engine::new();
```
→ [API Reference - Engine](API_REFERENCE.md#engine-api)

**Adding Rules**
```rust
engine.add_rule_from_string(grl_string)?;
```
→ [API Reference - Rules](API_REFERENCE.md#rules-api)

**Writing Queries**
```grl
query "MyQuery" {
    goal: pattern WHERE subquery
    enable-optimization: true
}
```
→ [GRL Query Syntax](GRL_QUERY_SYNTAX.md)

**Parsing GRL**
→ [Parser Cheat Sheet](PARSER_CHEAT_SHEET.md)

---

## 📖 Version History

| Version | Features | Documentation |
|---------|----------|---------------|
| **1.11.0** | Nested queries, Query optimization | [GRL Query Syntax](GRL_QUERY_SYNTAX.md) |
| **1.10.0** | Disjunction (OR) | [GRL Query Syntax](GRL_QUERY_SYNTAX.md#disjunction) |
| **1.9.0** | Explanation system | [GRL Query Syntax](GRL_QUERY_SYNTAX.md#explanation-system-v190) |
| **1.8.0** | Negation (NOT) | [GRL Query Syntax](GRL_QUERY_SYNTAX.md#negation-not-keyword) |
| **1.7.0** | Aggregation functions | [GRL Query Syntax](GRL_QUERY_SYNTAX.md#aggregation-functions) |

---

## 🔗 Related Documentation

- **[Getting Started](../getting-started/QUICK_START.md)** - Quick start guide
- **[Core Features](../core-features/)** - GRL syntax and features
- **[Guides](../guides/)** - How-to guides and best practices

---

## 📞 Need Help?

- **Can't find an API?** → Check [API Reference](API_REFERENCE.md)
- **Query not working?** → See [GRL Query Syntax](GRL_QUERY_SYNTAX.md)
- **Parser error?** → Use [Parser Cheat Sheet](PARSER_CHEAT_SHEET.md)
- **Still stuck?** → [Troubleshooting Guide](../guides/TROUBLESHOOTING.md)

---

## Navigation

📚 **[Documentation Home](../README.md)** | 📖 **[Getting Started](../getting-started/QUICK_START.md)**

# Changelog v1.8.0 - Negation in Backward Chaining

**Release Date:** December 3, 2025

## 🚫 Overview

Version 1.8.0 introduces **negation support** with the NOT keyword for backward chaining queries! This enables powerful absence checks and negative conditions using closed-world assumption semantics.

---

## ✨ New Features

### 1. NOT Keyword in Query Parser

**Description:** Query parser now supports NOT keyword prefix for negated goals.

**Syntax:**
```rust
let result = engine.query("NOT User.IsBanned == true", &mut facts)?;
```

**Implementation:**
- Added negation detection in `QueryParser::parse()` ([src/backward/query.rs:209-229](src/backward/query.rs#L209-L229))
- Parser strips "NOT " prefix and parses inner expression
- Creates negated Goal with `is_negated: true` flag

**Tests:**
- `test_query_parser_not_keyword` - Basic NOT parsing
- `test_query_parser_not_keyword_with_spaces` - Whitespace handling
- `test_query_parser_normal_query_not_negated` - Non-negated queries
- `test_query_parser_not_complex_expression` - Complex expressions with NOT

### 2. Negated Goal Support

**Description:** Extended Goal struct to support negation flag and semantics.

**Changes:**
- Added `is_negated: bool` field to Goal struct ([src/backward/goal.rs:34](src/backward/goal.rs#L34))
- Added `Goal::negated()` constructor ([src/backward/goal.rs:83-94](src/backward/goal.rs#L83-L94))
- Added `Goal::negated_with_expression()` constructor ([src/backward/goal.rs:97-108](src/backward/goal.rs#L97-L108))

**Backward Compatibility:**
- All existing constructors default `is_negated` to `false`
- Zero breaking changes - existing code works unchanged

**Tests:**
- `test_negated_goal` - Negated goal creation
- `test_negated_goal_with_expression` - Negated goal with expression
- `test_normal_goal_not_negated` - Normal goals not negated

### 3. Closed-World Assumption

**Description:** Implemented negation-as-failure semantics in backward chaining search.

**Logic:**
```rust
// For negated goals:
// - If fact IS proven → NOT fails
// - If fact CANNOT be proven → NOT succeeds (closed-world)
```

**Implementation:**
- Modified `search_recursive_with_execution()` to handle negated goals ([src/backward/search.rs](src/backward/search.rs))
- Updated `check_goal_in_facts()` to correctly evaluate negated goals
- Inverts success/failure for negated goals at end of search

**Semantics:**
1. **Explicit FALSE**: `User.IsBanned = false` → `NOT User.IsBanned == true` ✅ succeeds
2. **Missing Field**: No `User.IsBanned` field → `NOT User.IsBanned == true` ✅ succeeds (closed-world)
3. **Explicit TRUE**: `User.IsBanned = true` → `NOT User.IsBanned == true` ❌ fails

### 4. GRL File Support

**Description:** GRL files can now use NOT keyword in query goal definitions.

**GRL Syntax:**
```grl
query "NotBannedUsers" {
    goal: NOT User.IsBanned == true
    on-success: {
        User.Allowed = true;
        LogMessage("User is not banned");
    }
}

query "AvailableItems" {
    goal: NOT Item.Sold == true
    on-success: {
        Item.Available = true;
    }
}
```

**Files:**
- [examples/rules/09-backward-chaining/negation_queries.grl](examples/rules/09-backward-chaining/negation_queries.grl) - 8 query examples

---

## 📊 Examples & Demos

### 1. Negation Demo (Rust)

**File:** [examples/09-backward-chaining/negation_demo.rs](examples/09-backward-chaining/negation_demo.rs)

**Scenarios:**
1. **Simple NOT** - Check if user is NOT banned
2. **Available Items** - Items NOT sold (closed-world)
3. **Eligible Users** - Active AND NOT banned
4. **Closed-World Assumption** - Non-members

**Run:**
```bash
cargo run --features backward-chaining --example negation_demo
```

### 2. GRL Negation Demo

**File:** [examples/09-backward-chaining/grl_negation_demo.rs](examples/09-backward-chaining/grl_negation_demo.rs)

**Scenarios:**
1. **Programmatic GRL Queries** - Creating NOT queries in code
2. **E-commerce Order Approval** - Orders NOT requiring approval
3. **User Access Control** - Users NOT banned
4. **Inventory Availability** - Items NOT reserved

**Run:**
```bash
cargo run --features backward-chaining --example grl_negation_demo
```

### 3. GRL Query File

**File:** [examples/rules/09-backward-chaining/negation_queries.grl](examples/rules/09-backward-chaining/negation_queries.grl)

**Queries:**
- NotBannedUsers
- AvailableItems
- NonManagementEmployees
- AutoApprovedOrders
- RegularCustomers (non-VIP)
- FullPriceProducts (not discounted)
- ActiveAccounts (not expired)

---

## 📚 Documentation Updates

### 1. GRL Query Syntax Guide

**File:** [docs/GRL_QUERY_SYNTAX.md](docs/GRL_QUERY_SYNTAX.md)

**Added Section:**
- "Negation (NOT Keyword)" with comprehensive examples
- Negation semantics explanation (explicit FALSE, missing field, explicit TRUE)
- Real-world use cases (e-commerce, access control, inventory)
- Closed-world assumption details

### 2. README Updates

**File:** [README.md](README.md)

**Changes:**
- Updated version from v1.7.0 to v1.8.0
- Added "What's New in v1.8.0" section with negation features
- Updated installation instructions to v1.8.0
- Added negation to version history
- Included negation examples in Rust and GRL

---

## 🔧 Technical Changes

### Modified Files

1. **src/backward/goal.rs**
   - Added `is_negated: bool` field to Goal struct
   - Added `Goal::negated()` constructor
   - Added `Goal::negated_with_expression()` constructor
   - Updated all existing constructors to initialize `is_negated: false`
   - Added 3 unit tests for negated goals

2. **src/backward/query.rs**
   - Modified `QueryParser::parse()` to detect "NOT " prefix
   - Strip NOT prefix and parse inner expression
   - Create negated goals when NOT detected
   - Added 4 unit tests for NOT parsing

3. **src/backward/search.rs**
   - Modified `search_recursive_with_execution()` to handle negated goals
   - Updated `check_goal_in_facts()` to evaluate negated goals correctly
   - Implemented closed-world assumption logic
   - Inverts success/failure for negated goals

4. **Cargo.toml**
   - Updated version from 1.7.0 to 1.8.0
   - Added `negation_demo` example
   - Added `grl_negation_demo` example

5. **Makefile**
   - Added `negation_demo` to backward-chaining target
   - Added `grl_negation_demo` to backward-chaining target
   - Added individual targets: `negation_demo` and `grl_negation_demo`

### New Files

1. **examples/09-backward-chaining/negation_demo.rs** (236 lines)
   - 4 comprehensive scenarios demonstrating NOT keyword
   - Clear output showing negation semantics
   - Real-world use cases

2. **examples/09-backward-chaining/grl_negation_demo.rs** (327 lines)
   - Programmatic GRL query creation
   - 3 real-world scenarios (e-commerce, access control, inventory)
   - Integration with GRL query system

3. **examples/rules/09-backward-chaining/negation_queries.grl** (75 lines)
   - 8 example queries using NOT keyword
   - Various domains (users, items, employees, orders)
   - Production-ready query patterns

4. **CHANGELOG_v1.8.0.md** (this file)
   - Complete changelog for v1.8.0 release

---

## ✅ Testing

### Unit Tests

**All 284+ existing tests pass** - Zero breaking changes!

**New Tests (7 total):**

1. **src/backward/goal.rs:**
   - `test_negated_goal`
   - `test_negated_goal_with_expression`
   - `test_normal_goal_not_negated`

2. **src/backward/query.rs:**
   - `test_query_parser_not_keyword`
   - `test_query_parser_not_keyword_with_spaces`
   - `test_query_parser_normal_query_not_negated`
   - `test_query_parser_not_complex_expression`

### Integration Tests

**Run all tests:**
```bash
cargo test --all-features
```

**Run negation demos:**
```bash
make negation_demo
make grl_negation_demo
```

**Run full backward-chaining suite:**
```bash
make backward-chaining
```

---

## 🔄 Migration Guide

### From v1.7.0 to v1.8.0

**Good news:** No migration needed! v1.8.0 is 100% backward compatible.

**All existing code continues to work:**
```rust
// v1.7.0 code - still works in v1.8.0 ✅
let result = engine.query("User.IsVIP == true", &mut facts)?;
```

**New NOT feature is opt-in:**
```rust
// v1.8.0 new feature - use when needed ✅
let result = engine.query("NOT User.IsBanned == true", &mut facts)?;
```

**Zero Breaking Changes:**
- All constructors backward compatible
- All APIs unchanged
- All tests pass without modifications

---

## 📊 Use Cases

### 1. Access Control

**Check if user is NOT banned:**
```rust
let result = engine.query("NOT User.IsBanned == true", &mut facts)?;
if result.provable {
    // User is allowed to access
}
```

### 2. Inventory Management

**Find items that are NOT sold:**
```rust
let result = engine.query("NOT Item.Sold == true", &mut facts)?;
if result.provable {
    // Item is available for purchase
}
```

### 3. Order Processing

**Auto-approve orders that do NOT require manual review:**
```rust
let result = engine.query("NOT Order.RequiresApproval == true", &mut facts)?;
if result.provable {
    // Process order automatically
}
```

### 4. User Eligibility

**Check if account is NOT expired:**
```rust
let active = engine.query("User.IsActive == true", &mut facts)?;
let not_expired = engine.query("NOT Account.Expired == true", &mut facts)?;

if active.provable && not_expired.provable {
    // Account is eligible
}
```

---

## 🎯 Future Enhancements

### Planned for Future Releases:

1. **AND NOT in Complex Expressions**
   - Currently: NOT must be at beginning of query
   - Future: Support `User.IsActive == true AND NOT User.IsBanned == true`

2. **NOT with OR**
   - Support: `NOT (User.IsBanned == true OR User.IsSuspended == true)`

3. **Nested NOT**
   - Support: `NOT NOT User.IsVIP == true` (double negation)

4. **Performance Optimization**
   - Early termination for negated goals
   - Caching of negation results

---

## 📝 Notes

### Design Decisions

1. **NOT Keyword Prefix**
   - Simple, clear syntax
   - Easy to parse and implement
   - Familiar to CLIPS/Prolog users

2. **Closed-World Assumption**
   - Industry-standard semantics
   - Predictable behavior
   - Matches CLIPS and Prolog

3. **Non-Breaking Implementation**
   - Added `is_negated` field (default false)
   - No changes to existing APIs
   - All existing tests pass unchanged

### Known Limitations

1. **NOT Position**
   - Must be at beginning of query
   - Cannot use in middle of complex expressions (yet)
   - Example: `User.IsActive == true AND NOT User.IsBanned == true` - not supported yet

2. **No Nested NOT**
   - Currently no support for `NOT NOT goal`
   - Will be added in future release

### Performance

- **Query Parsing:** No measurable impact (<1µs difference)
- **Goal Evaluation:** Minimal overhead for negation check
- **Search:** Same performance as positive goals

---

## 👥 Contributors

- **Implementation:** Ton That Vu
- **Testing:** Comprehensive test suite
- **Documentation:** Complete guides and examples

---

## 🔗 Links

- **GitHub:** https://github.com/KSD-CO/rust-rule-engine
- **Crates.io:** https://crates.io/crates/rust-rule-engine
- **Documentation:** https://docs.rs/rust-rule-engine

---

## 📧 Support

- **Issues:** https://github.com/KSD-CO/rust-rule-engine/issues
- **Discussions:** https://github.com/KSD-CO/rust-rule-engine/discussions
- **Email:** ttvuhm@gmail.com

---

**Made with ❤️ by Ton That Vu**

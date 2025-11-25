# Backward Chaining Rules

This directory contains GRL rule files for backward chaining examples.

## Files

### ecommerce_approval.grl
Business rules for e-commerce order approval system:
- VIP customer rules (loyalty points, yearly spending)
- Small order auto-approval rules
- Risk assessment rules
- Auto approval logic
- Batch processing rules

**Used by:** `examples/09-backward-chaining/ecommerce_approval_demo.rs`

### ecommerce_queries.grl
Backward chaining queries for order approval:
- `CheckAutoApproval` - Main approval query with compound goals
- `CheckVIPStatus` - VIP customer verification
- `AssessOrderRisk` - Risk level assessment
- `CheckFastTrack` - Fast track processing eligibility
- `VerifyPayment` - Payment verification

**Used by:** `examples/09-backward-chaining/ecommerce_approval_demo.rs`

### purchasing_flow.grl
Business rules for purchasing and reorder system:
- Inventory shortage calculation
- Supplier validation (active/inactive)
- Order quantity logic (MOQ handling)
- Pricing and tax calculation
- Approval rules (high-value orders)
- Bulk discount application
- Purchase order creation logic

**Used by:** `examples/09-backward-chaining/purchasing_flow_demo.rs`

### purchasing_queries.grl
Backward chaining queries for purchasing decisions:
- `ShouldCreatePO` - Should a purchase order be created?
- `IsReorderNeeded` - Is reorder needed based on inventory?
- `IsOrderApproved` - Is the order auto-approved?
- `ShouldSendPO` - Should PO be sent to supplier?
- `IsSupplierValid` - Is supplier active and valid?
- `QualifiesForDiscount` - Does order qualify for bulk discount?

**Used by:** `examples/09-backward-chaining/purchasing_flow_demo.rs`

## Usage

These rule files are loaded by the backward chaining examples:

```rust
let rules = load_rules_from_file("ecommerce_approval.grl");
let query = load_query_from_file("ecommerce_queries.grl", "CheckAutoApproval");
```

## Note

⚠️ **ALPHA VERSION**: These rules are part of the backward chaining feature which is currently in alpha stage and not production-ready.

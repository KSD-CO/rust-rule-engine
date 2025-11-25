# Real-World Use Cases

Production-ready rule files for real business scenarios.

## Files

### 1. purchasing_rules.grl
**15 rules** - Complete purchasing workflow

**Business logic:**
- Volume discounts (tiered)
- Seasonal pricing
- Payment term validation
- Credit limit checks
- Tax calculations
- Shipping costs
- Total computation
- Approval workflows

**Example:**
```grl
rule "VolumeDiscount" salience 100 {
    when
        Purchase.Quantity > 100 && Purchase.UnitPrice > 0
    then
        Purchase.Discount = 15;
        Log("Applied 15% volume discount");
}

rule "CreditLimitCheck" salience 90 {
    when
        Purchase.Total > Customer.CreditLimit
    then
        Purchase.RequiresApproval = true;
        NotifyManager(Customer.Id);
}
```

**Use for:**
- E-commerce platforms
- B2B ordering systems
- Procurement automation
- Price calculation engines

### 2. fraud_detection.grl
**6 rules** - Multi-tier fraud detection

**Detection rules:**
- High-value transaction alerts
- Multiple failed attempts
- Geographic anomalies
- Unusual purchase patterns
- Velocity checks
- Account age verification

**Risk levels:**
- LOW: Normal transactions
- MEDIUM: Requires review
- HIGH: Auto-block
- CRITICAL: Immediate investigation

**Example:**
```grl
rule "HighValueAlert" salience 100 {
    when
        Transaction.Amount > 10000 &&
        Customer.AccountAge < 30
    then
        Transaction.RiskLevel = "HIGH";
        Transaction.RequiresManualReview = true;
        SendAlert("High-value transaction from new account");
}

rule "MultipleFailures" salience 90 {
    when
        Customer.FailedAttempts > 3 &&
        Customer.TimeWindow < 3600  // 1 hour
    then
        Customer.Status = "BLOCKED";
        NotifySecurityTeam(Customer.Id);
}
```

**Use for:**
- Payment gateways
- Banking systems
- E-commerce fraud prevention
- Account security

### 3. sales_analytics.grl
**9 rules** - Sales analytics and reporting

**Analytics:**
- Performance metrics calculation
- Tier classification (Gold, Silver, Bronze)
- Commission calculations
- Quota tracking
- Bonus eligibility
- Trend analysis
- KPI monitoring

**Example:**
```grl
rule "GoldTierCustomer" salience 80 {
    when
        Customer.AnnualSpending > 100000 &&
        Customer.OrderCount > 50
    then
        Customer.Tier = "GOLD";
        Customer.DiscountRate = 20;
        Customer.PrioritySupport = true;
}

rule "CalculateCommission" salience 50 {
    when
        Sale.Amount > 0 &&
        SalesPerson.CommissionRate > 0
    then
        Sale.Commission = Sale.Amount * SalesPerson.CommissionRate;
        SalesPerson.TotalCommission += Sale.Commission;
}
```

**Use for:**
- CRM systems
- Sales dashboards
- Performance tracking
- Incentive calculations

### 4. car_functions.grl
**5 rules** - Automotive control system

**Features:**
- Speed-based actions
- Safety checks
- Automatic responses
- Condition monitoring
- Warning systems

**Example:**
```grl
rule "HighSpeedWarning" salience 100 {
    when
        Car.Speed > 120 && Car.Location == "RESIDENTIAL"
    then
        Car.DisplayWarning("Speed limit exceeded");
        Car.ActivateSpeedLimiter();
        LogEvent("SPEED_WARNING", Car.Id);
}

rule "LowFuelAlert" salience 90 {
    when
        Car.FuelLevel < 10 &&
        Car.DistanceToStation > 5
    then
        Car.DisplayWarning("Low fuel - find gas station");
        Car.NavigateToNearestStation();
}
```

**Use for:**
- Automotive software
- Fleet management
- Driver assistance systems
- Vehicle diagnostics

### 5. complete_speedup.grl
**3 rules** - Performance optimization demo

**Optimizations:**
- Efficient pattern matching
- Minimal computations
- Indexed lookups
- Salience-based execution

**Example:**
```grl
rule "FastCheck" salience 100 {
    when
        Data.Status == "READY" &&
        Data.Id > 0
    then
        ProcessData(Data);
}
```

**Use for:**
- High-throughput systems
- Real-time processing
- Performance benchmarking
- Optimization techniques

## Business Domains

### E-Commerce & Retail
- `purchasing_rules.grl` - Order processing
- Dynamic pricing
- Inventory management
- Customer segmentation

### Finance & Banking
- `fraud_detection.grl` - Risk assessment
- Credit scoring
- Compliance checks
- Transaction monitoring

### Sales & Marketing
- `sales_analytics.grl` - Performance tracking
- Customer classification
- Campaign management
- Lead scoring

### Automotive & IoT
- `car_functions.grl` - Device control
- Sensor monitoring
- Automated responses
- Predictive maintenance

## Integration Patterns

### 1. Real-Time Processing
```rust
// Load production rules
let engine = RuleEngineBuilder::new()
    .add_grl_file("rules/04-use-cases/fraud_detection.grl")?
    .build()?;

// Process transaction
engine.add_fact("Transaction", transaction)?;
engine.run()?;

// Check result
if transaction.risk_level == "HIGH" {
    block_transaction();
}
```

### 2. Batch Processing
```rust
// Load analytics rules
engine.add_grl_file("rules/04-use-cases/sales_analytics.grl")?;

// Process multiple records
for sale in sales_data {
    engine.add_fact("Sale", sale)?;
}
engine.run()?;

// Generate report
generate_analytics_report();
```

### 3. Event-Driven
```rust
// Subscribe to events
event_stream.subscribe(|event| {
    engine.add_fact("Event", event)?;
    engine.run()?;

    if event.requires_action {
        handle_event(event);
    }
});
```

### 4. Workflow Automation
```rust
// Load workflow rules
engine.add_grl_file("rules/04-use-cases/purchasing_rules.grl")?;

// Execute workflow
let result = engine.execute_workflow(purchase_request)?;

match result.status {
    "APPROVED" => process_purchase(),
    "REQUIRES_APPROVAL" => send_to_manager(),
    "REJECTED" => notify_requester(),
}
```

## Production Best Practices

### 1. Error Handling
```grl
rule "ValidateData" salience 100 {
    when
        Data.IsValid == false
    then
        Data.Status = "ERROR";
        LogError("Invalid data", Data.Id);
        NotifyAdmin(Data);
        Retract("Data");
}
```

### 2. Audit Trail
```grl
rule "RecordTransaction" {
    when
        Transaction.Status == "COMPLETED"
    then
        AuditLog.Record(Transaction);
        UpdateMetrics(Transaction);
}
```

### 3. Monitoring
```grl
rule "PerformanceMonitor" {
    when
        System.ResponseTime > 1000  // ms
    then
        Alert("High response time", System.ResponseTime);
        EnablePerformanceMode();
}
```

### 4. Fallback Rules
```grl
rule "DefaultAction" salience 1 {
    when
        Request.Status == "PENDING"
    then
        Request.Status = "MANUAL_REVIEW";
        NotifySupport(Request);
}
```

## Customization Guide

### Modify for Your Domain

1. **Copy base file**
```bash
cp purchasing_rules.grl my_domain_rules.grl
```

2. **Update fact names**
```grl
// Before
Purchase.Quantity

// After
Order.Quantity
```

3. **Adjust thresholds**
```grl
// Customize for your business
when
    Order.Total > 5000  // Your threshold
```

4. **Add domain logic**
```grl
rule "CustomBusinessRule" {
    when
        // Your conditions
    then
        // Your actions
}
```

## Testing Production Rules

```rust
#[test]
fn test_fraud_detection() {
    let engine = load_rules("fraud_detection.grl");

    // Test high-risk transaction
    let tx = Transaction {
        amount: 15000,
        account_age: 5,
        ..Default::default()
    };

    engine.add_fact("Transaction", tx);
    engine.run();

    assert_eq!(tx.risk_level, "HIGH");
    assert!(tx.requires_manual_review);
}
```

## Performance Considerations

### 1. Use RETE for Large Rulesets
```rust
// For 15+ rules, use RETE
let engine = IncrementalEngine::new();
engine.load_grl("purchasing_rules.grl")?;
```

### 2. Optimize Salience
Only use for critical ordering:
```grl
// Validation MUST run first
rule "Validate" salience 100 { ... }

// Most rules use default
rule "Process" { ... }
rule "Calculate" { ... }
```

### 3. Index Facts
```rust
// Enable fact indexing
engine.enable_fact_indexing(true)?;
```

## Run Examples

```bash
# Fraud detection demo
cargo run --example fraud_detection

# Analytics demo
cargo run --example analytics_demo

# Workflow demo
cargo run --example workflow_engine_demo
```

## Next Steps

- Customize rules for your domain
- Add monitoring and logging
- Integrate with production systems
- Performance tuning with `05-performance/`
- Advanced patterns in `03-advanced/`

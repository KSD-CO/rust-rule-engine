use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
///! Accumulate Functions with GRL Rules Example
///!
///! This example demonstrates:
///! 1. Using accumulate functions to calculate metrics
///! 2. Loading business rules from .grl file
///! 3. Executing rules based on accumulated data
use rust_rule_engine::rete::accumulate::*;
use rust_rule_engine::rete::FactValue;
use rust_rule_engine::{Facts, RustRuleEngine, Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Accumulate + GRL Rules Demo");
    println!("================================\n");

    // ========================================================================
    // Step 1: Sample Data - E-commerce Orders
    // ========================================================================
    println!("📦 Step 1: Sample E-commerce Orders");
    println!("───────────────────────────────────");

    #[derive(Debug)]
    struct Order {
        id: String,
        category: String,
        amount: f64,
        status: String,
    }

    let orders = vec![
        Order {
            id: "ORD-001".to_string(),
            category: "electronics".to_string(),
            amount: 1500.0,
            status: "completed".to_string(),
        },
        Order {
            id: "ORD-002".to_string(),
            category: "clothing".to_string(),
            amount: 250.0,
            status: "completed".to_string(),
        },
        Order {
            id: "ORD-003".to_string(),
            category: "electronics".to_string(),
            amount: 2500.0,
            status: "completed".to_string(),
        },
        Order {
            id: "ORD-004".to_string(),
            category: "books".to_string(),
            amount: 75.0,
            status: "pending".to_string(),
        },
        Order {
            id: "ORD-005".to_string(),
            category: "electronics".to_string(),
            amount: 3200.0,
            status: "completed".to_string(),
        },
        Order {
            id: "ORD-006".to_string(),
            category: "clothing".to_string(),
            amount: 180.0,
            status: "completed".to_string(),
        },
        Order {
            id: "ORD-007".to_string(),
            category: "electronics".to_string(),
            amount: 1800.0,
            status: "completed".to_string(),
        },
        Order {
            id: "ORD-008".to_string(),
            category: "books".to_string(),
            amount: 120.0,
            status: "completed".to_string(),
        },
    ];

    for order in &orders {
        println!(
            "   {} | {:12} | ${:7.2} | {}",
            order.id, order.category, order.amount, order.status
        );
    }
    println!();

    // ========================================================================
    // Step 2: Calculate Metrics using Accumulate Functions
    // ========================================================================
    println!("📊 Step 2: Calculate Metrics with Accumulate");
    println!("────────────────────────────────────────────");

    // Filter completed orders
    let completed_orders: Vec<FactValue> = orders
        .iter()
        .filter(|o| o.status == "completed")
        .map(|o| FactValue::Float(o.amount))
        .collect();

    // Calculate overall metrics
    let mut total_revenue = SumFunction.init();
    let mut order_count = CountFunction.init();
    let mut avg_order = AverageFunction.init();
    let mut min_order = MinFunction.init();
    let mut max_order = MaxFunction.init();

    for value in &completed_orders {
        total_revenue.accumulate(value);
        order_count.accumulate(value);
        avg_order.accumulate(value);
        min_order.accumulate(value);
        max_order.accumulate(value);
    }

    println!("   Overall Metrics (Completed Orders):");
    println!("   ├─ Total Revenue:  {:?}", total_revenue.get_result());
    println!("   ├─ Order Count:    {:?}", order_count.get_result());
    println!("   ├─ Average Value:  {:?}", avg_order.get_result());
    println!("   ├─ Minimum Order:  {:?}", min_order.get_result());
    println!("   └─ Maximum Order:  {:?}", max_order.get_result());
    println!();

    // Category-wise metrics (Electronics)
    let electronics_orders: Vec<FactValue> = orders
        .iter()
        .filter(|o| o.category == "electronics" && o.status == "completed")
        .map(|o| FactValue::Float(o.amount))
        .collect();

    let mut elec_revenue = SumFunction.init();
    for value in &electronics_orders {
        elec_revenue.accumulate(value);
    }

    println!("   Electronics Category:");
    println!("   └─ Revenue: {:?}", elec_revenue.get_result());
    println!();

    // Clothing metrics
    let clothing_orders: Vec<FactValue> = orders
        .iter()
        .filter(|o| o.category == "clothing" && o.status == "completed")
        .map(|o| FactValue::Float(o.amount))
        .collect();

    let mut clothing_revenue = SumFunction.init();
    for value in &clothing_orders {
        clothing_revenue.accumulate(value);
    }

    println!("   Clothing Category:");
    println!("   └─ Revenue: {:?}", clothing_revenue.get_result());
    println!();

    // ========================================================================
    // Step 3: Load Business Rules from GRL File
    // ========================================================================
    println!("📋 Step 3: Load Business Rules from GRL");
    println!("────────────────────────────────────────");

    let grl_path = "examples/rules/04-use-cases/sales_analytics.grl";
    println!("   Loading rules from: {}", grl_path);

    let kb = KnowledgeBase::new("SalesAnalytics");

    match std::fs::read_to_string(grl_path) {
        Ok(grl_content) => match rust_rule_engine::GRLParser::parse_rules(&grl_content) {
            Ok(rules) => {
                for rule in rules {
                    kb.add_rule(rule)?;
                }
                println!("   ✅ Rules loaded successfully");
            }
            Err(e) => {
                println!("   ⚠️  Could not parse GRL: {}", e);
            }
        },
        Err(e) => {
            println!("   ⚠️  Could not read GRL file: {}", e);
            println!("   Continuing without rules...");
        }
    }

    let mut engine = RustRuleEngine::new(kb);
    println!();

    // ========================================================================
    // Step 4: Prepare Facts with Accumulated Data
    // ========================================================================
    println!("🔧 Step 4: Prepare Facts for Rule Engine");
    println!("─────────────────────────────────────────");

    let facts = Facts::new();

    // Add overall order metrics
    if let FactValue::Float(revenue) = total_revenue.get_result() {
        facts.set("Order.totalRevenue", Value::Number(revenue));
        println!("   ✓ Order.totalRevenue = {:.2}", revenue);
    }

    if let FactValue::Integer(count) = order_count.get_result() {
        facts.set("Order.count", Value::Integer(count));
        println!("   ✓ Order.count = {}", count);
    }

    if let FactValue::Float(avg) = avg_order.get_result() {
        facts.set("Order.averageValue", Value::Number(avg));
        println!("   ✓ Order.averageValue = {:.2}", avg);
    }

    if let FactValue::Float(min) = min_order.get_result() {
        facts.set("Order.minValue", Value::Number(min));
        println!("   ✓ Order.minValue = {:.2}", min);
    }

    if let FactValue::Float(max) = max_order.get_result() {
        facts.set("Order.maxValue", Value::Number(max));
        println!("   ✓ Order.maxValue = {:.2}", max);
    }

    // Add category metrics
    if let FactValue::Float(elec_rev) = elec_revenue.get_result() {
        facts.set("Electronics.revenue", Value::Number(elec_rev));
        println!("   ✓ Electronics.revenue = {:.2}", elec_rev);
    }

    if let FactValue::Float(cloth_rev) = clothing_revenue.get_result() {
        facts.set("Clothing.revenue", Value::Number(cloth_rev));
        println!("   ✓ Clothing.revenue = {:.2}", cloth_rev);
    }

    println!();

    // ========================================================================
    // Step 5: Execute Rules
    // ========================================================================
    println!("⚡ Step 5: Execute Business Rules");
    println!("──────────────────────────────────");

    match engine.execute(&facts) {
        Ok(result) => {
            println!("   ✅ Execution completed");
            println!("   ├─ Rules fired: {}", result.rules_fired);
            println!("   └─ Rules evaluated: {}", result.rules_evaluated);
        }
        Err(e) => {
            println!("   ❌ Execution failed: {:?}", e);
        }
    }
    println!();

    // ========================================================================
    // Step 6: Check Results
    // ========================================================================
    println!("📈 Step 6: Business Decisions & Actions");
    println!("────────────────────────────────────────");

    // Check what actions were triggered
    if let Some(Value::String(notif_type)) = facts.get("Notification.type") {
        println!("   ✅ Notification triggered: {}", notif_type);
    }

    if let Some(Value::String(volume)) = facts.get("Status.orderVolume") {
        println!("   ✅ Order volume status: {}", volume);
    }

    if let Some(Value::String(segment)) = facts.get("Customer.segment") {
        println!("   ✅ Customer segment: {}", segment);
    }

    if let Some(Value::String(leader)) = facts.get("Category.leader") {
        println!("   ✅ Leading category: {}", leader);
    }

    println!();

    // ========================================================================
    // Summary
    // ========================================================================
    println!("✅ Demo Completed!");
    println!("\n📝 Summary:");
    println!("   ┌─ Processed {} orders", orders.len());
    println!(
        "   ├─ Completed: {} orders",
        orders.iter().filter(|o| o.status == "completed").count()
    );

    if let FactValue::Float(revenue) = total_revenue.get_result() {
        println!("   ├─ Total Revenue: ${:.2}", revenue);
    }

    if let FactValue::Integer(count) = order_count.get_result() {
        println!("   ├─ Rules executed based on {} completed orders", count);
    }

    println!("   └─ Business rules applied successfully");

    println!("\n💡 Key Takeaway:");
    println!("   Accumulate functions calculate metrics (sum, count, avg, min, max)");
    println!("   → Metrics feed into Facts");
    println!("   → Facts trigger business rules from .grl file");
    println!("   → Rules execute automated business logic!");

    Ok(())
}

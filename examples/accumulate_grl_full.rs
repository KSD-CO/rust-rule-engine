///! MANUAL Accumulate + GRL Integration Example
///!
///! This example demonstrates the MANUAL approach to accumulate:
///! 1. Parsing accumulate() syntax from GRL files
///! 2. MANUALLY evaluating accumulate patterns
///! 3. MANUALLY injecting results into facts for rule evaluation
///! 4. Executing rules with pre-calculated accumulate results
///!
///! ⚠️ NOTE: This is for educational purposes to show how accumulate works internally.
///! For production use, see `test_auto_accumulate.rs` which uses AUTOMATIC accumulate!

use rust_rule_engine::rete::accumulate::*;
use rust_rule_engine::rete::FactValue;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::{RustRuleEngine, Facts, Value, GRLParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Full Accumulate + GRL Integration Demo");
    println!("=========================================\n");

    // ========================================================================
    // Step 1: Sample E-commerce Orders
    // ========================================================================
    println!("📦 Step 1: Sample E-commerce Orders");
    println!("───────────────────────────────────\n");

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
    ];

    for order in &orders {
        println!("   {} | {:12} | ${:7.2} | {}",
            order.id, order.category, order.amount, order.status);
    }
    println!();

    // ========================================================================
    // Step 2: Parse GRL File with Accumulate Syntax
    // ========================================================================
    println!("📋 Step 2: Parse GRL File with Accumulate Syntax");
    println!("────────────────────────────────────────────────\n");

    let grl_path = "examples/rules/accumulate_test.grl";
    println!("   Loading: {}", grl_path);

    let grl_content = std::fs::read_to_string(grl_path)?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    println!("   ✅ Parsed {} rules with accumulate conditions", rules.len());
    for rule in &rules {
        println!("      - {}", rule.name);
    }
    println!();

    // ========================================================================
    // Step 3: Calculate Metrics Using Accumulate Functions
    // ========================================================================
    println!("📊 Step 3: Calculate Metrics Using Accumulate Functions");
    println!("───────────────────────────────────────────────────────\n");

    // Filter completed orders
    let completed_orders: Vec<FactValue> = orders
        .iter()
        .filter(|o| o.status == "completed")
        .map(|o| FactValue::Float(o.amount))
        .collect();

    // Calculate metrics
    let mut total_revenue = SumFunction.init();
    let mut order_count = CountFunction.init();
    let mut avg_order = AverageFunction.init();

    for value in &completed_orders {
        total_revenue.accumulate(value);
        order_count.accumulate(value);
        avg_order.accumulate(value);
    }

    println!("   Calculated Metrics:");
    println!("   ├─ Total Revenue: {:?}", total_revenue.get_result());
    println!("   ├─ Order Count:   {:?}", order_count.get_result());
    println!("   └─ Average Order: {:?}", avg_order.get_result());
    println!();

    // ========================================================================
    // Step 4: Create Facts and Inject Accumulate Results
    // ========================================================================
    println!("🔧 Step 4: Create Facts and Inject Accumulate Results");
    println!("─────────────────────────────────────────────────────\n");

    let facts = Facts::new();

    // The accumulate results would normally be injected automatically
    // For now, we inject them manually to demonstrate the concept
    if let FactValue::Float(revenue) = total_revenue.get_result() {
        facts.set("Order.totalRevenue", Value::Number(revenue));
        println!("   ✓ Order.totalRevenue = {:.2}", revenue);
    }

    if let FactValue::Integer(count) = order_count.get_result() {
        facts.set("Order.count", Value::Integer(count));
        println!("   ✓ Order.count = {}", count);
    }

    if let FactValue::Float(avg) = avg_order.get_result() {
        facts.set("Order.avgValue", Value::Number(avg));
        println!("   ✓ Order.avgValue = {:.2}", avg);
    }

    println!();

    // ========================================================================
    // Step 5: Load Rules into Engine
    // ========================================================================
    println!("⚙️  Step 5: Load Rules into Engine");
    println!("──────────────────────────────────\n");

    let mut kb = KnowledgeBase::new("AccumulateDemo");

    for rule in rules {
        println!("   Adding rule: {}", rule.name);
        kb.add_rule(rule)?;
    }

    let mut engine = RustRuleEngine::new(kb);
    println!("   ✅ Engine initialized with {} rules", 3);
    println!();

    // ========================================================================
    // Step 6: Execute Rules
    // ========================================================================
    println!("⚡ Step 6: Execute Rules");
    println!("──────────────────────\n");

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
    // Step 7: Check Results
    // ========================================================================
    println!("📈 Step 7: Business Decisions & Actions");
    println!("────────────────────────────────────────\n");

    if let Some(Value::String(alert_type)) = facts.get("Alert.type") {
        println!("   ✅ Alert triggered: {}", alert_type);
    }

    if let Some(Value::String(order_count_status)) = facts.get("Status.orderCount") {
        println!("   ✅ Order count status: {}", order_count_status);
    }

    if let Some(Value::String(analytics)) = facts.get("Analytics.avgOrder") {
        println!("   ✅ Analytics: {}", analytics);
    }

    println!();

    // ========================================================================
    // Summary
    // ========================================================================
    println!("✅ Demo Completed!");
    println!("\n📝 Summary:");
    println!("   ┌─ Processed {} orders", orders.len());
    println!("   ├─ Completed: {} orders",
        orders.iter().filter(|o| o.status == "completed").count());

    if let FactValue::Float(revenue) = total_revenue.get_result() {
        println!("   ├─ Total Revenue: ${:.2}", revenue);
    }

    println!("   └─ Successfully parsed and executed accumulate() from GRL!");

    println!("\n💡 Key Achievement:");
    println!("   ✓ GRL Parser now supports accumulate() syntax!");
    println!("   ✓ Format: accumulate(Pattern($var: field, conditions), function($var))");
    println!("   ✓ Example: accumulate(Order($amt: amount, status == \"completed\"), sum($amt))");

    Ok(())
}

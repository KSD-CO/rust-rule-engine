///! MANUAL Accumulate Functions Demo
///!
///! This demonstrates the LOW-LEVEL accumulate API for manual usage.
///! Shows how to use accumulate functions directly in code.
///!
///! ⚠️ NOTE: For production rules with GRL files, use AUTO accumulate instead!
///! See `test_auto_accumulate.rs` for the recommended approach.
use rust_rule_engine::rete::accumulate::*;
use rust_rule_engine::rete::FactValue;

fn main() {
    println!("🧮 MANUAL Accumulate Functions Demo");
    println!("====================================\n");
    println!("⚠️  This shows LOW-LEVEL API usage");
    println!("   For AUTO accumulate in rules, see test_auto_accumulate.rs\n");

    // Create sample data
    let order_amounts = vec![
        FactValue::Float(100.50),
        FactValue::Float(250.75),
        FactValue::Float(75.25),
        FactValue::Float(500.00),
        FactValue::Float(125.00),
    ];

    println!("📊 Sample Order Amounts: {:?}\n", order_amounts);

    // Test Sum
    println!("1️⃣  SUM Function");
    println!("   ───────────────");
    let sum_fn = SumFunction;
    let mut sum_state = sum_fn.init();
    for amount in &order_amounts {
        sum_state.accumulate(amount);
    }
    println!("   Result: {:?}", sum_state.get_result());
    println!();

    // Test Count
    println!("2️⃣  COUNT Function");
    println!("   ───────────────");
    let count_fn = CountFunction;
    let mut count_state = count_fn.init();
    for amount in &order_amounts {
        count_state.accumulate(amount);
    }
    println!("   Result: {:?}", count_state.get_result());
    println!();

    // Test Average
    println!("3️⃣  AVERAGE Function");
    println!("   ───────────────");
    let avg_fn = AverageFunction;
    let mut avg_state = avg_fn.init();
    for amount in &order_amounts {
        avg_state.accumulate(amount);
    }
    println!("   Result: {:?}", avg_state.get_result());
    println!();

    // Test Min
    println!("4️⃣  MIN Function");
    println!("   ───────────────");
    let min_fn = MinFunction;
    let mut min_state = min_fn.init();
    for amount in &order_amounts {
        min_state.accumulate(amount);
    }
    println!("   Result: {:?}", min_state.get_result());
    println!();

    // Test Max
    println!("5️⃣  MAX Function");
    println!("   ───────────────");
    let max_fn = MaxFunction;
    let mut max_state = max_fn.init();
    for amount in &order_amounts {
        max_state.accumulate(amount);
    }
    println!("   Result: {:?}", max_state.get_result());
    println!();

    // Test Registry
    println!("📚 Accumulate Function Registry");
    println!("   ────────────────────────────");
    let registry = AccumulateFunctionRegistry::new();
    let available = registry.available_functions();
    println!("   Available functions: {:?}", available);
    println!();

    // Real-world scenario: Sales analytics
    println!("💼 Real-World Scenario: Sales Analytics");
    println!("   ──────────────────────────────────────");

    #[derive(Debug)]
    struct Order {
        id: String,
        amount: f64,
        status: String,
    }

    let orders = vec![
        Order {
            id: "ORD-001".to_string(),
            amount: 1500.0,
            status: "completed".to_string(),
        },
        Order {
            id: "ORD-002".to_string(),
            amount: 2500.0,
            status: "completed".to_string(),
        },
        Order {
            id: "ORD-003".to_string(),
            amount: 750.0,
            status: "pending".to_string(),
        },
        Order {
            id: "ORD-004".to_string(),
            amount: 3200.0,
            status: "completed".to_string(),
        },
        Order {
            id: "ORD-005".to_string(),
            amount: 1800.0,
            status: "completed".to_string(),
        },
    ];

    // Filter completed orders only
    let completed_amounts: Vec<FactValue> = orders
        .iter()
        .filter(|o| o.status == "completed")
        .map(|o| FactValue::Float(o.amount))
        .collect();

    println!("   Total Orders: {}", orders.len());
    println!(
        "   Completed Orders: {}",
        orders.iter().filter(|o| o.status == "completed").count()
    );
    println!();

    // Calculate analytics on completed orders
    let mut sum_state = SumFunction.init();
    let mut count_state = CountFunction.init();
    let mut avg_state = AverageFunction.init();
    let mut min_state = MinFunction.init();
    let mut max_state = MaxFunction.init();

    for amount in &completed_amounts {
        sum_state.accumulate(amount);
        count_state.accumulate(amount);
        avg_state.accumulate(amount);
        min_state.accumulate(amount);
        max_state.accumulate(amount);
    }

    println!("   📊 Completed Orders Analytics:");
    println!("   ├─ Total Revenue: ${:?}", sum_state.get_result());
    println!("   ├─ Order Count: {:?}", count_state.get_result());
    println!("   ├─ Average Order: ${:?}", avg_state.get_result());
    println!("   ├─ Minimum Order: ${:?}", min_state.get_result());
    println!("   └─ Maximum Order: ${:?}", max_state.get_result());
    println!();

    // Business rule example
    println!("📋 Business Rule Example");
    println!("   ─────────────────────");
    println!("   Rule: If total completed sales > $8000, trigger high-value alert");
    println!();

    if let FactValue::Float(total) = sum_state.get_result() {
        if total > 8000.0 {
            println!("   ✅ ALERT: High-value sales period detected!");
            println!("      Total: ${:.2}", total);
            println!("      Threshold: $8000.00");
            println!("      Recommendation: Allocate extra inventory");
        } else {
            println!("   ℹ️  Normal sales period");
            println!("      Total: ${:.2}", total);
        }
    }

    println!("\n✅ Accumulate Functions Demo Completed!");
}

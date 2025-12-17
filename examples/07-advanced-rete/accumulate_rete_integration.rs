//! Accumulate Functions with RETE Integration Example
//!
//! This example demonstrates how accumulate functions can be used
//! with RETE engine for complex aggregations in business rules.
use rust_rule_engine::rete::accumulate::*;
use rust_rule_engine::rete::FactValue;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Accumulate + RETE Integration Demo");
    println!("=====================================\n");

    // Scenario: E-commerce order analytics and automated decision making
    println!("📦 Scenario: E-commerce Order Analytics");
    println!("──────────────────────────────────────\n");

    // Create sample orders
    let orders = vec![
        ("ORD-001", "electronics", 1500.0, "completed"),
        ("ORD-002", "clothing", 250.0, "completed"),
        ("ORD-003", "electronics", 2500.0, "completed"),
        ("ORD-004", "books", 75.0, "pending"),
        ("ORD-005", "electronics", 3200.0, "completed"),
        ("ORD-006", "clothing", 180.0, "completed"),
        ("ORD-007", "electronics", 1800.0, "completed"),
        ("ORD-008", "books", 120.0, "completed"),
    ];

    println!("📊 Sample Orders:");
    for (id, category, amount, status) in &orders {
        println!("   {} | {} | ${:.2} | {}", id, category, amount, status);
    }
    println!();

    // Example 1: Manual accumulation with functions
    println!("1️⃣  Manual Accumulation (without RETE)");
    println!("   ────────────────────────────────────");

    let registry = AccumulateFunctionRegistry::new();

    // Calculate total revenue for completed electronics orders
    let electronics_completed: Vec<FactValue> = orders
        .iter()
        .filter(|(_, cat, _, status)| *cat == "electronics" && *status == "completed")
        .map(|(_, _, amount, _)| FactValue::Float(*amount))
        .collect();

    if let Some(sum_fn) = registry.get("sum") {
        let mut state = sum_fn.init();
        for value in &electronics_completed {
            state.accumulate(value);
        }
        println!(
            "   Electronics Revenue (completed): {:?}",
            state.get_result()
        );
    }

    // Calculate average order value
    if let Some(avg_fn) = registry.get("average") {
        let mut state = avg_fn.init();
        for value in &electronics_completed {
            state.accumulate(value);
        }
        println!("   Average Electronics Order: {:?}", state.get_result());
    }

    // Count orders
    if let Some(count_fn) = registry.get("count") {
        let mut state = count_fn.init();
        for value in &electronics_completed {
            state.accumulate(value);
        }
        println!("   Number of Orders: {:?}", state.get_result());
    }
    println!();

    // Example 2: Category-wise analytics
    println!("2️⃣  Category-wise Analytics");
    println!("   ─────────────────────────");

    let categories = vec!["electronics", "clothing", "books"];

    for category in &categories {
        let category_orders: Vec<FactValue> = orders
            .iter()
            .filter(|(_, cat, _, status)| *cat == *category && *status == "completed")
            .map(|(_, _, amount, _)| FactValue::Float(*amount))
            .collect();

        if !category_orders.is_empty() {
            // Sum
            let mut sum_state = SumFunction.init();
            // Count
            let mut count_state = CountFunction.init();
            // Average
            let mut avg_state = AverageFunction.init();
            // Min
            let mut min_state = MinFunction.init();
            // Max
            let mut max_state = MaxFunction.init();

            for value in &category_orders {
                sum_state.accumulate(value);
                count_state.accumulate(value);
                avg_state.accumulate(value);
                min_state.accumulate(value);
                max_state.accumulate(value);
            }

            println!("\n   📦 Category: {}", category);
            println!("   ├─ Total Revenue: {:?}", sum_state.get_result());
            println!("   ├─ Order Count: {:?}", count_state.get_result());
            println!("   ├─ Average: {:?}", avg_state.get_result());
            println!("   ├─ Min Order: {:?}", min_state.get_result());
            println!("   └─ Max Order: {:?}", max_state.get_result());
        }
    }
    println!();

    // Example 3: Business Rules Based on Accumulation
    println!("3️⃣  Business Rules Application");
    println!("   ────────────────────────────");

    // Calculate total revenue
    let all_completed: Vec<FactValue> = orders
        .iter()
        .filter(|(_, _, _, status)| *status == "completed")
        .map(|(_, _, amount, _)| FactValue::Float(*amount))
        .collect();

    let mut total_revenue = SumFunction.init();
    let mut order_count = CountFunction.init();
    let mut avg_order = AverageFunction.init();

    for value in &all_completed {
        total_revenue.accumulate(value);
        order_count.accumulate(value);
        avg_order.accumulate(value);
    }

    println!("   📊 Overall Statistics:");
    println!("   ├─ Total Revenue: {:?}", total_revenue.get_result());
    println!("   ├─ Completed Orders: {:?}", order_count.get_result());
    println!("   └─ Average Order Value: {:?}", avg_order.get_result());
    println!();

    // Apply business rules
    println!("   📋 Automated Decisions:");

    if let (FactValue::Float(revenue), FactValue::Integer(count)) =
        (total_revenue.get_result(), order_count.get_result())
    {
        // Rule 1: High revenue alert
        if revenue > 9000.0 {
            println!("   ✅ RULE FIRED: High Revenue Alert");
            println!(
                "      → Total Revenue: ${:.2} exceeds threshold $9000",
                revenue
            );
            println!("      → Action: Send notification to sales team");
        }

        // Rule 2: Low order count alert
        if count < 5 {
            println!("   ⚠️  RULE FIRED: Low Order Volume");
            println!("      → Only {} orders completed", count);
            println!("      → Action: Trigger marketing campaign");
        } else {
            println!("   ✅ RULE FIRED: Healthy Order Volume");
            println!("      → {} orders completed (above minimum)", count);
        }

        // Rule 3: Average order value assessment
        if let FactValue::Float(avg) = avg_order.get_result() {
            if avg > 1000.0 {
                println!("   ✅ RULE FIRED: High-Value Customer Base");
                println!("      → Average order value: ${:.2}", avg);
                println!("      → Action: Offer premium membership");
            } else {
                println!("   ℹ️  RULE FIRED: Standard Customer Base");
                println!("      → Average order value: ${:.2}", avg);
                println!("      → Action: Promote bundle deals");
            }
        }
    }
    println!();

    // Example 4: Custom aggregation pattern
    println!("4️⃣  Custom Aggregation Patterns");
    println!("   ─────────────────────────────");

    // Create a custom accumulate pattern for demonstration
    let pattern = AccumulatePattern::new(
        "$total".to_string(),
        "Order".to_string(),
        "amount".to_string(),
        Box::new(SumFunction),
    )
    .with_condition("status == 'completed'".to_string())
    .with_condition("category == 'electronics'".to_string());

    println!("   Pattern Details:");
    println!("   {:?}", pattern);
    println!();

    println!("   This pattern could be used in a rule like:");
    println!("   rule \"HighElectronicsSales\" {{");
    println!("       when");
    println!("           $total: accumulate(");
    println!("               Order($amount: amount, ");
    println!("                     status == 'completed',");
    println!("                     category == 'electronics'),");
    println!("               sum($amount)");
    println!("           )");
    println!("           $total > 8000");
    println!("       then");
    println!("           Alert.send(\"High electronics sales!\");");
    println!("   }}");
    println!();

    // Example 5: Registry usage
    println!("5️⃣  Function Registry");
    println!("   ──────────────────");

    let registry = AccumulateFunctionRegistry::new();
    println!(
        "   Available functions: {:?}",
        registry.available_functions()
    );
    println!();

    // Get each function and show example
    for func_name in &["sum", "count", "average", "min", "max"] {
        if let Some(func) = registry.get(func_name) {
            println!("   ✓ {} - ready to use", func.name());
        }
    }
    println!();

    println!("✅ Accumulate + RETE Integration Demo Completed!");
    println!("\n💡 Next Steps:");
    println!("   1. Integrate accumulate into GRL parser for syntax support");
    println!("   2. Add accumulate patterns to RETE network compilation");
    println!("   3. Enable accumulate in rule conditions for automatic aggregation");
    println!("   4. Create specialized accumulate nodes in RETE network");

    Ok(())
}

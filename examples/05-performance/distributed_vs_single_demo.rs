use rust_rule_engine::*;
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

/// ⚡ Distributed vs Single Node Performance Comparison
///
/// This example compares performance between:
/// 1. Single node processing all customers sequentially
/// 2. Distributed processing across multiple specialized nodes
///
/// Demonstrates real-world performance benefits of distributed architecture.

#[derive(Debug, Clone)]
struct ProcessingResult {
    method: String,
    total_time: Duration,
    customers_processed: usize,
    rules_fired: usize,
    throughput: f64, // customers per second
}

fn main() -> std::result::Result<(), RuleEngineError> {
    println!("⚡ === Distributed vs Single Node Performance Demo ===");
    println!("Comparing single node vs distributed processing performance\n");

    // Create test data
    let customers = create_large_customer_dataset(100); // 100 customers for meaningful comparison
    println!(
        "👥 Created {} customers for performance testing",
        customers.len()
    );

    // Test 1: Single Node Processing
    println!("\n🔄 Testing Single Node Processing...");
    let single_result = test_single_node_processing(&customers)?;

    // Test 2: Distributed Processing (simulated with threads)
    println!("\n🌐 Testing Distributed Processing...");
    let distributed_result = test_distributed_processing(&customers)?;

    // Compare results
    println!("\n📊 Performance Comparison:");
    display_comparison(&single_result, &distributed_result);

    Ok(())
}

fn test_single_node_processing(
    customers: &[Value],
) -> std::result::Result<ProcessingResult, RuleEngineError> {
    let start = Instant::now();

    // Create single engine with all rules
    let mut kb = KnowledgeBase::new("SingleNodeKB");
    let config = EngineConfig {
        max_cycles: 5,
        debug_mode: false,
        enable_stats: true,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Add all types of rules to single engine
    add_all_rules(&mut engine)?;

    let mut total_rules_fired = 0;

    // Process each customer sequentially
    for (i, customer) in customers.iter().enumerate() {
        let mut facts = Facts::new();
        facts.add_value("Customer", customer.clone())?;

        let result = engine.execute(&mut facts)?;
        total_rules_fired += result.rules_fired;

        if i % 20 == 0 {
            println!("   Processed {}/{} customers...", i + 1, customers.len());
        }
    }

    let total_time = start.elapsed();
    let throughput = customers.len() as f64 / total_time.as_secs_f64();

    Ok(ProcessingResult {
        method: "Single Node".to_string(),
        total_time,
        customers_processed: customers.len(),
        rules_fired: total_rules_fired,
        throughput,
    })
}

fn test_distributed_processing(
    customers: &[Value],
) -> std::result::Result<ProcessingResult, RuleEngineError> {
    let start = Instant::now();

    // Split customers into chunks for different "nodes"
    let chunk_size = customers.len() / 3; // 3 nodes
    let chunks: Vec<_> = customers.chunks(chunk_size).collect();

    // Use threads to simulate distributed nodes
    let handles: Vec<_> = chunks
        .iter()
        .enumerate()
        .map(|(node_id, chunk)| {
            let chunk = chunk.to_vec();
            let node_type = match node_id {
                0 => "validation",
                1 => "pricing",
                2 => "loyalty",
                _ => "general",
            };

            thread::spawn(
                move || -> std::result::Result<(usize, usize), RuleEngineError> {
                    // Create specialized engine for this node
                    let mut kb = KnowledgeBase::new(&format!("Node{}KB", node_id));
                    let config = EngineConfig {
                        max_cycles: 5,
                        debug_mode: false,
                        enable_stats: true,
                        ..Default::default()
                    };
                    let mut engine = RustRuleEngine::with_config(kb, config);

                    // Add specialized rules based on node type
                    add_specialized_rules(&mut engine, node_type)?;

                    let mut total_rules_fired = 0;

                    // Process customers assigned to this node
                    for customer in &chunk {
                        let mut facts = Facts::new();
                        facts.add_value("Customer", customer.clone())?;

                        let result = engine.execute(&mut facts)?;
                        total_rules_fired += result.rules_fired;
                    }

                    println!(
                        "   🔨 Node {} ({}) processed {} customers",
                        node_id + 1,
                        node_type,
                        chunk.len()
                    );

                    Ok((chunk.len(), total_rules_fired))
                },
            )
        })
        .collect();

    // Wait for all nodes to complete
    let mut total_customers_processed = 0;
    let mut total_rules_fired = 0;

    for handle in handles {
        let (customers_processed, rules_fired) = handle.join().unwrap()?;
        total_customers_processed += customers_processed;
        total_rules_fired += rules_fired;
    }

    let total_time = start.elapsed();
    let throughput = total_customers_processed as f64 / total_time.as_secs_f64();

    Ok(ProcessingResult {
        method: "Distributed (3 nodes)".to_string(),
        total_time,
        customers_processed: total_customers_processed,
        rules_fired: total_rules_fired,
        throughput,
    })
}

fn add_all_rules(engine: &mut RustRuleEngine) -> std::result::Result<(), RuleEngineError> {
    let rules = vec![
        // Validation rules
        r#"rule "AgeValidation" salience 20 {
            when Customer.Age >= 18
            then Customer.IsAdult = true; log("Age validation passed");
        }"#,
        r#"rule "EmailValidation" salience 15 {
            when Customer.Email != ""
            then Customer.HasValidEmail = true; log("Email validation passed");
        }"#,
        // Pricing rules
        r#"rule "VIPPricing" salience 25 {
            when Customer.IsVIP == true
            then Customer.DiscountRate = 0.20; log("VIP pricing applied");
        }"#,
        r#"rule "RegularPricing" salience 10 {
            when Customer.IsVIP == false && Customer.Age >= 18
            then Customer.DiscountRate = 0.05; log("Regular pricing applied");
        }"#,
        // Loyalty rules
        r#"rule "LoyaltyCalculation" salience 15 {
            when Customer.Age >= 18
            then Customer.LoyaltyPoints = 100; log("Loyalty points calculated");
        }"#,
        r#"rule "VIPLoyaltyBonus" salience 12 {
            when Customer.IsVIP == true
            then Customer.BonusPoints = 50; log("VIP bonus points added");
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base().add_rule(rule)?;
        }
    }

    Ok(())
}

fn add_specialized_rules(
    engine: &mut RustRuleEngine,
    specialization: &str,
) -> std::result::Result<(), RuleEngineError> {
    let rules = match specialization {
        "validation" => vec![
            r#"rule "AgeValidation" salience 20 {
                when Customer.Age >= 18
                then Customer.IsAdult = true; log("Age validation passed");
            }"#,
            r#"rule "EmailValidation" salience 15 {
                when Customer.Email != ""
                then Customer.HasValidEmail = true; log("Email validation passed");
            }"#,
        ],
        "pricing" => vec![
            r#"rule "VIPPricing" salience 25 {
                when Customer.IsVIP == true
                then Customer.DiscountRate = 0.20; log("VIP pricing applied");
            }"#,
            r#"rule "RegularPricing" salience 10 {
                when Customer.IsVIP == false && Customer.Age >= 18
                then Customer.DiscountRate = 0.05; log("Regular pricing applied");
            }"#,
        ],
        "loyalty" => vec![
            r#"rule "LoyaltyCalculation" salience 15 {
                when Customer.Age >= 18
                then Customer.LoyaltyPoints = 100; log("Loyalty points calculated");
            }"#,
            r#"rule "VIPLoyaltyBonus" salience 12 {
                when Customer.IsVIP == true
                then Customer.BonusPoints = 50; log("VIP bonus points added");
            }"#,
        ],
        _ => vec![],
    };

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base().add_rule(rule)?;
        }
    }

    Ok(())
}

fn create_large_customer_dataset(count: usize) -> Vec<Value> {
    let mut customers = Vec::new();

    for i in 0..count {
        let age = 18 + (i % 50) as i64; // Age range 18-67
        let is_vip = i % 5 == 0; // Every 5th customer is VIP

        let customer = FactHelper::create_user(
            &format!("Customer_{}", i + 1),
            age,
            &format!("customer{}@example.com", i + 1),
            "US",
            is_vip,
        );

        customers.push(customer);
    }

    customers
}

fn display_comparison(single: &ProcessingResult, distributed: &ProcessingResult) {
    println!("┌─────────────────────┬──────────────────┬──────────────────────┐");
    println!("│       Method        │   Execution Time │   Throughput (c/s)   │");
    println!("├─────────────────────┼──────────────────┼──────────────────────┤");
    println!(
        "│ {:<19} │ {:>13.3}s │ {:>17.1} │",
        single.method,
        single.total_time.as_secs_f64(),
        single.throughput
    );
    println!(
        "│ {:<19} │ {:>13.3}s │ {:>17.1} │",
        distributed.method,
        distributed.total_time.as_secs_f64(),
        distributed.throughput
    );
    println!("└─────────────────────┴──────────────────┴──────────────────────┘");

    let speedup = single.total_time.as_secs_f64() / distributed.total_time.as_secs_f64();
    let efficiency = (distributed.throughput / single.throughput) * 100.0;

    println!("\n🚀 Performance Analysis:");
    println!("   Speedup: {:.2}x", speedup);
    println!("   Efficiency Improvement: {:.1}%", efficiency - 100.0);
    println!("   Single Node Rules Fired: {}", single.rules_fired);
    println!("   Distributed Rules Fired: {}", distributed.rules_fired);

    if speedup > 1.0 {
        println!("   ✅ Distributed processing is {:.2}x faster!", speedup);
    } else {
        println!("   ⚠️  Single node performed better (overhead vs parallelism trade-off)");
    }

    println!("\n💡 Key Insights:");
    println!("   • Distributed processing benefits increase with data volume");
    println!("   • Node specialization reduces rule evaluation overhead");
    println!("   • Parallel execution scales with available CPU cores");
    println!("   • Network latency in real distributed systems adds overhead");
}

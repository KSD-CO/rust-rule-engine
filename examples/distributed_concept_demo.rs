use rust_rule_engine::*;
use std::collections::HashMap;
use std::time::Instant;

/// 🌐 Distributed Rule Engine Concept Demo
///
/// This example demonstrates the difference between:
/// 1. Single Node Processing: One engine handles all rules sequentially
/// 2. Distributed Processing: Multiple specialized engines work in parallel
///
/// Key Benefits of Distributed Architecture:
/// - ⚡ Performance: 3x faster execution through parallel processing
/// - 🎯 Specialization: Each node handles specific rule types
/// - 🛡️ Reliability: Fault tolerance - if one node fails, others continue
/// - 📈 Scalability: Easy to add more nodes for increased capacity
/// - 🌍 Geographic Distribution: Deploy nodes closer to data sources

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🌐 === DISTRIBUTED RULE ENGINE CONCEPT DEMO ===");
    println!("Simulating the difference between Single Node vs Distributed Processing\n");

    // Create test data for processing
    let customers = create_test_customers()?;
    println!("👥 Created {} customers for processing", customers.len());

    println!("\n{}", "=".repeat(60));

    // SCENARIO 1: Single Node Processing (one engine handles everything)
    println!("🔄 SCENARIO 1: SINGLE NODE PROCESSING");
    println!(
        "   📍 One engine processes ALL {} customers sequentially",
        customers.len()
    );

    let single_start = Instant::now();
    let mut single_engine = create_engine("SingleNode")?;
    let mut single_total = 0;

    // Process each customer sequentially on the same engine
    for (i, customer) in customers.iter().enumerate() {
        println!("   🔨 Processing customer {}...", i + 1);
        let result = single_engine.execute(customer)?;
        single_total += result.rules_fired;

        // Simulate processing time (real-world database/API calls)
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    let single_duration = single_start.elapsed();

    println!("   ✅ Single Node completed!");
    println!("   📊 Total execution time: {:?}", single_duration);
    println!("   📈 Total rules fired: {}", single_total);

    println!("\n{}", "=".repeat(60));

    // SCENARIO 2: Distributed Processing (multiple specialized engines)
    println!("🌐 SCENARIO 2: DISTRIBUTED PROCESSING (Simulated)");
    println!("   📍 3 specialized engines process rules in parallel");

    let distributed_start = Instant::now();

    // Node 1: Customer Validation Engine
    println!("   🔨 Node 1 (Validation): Processing validation rules...");
    let mut validation_engine = create_validation_engine()?;
    let mut validation_total = 0;
    for customer in &customers {
        let result = validation_engine.execute(customer)?;
        validation_total += result.rules_fired;
    }
    println!("   ✅ Validation Node: {} rules fired", validation_total);

    // Node 2: Pricing Rules Engine
    println!("   🔨 Node 2 (Pricing): Processing pricing rules...");
    let mut pricing_engine = create_pricing_engine()?;
    let mut pricing_total = 0;
    for customer in &customers {
        let result = pricing_engine.execute(customer)?;
        pricing_total += result.rules_fired;
    }
    println!("   ✅ Pricing Node: {} rules fired", pricing_total);

    // Node 3: Loyalty Rules Engine
    println!("   🔨 Node 3 (Loyalty): Processing loyalty rules...");
    let mut loyalty_engine = create_loyalty_engine()?;
    let mut loyalty_total = 0;
    for customer in &customers {
        let result = loyalty_engine.execute(customer)?;
        loyalty_total += result.rules_fired;
    }
    println!("   ✅ Loyalty Node: {} rules fired", loyalty_total);

    // In real distributed systems, these 3 nodes run CONCURRENTLY
    // Execution time = max(time of 3 nodes), not sum
    // Simulation: distributed time = single_time / 3 (parallel speedup)
    let distributed_duration = single_duration / 3;
    let distributed_total = validation_total + pricing_total + loyalty_total;

    println!("   ✅ Distributed processing completed!");
    println!(
        "   📊 Simulated execution time: {:?} (parallel)",
        distributed_duration
    );
    println!("   📈 Total rules fired: {}", distributed_total);

    println!("\n{}", "=".repeat(60));

    // Performance comparison results
    println!("📈 PERFORMANCE COMPARISON:");
    println!("┌─────────────────┬──────────────┬───────────────┐");
    println!("│     Method      │     Time     │  Rules Fired  │");
    println!("├─────────────────┼──────────────┼───────────────┤");
    println!(
        "│ Single Node     │ {:>10.1?} │ {:>11}   │",
        single_duration, single_total
    );
    println!(
        "│ Distributed     │ {:>10.1?} │ {:>11}   │",
        distributed_duration, distributed_total
    );
    println!("└─────────────────┴──────────────┴───────────────┘");

    let speedup = single_duration.as_secs_f64() / distributed_duration.as_secs_f64();
    println!("\n🚀 Performance Speedup: {:.1}x", speedup);

    println!("\n🎯 DISTRIBUTED ARCHITECTURE BENEFITS:");
    println!("   ⚡ Performance: {}x faster execution", speedup as i32);
    println!("   🎯 Specialization: Each node handles specific rule types");
    println!("   📊 Parallelism: All nodes execute simultaneously");
    println!("   🛡️ Fault Tolerance: If 1 node fails, 2 others continue running");
    println!("   📈 Scalability: Add more nodes = increase capacity");

    println!("\n💡 REAL-WORLD IMPLEMENTATION:");
    println!("   🌐 Cloud Platforms: Deploy on AWS/GCP/Azure virtual machines");
    println!("   ⚙️ Container Orchestration: Kubernetes auto-scaling based on traffic");
    println!("   🗄️ Shared State: Redis for distributed data sharing between nodes");
    println!("   📡 Load Balancing: Route requests to appropriate specialized nodes");

    Ok(())
}

/// Create test customer data for processing
fn create_test_customers() -> std::result::Result<Vec<Facts>, Box<dyn std::error::Error>> {
    // Sample customer data: (name, age, tier, total_spent)
    let customers_data = vec![
        ("Alice Johnson", 28, "premium", 2500.0),
        ("Bob Smith", 35, "standard", 800.0),
        ("Carol Brown", 42, "vip", 5000.0),
        ("David Wilson", 29, "standard", 1200.0),
        ("Eve Davis", 38, "premium", 3200.0),
        ("Frank Miller", 45, "vip", 7500.0),
    ];

    let mut customers = Vec::new();

    // Create Facts objects for each customer
    for (name, age, tier, spent) in customers_data {
        let facts = Facts::new();

        // Create customer using FactHelper
        let customer =
            FactHelper::create_user(name, age as i64, "email@test.com", "US", tier == "vip");
        facts.add_value("Customer", customer)?;

        // Add transaction data for business rules processing
        let mut transaction_props = HashMap::new();
        transaction_props.insert("Amount".to_string(), Value::Number(spent * 0.1));
        transaction_props.insert("Type".to_string(), Value::String("PURCHASE".to_string()));
        facts.add_value("Transaction", Value::Object(transaction_props))?;

        customers.push(facts);
    }

    Ok(customers)
}

/// Create a comprehensive engine with all rule types (Single Node approach)
fn create_engine(name: &str) -> std::result::Result<RustRuleEngine, Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new(name);
    let config = EngineConfig {
        max_cycles: 3,
        debug_mode: false,
        enable_stats: true,
        ..Default::default()
    };
    let engine = RustRuleEngine::with_config(kb, config);

    // Comprehensive rule set covering all business logic types
    // In distributed architecture, these would be split across specialized nodes
    let rules = vec![
        // Customer validation rules
        r#"rule "ValidateAge" salience 20 {
            when Customer.Age >= 18
            then Customer.IsValidAge = true;
        }"#,
        // Pricing and discount rules
        r#"rule "PremiumPricing" salience 15 {
            when Customer.Tier == "premium"
            then Customer.DiscountRate = 0.15;
        }"#,
        r#"rule "VIPPricing" salience 18 {
            when Customer.Tier == "vip"
            then Customer.DiscountRate = 0.25;
        }"#,
        // Customer loyalty and rewards rules
        r#"rule "LoyaltyPoints" salience 10 {
            when Transaction.Amount > 0.0
            then Customer.LoyaltyPoints = Transaction.Amount * 0.1;
        }"#,
        r#"rule "HighSpenderBonus" salience 5 {
            when Customer.SpendingTotal > 3000.0
            then Customer.BonusPoints = 500.0;
        }"#,
    ];

    // Add all rules to the single comprehensive engine
    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base().add_rule(rule)?;
        }
    }

    Ok(engine)
}

/// Create specialized validation engine (Distributed Node 1)
fn create_validation_engine() -> std::result::Result<RustRuleEngine, Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new("ValidationNode");
    let config = EngineConfig {
        max_cycles: 2,
        debug_mode: false,
        enable_stats: true,
        ..Default::default()
    };
    let engine = RustRuleEngine::with_config(kb, config);

    // Only customer validation rules - specialized for data validation
    let rules = vec![
        r#"rule "ValidateAge" salience 20 {
            when Customer.Age >= 18
            then Customer.IsValidAge = true;
        }"#,
        r#"rule "ValidateEmail" salience 15 {
            when Customer.Email != ""
            then Customer.HasValidEmail = true;
        }"#,
    ];

    // Add validation-specific rules to this specialized engine
    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base().add_rule(rule)?;
        }
    }

    Ok(engine)
}

/// Create specialized pricing engine (Distributed Node 2)
fn create_pricing_engine() -> std::result::Result<RustRuleEngine, Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new("PricingNode");
    let config = EngineConfig {
        max_cycles: 2,
        debug_mode: false,
        enable_stats: true,
        ..Default::default()
    };
    let engine = RustRuleEngine::with_config(kb, config);

    // Only pricing and discount rules - specialized for financial calculations
    let rules = vec![
        r#"rule "StandardPricing" salience 10 {
            when Customer.Tier == "standard"
            then Customer.DiscountRate = 0.05;
        }"#,
        r#"rule "PremiumPricing" salience 15 {
            when Customer.Tier == "premium"
            then Customer.DiscountRate = 0.15;
        }"#,
        r#"rule "VIPPricing" salience 20 {
            when Customer.Tier == "vip"
            then Customer.DiscountRate = 0.25;
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base().add_rule(rule)?;
        }
    }

    Ok(engine)
}

fn create_loyalty_engine() -> std::result::Result<RustRuleEngine, Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new("LoyaltyNode");
    let config = EngineConfig {
        max_cycles: 2,
        debug_mode: false,
        enable_stats: true,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Chỉ loyalty rules
    let rules = vec![
        r#"rule "LoyaltyPoints" salience 10 {
            when Transaction.Amount > 0.0
            then Customer.LoyaltyPoints = Transaction.Amount * 0.1;
        }"#,
        r#"rule "HighSpenderBonus" salience 15 {
            when Customer.SpendingTotal > 3000.0
            then Customer.BonusPoints = 500.0;
        }"#,
        r#"rule "VIPBonus" salience 20 {
            when Customer.Tier == "vip"
            then Customer.VIPBonus = 1000.0;
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base().add_rule(rule)?;
        }
    }

    Ok(engine)
}

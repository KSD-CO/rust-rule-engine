use rust_rule_engine::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🦀 Rust Rule Engine - Grule-Style Demo");
    println!("=======================================\n");

    // Demo 1: Basic GRL Rules
    println!("🔥 Demo 1: Basic GRL Rules");
    demo_basic_grule_rules()?;

    // Demo 2: E-commerce GRL Rules
    println!("\n🛒 Demo 2: E-commerce GRL Rules");
    demo_ecommerce_grule_rules()?;

    // Demo 3: Advanced GRL Features
    println!("\n⚡ Demo 3: Advanced GRL Features");
    demo_advanced_grule_rules()?;

    Ok(())
}

fn demo_basic_grule_rules() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create Knowledge Base
    let kb = KnowledgeBase::new("BasicRules");

    // Define rules in GRL format
    let grl_rules = r#"
    rule AgeVerification "Check if user is adult" salience 10 {
        when
            User.Age >= 18 && User.Country == "US"
        then
            User.IsAdult = true;
            User.CanVote = true;
            Log("User verified as adult");
    }
    
    rule SeniorDiscount "Senior citizen discount" salience 5 {
        when
            User.Age >= 65 && User.IsAdult == true
        then
            User.DiscountRate = 0.20;
            User.IsSenior = true;
            Log("Senior discount applied");
    }
    "#;

    // Load rules into Knowledge Base
    kb.add_rules_from_grl(grl_rules)?;

    // Create Grule Engine
    let mut engine = RustRuleEngine::new(kb);

    // Create Facts (working memory)
    let facts = Facts::new();
    let user = FactHelper::create_user("John", 70, "john@example.com", "US", false);
    facts.add_value("User", user)?;

    // Execute rules
    let result = engine.execute(&facts)?;

    println!("✅ Rules executed successfully");
    println!("   - Cycles: {}", result.cycle_count);
    println!("   - Rules fired: {}", result.rules_fired);
    println!("   - Execution time: {:?}", result.execution_time);

    // Show updated facts
    if let Some(user_age) = facts.get_nested("User.Age") {
        println!("   - User age: {}", user_age.to_string());
    }
    if let Some(is_adult) = facts.get_nested("User.IsAdult") {
        println!("   - Is adult: {}", is_adult.to_string());
    }
    if let Some(is_senior) = facts.get_nested("User.IsSenior") {
        println!("   - Is senior: {}", is_senior.to_string());
    }

    Ok(())
}

fn demo_ecommerce_grule_rules() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create Knowledge Base
    let kb = KnowledgeBase::new("EcommerceRules");

    // Define e-commerce rules in GRL format
    let grl_rules = r#"
    rule PremiumDiscount "Premium member discount" salience 15 {
        when
            Customer.Membership == "premium" && Order.Total > 100
        then
            Order.DiscountRate = 0.15;
            Order.FreeShipping = true;
            Log("Premium discount applied");
    }
    
    rule NewCustomerWelcome "New customer welcome bonus" salience 20 {
        when
            Customer.IsNew == true && Order.Total > 50
        then
            Order.WelcomeBonus = 10;
            Order.DiscountRate = 0.05;
            Log("Welcome bonus applied");
    }
    
    rule VIPUpgrade "VIP customer upgrade" salience 25 {
        when
            Customer.TotalSpent > 1000 && Customer.Membership != "VIP"
        then
            Customer.Membership = "VIP";
            Order.DiscountRate = 0.25;
            Log("Customer upgraded to VIP");
    }
    "#;

    // Load rules
    kb.add_rules_from_grl(grl_rules)?;

    // Create engine
    let mut engine = RustRuleEngine::new(kb);

    // Create facts
    let facts = Facts::new();

    // Add customer data
    let customer = FactHelper::create_object(vec![
        ("Membership", Value::String("premium".to_string())),
        ("IsNew", Value::Boolean(false)),
        ("TotalSpent", Value::Number(1200.0)),
    ]);
    facts.add_value("Customer", customer)?;

    // Add order data
    let order = FactHelper::create_object(vec![
        ("Total", Value::Number(150.0)),
        ("DiscountRate", Value::Number(0.0)),
        ("FreeShipping", Value::Boolean(false)),
    ]);
    facts.add_value("Order", order)?;

    // Execute rules
    let result = engine.execute(&facts)?;

    println!("✅ E-commerce rules executed");
    println!("   - Cycles: {}", result.cycle_count);
    println!("   - Rules fired: {}", result.rules_fired);

    // Show results
    if let Some(membership) = facts.get_nested("Customer.Membership") {
        println!("   - Customer membership: {}", membership.to_string());
    }
    if let Some(discount) = facts.get_nested("Order.DiscountRate") {
        println!(
            "   - Order discount: {}%",
            (discount.as_number().unwrap_or(0.0) * 100.0) as i32
        );
    }
    if let Some(shipping) = facts.get_nested("Order.FreeShipping") {
        println!("   - Free shipping: {}", shipping.to_string());
    }

    Ok(())
}

fn demo_advanced_grule_rules() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create Knowledge Base
    let kb = KnowledgeBase::new("AdvancedRules");

    // Define advanced rules with complex conditions
    let grl_rules = r#"
    rule FraudDetection "Fraud detection system" salience 30 {
        when
            Transaction.Amount > 1000 && User.Country != "US" && User.VerificationLevel < 3
        then
            Transaction.RequiresReview = true;
            Transaction.FraudScore = 85;
            Log("Transaction flagged for fraud review");
    }
    
    rule LoyaltyPoints "Loyalty points calculation" salience 10 {
        when
            Customer.Tier == "gold" && Purchase.Amount > 200
        then
            Customer.LoyaltyPoints = Customer.LoyaltyPoints + Purchase.Amount * 0.05;
            Log("Loyalty points awarded");
    }
    
    rule InventoryCheck "Low inventory alert" salience 40 {
        when
            Product.Stock < 10 && Product.IsActive == true
        then
            Product.LowStockAlert = true;
            Log("Low inventory alert triggered");
    }
    "#;

    // Load rules
    kb.add_rules_from_grl(grl_rules)?;

    // Create engine with configuration
    let config = EngineConfig {
        max_cycles: 10,
        timeout: None,
        enable_stats: true,
        debug_mode: false,
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Create facts
    let facts = Facts::new();

    // Add transaction data
    let transaction = FactHelper::create_object(vec![
        ("Amount", Value::Number(1500.0)),
        ("RequiresReview", Value::Boolean(false)),
        ("FraudScore", Value::Integer(0)),
    ]);
    facts.add_value("Transaction", transaction)?;

    // Add user data
    let user = FactHelper::create_object(vec![
        ("Country", Value::String("CA".to_string())),
        ("VerificationLevel", Value::Integer(2)),
    ]);
    facts.add_value("User", user)?;

    // Add product data
    let product = FactHelper::create_object(vec![
        ("Stock", Value::Integer(5)),
        ("IsActive", Value::Boolean(true)),
        ("LowStockAlert", Value::Boolean(false)),
    ]);
    facts.add_value("Product", product)?;

    // Execute rules
    let result = engine.execute(&facts)?;

    println!("✅ Advanced rules executed");
    println!("   - Cycles: {}", result.cycle_count);
    println!("   - Rules fired: {}", result.rules_fired);
    println!("   - Rules evaluated: {}", result.rules_evaluated);

    // Show results
    if let Some(requires_review) = facts.get_nested("Transaction.RequiresReview") {
        println!(
            "   - Transaction requires review: {}",
            requires_review.to_string()
        );
    }
    if let Some(fraud_score) = facts.get_nested("Transaction.FraudScore") {
        println!("   - Fraud score: {}", fraud_score.to_string());
    }
    if let Some(low_stock) = facts.get_nested("Product.LowStockAlert") {
        println!("   - Low stock alert: {}", low_stock.to_string());
    }

    Ok(())
}

use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl_parser::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Inline GRL Rules Demo");
    println!("========================\n");

    // Define rules directly in code as strings
    let grl_rules = r#"
        rule "HighValueCustomer" salience 20 {
            when
                Customer.TotalSpent > 1000.0
            then
                sendWelcomeEmail(Customer.Email, "GOLD");
                Customer.setTier("GOLD");
                log("Customer upgraded to GOLD tier");
        }

        rule "LoyaltyBonus" salience 15 {
            when
                Customer.OrderCount >= 10
            then
                applyLoyaltyBonus(Customer.Id, 50.0);
                Customer.setLoyaltyBonusApplied(true);
                log("Loyalty bonus applied");
        }

        rule "NewCustomerWelcome" salience 10 {
            when
                Customer.IsNew == false && Customer.WelcomeEmailSent == false
            then
                sendWelcomeEmail(Customer.Email, "EXISTING");
                Customer.setWelcomeEmailSent(true);
                log("Welcome email sent to existing customer");
        }

        rule "LowRiskTransaction" salience 5 {
            when
                Transaction.Amount < 1000.0 && Transaction.RiskProcessed == false
            then
                Transaction.setRiskProcessed(true);
                log("Low-risk transaction processed");
        }
    "#;

    println!("📋 Inline GRL Rules:");
    println!("---");
    println!("{}", grl_rules.trim());
    println!("---\n");

    // Create facts
    let facts = Facts::new();

    // Customer data
    let mut customer_props = HashMap::new();
    customer_props.insert("Id".to_string(), Value::String("CUST001".to_string()));
    customer_props.insert(
        "Email".to_string(),
        Value::String("john.doe@example.com".to_string()),
    );
    customer_props.insert("TotalSpent".to_string(), Value::Number(1250.0)); // Qualifies for GOLD
    customer_props.insert("YearsActive".to_string(), Value::Integer(4)); // Long-time customer
    customer_props.insert("OrderCount".to_string(), Value::Integer(12)); // Qualifies for loyalty
    customer_props.insert("Tier".to_string(), Value::String("SILVER".to_string()));
    customer_props.insert("IsNew".to_string(), Value::Boolean(false));
    customer_props.insert("RiskScore".to_string(), Value::Integer(35)); // Low risk
    customer_props.insert("WelcomeEmailSent".to_string(), Value::Boolean(false));
    customer_props.insert("LoyaltyBonusApplied".to_string(), Value::Boolean(false));

    // Transaction data
    let mut transaction_props = HashMap::new();
    transaction_props.insert("Id".to_string(), Value::String("TXN001".to_string()));
    transaction_props.insert("Amount".to_string(), Value::Number(750.0)); // Normal amount
    transaction_props.insert("Currency".to_string(), Value::String("USD".to_string()));
    transaction_props.insert("RiskProcessed".to_string(), Value::Boolean(false));

    facts.add_value("Customer", Value::Object(customer_props))?;
    facts.add_value("Transaction", Value::Object(transaction_props))?;

    println!("🏁 Initial state:");
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer = {customer:?}");
    }
    if let Some(transaction) = facts.get("Transaction") {
        println!("   Transaction = {transaction:?}");
    }
    println!();

    // Create knowledge base and parse inline rules
    let kb = KnowledgeBase::new("InlineRulesDemo");

    println!("🔧 Parsing inline GRL rules...");
    let parsed_rules = GRLParser::parse_rules(grl_rules)
        .map_err(|e| format!("Failed to parse inline GRL rules: {:?}", e))?;

    println!(
        "✅ Successfully parsed {} rules from inline strings",
        parsed_rules.len()
    );
    for rule in parsed_rules {
        println!("   📋 Rule: {} (salience: {})", rule.name, rule.salience);
        let _ = kb.add_rule(rule);
    }
    println!();

    // Create engine with configuration
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 1, // PREVENT INFINITE LOOPS by limiting to 1 cycle
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register custom functions called from the inline rules
    println!("📝 Registering custom functions for inline rules...");

    // Customer tier management
    engine.register_function("Customer.setTier", |args, facts| {
        let new_tier = args.get(0).unwrap().to_string();

        // ACTUALLY UPDATE THE FACTS in memory
        facts
            .set_nested("Customer.Tier", Value::String(new_tier.clone()))
            .unwrap();

        let result = format!("🏆 Customer tier updated to: {}", new_tier);
        println!("  {}", result);
        Ok(Value::String(result))
    });

    // Loyalty bonus management
    engine.register_function("Customer.setLoyaltyBonusApplied", |args, facts| {
        let applied = args.get(0).unwrap();

        // ACTUALLY UPDATE THE FACTS in memory
        facts
            .set_nested("Customer.LoyaltyBonusApplied", applied.clone())
            .unwrap();

        let result = format!("🎯 Loyalty bonus status updated: {:?}", applied);
        println!("  {}", result);
        Ok(Value::String(result))
    });

    // Email service
    engine.register_function("sendWelcomeEmail", |args, _facts| {
        let email = args.get(0).unwrap().to_string();
        let tier = args.get(1).unwrap().to_string();

        let result = format!("📧 Welcome email sent to {} for {} tier", email, tier);
        println!("  {}", result);
        Ok(Value::String(result))
    });

    // Loyalty system
    engine.register_function("applyLoyaltyBonus", |args, _facts| {
        let customer_id = args.get(0).unwrap().to_string();
        let bonus_amount = args.get(1).unwrap();

        let result = format!(
            "💰 Loyalty bonus of {:?} applied to customer {}",
            bonus_amount, customer_id
        );
        println!("  {}", result);
        Ok(Value::String(result))
    });

    // Security functions
    engine.register_function("flagForReview", |args, _facts| {
        let transaction_id = args.get(0).unwrap().to_string();

        let result = format!(
            "🚨 Transaction {} flagged for manual review",
            transaction_id
        );
        println!("  {}", result);
        Ok(Value::String(result))
    });

    engine.register_function("notifySecurityTeam", |args, _facts| {
        let customer_id = args.get(0).unwrap().to_string();
        let amount = args.get(1).unwrap();

        let result = format!(
            "🔒 Security team notified: Customer {} - Amount {:?}",
            customer_id, amount
        );
        println!("  {}", result);
        Ok(Value::String(result))
    });

    // Customer status updates
    engine.register_function("Customer.setWelcomeEmailSent", |args, facts| {
        let sent = args.get(0).unwrap();

        // ACTUALLY UPDATE THE FACTS in memory
        facts
            .set_nested("Customer.WelcomeEmailSent", sent.clone())
            .unwrap();

        let result = format!("✅ Welcome email status updated: {:?}", sent);
        println!("  {}", result);
        Ok(Value::String(result))
    });

    // Transaction status updates
    engine.register_function("Transaction.setRiskProcessed", |args, facts| {
        let processed = args.get(0).unwrap();

        // ACTUALLY UPDATE THE FACTS in memory
        facts
            .set_nested("Transaction.RiskProcessed", processed.clone())
            .unwrap();

        let result = format!("✅ Transaction risk processing completed: {:?}", processed);
        println!("  {}", result);
        Ok(Value::String(result))
    });

    println!("✅ Registered 8 custom functions for inline rules:");
    println!("   🏆 Customer.setTier");
    println!("   🎯 Customer.setLoyaltyBonusApplied");
    println!("   📧 sendWelcomeEmail");
    println!("   💰 applyLoyaltyBonus");
    println!("   🚨 flagForReview");
    println!("   🔒 notifySecurityTeam");
    println!("   ✅ Customer.setWelcomeEmailSent");
    println!("   ✅ Transaction.setRiskProcessed");
    println!();

    // Execute the inline rules
    println!("🚀 Executing inline GRL rules...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Inline Rules Execution Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    println!("\n🏁 Final state:");
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer = {customer:?}");
    }
    if let Some(transaction) = facts.get("Transaction") {
        println!("   Transaction = {transaction:?}");
    }

    println!("\n🎯 Inline GRL Rules Demonstrated:");
    println!("   📝 Rules defined as strings directly in code");
    println!("   🔧 No external files needed");
    println!("   ⚡ Quick prototyping and testing");
    println!("   🏆 Customer tier management");
    println!("   💰 Loyalty bonus system");
    println!("   🔒 Security and fraud detection");
    println!("   📧 Email notification system");

    Ok(())
}

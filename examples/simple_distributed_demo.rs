use rust_rule_engine::*;
use std::collections::HashMap;
use std::error::Error;

/// 🌐 Simple Distributed Simulation
/// 
/// This example shows the concept of distributed rule execution
/// by simulating multiple "nodes" in the same process.

fn main() -> Result<(), Box<dyn Error>> {
    println!("🌐 === Simple Distributed Rule Execution Demo ===");
    println!("Simulating distributed rule execution across 3 virtual nodes\n");

    // Create 3 "virtual nodes" - each with their own engine and rule sets
    let mut node1 = create_node("Node1", "Customer Validation Rules")?;
    let mut node2 = create_node("Node2", "Transaction Processing Rules")?; 
    let mut node3 = create_node("Node3", "Loyalty & Bonus Rules")?;

    // Create shared facts (in real distributed system, this would be in Redis/database)
    let shared_facts = create_shared_facts()?;

    println!("📊 Initial Facts:");
    display_facts(&shared_facts);

    // Node 1: Customer validation rules
    println!("\n🔨 Node1: Processing customer validation rules...");
    add_customer_rules(&mut node1)?;
    let result1 = node1.execute(&shared_facts)?;
    println!("   ✅ {} rules fired in {} cycles", result1.rules_fired, result1.cycle_count);

    // Node 2: Transaction processing rules
    println!("\n🔨 Node2: Processing transaction rules...");
    add_transaction_rules(&mut node2)?;
    let result2 = node2.execute(&shared_facts)?;
    println!("   ✅ {} rules fired in {} cycles", result2.rules_fired, result2.cycle_count);

    // Node 3: Loyalty and bonus rules  
    println!("\n🔨 Node3: Processing loyalty rules...");
    add_loyalty_rules(&mut node3)?;
    let result3 = node3.execute(&shared_facts)?;
    println!("   ✅ {} rules fired in {} cycles", result3.rules_fired, result3.cycle_count);

    println!("\n📈 Final Results After Distributed Processing:");
    display_facts(&shared_facts);

    println!("\n📊 Distributed Execution Summary:");
    println!("   Node1 (Customer): {} rules fired", result1.rules_fired);
    println!("   Node2 (Transaction): {} rules fired", result2.rules_fired);
    println!("   Node3 (Loyalty): {} rules fired", result3.rules_fired);
    println!("   Total: {} rules executed across {} nodes", 
        result1.rules_fired + result2.rules_fired + result3.rules_fired, 3);

    println!("\n🎯 Distributed Benefits Demonstrated:");
    println!("   🚀 Specialization: Each node handles specific rule types");
    println!("   ⚡ Parallel Processing: Rules can execute simultaneously"); 
    println!("   🗄️ Shared State: All nodes work on same fact base");
    println!("   📊 Coordination: Results combined for final outcome");

    Ok(())
}

fn create_node(name: &str, description: &str) -> Result<RustRuleEngine, Box<dyn Error>> {
    println!("🏗️ Creating {}: {}", name, description);
    
    let kb = KnowledgeBase::new(name);
    let config = EngineConfig {
        max_cycles: 3,
        debug_mode: false,
        enable_stats: true,
        ..Default::default()
    };
    
    Ok(RustRuleEngine::with_config(kb, config))
}

fn create_shared_facts() -> Result<Facts, Box<dyn Error>> {
    let facts = Facts::new();

    // Customer data
    let customer = FactHelper::create_customer(
        "John Smith",
        32,
        "john@example.com", 
        "standard",
        1500.0,
        false
    );
    facts.add_value("Customer", customer)?;

    // Transaction data
    let mut transaction_props = HashMap::new();
    transaction_props.insert("Amount".to_string(), Value::Number(250.0));
    transaction_props.insert("Type".to_string(), Value::String("PURCHASE".to_string()));
    transaction_props.insert("Status".to_string(), Value::String("PENDING".to_string()));
    facts.add_value("Transaction", Value::Object(transaction_props))?;

    // Account data
    let mut account_props = HashMap::new();
    account_props.insert("Balance".to_string(), Value::Number(5000.0));
    account_props.insert("Type".to_string(), Value::String("CHECKING".to_string()));
    account_props.insert("IsActive".to_string(), Value::Boolean(true));
    facts.add_value("Account", Value::Object(account_props))?;

    Ok(facts)
}

fn add_customer_rules(engine: &mut RustRuleEngine) -> Result<(), Box<dyn Error>> {
    let rules = vec![
        r#"rule "ValidateAge" salience 10 {
            when Customer.Age >= 18
            then Customer.IsAdult = true;
        }"#,
        
        r#"rule "ValidateEmail" salience 10 {
            when Customer.Email != ""
            then Customer.HasValidEmail = true;
        }"#,
        
        r#"rule "CustomerStatus" salience 5 {
            when Customer.IsAdult == true && Customer.HasValidEmail == true
            then Customer.ValidationStatus = "VERIFIED";
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base.add_rule(rule)?;
        }
    }

    Ok(())
}

fn add_transaction_rules(engine: &mut RustRuleEngine) -> Result<(), Box<dyn Error>> {
    let rules = vec![
        r#"rule "CheckBalance" salience 15 {
            when Account.Balance >= Transaction.Amount && Account.IsActive == true
            then Transaction.BalanceCheck = "PASSED";
        }"#,
        
        r#"rule "ProcessTransaction" salience 10 {
            when Transaction.BalanceCheck == "PASSED" && Customer.ValidationStatus == "VERIFIED"
            then Transaction.Status = "APPROVED";
        }"#,
        
        r#"rule "UpdateBalance" salience 5 {
            when Transaction.Status == "APPROVED"
            then Account.Balance = Account.Balance - Transaction.Amount;
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base.add_rule(rule)?;
        }
    }

    Ok(())
}

fn add_loyalty_rules(engine: &mut RustRuleEngine) -> Result<(), Box<dyn Error>> {
    let rules = vec![
        r#"rule "LoyaltyPoints" salience 10 {
            when Transaction.Status == "APPROVED"
            then Customer.LoyaltyPoints = Customer.LoyaltyPoints + Transaction.Amount * 0.1;
        }"#,
        
        r#"rule "TierUpgrade" salience 5 {
            when Customer.LoyaltyPoints > 200.0 && Customer.Tier == "standard"
            then Customer.Tier = "gold";
        }"#,
        
        r#"rule "GoldBenefits" salience 5 {
            when Customer.Tier == "gold"
            then Customer.HasPremiumBenefits = true;
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base.add_rule(rule)?;
        }
    }

    Ok(())
}

fn display_facts(facts: &Facts) {
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer: {:?}", customer);
    }
    if let Some(transaction) = facts.get("Transaction") {
        println!("   Transaction: {:?}", transaction);
    }
    if let Some(account) = facts.get("Account") {
        println!("   Account: {:?}", account);
    }
}

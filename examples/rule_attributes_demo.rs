use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Rule Attributes Enhancement Demo v0.6.0");
    println!("===========================================");

    // Demo 1: Agenda Groups
    demo_agenda_groups()?;

    // Demo 2: Activation Groups
    demo_activation_groups()?;

    // Demo 3: Lock-on-Active
    demo_lock_on_active()?;

    // Demo 4: Date Effective/Expires
    demo_date_attributes()?;

    println!("\n🎉 All Rule Attributes Enhancement demos completed successfully!");
    Ok(())
}

fn demo_agenda_groups() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 1: Agenda Groups (Workflow Control)");
    println!("--------------------------------------------");

    let grl_content = r#"
    rule "ValidateAge" agenda-group "validation" salience 10 {
        when
            User.age >= 18
        then
            User.status = "valid";
            log("Age validation passed");
    }

    rule "ValidateCredit" agenda-group "validation" salience 5 {
        when
            User.creditScore >= 600
        then
            User.creditStatus = "approved";
            log("Credit validation passed");
    }

    rule "ProcessPayment" agenda-group "processing" salience 10 {
        when
            User.status == "valid" && User.creditStatus == "approved"
        then
            Order.status = "processed";
            log("Payment processed");
    }

    rule "SendConfirmation" agenda-group "notification" salience 5 {
        when
            Order.status == "processed"
        then
            User.emailSent = true;
            log("Confirmation email sent");
    }
    "#;

    let rules = GRLParser::parse_rules(grl_content)?;
    let kb = KnowledgeBase::new("AgendaDemo");
    for rule in rules {
        let _ = kb.add_rule(rule);
    }

    let config = EngineConfig {
        max_cycles: 3,
        debug_mode: true,
        ..Default::default()
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    // Create test facts
    let facts = Facts::new();
    facts.set(
        "User",
        Value::Object({
            let mut user = HashMap::new();
            user.insert("age".to_string(), Value::Integer(25));
            user.insert("creditScore".to_string(), Value::Integer(650));
            user
        }),
    );
    facts.set("Order", Value::Object(HashMap::new()));

    println!(
        "📊 Available agenda groups: {:?}",
        engine.get_agenda_groups()
    );

    // Step 1: Validation phase
    println!("\n🔍 Phase 1: Validation (agenda-group: validation)");
    engine.set_agenda_focus("validation");
    let result1 = engine.execute(&facts)?;
    println!("   Result: {} rules fired", result1.rules_fired);

    // Step 2: Processing phase
    println!("\n⚙️  Phase 2: Processing (agenda-group: processing)");
    engine.set_agenda_focus("processing");
    let result2 = engine.execute(&facts)?;
    println!("   Result: {} rules fired", result2.rules_fired);

    // Step 3: Notification phase
    println!("\n📧 Phase 3: Notification (agenda-group: notification)");
    engine.set_agenda_focus("notification");
    let result3 = engine.execute(&facts)?;
    println!("   Result: {} rules fired", result3.rules_fired);

    println!("✅ Agenda Groups demo completed - Structured workflow execution!");

    Ok(())
}

fn demo_activation_groups() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🎯 Demo 2: Activation Groups (Mutually Exclusive Rules)");
    println!("-------------------------------------------------------");

    let grl_content = r#"
    rule "PremiumDiscount" activation-group "discount" salience 10 {
        when
            Customer.tier == "premium"
        then
            Order.discount = 0.20;
            log("Premium discount applied: 20%");
    }

    rule "GoldDiscount" activation-group "discount" salience 8 {
        when
            Customer.tier == "gold"
        then
            Order.discount = 0.15;
            log("Gold discount applied: 15%");
    }

    rule "SilverDiscount" activation-group "discount" salience 6 {
        when
            Customer.tier == "silver"
        then
            Order.discount = 0.10;
            log("Silver discount applied: 10%");
    }

    rule "StandardDiscount" activation-group "discount" salience 4 {
        when
            Customer.tier == "standard"
        then
            Order.discount = 0.05;
            log("Standard discount applied: 5%");
    }
    "#;

    let rules = GRLParser::parse_rules(grl_content)?;
    let kb = KnowledgeBase::new("ActivationDemo");
    for rule in rules {
        let _ = kb.add_rule(rule);
    }

    let config = EngineConfig {
        max_cycles: 2,
        debug_mode: true,
        ..Default::default()
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    // Test with multiple customer tiers (but only one discount should apply)
    let facts = Facts::new();
    facts.set(
        "Customer",
        Value::Object({
            let mut customer = HashMap::new();
            customer.insert("tier".to_string(), Value::String("gold".to_string()));
            customer
        }),
    );
    facts.set("Order", Value::Object(HashMap::new()));

    println!(
        "📊 Available activation groups: {:?}",
        engine.get_activation_groups()
    );
    println!("💳 Customer tier: gold");
    println!("🎯 Expected: Only ONE discount rule should fire (highest salience)");

    let result = engine.execute(&facts)?;
    println!(
        "✅ Activation Groups demo completed - Only {} rule fired (mutually exclusive)!",
        result.rules_fired
    );

    Ok(())
}

fn demo_lock_on_active() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔒 Demo 3: Lock-on-Active (One-time execution per agenda activation)");
    println!("---------------------------------------------------------------------");

    let grl_content = r#"
    rule "WelcomeEmail" lock-on-active salience 10 {
        when
            Customer.isNew == true
        then
            Customer.welcomeEmailSent = true;
            log("Welcome email sent to new customer");
    }

    rule "SetupAccount" lock-on-active salience 8 {
        when
            Customer.isNew == true
        then
            Customer.accountSetup = true;
            log("Account setup completed");
    }

    rule "RegularRule" salience 5 {
        when
            Customer.isNew == true
        then
            Customer.regularProcessing = true;
            log("Regular processing (can fire multiple times)");
    }
    "#;

    let rules = GRLParser::parse_rules(grl_content)?;
    let kb = KnowledgeBase::new("LockOnActiveDemo");
    for rule in rules {
        let _ = kb.add_rule(rule);
    }

    let config = EngineConfig {
        max_cycles: 5,
        debug_mode: true,
        ..Default::default()
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    let facts = Facts::new();
    facts.set(
        "Customer",
        Value::Object({
            let mut customer = HashMap::new();
            customer.insert("isNew".to_string(), Value::Boolean(true));
            customer
        }),
    );

    println!("👤 Customer.isNew = true");
    println!("🔒 Lock-on-active rules should fire only ONCE per agenda activation");
    println!("🔄 Regular rules can fire multiple times");

    let result = engine.execute(&facts)?;
    println!("✅ Lock-on-Active demo completed - Rules with lock-on-active fired only once!");

    Ok(())
}

fn demo_date_attributes() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n⏰ Demo 4: Date Effective/Expires (Time-based rule activation)");
    println!("--------------------------------------------------------------");

    let grl_content = r#"
    rule "ChristmasDiscount" date-effective "2025-12-01T00:00:00Z" date-expires "2025-12-31T23:59:59Z" salience 10 {
        when
            Order.total > 100
        then
            Order.christmasDiscount = 0.25;
            log("Christmas discount applied!");
    }

    rule "NewYearPromotion" date-effective "2026-01-01T00:00:00Z" date-expires "2026-01-31T23:59:59Z" salience 8 {
        when
            Customer.isVIP == true
        then
            Customer.newYearBonus = 100;
            log("New Year VIP bonus applied!");
    }

    rule "AlwaysActiveRule" salience 5 {
        when
            Order.total > 50
        then
            Order.standardProcessing = true;
            log("Standard processing (always active)");
    }
    "#;

    let rules = GRLParser::parse_rules(grl_content)?;
    let kb = KnowledgeBase::new("DateAttributesDemo");
    for rule in rules {
        let _ = kb.add_rule(rule);
    }

    let config = EngineConfig {
        max_cycles: 2,
        debug_mode: true,
        ..Default::default()
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    let facts = Facts::new();
    facts.set(
        "Order",
        Value::Object({
            let mut order = HashMap::new();
            order.insert("total".to_string(), Value::Integer(150));
            order
        }),
    );
    facts.set(
        "Customer",
        Value::Object({
            let mut customer = HashMap::new();
            customer.insert("isVIP".to_string(), Value::Boolean(true));
            customer
        }),
    );

    println!("📅 Testing with current date (October 2025)");
    println!("🎄 ChristmasDiscount: effective Dec 1-31, 2025 (NOT YET ACTIVE)");
    println!("🎆 NewYearPromotion: effective Jan 1-31, 2026 (NOT YET ACTIVE)");
    println!("✅ AlwaysActiveRule: no date restrictions (ACTIVE)");

    // Test with current time (should only fire AlwaysActiveRule)
    let result = engine.execute(&facts)?;
    println!(
        "📊 Current time result: {} rules fired (only always-active rule)",
        result.rules_fired
    );

    // Test with Christmas time
    println!("\n🎄 Testing with Christmas time (2025-12-15)");
    let christmas_time =
        chrono::DateTime::parse_from_rfc3339("2025-12-15T12:00:00Z")?.with_timezone(&chrono::Utc);
    let result_christmas = engine.execute_at_time(&facts, christmas_time)?;
    println!(
        "📊 Christmas time result: {} rules fired (should include Christmas discount)",
        result_christmas.rules_fired
    );

    println!("✅ Date Attributes demo completed - Time-based rule activation working!");

    Ok(())
}

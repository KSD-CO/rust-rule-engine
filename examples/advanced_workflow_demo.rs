use rust_rule_engine::engine::engine::{RustRuleEngine, EngineConfig};
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use rust_rule_engine::Facts;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Workflow Engine Demo - v0.8.0");
    println!("==========================================");
    println!("Demonstrating: Scheduled Tasks, Multiple Workflows, Complex Routing");

    // Create rule engine
    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 200,
        timeout: None,
        enable_stats: true,
    };
    let mut engine = RustRuleEngine::with_config(KnowledgeBase::new("AdvancedWorkflowDemo"), config);

    // Create facts for advanced order processing
    let facts = Facts::new();
    facts.set("Order.ID", Value::String("ORD-54321".to_string()));
    facts.set("Order.Amount", Value::Number(1500.0));
    facts.set("Order.Status", Value::String("pending".to_string()));
    facts.set("Customer.VIP", Value::Boolean(false)); // Regular customer
    facts.set("Customer.CreditScore", Value::Integer(720));
    facts.set("Inventory.Available", Value::Boolean(true));
    facts.set("Payment.Risk", Value::String("medium".to_string()));

    println!("\n📋 Initial Facts:");
    println!("  Order.ID = {:?}", facts.get("Order.ID"));
    println!("  Order.Amount = {:?}", facts.get("Order.Amount"));
    println!("  Customer.VIP = {:?}", facts.get("Customer.VIP"));
    println!("  Customer.CreditScore = {:?}", facts.get("Customer.CreditScore"));
    println!("  Payment.Risk = {:?}", facts.get("Payment.Risk"));

    // Advanced workflow with conditional routing and scheduled tasks
    let workflow_rules = vec![
        // Step 1: Start workflow
        r#"
        rule "StartAdvancedWorkflow" salience 100 agenda-group "start" {
            when
                Order.Status == "pending"
            then
                log("🚀 Starting advanced order processing workflow");
                ActivateAgendaGroup("risk-assessment");
        }
        "#,

        // Step 2: Risk Assessment Branch
        r#"
        rule "HighRiskAssessment" salience 90 agenda-group "risk-assessment" {
            when
                Payment.Risk == "high" || Order.Amount > 5000
            then
                log("⚠️ High risk order detected - manual review required");
                Order.Status = "manual-review";
                ScheduleRule(3000, "ReviewTimeout");
                ActivateAgendaGroup("manual-review");
        }
        "#,

        r#"
        rule "MediumRiskAssessment" salience 80 agenda-group "risk-assessment" {
            when
                Payment.Risk == "medium" && Order.Amount <= 5000 && Customer.CreditScore >= 650
            then
                log("🔍 Medium risk order - additional verification needed");
                Order.Status = "verification";
                ActivateAgendaGroup("verification");
        }
        "#,

        r#"
        rule "LowRiskAssessment" salience 70 agenda-group "risk-assessment" {
            when
                Payment.Risk == "low" && Customer.CreditScore >= 700
            then
                log("✅ Low risk order - proceeding to payment");
                Order.Status = "validated";
                ActivateAgendaGroup("payment");
        }
        "#,

        // Step 3: Verification workflow
        r#"
        rule "AdditionalVerification" salience 60 agenda-group "verification" {
            when
                Order.Status == "verification"
            then
                log("🔒 Performing additional verification checks");
                Order.Status = "verified";
                ScheduleRule(2000, "VerificationComplete");
        }
        "#,

        r#"
        rule "VerificationComplete" salience 50 {
            when
                Order.Status == "verified"
            then
                log("✅ Verification completed successfully");
                Order.Status = "validated";
                ActivateAgendaGroup("payment");
        }
        "#,

        // Step 4: Payment processing with VIP/Regular branching
        r#"
        rule "ProcessVIPPayment" salience 80 agenda-group "payment" {
            when
                Order.Status == "validated" && Customer.VIP == true
            then
                log("💳 Processing VIP payment with priority");
                Order.Status = "paid";
                Order.PaymentMethod = "VIP Express";
                ActivateAgendaGroup("fulfillment");
        }
        "#,

        r#"
        rule "ProcessRegularPayment" salience 70 agenda-group "payment" {
            when
                Order.Status == "validated" && Customer.VIP == false
            then
                log("💳 Processing regular payment");
                Order.Status = "paid";
                Order.PaymentMethod = "Standard";
                ScheduleRule(1500, "PaymentConfirmation");
        }
        "#,

        r#"
        rule "PaymentConfirmation" salience 60 {
            when
                Order.Status == "paid" && Order.PaymentMethod == "Standard"
            then
                log("✅ Payment confirmation received");
                ActivateAgendaGroup("fulfillment");
        }
        "#,

        // Step 5: Fulfillment
        r#"
        rule "ExpressFulfillment" salience 80 agenda-group "fulfillment" {
            when
                Order.Status == "paid" && (Customer.VIP == true || Order.Amount > 1000)
            then
                log("🚚 Express fulfillment initiated");
                Order.Status = "shipped";
                Order.ShippingType = "Express";
                Order.ShippingDate = "Next Day";
                ActivateAgendaGroup("completion");
        }
        "#,

        r#"
        rule "StandardFulfillment" salience 70 agenda-group "fulfillment" {
            when
                Order.Status == "paid" && Customer.VIP == false && Order.Amount <= 1000
            then
                log("📦 Standard fulfillment initiated");
                Order.Status = "shipped";
                Order.ShippingType = "Standard";
                Order.ShippingDate = "3-5 Days";
                ActivateAgendaGroup("completion");
        }
        "#,

        // Step 6: Completion
        r#"
        rule "CompleteAdvancedWorkflow" salience 40 agenda-group "completion" no-loop {
            when
                Order.Status == "shipped"
            then
                log("🎉 Advanced workflow completed successfully!");
                SetWorkflowData("order-process", final_status="completed");
                CompleteWorkflow("order-process");
        }
        "#,

        // Timeout and error handling
        r#"
        rule "ReviewTimeout" salience 30 {
            when
                Order.Status == "manual-review"
            then
                log("⏰ Manual review timeout - escalating");
                Order.Status = "escalated";
                SetWorkflowData("order-process", escalation="timeout");
        }
        "#,
    ];

    // Parse and add rules
    println!("\n📝 Adding advanced workflow rules...");
    for (i, rule_grl) in workflow_rules.iter().enumerate() {
        match GRLParser::parse_rule(rule_grl) {
            Ok(rule) => {
                println!("  ✅ Added rule: {}", rule.name);
                engine.knowledge_base().add_rule(rule).unwrap();
            }
            Err(e) => {
                println!("  ❌ Failed to parse rule {}: {}", i + 1, e);
                return Err(e.into());
            }
        }
    }

    // Start multiple workflows
    println!("\n🏁 Starting advanced workflow execution...");
    engine.start_workflow(Some("order-process".to_string()));
    engine.activate_agenda_group("start".to_string());

    // Execute initial workflow
    println!("\n⚡ Executing workflow phase 1...");
    let result1 = engine.execute(&facts)?;
    println!("📊 Phase 1 Results: {} rules evaluated, {} fired", result1.rules_evaluated, result1.rules_fired);

    // Simulate time passing and execute scheduled tasks
    println!("\n⏰ Simulating time passage for scheduled tasks...");
    let scheduled_tasks = engine.get_ready_tasks();
    println!("  Currently {} tasks are ready immediately", scheduled_tasks.len());
    
    // Wait a bit for tasks to become ready (simulating time passing)
    println!("  Waiting for scheduled tasks to become ready...");
    thread::sleep(Duration::from_millis(2100)); // Wait slightly longer than 2000ms
    
    let ready_tasks = engine.get_ready_tasks();
    if !ready_tasks.is_empty() {
        println!("  Found {} scheduled tasks ready for execution", ready_tasks.len());
        
        // Execute scheduled tasks
        engine.execute_scheduled_tasks(&facts)?;
        
        println!("\n⚡ Executing workflow phase 2 (after scheduled tasks)...");
        let result2 = engine.execute(&facts)?;
        println!("📊 Phase 2 Results: {} rules evaluated, {} fired", result2.rules_evaluated, result2.rules_fired);
        
        // Continue normal workflow execution to process the state change
        println!("\n⚡ Continuing workflow execution...");
        let result2b = engine.execute(&facts)?;
        println!("📊 Phase 2b Results: {} rules evaluated, {} fired", result2b.rules_evaluated, result2b.rules_fired);
    } else {
        println!("  No scheduled tasks became ready");
    }

    // Check for more scheduled tasks
    thread::sleep(Duration::from_millis(1600)); // Wait for payment confirmation
    let remaining_tasks = engine.get_ready_tasks();
    if !remaining_tasks.is_empty() {
        println!("\n⏰ Processing remaining scheduled tasks...");
        println!("  Found {} more tasks ready", remaining_tasks.len());
        engine.execute_scheduled_tasks(&facts)?;
        
        println!("\n⚡ Executing workflow phase 3 (final)...");
        let result3 = engine.execute(&facts)?;
        println!("📊 Phase 3 Results: {} rules evaluated, {} fired", result3.rules_evaluated, result3.rules_fired);
    }

    // Show final results
    println!("\n📋 Final Order State:");
    println!("  Order.ID = {:?}", facts.get("Order.ID"));
    println!("  Order.Status = {:?}", facts.get("Order.Status"));
    println!("  Order.PaymentMethod = {:?}", facts.get("Order.PaymentMethod"));
    println!("  Order.ShippingType = {:?}", facts.get("Order.ShippingType"));
    println!("  Order.ShippingDate = {:?}", facts.get("Order.ShippingDate"));

    // Workflow statistics
    println!("\n📈 Final Workflow Statistics:");
    let stats = engine.get_workflow_stats();
    println!("  Total workflows: {}", stats.total_workflows);
    println!("  Completed workflows: {}", stats.completed_workflows);
    println!("  Running workflows: {}", stats.running_workflows);
    println!("  Failed workflows: {}", stats.failed_workflows);

    println!("\n🏆 Advanced workflow demo completed successfully!");
    println!("✨ Demonstrated: Conditional routing, scheduled tasks, risk assessment, and complex business logic");
    
    Ok(())
}

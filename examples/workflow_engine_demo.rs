use rust_rule_engine::engine::engine::{RustRuleEngine, EngineConfig};
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use rust_rule_engine::Facts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Workflow Engine Demo - v0.8.0");
    println!("=================================");

    // Create rule engine with minimal debug
    let config = EngineConfig {
        debug_mode: false,  // Disable debug for cleaner output
        max_cycles: 100,
        timeout: None,
        enable_stats: true,
    };
    let mut engine = RustRuleEngine::with_config(KnowledgeBase::new("WorkflowDemo"), config);

    // Create facts for order processing workflow
    let mut facts = Facts::new();
    facts.set("Order.ID", Value::String("ORD-12345".to_string()));
    facts.set("Order.Amount", Value::Number(250.0));
    facts.set("Order.Status", Value::String("pending".to_string()));
    facts.set("Customer.VIP", Value::Boolean(true));
    facts.set("Inventory.Available", Value::Boolean(true));

    println!("\n📋 Initial Facts:");
    println!("  Order.ID = {:?}", facts.get("Order.ID"));
    println!("  Order.Amount = {:?}", facts.get("Order.Amount"));
    println!("  Order.Status = {:?}", facts.get("Order.Status"));
    println!("  Customer.VIP = {:?}", facts.get("Customer.VIP"));
    println!("  Inventory.Available = {:?}", facts.get("Inventory.Available"));

    // Define workflow rules using GRL syntax
    let workflow_rules = vec![
        // Step 1: Start workflow and validate order
        r#"
        rule "StartOrderWorkflow" salience 100 agenda-group "start" {
            when
                Order.Status == "pending"
            then
                log("🔄 Starting order processing workflow");
                ActivateAgendaGroup("validation");
                SetWorkflowData("order-process", status="started");
        }
        "#,

        // Step 2: Validate order details
        r#"
        rule "ValidateOrder" salience 90 agenda-group "validation" {
            when
                Order.Amount > 0 && Inventory.Available == true
            then
                log("✅ Order validation passed");
                Order.Status = "validated";
                ActivateAgendaGroup("payment");
        }
        "#,

        // Step 3: Process payment (VIP customers get priority)
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

        // Step 3b: Process regular payment
        r#"
        rule "ProcessRegularPayment" salience 70 agenda-group "payment" {
            when
                Order.Status == "validated" && Customer.VIP == false
            then
                log("💳 Processing regular payment");
                Order.Status = "paid";
                Order.PaymentMethod = "Standard";
                ScheduleRule(2000, "CheckPaymentStatus");
        }
        "#,

        // Step 4: Schedule payment verification (for demo of scheduling)
        r#"
        rule "CheckPaymentStatus" salience 60 {
            when
                Order.Status == "paid"
            then
                log("🔍 Payment verification completed");
                ActivateAgendaGroup("fulfillment");
        }
        "#,

        // Step 5: Fulfill order
        r#"
        rule "FulfillOrder" salience 50 agenda-group "fulfillment" {
            when
                Order.Status == "paid"
            then
                log("📦 Order fulfillment started");
                Order.Status = "shipped";
                Order.ShippingDate = "2024-01-15";
                ActivateAgendaGroup("completion");
        }
        "#,

        // Step 6: Complete workflow
        r#"
        rule "CompleteOrderWorkflow" salience 40 agenda-group "completion" no-loop {
            when
                Order.Status == "shipped"
            then
                log("🎉 Order processing workflow completed!");
                SetWorkflowData("order-process", status="completed");
                CompleteWorkflow("order-process");
        }
        "#,
    ];

    // Parse and add rules to engine
    println!("\n📝 Adding workflow rules...");
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

    // Start workflow by activating initial agenda group
    println!("\n🏁 Starting workflow execution...");
    engine.start_workflow(Some("order-process".to_string()));
    
    // Set initial agenda group to "start"
    engine.activate_agenda_group("start".to_string());

    // Execute rules
    println!("\n⚡ Executing workflow...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Execution Results:");
    println!("  Rules evaluated: {}", result.rules_evaluated);
    println!("  Rules fired: {}", result.rules_fired);
    println!("  Execution cycles: {}", result.cycle_count);
    println!("  Total time: {:?}", result.execution_time);

    // Show final facts state
    println!("\n📋 Final Facts:");
    println!("  Order.ID = {:?}", facts.get("Order.ID"));
    println!("  Order.Amount = {:?}", facts.get("Order.Amount"));
    println!("  Order.Status = {:?}", facts.get("Order.Status"));
    println!("  Customer.VIP = {:?}", facts.get("Customer.VIP"));
    println!("  Inventory.Available = {:?}", facts.get("Inventory.Available"));
    println!("  Order.PaymentMethod = {:?}", facts.get("Order.PaymentMethod"));
    println!("  Order.ShippingDate = {:?}", facts.get("Order.ShippingDate"));

    // Process any scheduled tasks
    println!("\n⏰ Processing scheduled tasks...");
    let scheduled_tasks = engine.get_ready_tasks();
    if !scheduled_tasks.is_empty() {
        println!("  Found {} scheduled tasks", scheduled_tasks.len());
        // In a real scenario, you would execute these after their delay
        for task in scheduled_tasks {
            println!("  📅 Scheduled task: {} (execution time: {:?})", task.rule_name, task.execute_at);
        }
    } else {
        println!("  No scheduled tasks found");
    }

    // Show workflow statistics
    println!("\n📈 Workflow Statistics:");
    let stats = engine.get_workflow_stats();
    println!("  Total workflows: {}", stats.total_workflows);
    println!("  Running workflows: {}", stats.running_workflows);
    println!("  Completed workflows: {}", stats.completed_workflows);
    println!("  Failed workflows: {}", stats.failed_workflows);
    println!("  Pending scheduled tasks: {}", stats.pending_scheduled_tasks);
    println!("  Pending agenda activations: {}", stats.pending_agenda_activations);

    println!("\n🏆 Workflow demo completed successfully!");
    Ok(())
}

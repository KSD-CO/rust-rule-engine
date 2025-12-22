use rust_rule_engine::engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use rust_rule_engine::Facts;

// Helper: convert parser ConditionGroup to auto_network ConditionGroup
fn convert_condition_group(
    src: &rust_rule_engine::ConditionGroup,
) -> rust_rule_engine::rete::auto_network::ConditionGroup {
    use rust_rule_engine::rete::auto_network::{
        Condition as AutoCond, ConditionGroup as AutoGroup,
    };
    match src {
        rust_rule_engine::ConditionGroup::Single(cond) => {
            // Map operator enum to RETE-UL string
            let op_str = match format!("{:?}", cond.operator).as_str() {
                "Eq" => "==",
                "Ne" => "!=",
                "Gt" => ">",
                "Lt" => "<",
                "Ge" => ">=",
                "Le" => "<=",
                _ => "==",
            };
            // Use correct value string
            let val_str = match &cond.value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Boolean(b) => b.to_string(),
                _ => cond.value.to_string(),
            };
            // Extract field name from expression
            let field_name = match &cond.expression {
                rust_rule_engine::engine::rule::ConditionExpression::Field(f) => f.clone(),
                rust_rule_engine::engine::rule::ConditionExpression::FunctionCall {
                    name, ..
                } => name.clone(),
                rust_rule_engine::engine::rule::ConditionExpression::Test { name, .. } => {
                    name.clone()
                }
                rust_rule_engine::engine::rule::ConditionExpression::MultiField {
                    field, ..
                } => field.clone(),
            };
            AutoGroup::Single(AutoCond {
                field: field_name,
                operator: op_str.to_string(),
                value: val_str,
            })
        }
        rust_rule_engine::ConditionGroup::Compound {
            left,
            operator,
            right,
        } => {
            let op_str = match format!("{:?}", operator).as_str() {
                "And" => "AND",
                "Or" => "OR",
                _ => "AND",
            };
            AutoGroup::Compound {
                left: Box::new(convert_condition_group(left)),
                operator: op_str.to_string(),
                right: Box::new(convert_condition_group(right)),
            }
        }
        rust_rule_engine::ConditionGroup::Not(inner) => {
            AutoGroup::Not(Box::new(convert_condition_group(inner)))
        }
        rust_rule_engine::ConditionGroup::Exists(inner) => {
            AutoGroup::Exists(Box::new(convert_condition_group(inner)))
        }
        rust_rule_engine::ConditionGroup::Forall(inner) => {
            AutoGroup::Forall(Box::new(convert_condition_group(inner)))
        }
        rust_rule_engine::ConditionGroup::Accumulate { .. } => {
            // Accumulate is not supported in auto_network yet
            AutoGroup::Single(AutoCond {
                field: "true".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            })
        }
        _ => {
            // Stream patterns (or unknown condition groups) are not supported in this demo conversion;
            // fallback to a no-op true condition so the example builds without the "streaming" feature.
            AutoGroup::Single(AutoCond {
                field: "true".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            })
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Workflow Engine Demo - v0.8.0");
    println!("=================================");

    // Create rule engine with minimal debug
    let config = EngineConfig {
        debug_mode: false, // Disable debug for cleaner output
        max_cycles: 100,
        timeout: None,
        enable_stats: true,
    };
    let mut engine = RustRuleEngine::with_config(KnowledgeBase::new("WorkflowDemo"), config);

    // Create facts for order processing workflow
    let facts = Facts::new();
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
    println!(
        "  Inventory.Available = {:?}",
        facts.get("Inventory.Available")
    );

    // Define workflow rules using GRL syntax
    let workflow_rules = [
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
    let mut rete_rules = Vec::new();
    for (i, rule_grl) in workflow_rules.iter().enumerate() {
        match GRLParser::parse_rule(rule_grl) {
            Ok(rule) => {
                println!("  ✅ Added rule: {}", rule.name);
                engine.knowledge_base().add_rule(rule.clone()).unwrap();
                // Chuyển sang auto_network::Rule để test với RETE
                let auto_rule = rust_rule_engine::rete::auto_network::Rule {
                    name: rule.name.clone(),
                    conditions: convert_condition_group(&rule.conditions),
                    action: rule
                        .actions
                        .first()
                        .map(|a| format!("{:?}", a))
                        .unwrap_or_default(),
                };
                rete_rules.push(auto_rule);
            }
            Err(e) => {
                println!("  ❌ Failed to parse rule {}: {}", i + 1, e);
                return Err(e.into());
            }
        }
    }

    // --- Chạy với engine gốc ---
    use std::time::Instant;
    println!("\n🏁 Starting workflow execution (engine gốc)...");
    engine.start_workflow(Some("order-process".to_string()));
    engine.activate_agenda_group("start".to_string());
    println!("\n⚡ Executing workflow (engine gốc)...");
    let start_engine = Instant::now();
    let result = engine.execute(&facts)?;
    let duration_engine = start_engine.elapsed();
    println!("\n📊 Execution Results (engine gốc):");
    println!("  Rules evaluated: {}", result.rules_evaluated);
    println!("  Rules fired: {}", result.rules_fired);
    println!("  Execution cycles: {}", result.cycle_count);
    println!("  Total time (engine): {:?}", duration_engine);

    // --- Chạy với RETE-UL node ---
    use rust_rule_engine::rete::auto_network::build_rete_ul_from_rule;
    use rust_rule_engine::rete::evaluate_rete_ul_node;

    println!("\n🔬 So sánh với RETE-UL node:");
    let mut facts_map = std::collections::HashMap::new();
    for (k, v) in facts.get_all_facts().iter() {
        facts_map.insert(k.clone(), v.to_string());
    }
    let start_rete = Instant::now();
    let mut rete_fired = 0;
    for rule in &rete_rules {
        let rete_node = build_rete_ul_from_rule(rule);
        let matched = evaluate_rete_ul_node(&rete_node, &facts_map);
        println!("  Rule: {:<25} | RETE match: {}", rule.name, matched);
        if matched {
            rete_fired += 1;
        }
    }
    let duration_rete = start_rete.elapsed();
    println!(
        "\n📊 RETE-UL: Số rule match: {} / {}",
        rete_fired,
        rete_rules.len()
    );
    println!("\n⏱️ So sánh tốc độ:");
    println!("  Engine gốc:   {:?}", duration_engine);
    println!("  RETE-UL node: {:?}", duration_rete);

    // Show final facts state
    println!("\n📋 Final Facts:");
    println!("  Order.ID = {:?}", facts.get("Order.ID"));
    println!("  Order.Amount = {:?}", facts.get("Order.Amount"));
    println!("  Order.Status = {:?}", facts.get("Order.Status"));
    println!("  Customer.VIP = {:?}", facts.get("Customer.VIP"));
    println!(
        "  Inventory.Available = {:?}",
        facts.get("Inventory.Available")
    );
    println!(
        "  Order.PaymentMethod = {:?}",
        facts.get("Order.PaymentMethod")
    );
    println!(
        "  Order.ShippingDate = {:?}",
        facts.get("Order.ShippingDate")
    );

    // Verify workflow state was saved
    println!("\n🔍 Workflow State Verification:");
    println!(
        "  workflow.order-process.completed = {:?}",
        facts.get("workflow.order-process.completed")
    );
    println!(
        "  workflow.order-process.completed_at = {:?}",
        facts.get("workflow.order-process.completed_at")
    );
    println!(
        "  workflow.data.status = {:?}",
        facts.get("workflow.data.status")
    );

    // Process any scheduled tasks
    println!("\n⏰ Processing scheduled tasks...");
    let scheduled_tasks = engine.get_ready_tasks();
    if !scheduled_tasks.is_empty() {
        println!("  Found {} scheduled tasks", scheduled_tasks.len());
        for task in scheduled_tasks {
            println!(
                "  📅 Scheduled task: {} (execution time: {:?})",
                task.rule_name, task.execute_at
            );
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
    println!(
        "  Pending scheduled tasks: {}",
        stats.pending_scheduled_tasks
    );
    println!(
        "  Pending agenda activations: {}",
        stats.pending_agenda_activations
    );

    println!("\n🏆 Workflow demo completed successfully!");
    Ok(())
}

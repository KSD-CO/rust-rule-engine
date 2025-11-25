/// E-commerce Order Approval System - Real-World Example
///
/// Scenario: An automated approval engine for orders based on:
/// - Customer purchase history
/// - Loyalty / risk signals
/// - Order value
/// - Payment status
///
/// Backward chaining is used to answer: "Is this order auto-approved?"
///
/// Rules & Queries:
/// - examples/rules/09-backward-chaining/ecommerce_approval.grl (business rules)
/// - examples/rules/09-backward-chaining/ecommerce_queries.grl (backward-chaining queries)
///
/// HYBRID APPROACH:
/// 1. Forward chaining: derive facts (VIP status, risk level, payment verification)
/// 2. Backward chaining: evaluate compound goals (AutoApproved && !RequiresManualReview)
///
/// Demonstrates compound goal support (&&, !=) in the backward query executor.

use rust_rule_engine::{Facts, KnowledgeBase, Value, GRLParser};
use std::fs;
use std::path::Path;

#[cfg(feature = "backward-chaining")]
use rust_rule_engine::backward::{BackwardEngine, GRLQueryParser, GRLQueryExecutor};

/// Load rules from a .grl file
fn load_rules_from_file(filename: &str) -> String {
    let path = Path::new("examples/rules/09-backward-chaining").join(filename);
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("❌ Could not read file: {}", path.display()))
}

/// Load a query block from a .grl file
fn load_query_from_file(filename: &str, query_name: &str) -> String {
    let content = load_rules_from_file(filename);
    
    // Extract query block from file
    let query_start = format!("query \"{}\"", query_name);
    if let Some(start_idx) = content.find(&query_start) {
        // Tìm closing brace của query
        let remaining = &content[start_idx..];
        let mut brace_count = 0;
        let mut in_query = false;
        let mut end_idx = 0;
        
        for (i, ch) in remaining.chars().enumerate() {
            if ch == '{' {
                brace_count += 1;
                in_query = true;
            } else if ch == '}' {
                brace_count -= 1;
                if in_query && brace_count == 0 {
                    end_idx = i + 1;
                    break;
                }
            }
        }
        
        if end_idx > 0 {
            return remaining[..end_idx].to_string();
        }
    }
    
    panic!("❌ Query '{}' not found in file {}", query_name, filename);
}

fn main() {
    #[cfg(not(feature = "backward-chaining"))]
    {
    println!("❌ This example requires the 'backward-chaining' feature");
    println!("   Run: cargo run --example ecommerce_approval_demo --features backward-chaining");
        return;
    }

    #[cfg(feature = "backward-chaining")]
    {
        println!("🛒 E-COMMERCE ORDER APPROAL SYSTEM");
        println!("====================================\n");

        scenario_1_vip_customer();
        scenario_2_new_customer_small_order();
        scenario_3_risky_large_order();
        scenario_4_batch_approval();
    }
}

#[cfg(feature = "backward-chaining")]
fn scenario_1_vip_customer() {
    println!("📦 SCENARIO 1: VIP customer buying a 5,000,000 order");
    println!("─────────────────────────────────────────\n");

    // Load rules from .grl file
    let rules = load_rules_from_file("ecommerce_approval.grl");

    let mut kb = KnowledgeBase::new("VIPApproval");
    let parsed_rules = GRLParser::parse_rules(&rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    // Load query from file
    let query_str = load_query_from_file("ecommerce_queries.grl", "CheckAutoApproval");

    // Thông tin khách hàng thực tế
    let mut facts = Facts::new();
    facts.set("Customer.Name", Value::String("Nguyen Van A".to_string()));
    facts.set("Customer.LoyaltyPoints", Value::Number(150.0));
    facts.set("Customer.YearlySpending", Value::Number(25000000.0));
    
    facts.set("Order.Amount", Value::Number(5000000.0));
    facts.set("Order.Items", Value::String("iPhone 15 Pro Max".to_string()));

    println!("👤 CUSTOMER INFORMATION:");
    println!("   Name: Nguyen Van A");
    println!("   Loyalty points: 150");
    println!("   Year-to-date spending: 25,000,000 VND");

    println!("\n📦 ORDER SUMMARY:");
    println!("   Product: iPhone 15 Pro Max");
    println!("   Amount: 5,000,000 VND");

    println!("\n🔍 AUTO-APPROVAL CHECK...");
    println!("   Reasoning chain:");
    println!("   1. Loyalty 150 → VIP");
    println!("   2. Order 5,000,000 < 10,000,000 → within threshold");
    println!("   3. VIP + under threshold → auto-approve");

    // Execute query
    let query = GRLQueryParser::parse(&query_str).unwrap();
    let mut bc_engine = BackwardEngine::new(kb);
    let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts).unwrap();

    if result.provable {
    println!("\n✅ RESULT: AUTO-APPROVED");
    println!("   Status: {}", facts.get("Order.Status").map(|v| format!("{:?}", v)).unwrap_or_default());
    println!("   Processing time: Instant");
    println!("   Reason: VIP customer - Auto approved");
    }

    println!();
}

#[cfg(feature = "backward-chaining")]
fn scenario_2_new_customer_small_order() {
    println!("📦 SCENARIO 2: New customer placing a 500,000 order");
    println!("───────────────────────────────────────\n");

    // Load rules from .grl file
    let rules = load_rules_from_file("ecommerce_approval.grl");

    let mut kb = KnowledgeBase::new("SmallOrder");
    let parsed_rules = GRLParser::parse_rules(&rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    // Load query from file
    let query_str = load_query_from_file("ecommerce_queries.grl", "CheckAutoApproval");

    let mut facts = Facts::new();
    facts.set("Customer.Name", Value::String("Tran Thi B".to_string()));
    facts.set("Customer.AccountAge", Value::String("New".to_string()));
    
    facts.set("Order.Amount", Value::Number(500000.0));
    facts.set("Order.Items", Value::String("Áo thun Nike".to_string()));
    facts.set("Payment.Method", Value::String("COD".to_string()));

    println!("👤 CUSTOMER INFORMATION:");
    println!("   Name: Tran Thi B");
    println!("   Account age: NEW (no history)");

    println!("\n📦 ORDER SUMMARY:");
    println!("   Product: Nike T-shirt");
    println!("   Amount: 500,000 VND");
    println!("   Payment: COD (cash on delivery)");

    println!("\n🔍 AUTO-APPROVAL CHECK...");
    println!("   Reasoning:");
    println!("   1. Order 500,000 < 2,000,000 → small order");
    println!("   2. Payment COD → low payment risk");
    println!("   3. Small + COD → auto-approve");

    let query = GRLQueryParser::parse(&query_str).unwrap();
    let mut bc_engine = BackwardEngine::new(kb);
    let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts).unwrap();

    if result.provable {
    println!("\n✅ RESULT: AUTO-APPROVED");
    println!("   Reason: small order + COD (low payment risk)");
    }

    println!();
}

#[cfg(feature = "backward-chaining")]
fn scenario_3_risky_large_order() {
    println!("📦 SCENARIO 3: Large order 50,000,000 - New account");
    println!("────────────────────────────────────────────────\n");

    // Load rules from .grl file
    let rules = load_rules_from_file("ecommerce_approval.grl");

    let mut kb = KnowledgeBase::new("RiskyOrder");
    let parsed_rules = GRLParser::parse_rules(&rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    // Load query from file
    let query_str = load_query_from_file("ecommerce_queries.grl", "CheckAutoApproval");

    let mut facts = Facts::new();
    facts.set("Customer.Name", Value::String("Le Van C".to_string()));
    facts.set("Customer.AccountAge", Value::String("New".to_string()));
    
    facts.set("Order.Amount", Value::Number(50000000.0));
    facts.set("Order.Items", Value::String("Laptop Dell XPS 15 x2".to_string()));
    facts.set("Payment.Method", Value::String("Bank Transfer".to_string()));

    println!("👤 CUSTOMER INFORMATION:");
    println!("   Name: Le Van C");
    println!("   Account age: NEW (created yesterday)");

    println!("\n📦 ORDER SUMMARY:");
    println!("   Product: Laptop Dell XPS 15 x2");
    println!("   Amount: 50,000,000 VND");
    println!("   Payment: Bank transfer");

    println!("\n🔍 AUTO-APPROVAL CHECK...");
    println!("   Reasoning:");
    println!("   1. Order >= 20,000,000 → large order");
    println!("   2. Account age: NEW → no history");
    println!("   3. Large + new account → HIGH RISK (manual review)");

    let query = GRLQueryParser::parse(&query_str).unwrap();
    let mut bc_engine = BackwardEngine::new(kb);
    let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts).unwrap();

    if !result.provable {
    println!("\n⏳ RESULT: REQUIRES MANUAL REVIEW");
    println!("   Status: {}", facts.get("Order.Status").map(|v| format!("{:?}", v)).unwrap_or("PENDING_REVIEW".to_string()));
    println!("   Reasons:");
    println!("   • High order value (50,000,000)");
    println!("   • New account with no history");
    println!("   • Needs CS verification");
        
    println!("\n📋 NEXT STEPS:");
    println!("   1. Call customer to verify");
    println!("   2. Check bank transfer details");
    println!("   3. CS lead approval");
    println!("   Estimated handling time: 1-2 business days");
    } else {
        println!("\n✅ RESULT: AUTO-APPROVED (Unexpected - should require manual review)");
    }

    println!();
}

#[cfg(feature = "backward-chaining")]
fn scenario_4_batch_approval() {
    println!("📦 SCENARIO 4: Batch processing sample orders");
    println!("──────────────────────────────────────────────\n");

    // Load rules from .grl file
    let rules = load_rules_from_file("ecommerce_approval.grl");

    let mut kb = KnowledgeBase::new("BatchApproval");
    let parsed_rules = GRLParser::parse_rules(&rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    // Load query from file
    let query_str = load_query_from_file("ecommerce_queries.grl", "CheckAutoApproval");

    println!("🔄 Processing a batch of sample orders...\n");

    // Simulate 10 sample orders - mix of auto vs manual review
    let orders = vec![
        // Auto approve: VIP + small orders
        ("Order #1001", 500000.0, 150.0, "COD", "Existing"),          // VIP, small → Auto
        ("Order #1002", 8000000.0, 200.0, "Bank", "Existing"),        // VIP, under 10M → Auto
        ("Order #1003", 1500000.0, 50.0, "COD", "Existing"),          // Small COD → Auto

        // Need manual review: High risk cases
        ("Order #1004", 25000000.0, 80.0, "Bank", "New"),             // 25M + New account → REVIEW
        ("Order #1005", 55000000.0, 250.0, "Bank", "Existing"),       // 55M critical → REVIEW
        ("Order #1006", 15000000.0, 30.0, "Bank Transfer", "New"),    // 15M + New + Bank → REVIEW

        // More auto cases
        ("Order #1007", 900000.0, 20.0, "COD", "New"),                // Tiny order → Auto
        ("Order #1008", 3000000.0, 120.0, "COD", "Existing"),         // VIP + small → Auto

        // More review cases
        ("Order #1009", 45000000.0, 40.0, "Bank", "New"),             // 45M + New → REVIEW
        ("Order #1010", 18000000.0, 15.0, "Bank Transfer", "New"),    // 18M + New + Bank → REVIEW
    ];

    let mut auto_approved = 0;
    let mut manual_review = 0;

    let query = GRLQueryParser::parse(&query_str).unwrap();

    // DEBUG: Test first order only
    let (order_id, amount, loyalty, payment, account_age) = orders[0];
    
    let mut facts = Facts::new();
    facts.set("Order.Amount", Value::Number(amount));
    facts.set("Customer.LoyaltyPoints", Value::Number(loyalty));
    facts.set("Payment.Method", Value::String(payment.to_string()));
    facts.set("Customer.AccountAge", Value::String(account_age.to_string()));

    println!("📋 Initial facts:");
    for key in facts.get_all_facts().keys() {
        println!("   {} = {:?}", key, facts.get(key));
    }

    use rust_rule_engine::RustRuleEngine;
    let mut engine = RustRuleEngine::new(kb.clone());
    
    // Register LogMessage handler to prevent errors
    engine.register_action_handler("LogMessage", |_args, _facts| {
        // Silently ignore log messages during batch processing
        Ok(())
    });
    
    let exec_result = engine.execute(&mut facts);
    println!("\n📋 After forward chaining:");
    println!("   Execution result: {:?}", exec_result);
    for key in facts.get_all_facts().keys() {
        println!("   {} = {:?}", key, facts.get(key));
    }

    let mut bc_engine = BackwardEngine::new(kb.clone());
    let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts).unwrap();
    
    println!("\n📋 Backward query result:");
    println!("   Provable: {}", result.provable);
    println!("   Goal: {}", query.goal);
    
    // Process all orders
    for (order_id, amount, loyalty, payment, account_age) in &orders {
        let mut facts = Facts::new();
        facts.set("Order.Amount", Value::Number(*amount));
        facts.set("Customer.LoyaltyPoints", Value::Number(*loyalty));
        facts.set("Payment.Method", Value::String(payment.to_string()));
        facts.set("Customer.AccountAge", Value::String(account_age.to_string()));

    // HYBRID APPROACH (Forward + Backward Chaining):
    //
    // 1. FORWARD CHAINING: execute all rules to derive facts
    //    - VIP detection, risk assessment, payment verification
    //    - auto-approval rules will set Order.AutoApproved when applicable
    //
    // 2. BACKWARD CHAINING: evaluate the compound goal
    //    Goal: Order.AutoApproved == true && Order.RequiresManualReview != true
    //    - Both sub-goals must be satisfied for a provable=true result
    //
    // NOTE: current backward engine focuses on goal checking; full
    // backward rule execution (firing actions from rules) is a separate
    // task. This example uses forward chaining to derive facts first.
        
        use rust_rule_engine::RustRuleEngine;
        let mut engine = RustRuleEngine::new(kb.clone());
        
        // Register LogMessage handler
        engine.register_action_handler("LogMessage", |_args, _facts| Ok(()));
        
        engine.execute(&mut facts).ok();

        // NOW use backward chaining to check the compound goal
        let mut bc_engine = BackwardEngine::new(kb.clone());
        let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts).unwrap();

        let status = if result.provable {
            auto_approved += 1;
            "✅ Auto"
        } else {
            manual_review += 1;
            "⏳ Manual"
        };

    println!("   {} - {:>12} VND - {:>3} pts - {:>13} - {} → {}",
                 order_id,
                 format!("{:.0}", amount),
                 format!("{:.0}", loyalty),
                 payment,
                 account_age,
                 status);
    }

    println!("\n📊 PROCESSING SUMMARY:");
    println!("   ✅ Auto-approved: {} orders ({}%)", 
             auto_approved,
             (auto_approved * 100) / (auto_approved + manual_review));
    println!("   ⏳ Manual review: {} orders ({}%)",
             manual_review,
             (manual_review * 100) / (auto_approved + manual_review));
    
    println!("\n💡 INSIGHTS:");
    println!("   • Backward chaining helped auto-approve {}% of orders",
             (auto_approved * 100) / (auto_approved + manual_review));
    println!("   • Reduced CS workload");
    println!("   • Customers receive instant feedback when possible");

    println!();
}

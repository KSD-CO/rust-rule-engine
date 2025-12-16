/// Purchasing Flow System - Backward Chaining Example
///
/// Scenario: Automated purchasing decision system
///
/// This example demonstrates backward chaining for purchasing decisions:
/// - Should a purchase order be created?
/// - Is the order approved?
/// - Does it qualify for bulk discount?
///
/// Rules & Queries:
/// - examples/rules/09-backward-chaining/purchasing_flow.grl (business rules)
/// - examples/rules/09-backward-chaining/purchasing_queries.grl (backward queries)
///
/// HYBRID APPROACH:
/// 1. Forward chaining: derive facts (shortage, order_qty, totals, approval status)
/// 2. Backward chaining: answer questions (should create PO? is approved?)
use rust_rule_engine::{Facts, GRLParser, KnowledgeBase, RustRuleEngine, Value};
use std::fs;
use std::path::Path;

#[cfg(feature = "backward-chaining")]
use rust_rule_engine::backward::{BackwardEngine, GRLQueryExecutor, GRLQueryParser};

/// Load rules from a .grl file
fn load_rules_from_file(filename: &str) -> String {
    let path = Path::new("examples/rules/09-backward-chaining").join(filename);
    fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("❌ Could not read file: {}", path.display()))
}

/// Load a query block from a .grl file
fn load_query_from_file(filename: &str, query_name: &str) -> String {
    let content = load_rules_from_file(filename);

    let query_start = format!("query \"{}\"", query_name);
    if let Some(start_idx) = content.find(&query_start) {
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
        println!("   Run: cargo run --example purchasing_flow_demo --features backward-chaining");
        return;
    }

    #[cfg(feature = "backward-chaining")]
    {
        println!("🏭 PURCHASING FLOW DECISION SYSTEM");
        println!("====================================\n");

        scenario_1_normal_reorder();
        scenario_2_high_value_order();
    }
}

#[cfg(feature = "backward-chaining")]
fn scenario_1_normal_reorder() {
    println!("📦 SCENARIO 1: Normal reorder (shortage = 150, MOQ = 100)");
    println!("──────────────────────────────────────────────────────\n");

    // Load rules
    let rules = load_rules_from_file("purchasing_flow.grl");
    let kb = KnowledgeBase::new("PurchasingFlow");
    let parsed_rules = GRLParser::parse_rules(&rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    // Initial facts
    let mut facts = Facts::new();
    facts.set("required_qty", Value::Number(500.0));
    facts.set("available_qty", Value::Number(100.0));
    facts.set("moq", Value::Number(100.0));
    facts.set("unit_price", Value::Number(120.0));
    facts.set("is_active", Value::Boolean(true));

    println!("📊 INPUT:");
    println!("   Required qty: 200");
    println!("   Available qty: 50");
    println!("   MOQ: 100");
    println!("   Unit price: $25");

    // Step 1: Forward chaining
    println!("\n🔄 Step 1: Forward chaining (deriving facts)...\n");
    let mut forward_engine = RustRuleEngine::new(kb.clone());

    // Register LogMessage handler
    forward_engine.register_action_handler("LogMessage", |_args, _facts| Ok(()));

    forward_engine.execute(&mut facts).unwrap();

    println!("✅ Forward chaining complete. Derived facts:");
    println!("   Shortage: {:?}", facts.get("shortage"));
    println!("   Order Qty: {:?}", facts.get("order_qty"));
    println!("   Total Amount: {:?}", facts.get("total_amount"));
    println!("   Approval Status: {:?}", facts.get("approval_status"));

    // Step 2: Backward chaining
    println!("\n🔍 Step 2: Backward chaining (answering questions)...\n");

    // Query: Should create PO?
    println!("❓ Question: Should we create a purchase order?");
    let query = load_query_from_file("purchasing_queries.grl", "ShouldCreatePO");
    let parsed_query = GRLQueryParser::parse(&query).unwrap();
    let mut bc_engine = BackwardEngine::new(kb);
    let result = GRLQueryExecutor::execute(&parsed_query, &mut bc_engine, &mut facts).unwrap();

    if result.provable {
        println!("✅ ANSWER: Yes, create PO");
        println!("   PO Status: {:?}", facts.get("po_status"));
        println!("   Order Qty: {:?}", facts.get("order_qty"));
        println!("   Grand Total: {:?}", facts.get("grand_total"));
    } else {
        println!("❌ ANSWER: No, cannot create PO");
    }

    println!("\n{}\n", "=".repeat(50));
}

#[cfg(feature = "backward-chaining")]
fn scenario_2_high_value_order() {
    println!("💰 SCENARIO 2: High value order (requires approval)");
    println!("──────────────────────────────────────────────────────\n");

    let rules = load_rules_from_file("purchasing_flow.grl");
    let kb = KnowledgeBase::new("HighValueOrder");
    let parsed_rules = GRLParser::parse_rules(&rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    let mut facts = Facts::new();
    facts.set("required_qty", Value::Number(1000.0));
    facts.set("available_qty", Value::Number(200.0));
    facts.set("moq", Value::Number(100.0));
    facts.set("unit_price", Value::Number(50.0));
    facts.set("is_active", Value::Boolean(true));

    println!("📊 INPUT:");
    println!("   Required qty: 1000");
    println!("   Available qty: 200");
    println!("   MOQ: 100");
    println!("   Unit price: $50");

    println!("\n🔄 Step 1: Forward chaining...\n");
    let mut forward_engine = RustRuleEngine::new(kb.clone());

    // Register LogMessage handler
    forward_engine.register_action_handler("LogMessage", |_args, _facts| Ok(()));

    forward_engine.execute(&mut facts).unwrap();

    println!("✅ Forward chaining complete:");
    println!("   Total Amount: {:?}", facts.get("total_amount"));
    println!("   Final Amount: {:?}", facts.get("final_amount"));
    println!("   Grand Total: {:?}", facts.get("grand_total"));

    println!("\n🔍 Step 2: Backward chaining queries...\n");

    // Query: Is order approved?
    println!("❓ Question: Is this order auto-approved?");
    let query = load_query_from_file("purchasing_queries.grl", "IsOrderApproved");
    let parsed_query = GRLQueryParser::parse(&query).unwrap();
    let mut bc_engine = BackwardEngine::new(kb.clone());
    let result = GRLQueryExecutor::execute(&parsed_query, &mut bc_engine, &mut facts).unwrap();

    if result.provable {
        println!("✅ ANSWER: Yes, auto-approved");
    } else {
        println!("⏳ ANSWER: No, requires manual approval");
        println!("   Total: {:?}", facts.get("total_amount"));
    }

    // Query: Qualifies for discount?
    println!("\n❓ Question: Does this order qualify for bulk discount?");
    let query = load_query_from_file("purchasing_queries.grl", "QualifiesForDiscount");
    let parsed_query = GRLQueryParser::parse(&query).unwrap();
    let mut bc_engine = BackwardEngine::new(kb);
    let result = GRLQueryExecutor::execute(&parsed_query, &mut bc_engine, &mut facts).unwrap();

    if result.provable {
        println!("🎉 ANSWER: Yes, bulk discount applied!");
        println!("   Discount: {:?}", facts.get("discount_amount"));
        println!("   Final: {:?}", facts.get("final_amount"));
    } else {
        println!("💵 ANSWER: No discount applicable");
    }

    println!("\n{}\n", "=".repeat(50));
}

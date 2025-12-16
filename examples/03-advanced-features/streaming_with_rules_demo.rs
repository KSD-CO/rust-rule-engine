//! Streaming + Rule Engine Integration Demo
//!
//! This example shows how to combine **streaming operators** with **Rule Engine**
//! to create real-time decision-making systems using proper rule evaluation.
//!
//! Key concept: Stream operators process events → Rule Engine evaluates → Actions taken
//!
//! Run with: cargo run --example streaming_with_rules_demo --features streaming

use rust_rule_engine::engine::{facts::Facts, knowledge_base::KnowledgeBase, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::streaming::*;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::fs;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Streaming + Rule Engine Integration Demo");
    println!("{}", "=".repeat(80));
    println!("\n💡 Shows how to use REAL Rule Engine with streaming\n");

    demo1_fraud_detection_with_rules()?;
    demo2_dynamic_pricing_with_rules()?;
    demo3_compliance_with_rules()?;

    println!("\n{}", "=".repeat(80));
    println!("✅ All 3 demos completed!");
    println!("\n📝 Key Takeaways:");
    println!("   ✅ Stream operators (.map, .filter, .for_each) process events");
    println!("   ✅ Rule Engine (KnowledgeBase + Facts) evaluates business rules");
    println!("   ✅ Rules loaded from GRL files (not hardcoded!)");
    println!("   ✅ Facts updated based on rule conditions and actions");
    println!("   ✅ Fully integrated real-time decision system");
    println!("\n🎯 This is the CORRECT way to use Rule Engine with Streaming!");
    println!("\n📚 Advanced features like state management & watermarks");
    println!("   are available in the streaming module (state.rs, watermark.rs)");

    Ok(())
}

/// Demo 1: Fraud Detection using Rule Engine
fn demo1_fraud_detection_with_rules() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💳 Demo 1: Fraud Detection with Rule Engine");
    println!("{}", "-".repeat(80));

    // Load rules from GRL file instead of hardcoding
    let grl_content =
        fs::read_to_string("examples/03-advanced-features/streaming_fraud_detection.grl")?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    let kb = KnowledgeBase::new("FraudDetection");
    for rule in rules {
        kb.add_rule(rule)?;
    }

    let engine = Arc::new(Mutex::new(RustRuleEngine::new(kb)));

    println!("📋 Rules loaded from: streaming_fraud_detection.grl");
    println!("💰 Processing payment transactions with Rule Engine...\n");

    // Generate sample transactions
    let payments = generate_payment_stream();

    let stream = DataStream::from_events(payments);

    stream
        .map(move |e| {
            // Convert StreamEvent to Facts for rule engine
            let facts = Facts::new();

            // Add transaction data
            let mut tx_data = HashMap::new();
            if let Some(amount) = e.get_numeric("Transaction.Amount") {
                tx_data.insert("Amount".to_string(), Value::Number(amount));
            }
            if let Some(tx_type) = e.get_string("Transaction.Type") {
                tx_data.insert("Type".to_string(), Value::String(tx_type.to_string()));
            }
            if let Some(merchant) = e.get_string("Transaction.MerchantCategory") {
                tx_data.insert(
                    "MerchantCategory".to_string(),
                    Value::String(merchant.to_string()),
                );
            }
            tx_data.insert("Status".to_string(), Value::String("APPROVED".to_string()));

            // Initialize risk and alert
            let mut risk_data = HashMap::new();
            risk_data.insert("Score".to_string(), Value::Number(0.0));

            let mut alert_data = HashMap::new();
            alert_data.insert("Type".to_string(), Value::String("NONE".to_string()));
            alert_data.insert("RequiresReview".to_string(), Value::Boolean(false));

            let _ = facts.add_value("Transaction", Value::Object(tx_data));
            let _ = facts.add_value("Risk", Value::Object(risk_data));
            let _ = facts.add_value("Alert", Value::Object(alert_data));

            // Execute rule engine
            let mut eng = engine.lock().unwrap();
            let _ = eng.execute(&facts);

            // Extract results back to StreamEvent
            let mut result_data = e.data.clone();
            if let Some(Value::Object(tx)) = facts.get("Transaction") {
                if let Some(status) = tx.get("Status") {
                    result_data.insert("Status".to_string(), status.clone());
                }
            }
            if let Some(Value::Object(risk)) = facts.get("Risk") {
                if let Some(score) = risk.get("Score") {
                    result_data.insert("RiskScore".to_string(), score.clone());
                }
            }
            if let Some(Value::Object(alert)) = facts.get("Alert") {
                if let Some(alert_type) = alert.get("Type") {
                    result_data.insert("AlertType".to_string(), alert_type.clone());
                }
            }

            // Return event with updated data
            StreamEvent::new(e.event_type, result_data, e.metadata.source)
        })
        .filter(|e| {
            // Show only flagged transactions
            e.get_string("AlertType").unwrap_or("") != "NONE"
                || e.get_string("Status").unwrap_or("") == "BLOCKED"
        })
        .for_each(|e| {
            let tx_id = e.get_string("Transaction.ID").unwrap_or("");
            let amount = e.get_numeric("Transaction.Amount").unwrap_or(0.0);
            let status = e.get_string("Status").unwrap_or("APPROVED");
            let risk = e.get_numeric("RiskScore").unwrap_or(0.0);
            let alert = e.get_string("AlertType").unwrap_or("NONE");

            let icon = if status == "BLOCKED" {
                "🚫"
            } else {
                "⚠️ "
            };
            println!(
                "{} TX-{} | ${:.2} | {} | Risk: {:.0} | Alert: {}",
                icon, tx_id, amount, status, risk, alert
            );
        });

    println!("\n✅ Fraud detection completed using Rule Engine");
    Ok(())
}

/// Demo 2: Dynamic Pricing with Rules
fn demo2_dynamic_pricing_with_rules() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n💵 Demo 2: Dynamic Pricing with Rule Engine");
    println!("{}", "-".repeat(80));

    // Load pricing rules from GRL file
    let grl_content = fs::read_to_string("examples/03-advanced-features/streaming_pricing.grl")?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    let kb = KnowledgeBase::new("DynamicPricing");
    for rule in rules {
        kb.add_rule(rule)?;
    }

    let engine = Arc::new(Mutex::new(RustRuleEngine::new(kb)));

    println!("📋 Rules loaded from: streaming_pricing.grl");
    println!("💰 Processing product pricing...\n");

    let products = vec![
        ("Laptop", 1000.0, 150.0, 30.0),
        ("Phone", 800.0, 80.0, 15.0),
        ("Tablet", 600.0, 120.0, 45.0),
    ];

    let mut events = Vec::new();
    for (name, price, demand, inventory) in products {
        let mut data = HashMap::new();
        data.insert("Product.Name".to_string(), Value::String(name.to_string()));
        data.insert("Product.BasePrice".to_string(), Value::Number(price));
        data.insert("Product.Demand".to_string(), Value::Number(demand));
        data.insert("Product.Inventory".to_string(), Value::Number(inventory));
        events.push(StreamEvent::new("PriceUpdate", data, "pricing"));
    }

    let stream = DataStream::from_events(events);

    stream
        .map(move |e| {
            let facts = Facts::new();

            let mut product_data = HashMap::new();
            if let Some(name) = e.get_string("Product.Name") {
                product_data.insert("Name".to_string(), Value::String(name.to_string()));
            }
            if let Some(price) = e.get_numeric("Product.BasePrice") {
                product_data.insert("BasePrice".to_string(), Value::Number(price));
            }
            if let Some(demand) = e.get_numeric("Product.Demand") {
                product_data.insert("Demand".to_string(), Value::Number(demand));
            }
            if let Some(inventory) = e.get_numeric("Product.Inventory") {
                product_data.insert("Inventory".to_string(), Value::Number(inventory));
            }

            let mut pricing_data = HashMap::new();
            pricing_data.insert("Multiplier".to_string(), Value::Number(1.0));
            pricing_data.insert("Reason".to_string(), Value::String("NORMAL".to_string()));

            let _ = facts.add_value("Product", Value::Object(product_data));
            let _ = facts.add_value("Pricing", Value::Object(pricing_data));

            let mut eng = engine.lock().unwrap();
            let _ = eng.execute(&facts);

            let mut result_data = e.data.clone();
            if let Some(Value::Object(pricing)) = facts.get("Pricing") {
                if let Some(multiplier) = pricing.get("Multiplier") {
                    result_data.insert("Pricing.Multiplier".to_string(), multiplier.clone());
                }
                if let Some(reason) = pricing.get("Reason") {
                    result_data.insert("Pricing.Reason".to_string(), reason.clone());
                }
            }

            // Return event with updated data
            StreamEvent::new(e.event_type, result_data, e.metadata.source)
        })
        .for_each(|e| {
            let name = e.get_string("Product.Name").unwrap_or("");
            let base = e.get_numeric("Product.BasePrice").unwrap_or(0.0);
            let multiplier = e.get_numeric("Pricing.Multiplier").unwrap_or(1.0);
            let reason = e.get_string("Pricing.Reason").unwrap_or("NORMAL");
            let final_price = base * multiplier;

            let icon = if multiplier > 1.0 { "📈" } else { "➡️ " };
            println!(
                "{} {} | Base: ${:.2} | Multiplier: {:.1}x | Final: ${:.2} | Reason: {}",
                icon, name, base, multiplier, final_price, reason
            );
        });

    println!("\n✅ Dynamic pricing completed using Rule Engine");
    Ok(())
}

/// Demo 3: Compliance Checking with Rules
fn demo3_compliance_with_rules() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n⚖️  Demo 3: Compliance Checking with Rule Engine");
    println!("{}", "-".repeat(80));

    // Load compliance rules from GRL file
    let grl_content = fs::read_to_string("examples/03-advanced-features/streaming_compliance.grl")?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    let kb = KnowledgeBase::new("Compliance");
    for rule in rules {
        kb.add_rule(rule)?;
    }

    let engine = Arc::new(Mutex::new(RustRuleEngine::new(kb)));

    println!("📋 Rules loaded from: streaming_compliance.grl");
    println!("💼 Processing transactions...\n");

    let transactions = vec![
        ("TX-001", 5000.0, "US", "company-A"),
        ("TX-002", 15000.0, "high_risk", "company-B"),
        ("TX-003", 20000.0, "UK", "sanctioned-entity"),
    ];

    let mut events = Vec::new();
    for (id, amount, country, counterparty) in transactions {
        let mut data = HashMap::new();
        data.insert("Transaction.ID".to_string(), Value::String(id.to_string()));
        data.insert("Transaction.Amount".to_string(), Value::Number(amount));
        data.insert(
            "Transaction.Country".to_string(),
            Value::String(country.to_string()),
        );
        data.insert(
            "Transaction.Counterparty".to_string(),
            Value::String(counterparty.to_string()),
        );
        events.push(StreamEvent::new("Transaction", data, "payment"));
    }

    let stream = DataStream::from_events(events);

    stream
        .map(move |e| {
            let facts = Facts::new();

            let mut tx_data = HashMap::new();
            if let Some(id) = e.get_string("Transaction.ID") {
                tx_data.insert("ID".to_string(), Value::String(id.to_string()));
            }
            if let Some(amount) = e.get_numeric("Transaction.Amount") {
                tx_data.insert("Amount".to_string(), Value::Number(amount));
            }
            if let Some(country) = e.get_string("Transaction.Country") {
                tx_data.insert("Country".to_string(), Value::String(country.to_string()));
            }
            if let Some(counterparty) = e.get_string("Transaction.Counterparty") {
                tx_data.insert(
                    "Counterparty".to_string(),
                    Value::String(counterparty.to_string()),
                );
            }

            let mut compliance_data = HashMap::new();
            compliance_data.insert("Status".to_string(), Value::String("APPROVED".to_string()));
            compliance_data.insert("Flag".to_string(), Value::String("OK".to_string()));

            let _ = facts.add_value("Transaction", Value::Object(tx_data));
            let _ = facts.add_value("Compliance", Value::Object(compliance_data));

            let mut eng = engine.lock().unwrap();
            let _ = eng.execute(&facts);

            let mut result_data = e.data.clone();
            if let Some(Value::Object(compliance)) = facts.get("Compliance") {
                if let Some(status) = compliance.get("Status") {
                    result_data.insert("Compliance.Status".to_string(), status.clone());
                }
                if let Some(flag) = compliance.get("Flag") {
                    result_data.insert("Compliance.Flag".to_string(), flag.clone());
                }
            }

            // Return event with updated data
            StreamEvent::new(e.event_type, result_data, e.metadata.source)
        })
        .for_each(|e| {
            let id = e.get_string("Transaction.ID").unwrap_or("");
            let amount = e.get_numeric("Transaction.Amount").unwrap_or(0.0);
            let status = e.get_string("Compliance.Status").unwrap_or("APPROVED");
            let flag = e.get_string("Compliance.Flag").unwrap_or("OK");

            let icon = if status == "BLOCKED" {
                "🚫"
            } else if flag != "OK" {
                "⚠️ "
            } else {
                "✅"
            };
            println!("{} {} | ${:.2} | {} | {}", icon, id, amount, status, flag);
        });

    println!("\n✅ Compliance checking completed using Rule Engine");
    Ok(())
}

// Helper: Generate sample payment transactions
fn generate_payment_stream() -> Vec<StreamEvent> {
    let mut events = Vec::new();
    let categories = ["retail", "gambling", "electronics", "crypto", "retail"];

    for i in 0..10 {
        let mut data = HashMap::new();
        let amount = 2000.0 + (i as f64 * 2000.0);

        data.insert(
            "Transaction.ID".to_string(),
            Value::String(format!("{:03}", i)),
        );
        data.insert("Transaction.Amount".to_string(), Value::Number(amount));
        data.insert(
            "Transaction.Type".to_string(),
            Value::String("credit_card".to_string()),
        );
        data.insert(
            "Transaction.MerchantCategory".to_string(),
            Value::String(categories[i % 5].to_string()),
        );

        events.push(StreamEvent::new("Payment", data, "payment-gateway"));
    }

    events
}

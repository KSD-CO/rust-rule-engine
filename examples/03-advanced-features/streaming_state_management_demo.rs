//! Stateful Stream Processing with Rule Engine
//!
//! This example demonstrates using StateStore for stateful operations
//! combined with Rule Engine for complex business logic.
//!
//! Use cases:
//! - Session tracking with user behavior rules
//! - Aggregations with threshold alerts
//! - Stateful fraud detection
//!
//! Run with: cargo run --example streaming_state_management_demo --features streaming

use rust_rule_engine::streaming::*;
use rust_rule_engine::streaming::state::{StateStore, StateBackend, StateConfig};
use rust_rule_engine::types::Value;
use rust_rule_engine::engine::{
    RustRuleEngine,
    rule::{Rule, Condition, ConditionGroup},
    knowledge_base::KnowledgeBase,
    facts::Facts,
};
use rust_rule_engine::parser::grl::GRLParser;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Stateful Stream Processing + Rule Engine Demo");
    println!("{}", "=".repeat(80));
    
    demo1_session_tracking_with_rules()?;
    demo2_aggregation_with_alerts()?;
    demo3_stateful_fraud_detection()?;
    
    println!("\n{}", "=".repeat(80));
    println!("✅ All stateful demos completed!");
    println!("\n📝 Key Features Demonstrated:");
    println!("   ✅ StateStore for maintaining state across events");
    println!("   ✅ Rule Engine evaluates business logic on stateful data");
    println!("   ✅ Session tracking, aggregations, and fraud detection");
    
    Ok(())
}

/// Demo 1: Session tracking with user behavior rules
fn demo1_session_tracking_with_rules() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n👤 Demo 1: Session Tracking with State + Rules");
    println!("{}", "-".repeat(80));

    // Create stateful store
    let state = Arc::new(Mutex::new(StateStore::new(StateBackend::Memory)));
    
    // Load session rules from GRL
    let grl_rules = r#"
rule SuspiciousActivity "Flag suspicious session patterns" salience 100 {
    when
        Session.EventCount > 50 &&
        Session.Duration < 300
    then
        Session.Status = "SUSPICIOUS";
        Session.Alert = "TOO_MANY_EVENTS";
}

rule InactiveSession "Flag inactive sessions" salience 90 {
    when
        Session.Duration > 1800 &&
        Session.LastEventTime > 600
    then
        Session.Status = "INACTIVE";
}

rule HighValueUser "Upgrade high-value user sessions" salience 80 {
    when
        Session.TotalValue > 10000
    then
        Session.Tier = "GOLD";
}
"#;

    let rules = GRLParser::parse_rules(grl_rules)?;
    let kb = KnowledgeBase::new("SessionTracking");
    for rule in rules {
        kb.add_rule(rule)?;
    }
    let engine = Arc::new(Mutex::new(RustRuleEngine::new(kb)));

    println!("📋 Loaded 3 session tracking rules");
    println!("💻 Processing user events...\n");

    // Simulate user events
    let user_events = vec![
        ("user-1", "login", 0.0),
        ("user-1", "view_product", 100.0),
        ("user-1", "add_to_cart", 500.0),
        ("user-2", "login", 0.0),
        ("user-1", "checkout", 5000.0),
        ("user-2", "view_product", 50.0),
        ("user-1", "view_product", 200.0),
        ("user-2", "checkout", 8000.0),
    ];

    let mut events = Vec::new();
    for (user_id, event_type, value) in user_events {
        let mut data = HashMap::new();
        data.insert("user_id".to_string(), Value::String(user_id.to_string()));
        data.insert("event_type".to_string(), Value::String(event_type.to_string()));
        data.insert("value".to_string(), Value::Number(value));
        events.push(StreamEvent::new("UserEvent", data, "app"));
    }

    let stream = DataStream::from_events(events);

    stream.for_each(move |e| {
        let user_id = e.get_string("user_id").unwrap_or("");
        let event_type = e.get_string("event_type").unwrap_or("");
        let value = e.get_numeric("value").unwrap_or(0.0);

        // Get or create session state
        let session_key = format!("session:{}", user_id);
        let mut state_lock = state.lock().unwrap();
        
        let mut session_data = if let Ok(Some(existing)) = state_lock.get(&session_key) {
            if let Value::Object(map) = existing {
                map
            } else {
                HashMap::new()
            }
        } else {
            let mut new_session = HashMap::new();
            new_session.insert("EventCount".to_string(), Value::Number(0.0));
            new_session.insert("TotalValue".to_string(), Value::Number(0.0));
            new_session.insert("Duration".to_string(), Value::Number(0.0));
            new_session.insert("LastEventTime".to_string(), Value::Number(0.0));
            new_session.insert("Status".to_string(), Value::String("ACTIVE".to_string()));
            new_session.insert("Tier".to_string(), Value::String("STANDARD".to_string()));
            new_session
        };

        // Update session state
        if let Some(Value::Number(count)) = session_data.get("EventCount") {
            session_data.insert("EventCount".to_string(), Value::Number(count + 1.0));
        }
        if let Some(Value::Number(total)) = session_data.get("TotalValue") {
            session_data.insert("TotalValue".to_string(), Value::Number(total + value));
        }

        // Update state store
        let _ = state_lock.put(&session_key, Value::Object(session_data.clone()));
        drop(state_lock);

        // Evaluate rules on session data
        let facts = Facts::new();
        let _ = facts.add_value("Session", Value::Object(session_data.clone()));
        
        let mut eng = engine.lock().unwrap();
        let _ = eng.execute(&facts);

        // Extract rule results
        if let Some(Value::Object(updated_session)) = facts.get("Session") {
            let status = updated_session.get("Status")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("ACTIVE");
            let tier = updated_session.get("Tier")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("STANDARD");
            let count = updated_session.get("EventCount")
                .and_then(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .unwrap_or(0.0);
            let total = updated_session.get("TotalValue")
                .and_then(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .unwrap_or(0.0);

            let icon = match status {
                "SUSPICIOUS" => "🚨",
                "INACTIVE" => "💤",
                _ if tier == "GOLD" => "⭐",
                _ => "✅",
            };

            println!("{} {} | {} | Events: {:.0} | Value: ${:.0} | Status: {} | Tier: {}",
                     icon, user_id, event_type, count, total, status, tier);

            // Save updated session back to state
            let mut state_lock = state.lock().unwrap();
            let _ = state_lock.put(&session_key, Value::Object(updated_session.clone()));
        }
    });

    println!("\n✅ Session tracking completed");
    Ok(())
}

/// Demo 2: Aggregation with threshold alerts
fn demo2_aggregation_with_alerts() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📊 Demo 2: Aggregation with Threshold Alerts");
    println!("{}", "-".repeat(80));

    let state = Arc::new(Mutex::new(StateStore::new(StateBackend::Memory)));
    
    // Load aggregation alert rules
    let grl_rules = r#"
rule HighVolumeAlert "Alert on high transaction volume" salience 100 {
    when
        Metrics.TotalTransactions > 100
    then
        Metrics.Alert = "HIGH_VOLUME";
}

rule HighValueAlert "Alert on high total value" salience 90 {
    when
        Metrics.TotalValue > 50000
    then
        Metrics.Alert = "HIGH_VALUE";
}

rule LowAverageValue "Flag low average transaction value" salience 80 {
    when
        Metrics.AverageValue < 100
    then
        Metrics.Warning = "LOW_AVG";
}
"#;

    let rules = GRLParser::parse_rules(grl_rules)?;
    let kb = KnowledgeBase::new("Aggregation");
    for rule in rules {
        kb.add_rule(rule)?;
    }
    let engine = Arc::new(Mutex::new(RustRuleEngine::new(kb)));

    println!("📋 Loaded 3 aggregation alert rules");
    println!("💳 Processing transactions...\n");

    // Simulate transactions
    let mut events = Vec::new();
    for i in 0..120 {
        let mut data = HashMap::new();
        data.insert("tx_id".to_string(), Value::String(format!("TX-{:03}", i)));
        data.insert("amount".to_string(), Value::Number(100.0 + (i as f64 * 50.0)));
        data.insert("category".to_string(), Value::String(format!("cat-{}", i % 3)));
        events.push(StreamEvent::new("Transaction", data, "payment"));
    }

    let stream = DataStream::from_events(events);

    stream.for_each(move |e| {
        let category = e.get_string("category").unwrap_or("");
        let amount = e.get_numeric("amount").unwrap_or(0.0);

        // Get or create aggregation state
        let agg_key = format!("agg:{}", category);
        let mut state_lock = state.lock().unwrap();
        
        let mut metrics = if let Ok(Some(existing)) = state_lock.get(&agg_key) {
            if let Value::Object(map) = existing {
                map
            } else {
                HashMap::new()
            }
        } else {
            let mut new_metrics = HashMap::new();
            new_metrics.insert("TotalTransactions".to_string(), Value::Number(0.0));
            new_metrics.insert("TotalValue".to_string(), Value::Number(0.0));
            new_metrics.insert("AverageValue".to_string(), Value::Number(0.0));
            new_metrics.insert("Alert".to_string(), Value::String("NONE".to_string()));
            new_metrics.insert("Warning".to_string(), Value::String("NONE".to_string()));
            new_metrics
        };

        // Update aggregations
        if let Some(Value::Number(count)) = metrics.get("TotalTransactions") {
            let new_count = count + 1.0;
            metrics.insert("TotalTransactions".to_string(), Value::Number(new_count));
            
            if let Some(Value::Number(total)) = metrics.get("TotalValue") {
                let new_total = total + amount;
                metrics.insert("TotalValue".to_string(), Value::Number(new_total));
                metrics.insert("AverageValue".to_string(), Value::Number(new_total / new_count));
            }
        }

        // Update state
        let _ = state_lock.put(&agg_key, Value::Object(metrics.clone()));
        drop(state_lock);

        // Evaluate rules
        let facts = Facts::new();
        let _ = facts.add_value("Metrics", Value::Object(metrics.clone()));
        
        let mut eng = engine.lock().unwrap();
        let _ = eng.execute(&facts);

        // Check for alerts
        if let Some(Value::Object(updated_metrics)) = facts.get("Metrics") {
            let alert = updated_metrics.get("Alert")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("NONE");
            let count = updated_metrics.get("TotalTransactions")
                .and_then(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                .unwrap_or(0.0);

            if alert != "NONE" && count % 20.0 == 1.0 {
                let total = updated_metrics.get("TotalValue")
                    .and_then(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                    .unwrap_or(0.0);
                let avg = updated_metrics.get("AverageValue")
                    .and_then(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                    .unwrap_or(0.0);

                println!("🚨 ALERT | {} | Count: {:.0} | Total: ${:.0} | Avg: ${:.0} | Alert: {}",
                         category, count, total, avg, alert);
            }

            // Save updated metrics
            let mut state_lock = state.lock().unwrap();
            let _ = state_lock.put(&agg_key, Value::Object(updated_metrics.clone()));
        }
    });

    println!("\n✅ Aggregation with alerts completed");
    Ok(())
}

/// Demo 3: Stateful fraud detection
fn demo3_stateful_fraud_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n🛡️  Demo 3: Stateful Fraud Detection");
    println!("{}", "-".repeat(80));

    let state = Arc::new(Mutex::new(StateStore::new(StateBackend::Memory)));
    
    // Load fraud detection rules
    let grl_rules = r#"
rule RapidTransactions "Detect rapid transaction pattern" salience 100 {
    when
        UserMetrics.TransactionsLast5Min > 10
    then
        FraudScore.Value = 50;
        FraudScore.Reason = "RAPID_TRANSACTIONS";
}

rule VelocityCheck "Check transaction velocity" salience 90 {
    when
        UserMetrics.TotalLast5Min > 20000
    then
        FraudScore.Value = 40;
        FraudScore.Reason = "HIGH_VELOCITY";
}

rule BlockHighFraud "Block transactions with high fraud score" salience 80 {
    when
        FraudScore.Value >= 40
    then
        Transaction.Status = "BLOCKED";
}
"#;

    let rules = GRLParser::parse_rules(grl_rules)?;
    let kb = KnowledgeBase::new("FraudDetection");
    for rule in rules {
        kb.add_rule(rule)?;
    }
    let engine = Arc::new(Mutex::new(RustRuleEngine::new(kb)));

    println!("📋 Loaded 3 fraud detection rules");
    println!("💳 Processing transactions with stateful tracking...\n");

    // Simulate transactions with some fraudulent patterns
    let mut events = Vec::new();
    let users = vec!["user-A", "user-B", "user-C"];
    for i in 0..30 {
        let user = users[i % 3];
        let amount = if user == "user-B" && i > 10 { 5000.0 } else { 200.0 }; // user-B has high velocity
        
        let mut data = HashMap::new();
        data.insert("tx_id".to_string(), Value::String(format!("TX-{:03}", i)));
        data.insert("user_id".to_string(), Value::String(user.to_string()));
        data.insert("amount".to_string(), Value::Number(amount));
        events.push(StreamEvent::new("Transaction", data, "payment"));
    }

    let stream = DataStream::from_events(events);

    stream.for_each(move |e| {
        let user_id = e.get_string("user_id").unwrap_or("");
        let tx_id = e.get_string("tx_id").unwrap_or("");
        let amount = e.get_numeric("amount").unwrap_or(0.0);

        // Get user metrics from state
        let metrics_key = format!("fraud_metrics:{}", user_id);
        let mut state_lock = state.lock().unwrap();
        
        let mut user_metrics = if let Ok(Some(existing)) = state_lock.get(&metrics_key) {
            if let Value::Object(map) = existing {
                map
            } else {
                HashMap::new()
            }
        } else {
            let mut new_metrics = HashMap::new();
            new_metrics.insert("TransactionsLast5Min".to_string(), Value::Number(0.0));
            new_metrics.insert("TotalLast5Min".to_string(), Value::Number(0.0));
            new_metrics
        };

        // Update metrics
        if let Some(Value::Number(count)) = user_metrics.get("TransactionsLast5Min") {
            user_metrics.insert("TransactionsLast5Min".to_string(), Value::Number(count + 1.0));
        }
        if let Some(Value::Number(total)) = user_metrics.get("TotalLast5Min") {
            user_metrics.insert("TotalLast5Min".to_string(), Value::Number(total + amount));
        }

        let _ = state_lock.put(&metrics_key, Value::Object(user_metrics.clone()));
        drop(state_lock);

        // Evaluate fraud rules
        let facts = Facts::new();
        let _ = facts.add_value("UserMetrics", Value::Object(user_metrics));
        
        let mut tx_data = HashMap::new();
        tx_data.insert("ID".to_string(), Value::String(tx_id.to_string()));
        tx_data.insert("Amount".to_string(), Value::Number(amount));
        tx_data.insert("Status".to_string(), Value::String("APPROVED".to_string()));
        let _ = facts.add_value("Transaction", Value::Object(tx_data));

        let mut fraud_score = HashMap::new();
        fraud_score.insert("Value".to_string(), Value::Number(0.0));
        fraud_score.insert("Reason".to_string(), Value::String("NONE".to_string()));
        let _ = facts.add_value("FraudScore", Value::Object(fraud_score));

        let mut eng = engine.lock().unwrap();
        let _ = eng.execute(&facts);

        // Check results
        if let Some(Value::Object(tx)) = facts.get("Transaction") {
            let status = tx.get("Status")
                .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                .unwrap_or("APPROVED");
            
            if let Some(Value::Object(score)) = facts.get("FraudScore") {
                let fraud_value = score.get("Value")
                    .and_then(|v| if let Value::Number(n) = v { Some(*n) } else { None })
                    .unwrap_or(0.0);
                let reason = score.get("Reason")
                    .and_then(|v| if let Value::String(s) = v { Some(s.as_str()) } else { None })
                    .unwrap_or("NONE");

                if status == "BLOCKED" {
                    println!("🚫 {} | {} | ${:.0} | BLOCKED | Fraud: {:.0} | Reason: {}",
                             user_id, tx_id, amount, fraud_value, reason);
                } else if fraud_value > 0.0 {
                    println!("⚠️  {} | {} | ${:.0} | {} | Fraud: {:.0}",
                             user_id, tx_id, amount, status, fraud_value);
                }
            }
        }
    });

    println!("\n✅ Stateful fraud detection completed");
    Ok(())
}

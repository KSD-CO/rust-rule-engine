use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚨 Advanced Action Handlers from GRL File Demo");
    println!("===============================================");

    // Create facts for testing
    let facts = Facts::new();

    // Add comprehensive test data
    let mut customer_props = HashMap::new();
    customer_props.insert(
        "name".to_string(),
        Value::String("Alice Johnson".to_string()),
    );
    customer_props.insert(
        "email".to_string(),
        Value::String("alice.johnson@example.com".to_string()),
    );
    customer_props.insert("tier".to_string(), Value::String("VIP".to_string()));
    customer_props.insert("total_spent".to_string(), Value::Number(12500.0));
    customer_props.insert("welcome_sent".to_string(), Value::Boolean(false));
    facts.add_value("Customer", Value::Object(customer_props))?;

    // Add order data
    let mut order_props = HashMap::new();
    order_props.insert("id".to_string(), Value::String("ORD-002".to_string()));
    order_props.insert("total".to_string(), Value::Number(3500.0));
    order_props.insert("status".to_string(), Value::String("pending".to_string()));
    order_props.insert("alert_sent".to_string(), Value::Boolean(false));
    order_props.insert("processed".to_string(), Value::Boolean(false));
    order_props.insert("payment_complete".to_string(), Value::Boolean(false));
    facts.add_value("Order", Value::Object(order_props))?;

    // Add transaction data for fraud detection
    let mut transaction_props = HashMap::new();
    transaction_props.insert("id".to_string(), Value::String("TXN-001".to_string()));
    transaction_props.insert("amount".to_string(), Value::Number(3500.0));
    transaction_props.insert("suspicious".to_string(), Value::Boolean(true));
    facts.add_value("Transaction", Value::Object(transaction_props))?;

    // Add payment data
    let mut payment_props = HashMap::new();
    payment_props.insert(
        "method".to_string(),
        Value::String("credit_card".to_string()),
    );
    payment_props.insert("status".to_string(), Value::String("verified".to_string()));
    payment_props.insert("amount".to_string(), Value::Number(3500.0));
    facts.add_value("Payment", Value::Object(payment_props))?;

    // Add alert tracking
    let mut alert_props = HashMap::new();
    alert_props.insert("fraud_sent".to_string(), Value::Boolean(false));
    facts.add_value("Alert", Value::Object(alert_props))?;

    println!("\n📊 Initial Facts:");
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer: {customer:?}");
    }
    if let Some(order) = facts.get("Order") {
        println!("   Order: {order:?}");
    }
    if let Some(transaction) = facts.get("Transaction") {
        println!("   Transaction: {transaction:?}");
    }
    if let Some(payment) = facts.get("Payment") {
        println!("   Payment: {payment:?}");
    }
    if let Some(alert) = facts.get("Alert") {
        println!("   Alert: {alert:?}");
    }

    // 📄 Load rules from GRL file
    println!("\n📄 Loading Rules from GRL File...");
    let grl_content = std::fs::read_to_string("examples/rules/03-advanced/action_handlers.grl")?;
    println!("   File: examples/rules/03-advanced/action_handlers.grl");
    println!("   Size: {} bytes", grl_content.len());

    // Parse rules from GRL
    let rules = GRLParser::parse_rules(&grl_content)?;
    println!("   Parsed {} rules from GRL file", rules.len());

    // Create knowledge base and add rules
    let mut kb = KnowledgeBase::new("ActionHandlerGRLDemo");
    for rule in rules {
        println!(
            "   📝 Added rule: {} (salience: {})",
            rule.name, rule.salience
        );
        kb.add_rule(rule)?;
    }

    // Create engine with debug mode
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 5,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // 🎯 Register Action Handlers
    println!("\n🎯 Registering Action Handlers...");

    // 1. Email Handler - access parameters by index
    engine.register_action_handler("SendEmail", |params, facts| {
        // Get parameters by index (0, 1, 2...)
        let to = if let Some(arg) = params.get("0") {
            match arg {
                Value::String(s) => {
                    // Try to resolve from facts if it looks like a reference
                    if let Some(resolved) = facts.get_nested(s) {
                        resolved.to_string()
                    } else {
                        s.clone()
                    }
                }
                _ => arg.to_string(),
            }
        } else {
            "unknown@example.com".to_string()
        };

        let subject = if let Some(arg) = params.get("1") {
            match arg {
                Value::String(s) => {
                    if let Some(resolved) = facts.get_nested(s) {
                        resolved.to_string()
                    } else {
                        s.clone()
                    }
                }
                _ => arg.to_string(),
            }
        } else {
            "No Subject".to_string()
        };

        let body = if let Some(arg) = params.get("2") {
            match arg {
                Value::String(s) => {
                    if let Some(resolved) = facts.get_nested(s) {
                        resolved.to_string()
                    } else {
                        s.clone()
                    }
                }
                _ => arg.to_string(),
            }
        } else {
            "No Body".to_string()
        };

        println!("📧 EMAIL SENT:");
        println!("   ├── To: {}", to);
        println!("   ├── Subject: {}", subject);
        println!("   ├── Body: {}", body);
        println!("   └── Status: ✅ Successfully delivered");

        // Optional: Update facts to track email history
        if let Some(Value::Object(customer_obj)) = facts.get("Customer") {
            let mut updated_customer = customer_obj.clone();
            updated_customer.insert(
                "last_email_sent".to_string(),
                Value::String(chrono::Utc::now().to_string()),
            );
            facts
                .add_value("Customer", Value::Object(updated_customer))
                .unwrap();
        }

        Ok(())
    });

    // 2. Database Logger - access by index
    engine.register_action_handler("LogToDatabase", |params, facts| {
        let table = if let Some(arg) = params.get("0") {
            match arg {
                Value::String(s) => {
                    if let Some(resolved) = facts.get_nested(s) {
                        resolved.to_string()
                    } else {
                        s.clone()
                    }
                }
                _ => arg.to_string(),
            }
        } else {
            "default_table".to_string()
        };

        let event = if let Some(arg) = params.get("1") {
            match arg {
                Value::String(s) => {
                    if let Some(resolved) = facts.get_nested(s) {
                        resolved.to_string()
                    } else {
                        s.clone()
                    }
                }
                _ => arg.to_string(),
            }
        } else {
            "unknown_event".to_string()
        };
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        println!("🗄️ DATABASE LOG:");
        println!("   ├── Table: {}", table);
        println!("   ├── Event: {}", event);
        println!("   ├── Timestamp: {}", timestamp);

        // Add context from facts
        if let Some(customer) = facts.get("Customer") {
            if let Value::Object(customer_obj) = customer {
                if let Some(name) = customer_obj.get("name") {
                    println!("   ├── Customer: {}", name.to_string());
                }
                if let Some(tier) = customer_obj.get("tier") {
                    println!("   ├── Tier: {}", tier.to_string());
                }
            }
        }

        if let Some(order) = facts.get("Order") {
            if let Value::Object(order_obj) = order {
                if let Some(order_id) = order_obj.get("id") {
                    println!("   ├── Order ID: {}", order_id.to_string());
                }
                if let Some(total) = order_obj.get("total") {
                    println!("   ├── Order Total: ${}", total.to_string());
                }
            }
        }

        println!("   └── Status: ✅ Logged to database");

        Ok(())
    });

    // 3. Advanced Alert Handler - access by index
    engine.register_action_handler("SendAlert", |params, _facts| {
        let level = if let Some(arg) = params.get("0") {
            arg.to_string()
        } else {
            "INFO".to_string()
        };

        let message = if let Some(arg) = params.get("1") {
            arg.to_string()
        } else {
            "Alert triggered".to_string()
        };

        let (emoji, priority) = match level.to_uppercase().as_str() {
            "CRITICAL" => ("🚨", "URGENT"),
            "HIGH" => ("⚠️", "HIGH"),
            "MEDIUM" => ("🔔", "NORMAL"),
            "LOW" => ("ℹ️", "LOW"),
            _ => ("ℹ️", "INFO"),
        };

        println!("{} ALERT [{}]:", emoji, level.to_uppercase());
        println!("   ├── Priority: {}", priority);
        println!("   ├── Message: {}", message);
        println!(
            "   ├── Timestamp: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        // Add alert context
        if level.to_uppercase() == "CRITICAL" {
            println!("   ├── 🚨 IMMEDIATE ACTION REQUIRED");
            println!("   ├── 📞 Notifying security team");
            println!("   └── 🔒 Escalating to management");
        } else {
            println!("   └── 📋 Alert logged for review");
        }

        Ok(())
    });

    // 4. Payment Processing Handler - access by index with fact resolution
    engine.register_action_handler("ProcessPayment", |params, facts| {
        let amount = if let Some(arg) = params.get("0") {
            match arg {
                Value::Number(n) => *n,
                Value::Integer(i) => *i as f64,
                Value::String(s) => {
                    // Try to resolve from facts first
                    if let Some(fact_value) = facts.get_nested(s) {
                        match fact_value {
                            Value::Number(n) => n,
                            Value::Integer(i) => i as f64,
                            _ => s.parse::<f64>().unwrap_or(0.0),
                        }
                    } else {
                        s.parse::<f64>().unwrap_or(0.0)
                    }
                }
                _ => 0.0,
            }
        } else {
            0.0
        };

        let method = if let Some(arg) = params.get("1") {
            match arg {
                Value::String(s) => {
                    if let Some(resolved) = facts.get_nested(s) {
                        resolved.to_string()
                    } else {
                        s.clone()
                    }
                }
                _ => arg.to_string(),
            }
        } else {
            "credit_card".to_string()
        };

        println!("💳 PAYMENT PROCESSING:");
        println!("   ├── Amount: ${:.2}", amount);
        println!("   ├── Method: {}", method);

        // Business logic based on amount
        if amount > 5000.0 {
            println!("   ├── 🔐 High-value payment detected");
            println!("   ├── ✅ Additional verification completed");
            println!("   ├── 🛡️ Fraud check: PASSED");
        } else if amount > 1000.0 {
            println!("   ├── 🔍 Standard verification applied");
        }

        // Processing status
        let processing_fee = amount * 0.029; // 2.9% processing fee
        println!("   ├── Processing Fee: ${:.2}", processing_fee);
        println!("   ├── Net Amount: ${:.2}", amount - processing_fee);

        // Update payment status in facts
        if let Some(Value::Object(payment_obj)) = facts.get("Payment") {
            let mut updated_payment = payment_obj.clone();
            updated_payment.insert("status".to_string(), Value::String("processed".to_string()));
            updated_payment.insert(
                "processed_amount".to_string(),
                Value::Number(amount - processing_fee),
            );
            updated_payment.insert("processing_fee".to_string(), Value::Number(processing_fee));
            updated_payment.insert(
                "processed_at".to_string(),
                Value::String(chrono::Utc::now().to_string()),
            );
            facts
                .add_value("Payment", Value::Object(updated_payment))
                .unwrap();
        }

        println!("   └── Status: ✅ Payment processed successfully");

        Ok(())
    });

    println!("✅ Registered {} action handlers", 4);

    // 🚀 Execute Rules from GRL File
    println!("\n🚀 Executing Rules from GRL File...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Execution Results:");
    println!("   ├── Cycles: {}", result.cycle_count);
    println!("   ├── Rules evaluated: {}", result.rules_evaluated);
    println!("   ├── Rules fired: {}", result.rules_fired);
    println!("   └── Execution time: {:?}", result.execution_time);

    println!("\n🏁 Final Facts State:");
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer: {customer:?}");
    }
    if let Some(order) = facts.get("Order") {
        println!("   Order: {order:?}");
    }
    if let Some(payment) = facts.get("Payment") {
        println!("   Payment: {payment:?}");
    }
    if let Some(alert) = facts.get("Alert") {
        println!("   Alert: {alert:?}");
    }

    println!("\n🎯 GRL File Action Handlers Demonstrated:");
    println!("   📄 Rules loaded from external .grl file");
    println!("   🚨 Advanced action handlers with business logic");
    println!("   🧩 Pattern matching (EXISTS, FORALL, NOT) integration");
    println!("   📧 Email notifications with named parameter syntax");
    println!("   🗄️ Database logging with contextual information");
    println!("   ⚠️ Multi-level alert system with escalation");
    println!("   💳 Payment processing with fee calculation");
    println!("   🔄 Real-time fact updates during execution");
    println!("   🛡️ No-loop protection for stable execution");
    println!("   🎯 Custom action syntax: ActionName(param: value)");

    Ok(())
}

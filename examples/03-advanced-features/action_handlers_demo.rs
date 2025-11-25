use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚨 Advanced Action Handlers Demo");
    println!("=================================");

    // Create facts
    let facts = Facts::new();

    // Add customer data
    let mut customer_props = HashMap::new();
    customer_props.insert("name".to_string(), Value::String("John Doe".to_string()));
    customer_props.insert(
        "email".to_string(),
        Value::String("john.doe@example.com".to_string()),
    );
    customer_props.insert("tier".to_string(), Value::String("VIP".to_string()));
    customer_props.insert("balance".to_string(), Value::Number(1500.0));
    facts.add_value("Customer", Value::Object(customer_props))?;

    // Add order data
    let mut order_props = HashMap::new();
    order_props.insert("id".to_string(), Value::String("ORD-001".to_string()));
    order_props.insert("total".to_string(), Value::Number(2500.0));
    order_props.insert("status".to_string(), Value::String("pending".to_string()));
    facts.add_value("Order", Value::Object(order_props))?;

    println!("\n📊 Initial Facts:");
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer: {customer:?}");
    }
    if let Some(order) = facts.get("Order") {
        println!("   Order: {order:?}");
    }

    // Create knowledge base and engine
    let kb = KnowledgeBase::new("ActionHandlerDemo");
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 3,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // 🎯 Register Action Handlers
    println!("\n🎯 Registering Action Handlers...");

    // 1. Email Handler
    engine.register_action_handler("SendEmail", |params, _facts| {
        let to = params
            .get("to")
            .map(|v| v.to_string())
            .unwrap_or("unknown".to_string());
        let subject = params
            .get("subject")
            .map(|v| v.to_string())
            .unwrap_or("No Subject".to_string());
        let body = params
            .get("body")
            .map(|v| v.to_string())
            .unwrap_or("No Body".to_string());

        println!("📧 EMAIL SENT:");
        println!("   To: {}", to);
        println!("   Subject: {}", subject);
        println!("   Body: {}", body);
        println!("   Status: ✅ Successfully sent");

        Ok(())
    });

    // 2. Database Logger Handler
    engine.register_action_handler("LogToDatabase", |params, facts| {
        let table = params
            .get("table")
            .map(|v| v.to_string())
            .unwrap_or("default_table".to_string());
        let event = params
            .get("event")
            .map(|v| v.to_string())
            .unwrap_or("unknown_event".to_string());

        println!("🗄️ DATABASE LOG:");
        println!("   Table: {}", table);
        println!("   Event: {}", event);
        println!(
            "   Timestamp: {}",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
        );

        // Could access facts for additional context
        if let Some(customer) = facts.get("Customer") {
            if let Value::Object(customer_obj) = customer {
                if let Some(name) = customer_obj.get("name") {
                    println!("   Customer: {}", name.to_string());
                }
            }
        }

        Ok(())
    });

    // 3. Alert Handler
    engine.register_action_handler("SendAlert", |params, _facts| {
        let level = params
            .get("level")
            .map(|v| v.to_string())
            .unwrap_or("INFO".to_string());
        let message = params
            .get("message")
            .map(|v| v.to_string())
            .unwrap_or("Alert triggered".to_string());

        let emoji = match level.to_uppercase().as_str() {
            "CRITICAL" => "🚨",
            "HIGH" => "⚠️",
            "MEDIUM" => "🔔",
            _ => "ℹ️",
        };

        println!("{} ALERT [{}]: {}", emoji, level.to_uppercase(), message);

        Ok(())
    });

    // 4. Payment Processing Handler
    engine.register_action_handler("ProcessPayment", |params, facts| {
        let amount = params
            .get("amount")
            .and_then(|v| match v {
                Value::Number(n) => Some(*n),
                Value::Integer(i) => Some(*i as f64),
                _ => None,
            })
            .unwrap_or(0.0);

        let method = params
            .get("method")
            .map(|v| v.to_string())
            .unwrap_or("credit_card".to_string());

        println!("💳 PAYMENT PROCESSING:");
        println!("   Amount: ${:.2}", amount);
        println!("   Method: {}", method);

        if amount > 1000.0 {
            println!("   Status: 🔐 Requires additional verification");
        } else {
            println!("   Status: ✅ Payment approved");
        }

        // Update order status in facts
        if let Some(Value::Object(order_obj)) = facts.get("Order") {
            let mut updated_order = order_obj.clone();
            updated_order.insert(
                "payment_status".to_string(),
                Value::String("processed".to_string()),
            );
            facts
                .add_value("Order", Value::Object(updated_order))
                .unwrap();
        }

        Ok(())
    });

    println!("✅ Registered {} action handlers", 4);

    // 🔧 Create Rules with Custom Actions
    println!("\n🔧 Creating Rules with Custom Actions...");

    // Rule 1: VIP Customer Welcome Email
    let welcome_rule = Rule::new(
        "VIPCustomerWelcome".to_string(),
        ConditionGroup::single(Condition::new(
            "Customer.tier".to_string(),
            Operator::Equal,
            Value::String("VIP".to_string()),
        )),
        vec![
            ActionType::Custom {
                action_type: "SendEmail".to_string(),
                params: {
                    let mut params = HashMap::new();
                    params.insert(
                        "to".to_string(),
                        Value::String("Customer.email".to_string()),
                    );
                    params.insert(
                        "subject".to_string(),
                        Value::String("Welcome VIP Customer!".to_string()),
                    );
                    params.insert(
                        "body".to_string(),
                        Value::String(
                            "Thank you for being a VIP customer. Enjoy exclusive benefits!"
                                .to_string(),
                        ),
                    );
                    params
                },
            },
            ActionType::Custom {
                action_type: "LogToDatabase".to_string(),
                params: {
                    let mut params = HashMap::new();
                    params.insert(
                        "table".to_string(),
                        Value::String("customer_events".to_string()),
                    );
                    params.insert(
                        "event".to_string(),
                        Value::String("vip_welcome_email_sent".to_string()),
                    );
                    params
                },
            },
            ActionType::Set {
                field: "Customer.welcome_sent".to_string(),
                value: Value::Boolean(true),
            },
        ],
    )
    .with_salience(20)
    .with_no_loop(true);

    // Rule 2: High Value Order Alert
    let high_value_rule = Rule::new(
        "HighValueOrderAlert".to_string(),
        ConditionGroup::single(Condition::new(
            "Order.total".to_string(),
            Operator::GreaterThan,
            Value::Number(2000.0),
        )),
        vec![
            ActionType::Custom {
                action_type: "SendAlert".to_string(),
                params: {
                    let mut params = HashMap::new();
                    params.insert("level".to_string(), Value::String("HIGH".to_string()));
                    params.insert(
                        "message".to_string(),
                        Value::String("High value order detected - requires review".to_string()),
                    );
                    params
                },
            },
            ActionType::Custom {
                action_type: "ProcessPayment".to_string(),
                params: {
                    let mut params = HashMap::new();
                    params.insert(
                        "amount".to_string(),
                        Value::String("Order.total".to_string()),
                    );
                    params.insert(
                        "method".to_string(),
                        Value::String("premium_processing".to_string()),
                    );
                    params
                },
            },
            ActionType::Set {
                field: "Order.payment_processed".to_string(),
                value: Value::Boolean(true),
            },
        ],
    )
    .with_salience(15)
    .with_no_loop(true);

    // Rule 3: Order Status Update
    let status_update_rule = Rule::new(
        "OrderStatusUpdate".to_string(),
        ConditionGroup::single(Condition::new(
            "Order.status".to_string(),
            Operator::Equal,
            Value::String("pending".to_string()),
        )),
        vec![
            ActionType::Set {
                field: "Order.status".to_string(),
                value: Value::String("processing".to_string()),
            },
            ActionType::Custom {
                action_type: "LogToDatabase".to_string(),
                params: {
                    let mut params = HashMap::new();
                    params.insert(
                        "table".to_string(),
                        Value::String("order_status_log".to_string()),
                    );
                    params.insert(
                        "event".to_string(),
                        Value::String("status_changed_to_processing".to_string()),
                    );
                    params
                },
            },
        ],
    )
    .with_salience(10)
    .with_no_loop(true);

    // Add rules to knowledge base
    engine.knowledge_base_mut().add_rule(welcome_rule)?;
    engine.knowledge_base_mut().add_rule(high_value_rule)?;
    engine.knowledge_base_mut().add_rule(status_update_rule)?;

    println!("✅ Added {} rules with custom actions", 3);

    // 🚀 Execute Rules
    println!("\n🚀 Executing Rules...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Execution Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    println!("\n🏁 Final Facts State:");
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer: {customer:?}");
    }
    if let Some(order) = facts.get("Order") {
        println!("   Order: {order:?}");
    }

    println!("\n🎯 Advanced Action Handlers Demonstrated:");
    println!("   📧 Email sending with custom parameters");
    println!("   🗄️ Database logging with fact context");
    println!("   🚨 Multi-level alert system");
    println!("   💳 Payment processing with business logic");
    println!("   🔄 Fact updates within action handlers");

    Ok(())
}

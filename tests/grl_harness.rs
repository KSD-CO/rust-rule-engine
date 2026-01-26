#![allow(clippy::unnecessary_get_then_check)]

use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::GRLParser;
use rust_rule_engine::types::Value;

use std::collections::HashMap;

#[test]
fn action_handlers_end_to_end() -> Result<(), Box<dyn std::error::Error>> {
    // Build initial facts similar to the example demo
    let facts = Facts::new();

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

    let mut order_props = HashMap::new();
    order_props.insert("id".to_string(), Value::String("ORD-002".to_string()));
    order_props.insert("total".to_string(), Value::Number(3500.0));
    order_props.insert("status".to_string(), Value::String("pending".to_string()));
    order_props.insert("alert_sent".to_string(), Value::Boolean(false));
    order_props.insert("processed".to_string(), Value::Boolean(false));
    order_props.insert("payment_complete".to_string(), Value::Boolean(false));
    facts.add_value("Order", Value::Object(order_props))?;

    let mut transaction_props = HashMap::new();
    transaction_props.insert("id".to_string(), Value::String("TXN-001".to_string()));
    transaction_props.insert("amount".to_string(), Value::Number(3500.0));
    transaction_props.insert("suspicious".to_string(), Value::Boolean(true));
    facts.add_value("Transaction", Value::Object(transaction_props))?;

    let mut payment_props = HashMap::new();
    payment_props.insert(
        "method".to_string(),
        Value::String("credit_card".to_string()),
    );
    payment_props.insert("status".to_string(), Value::String("verified".to_string()));
    payment_props.insert("amount".to_string(), Value::Number(3500.0));
    facts.add_value("Payment", Value::Object(payment_props))?;

    let mut alert_props = HashMap::new();
    alert_props.insert("fraud_sent".to_string(), Value::Boolean(false));
    facts.add_value("Alert", Value::Object(alert_props))?;

    // Load rules from GRL file
    let grl_content = std::fs::read_to_string("examples/rules/03-advanced/action_handlers.grl")?;
    let rules = GRLParser::parse_rules(&grl_content)?;

    let kb = KnowledgeBase::new("TestKB");
    for rule in rules {
        kb.add_rule(rule)?;
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 5,
        ..Default::default()
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    // Minimal action handlers performing same effects as demo but quiet
    engine.register_action_handler("SendEmail", |_, facts| {
        // set last_email_sent on Customer
        if let Some(Value::Object(customer_obj)) = facts.get("Customer") {
            let mut updated = customer_obj.clone();
            updated.insert(
                "last_email_sent".to_string(),
                Value::String(chrono::Utc::now().to_string()),
            );
            facts.add_value("Customer", Value::Object(updated)).unwrap();
        }
        Ok(())
    });

    engine.register_action_handler("LogToDatabase", |_, _| Ok(()));
    engine.register_action_handler("SendAlert", |_, facts| {
        // set Alert.fraud_sent = true
        if let Some(Value::Object(alert_obj)) = facts.get("Alert") {
            let mut updated = alert_obj.clone();
            updated.insert("fraud_sent".to_string(), Value::Boolean(true));
            facts.add_value("Alert", Value::Object(updated)).unwrap();
        }
        Ok(())
    });

    engine.register_action_handler("ProcessPayment", |_, facts| {
        if let Some(Value::Object(payment_obj)) = facts.get("Payment") {
            let mut updated = payment_obj.clone();
            updated.insert("status".to_string(), Value::String("processed".to_string()));
            facts.add_value("Payment", Value::Object(updated)).unwrap();
        }
        Ok(())
    });

    // Execute
    let result = engine.execute(&facts)?;

    // Expect at least one rule fired
    assert!(result.rules_fired > 0, "expected some rules to fire");

    // Check facts updated by actions
    if let Some(Value::Object(alert_obj)) = facts.get("Alert") {
        let v = alert_obj
            .get("fraud_sent")
            .cloned()
            .unwrap_or(Value::Boolean(false));
        assert_eq!(v, Value::Boolean(true));
    } else {
        panic!("Alert fact missing");
    }

    if let Some(Value::Object(customer_obj)) = facts.get("Customer") {
        let _ = customer_obj
            .get("welcome_sent")
            .cloned()
            .unwrap_or(Value::Boolean(false));
        // Ensure SendEmail updated last_email_sent
        assert!(
            customer_obj.get("last_email_sent").is_some(),
            "expected last_email_sent set"
        );
    } else {
        panic!("Customer fact missing");
    }

    if let Some(Value::Object(payment_obj)) = facts.get("Payment") {
        let status = payment_obj
            .get("status")
            .cloned()
            .unwrap_or(Value::String("".to_string()));
        assert_eq!(status, Value::String("processed".to_string()));
    } else {
        panic!("Payment fact missing");
    }

    Ok(())
}

#[test]
fn method_calls_smoke() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure parsing and execution of method_calls.grl does not panic and returns Ok
    let grl_content = std::fs::read_to_string("examples/rules/01-basic/method_calls.grl")?;
    let rules = GRLParser::parse_rules(&grl_content)?;
    let kb = KnowledgeBase::new("MethodCallsTest");
    for rule in rules {
        kb.add_rule(rule)?;
    }

    // Create initial TestCar facts like the example
    let facts = Facts::new();
    let mut car = HashMap::new();
    car.insert("SpeedIncrement".to_string(), Value::Number(10.0));
    car.insert("MaxSpeed".to_string(), Value::Number(100.0));
    car.insert("Speed".to_string(), Value::Number(30.0));
    car.insert("SpeedUp".to_string(), Value::Boolean(true));
    facts.add_value("TestCar", Value::Object(car))?;

    let config = EngineConfig {
        debug_mode: false,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);
    // Register action handlers for the method calls used in the GRL file
    // Note: Actions like TestCar.setSpeed(...) are parsed as Custom actions with
    // the full name "TestCar.setSpeed"
    engine.register_action_handler("TestCar.setSpeed", |params, facts| {
        if let Some(speed_value) = params.get("0") {
            if let Some(car) = facts.get("TestCar") {
                if let Value::Object(mut car_obj) = car.clone() {
                    car_obj.insert("Speed".to_string(), speed_value.clone());
                    // Note: In a real implementation, you'd update the fact in the facts collection
                    // For this test, we just ensure the handler exists and doesn't panic
                }
            }
        }
        Ok(())
    });

    engine.register_action_handler("TestCar.setSpeedUp", |params, facts| {
        if let Some(speed_up_value) = params.get("0") {
            if let Some(car) = facts.get("TestCar") {
                if let Value::Object(mut car_obj) = car.clone() {
                    car_obj.insert("SpeedUp".to_string(), speed_up_value.clone());
                    // Note: In a real implementation, you'd update the fact in the facts collection
                }
            }
        }
        Ok(())
    });
    // We only assert that execute returns Ok (no panic). Use a mutable engine to conform API
    let _ = engine.execute(&facts)?;

    Ok(())
}

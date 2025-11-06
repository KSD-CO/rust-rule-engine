//! Test CE (Conditional Element) Demo
//!
//! Demonstrates CLIPS-inspired Test CE feature:
//! - Arbitrary boolean expressions in WHEN clause
//! - Function calls that return boolean results
//! - No need for operator comparison
//!
//! Run: cargo run --example test_ce_demo

use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use rust_rule_engine::errors::Result;
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("=== Test CE Demo ===\n");

    // Demo 1: Simple test function
    demo_simple_test()?;

    // Demo 2: Test with arguments
    demo_test_with_args()?;

    // Demo 3: Combined with regular conditions
    demo_combined_conditions()?;

    println!("\n✅ Demo completed successfully!");
    Ok(())
}

fn demo_simple_test() -> Result<()> {
    println!("1️⃣ Simple Test CE - Email Validation");
    println!("   Using: test(is_valid_email(User.email))\n");

    // Create facts
    let facts = Facts::new();
    let mut user_props = HashMap::new();
    user_props.insert("email".to_string(), Value::String("user@example.com".to_string()));
    user_props.insert("status".to_string(), Value::String("unknown".to_string()));
    facts.add_value("User", Value::Object(user_props))?;

    // Create knowledge base
    let kb = KnowledgeBase::new("TestCEDemo");

    // Create rule with Test CE
    let rule = Rule::new(
        "ValidateEmail".to_string(),
        ConditionGroup::single(
            Condition::with_test(
                "is_valid_email".to_string(),
                vec!["User.email".to_string()],
            )
        ),
        vec![
            ActionType::MethodCall {
                object: "User".to_string(),
                method: "setStatus".to_string(),
                args: vec![Value::String("valid".to_string())],
            },
        ],
    );

    kb.add_rule(rule)?;

    // Create engine with function
    let config = EngineConfig {
        debug_mode: false,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register test function
    engine.register_function(
        "is_valid_email",
        |args: &[Value], _facts: &Facts| {
            if let Some(Value::String(email)) = args.first() {
                Ok(Value::Boolean(email.contains('@') && email.contains('.')))
            } else {
                Ok(Value::Boolean(false))
            }
        },
    );

    // Execute
    engine.execute(&facts)?;

    println!("\n   Input: User.email = user@example.com");
    if let Some(Value::Object(user)) = facts.get("User") {
        if let Some(status) = user.get("status") {
            println!("   Result: User.status = {:?}", status);
        }
    }
    println!("   ✅ Test CE evaluated function and set status\n");

    Ok(())
}

fn demo_test_with_args() -> Result<()> {
    println!("2️⃣ Test CE with Multiple Arguments");
    println!("   Using: test(in_range(Product.price, 100, 1000))\n");

    // Create facts
    let facts = Facts::new();
    let mut product_props = HashMap::new();
    product_props.insert("price".to_string(), Value::Number(500.0));
    product_props.insert("category".to_string(), Value::String("unknown".to_string()));
    facts.add_value("Product", Value::Object(product_props))?;

    // Create knowledge base
    let kb = KnowledgeBase::new("PriceRangeDemo");

    // Create rule with Test CE and multiple arguments
    let rule = Rule::new(
        "CheckPriceRange".to_string(),
        ConditionGroup::single(
            Condition::with_test(
                "in_range".to_string(),
                vec!["Product.price".to_string(), "100".to_string(), "1000".to_string()],
            )
        ),
        vec![
            ActionType::MethodCall {
                object: "Product".to_string(),
                method: "setCategory".to_string(),
                args: vec![Value::String("mid-range".to_string())],
            },
        ],
    );

    kb.add_rule(rule)?;

    // Create engine
    let mut engine = RustRuleEngine::with_config(kb, EngineConfig::default());

    // Register range check function
    engine.register_function(
        "in_range",
        |args: &[Value], _facts: &Facts| {
            if args.len() >= 3 {
                if let (Some(Value::Number(val)), Some(Value::String(min_str)), Some(Value::String(max_str))) =
                    (args.get(0), args.get(1), args.get(2))
                {
                    if let (Ok(min), Ok(max)) = (min_str.parse::<f64>(), max_str.parse::<f64>()) {
                        return Ok(Value::Boolean(*val >= min && *val <= max));
                    }
                }
            }
            Ok(Value::Boolean(false))
        },
    );

    // Execute
    engine.execute(&facts)?;

    println!("   Input: Product.price = 500.0");
    println!("   Range: 100 to 1000");
    if let Some(Value::Object(product)) = facts.get("Product") {
        if let Some(category) = product.get("category") {
            println!("   Result: Product.category = {:?}", category);
        }
    }
    println!("   ✅ Test CE with arguments works correctly\n");

    Ok(())
}

fn demo_combined_conditions() -> Result<()> {
    println!("3️⃣ Test CE Combined with Regular Conditions");
    println!("   Using: Order.amount > 100 && test(is_valid_email(Customer.email))\n");

    // Create facts
    let facts = Facts::new();
    let mut order_props = HashMap::new();
    order_props.insert("amount".to_string(), Value::Number(150.0));
    order_props.insert("status".to_string(), Value::String("pending".to_string()));
    order_props.insert("discount".to_string(), Value::Number(0.0));
    facts.add_value("Order", Value::Object(order_props))?;

    let mut customer_props = HashMap::new();
    customer_props.insert("email".to_string(), Value::String("customer@shop.com".to_string()));
    facts.add_value("Customer", Value::Object(customer_props))?;

    // Create knowledge base
    let kb = KnowledgeBase::new("CombinedConditionsDemo");

    // Create rule with combined conditions
    let rule = Rule::new(
        "ProcessOrder".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(
                Condition::new(
                    "Order.amount".to_string(),
                    Operator::GreaterThan,
                    Value::Number(100.0),
                )
            ),
            ConditionGroup::single(
                Condition::with_test(
                    "is_valid_email".to_string(),
                    vec!["Customer.email".to_string()],
                )
            ),
        ),
        vec![
            ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setStatus".to_string(),
                args: vec![Value::String("approved".to_string())],
            },
            ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setDiscount".to_string(),
                args: vec![Value::Number(50.0)],
            },
        ],
    );

    kb.add_rule(rule)?;

    // Create engine
    let mut engine = RustRuleEngine::with_config(kb, EngineConfig::default());

    // Register email validation function
    engine.register_function(
        "is_valid_email",
        |args: &[Value], _facts: &Facts| {
            if let Some(Value::String(email)) = args.first() {
                Ok(Value::Boolean(email.contains('@') && email.contains('.')))
            } else {
                Ok(Value::Boolean(false))
            }
        },
    );

    // Execute
    engine.execute(&facts)?;

    println!("   Input:");
    println!("     Order.amount = 150.0");
    println!("     Customer.email = customer@shop.com");
    println!("   Results:");
    if let Some(Value::Object(order)) = facts.get("Order") {
        if let Some(status) = order.get("status") {
            println!("     Order.status = {:?}", status);
        }
        if let Some(discount) = order.get("discount") {
            println!("     Order.discount = {:?}", discount);
        }
    }
    println!("   ✅ Combined conditions work together\n");

    Ok(())
}

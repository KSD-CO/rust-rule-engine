//! Simple Test CE Demo - Shows Test CE condition evaluation
//!
//! Demonstrates that Test CE works by showing:
//! 1. Rules fire when test() returns true
//! 2. Rules DON'T fire when test() returns false
//!
//! Run: cargo run --example test_ce_simple_demo

use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Value};
use rust_rule_engine::errors::Result;
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("=== Test CE Simple Demo ===\n");
    println!("This demo shows that Test CE condition evaluation works correctly.\n");

    // Demo 1: Test CE returns true - rule should fire
    demo_test_returns_true()?;

    // Demo 2: Test CE returns false - rule should NOT fire
    demo_test_returns_false()?;

    // Demo 3: Combined condition (regular + test CE)
    demo_combined_conditions()?;

    println!("\n✅ All Test CE scenarios verified!");
    Ok(())
}

fn demo_test_returns_true() -> Result<()> {
    println!("1️⃣ Test CE returns TRUE - Rule SHOULD fire");
    println!("   test(is_valid_email(User.email)) with valid email\n");

    // Create facts with valid email
    let facts = Facts::new();
    let mut user_props = HashMap::new();
    user_props.insert("email".to_string(), Value::String("valid@example.com".to_string()));
    user_props.insert("verified".to_string(), Value::Boolean(false));
    facts.add_value("User", Value::Object(user_props))?;

    // Create rule
    let kb = KnowledgeBase::new("TestTrue");
    let rule = Rule::new(
        "VerifyEmail".to_string(),
        ConditionGroup::single(
            Condition::with_test(
                "is_valid_email".to_string(),
                vec!["User.email".to_string()],
            )
        ),
        vec![
            ActionType::MethodCall {
                object: "User".to_string(),
                method: "setVerified".to_string(),
                args: vec![Value::Boolean(true)],
            },
        ],
    );
    kb.add_rule(rule)?;

    // Execute
    let mut engine = RustRuleEngine::with_config(kb, EngineConfig::default());
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

    let result = engine.execute(&facts)?;

    println!("   Input: User.email = 'valid@example.com'");
    println!("   Rules fired: {}", result.rules_fired);

    if result.rules_fired > 0 {
        println!("   ✅ SUCCESS: Rule fired as expected (Test CE returned true)\n");
    } else {
        println!("   ❌ FAILED: Rule should have fired but didn't\n");
    }

    Ok(())
}

fn demo_test_returns_false() -> Result<()> {
    println!("2️⃣ Test CE returns FALSE - Rule should NOT fire");
    println!("   test(is_valid_email(User.email)) with invalid email\n");

    // Create facts with INVALID email
    let facts = Facts::new();
    let mut user_props = HashMap::new();
    user_props.insert("email".to_string(), Value::String("invalid-email".to_string()));
    user_props.insert("verified".to_string(), Value::Boolean(false));
    facts.add_value("User", Value::Object(user_props))?;

    // Create rule
    let kb = KnowledgeBase::new("TestFalse");
    let rule = Rule::new(
        "VerifyEmail".to_string(),
        ConditionGroup::single(
            Condition::with_test(
                "is_valid_email".to_string(),
                vec!["User.email".to_string()],
            )
        ),
        vec![
            ActionType::MethodCall {
                object: "User".to_string(),
                method: "setVerified".to_string(),
                args: vec![Value::Boolean(true)],
            },
        ],
    );
    kb.add_rule(rule)?;

    // Execute
    let mut engine = RustRuleEngine::with_config(kb, EngineConfig::default());
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

    let result = engine.execute(&facts)?;

    println!("   Input: User.email = 'invalid-email'");
    println!("   Rules fired: {}", result.rules_fired);

    if result.rules_fired == 0 {
        println!("   ✅ SUCCESS: Rule did NOT fire (Test CE returned false)\n");
    } else {
        println!("   ❌ FAILED: Rule fired when it shouldn't have\n");
    }

    Ok(())
}

fn demo_combined_conditions() -> Result<()> {
    println!("3️⃣ Combined Conditions - Both must be true");
    println!("   Order.amount > 100 AND test(is_valid_email(Customer.email))\n");

    // Scenario A: Both conditions true - should fire
    println!("   Scenario A: amount=150, email=valid@shop.com");
    let facts_a = Facts::new();
    let mut order_a = HashMap::new();
    order_a.insert("amount".to_string(), Value::Number(150.0));
    facts_a.add_value("Order", Value::Object(order_a))?;
    let mut customer_a = HashMap::new();
    customer_a.insert("email".to_string(), Value::String("valid@shop.com".to_string()));
    facts_a.add_value("Customer", Value::Object(customer_a))?;

    let result_a = execute_combined_rule(&facts_a)?;
    println!("     Rules fired: {} {}", result_a.rules_fired,
        if result_a.rules_fired == 1 { "✅" } else { "❌" });

    // Scenario B: First true, second false - should NOT fire
    println!("\n   Scenario B: amount=150, email=invalid");
    let facts_b = Facts::new();
    let mut order_b = HashMap::new();
    order_b.insert("amount".to_string(), Value::Number(150.0));
    facts_b.add_value("Order", Value::Object(order_b))?;
    let mut customer_b = HashMap::new();
    customer_b.insert("email".to_string(), Value::String("invalid".to_string()));
    facts_b.add_value("Customer", Value::Object(customer_b))?;

    let result_b = execute_combined_rule(&facts_b)?;
    println!("     Rules fired: {} {}", result_b.rules_fired,
        if result_b.rules_fired == 0 { "✅" } else { "❌" });

    // Scenario C: First false, second true - should NOT fire
    println!("\n   Scenario C: amount=50, email=valid@shop.com");
    let facts_c = Facts::new();
    let mut order_c = HashMap::new();
    order_c.insert("amount".to_string(), Value::Number(50.0));
    facts_c.add_value("Order", Value::Object(order_c))?;
    let mut customer_c = HashMap::new();
    customer_c.insert("email".to_string(), Value::String("valid@shop.com".to_string()));
    facts_c.add_value("Customer", Value::Object(customer_c))?;

    let result_c = execute_combined_rule(&facts_c)?;
    println!("     Rules fired: {} {}", result_c.rules_fired,
        if result_c.rules_fired == 0 { "✅" } else { "❌" });

    println!();
    Ok(())
}

fn execute_combined_rule(facts: &Facts) -> Result<rust_rule_engine::engine::GruleExecutionResult> {
    use rust_rule_engine::types::Operator;

    let kb = KnowledgeBase::new("Combined");
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
                method: "setApproved".to_string(),
                args: vec![Value::Boolean(true)],
            },
        ],
    );
    kb.add_rule(rule)?;

    let config = EngineConfig {
        max_cycles: 1,  // Only run once to avoid flooding output
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);
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

    engine.execute(facts)
}

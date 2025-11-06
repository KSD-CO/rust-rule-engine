//! Comprehensive Test CE Demo
//!
//! Demonstrates CLIPS-inspired Test CE feature with:
//! - GRL file parsing (not hardcoded rules)
//! - Native Engine execution
//! - RETE-UL Engine execution
//! - Multiple test scenarios
//!
//! Run: cargo run --example test_ce_comprehensive

use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::Value;
use rust_rule_engine::errors::Result;
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::rete::{
    IncrementalEngine, TypedFacts, FactValue, TemplateBuilder, FieldType
};
use std::collections::HashMap;
use std::fs;

fn main() -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║          Test CE Comprehensive Demo                          ║");
    println!("║  CLIPS-Inspired Test Conditional Element Feature             ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Test 1: Native Engine with GRL file
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Part 1: Native Engine (Traditional) with GRL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    test_native_engine()?;

    println!("\n");

    // Test 2: RETE-UL Engine with GRL file
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔥 Part 2: RETE-UL Engine (High Performance) with GRL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    test_rete_engine()?;

    println!("\n");

    // Test 3: Comparison scenarios
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Part 3: Test CE Behavior Verification");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    test_scenarios()?;

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                  ✅ All Tests Completed!                      ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    Ok(())
}

fn test_native_engine() -> Result<()> {
    println!("🔧 Testing Native Engine with Test CE from GRL file\n");

    // Read GRL file
    let grl_content = fs::read_to_string("examples/rules/test_ce_rules.grl")
        .expect("Failed to read GRL file");

    // Create Knowledge Base and parse rules
    let kb = KnowledgeBase::new("TestCE_Native");

    println!("   📄 Loading rules from: examples/rules/test_ce_rules.grl");

    // Parse GRL rules
    let rules = GRLParser::parse_rules(&grl_content)?;
    println!("   ✓ Parsed {} rules from GRL file", rules.len());

    // Add rules to knowledge base
    for rule in rules {
        println!("      • {}", rule.name);
        kb.add_rule(rule)?;
    }
    println!();

    // Create facts for testing
    let facts = Facts::new();

    // Scenario 1: Valid email
    let mut user_props = HashMap::new();
    user_props.insert("email".to_string(), Value::String("user@example.com".to_string()));
    user_props.insert("status".to_string(), Value::String("unknown".to_string()));
    facts.add_value("User", Value::Object(user_props))?;

    // Create engine
    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 1,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register test functions
    register_native_functions(&mut engine);

    println!("   📋 Test Scenario: Email Validation");
    println!("      Input: User.email = 'user@example.com'");

    let result = engine.execute(&facts)?;

    println!("      Result: {} rules fired", result.rules_fired);

    if let Some(Value::Object(user)) = facts.get("User") {
        if let Some(status) = user.get("status") {
            println!("      Status: {:?}", status);
        }
    }

    if result.rules_fired > 0 {
        println!("      ✅ Native Engine: Test CE working correctly\n");
    } else {
        println!("      ⚠️  Native Engine: No rules fired\n");
    }

    Ok(())
}

fn test_rete_engine() -> Result<()> {
    println!("🚀 Testing RETE-UL Engine with Test CE\n");

    // Note: RETE engine would need Test CE support to be added
    // This is a placeholder showing the structure

    println!("   📋 RETE-UL Engine Status:");
    println!("      ⚠️  Test CE support for RETE engine is pending");
    println!("      📌 Currently demonstrates Native engine only");
    println!("      🔜 RETE integration planned for next phase\n");

    println!("   📖 When implemented, RETE will provide:");
    println!("      • Incremental evaluation of test() conditions");
    println!("      • Memoization of test function results");
    println!("      • Better performance for 50+ rules\n");

    Ok(())
}

fn test_scenarios() -> Result<()> {
    println!("🧪 Testing various Test CE scenarios\n");

    // Scenario 1: Test returns TRUE
    println!("1️⃣  Scenario: test() returns TRUE → Rule SHOULD fire");
    let result1 = test_email_validation("valid@example.com")?;
    println!("   Input: 'valid@example.com'");
    println!("   Rules fired: {} {}\n", result1,
        if result1 > 0 { "✅" } else { "❌" });

    // Scenario 2: Test returns FALSE
    println!("2️⃣  Scenario: test() returns FALSE → Rule should NOT fire");
    let result2 = test_email_validation("invalid-email")?;
    println!("   Input: 'invalid-email'");
    println!("   Rules fired: {} {}\n", result2,
        if result2 == 0 { "✅" } else { "❌" });

    // Scenario 3: Multiple arguments
    println!("3️⃣  Scenario: test() with multiple arguments");
    let result3 = test_range_check(500.0, 100.0, 1000.0)?;
    println!("   Input: value=500, min=100, max=1000");
    println!("   Rules fired: {} {}\n", result3,
        if result3 > 0 { "✅" } else { "❌" });

    // Scenario 4: Out of range
    println!("4️⃣  Scenario: test() out of range → Should NOT fire");
    let result4 = test_range_check(50.0, 100.0, 1000.0)?;
    println!("   Input: value=50, min=100, max=1000");
    println!("   Rules fired: {} {}\n", result4,
        if result4 == 0 { "✅" } else { "❌" });

    // Scenario 5: Combined conditions
    println!("5️⃣  Scenario: Regular condition AND test() → Both must be true");
    let result5a = test_combined(150.0, "valid@shop.com")?;
    println!("   Input: amount=150, email='valid@shop.com'");
    println!("   Rules fired: {} {}", result5a,
        if result5a > 0 { "✅" } else { "❌" });

    let result5b = test_combined(150.0, "invalid")?;
    println!("   Input: amount=150, email='invalid'");
    println!("   Rules fired: {} {}", result5b,
        if result5b == 0 { "✅" } else { "❌" });

    let result5c = test_combined(50.0, "valid@shop.com")?;
    println!("   Input: amount=50, email='valid@shop.com'");
    println!("   Rules fired: {} {}\n", result5c,
        if result5c == 0 { "✅" } else { "❌" });

    Ok(())
}

fn test_email_validation(email: &str) -> Result<usize> {
    use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
    use rust_rule_engine::types::ActionType;

    let facts = Facts::new();
    let mut user = HashMap::new();
    user.insert("email".to_string(), Value::String(email.to_string()));
    user.insert("verified".to_string(), Value::Boolean(false));
    facts.add_value("User", Value::Object(user))?;

    let kb = KnowledgeBase::new("EmailTest");
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
                method: "setVerified".to_string(),
                args: vec![Value::Boolean(true)],
            },
        ],
    );
    kb.add_rule(rule)?;

    let mut engine = RustRuleEngine::with_config(kb, EngineConfig {
        max_cycles: 1,
        ..Default::default()
    });
    register_native_functions(&mut engine);

    let result = engine.execute(&facts)?;
    Ok(result.rules_fired)
}

fn test_range_check(value: f64, min: f64, max: f64) -> Result<usize> {
    use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
    use rust_rule_engine::types::ActionType;

    let facts = Facts::new();
    let mut product = HashMap::new();
    product.insert("price".to_string(), Value::Number(value));
    product.insert("category".to_string(), Value::String("unknown".to_string()));
    facts.add_value("Product", Value::Object(product))?;

    let kb = KnowledgeBase::new("RangeTest");
    let rule = Rule::new(
        "CheckRange".to_string(),
        ConditionGroup::single(
            Condition::with_test(
                "in_range".to_string(),
                vec![
                    "Product.price".to_string(),
                    min.to_string(),
                    max.to_string(),
                ],
            )
        ),
        vec![
            ActionType::MethodCall {
                object: "Product".to_string(),
                method: "setCategory".to_string(),
                args: vec![Value::String("in-range".to_string())],
            },
        ],
    );
    kb.add_rule(rule)?;

    let mut engine = RustRuleEngine::with_config(kb, EngineConfig {
        max_cycles: 1,
        ..Default::default()
    });
    register_native_functions(&mut engine);

    let result = engine.execute(&facts)?;
    Ok(result.rules_fired)
}

fn test_combined(amount: f64, email: &str) -> Result<usize> {
    use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
    use rust_rule_engine::types::{ActionType, Operator};

    let facts = Facts::new();
    let mut order = HashMap::new();
    order.insert("amount".to_string(), Value::Number(amount));
    order.insert("approved".to_string(), Value::Boolean(false));
    facts.add_value("Order", Value::Object(order))?;

    let mut customer = HashMap::new();
    customer.insert("email".to_string(), Value::String(email.to_string()));
    facts.add_value("Customer", Value::Object(customer))?;

    let kb = KnowledgeBase::new("CombinedTest");
    let rule = Rule::new(
        "ApproveOrder".to_string(),
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

    let mut engine = RustRuleEngine::with_config(kb, EngineConfig {
        max_cycles: 1,
        ..Default::default()
    });
    register_native_functions(&mut engine);

    let result = engine.execute(&facts)?;
    Ok(result.rules_fired)
}

fn register_native_functions(engine: &mut RustRuleEngine) {
    // Email validation function
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

    // Range check function
    engine.register_function(
        "in_range",
        |args: &[Value], _facts: &Facts| {
            if args.len() >= 3 {
                // First arg should be Number, rest could be String or Number
                if let Some(Value::Number(val)) = args.get(0) {
                    let min = match args.get(1) {
                        Some(Value::Number(n)) => *n,
                        Some(Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
                        _ => return Ok(Value::Boolean(false)),
                    };
                    let max = match args.get(2) {
                        Some(Value::Number(n)) => *n,
                        Some(Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
                        _ => return Ok(Value::Boolean(false)),
                    };
                    return Ok(Value::Boolean(*val >= min && *val <= max));
                }
            }
            Ok(Value::Boolean(false))
        },
    );

    // Age validation function
    engine.register_function(
        "is_adult",
        |args: &[Value], _facts: &Facts| {
            if let Some(Value::Integer(age)) = args.first() {
                Ok(Value::Boolean(*age >= 18))
            } else if let Some(Value::Number(age)) = args.first() {
                Ok(Value::Boolean(*age >= 18.0))
            } else {
                Ok(Value::Boolean(false))
            }
        },
    );
}

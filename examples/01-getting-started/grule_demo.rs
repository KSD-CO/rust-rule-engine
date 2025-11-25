use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Grule-Style Rule Engine Demo");
    println!("================================\n");

    // Demo 1: Create Knowledge Base and load rules
    demo_knowledge_base()?;

    // Demo 2: Facts manipulation
    demo_facts_manipulation()?;

    // Demo 3: Engine execution
    demo_engine_execution()?;

    // Demo 4: E-commerce scenario
    demo_ecommerce_scenario()?;

    Ok(())
}

fn demo_knowledge_base() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("📚 Demo 1: Knowledge Base with Code-based Rules");
    println!("-----------------------------------------------");

    // Create knowledge base
    let kb = KnowledgeBase::new("UserRules");

    println!("✅ Knowledge Base created: {}", kb.get_statistics().name);
    println!("   Total rules: {}", kb.get_statistics().total_rules);
    println!("   Version: {}\n", kb.get_statistics().version);

    Ok(())
}

fn demo_facts_manipulation() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🗂️ Demo 2: Facts Manipulation");
    println!("-----------------------------");

    // Create facts
    let facts = Facts::new();

    // Add user data
    let mut user_props = HashMap::new();
    user_props.insert("Name".to_string(), Value::String("John Doe".to_string()));
    user_props.insert("Age".to_string(), Value::Integer(25));
    user_props.insert("Country".to_string(), Value::String("US".to_string()));
    user_props.insert("SpendingTotal".to_string(), Value::Number(1500.0));
    user_props.insert("IsAdult".to_string(), Value::Boolean(false));
    user_props.insert("IsVIP".to_string(), Value::Boolean(false));

    facts.add_value("User", Value::Object(user_props))?;

    println!("✅ Facts created and populated:");
    if let Some(user) = facts.get("User") {
        println!("   User = {user:?}");
    }
    println!();

    Ok(())
}

fn demo_engine_execution() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Demo 3: Engine Execution");
    println!("---------------------------");

    // Create facts
    let facts = Facts::new();
    let mut user_props = HashMap::new();
    user_props.insert("Age".to_string(), Value::Integer(25));
    user_props.insert("Country".to_string(), Value::String("US".to_string()));
    user_props.insert("SpendingTotal".to_string(), Value::Number(1500.0));
    user_props.insert("IsAdult".to_string(), Value::Boolean(false));
    user_props.insert("IsVIP".to_string(), Value::Boolean(false));
    user_props.insert("Category".to_string(), Value::String("unknown".to_string()));
    user_props.insert("DiscountRate".to_string(), Value::Number(0.0));

    facts.add_value("User", Value::Object(user_props))?;

    // Create knowledge base
    let kb = KnowledgeBase::new("UserRules");

    // Rule 1: Adult Check (salience 10)
    let adult_rule = Rule::new(
        "AdultCheck".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(Condition::new(
                "User.Age".to_string(),
                Operator::GreaterThanOrEqual,
                Value::Integer(18),
            )),
            ConditionGroup::single(Condition::new(
                "User.Country".to_string(),
                Operator::Equal,
                Value::String("US".to_string()),
            )),
        ),
        vec![
            ActionType::MethodCall {
                object: "User".to_string(),
                method: "setIsAdult".to_string(),
                args: vec![Value::Boolean(true)],
            },
            ActionType::MethodCall {
                object: "User".to_string(),
                method: "setCategory".to_string(),
                args: vec![Value::String("Adult".to_string())],
            },
            ActionType::Log {
                message: "User qualified as adult".to_string(),
            },
        ],
    )
    .with_salience(10);

    // Rule 2: VIP Check (salience 20)
    let vip_rule = Rule::new(
        "VIPCheck".to_string(),
        ConditionGroup::and(
            ConditionGroup::and(
                ConditionGroup::single(Condition::new(
                    "User.Age".to_string(),
                    Operator::GreaterThanOrEqual,
                    Value::Integer(21),
                )),
                ConditionGroup::single(Condition::new(
                    "User.IsAdult".to_string(),
                    Operator::Equal,
                    Value::Boolean(true),
                )),
            ),
            ConditionGroup::single(Condition::new(
                "User.SpendingTotal".to_string(),
                Operator::GreaterThan,
                Value::Number(1000.0),
            )),
        ),
        vec![
            ActionType::MethodCall {
                object: "User".to_string(),
                method: "setIsVIP".to_string(),
                args: vec![Value::Boolean(true)],
            },
            ActionType::MethodCall {
                object: "User".to_string(),
                method: "setDiscountRate".to_string(),
                args: vec![Value::Number(0.15)],
            },
            ActionType::Log {
                message: "User upgraded to VIP".to_string(),
            },
        ],
    )
    .with_salience(20);

    // Rule 3: Senior Discount (salience 15)
    let senior_rule = Rule::new(
        "SeniorDiscount".to_string(),
        ConditionGroup::single(Condition::new(
            "User.Age".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Integer(65),
        )),
        vec![
            ActionType::MethodCall {
                object: "User".to_string(),
                method: "setDiscountRate".to_string(),
                args: vec![Value::Number(0.20)],
            },
            ActionType::MethodCall {
                object: "User".to_string(),
                method: "setCategory".to_string(),
                args: vec![Value::String("Senior".to_string())],
            },
            ActionType::Log {
                message: "Senior discount applied".to_string(),
            },
        ],
    )
    .with_salience(15);

    // Add rules to knowledge base
    let _ = kb.add_rule(adult_rule);
    let _ = kb.add_rule(vip_rule);
    let _ = kb.add_rule(senior_rule);

    // Create engine
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 3,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    println!("🏁 Initial state:");
    if let Some(user) = facts.get("User") {
        println!("   User = {user:?}");
    }
    println!();

    // Execute rules
    println!("🚀 Executing rules...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Execution Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    println!("\n🏁 Final state:");
    if let Some(user) = facts.get("User") {
        println!("   User = {user:?}");
    }
    println!();

    Ok(())
}

fn demo_ecommerce_scenario() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🛒 Demo 4: E-commerce Scenario");
    println!("------------------------------");

    // Create facts
    let facts = Facts::new();

    // Customer data
    let mut customer_props = HashMap::new();
    customer_props.insert(
        "Email".to_string(),
        Value::String("customer@example.com".to_string()),
    );
    customer_props.insert("Age".to_string(), Value::Integer(28));
    customer_props.insert("IsNew".to_string(), Value::Boolean(true));
    customer_props.insert("LoyaltyPoints".to_string(), Value::Integer(0));
    customer_props.insert("TotalSpent".to_string(), Value::Number(0.0));

    // Order data
    let mut order_props = HashMap::new();
    order_props.insert("Id".to_string(), Value::String("ORD-12345".to_string()));
    order_props.insert("Amount".to_string(), Value::Number(150.0));
    order_props.insert(
        "Category".to_string(),
        Value::String("electronics".to_string()),
    );
    order_props.insert("DiscountPercent".to_string(), Value::Number(0.0));
    order_props.insert("FinalAmount".to_string(), Value::Number(150.0));

    facts.add_value("Customer", Value::Object(customer_props))?;
    facts.add_value("Order", Value::Object(order_props))?;

    // Create knowledge base
    let kb = KnowledgeBase::new("EcommerceRules");

    // Rule 1: New Customer Discount
    let new_customer_rule = Rule::new(
        "NewCustomerDiscount".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(Condition::new(
                "Customer.IsNew".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            ConditionGroup::single(Condition::new(
                "Order.Amount".to_string(),
                Operator::GreaterThan,
                Value::Number(100.0),
            )),
        ),
        vec![
            ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setDiscountPercent".to_string(),
                args: vec![Value::Number(10.0)],
            },
            ActionType::MethodCall {
                object: "Customer".to_string(),
                method: "setLoyaltyPoints".to_string(),
                args: vec![Value::Integer(100)],
            },
            ActionType::Log {
                message: "New customer discount applied".to_string(),
            },
        ],
    )
    .with_salience(10);

    // Rule 2: Calculate Final Amount
    let calculate_final_rule = Rule::new(
        "CalculateFinalAmount".to_string(),
        ConditionGroup::single(Condition::new(
            "Order.DiscountPercent".to_string(),
            Operator::GreaterThan,
            Value::Number(0.0),
        )),
        vec![ActionType::Log {
            message: "Calculating final amount with discount".to_string(),
        }],
    )
    .with_salience(5);

    // Add rules
    let _ = kb.add_rule(new_customer_rule);
    let _ = kb.add_rule(calculate_final_rule);

    // Create engine
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 3,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    println!("🏁 Initial e-commerce state:");
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer = {customer:?}");
    }
    if let Some(order) = facts.get("Order") {
        println!("   Order = {order:?}");
    }
    println!();

    // Execute rules
    println!("🚀 Executing e-commerce rules...");
    let result = engine.execute(&facts)?;

    println!("\n📊 E-commerce Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    println!("\n🏁 Final e-commerce state:");
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer = {customer:?}");
    }
    if let Some(order) = facts.get("Order") {
        println!("   Order = {order:?}");
    }

    println!("\n🎯 Demo Completed Successfully!");
    println!("   ✅ Knowledge Base management");
    println!("   ✅ Facts manipulation");
    println!("   ✅ Rule execution engine");
    println!("   ✅ E-commerce scenario");
    println!("   ✅ Method calls and function calls");
    println!("   ✅ Salience-based rule ordering");

    Ok(())
}

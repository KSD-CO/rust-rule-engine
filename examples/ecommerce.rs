use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("=== E-commerce Rules Demo ===");
    println!("==============================\n");

    // Create customer data
    let mut customer_props = HashMap::new();
    customer_props.insert("IsNew".to_string(), Value::Boolean(true));
    customer_props.insert(
        "Membership".to_string(),
        Value::String("standard".to_string()),
    );
    customer_props.insert("TotalSpent".to_string(), Value::Number(750.0));
    customer_props.insert("WelcomeEmailSent".to_string(), Value::Boolean(false));

    // Create order data
    let mut order_props = HashMap::new();
    order_props.insert("Total".to_string(), Value::Number(75.0));
    order_props.insert("ItemCount".to_string(), Value::Integer(2));
    order_props.insert(
        "Category".to_string(),
        Value::String("clothing".to_string()),
    );
    order_props.insert("DiscountRate".to_string(), Value::Number(0.0));
    order_props.insert("FreeShipping".to_string(), Value::Boolean(false));
    order_props.insert(
        "DiscountType".to_string(),
        Value::String("none".to_string()),
    );

    // Create promotion data
    let mut promotion_props = HashMap::new();
    promotion_props.insert("SeasonalDiscount".to_string(), Value::Number(0.05));
    promotion_props.insert("BulkDiscount".to_string(), Value::Number(0.10));
    promotion_props.insert("Active".to_string(), Value::Boolean(true));

    // Create facts
    let facts = Facts::new();
    facts.add_value("Customer", Value::Object(customer_props))?;
    facts.add_value("Order", Value::Object(order_props))?;
    facts.add_value("Promotion", Value::Object(promotion_props))?;

    println!("🏁 Initial e-commerce data:");
    if let Some(customer) = facts.get("Customer") {
        println!("   Customer = {customer:?}");
    }
    if let Some(order) = facts.get("Order") {
        println!("   Order = {order:?}");
    }
    if let Some(promotion) = facts.get("Promotion") {
        println!("   Promotion = {promotion:?}");
    }
    println!();

    // Create knowledge base for ecommerce rules
    let kb = KnowledgeBase::new("EcommerceRules");

    // Rule 1: New Customer Discount (salience 25)
    let new_customer_rule = Rule::new(
        "NewCustomerDiscount".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(Condition::new(
                "Customer.IsNew".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            ConditionGroup::single(Condition::new(
                "Order.Total".to_string(),
                Operator::GreaterThan,
                Value::Number(50.0),
            )),
        ),
        vec![
            ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setDiscountRate".to_string(),
                args: vec![Value::Number(0.10)],
            },
            ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setDiscountType".to_string(),
                args: vec![Value::String("welcome".to_string())],
            },
            ActionType::MethodCall {
                object: "Customer".to_string(),
                method: "setWelcomeEmailSent".to_string(),
                args: vec![Value::Boolean(true)],
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String(
                    "New customer welcome discount applied".to_string(),
                )],
            },
        ],
    )
    .with_salience(25);

    // Rule 2: Premium Member Benefits (salience 20)
    let premium_member_rule = Rule::new(
        "PremiumMemberBenefits".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(Condition::new(
                "Customer.Membership".to_string(),
                Operator::Equal,
                Value::String("premium".to_string()),
            )),
            ConditionGroup::single(Condition::new(
                "Order.Total".to_string(),
                Operator::GreaterThan,
                Value::Number(100.0),
            )),
        ),
        vec![
            ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setDiscountRate".to_string(),
                args: vec![Value::Number(0.15)],
            },
            ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setFreeShipping".to_string(),
                args: vec![Value::Boolean(true)],
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String("Premium member benefits applied".to_string())],
            },
        ],
    )
    .with_salience(20);

    // Rule 3: VIP Upgrade (salience 30)
    let vip_upgrade_rule = Rule::new(
        "VIPUpgrade".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(Condition::new(
                "Customer.TotalSpent".to_string(),
                Operator::GreaterThan,
                Value::Number(1000.0),
            )),
            ConditionGroup::single(Condition::new(
                "Customer.Membership".to_string(),
                Operator::NotEqual,
                Value::String("VIP".to_string()),
            )),
        ),
        vec![
            ActionType::MethodCall {
                object: "Customer".to_string(),
                method: "setMembership".to_string(),
                args: vec![Value::String("VIP".to_string())],
            },
            ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setDiscountRate".to_string(),
                args: vec![Value::Number(0.25)],
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String(
                    "Customer upgraded to VIP membership".to_string(),
                )],
            },
        ],
    )
    .with_salience(30);

    // Rule 4: Free Shipping for Large Orders (salience 15)
    let free_shipping_rule = Rule::new(
        "FreeShippingLargeOrders".to_string(),
        ConditionGroup::single(Condition::new(
            "Order.Total".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Number(100.0),
        )),
        vec![
            ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setFreeShipping".to_string(),
                args: vec![Value::Boolean(true)],
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String(
                    "Free shipping applied for large order".to_string(),
                )],
            },
        ],
    )
    .with_salience(15);

    // Rule 5: Bulk Discount (salience 12)
    let bulk_discount_rule = Rule::new(
        "BulkDiscount".to_string(),
        ConditionGroup::single(Condition::new(
            "Order.ItemCount".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Integer(5),
        )),
        vec![
            ActionType::Call {
                function: "max".to_string(),
                args: vec![
                    Value::String("Order.DiscountRate".to_string()),
                    Value::Number(0.10),
                ],
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String("Bulk discount applied".to_string())],
            },
        ],
    )
    .with_salience(12);

    // Rule 6: Seasonal Promotion (salience 10)
    let seasonal_promotion_rule = Rule::new(
        "SeasonalPromotion".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(Condition::new(
                "Promotion.Active".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            )),
            ConditionGroup::single(Condition::new(
                "Order.Category".to_string(),
                Operator::Equal,
                Value::String("clothing".to_string()),
            )),
        ),
        vec![
            ActionType::Call {
                function: "max".to_string(),
                args: vec![
                    Value::String("Order.DiscountRate".to_string()),
                    Value::String("Promotion.SeasonalDiscount".to_string()),
                ],
            },
            ActionType::Call {
                function: "log".to_string(),
                args: vec![Value::String("Seasonal promotion applied".to_string())],
            },
        ],
    )
    .with_salience(10);

    // Add rules to knowledge base
    let _ = kb.add_rule(new_customer_rule);
    let _ = kb.add_rule(premium_member_rule);
    let _ = kb.add_rule(vip_upgrade_rule);
    let _ = kb.add_rule(free_shipping_rule);
    let _ = kb.add_rule(bulk_discount_rule);
    let _ = kb.add_rule(seasonal_promotion_rule);

    println!(
        "📚 Knowledge Base loaded with {} rules",
        kb.get_statistics().total_rules
    );
    println!("🔥 Rules: {:?}\n", kb.get_rule_names());

    // Create engine with debug mode
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 5,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Execute rules
    println!("🚀 Executing e-commerce rules...");
    let result = engine.execute(&facts)?;

    println!("\n📊 E-commerce Execution Results:");
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
    if let Some(promotion) = facts.get("Promotion") {
        println!("   Promotion = {promotion:?}");
    }

    println!("\n🎯 E-commerce Rules Demonstrated:");
    println!("   🎁 New Customer Welcome Discount");
    println!("   💎 Premium Member Benefits");
    println!("   🌟 VIP Membership Upgrades");
    println!("   🚚 Free Shipping Rules");
    println!("   📦 Bulk Purchase Discounts");
    println!("   🏷️ Seasonal Promotions");

    Ok(())
}

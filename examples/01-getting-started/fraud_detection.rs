use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Demo: Fraud Detection System ===\n");

    // Create transaction data
    let mut transaction_props = HashMap::new();
    transaction_props.insert("Amount".to_string(), Value::Number(5000.0));
    transaction_props.insert("Location".to_string(), Value::String("FOREIGN".to_string()));
    transaction_props.insert("Time".to_string(), Value::String("02:30".to_string()));
    transaction_props.insert("CardType".to_string(), Value::String("CREDIT".to_string()));
    transaction_props.insert(
        "MerchantCategory".to_string(),
        Value::String("CASINO".to_string()),
    );

    let mut account_props = HashMap::new();
    account_props.insert("Balance".to_string(), Value::Number(2000.0));
    account_props.insert("DailyLimit".to_string(), Value::Number(3000.0));
    account_props.insert("RiskLevel".to_string(), Value::String("LOW".to_string()));
    account_props.insert("IsActive".to_string(), Value::Boolean(true));
    account_props.insert(
        "LastLoginLocation".to_string(),
        Value::String("DOMESTIC".to_string()),
    );

    let mut alert_props = HashMap::new();
    alert_props.insert("FraudScore".to_string(), Value::Number(0.0));
    alert_props.insert("Status".to_string(), Value::String("PENDING".to_string()));
    alert_props.insert("Alerts".to_string(), Value::Array(vec![]));

    // Create facts
    let facts = Facts::new();
    facts.add_value("Transaction", Value::Object(transaction_props))?;
    facts.add_value("Account", Value::Object(account_props))?;
    facts.add_value("Alert", Value::Object(alert_props))?;

    println!("🏁 Initial transaction data:");
    if let Some(transaction) = facts.get("Transaction") {
        println!("   Transaction = {transaction:?}");
    }
    if let Some(account) = facts.get("Account") {
        println!("   Account = {account:?}");
    }
    if let Some(alert) = facts.get("Alert") {
        println!("   Alert = {alert:?}");
    }
    println!();

    // Create knowledge base for fraud detection rules
    let kb = KnowledgeBase::new("FraudDetectionSystem"); // Rule 1: High Amount Transaction Alert
    let high_amount_rule = Rule::new(
        "HighAmountAlert".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(Condition::new(
                "Transaction.Amount".to_string(),
                Operator::GreaterThan,
                Value::Number(3000.0),
            )),
            ConditionGroup::single(Condition::new(
                "Account.DailyLimit".to_string(),
                Operator::LessThan,
                Value::String("Transaction.Amount".to_string()),
            )),
        ),
        vec![
            ActionType::Log {
                message: "HIGH AMOUNT ALERT: Transaction exceeds daily limit".to_string(),
            },
            ActionType::MethodCall {
                object: "Alert".to_string(),
                method: "setFraudScore".to_string(),
                args: vec![Value::Number(50.0)],
            },
            ActionType::Set {
                field: "Alert.Status".to_string(),
                value: Value::String("FLAGGED".to_string()),
            },
        ],
    )
    .with_salience(10);

    // Rule 2: Foreign Location Alert
    let foreign_location_rule = Rule::new(
        "ForeignLocationAlert".to_string(),
        ConditionGroup::and(
            ConditionGroup::single(Condition::new(
                "Transaction.Location".to_string(),
                Operator::Equal,
                Value::String("FOREIGN".to_string()),
            )),
            ConditionGroup::single(Condition::new(
                "Account.LastLoginLocation".to_string(),
                Operator::Equal,
                Value::String("DOMESTIC".to_string()),
            )),
        ),
        vec![
            ActionType::Log {
                message: "LOCATION ALERT: Foreign transaction detected".to_string(),
            },
            ActionType::MethodCall {
                object: "Alert".to_string(),
                method: "addFraudScore".to_string(),
                args: vec![Value::Number(30.0)],
            },
        ],
    )
    .with_salience(8);

    // Rule 3: Late Night Transaction Alert
    let late_night_rule = Rule::new(
        "LateNightAlert".to_string(),
        ConditionGroup::single(Condition::new(
            "Transaction.Time".to_string(),
            Operator::Contains,
            Value::String("02:".to_string()),
        )),
        vec![
            ActionType::Log {
                message: "TIME ALERT: Late night transaction".to_string(),
            },
            ActionType::MethodCall {
                object: "Alert".to_string(),
                method: "setFraudScore".to_string(),
                args: vec![Value::Number(20.0)],
            },
        ],
    )
    .with_salience(6);

    // Rule 4: High Risk Merchant Category
    let high_risk_merchant_rule = Rule::new(
        "HighRiskMerchantAlert".to_string(),
        ConditionGroup::single(Condition::new(
            "Transaction.MerchantCategory".to_string(),
            Operator::Equal,
            Value::String("CASINO".to_string()),
        )),
        vec![
            ActionType::Log {
                message: "MERCHANT ALERT: High-risk merchant category".to_string(),
            },
            ActionType::MethodCall {
                object: "Alert".to_string(),
                method: "setFraudScore".to_string(),
                args: vec![Value::Number(40.0)],
            },
        ],
    )
    .with_salience(7);

    // Rule 5: Critical Fraud Score Alert
    let critical_fraud_rule = Rule::new(
        "CriticalFraudAlert".to_string(),
        ConditionGroup::single(Condition::new(
            "Alert.FraudScore".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Number(70.0),
        )),
        vec![
            ActionType::Log {
                message: "🚨 CRITICAL FRAUD ALERT! Blocking transaction!".to_string(),
            },
            ActionType::Set {
                field: "Alert.Status".to_string(),
                value: Value::String("BLOCKED".to_string()),
            },
            ActionType::Custom {
                action_type: "format".to_string(),
                params: std::collections::HashMap::from([("args".to_string(), Value::String("Transaction TX123456 BLOCKED - Fraud Score: Alert.FraudScore".to_string()))]),
            },
        ],
    )
    .with_salience(15);

    // Rule 6: Account Insufficient Funds
    let insufficient_funds_rule = Rule::new(
        "InsufficientFundsAlert".to_string(),
        ConditionGroup::single(Condition::new(
            "Account.Balance".to_string(),
            Operator::LessThan,
            Value::String("Transaction.Amount".to_string()),
        )),
        vec![
            ActionType::Log {
                message: "💰 INSUFFICIENT FUNDS: Transaction declined".to_string(),
            },
            ActionType::Set {
                field: "Alert.Status".to_string(),
                value: Value::String("DECLINED".to_string()),
            },
        ],
    )
    .with_salience(20); // Highest priority

    // Add rules to knowledge base
    let _ = kb.add_rule(high_amount_rule);
    let _ = kb.add_rule(foreign_location_rule);
    let _ = kb.add_rule(late_night_rule);
    let _ = kb.add_rule(high_risk_merchant_rule);
    let _ = kb.add_rule(critical_fraud_rule);
    let _ = kb.add_rule(insufficient_funds_rule);

    // Create engine with debug mode
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 10,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    println!("🔍 Running fraud detection analysis...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Fraud Detection Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    println!("\n🏁 Final analysis:");
    if let Some(transaction) = facts.get("Transaction") {
        println!("   Transaction = {transaction:?}");
    }
    if let Some(account) = facts.get("Account") {
        println!("   Account = {account:?}");
    }
    if let Some(alert) = facts.get("Alert") {
        println!("   Alert = {alert:?}");
    }

    println!("\n🎯 Fraud Detection Rules Demonstrated:");
    println!("   🚨 High Amount Transactions");
    println!("   🌍 Foreign Location Detection");
    println!("   🌙 Late Night Activity");
    println!("   🎰 High-Risk Merchant Categories");
    println!("   ⚠️ Critical Fraud Score Alerts");
    println!("   💰 Insufficient Funds Checks");

    Ok(())
}

/// AI Functions in WHEN Clause - Demonstration
///
/// This example demonstrates the NEW feature in v0.10.0:
/// Calling AI functions directly in the WHEN clause of rules!
///
/// Previously, you had to use rule chaining:
///   Rule 1: Call AI in THEN, store result
///   Rule 2: Check result in WHEN
///
/// Now you can directly check AI results:
///   Rule: Check AI result in WHEN clause directly!

use rust_rule_engine::{
    engine::facts::Facts,
    engine::knowledge_base::KnowledgeBase,
    engine::{EngineConfig, RustRuleEngine},
    types::Value,
    GRLParser,
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AI Functions in WHEN Clause Demo ===\n");

    let mut engine = create_engine()?;

    // Test 1: Sentiment analysis in WHEN
    println!("📝 Test 1: Direct Sentiment Check in WHEN");
    println!("──────────────────────────────────────");
    test_sentiment_in_when(&mut engine)?;

    println!("\n💰 Test 2: Direct Fraud Detection in WHEN");
    println!("──────────────────────────────────────");
    test_fraud_in_when(&mut engine)?;

    println!("\n🧠 Test 3: Direct LLM Decision in WHEN");
    println!("──────────────────────────────────────");
    test_llm_in_when(&mut engine)?;

    println!("\n✅ All tests completed!");
    Ok(())
}

fn create_engine() -> Result<RustRuleEngine, Box<dyn std::error::Error>> {
    let config = EngineConfig {
        max_cycles: 3,  // Reduce cycles for cleaner output
        timeout: Some(Duration::from_secs(5)),
        enable_stats: true,
        debug_mode: false,  // Keep false for cleaner output
    };

    let mut engine = RustRuleEngine::with_config(KnowledgeBase::new("AI-WHEN"), config);

    // Register AI Functions
    register_ai_functions(&mut engine)?;

    // Register action handlers
    engine.register_action_handler("set", |_, _| Ok(()));
    engine.register_action_handler("Log", |_, _| Ok(()));
    engine.register_action_handler("Alert", |_, _| Ok(()));

    // Load rules with AI functions in WHEN clause
    load_rules_with_ai_in_when(&mut engine)?;

    Ok(engine)
}

fn register_ai_functions(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    // AI Sentiment Analysis
    engine.register_function("aiSentiment", |args, _facts| {
        let text = args[0].as_string().unwrap_or("".to_string());
        let sentiment = if text.contains("terrible") || text.contains("awful") {
            "negative"
        } else if text.contains("great") || text.contains("excellent") {
            "positive"
        } else if text.contains("toxic") || text.contains("hate") {
            "toxic"
        } else {
            "neutral"
        };
        println!("   🤖 aiSentiment('{}...') -> {}", &text[..text.len().min(30)], sentiment);
        Ok(Value::String(sentiment.to_string()))
    });

    // AI Fraud Detection
    engine.register_function("aiFraud", |args, _facts| {
        let amount = args[0].as_number().unwrap_or(0.0);
        let user_id = args[1].as_string().unwrap_or("unknown".to_string());
        let is_fraud = amount > 2000.0 || user_id.contains("suspicious");
        println!("   🛡️ aiFraud(${:.2}, '{}') -> {}", amount, user_id, is_fraud);
        Ok(Value::Boolean(is_fraud))
    });

    // AI LLM Reasoning
    engine.register_function("aiLLM", |args, _facts| {
        let prompt = args[0].as_string().unwrap_or("".to_string());
        let decision = if prompt.contains("high complexity") {
            "REQUIRES_MANUAL_REVIEW"
        } else if prompt.contains("approve") {
            "APPROVED"
        } else {
            "REJECTED"
        };
        println!("   🧠 aiLLM('{}...') -> {}", &prompt[..prompt.len().min(30)], decision);
        Ok(Value::String(decision.to_string()))
    });

    println!("✅ Registered 3 AI functions\n");
    Ok(())
}

fn load_rules_with_ai_in_when(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let rules = vec![
        // ✨ NEW: Direct sentiment check in WHEN clause
        r#"
rule "Flag Negative Sentiment" salience 100 {
    when
        aiSentiment(Text.content) == "negative"
    then
        set(Text.flagged, true);
        set(Text.action, "escalate");
        Alert("Negative sentiment detected!");
}
        "#,
        // ✨ NEW: Direct fraud check in WHEN clause
        r#"
rule "Block Fraudulent Transaction" salience 90 {
    when
        aiFraud(Transaction.amount, Transaction.userId) == true
    then
        set(Transaction.status, "blocked");
        Alert("Fraud detected - transaction blocked!");
}
        "#,
        // ✨ NEW: Direct LLM decision in WHEN clause
        r#"
rule "Manual Review Required" salience 80 {
    when
        aiLLM(Case.description) == "REQUIRES_MANUAL_REVIEW"
    then
        set(Case.status, "pending_review");
        set(Case.assignee, "senior_analyst");
        Alert("Complex case requires manual review");
}
        "#,
        // Positive sentiment handler
        r#"
rule "Reward Positive Feedback" salience 70 {
    when
        aiSentiment(Text.content) == "positive"
    then
        set(Text.reward, 10);
        Log("Positive feedback - reward issued");
}
        "#,
        // Safe transaction handler
        r#"
rule "Approve Safe Transaction" salience 60 {
    when
        aiFraud(Transaction.amount, Transaction.userId) == false
    then
        set(Transaction.status, "approved");
        Log("Transaction approved");
}
        "#,
    ];

    for rule_grl in &rules {
        let parsed_rules = GRLParser::parse_rules(rule_grl)?;
        for rule in parsed_rules {
            engine.knowledge_base_mut().add_rule(rule)?;
        }
    }

    println!("✅ Loaded {} rules with AI in WHEN clause\n", rules.len());
    Ok(())
}

fn test_sentiment_in_when(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    facts.add_value("Text.content", Value::String("This product is terrible and awful!".to_string()))?;

    println!("Input: 'This product is terrible and awful!'");
    let result = engine.execute(&facts)?;

    println!("Rules fired: {}", result.rules_fired);
    if let Some(flagged) = facts.get("Text.flagged") {
        println!("✅ Text flagged: {:?}", flagged);
    }
    if let Some(action) = facts.get("Text.action") {
        println!("✅ Action: {:?}", action);
    }

    Ok(())
}

fn test_fraud_in_when(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    facts.add_value("Transaction.amount", Value::Number(2500.0))?;
    facts.add_value("Transaction.userId", Value::String("user_123".to_string()))?;

    println!("Input: amount=$2500, user='user_123'");
    let result = engine.execute(&facts)?;

    println!("Rules fired: {}", result.rules_fired);
    if let Some(status) = facts.get("Transaction.status") {
        println!("✅ Transaction status: {:?}", status);
    }

    Ok(())
}

fn test_llm_in_when(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    facts.add_value("Case.description", Value::String("Analyze this high complexity business case".to_string()))?;

    println!("Input: 'Analyze this high complexity business case'");
    let result = engine.execute(&facts)?;

    println!("Rules fired: {}", result.rules_fired);
    if let Some(status) = facts.get("Case.status") {
        println!("✅ Case status: {:?}", status);
    }
    if let Some(assignee) = facts.get("Case.assignee") {
        println!("✅ Assigned to: {:?}", assignee);
    }

    Ok(())
}

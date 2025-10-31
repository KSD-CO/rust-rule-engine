/// Test AI Functions in Rules
///
/// This example demonstrates how AI functions (aiSentiment, aiFraud, aiLLM, aiScore)
/// can be called within rule actions to make intelligent decisions.

use rust_rule_engine::{
    engine::facts::Facts,
    engine::knowledge_base::KnowledgeBase,
    engine::{EngineConfig, RustRuleEngine},
    types::Value,
    GRLParser,
};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing AI Functions in Rules ===\n");

    // Create engine with AI functions
    let mut engine = create_engine_with_ai()?;

    // Test 1: Sentiment Analysis
    println!("📝 Test 1: AI Sentiment Analysis");
    println!("─────────────────────────────────");
    test_sentiment_analysis(&mut engine)?;

    println!("\n💰 Test 2: AI Fraud Detection");
    println!("─────────────────────────────────");
    test_fraud_detection(&mut engine)?;

    println!("\n🧠 Test 3: LLM Business Decision");
    println!("─────────────────────────────────");
    test_llm_decision(&mut engine)?;

    println!("\n📊 Test 4: ML Price Scoring");
    println!("─────────────────────────────────");
    test_ml_scoring(&mut engine)?;

    println!("\n✅ All AI function tests completed!");
    Ok(())
}

fn create_engine_with_ai() -> Result<RustRuleEngine, Box<dyn std::error::Error>> {
    let config = EngineConfig {
        max_cycles: 10,
        timeout: Some(Duration::from_secs(5)),
        enable_stats: true,
        debug_mode: false,
    };

    let mut engine = RustRuleEngine::with_config(KnowledgeBase::new("AI-Test"), config);

    // Register AI Functions
    register_ai_functions(&mut engine)?;

    // Register simple action handlers
    engine.register_action_handler("set", |_, _| Ok(()));
    engine.register_action_handler("Log", |_, _| Ok(()));

    // Load AI Rules
    load_ai_rules(&mut engine)?;

    Ok(engine)
}

fn register_ai_functions(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    // Sentiment Analysis
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

    // Fraud Detection
    engine.register_function("aiFraud", |args, _facts| {
        let amount = args[0].as_number().unwrap_or(0.0);
        let user_id = args[1].as_string().unwrap_or("unknown".to_string());
        let is_fraud = amount > 2000.0 || user_id.contains("suspicious");
        println!("   🛡️ aiFraud(${:.2}, '{}') -> {}", amount, user_id, is_fraud);
        Ok(Value::Boolean(is_fraud))
    });

    // LLM Reasoning
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

    // ML Scoring
    engine.register_function("aiScore", |args, _facts| {
        let features: Vec<f64> = args.iter().filter_map(|v| v.as_number()).collect();
        let score = features.iter().sum::<f64>() / features.len().max(1) as f64;
        println!("   📊 aiScore({:?}) -> {:.3}", features, score);
        Ok(Value::Number(score))
    });

    println!("✅ Registered 4 AI functions\n");
    Ok(())
}

fn load_ai_rules(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let rules = vec![
        // Sentiment analysis rules
        r#"
rule "Analyze Text Sentiment" salience 100 {
    when
        Text.needsAnalysis == true
    then
        set(Text.sentiment, aiSentiment(Text.content));
        set(Text.analyzed, true);
        Log("Sentiment analysis completed");
}
        "#,
        r#"
rule "Flag Negative Content" salience 90 {
    when
        Text.sentiment == "negative"
    then
        set(Text.flagged, true);
        set(Text.priority, "high");
        Log("Negative sentiment flagged");
}
        "#,
        // Fraud detection rules
        r#"
rule "Check Transaction Fraud" salience 80 {
    when
        Transaction.needsCheck == true
    then
        set(Transaction.isFraud, aiFraud(Transaction.amount, Transaction.userId));
        Log("Fraud check completed");
}
        "#,
        r#"
rule "Block Fraud Transaction" salience 75 {
    when
        Transaction.isFraud == true
    then
        set(Transaction.status, "blocked");
        Log("Transaction blocked - fraud detected");
}
        "#,
        // LLM decision rules
        r#"
rule "Get LLM Decision" salience 70 {
    when
        Decision.needsAI == true
    then
        set(Decision.recommendation, aiLLM(Decision.prompt));
        Log("LLM recommendation obtained");
}
        "#,
        // ML scoring rules
        r#"
rule "Calculate ML Score" salience 60 {
    when
        Item.needsScoring == true
    then
        set(Item.score, aiScore(Item.feature1, Item.feature2, Item.feature3));
        Log("ML scoring completed");
}
        "#,
        r#"
rule "High Score Action" salience 55 {
    when
        Item.score > 5.0
    then
        set(Item.category, "premium");
        Log("Item categorized as premium");
}
        "#,
    ];

    for rule_grl in &rules {
        let parsed_rules = GRLParser::parse_rules(rule_grl)?;
        for rule in parsed_rules {
            engine.knowledge_base_mut().add_rule(rule)?;
        }
    }

    println!("✅ Loaded {} AI-enhanced rules\n", rules.len());
    Ok(())
}

fn test_sentiment_analysis(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    facts.add_value("Text.needsAnalysis", Value::Boolean(true))?;
    facts.add_value("Text.content", Value::String("This product is terrible and awful!".to_string()))?;

    println!("Input: 'This product is terrible and awful!'");
    let result = engine.execute(&facts)?;

    println!("Rules fired: {}", result.rules_fired);
    if let Some(sentiment) = facts.get("Text.sentiment") {
        println!("✅ Detected sentiment: {:?}", sentiment);
    }
    if let Some(flagged) = facts.get("Text.flagged") {
        println!("✅ Content flagged: {:?}", flagged);
    }

    Ok(())
}

fn test_fraud_detection(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    facts.add_value("Transaction.needsCheck", Value::Boolean(true))?;
    facts.add_value("Transaction.amount", Value::Number(2500.0))?;
    facts.add_value("Transaction.userId", Value::String("user_123".to_string()))?;

    println!("Input: amount=$2500, user='user_123'");
    let result = engine.execute(&facts)?;

    println!("Rules fired: {}", result.rules_fired);
    if let Some(is_fraud) = facts.get("Transaction.isFraud") {
        println!("✅ Fraud detected: {:?}", is_fraud);
    }
    if let Some(status) = facts.get("Transaction.status") {
        println!("✅ Transaction status: {:?}", status);
    }

    Ok(())
}

fn test_llm_decision(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    facts.add_value("Decision.needsAI", Value::Boolean(true))?;
    facts.add_value("Decision.prompt", Value::String("Analyze this high complexity business case".to_string()))?;

    println!("Input: 'Analyze this high complexity business case'");
    let result = engine.execute(&facts)?;

    println!("Rules fired: {}", result.rules_fired);
    if let Some(recommendation) = facts.get("Decision.recommendation") {
        println!("✅ LLM recommendation: {:?}", recommendation);
    }

    Ok(())
}

fn test_ml_scoring(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    facts.add_value("Item.needsScoring", Value::Boolean(true))?;
    facts.add_value("Item.feature1", Value::Number(7.5))?;
    facts.add_value("Item.feature2", Value::Number(8.2))?;
    facts.add_value("Item.feature3", Value::Number(6.8))?;

    println!("Input: features=[7.5, 8.2, 6.8]");
    let result = engine.execute(&facts)?;

    println!("Rules fired: {}", result.rules_fired);
    if let Some(score) = facts.get("Item.score") {
        println!("✅ ML score: {:?}", score);
    }
    if let Some(category) = facts.get("Item.category") {
        println!("✅ Item category: {:?}", category);
    }

    Ok(())
}

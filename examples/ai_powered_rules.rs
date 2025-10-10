/// AI-Powered Rule Engine Example
///
/// This example demonstrates integrating AI/ML models into the Rust Rule Engine,
/// similar to Drools' Pragmatic AI approach. Features include:
/// - AI model inference in rules
/// - LLM-based decision making
/// - ML model predictions for rule conditions
/// - Dynamic rule generation from AI
/// - Sentiment analysis for customer service rules
/// - Fraud detection with ML models
use rust_rule_engine::{
    engine::facts::Facts,
    engine::knowledge_base::KnowledgeBase,
    engine::{EngineConfig, RustRuleEngine},
    types::Value,
    GRLParser,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio;

/// AI Model Response for sentiment analysis
#[derive(Debug, Serialize, Deserialize)]
struct SentimentResponse {
    sentiment: String,
    confidence: f64,
    emotions: HashMap<String, f64>,
}

/// AI Model Response for fraud detection
#[derive(Debug, Serialize, Deserialize)]
struct FraudResponse {
    is_fraud: bool,
    risk_score: f64,
    reasons: Vec<String>,
}

/// AI Model Response for customer tier prediction
#[derive(Debug, Serialize, Deserialize)]
struct TierPredictionResponse {
    predicted_tier: String,
    confidence: f64,
    factors: HashMap<String, f64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting AI-Powered Rust Rule Engine");
    println!("======================================\n");

    // Create AI-enhanced rule engine
    let mut engine = create_ai_engine().await?;

    // Test AI-powered rules with different scenarios
    println!("🧪 Testing AI-Enhanced Rule Scenarios:");

    // Scenario 1: Customer Service with Sentiment Analysis
    println!("\n📞 Scenario 1: Customer Service Automation");
    test_customer_service_ai(&mut engine).await?;

    // Scenario 2: Fraud Detection with ML
    println!("\n🛡️ Scenario 2: AI-Powered Fraud Detection");
    test_fraud_detection_ai(&mut engine).await?;

    // Scenario 3: Dynamic Customer Tier Assignment
    println!("\n🏆 Scenario 3: ML-Based Customer Tier Prediction");
    test_tier_prediction_ai(&mut engine).await?;

    // Scenario 4: LLM-Generated Rules
    println!("\n🧠 Scenario 4: LLM-Generated Business Rules");
    test_llm_rule_generation(&mut engine).await?;

    println!("\n✅ AI-Powered Rule Engine Demo Complete!");
    Ok(())
}

/// Create AI-enhanced rule engine with ML/AI function integrations
async fn create_ai_engine() -> Result<RustRuleEngine, Box<dyn std::error::Error>> {
    let config = EngineConfig {
        max_cycles: 10,
        timeout: Some(Duration::from_secs(30)),
        enable_stats: true,
        debug_mode: true,
    };

    let mut engine = RustRuleEngine::with_config(KnowledgeBase::new("AI-Enhanced-Engine"), config);

    // Register AI-powered functions
    register_ai_functions(&mut engine).await?;

    // Register basic utility functions
    engine.register_function("logMessage", |args, _facts| {
        let message = args[0].as_string().unwrap_or("".to_string());
        println!("📋 LOG: {}", message);
        Ok(Value::Boolean(true))
    });

    engine.register_function("sendNotification", |args, _facts| {
        let message = args[0].as_string().unwrap_or("".to_string());
        let recipient = args[1].as_string().unwrap_or("unknown".to_string());
        println!("📧 Notification to {}: {}", recipient, message);
        Ok(Value::Boolean(true))
    });

    // Load AI-enhanced rules
    load_ai_rules(&mut engine).await?;

    Ok(engine)
}

/// Register AI/ML model functions
async fn register_ai_functions(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Sentiment Analysis Function
    engine.register_function("analyzeSentiment", |args, _facts| {
        let text = args[0].as_string().unwrap_or("".to_string());

        // Simulate AI sentiment analysis (in real implementation, call actual AI API)
        let sentiment_result = simulate_sentiment_analysis(&text);

        println!(
            "🤖 AI Sentiment Analysis: '{}' -> {} (confidence: {:.2})",
            text, sentiment_result.sentiment, sentiment_result.confidence
        );

        Ok(Value::String(sentiment_result.sentiment))
    });

    // 2. Fraud Detection Function
    engine.register_function("detectFraud", |args, facts| {
        let transaction_amount = args[0].as_number().unwrap_or(0.0);
        let user_id = args[1].as_string().unwrap_or("unknown".to_string());

        // Get additional context from facts
        let user_history = facts.get("UserHistory");
        let location = facts.get("Location");

        // Simulate ML fraud detection
        let fraud_result = simulate_fraud_detection(
            transaction_amount,
            &user_id,
            user_history.as_ref(),
            location.as_ref(),
        );

        println!(
            "🛡️ AI Fraud Detection: ${:.2} by {} -> {} (risk: {:.1}%)",
            transaction_amount,
            user_id,
            if fraud_result.is_fraud {
                "FRAUD"
            } else {
                "SAFE"
            },
            fraud_result.risk_score * 100.0
        );

        Ok(Value::Boolean(fraud_result.is_fraud))
    });

    // 3. Customer Tier Prediction Function
    engine.register_function("predictTier", |args, facts| {
        let customer_id = args[0].as_string().unwrap_or("unknown".to_string());

        // Get customer data from facts
        let customer_data = facts.get("Customer");

        // Simulate ML tier prediction
        let tier_result = simulate_tier_prediction(&customer_id, customer_data.as_ref());

        println!(
            "🏆 AI Tier Prediction: {} -> {} (confidence: {:.1}%)",
            customer_id,
            tier_result.predicted_tier,
            tier_result.confidence * 100.0
        );

        Ok(Value::String(tier_result.predicted_tier))
    });

    // 4. LLM Decision Function
    engine.register_function("llmDecision", |args, facts| {
        let context = args[0].as_string().unwrap_or("".to_string());
        let question = args[1].as_string().unwrap_or("".to_string());

        // Simulate LLM reasoning
        let decision = simulate_llm_decision(&context, &question, facts);

        println!("🧠 LLM Decision: '{}' -> {}", question, decision);

        Ok(Value::String(decision))
    });

    // 5. Real-time ML Model Scoring
    engine.register_function("mlScore", |args, _facts| {
        let features = &args[0..]; // All args as features

        // Simulate real-time ML model scoring
        let score = simulate_ml_scoring(features);

        println!("📊 ML Model Score: {:.3}", score);

        Ok(Value::Number(score))
    });

    println!("✅ Registered 5 AI-powered functions");
    Ok(())
}

/// Load AI-enhanced business rules
async fn load_ai_rules(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let ai_rules = vec![
        // Rule 1: AI-powered customer service routing
        r#"
rule "AI Customer Service Routing" salience 100 {
    when
        CustomerMessage.type == "complaint"
    then
        analyzeSentiment(CustomerMessage.text);
        set(Ticket.priority, "high");
        set(Ticket.assignTo, "senior_agent");
        logMessage("🤖 AI analyzing customer sentiment");
}
        "#,
        // Rule 2: AI fraud detection
        r#"
rule "AI Fraud Detection" salience 90 {
    when
        Transaction.amount > 1000
    then
        detectFraud(Transaction.amount, Transaction.userId);
        set(Transaction.status, "under_review");
        set(Transaction.requiresReview, true);
        sendNotification("🛡️ Checking for potential fraud", "security@company.com");
        logMessage("AI fraud detection initiated");
}
        "#,
        // Rule 3: ML-based tier assignment
        r#"
rule "AI Tier Assignment" salience 80 {
    when
        Customer.tier == "pending"
    then
        predictTier(Customer.id);
        set(Customer.tierAssignedBy, "AI");
        logMessage("🏆 AI predicting customer tier");
}
        "#,
        // Rule 4: LLM-powered decision making
        r#"
rule "LLM Business Decision" salience 70 {
    when
        BusinessCase.status == "review" &&
        BusinessCase.complexity == "high"
    then
        llmDecision(BusinessCase.description, "Should we approve this case?");
        set(BusinessCase.status, "ai_reviewed");
        logMessage("🧠 LLM analyzing business case");
}
        "#,
        // Rule 5: Real-time ML scoring for dynamic pricing
        r#"
rule "AI Dynamic Pricing" salience 60 {
    when
        Product.category == "premium"
    then
        mlScore(Customer.spending, Market.demand, Product.inventory);
        set(Product.pricingReason, "AI_PREMIUM");
        logMessage("📊 AI calculating dynamic pricing");
}
        "#,
        // Rule 6: Multi-AI ensemble decision
        r#"
rule "AI Ensemble Decision" salience 50 {
    when
        LoanApplication.status == "pending"
    then
        mlScore(Applicant.creditScore, Applicant.income, Applicant.debt);
        analyzeSentiment(Applicant.interview);
        set(LoanApplication.approvedBy, "AI_ENSEMBLE");
        logMessage("🎯 AI ensemble evaluating loan");
}
        "#,
    ];

    for rule_grl in &ai_rules {
        let rules = GRLParser::parse_rules(rule_grl)?;
        for rule in rules {
            engine.knowledge_base_mut().add_rule(rule)?;
        }
    }

    println!("✅ Loaded {} AI-enhanced rules", ai_rules.len());
    Ok(())
}

/// Test customer service AI scenario
async fn test_customer_service_ai(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    // Customer complaint scenario
    let message_data = json!({
        "type": "complaint",
        "text": "This product is terrible and I want my money back immediately!",
        "customerId": "CUST123",
        "timestamp": "2025-10-10T10:00:00Z"
    });

    let ticket_data = json!({
        "id": "TICKET789",
        "priority": "normal",
        "assignTo": "auto_queue",
        "status": "new"
    });

    facts.add_value("CustomerMessage", json_to_engine_value(message_data))?;
    facts.add_value("Ticket", json_to_engine_value(ticket_data))?;

    let result = engine.execute(&facts)?;
    println!(
        "   📊 Rules fired: {}, Cycles: {}",
        result.rules_fired, result.cycle_count
    );

    Ok(())
}

/// Test fraud detection AI scenario  
async fn test_fraud_detection_ai(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    // Suspicious transaction scenario
    let transaction_data = json!({
        "amount": 5000.0,
        "userId": "USER456",
        "location": "unusual_country",
        "time": "03:00:00",
        "status": "pending"
    });

    let user_history = json!({
        "avgTransaction": 150.0,
        "lastLogin": "2025-10-09",
        "riskScore": 0.3
    });

    facts.add_value("Transaction", json_to_engine_value(transaction_data))?;
    facts.add_value("UserHistory", json_to_engine_value(user_history))?;

    let result = engine.execute(&facts)?;
    println!(
        "   📊 Rules fired: {}, Cycles: {}",
        result.rules_fired, result.cycle_count
    );

    Ok(())
}

/// Test tier prediction AI scenario
async fn test_tier_prediction_ai(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    // Customer tier assignment scenario
    let customer_data = json!({
        "id": "CUSTOMER789",
        "tier": "pending",
        "totalSpent": 15000.0,
        "yearsActive": 3,
        "orderFrequency": 2.5,
        "supportTickets": 1
    });

    facts.add_value("Customer", json_to_engine_value(customer_data))?;

    let result = engine.execute(&facts)?;
    println!(
        "   📊 Rules fired: {}, Cycles: {}",
        result.rules_fired, result.cycle_count
    );

    Ok(())
}

/// Test LLM rule generation scenario
async fn test_llm_rule_generation(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let facts = Facts::new();

    // Complex business case scenario
    let business_case = json!({
        "id": "CASE001",
        "status": "review",
        "complexity": "high",
        "description": "Customer requesting refund for custom enterprise software after 6 months of usage",
        "value": 50000.0,
        "customerTier": "enterprise"
    });

    facts.add_value("BusinessCase", json_to_engine_value(business_case))?;

    let result = engine.execute(&facts)?;
    println!(
        "   📊 Rules fired: {}, Cycles: {}",
        result.rules_fired, result.cycle_count
    );

    Ok(())
}

// === AI Simulation Functions (In real implementation, these would call actual AI APIs) ===

fn simulate_sentiment_analysis(text: &str) -> SentimentResponse {
    let sentiment = if text.contains("terrible") || text.contains("awful") || text.contains("worst")
    {
        "negative"
    } else if text.contains("great") || text.contains("excellent") || text.contains("amazing") {
        "positive"
    } else {
        "neutral"
    };

    SentimentResponse {
        sentiment: sentiment.to_string(),
        confidence: 0.85,
        emotions: HashMap::from([
            (
                "anger".to_string(),
                if sentiment == "negative" { 0.8 } else { 0.1 },
            ),
            (
                "joy".to_string(),
                if sentiment == "positive" { 0.9 } else { 0.2 },
            ),
        ]),
    }
}

fn simulate_fraud_detection(
    amount: f64,
    _user_id: &str,
    _user_history: Option<&Value>,
    _location: Option<&Value>,
) -> FraudResponse {
    let is_fraud = amount > 3000.0; // Simple simulation
    let risk_score = (amount / 10000.0).min(1.0);

    FraudResponse {
        is_fraud,
        risk_score,
        reasons: if is_fraud {
            vec![
                "High transaction amount".to_string(),
                "Unusual time".to_string(),
            ]
        } else {
            vec![]
        },
    }
}

fn simulate_tier_prediction(
    _customer_id: &str,
    customer_data: Option<&Value>,
) -> TierPredictionResponse {
    let predicted_tier = if let Some(Value::Object(data)) = customer_data {
        if let Some(Value::Number(spent)) = data.get("totalSpent") {
            if *spent > 10000.0 {
                "premium"
            } else if *spent > 5000.0 {
                "gold"
            } else {
                "standard"
            }
        } else {
            "standard"
        }
    } else {
        "standard"
    };

    TierPredictionResponse {
        predicted_tier: predicted_tier.to_string(),
        confidence: 0.92,
        factors: HashMap::from([
            ("spending_history".to_string(), 0.4),
            ("engagement".to_string(), 0.3),
            ("loyalty".to_string(), 0.3),
        ]),
    }
}

fn simulate_llm_decision(context: &str, _question: &str, _facts: &Facts) -> String {
    // Simulate LLM reasoning
    if context.contains("refund") && context.contains("enterprise") {
        "APPROVE - Enterprise customer, goodwill gesture recommended"
    } else if context.contains("refund") && context.contains("6 months") {
        "REVIEW - Long usage period, needs manual review"
    } else {
        "STANDARD_PROCESS - Follow normal procedures"
    }
    .to_string()
}

fn simulate_ml_scoring(features: &[Value]) -> f64 {
    // Simple ML scoring simulation
    let mut score = 0.5;
    for feature in features {
        match feature {
            Value::Number(n) => score += (n / 1000.0).min(0.3),
            Value::Boolean(true) => score += 0.2,
            _ => {}
        }
    }
    score.min(1.0)
}

fn json_to_engine_value(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else {
                Value::Number(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, val) in obj {
                map.insert(key, json_to_engine_value(val));
            }
            Value::Object(map)
        }
        serde_json::Value::Array(arr) => {
            let values: Vec<Value> = arr.into_iter().map(json_to_engine_value).collect();
            Value::Array(values)
        }
        serde_json::Value::Null => Value::String("null".to_string()),
    }
}

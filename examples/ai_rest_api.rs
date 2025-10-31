/// AI-Enhanced REST API for Rule Engine
///
/// This API demonstrates how to integrate AI models with the Rust Rule Engine
/// in a production environment, similar to Drools Pragmatic AI approach.
use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
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
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

/// AI-enhanced application state
#[derive(Clone)]
pub struct AIAppState {
    pub engine: Arc<Mutex<RustRuleEngine>>,
    pub ai_models: Arc<AIModelRegistry>,
}

/// Registry for AI models and their endpoints
pub struct AIModelRegistry {
    pub sentiment_api: String,
    pub fraud_api: String,
    pub llm_api: String,
}

/// Request for AI-powered rule execution
#[derive(Debug, Deserialize)]
pub struct AIRuleRequest {
    pub facts: HashMap<String, serde_json::Value>,
    pub ai_features: Option<AIFeatures>,
    pub rules: Option<Vec<String>>,
}

/// AI feature flags and configurations
#[derive(Debug, Deserialize)]
pub struct AIFeatures {
    pub enable_sentiment: Option<bool>,
    pub enable_fraud_detection: Option<bool>,
    pub enable_llm_reasoning: Option<bool>,
    pub confidence_threshold: Option<f64>,
}

/// AI-enhanced response
#[derive(Debug, Serialize)]
pub struct AIRuleResponse {
    pub success: bool,
    pub rules_fired: usize,
    pub cycle_count: usize,
    pub execution_time_ms: f64,
    pub ai_insights: AIInsights,
    pub updated_facts: HashMap<String, serde_json::Value>,
    pub message: String,
}

/// AI insights from rule execution
#[derive(Debug, Serialize)]
pub struct AIInsights {
    pub models_used: Vec<String>,
    pub confidence_scores: HashMap<String, f64>,
    pub ai_decisions: Vec<AIDecision>,
    pub recommendations: Vec<String>,
}

/// Individual AI decision made during rule execution
#[derive(Debug, Serialize)]
pub struct AIDecision {
    pub model: String,
    pub input: String,
    pub output: String,
    pub confidence: f64,
    pub rule_triggered: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Starting AI-Enhanced Rule Engine REST API");

    // Create AI-enhanced engine
    let engine = create_ai_production_engine().await?;

    // Setup AI model registry
    let ai_models = Arc::new(AIModelRegistry {
        sentiment_api: "https://api.openai.com/v1/chat/completions".to_string(),
        fraud_api: "https://ml-api.company.com/fraud-detection".to_string(),
        llm_api: "https://api.anthropic.com/v1/messages".to_string(),
    });

    let app_state = AIAppState {
        engine: Arc::new(Mutex::new(engine)),
        ai_models,
    };

    // Build router with AI-enhanced endpoints
    let app = Router::new()
        .route("/api/v1/ai/execute", post(execute_ai_rules))
        .route("/api/v1/ai/sentiment", post(analyze_sentiment))
        .route("/api/v1/ai/fraud", post(detect_fraud))
        .route("/api/v1/ai/llm", post(llm_reasoning))
        .route("/api/v1/ai/models", get(list_ai_models))
        .route("/", get(ai_api_docs))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .into_inner(),
        )
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:3001").await?;
    println!("🌐 AI-Enhanced API running on http://localhost:3001");
    println!("🤖 AI Models Dashboard: http://localhost:3001/api/v1/ai/models");
    println!("📚 AI API Docs: http://localhost:3001");

    axum::serve(listener, app).await?;
    Ok(())
}

/// Create production AI-enhanced rule engine
async fn create_ai_production_engine() -> Result<RustRuleEngine, Box<dyn std::error::Error>> {
    let config = EngineConfig {
        max_cycles: 5,
        timeout: Some(Duration::from_secs(30)),
        enable_stats: true,
        debug_mode: true,
    };

    let mut engine = RustRuleEngine::with_config(KnowledgeBase::new("AI-Production"), config);

    // Register AI functions for production use
    register_production_ai_functions(&mut engine).await?;
    load_production_ai_rules(&mut engine).await?;

    Ok(engine)
}

/// Register production-ready AI functions
async fn register_production_ai_functions(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    // Thread-safe AI insight storage
    let ai_insights = Arc::new(Mutex::new(Vec::<AIDecision>::new()));

    // Sentiment Analysis with confidence tracking
    let insights_clone = ai_insights.clone();
    engine.register_function("aiSentiment", move |args, _facts| {
        let text = args[0].as_string().unwrap_or("".to_string());

        // In production: call actual AI API
        let (sentiment, confidence) = call_sentiment_api(&text);

        // Track AI decision
        let decision = AIDecision {
            model: "sentiment-analyzer-v2".to_string(),
            input: text.clone(),
            output: sentiment.clone(),
            confidence,
            rule_triggered: "sentiment_rule".to_string(),
        };

        if let Ok(mut insights) = insights_clone.lock() {
            insights.push(decision);
        }

        println!(
            "🤖 AI Sentiment: '{}' -> {} ({:.1}%)",
            text.chars().take(50).collect::<String>(),
            sentiment,
            confidence * 100.0
        );

        Ok(Value::String(sentiment))
    });

    // Fraud Detection with ML model
    let insights_clone = ai_insights.clone();
    engine.register_function("aiFraud", move |args, facts| {
        let amount = args[0].as_number().unwrap_or(0.0);
        let user_id = args[1].as_string().unwrap_or("unknown".to_string());

        // Extract features from facts
        let features = extract_fraud_features(facts);

        // In production: call ML fraud detection API
        let (is_fraud, risk_score) = call_fraud_api(amount, &user_id, &features);

        // Track AI decision
        let decision = AIDecision {
            model: "fraud-detector-xgb-v3".to_string(),
            input: format!("amount:{}, user:{}", amount, user_id),
            output: if is_fraud {
                "FRAUD".to_string()
            } else {
                "SAFE".to_string()
            },
            confidence: risk_score,
            rule_triggered: "fraud_detection_rule".to_string(),
        };

        if let Ok(mut insights) = insights_clone.lock() {
            insights.push(decision);
        }

        println!(
            "🛡️ AI Fraud: ${:.2} -> {} (risk: {:.1}%)",
            amount,
            if is_fraud { "FRAUD" } else { "SAFE" },
            risk_score * 100.0
        );

        Ok(Value::Boolean(is_fraud))
    });

    // LLM Reasoning for complex decisions
    let insights_clone = ai_insights.clone();
    engine.register_function("aiLLM", move |args, facts| {
        let prompt = args[0].as_string().unwrap_or("".to_string());
        let context = build_context_from_facts(facts);

        // In production: call LLM API (OpenAI, Anthropic, etc.)
        let (decision, confidence) = call_llm_api(&prompt, &context);

        // Track AI decision
        let ai_decision = AIDecision {
            model: "gpt-4-turbo".to_string(),
            input: prompt.clone(),
            output: decision.clone(),
            confidence,
            rule_triggered: "llm_reasoning_rule".to_string(),
        };

        if let Ok(mut insights) = insights_clone.lock() {
            insights.push(ai_decision);
        }

        println!(
            "🧠 AI LLM: '{}' -> {} ({:.1}%)",
            prompt.chars().take(50).collect::<String>(),
            decision,
            confidence * 100.0
        );

        Ok(Value::String(decision))
    });

    // Real-time ML scoring for dynamic rules
    engine.register_function("aiScore", |args, _facts| {
        let features: Vec<f64> = args.iter().filter_map(|v| v.as_number()).collect();

        // In production: call real-time ML inference endpoint
        let score = call_ml_scoring_api(&features);

        println!("📊 AI Score: features={:?} -> {:.3}", features, score);

        Ok(Value::Number(score))
    });

    println!("✅ Registered production AI functions");
    Ok(())
}

/// Load production AI-enhanced rules
async fn load_production_ai_rules(
    engine: &mut RustRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    let ai_rules = vec![
        // Customer service automation with AI - analyze sentiment in action
        r#"
rule "Analyze Customer Ticket" salience 100 {
    when
        Support.ticket.type == "complaint"
    then
        set(Support.ticket.sentiment, aiSentiment(Support.ticket.message));
        set(Support.ticket.aiProcessed, true);
        Log("Analyzed ticket sentiment with AI");
}
        "#,
        r#"
rule "High Priority Negative Sentiment" salience 95 {
    when
        Support.ticket.sentiment == "negative"
    then
        set(Support.ticket.priority, "urgent");
        Log("Escalated negative sentiment ticket");
}
        "#,
        // Smart fraud prevention - detect fraud in action
        r#"
rule "Fraud Risk Assessment" salience 90 {
    when
        Payment.amount > 500
    then
        set(Payment.isFraud, aiFraud(Payment.amount, Payment.userId));
        Log("AI fraud detection completed");
}
        "#,
        r#"
rule "Block Fraudulent Payment" salience 85 {
    when
        Payment.isFraud == true
    then
        set(Payment.status, "blocked");
        set(Payment.reviewRequired, true);
        Log("Payment blocked due to fraud detection");
}
        "#,
        // Business decisions with LLM
        r#"
rule "Complex Case AI Analysis" salience 80 {
    when
        Case.complexity == "high" &&
        Case.status == "pending"
    then
        set(Case.aiRecommendation, aiLLM("Should this case be approved or rejected?"));
        set(Case.status, "ai_reviewed");
        Log("LLM provided business case recommendation");
}
        "#,
        // Dynamic pricing with ML scoring
        r#"
rule "Calculate Price Score" salience 75 {
    when
        Product.isPremium == true
    then
        set(Product.mlScore, aiScore(Market.demand, Product.inventory, Customer.tier));
        Log("ML price scoring completed");
}
        "#,
        r#"
rule "Premium Pricing" salience 70 {
    when
        Product.mlScore > 0.8
    then
        set(Product.pricingStrategy, "PREMIUM");
        set(Product.dynamicPrice, Product.basePrice * 1.15);
        Log("Premium pricing strategy applied");
}
        "#,
        // Content moderation
        r#"
rule "Analyze Content" salience 65 {
    when
        Content.type == "user_post"
    then
        set(Content.sentiment, aiSentiment(Content.text));
        Log("Content sentiment analyzed");
}
        "#,
        r#"
rule "Moderate Toxic Content" salience 60 {
    when
        Content.sentiment == "toxic"
    then
        set(Content.status, "moderated");
        set(Content.aiModerated, true);
        Log("Toxic content moderated by AI");
}
        "#,
    ];

    for rule_grl in &ai_rules {
        let rules = GRLParser::parse_rules(rule_grl)?;
        for rule in rules {
            engine.knowledge_base_mut().add_rule(rule)?;
        }
    }

    println!("✅ Loaded {} production AI rules", ai_rules.len());
    println!("💡 AI functions (aiSentiment, aiFraud, aiLLM, aiScore) are called in rule actions");
    println!("💡 Rules are chained: first rule calls AI, second rule uses AI result");
    Ok(())
}

/// Execute AI-enhanced rules
async fn execute_ai_rules(
    State(state): State<AIAppState>,
    Json(request): Json<AIRuleRequest>,
) -> Result<ResponseJson<AIRuleResponse>, StatusCode> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Setup facts
    let facts = Facts::new();
    for (key, value) in request.facts {
        let engine_value = json_to_engine_value(value);
        facts
            .add_value(&key, engine_value)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
    }

    // Execute rules with AI
    let start_time = std::time::Instant::now();
    let result = engine
        .execute(&facts)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let execution_time = start_time.elapsed().as_secs_f64() * 1000.0;

    // Collect AI insights (in production, extract from engine state)
    let ai_insights = AIInsights {
        models_used: vec![
            "sentiment-analyzer-v2".to_string(),
            "fraud-detector-xgb-v3".to_string(),
        ],
        confidence_scores: HashMap::from([
            ("sentiment".to_string(), 0.87),
            ("fraud_detection".to_string(), 0.92),
        ]),
        ai_decisions: vec![], // Would be populated from actual execution
        recommendations: vec![
            "Consider implementing ML-based risk scoring".to_string(),
            "AI detected patterns suggest reviewing fraud rules".to_string(),
        ],
    };

    let response = AIRuleResponse {
        success: true,
        rules_fired: result.rules_fired,
        cycle_count: result.cycle_count,
        execution_time_ms: execution_time,
        ai_insights,
        updated_facts: HashMap::new(), // Would extract from facts
        message: format!(
            "AI-enhanced execution: {} rules fired with ML insights",
            result.rules_fired
        ),
    };

    Ok(ResponseJson(response))
}

/// Standalone sentiment analysis endpoint
async fn analyze_sentiment(
    State(_state): State<AIAppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let text = request["text"].as_str().unwrap_or("");
    let (sentiment, confidence) = call_sentiment_api(text);

    Ok(ResponseJson(json!({
        "sentiment": sentiment,
        "confidence": confidence,
        "model": "sentiment-analyzer-v2",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Standalone fraud detection endpoint
async fn detect_fraud(
    State(_state): State<AIAppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let amount = request["amount"].as_f64().unwrap_or(0.0);
    let user_id = request["userId"].as_str().unwrap_or("unknown");

    let (is_fraud, risk_score) = call_fraud_api(amount, user_id, &HashMap::new());

    Ok(ResponseJson(json!({
        "is_fraud": is_fraud,
        "risk_score": risk_score,
        "model": "fraud-detector-xgb-v3",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Standalone LLM reasoning endpoint
async fn llm_reasoning(
    State(_state): State<AIAppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<ResponseJson<serde_json::Value>, StatusCode> {
    let prompt = request["prompt"].as_str().unwrap_or("");
    let context = request["context"].as_str().unwrap_or("");

    let (decision, confidence) = call_llm_api(prompt, context);

    Ok(ResponseJson(json!({
        "decision": decision,
        "confidence": confidence,
        "model": "gpt-4-turbo",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// List available AI models
async fn list_ai_models(State(state): State<AIAppState>) -> ResponseJson<serde_json::Value> {
    ResponseJson(json!({
        "ai_models": {
            "sentiment_analysis": {
                "model": "sentiment-analyzer-v2",
                "endpoint": state.ai_models.sentiment_api,
                "capabilities": ["emotion_detection", "toxicity_detection", "intent_classification"],
                "languages": ["en", "es", "fr", "de"],
                "latency_ms": 150
            },
            "fraud_detection": {
                "model": "fraud-detector-xgb-v3",
                "endpoint": state.ai_models.fraud_api,
                "capabilities": ["real_time_scoring", "risk_assessment", "pattern_detection"],
                "accuracy": 0.94,
                "latency_ms": 50
            },
            "llm_reasoning": {
                "model": "gpt-4-turbo",
                "endpoint": state.ai_models.llm_api,
                "capabilities": ["reasoning", "decision_support", "text_generation"],
                "context_length": 128000,
                "latency_ms": 2000
            }
        },
        "integration_status": "active",
        "total_models": 3
    }))
}

/// AI API documentation
async fn ai_api_docs() -> ResponseJson<serde_json::Value> {
    ResponseJson(json!({
        "name": "AI-Enhanced Rust Rule Engine API",
        "version": "1.0.0",
        "description": "Production-ready rule engine with integrated AI/ML capabilities",
        "ai_features": [
            "Real-time sentiment analysis",
            "ML-powered fraud detection",
            "LLM reasoning and decision support",
            "Dynamic ML scoring",
            "AI-generated recommendations"
        ],
        "endpoints": {
            "ai_execution": {
                "POST /api/v1/ai/execute": "Execute rules with AI enhancements",
                "POST /api/v1/ai/sentiment": "Analyze text sentiment",
                "POST /api/v1/ai/fraud": "Detect fraudulent activity",
                "POST /api/v1/ai/llm": "LLM-powered reasoning"
            },
            "ai_management": {
                "GET /api/v1/ai/models": "List available AI models"
            }
        },
        "example_ai_rule": {
            "grl": "rule \"AI Decision\" { when aiSentiment(text) == \"negative\" then set(priority, \"high\"); }",
            "description": "Uses AI sentiment analysis in rule condition"
        },
        "ai_integration": "Seamlessly integrate OpenAI, Anthropic, Hugging Face, and custom ML models"
    }))
}

// === AI API Simulation Functions (Replace with actual API calls in production) ===

fn call_sentiment_api(text: &str) -> (String, f64) {
    // In production: HTTP request to sentiment analysis API
    let sentiment = if text.contains("terrible") || text.contains("awful") {
        "negative"
    } else if text.contains("great") || text.contains("excellent") {
        "positive"
    } else {
        "neutral"
    };

    (sentiment.to_string(), 0.87)
}

fn call_fraud_api(amount: f64, _user_id: &str, _features: &HashMap<String, f64>) -> (bool, f64) {
    // In production: HTTP request to ML fraud detection API
    let is_fraud = amount > 2000.0;
    let risk_score = (amount / 5000.0).min(1.0);

    (is_fraud, risk_score)
}

fn call_llm_api(prompt: &str, _context: &str) -> (String, f64) {
    // In production: HTTP request to LLM API (OpenAI, Anthropic, etc.)
    let decision = if prompt.contains("approve") {
        "APPROVED"
    } else if prompt.contains("reject") {
        "REJECTED"
    } else {
        "REQUIRES_REVIEW"
    };

    (decision.to_string(), 0.91)
}

fn call_ml_scoring_api(features: &[f64]) -> f64 {
    // In production: HTTP request to ML scoring endpoint
    features.iter().sum::<f64>() / features.len().max(1) as f64 / 100.0
}

fn extract_fraud_features(_facts: &Facts) -> HashMap<String, f64> {
    // Extract ML features from facts for fraud detection
    HashMap::new()
}

fn build_context_from_facts(_facts: &Facts) -> String {
    // Build context string from facts for LLM
    "Business context from current facts".to_string()
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

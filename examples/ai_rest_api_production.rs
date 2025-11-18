use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use reqwest::Client;
use rust_rule_engine::{Facts, RuleEngineBuilder, Value};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct AIAPIConfig {
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub hf_api_key: String,
    pub enable_caching: bool,
    pub cache_ttl_seconds: u64,
    pub max_retries: u32,
    pub request_timeout_seconds: u64,
}

impl Default for AIAPIConfig {
    fn default() -> Self {
        Self {
            openai_api_key: std::env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| "demo-key".to_string()),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY")
                .unwrap_or_else(|_| "demo-key".to_string()),
            hf_api_key: std::env::var("HF_API_KEY").unwrap_or_else(|_| "demo-key".to_string()),
            enable_caching: true,
            cache_ttl_seconds: 300, // 5 minutes
            max_retries: 3,
            request_timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CachedResponse {
    pub response: String,
    pub timestamp: Instant,
    pub provider: String,
    pub cost_estimate: f64,
}

#[derive(Debug)]
pub struct AIServiceManager {
    client: Client,
    config: AIAPIConfig,
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    stats: Arc<RwLock<AIServiceStats>>,
}

#[derive(Debug, Default)]
pub struct AIServiceStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub openai_requests: u64,
    pub anthropic_requests: u64,
    pub hf_requests: u64,
    pub total_errors: u64,
    pub total_cost_estimate: f64,
}

#[derive(Deserialize)]
pub struct ExecuteRulesRequest {
    pub facts: HashMap<String, serde_json::Value>,
    pub ai_providers: Option<Vec<String>>, // ["openai", "anthropic", "huggingface"]
    pub enable_caching: Option<bool>,
    pub max_cost: Option<f64>, // Cost limit in USD
}

#[derive(Serialize)]
pub struct ExecuteRulesResponse {
    pub success: bool,
    pub rules_fired: usize,
    pub cycles: usize,
    pub duration_ms: f64,
    pub ai_results: HashMap<String, serde_json::Value>,
    pub cost_estimate: f64,
    pub cache_info: CacheInfo,
    pub errors: Vec<String>,
}

#[derive(Serialize)]
pub struct CacheInfo {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_hit_rate: f64,
}

#[derive(Serialize)]
pub struct AIStatsResponse {
    pub total_requests: u64,
    pub cache_performance: CacheInfo,
    pub provider_usage: HashMap<String, u64>,
    pub error_rate: f64,
    pub total_cost_estimate: f64,
    pub uptime_seconds: u64,
}

impl AIServiceManager {
    pub fn new(config: AIAPIConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(AIServiceStats::default())),
        }
    }

    async fn get_cached_response(&self, key: &str) -> Option<CachedResponse> {
        if !self.config.enable_caching {
            return None;
        }

        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(key) {
            if cached.timestamp.elapsed().as_secs() < self.config.cache_ttl_seconds {
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
                return Some(cached.clone());
            }
        }

        let mut stats = self.stats.write().await;
        stats.cache_misses += 1;
        None
    }

    async fn cache_response(&self, key: String, response: String, provider: String, cost: f64) {
        if !self.config.enable_caching {
            return;
        }

        let mut cache = self.cache.write().await;
        cache.insert(
            key,
            CachedResponse {
                response,
                timestamp: Instant::now(),
                provider,
                cost_estimate: cost,
            },
        );

        // Cleanup expired entries
        let ttl = Duration::from_secs(self.config.cache_ttl_seconds);
        cache.retain(|_, cached| cached.timestamp.elapsed() < ttl);
    }

    pub async fn analyze_sentiment_openai(
        &self,
        text: &str,
    ) -> Result<(String, f64), Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("openai_sentiment:{}", text);

        if let Some(cached) = self.get_cached_response(&cache_key).await {
            info!("Cache hit for OpenAI sentiment analysis");
            return Ok((cached.response, cached.cost_estimate));
        }

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.openai_requests += 1;
        drop(stats);

        for attempt in 1..=self.config.max_retries {
            match self.call_openai_sentiment_api(text).await {
                Ok(sentiment) => {
                    let cost = 0.002; // Estimated cost per request
                    self.cache_response(cache_key, sentiment.clone(), "openai".to_string(), cost)
                        .await;

                    let mut stats = self.stats.write().await;
                    stats.total_cost_estimate += cost;

                    return Ok((sentiment, cost));
                }
                Err(e) => {
                    warn!(
                        "OpenAI attempt {}/{} failed: {}",
                        attempt, self.config.max_retries, e
                    );
                    if attempt == self.config.max_retries {
                        let mut stats = self.stats.write().await;
                        stats.total_errors += 1;
                        return Err(e);
                    }
                    tokio::time::sleep(Duration::from_millis(1000 * attempt as u64)).await;
                }
            }
        }

        unreachable!()
    }

    async fn call_openai_sentiment_api(
        &self,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a sentiment analysis expert. Respond with exactly one word: positive, negative, or neutral"
                },
                {
                    "role": "user",
                    "content": format!("Analyze sentiment: {}", text)
                }
            ],
            "max_tokens": 5,
            "temperature": 0.0
        });

        let response = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header(
                "Authorization",
                format!("Bearer {}", self.config.openai_api_key),
            )
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status_code = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI API error {}: {}", status_code, error_text).into());
        }

        let response_json: serde_json::Value = response.json().await?;

        let sentiment = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("neutral")
            .trim()
            .to_lowercase();

        Ok(sentiment)
    }

    pub async fn analyze_sentiment_huggingface(
        &self,
        text: &str,
    ) -> Result<(String, f64), Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("hf_sentiment:{}", text);

        if let Some(cached) = self.get_cached_response(&cache_key).await {
            info!("Cache hit for Hugging Face sentiment analysis");
            return Ok((cached.response, cached.cost_estimate));
        }

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.hf_requests += 1;
        drop(stats);

        let request_body = json!({
            "inputs": text
        });

        let response = self.client
            .post("https://api-inference.huggingface.co/models/cardiffnlp/twitter-roberta-base-sentiment-latest")
            .header("Authorization", format!("Bearer {}", self.config.hf_api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let mut stats = self.stats.write().await;
            stats.total_errors += 1;
            let status_code = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Hugging Face API error {}: {}", status_code, error_text).into());
        }

        let response_json: serde_json::Value = response.json().await?;

        let sentiment = self.parse_hf_sentiment_response(&response_json)?;
        let cost = 0.001; // Estimated cost per request

        self.cache_response(
            cache_key,
            sentiment.clone(),
            "huggingface".to_string(),
            cost,
        )
        .await;

        let mut stats = self.stats.write().await;
        stats.total_cost_estimate += cost;

        Ok((sentiment, cost))
    }

    fn parse_hf_sentiment_response(
        &self,
        response: &serde_json::Value,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(predictions) = response.as_array() {
            if let Some(first_prediction) = predictions.first() {
                if let Some(predictions_array) = first_prediction.as_array() {
                    let mut best_sentiment = "neutral";
                    let mut best_score = 0.0;

                    for prediction in predictions_array {
                        if let (Some(label), Some(score)) =
                            (prediction["label"].as_str(), prediction["score"].as_f64())
                        {
                            if score > best_score {
                                best_score = score;
                                best_sentiment = match label.to_uppercase().as_str() {
                                    "LABEL_0" | "NEGATIVE" => "negative",
                                    "LABEL_1" | "NEUTRAL" => "neutral",
                                    "LABEL_2" | "POSITIVE" => "positive",
                                    _ => "neutral",
                                };
                            }
                        }
                    }

                    return Ok(best_sentiment.to_string());
                }
            }
        }

        Ok("neutral".to_string())
    }

    pub async fn ask_claude_decision(
        &self,
        question: &str,
        context: &str,
    ) -> Result<(String, f64), Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("claude_decision:{}:{}", question, context);

        if let Some(cached) = self.get_cached_response(&cache_key).await {
            info!("Cache hit for Claude decision");
            return Ok((cached.response, cached.cost_estimate));
        }

        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        stats.anthropic_requests += 1;
        drop(stats);

        let request_body = json!({
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 50,
            "messages": [
                {
                    "role": "user",
                    "content": format!("Business Decision: {}\nContext: {}\nRespond with exactly one word: approve, deny, or review", question, context)
                }
            ]
        });

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.config.anthropic_api_key)
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let mut stats = self.stats.write().await;
            stats.total_errors += 1;
            let status_code = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Anthropic API error {}: {}", status_code, error_text).into());
        }

        let response_json: serde_json::Value = response.json().await?;

        let decision_text = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("review")
            .to_lowercase();

        let decision = if decision_text.contains("approve") {
            "approve".to_string()
        } else if decision_text.contains("deny") {
            "deny".to_string()
        } else {
            "review".to_string()
        };

        let cost = 0.015; // Estimated cost per request
        self.cache_response(cache_key, decision.clone(), "anthropic".to_string(), cost)
            .await;

        let mut stats = self.stats.write().await;
        stats.total_cost_estimate += cost;

        Ok((decision, cost))
    }

    pub async fn get_stats(&self) -> AIStatsResponse {
        let stats = self.stats.read().await;
        let cache_hits = stats.cache_hits;
        let cache_misses = stats.cache_misses;
        let total_cache_requests = cache_hits + cache_misses;

        let cache_hit_rate = if total_cache_requests > 0 {
            cache_hits as f64 / total_cache_requests as f64
        } else {
            0.0
        };

        let error_rate = if stats.total_requests > 0 {
            stats.total_errors as f64 / stats.total_requests as f64
        } else {
            0.0
        };

        let mut provider_usage = HashMap::new();
        provider_usage.insert("openai".to_string(), stats.openai_requests);
        provider_usage.insert("anthropic".to_string(), stats.anthropic_requests);
        provider_usage.insert("huggingface".to_string(), stats.hf_requests);

        AIStatsResponse {
            total_requests: stats.total_requests,
            cache_performance: CacheInfo {
                cache_hits,
                cache_misses,
                cache_hit_rate,
            },
            provider_usage,
            error_rate,
            total_cost_estimate: stats.total_cost_estimate,
            uptime_seconds: 0, // You'd track this with application start time
        }
    }
}

pub type AppState = Arc<AIServiceManager>;

async fn execute_ai_rules(
    State(ai_service): State<AppState>,
    Json(request): Json<ExecuteRulesRequest>,
) -> Result<Json<ExecuteRulesResponse>, StatusCode> {
    let start_time = Instant::now();
    let mut total_cost = 0.0;
    let mut ai_results = HashMap::new();
    let mut errors = Vec::new();

    // Create AI-powered rules
    let ai_rules = r#"
        rule "AI Sentiment Analysis" salience 100 {
            when
                CustomerMessage.text != ""
            then
                analyzeAISentiment(CustomerMessage.text, CustomerMessage.provider);
                set(Analysis.ai_complete, true);
                logMessage("🤖 AI sentiment analysis complete");
        }

        rule "AI Business Decision" salience 90 {
            when
                Decision.question != "" && Decision.context != ""
            then
                makeAIDecision(Decision.question, Decision.context);
                set(Decision.ai_complete, true);
                logMessage("🧠 AI business decision complete");
        }
    "#;

    let mut engine = RuleEngineBuilder::new()
        .with_inline_grl(ai_rules)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .build();

    // Register AI sentiment analysis function
    let ai_service_clone = ai_service.clone();
    engine.register_function("analyzeAISentiment", move |args, facts| {
        let text = args[0].as_string().unwrap_or("".to_string());
        let provider = args[1].as_string().unwrap_or("openai".to_string());
        let service = ai_service_clone.clone();

        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = match provider.as_str() {
            "openai" => rt.block_on(service.analyze_sentiment_openai(&text)),
            "huggingface" => rt.block_on(service.analyze_sentiment_huggingface(&text)),
            _ => rt.block_on(service.analyze_sentiment_openai(&text)), // Default to OpenAI
        };

        match result {
            Ok((sentiment, cost)) => {
                facts.add_value("AIResult.sentiment", Value::String(sentiment.clone()))?;
                facts.add_value("AIResult.cost", Value::Number(cost))?;
                facts.add_value("AIResult.provider", Value::String(provider))?;
                Ok(Value::String(sentiment))
            }
            Err(e) => {
                eprintln!("AI sentiment analysis error: {}", e);
                Ok(Value::String("error".to_string()))
            }
        }
    });

    // Register AI decision function
    let ai_service_clone2 = ai_service.clone();
    engine.register_function("makeAIDecision", move |args, facts| {
        let question = args[0].as_string().unwrap_or("".to_string());
        let context = args[1].as_string().unwrap_or("".to_string());
        let service = ai_service_clone2.clone();

        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(service.ask_claude_decision(&question, &context)) {
            Ok((decision, cost)) => {
                facts.add_value("AIResult.decision", Value::String(decision.clone()))?;
                facts.add_value("AIResult.decision_cost", Value::Number(cost))?;
                Ok(Value::String(decision))
            }
            Err(e) => {
                eprintln!("AI decision making error: {}", e);
                Ok(Value::String("error".to_string()))
            }
        }
    });

    // Helper functions
    engine.register_function("set", |args, facts| {
        if args.len() >= 2 {
            let key = args[0].as_string().unwrap_or("unknown".to_string());
            facts.add_value(&key, args[1].clone())?;
        }
        Ok(Value::Boolean(true))
    });

    engine.register_function("logMessage", |args, _| {
        info!("Rule Engine: {:?}", args[0]);
        Ok(Value::Boolean(true))
    });

    // Convert request facts to engine facts
    let facts = Facts::new();
    for (key, value) in request.facts {
        let engine_value = convert_json_to_value(value);
        facts
            .add_value(&key, engine_value)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
    }

    // Execute rules
    let result = engine
        .execute(&facts)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract AI results from facts
    if let Some(ai_result_value) = facts.get("AIResult") {
        if let Value::Object(ai_result_map) = ai_result_value {
            for (key, value) in ai_result_map {
                // Add up costs first before moving the value
                if key == "cost" || key == "decision_cost" {
                    if let Some(cost) = value.as_number() {
                        total_cost += cost;
                    }
                }

                ai_results.insert(key, convert_value_to_json(value));
            }
        }
    }

    let stats = ai_service.get_stats().await;

    let response = ExecuteRulesResponse {
        success: true,
        rules_fired: result.rules_fired,
        cycles: result.cycle_count,
        duration_ms: start_time.elapsed().as_secs_f64() * 1000.0,
        ai_results,
        cost_estimate: total_cost,
        cache_info: stats.cache_performance,
        errors,
    };

    Ok(Json(response))
}

async fn get_ai_stats(State(ai_service): State<AppState>) -> Json<AIStatsResponse> {
    let stats = ai_service.get_stats().await;
    Json(stats)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "AI-Powered Rule Engine",
        "version": "0.5.0",
        "features": ["openai", "anthropic", "huggingface", "caching", "error_handling"],
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn api_documentation() -> Json<serde_json::Value> {
    Json(json!({
        "name": "AI-Powered Rule Engine API",
        "version": "0.5.0",
        "description": "Execute business rules with real AI integration",
        "endpoints": {
            "POST /api/v1/rules/execute": {
                "description": "Execute rules with AI-powered functions",
                "body": {
                    "facts": "Object containing rule facts",
                    "ai_providers": "Optional array of AI providers ['openai', 'anthropic', 'huggingface']",
                    "enable_caching": "Optional boolean to enable/disable caching",
                    "max_cost": "Optional maximum cost limit in USD"
                }
            },
            "GET /api/v1/ai/stats": {
                "description": "Get AI service statistics and performance metrics"
            },
            "GET /api/v1/health": {
                "description": "Health check endpoint"
            }
        },
        "features": [
            "Real OpenAI GPT integration",
            "Anthropic Claude decision making",
            "Hugging Face sentiment analysis",
            "Intelligent response caching",
            "Automatic retry with exponential backoff",
            "Cost tracking and limits",
            "Comprehensive error handling",
            "Production-ready monitoring"
        ]
    }))
}

fn convert_json_to_value(json_value: serde_json::Value) -> Value {
    match json_value {
        serde_json::Value::Null => Value::String("null".to_string()),
        serde_json::Value::Bool(b) => Value::Boolean(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Number(f)
            } else {
                Value::Number(0.0)
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            let mut vec = Vec::new();
            for item in arr {
                vec.push(convert_json_to_value(item));
            }
            Value::Array(vec)
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (key, value) in obj {
                map.insert(key, convert_json_to_value(value));
            }
            Value::Object(map)
        }
    }
}

fn convert_value_to_json(value: Value) -> serde_json::Value {
    match value {
        Value::Integer(i) => json!(i),
        Value::Number(f) => json!(f),
        Value::String(s) => json!(s),
        Value::Boolean(b) => json!(b),
        Value::Array(arr) => {
            let json_arr: Vec<serde_json::Value> =
                arr.into_iter().map(convert_value_to_json).collect();
            json!(json_arr)
        }
        Value::Object(obj) => {
            let json_obj: serde_json::Map<String, serde_json::Value> = obj
                .into_iter()
                .map(|(k, v)| (k, convert_value_to_json(v)))
                .collect();
            json!(json_obj)
        }
        Value::Null => json!(null),
        Value::Expression(expr) => json!(expr), // Convert expression to string
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Starting AI-Powered Rule Engine REST API...");

    // Load AI service configuration
    let ai_config = AIAPIConfig::default();
    let ai_service = Arc::new(AIServiceManager::new(ai_config));

    // Build the application router
    let app = Router::new()
        .route("/", get(api_documentation))
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/rules/execute", post(execute_ai_rules))
        .route("/api/v1/ai/stats", get(get_ai_stats))
        .layer(CorsLayer::permissive())
        .with_state(ai_service);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("🌐 AI-Powered Rule Engine API running on http://0.0.0.0:3000");
    println!("📚 API Documentation: http://localhost:3000");
    println!("🏥 Health Check: http://localhost:3000/api/v1/health");
    println!("📊 AI Statistics: http://localhost:3000/api/v1/ai/stats");

    println!("\n🔧 Setup Instructions:");
    println!("   1. Set environment variables:");
    println!("      export OPENAI_API_KEY=your-openai-key");
    println!("      export ANTHROPIC_API_KEY=your-anthropic-key");
    println!("      export HF_API_KEY=your-huggingface-key");
    println!("   2. Test with curl:");
    println!("      curl -X POST http://localhost:3000/api/v1/rules/execute \\");
    println!("        -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"facts\": {{\"CustomerMessage\": {{\"text\": \"Great service!\", \"provider\": \"openai\"}}}}}}'");

    axum::serve(listener, app).await?;

    Ok(())
}

use reqwest::Client;
use rust_rule_engine::{Facts, RuleEngineBuilder, Value};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio;

#[derive(Debug, Clone)]
pub struct AIServiceConfig {
    pub openai_api_key: String,
    pub anthropic_api_key: String,
    pub hf_api_key: String,
    pub max_retries: u32,
    pub request_timeout: Duration,
    pub enable_caching: bool,
    pub cache_ttl: Duration,
    pub enable_fallback: bool,
}

impl Default for AIServiceConfig {
    fn default() -> Self {
        Self {
            openai_api_key: std::env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| "demo-key".to_string()),
            anthropic_api_key: std::env::var("ANTHROPIC_API_KEY")
                .unwrap_or_else(|_| "demo-key".to_string()),
            hf_api_key: std::env::var("HF_API_KEY").unwrap_or_else(|_| "demo-key".to_string()),
            max_retries: 3,
            request_timeout: Duration::from_secs(30),
            enable_caching: true,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            enable_fallback: true,
        }
    }
}

pub struct AIService {
    client: Client,
    config: AIServiceConfig,
    cache: Arc<tokio::sync::RwLock<HashMap<String, (String, std::time::Instant)>>>,
}

impl AIService {
    pub fn new(config: AIServiceConfig) -> Self {
        let client = Client::builder()
            .timeout(config.request_timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    async fn get_cached_response(&self, key: &str) -> Option<String> {
        if !self.config.enable_caching {
            return None;
        }

        let cache = self.cache.read().await;
        if let Some((response, timestamp)) = cache.get(key) {
            if timestamp.elapsed() < self.config.cache_ttl {
                return Some(response.clone());
            }
        }
        None
    }

    async fn cache_response(&self, key: String, response: String) {
        if !self.config.enable_caching {
            return;
        }

        let mut cache = self.cache.write().await;
        cache.insert(key, (response, std::time::Instant::now()));

        // Simple cleanup: remove expired entries
        cache.retain(|_, (_, timestamp)| timestamp.elapsed() < self.config.cache_ttl);
    }

    pub async fn analyze_sentiment_openai(
        &self,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("openai_sentiment:{}", text);

        // Check cache first
        if let Some(cached_response) = self.get_cached_response(&cache_key).await {
            println!("📱 Using cached OpenAI response for: {}", text);
            return Ok(cached_response);
        }

        // Try API call with retries
        for attempt in 1..=self.config.max_retries {
            match self.call_openai_sentiment_api(text).await {
                Ok(sentiment) => {
                    self.cache_response(cache_key, sentiment.clone()).await;
                    return Ok(sentiment);
                }
                Err(e) => {
                    eprintln!(
                        "❌ OpenAI attempt {}/{}: {}",
                        attempt, self.config.max_retries, e
                    );
                    if attempt == self.config.max_retries {
                        if self.config.enable_fallback {
                            println!("🔄 Falling back to simple sentiment analysis");
                            return Ok(self.fallback_sentiment_analysis(text));
                        }
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
                    "content": "Analyze sentiment. Respond with only: positive, negative, or neutral"
                },
                {
                    "role": "user",
                    "content": format!("Sentiment analysis: {}", text)
                }
            ],
            "max_tokens": 10,
            "temperature": 0.1
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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("OpenAI API error: {}", error_text).into());
        }

        let response_json: serde_json::Value = response.json().await?;

        let sentiment = response_json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("neutral")
            .trim()
            .to_lowercase();

        Ok(sentiment)
    }

    pub async fn ask_claude_decision(
        &self,
        question: &str,
        context: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("claude_decision:{}:{}", question, context);

        if let Some(cached_response) = self.get_cached_response(&cache_key).await {
            println!("📱 Using cached Claude response");
            return Ok(cached_response);
        }

        for attempt in 1..=self.config.max_retries {
            match self.call_anthropic_api(question, context).await {
                Ok(decision) => {
                    self.cache_response(cache_key, decision.clone()).await;
                    return Ok(decision);
                }
                Err(e) => {
                    eprintln!(
                        "❌ Anthropic attempt {}/{}: {}",
                        attempt, self.config.max_retries, e
                    );
                    if attempt == self.config.max_retries {
                        if self.config.enable_fallback {
                            println!("🔄 Falling back to rule-based decision");
                            return Ok(self.fallback_decision_logic(context));
                        }
                        return Err(e);
                    }
                    tokio::time::sleep(Duration::from_millis(1000 * attempt as u64)).await;
                }
            }
        }

        unreachable!()
    }

    async fn call_anthropic_api(
        &self,
        question: &str,
        context: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request_body = json!({
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 100,
            "messages": [
                {
                    "role": "user",
                    "content": format!("Business Decision: {}\nContext: {}\nRespond with: approve, deny, or review", question, context)
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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Anthropic API error: {}", error_text).into());
        }

        let response_json: serde_json::Value = response.json().await?;

        let decision_text = response_json["content"][0]["text"]
            .as_str()
            .unwrap_or("review")
            .to_lowercase();

        // Extract decision
        if decision_text.contains("approve") {
            Ok("approve".to_string())
        } else if decision_text.contains("deny") {
            Ok("deny".to_string())
        } else {
            Ok("review".to_string())
        }
    }

    pub async fn analyze_sentiment_huggingface(
        &self,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let cache_key = format!("hf_sentiment:{}", text);

        if let Some(cached_response) = self.get_cached_response(&cache_key).await {
            println!("📱 Using cached Hugging Face response");
            return Ok(cached_response);
        }

        for attempt in 1..=self.config.max_retries {
            match self.call_huggingface_api(text).await {
                Ok(sentiment) => {
                    self.cache_response(cache_key, sentiment.clone()).await;
                    return Ok(sentiment);
                }
                Err(e) => {
                    eprintln!(
                        "❌ Hugging Face attempt {}/{}: {}",
                        attempt, self.config.max_retries, e
                    );
                    if attempt == self.config.max_retries {
                        if self.config.enable_fallback {
                            println!("🔄 Falling back to simple sentiment analysis");
                            return Ok(self.fallback_sentiment_analysis(text));
                        }
                        return Err(e);
                    }
                    tokio::time::sleep(Duration::from_millis(1000 * attempt as u64)).await;
                }
            }
        }

        unreachable!()
    }

    async fn call_huggingface_api(
        &self,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Hugging Face API error: {}", error_text).into());
        }

        let response_json: serde_json::Value = response.json().await?;

        // Parse HF response
        if let Some(predictions) = response_json.as_array() {
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

    // Fallback methods for when AI APIs are unavailable
    fn fallback_sentiment_analysis(&self, text: &str) -> String {
        let positive_words = [
            "good",
            "great",
            "excellent",
            "amazing",
            "love",
            "fantastic",
            "wonderful",
        ];
        let negative_words = [
            "bad",
            "terrible",
            "awful",
            "hate",
            "horrible",
            "disappointed",
            "worst",
        ];

        let text_lower = text.to_lowercase();
        let positive_count = positive_words
            .iter()
            .filter(|&&word| text_lower.contains(word))
            .count();
        let negative_count = negative_words
            .iter()
            .filter(|&&word| text_lower.contains(word))
            .count();

        if positive_count > negative_count {
            "positive".to_string()
        } else if negative_count > positive_count {
            "negative".to_string()
        } else {
            "neutral".to_string()
        }
    }

    fn fallback_decision_logic(&self, context: &str) -> String {
        // Simple rule-based fallback decision
        if context.contains("VIP") || context.contains("premium") {
            "approve".to_string()
        } else if context.contains("risk") || context.contains("suspicious") {
            "deny".to_string()
        } else {
            "review".to_string()
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { run_production_ai_example().await })
}

async fn run_production_ai_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Production AI-Powered Rule Engine with Error Handling & Caching");

    // Load configuration from environment
    let config = AIServiceConfig::default();
    let ai_service = Arc::new(AIService::new(config));

    let ai_rules = r#"
        rule "Production AI Sentiment" salience 100 {
            when
                CustomerMessage.type == "support_ticket"
            then
                analyzeProductionSentiment(CustomerMessage.text);
                set(Ticket.ai_processed, true);
                logMessage("🤖 Production AI analyzing sentiment with fallback");
        }

        rule "Production AI Decision" salience 90 {
            when
                Customer.needsApproval == true
            then
                makeProductionDecision("Approve customer for premium features?", Customer.context);
                set(Customer.ai_decision_made, true);
                logMessage("🧠 Production AI making business decision");
        }

        rule "Multi-AI Sentiment Comparison" salience 80 {
            when
                CustomerMessage.type == "feedback"
            then
                compareSentimentModels(CustomerMessage.text);
                set(Analysis.multi_ai_complete, true);
                logMessage("🤗 Comparing multiple AI sentiment models");
        }
    "#;

    let mut engine = RuleEngineBuilder::new().with_inline_grl(ai_rules)?.build();

    // Register production AI functions with error handling
    let ai_service_clone = ai_service.clone();
    engine.register_function("analyzeProductionSentiment", move |args, facts| {
        let text = args[0].as_string().unwrap_or("".to_string());
        let service = ai_service_clone.clone();
        let text_clone = text.clone();

        match std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(service.analyze_sentiment_openai(&text_clone))
        })
        .join()
        .unwrap()
        {
            Ok(sentiment) => {
                facts.add_value(
                    "Analysis.production_sentiment",
                    Value::String(sentiment.clone()),
                )?;
                println!("✅ Production Sentiment: '{}' → {}", text, sentiment);
                Ok(Value::String(sentiment))
            }
            Err(e) => {
                eprintln!("❌ Production sentiment analysis failed: {}", e);
                Ok(Value::String("error".to_string()))
            }
        }
    });

    let ai_service_clone2 = ai_service.clone();
    engine.register_function("makeProductionDecision", move |args, facts| {
        let question = args[0].as_string().unwrap_or("".to_string());
        let context = args[1].as_string().unwrap_or("".to_string());
        let service = ai_service_clone2.clone();
        let question_clone = question.clone();
        let context_clone = context.clone();

        match std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(service.ask_claude_decision(&question_clone, &context_clone))
        })
        .join()
        .unwrap()
        {
            Ok(decision) => {
                facts.add_value(
                    "Decision.production_result",
                    Value::String(decision.clone()),
                )?;
                println!("✅ Production Decision: {} → {}", question, decision);
                Ok(Value::String(decision))
            }
            Err(e) => {
                eprintln!("❌ Production decision making failed: {}", e);
                Ok(Value::String("error".to_string()))
            }
        }
    });

    let ai_service_clone3 = ai_service.clone();
    engine.register_function("compareSentimentModels", move |args, facts| {
        let text = args[0].as_string().unwrap_or("".to_string());
        let service = ai_service_clone3.clone();
        let text_clone = text.clone();

        match std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();

            // Try both OpenAI and Hugging Face
            let openai_result = rt.block_on(service.analyze_sentiment_openai(&text_clone));
            let hf_result = rt.block_on(service.analyze_sentiment_huggingface(&text_clone));
            (openai_result, hf_result)
        })
        .join()
        .unwrap()
        {
            (Ok(openai_sentiment), Ok(hf_sentiment)) => {
                facts.add_value(
                    "Comparison.openai_sentiment",
                    Value::String(openai_sentiment.clone()),
                )?;
                facts.add_value(
                    "Comparison.hf_sentiment",
                    Value::String(hf_sentiment.clone()),
                )?;

                let consensus = if openai_sentiment == hf_sentiment {
                    openai_sentiment.clone()
                } else {
                    "mixed".to_string()
                };

                facts.add_value("Comparison.consensus", Value::String(consensus.clone()))?;
                println!(
                    "🔬 Multi-AI Comparison: OpenAI: {}, HF: {}, Consensus: {}",
                    openai_sentiment, hf_sentiment, consensus
                );

                Ok(Value::String(consensus))
            }
            _ => {
                println!("❌ Multi-AI comparison failed, using fallback");
                Ok(Value::String("fallback".to_string()))
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
        println!("📝 {:?}", args[0]);
        Ok(Value::Boolean(true))
    });

    // Set up test facts
    let facts = Facts::new();

    // Support ticket for production sentiment analysis
    let mut customer_message = HashMap::new();
    customer_message.insert(
        "type".to_string(),
        Value::String("support_ticket".to_string()),
    );
    customer_message.insert(
        "text".to_string(),
        Value::String("The product is great but the delivery was delayed".to_string()),
    );
    facts.add_value("CustomerMessage", Value::Object(customer_message))?;

    // Customer feedback for multi-AI comparison
    let mut customer_feedback = HashMap::new();
    customer_feedback.insert("type".to_string(), Value::String("feedback".to_string()));
    customer_feedback.insert(
        "text".to_string(),
        Value::String("Absolutely love this service! Outstanding quality.".to_string()),
    );
    facts.add_value("CustomerFeedback", Value::Object(customer_feedback))?;

    // Customer for decision making
    let mut customer = HashMap::new();
    customer.insert("needsApproval".to_string(), Value::Boolean(true));
    customer.insert(
        "context".to_string(),
        Value::String("VIP_customer_with_excellent_history".to_string()),
    );
    facts.add_value("Customer", Value::Object(customer))?;

    let mut ticket = HashMap::new();
    facts.add_value("Ticket", Value::Object(ticket))?;

    let mut analysis = HashMap::new();
    facts.add_value("Analysis", Value::Object(analysis))?;

    // Execute production AI-powered rules
    println!("\n🎯 Executing Production-Ready AI Rules...\n");
    let result = engine.execute(&facts)?;

    println!("\n📊 Production Execution Results:");
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Cycles: {}", result.cycle_count);
    println!("   Duration: {:?}", result.execution_time);

    // Display cache statistics
    let cache_size = ai_service.cache.read().await.len();
    println!("   Cached responses: {}", cache_size);

    println!("\n🛡️ Production Features Demonstrated:");
    println!("   ✅ Automatic retry with exponential backoff");
    println!("   ✅ Intelligent caching with TTL");
    println!("   ✅ Graceful fallback to rule-based logic");
    println!("   ✅ Comprehensive error handling");
    println!("   ✅ Multi-AI model comparison");
    println!("   ✅ Timeout protection");

    println!("\n🔧 Setup Instructions:");
    println!("   1. Copy .env.example to .env");
    println!("   2. Add your API keys: OPENAI_API_KEY, ANTHROPIC_API_KEY, HF_API_KEY");
    println!("   3. Configure retry and caching settings");
    println!("   4. Run: cargo run --example production_ai_service");

    Ok(())
}

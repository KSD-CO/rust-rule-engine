use reqwest::Client;
use rust_rule_engine::{Facts, RuleEngineBuilder, Value};
use serde_json::json;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct HuggingFaceConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("OPENAI_API_KEY")
                .unwrap_or_else(|_| "your-openai-key".to_string()),
            model: "gpt-3.5-turbo".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
        }
    }
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("ANTHROPIC_API_KEY")
                .unwrap_or_else(|_| "your-anthropic-key".to_string()),
            model: "claude-3-sonnet-20240229".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }
}

impl Default for HuggingFaceConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("HF_API_KEY").unwrap_or_else(|_| "your-hf-key".to_string()),
            model: "cardiffnlp/twitter-roberta-base-sentiment-latest".to_string(),
            base_url: "https://api-inference.huggingface.co/models".to_string(),
        }
    }
}

// OpenAI API Integration
async fn call_openai_sentiment(
    text: &str,
    config: &OpenAIConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let request_body = json!({
        "model": config.model,
        "messages": [
            {
                "role": "system",
                "content": "You are a sentiment analysis expert. Respond with only 'positive', 'negative', or 'neutral'."
            },
            {
                "role": "user",
                "content": format!("Analyze the sentiment of this text: {}", text)
            }
        ],
        "max_tokens": 10,
        "temperature": 0.1
    });

    let response = client
        .post(&format!("{}/chat/completions", config.base_url))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("OpenAI API error: {}", response.status()).into());
    }

    let response_json: serde_json::Value = response.json().await?;

    let sentiment = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("neutral")
        .trim()
        .to_lowercase();

    Ok(sentiment)
}

// Anthropic Claude API Integration
async fn call_anthropic_llm_decision(
    question: &str,
    customer_id: &str,
    config: &AnthropicConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let request_body = json!({
        "model": config.model,
        "max_tokens": 100,
        "messages": [
            {
                "role": "user",
                "content": format!("Business Decision Question: {}\nCustomer ID: {}\n\nProvide a clear decision (approve/deny/review) with brief reasoning.", question, customer_id)
            }
        ]
    });

    let response = client
        .post(&format!("{}/messages", config.base_url))
        .header("x-api-key", &config.api_key)
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Anthropic API error: {}", response.status()).into());
    }

    let response_json: serde_json::Value = response.json().await?;

    let decision = response_json["content"][0]["text"]
        .as_str()
        .unwrap_or("review")
        .to_lowercase();

    // Extract decision keyword
    if decision.contains("approve") {
        Ok("approve".to_string())
    } else if decision.contains("deny") {
        Ok("deny".to_string())
    } else {
        Ok("review".to_string())
    }
}

// Hugging Face API Integration
async fn call_huggingface_sentiment(
    text: &str,
    config: &HuggingFaceConfig,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let request_body = json!({
        "inputs": text
    });

    let response = client
        .post(&format!("{}/{}", config.base_url, config.model))
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Hugging Face API error: {}", response.status()).into());
    }

    let response_json: serde_json::Value = response.json().await?;

    // Parse HF sentiment response
    if let Some(predictions) = response_json.as_array() {
        if let Some(first_prediction) = predictions.first() {
            if let Some(predictions_array) = first_prediction.as_array() {
                // Find highest scoring sentiment
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

// Custom ML fraud detection API (example with your own model)
async fn call_fraud_detection_api(
    amount: f64,
    user_id: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    // Example call to your ML fraud detection service
    let request_body = json!({
        "transaction": {
            "amount": amount,
            "user_id": user_id,
            "timestamp": chrono::Utc::now().timestamp()
        }
    });

    // Replace with your actual fraud detection API endpoint
    let api_endpoint = std::env::var("FRAUD_API_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8080/api/fraud/detect".to_string());

    let response = client
        .post(&api_endpoint)
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                std::env::var("FRAUD_API_KEY").unwrap_or_else(|_| "your-fraud-api-key".to_string())
            ),
        )
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Fraud API error: {}", response.status()).into());
    }

    let response_json: serde_json::Value = response.json().await?;

    let risk_score = response_json["risk_score"].as_f64().unwrap_or(0.0);
    let is_fraud = risk_score > 0.7; // 70% threshold

    Ok(is_fraud)
}

// Customer tier prediction using your ML model
async fn call_tier_prediction_api(
    customer_id: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = Client::new();

    let request_body = json!({
        "customer_id": customer_id,
        "features": {
            // These would be populated from your customer data
            "total_spent": 0.0,
            "order_count": 0,
            "days_active": 0,
            "avg_order_value": 0.0
        }
    });

    // Replace with your actual tier prediction API endpoint
    let api_endpoint = std::env::var("TIER_API_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8080/api/ml/predict-tier".to_string());

    let response = client
        .post(&api_endpoint)
        .header("Content-Type", "application/json")
        .header(
            "Authorization",
            format!(
                "Bearer {}",
                std::env::var("ML_API_KEY").unwrap_or_else(|_| "your-ml-api-key".to_string())
            ),
        )
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Tier prediction API error: {}", response.status()).into());
    }

    let response_json: serde_json::Value = response.json().await?;

    let predicted_tier = response_json["predicted_tier"]
        .as_str()
        .unwrap_or("bronze")
        .to_string();

    Ok(predicted_tier)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { run_ai_example().await })
}

async fn run_ai_example() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize AI service configurations
    let openai_config = OpenAIConfig::default();
    let anthropic_config = AnthropicConfig::default();
    let hf_config = HuggingFaceConfig::default();

    println!("🤖 Initializing Real AI-Powered Rule Engine...");
    println!("🔑 OpenAI Model: {}", openai_config.model);
    println!("🧠 Anthropic Model: {}", anthropic_config.model);
    println!("🤗 Hugging Face Model: {}", hf_config.model);

    let ai_rules = r#"
        rule "AI Sentiment Analysis" salience 100 {
            when
                CustomerMessage.type == "complaint"
            then
                analyzeSentimentOpenAI(CustomerMessage.text);
                set(Ticket.priority, "high");
                logMessage("🤖 OpenAI analyzing customer sentiment");
        }

        rule "AI Sentiment Analysis HF" salience 95 {
            when
                CustomerMessage.type == "feedback"
            then
                analyzeSentimentHF(CustomerMessage.text);
                set(Ticket.source, "huggingface");
                logMessage("🤗 Hugging Face analyzing feedback sentiment");
        }

        rule "AI Business Decision" salience 90 {
            when
                Customer.needsReview == true
            then
                askClaudeDecision("Should we approve this customer for premium tier?", Customer.id);
                set(Customer.reviewedBy, "AI-Claude");
                logMessage("🧠 Claude analyzing customer for approval");
        }

        rule "AI Fraud Detection" salience 85 {
            when
                Transaction.amount > 1000
            then
                detectFraudML(Transaction.amount, Transaction.userId);
                set(Transaction.status, "ai_reviewed");
                logMessage("🛡️ ML model detecting fraud patterns");
        }

        rule "AI Tier Prediction" salience 80 {
            when
                Customer.tier == "pending"
            then
                predictTierML(Customer.id);
                set(Customer.tierAssignedBy, "AI-ML");
                logMessage("📊 ML predicting customer tier");
        }
    "#;

    let mut engine = RuleEngineBuilder::new().with_inline_grl(ai_rules)?.build();

    // Register OpenAI sentiment analysis
    let openai_config_clone = openai_config.clone();
    engine.register_function("analyzeSentimentOpenAI", move |args, facts| {
        let text = args[0].as_string().unwrap_or("".to_string());
        let config = openai_config_clone.clone();
        let text_clone = text.clone();

        // Use simple async blocking call in sync context
        match std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(call_openai_sentiment(&text_clone, &config))
        })
        .join()
        .unwrap()
        {
            Ok(sentiment) => {
                facts.add_value(
                    "Analysis.openai_sentiment",
                    Value::String(sentiment.clone()),
                )?;
                println!("🤖 OpenAI Sentiment: '{}' → {}", text, sentiment);
                Ok(Value::String(sentiment))
            }
            Err(e) => {
                eprintln!("❌ OpenAI API error: {}", e);
                // Fallback to simple analysis
                let sentiment = if text.contains("terrible") {
                    "negative"
                } else {
                    "neutral"
                };
                Ok(Value::String(sentiment.to_string()))
            }
        }
    });

    // Register Hugging Face sentiment analysis
    let hf_config_clone = hf_config.clone();
    engine.register_function("analyzeSentimentHF", move |args, facts| {
        let text = args[0].as_string().unwrap_or("".to_string());
        let config = hf_config_clone.clone();
        let text_clone = text.clone();

        let _rt = tokio::runtime::Handle::current();
        match std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(call_huggingface_sentiment(&text_clone, &config))
        })
        .join()
        .unwrap()
        {
            Ok(sentiment) => {
                facts.add_value("Analysis.hf_sentiment", Value::String(sentiment.clone()))?;
                println!("🤗 Hugging Face Sentiment: '{}' → {}", text, sentiment);
                Ok(Value::String(sentiment))
            }
            Err(e) => {
                eprintln!("❌ Hugging Face API error: {}", e);
                let sentiment = "neutral";
                Ok(Value::String(sentiment.to_string()))
            }
        }
    });

    // Register Anthropic Claude decision making
    let anthropic_config_clone = anthropic_config.clone();
    engine.register_function("askClaudeDecision", move |args, facts| {
        let question = args[0].as_string().unwrap_or("".to_string());
        let customer_id = args[1].as_string().unwrap_or("unknown".to_string());
        let config = anthropic_config_clone.clone();
        let question_clone = question.clone();
        let customer_id_clone = customer_id.clone();

        let _rt = tokio::runtime::Handle::current();
        match std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(call_anthropic_llm_decision(
                &question_clone,
                &customer_id_clone,
                &config,
            ))
        })
        .join()
        .unwrap()
        {
            Ok(decision) => {
                facts.add_value("Decision.claude_result", Value::String(decision.clone()))?;
                println!("🧠 Claude Decision: {} → {}", question, decision);
                Ok(Value::String(decision))
            }
            Err(e) => {
                eprintln!("❌ Anthropic API error: {}", e);
                let decision = "review";
                Ok(Value::String(decision.to_string()))
            }
        }
    });

    // Register ML fraud detection
    engine.register_function("detectFraudML", move |args, facts| {
        let amount = args[0].as_number().unwrap_or(0.0);
        let user_id = args[1].as_string().unwrap_or("unknown".to_string());
        let user_id_clone = user_id.clone();

        let _rt = tokio::runtime::Handle::current();
        match std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(call_fraud_detection_api(amount, &user_id_clone))
        })
        .join()
        .unwrap()
        {
            Ok(is_fraud) => {
                facts.add_value("FraudCheck.ml_result", Value::Boolean(is_fraud))?;
                println!(
                    "🛡️ ML Fraud Detection: Amount {}, User {} → Fraud: {}",
                    amount, user_id, is_fraud
                );
                Ok(Value::Boolean(is_fraud))
            }
            Err(e) => {
                eprintln!("❌ Fraud detection API error: {}", e);
                // Fallback logic
                let is_fraud = amount > 5000.0;
                Ok(Value::Boolean(is_fraud))
            }
        }
    });

    // Register ML tier prediction
    engine.register_function("predictTierML", move |args, facts| {
        let customer_id = args[0].as_string().unwrap_or("unknown".to_string());
        let customer_id_clone = customer_id.clone();

        let _rt = tokio::runtime::Handle::current();
        match std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(call_tier_prediction_api(&customer_id_clone))
        })
        .join()
        .unwrap()
        {
            Ok(tier) => {
                facts.add_value("TierPrediction.ml_result", Value::String(tier.clone()))?;
                println!("📊 ML Tier Prediction: Customer {} → {}", customer_id, tier);
                Ok(Value::String(tier))
            }
            Err(e) => {
                eprintln!("❌ Tier prediction API error: {}", e);
                // Fallback logic
                let tier = "bronze";
                Ok(Value::String(tier.to_string()))
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

    // Customer complaint for OpenAI sentiment analysis
    let mut customer_message = HashMap::new();
    customer_message.insert("type".to_string(), Value::String("complaint".to_string()));
    customer_message.insert(
        "text".to_string(),
        Value::String("This service is absolutely terrible! I'm very disappointed.".to_string()),
    );
    facts.add_value("CustomerMessage", Value::Object(customer_message))?;

    // Customer feedback for Hugging Face sentiment analysis
    let mut customer_feedback = HashMap::new();
    customer_feedback.insert("type".to_string(), Value::String("feedback".to_string()));
    customer_feedback.insert(
        "text".to_string(),
        Value::String("The product quality is excellent and delivery was fast!".to_string()),
    );
    facts.add_value("CustomerFeedback", Value::Object(customer_feedback))?;

    // Transaction for fraud detection
    let mut transaction = HashMap::new();
    transaction.insert("amount".to_string(), Value::Number(2500.0));
    transaction.insert("userId".to_string(), Value::String("user123".to_string()));
    facts.add_value("Transaction", Value::Object(transaction))?;

    // Customer for tier prediction and decision making
    let mut customer = HashMap::new();
    customer.insert(
        "id".to_string(),
        Value::String("premium_candidate_456".to_string()),
    );
    customer.insert("tier".to_string(), Value::String("pending".to_string()));
    customer.insert("needsReview".to_string(), Value::Boolean(true));
    facts.add_value("Customer", Value::Object(customer))?;

    let ticket = HashMap::new();
    facts.add_value("Ticket", Value::Object(ticket))?;

    // Execute AI-powered rules
    println!("\n🚀 Executing Real AI-Powered Rule Engine...\n");
    let result = engine.execute(&facts)?;

    println!("\n📊 Execution Results:");
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Cycles: {}", result.cycle_count);
    println!("   Duration: {:?}", result.execution_time);

    println!("\n💡 Tips for Production:");
    println!("   1. Set environment variables: OPENAI_API_KEY, ANTHROPIC_API_KEY, HF_API_KEY");
    println!("   2. Configure your ML API endpoints: FRAUD_API_ENDPOINT, TIER_API_ENDPOINT");
    println!("   3. Implement proper error handling and retry logic");
    println!("   4. Add caching for frequently used AI responses");
    println!("   5. Monitor API usage and costs");

    Ok(())
}

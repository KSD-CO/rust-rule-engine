# Real AI Integration Setup Guide

This guide shows how to replace simulation functions with actual AI API calls from OpenAI, Anthropic, and Hugging Face.

## 🔑 API Keys Setup

### 1. Get Your API Keys

**OpenAI (GPT-3.5/GPT-4):**
```bash
# Sign up at https://platform.openai.com/
# Go to API Keys section
# Create new secret key
export OPENAI_API_KEY="sk-your-openai-key-here"
```

**Anthropic (Claude):**
```bash
# Sign up at https://console.anthropic.com/
# Go to API Keys section  
# Create new API key
export ANTHROPIC_API_KEY="sk-ant-your-anthropic-key-here"
```

**Hugging Face:**
```bash
# Sign up at https://huggingface.co/
# Go to Settings > Access Tokens
# Create new token with read permissions
export HF_API_KEY="hf_your-huggingface-token-here"
```

### 2. Environment Configuration

Create a `.env` file in your project root:

```bash
cp .env.example .env
# Edit .env with your actual API keys
```

Or set environment variables directly:

```bash
export OPENAI_API_KEY="your-openai-key"
export ANTHROPIC_API_KEY="your-anthropic-key"  
export HF_API_KEY="your-huggingface-key"
export FRAUD_API_ENDPOINT="http://your-ml-service.com/api/fraud"
export TIER_API_ENDPOINT="http://your-ml-service.com/api/tier"
```

## 🚀 Real AI Examples

### 1. Basic Real AI Integration

```rust
// examples/real_ai_integration.rs
cargo run --example real_ai_integration
```

**Features:**
- ✅ OpenAI GPT-3.5 sentiment analysis
- ✅ Anthropic Claude business decisions
- ✅ Hugging Face sentiment models
- ✅ Custom ML API integration
- ✅ Error handling with fallbacks

### 2. Production AI Service

```rust
// examples/production_ai_service.rs  
cargo run --example production_ai_service
```

**Features:**
- ✅ Intelligent caching with TTL
- ✅ Automatic retry with exponential backoff
- ✅ Cost tracking and monitoring
- ✅ Multi-AI model comparison
- ✅ Graceful fallback strategies

### 3. AI REST API

```rust
// examples/ai_rest_api_production.rs
cargo run --example ai_rest_api_production
```

**Features:**
- ✅ Production REST API with AI integration
- ✅ Real-time cost tracking
- ✅ Cache performance monitoring
- ✅ Provider usage statistics
- ✅ Health checks and monitoring

## 🔧 Integration Examples

### OpenAI Integration

```rust
async fn call_openai_sentiment_api(text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system", 
                "content": "Analyze sentiment. Respond with: positive, negative, or neutral"
            },
            {
                "role": "user",
                "content": format!("Sentiment: {}", text)
            }
        ],
        "max_tokens": 10,
        "temperature": 0.1
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", env::var("OPENAI_API_KEY")?))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;
    let sentiment = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("neutral")
        .trim()
        .to_lowercase();

    Ok(sentiment)
}
```

### Anthropic Claude Integration

```rust
async fn call_anthropic_decision(question: &str, context: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    let request_body = json!({
        "model": "claude-3-sonnet-20240229",
        "max_tokens": 100,
        "messages": [
            {
                "role": "user",
                "content": format!("Business Decision: {}\nContext: {}\nRespond: approve, deny, or review", question, context)
            }
        ]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", env::var("ANTHROPIC_API_KEY")?)
        .header("Content-Type", "application/json")
        .header("anthropic-version", "2023-06-01")
        .json(&request_body)
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;
    let decision_text = response_json["content"][0]["text"]
        .as_str()
        .unwrap_or("review")
        .to_lowercase();

    let decision = if decision_text.contains("approve") {
        "approve"
    } else if decision_text.contains("deny") {
        "deny"
    } else {
        "review"
    };

    Ok(decision.to_string())
}
```

### Hugging Face Integration

```rust
async fn call_huggingface_sentiment(text: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();
    
    let request_body = json!({
        "inputs": text
    });

    let response = client
        .post("https://api-inference.huggingface.co/models/cardiffnlp/twitter-roberta-base-sentiment-latest")
        .header("Authorization", format!("Bearer {}", env::var("HF_API_KEY")?))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;
    
    // Parse Hugging Face response format
    if let Some(predictions) = response_json.as_array() {
        if let Some(first_prediction) = predictions.first() {
            if let Some(predictions_array) = first_prediction.as_array() {
                let mut best_sentiment = "neutral";
                let mut best_score = 0.0;
                
                for prediction in predictions_array {
                    if let (Some(label), Some(score)) = (
                        prediction["label"].as_str(),
                        prediction["score"].as_f64()
                    ) {
                        if score > best_score {
                            best_score = score;
                            best_sentiment = match label {
                                "LABEL_0" => "negative",
                                "LABEL_1" => "neutral",
                                "LABEL_2" => "positive",
                                _ => "neutral"
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
```

## 💡 Production Best Practices

### 1. Error Handling & Retries

```rust
pub struct AIServiceConfig {
    pub max_retries: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub timeout_seconds: u64,
}

async fn call_with_retry<F, Fut, T>(
    config: &AIServiceConfig,
    operation: F,
) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>>,
{
    let mut delay = config.base_delay_ms;
    
    for attempt in 1..=config.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == config.max_retries {
                    return Err(e);
                }
                
                eprintln!("Attempt {}/{} failed: {}", attempt, config.max_retries, e);
                tokio::time::sleep(Duration::from_millis(delay)).await;
                delay = std::cmp::min(delay * 2, config.max_delay_ms);
            }
        }
    }
    
    unreachable!()
}
```

### 2. Intelligent Caching

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct AICache {
    cache: HashMap<String, (String, Instant, f64)>, // response, timestamp, cost
    ttl: Duration,
}

impl AICache {
    pub fn get(&self, key: &str) -> Option<String> {
        if let Some((response, timestamp, _)) = self.cache.get(key) {
            if timestamp.elapsed() < self.ttl {
                return Some(response.clone());
            }
        }
        None
    }
    
    pub fn put(&mut self, key: String, response: String, cost: f64) {
        self.cache.insert(key, (response, Instant::now(), cost));
        self.cleanup_expired();
    }
    
    fn cleanup_expired(&mut self) {
        self.cache.retain(|_, (_, timestamp, _)| timestamp.elapsed() < self.ttl);
    }
}
```

### 3. Cost Management

```rust
pub struct CostTracker {
    total_cost: f64,
    monthly_limit: f64,
    provider_costs: HashMap<String, f64>,
}

impl CostTracker {
    pub fn can_afford(&self, estimated_cost: f64) -> bool {
        self.total_cost + estimated_cost <= self.monthly_limit
    }
    
    pub fn record_usage(&mut self, provider: &str, cost: f64) {
        self.total_cost += cost;
        *self.provider_costs.entry(provider.to_string()).or_insert(0.0) += cost;
    }
    
    pub fn get_cost_breakdown(&self) -> HashMap<String, f64> {
        self.provider_costs.clone()
    }
}
```

## 📊 Testing & Monitoring

### 1. Test Real Integration

```bash
# Start the AI REST API
cargo run --example ai_rest_api_production

# Run comprehensive tests
./test_real_ai.sh
```

### 2. Monitor Performance

```bash
# Check AI service statistics
curl http://localhost:3000/api/v1/ai/stats | jq '.'

# Monitor costs
curl http://localhost:3000/api/v1/ai/stats | jq '.total_cost_estimate'

# Check cache performance  
curl http://localhost:3000/api/v1/ai/stats | jq '.cache_performance'
```

### 3. Load Testing

```bash
# Install hey for load testing
go install github.com/rakyll/hey@latest

# Test API performance
hey -n 100 -c 10 -m POST \
  -H "Content-Type: application/json" \
  -d '{"facts":{"CustomerMessage":{"text":"Test message","provider":"openai"}}}' \
  http://localhost:3000/api/v1/rules/execute
```

## 💰 Cost Optimization

### 1. API Cost Estimates (October 2025)

| Provider | Model | Cost per 1K tokens | Use Case |
|----------|--------|-------------------|----------|
| OpenAI | GPT-3.5-turbo | $0.0015/$0.002 | Fast sentiment analysis |
| OpenAI | GPT-4 | $0.03/$0.06 | Complex reasoning |
| Anthropic | Claude-3-Sonnet | $0.003/$0.015 | Business decisions |
| Hugging Face | Inference API | $0.001 | Basic sentiment analysis |

### 2. Optimization Strategies

- **Caching**: Cache identical requests (50-80% cost reduction)
- **Model Selection**: Use cheaper models for simple tasks
- **Batch Processing**: Group multiple requests when possible
- **Fallback Logic**: Use rule-based logic when AI fails
- **Rate Limiting**: Prevent cost runaway in high-traffic scenarios

### 3. Production Configuration

```rust
// Production-optimized configuration
let config = AIServiceConfig {
    enable_caching: true,
    cache_ttl_seconds: 3600, // 1 hour
    max_retries: 3,
    request_timeout_seconds: 15,
    monthly_cost_limit: 100.0, // $100/month
    fallback_enabled: true,
    rate_limit_per_minute: 60,
};
```

## 🛡️ Security & Privacy

### 1. API Key Management

```bash
# Use environment variables (never commit keys)
export OPENAI_API_KEY="$(cat /path/to/secure/openai.key)"

# Use secret management in production
kubectl create secret generic ai-keys \
  --from-literal=openai-key="$OPENAI_API_KEY" \
  --from-literal=anthropic-key="$ANTHROPIC_API_KEY"
```

### 2. Data Privacy

- **Data Retention**: Don't log sensitive customer data
- **Encryption**: Encrypt data in transit and at rest
- **Compliance**: Follow GDPR, CCPA, SOC2 requirements
- **Audit Logs**: Track all AI API calls for compliance

### 3. Rate Limiting

```rust
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    limit_per_minute: u32,
}

impl RateLimiter {
    pub fn can_proceed(&mut self, key: &str) -> bool {
        let now = Instant::now();
        let requests = self.requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // Remove requests older than 1 minute
        requests.retain(|&timestamp| now.duration_since(timestamp) < Duration::from_secs(60));
        
        if requests.len() >= self.limit_per_minute as usize {
            return false;
        }
        
        requests.push(now);
        true
    }
}
```

## 📈 Scaling to Production

### 1. Horizontal Scaling

```yaml
# kubernetes deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ai-rule-engine
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ai-rule-engine
  template:
    metadata:
      labels:
        app: ai-rule-engine
    spec:
      containers:
      - name: ai-rule-engine
        image: your-registry/ai-rule-engine:latest
        env:
        - name: OPENAI_API_KEY
          valueFrom:
            secretKeyRef:
              name: ai-keys
              key: openai-key
        ports:
        - containerPort: 3000
```

### 2. Redis Caching

```rust
use redis::AsyncCommands;

pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.client.get_async_connection().await.ok()?;
        conn.get(key).await.ok()
    }
    
    pub async fn set(&self, key: &str, value: &str, ttl: Duration) {
        if let Ok(mut conn) = self.client.get_async_connection().await {
            let _: Result<(), _> = conn.set_ex(key, value, ttl.as_secs()).await;
        }
    }
}
```

### 3. Observability

```rust
use tracing::{info, warn, error, instrument};

#[instrument]
pub async fn analyze_sentiment_with_monitoring(
    text: &str,
    provider: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let start = Instant::now();
    
    match provider {
        "openai" => {
            let result = call_openai_sentiment_api(text).await;
            info!(
                provider = provider,
                duration_ms = start.elapsed().as_millis(),
                success = result.is_ok(),
                "AI sentiment analysis completed"
            );
            result
        }
        _ => Err("Unsupported provider".into())
    }
}
```

## 🎯 Next Steps

1. **Start with one provider** (OpenAI is easiest to begin with)
2. **Implement caching** for cost efficiency
3. **Add error handling** with graceful fallbacks  
4. **Monitor costs** and set appropriate limits
5. **Scale gradually** based on usage patterns
6. **Add security measures** for production deployment

## 📚 Resources

- [OpenAI API Documentation](https://platform.openai.com/docs)
- [Anthropic Claude API](https://docs.anthropic.com/claude/reference)
- [Hugging Face Inference API](https://huggingface.co/docs/api-inference)
- [Rust Async Programming](https://rust-lang.github.io/async-book/)
- [Production Rust Deployment](https://doc.rust-lang.org/cargo/guide/build-cache.html)

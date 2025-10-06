/// REST API Example with Advanced Analytics Monitoring
///
/// This example demonstrates how to build a REST API that integrates
/// the Rust Rule Engine with Advanced Analytics monitoring.
///
/// Features:
/// - Real-time rule execution via HTTP endpoints
/// - Live analytics dashboard accessible via API
/// - Performance monitoring and optimization insights
/// - Production-ready configuration
/// - Health checks and status endpoints
use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{get, post},
    Router,
};
use rust_rule_engine::{
    engine::{AnalyticsConfig, RuleAnalytics},
    EngineConfig, Facts, GRLParser, RustRuleEngine, Value,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub engine: Arc<Mutex<RustRuleEngine>>,
}

/// Request payload for rule execution
#[derive(Debug, Deserialize)]
pub struct ExecuteRulesRequest {
    pub facts: HashMap<String, serde_json::Value>,
    pub rules: Option<Vec<String>>, // Optional: specific rules to execute
}

/// Response for rule execution
#[derive(Debug, Serialize)]
pub struct ExecuteRulesResponse {
    pub success: bool,
    pub rules_fired: usize,
    pub cycle_count: usize,
    pub execution_time_ms: f64,
    pub updated_facts: HashMap<String, serde_json::Value>,
    pub message: String,
}

/// Analytics dashboard response
#[derive(Debug, Serialize)]
pub struct AnalyticsDashboard {
    pub overall_stats: OverallStatsResponse,
    pub top_performing_rules: Vec<RulePerformance>,
    pub slow_rules: Vec<RulePerformance>,
    pub recent_activity: Vec<ExecutionEvent>,
    pub recommendations: Vec<String>,
    pub system_health: SystemHealth,
}

#[derive(Debug, Serialize)]
pub struct OverallStatsResponse {
    pub total_rules: usize,
    pub total_executions: u64,
    pub avg_execution_time_ms: f64,
    pub success_rate: f64,
    pub rules_per_second: f64,
    pub uptime_hours: f64,
}

#[derive(Debug, Serialize, Clone)]
pub struct RulePerformance {
    pub rule_name: String,
    pub avg_execution_time_ms: f64,
    pub success_rate: f64,
    pub total_executions: u64,
    pub total_fires: u64,
}

#[derive(Debug, Serialize)]
pub struct ExecutionEvent {
    pub rule_name: String,
    pub duration_ms: f64,
    pub success: bool,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct SystemHealth {
    pub status: String,
    pub analytics_enabled: bool,
    pub sampling_rate: f64,
    pub memory_usage_mb: f64,
    pub rules_count: usize,
}

/// Query parameters for analytics filtering
#[derive(Debug, Deserialize)]
pub struct AnalyticsQuery {
    pub limit: Option<usize>,
    pub rule_name: Option<String>,
    pub include_recommendations: Option<bool>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Rust Rule Engine REST API with Analytics Monitoring");

    // Create and configure the rule engine with analytics
    let engine = create_production_engine().await?;

    // Wrap engine in shared state
    let app_state = AppState {
        engine: Arc::new(Mutex::new(engine)),
    };

    // Build our application with routes
    let app = Router::new()
        // Rule execution endpoints
        .route("/api/v1/rules/execute", post(execute_rules))
        .route("/api/v1/rules/batch", post(execute_batch_rules))
        // Analytics endpoints
        .route("/api/v1/analytics/dashboard", get(get_analytics_dashboard))
        .route("/api/v1/analytics/stats", get(get_analytics_stats))
        .route(
            "/api/v1/analytics/rules/:rule_name",
            get(get_rule_analytics),
        )
        .route(
            "/api/v1/analytics/recommendations",
            get(get_recommendations),
        )
        .route("/api/v1/analytics/recent", get(get_recent_activity))
        // System endpoints
        .route("/api/v1/health", get(health_check))
        .route("/api/v1/status", get(system_status))
        .route("/", get(api_documentation))
        // Add CORS and other middleware
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .into_inner(),
        )
        .with_state(app_state);

    // Start the server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🌐 Server running on http://localhost:3000");
    println!("📊 Analytics Dashboard: http://localhost:3000/api/v1/analytics/dashboard");
    println!("📚 API Documentation: http://localhost:3000");

    axum::serve(listener, app).await?;
    Ok(())
}

/// Create a production-ready rule engine with analytics
async fn create_production_engine() -> Result<RustRuleEngine, Box<dyn std::error::Error>> {
    // Engine configuration với max_cycles = 1 để tránh infinite loop
    let engine_config = EngineConfig {
        max_cycles: 1, // CHỈ 1 cycle để an toàn
        timeout: Some(Duration::from_secs(10)),
        enable_stats: true,
        debug_mode: true, // Enable debug to track rule firing
    };

    // Create engine
    let mut engine = RustRuleEngine::with_config(
        rust_rule_engine::KnowledgeBase::new("ProductionAPI"),
        engine_config,
    );

    // Configure production analytics
    let analytics_config = AnalyticsConfig {
        track_execution_time: true,
        track_memory_usage: true,
        track_success_rate: true,
        sampling_rate: 0.9, // 90% sampling for demo purposes
        retention_period: Duration::from_secs(24 * 60 * 60), // 24 hours
        max_recent_samples: 1000,
    };

    let analytics = RuleAnalytics::new(analytics_config);
    engine.enable_analytics(analytics);

    // Register sample custom functions
    engine.register_function("calculateDiscount", |args, _facts| {
        let base_amount = args[0].as_number().unwrap_or(0.0);
        let discount_rate = args[1].as_number().unwrap_or(0.0);
        let discount = base_amount * discount_rate;
        Ok(Value::Number(discount))
    });

    engine.register_function("sendNotification", |args, _facts| {
        let message = args[0].as_string().unwrap_or("".to_string());
        let recipient = args[1].as_string().unwrap_or("unknown".to_string());
        tracing::info!("📧 Notification sent to {}: {}", recipient, message);
        Ok(Value::Boolean(true))
    });

    engine.register_function("logMessage", |args, _facts| {
        let message = args[0].as_string().unwrap_or("".to_string());
        tracing::info!("📋 LOG: {}", message);
        Ok(Value::Boolean(true))
    });

    // Register Customer methods for safe fact updates
    engine.register_function("Customer.setTier", |args, facts| {
        let new_tier = args[0].as_string().unwrap_or("STANDARD".to_string());

        // Safely update customer tier in facts
        if let Err(e) = facts.set_nested("Customer.Tier", Value::String(new_tier.clone())) {
            tracing::warn!("Failed to set Customer.Tier: {:?}", e);
        }

        tracing::info!("🏆 Customer tier updated to: {}", new_tier);
        Ok(Value::String(new_tier))
    });

    engine.register_function("updateUserTier", |args, _facts| {
        let tier = args[0].as_string().unwrap_or("STANDARD".to_string());
        tracing::info!("🏆 User tier updated to: {}", tier);
        Ok(Value::String(tier))
    });

    // Load sample rules
    load_sample_rules(&mut engine).await?;

    Ok(engine)
}

/// Load sample business rules for demonstration
async fn load_sample_rules(engine: &mut RustRuleEngine) -> Result<(), Box<dyn std::error::Error>> {
    let sample_rules = vec![
        // Rule phức tạp với nhiều điều kiện AND - chỉ fire 1 lần với max_cycles = 1
        r#"
rule "ComplexEligibilityCheck" salience 10 {
    when
        Customer.Age >= 18 && Customer.Age <= 65 && 
        Customer.IsNew == true && Customer.TotalSpent > 500.0 &&
        Customer.YearsActive < 5
    then
        sendNotification("Complex eligibility passed!", "admin@example.com");
        logMessage("Customer qualified for premium tier");
}
        "#,
        // Rule không match với data hiện tại (TotalSpent = 750 < 1000)
        r#"
rule "HighValueCustomer" salience 5 {
    when
        Customer.TotalSpent > 1000.0 && Customer.Age >= 21
    then
        sendNotification("High value customer detected!", "vip@example.com");
        logMessage("VIP tier assigned");
}
        "#,
        // Rule đơn giản sẽ match
        r#"
rule "BasicWelcome" salience 1 {
    when
        Customer.IsNew == true && Customer.Age >= 18
    then
        sendNotification("Welcome new customer!", "welcome@example.com");
        logMessage("Welcome message sent");
}
        "#,
    ];

    for rule_grl in sample_rules {
        println!("🔍 Parsing rule: {}", rule_grl);
        let rules = GRLParser::parse_rules(rule_grl)?;
        println!("✅ Parsed {} rules", rules.len());
        for rule in rules {
            println!("📝 Adding rule: {}", rule.name);
            engine.knowledge_base_mut().add_rule(rule)?;
        }
    }

    println!("✅ Loaded {} sample rules", 3);

    // Test rule execution with sample data
    println!("🧪 Testing compound conditions...");
    let facts = Facts::new();

    // Add test customer data with more comprehensive fields
    let customer_data = json!({
        "Age": 25,              // Đủ tuổi (18-65)
        "IsNew": true,          // Khách hàng mới
        "OrderCount": 3,        // Ít đơn hàng
        "TotalSpent": 750.0,    // Spending trung bình (>500 để qualify)
        "Email": "customer@example.com",
        "YearsActive": 2,       // < 5 years để qualify
        "Tier": "STANDARD",     // Tier hiện tại
        "processed": false,     // Chưa được xử lý
        "vipProcessed": false,  // Chưa được VIP process
        "welcomed": false       // Chưa được welcome
    });

    let order_data = json!({
        "Amount": 100.0,
        "CustomerEmail": "customer@example.com",
        "IsWeekend": false
    });

    facts.add_value("Customer", json_to_engine_value(customer_data))?;
    facts.add_value("Order", json_to_engine_value(order_data))?;

    tracing::info!("🔍 Adding fact: Customer = Object with Age, IsNew, OrderCount, etc.");
    tracing::info!("🔍 Adding fact: Order = Object with Amount, CustomerEmail, IsWeekend");
    tracing::info!("📊 About to execute rules with facts setup");

    // Execute rules to test compound conditions
    match engine.execute(&facts) {
        Ok(result) => {
            tracing::info!(
                "🎯 Execution result: fired={}, cycles={}",
                result.rules_fired,
                result.cycle_count
            );
        }
        Err(e) => {
            tracing::error!("❌ Execution failed: {:?}", e);
        }
    }

    Ok(())
}

/// Execute rules endpoint
async fn execute_rules(
    State(state): State<AppState>,
    Json(request): Json<ExecuteRulesRequest>,
) -> Result<ResponseJson<ExecuteRulesResponse>, StatusCode> {
    let mut engine = state
        .engine
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert request facts to engine facts
    let facts = Facts::new();
    for (key, value) in request.facts {
        let engine_value = json_to_engine_value(value);
        tracing::info!("🔍 Adding fact: {} = {:?}", key, engine_value);
        facts
            .add_value(&key, engine_value)
            .map_err(|_| StatusCode::BAD_REQUEST)?;
    }

    tracing::info!("📊 About to execute rules with facts setup");

    // Execute rules
    let result = engine
        .execute(&facts)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!(
        "🎯 Execution result: fired={}, cycles={}",
        result.rules_fired,
        result.cycle_count
    );

    // Convert updated facts back to JSON
    let updated_facts = facts_to_json(&facts);

    let response = ExecuteRulesResponse {
        success: true,
        rules_fired: result.rules_fired,
        cycle_count: result.cycle_count,
        execution_time_ms: result.execution_time.as_secs_f64() * 1000.0,
        updated_facts,
        message: format!(
            "Executed {} rules in {} cycles",
            result.rules_fired, result.cycle_count
        ),
    };

    Ok(ResponseJson(response))
}

/// Batch rule execution endpoint
async fn execute_batch_rules(
    State(state): State<AppState>,
    Json(requests): Json<Vec<ExecuteRulesRequest>>,
) -> Result<ResponseJson<Vec<ExecuteRulesResponse>>, StatusCode> {
    let mut responses = Vec::new();

    for request in requests {
        let mut engine = state
            .engine
            .lock()
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let facts = Facts::new();
        for (key, value) in request.facts {
            let engine_value = json_to_engine_value(value);
            facts
                .add_value(&key, engine_value)
                .map_err(|_| StatusCode::BAD_REQUEST)?;
        }

        let result = engine
            .execute(&facts)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let updated_facts = facts_to_json(&facts);

        responses.push(ExecuteRulesResponse {
            success: true,
            rules_fired: result.rules_fired,
            cycle_count: result.cycle_count,
            execution_time_ms: result.execution_time.as_secs_f64() * 1000.0,
            updated_facts,
            message: format!("Batch execution: {} rules fired", result.rules_fired),
        });
    }

    Ok(ResponseJson(responses))
}

/// Get comprehensive analytics dashboard
async fn get_analytics_dashboard(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<ResponseJson<AnalyticsDashboard>, StatusCode> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analytics = engine.analytics().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let stats = analytics.overall_stats();

    // Overall statistics
    let overall_stats = OverallStatsResponse {
        total_rules: stats.total_rules,
        total_executions: stats.total_evaluations,
        avg_execution_time_ms: stats.avg_execution_time.as_secs_f64() * 1000.0,
        success_rate: stats.success_rate,
        rules_per_second: stats.rules_per_second,
        uptime_hours: stats.uptime.as_secs_f64() / 3600.0,
    };

    // Top performing rules
    let top_performing = analytics
        .get_all_rule_metrics()
        .iter()
        .map(|(name, metrics)| RulePerformance {
            rule_name: name.clone(),
            avg_execution_time_ms: metrics.avg_execution_time().as_secs_f64() * 1000.0,
            success_rate: metrics.success_rate(),
            total_executions: metrics.total_evaluations,
            total_fires: metrics.total_fires,
        })
        .collect::<Vec<_>>();

    // Slow rules (sorted by execution time)
    let mut slow_rules = top_performing.clone();
    slow_rules.sort_by(|a, b| {
        b.avg_execution_time_ms
            .partial_cmp(&a.avg_execution_time_ms)
            .unwrap()
    });
    slow_rules.truncate(query.limit.unwrap_or(5));

    // Recent activity
    let recent_activity = analytics
        .get_recent_events(query.limit.unwrap_or(10))
        .into_iter()
        .map(|event| ExecutionEvent {
            rule_name: event.rule_name.clone(),
            duration_ms: event.duration.as_secs_f64() * 1000.0,
            success: event.success,
            timestamp: format!("{:?}", event.timestamp),
        })
        .collect();

    // Recommendations
    let recommendations = if query.include_recommendations.unwrap_or(true) {
        analytics.generate_recommendations()
    } else {
        vec![]
    };

    // System health
    let system_health = SystemHealth {
        status: "healthy".to_string(),
        analytics_enabled: true,
        sampling_rate: analytics.config().sampling_rate,
        memory_usage_mb: 0.0, // Would calculate actual memory usage in production
        rules_count: stats.total_rules,
    };

    let dashboard = AnalyticsDashboard {
        overall_stats,
        top_performing_rules: top_performing,
        slow_rules,
        recent_activity,
        recommendations,
        system_health,
    };

    Ok(ResponseJson(dashboard))
}

/// Get overall analytics statistics
async fn get_analytics_stats(
    State(state): State<AppState>,
) -> Result<ResponseJson<OverallStatsResponse>, StatusCode> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analytics = engine.analytics().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let stats = analytics.overall_stats();

    let response = OverallStatsResponse {
        total_rules: stats.total_rules,
        total_executions: stats.total_evaluations,
        avg_execution_time_ms: stats.avg_execution_time.as_secs_f64() * 1000.0,
        success_rate: stats.success_rate,
        rules_per_second: stats.rules_per_second,
        uptime_hours: stats.uptime.as_secs_f64() / 3600.0,
    };

    Ok(ResponseJson(response))
}

/// Get analytics for a specific rule
async fn get_rule_analytics(
    State(state): State<AppState>,
    Path(rule_name): Path<String>,
) -> Result<ResponseJson<RulePerformance>, StatusCode> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analytics = engine.analytics().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let metrics = analytics
        .get_rule_metrics(&rule_name)
        .ok_or(StatusCode::NOT_FOUND)?;

    let performance = RulePerformance {
        rule_name: rule_name.clone(),
        avg_execution_time_ms: metrics.avg_execution_time().as_secs_f64() * 1000.0,
        success_rate: metrics.success_rate(),
        total_executions: metrics.total_evaluations,
        total_fires: metrics.total_fires,
    };

    Ok(ResponseJson(performance))
}

/// Get optimization recommendations
async fn get_recommendations(
    State(state): State<AppState>,
) -> Result<ResponseJson<Vec<String>>, StatusCode> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analytics = engine.analytics().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let recommendations = analytics.generate_recommendations();

    Ok(ResponseJson(recommendations))
}

/// Get recent execution activity
async fn get_recent_activity(
    State(state): State<AppState>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<ResponseJson<Vec<ExecutionEvent>>, StatusCode> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let analytics = engine.analytics().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;

    let events = analytics
        .get_recent_events(query.limit.unwrap_or(20))
        .into_iter()
        .map(|event| ExecutionEvent {
            rule_name: event.rule_name.clone(),
            duration_ms: event.duration.as_secs_f64() * 1000.0,
            success: event.success,
            timestamp: format!("{:?}", event.timestamp),
        })
        .collect();

    Ok(ResponseJson(events))
}

/// Health check endpoint
async fn health_check() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// System status endpoint
async fn system_status(
    State(state): State<AppState>,
) -> Result<ResponseJson<SystemHealth>, StatusCode> {
    let engine = state
        .engine
        .lock()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let (analytics_enabled, sampling_rate, rules_count) =
        if let Some(analytics) = engine.analytics() {
            (
                true,
                analytics.config().sampling_rate,
                analytics.get_all_rule_metrics().len(),
            )
        } else {
            (false, 0.0, 0)
        };

    let status = SystemHealth {
        status: "operational".to_string(),
        analytics_enabled,
        sampling_rate,
        memory_usage_mb: 0.0, // Would implement actual memory tracking
        rules_count,
    };

    Ok(ResponseJson(status))
}

/// API documentation endpoint
async fn api_documentation() -> ResponseJson<serde_json::Value> {
    ResponseJson(serde_json::json!({
        "name": "Rust Rule Engine REST API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Production-ready rule engine with advanced analytics monitoring",
        "features": [
            "Real-time rule execution",
            "Advanced analytics monitoring",
            "Performance optimization insights",
            "Batch processing",
            "Health monitoring"
        ],
        "endpoints": {
            "rule_execution": {
                "POST /api/v1/rules/execute": "Execute rules with provided facts",
                "POST /api/v1/rules/batch": "Execute rules in batch mode"
            },
            "analytics": {
                "GET /api/v1/analytics/dashboard": "Comprehensive analytics dashboard",
                "GET /api/v1/analytics/stats": "Overall performance statistics",
                "GET /api/v1/analytics/rules/{rule_name}": "Rule-specific analytics",
                "GET /api/v1/analytics/recommendations": "Optimization recommendations",
                "GET /api/v1/analytics/recent": "Recent execution activity"
            },
            "system": {
                "GET /api/v1/health": "Health check",
                "GET /api/v1/status": "System status"
            }
        },
        "example_request": {
            "url": "/api/v1/rules/execute",
            "method": "POST",
            "body": {
                "facts": {
                    "Customer": {
                        "TotalSpent": 15000.0,
                        "OrderCount": 75,
                        "Email": "customer@example.com",
                        "YearsActive": 3,
                        "IsNew": false,
                        "Age": 35
                    },
                    "Order": {
                        "Amount": 750.0,
                        "CustomerEmail": "customer@example.com"
                    }
                }
            }
        },
        "analytics_dashboard": "http://localhost:3000/api/v1/analytics/dashboard"
    }))
}

/// Helper function to convert JSON values to engine values
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
        _ => Value::String("null".to_string()),
    }
}

/// Helper function to convert facts back to JSON
fn facts_to_json(_facts: &Facts) -> HashMap<String, serde_json::Value> {
    // In a real implementation, you'd iterate through facts
    // For now, return empty map as Facts doesn't expose iteration
    HashMap::new()
}

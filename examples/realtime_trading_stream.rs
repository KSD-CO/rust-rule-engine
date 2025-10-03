// Real-time Trading Stream Example
//
// This example demonstrates the streaming rule engine processing
// real-time trading events with sophisticated rule evaluation.

#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::engine::StreamConfig;
#[cfg(feature = "streaming")]
use rust_rule_engine::streaming::*;
#[cfg(feature = "streaming")]
use rust_rule_engine::types::Value;
#[cfg(feature = "streaming")]
use std::collections::HashMap;
#[cfg(feature = "streaming")]
use std::time::Duration;
#[cfg(feature = "streaming")]
use tokio::time::{interval, sleep};

#[cfg(feature = "streaming")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Real-time Trading Stream Processing Demo");
    println!("==========================================\n");

    // Create streaming engine with custom config
    let config = StreamConfig {
        buffer_size: 5000,
        window_duration: Duration::from_secs(30), // 30-second windows
        max_events_per_window: 500,
        max_windows: 20,
        window_type: WindowType::Sliding,
        analytics_cache_ttl_ms: 15000,
        processing_interval: Duration::from_millis(500),
    };

    let mut engine = StreamRuleEngine::with_config(config);

    // Add sophisticated trading rules
    add_trading_rules(&mut engine).await?;

    // Register action handlers
    register_action_handlers(&engine).await;

    // Start the streaming engine
    println!("📡 Starting stream processing...");
    engine.start().await?;

    // Wrap engine in Arc for sharing
    let engine = std::sync::Arc::new(engine);

    // Simulate real-time trading events
    println!("💹 Generating trading events...\n");

    // Run trading simulation for 2 minutes
    let simulation_task = tokio::spawn(simulate_trading_events(engine.clone()));
    let monitoring_task = tokio::spawn(monitor_engine_metrics(engine.clone()));

    // Wait for simulation to complete
    tokio::select! {
        _ = simulation_task => println!("Simulation completed"),
        _ = monitoring_task => println!("Monitoring completed"),
        _ = tokio::signal::ctrl_c() => println!("Received shutdown signal"),
    }

    engine.stop().await;
    println!("\n✅ Stream processing demo completed!");

    Ok(())
}

#[cfg(feature = "streaming")]
async fn add_trading_rules(
    engine: &mut StreamRuleEngine,
) -> Result<(), Box<dyn std::error::Error>> {
    // Rule 1: High frequency trading detection
    let high_frequency_rule = r#"
    rule "HighFrequencyAlert" salience 100 {
        when
            WindowEventCount > 50 && WindowDurationMs <= 30000
        then
            AlertService.trigger("High frequency trading detected");
            log("🚨 HIGH FREQUENCY: " + WindowEventCount + " trades in " + WindowDurationMs + "ms");
    }
    "#;

    // Rule 2: Price volatility detection
    let volatility_rule = r#"
    rule "VolatilityAlert" salience 90 {
        when
            priceMax - priceMin > 10.0 && WindowEventCount > 5
        then
            AlertService.trigger("High volatility detected");
            log("📊 VOLATILITY: Price range $" + (priceMax - priceMin));
    }
    "#;

    // Rule 3: Large trade detection
    let large_trade_rule = r#"
    rule "LargeTradeAlert" salience 80 {
        when
            volumeMax > 1000000
        then
            AlertService.trigger("Large trade detected");
            log("💰 LARGE TRADE: Volume " + volumeMax);
    }
    "#;

    // Rule 4: Trend analysis
    let trend_rule = r#"
    rule "TrendAnalysis" salience 70 {
        when
            priceAverage > 100.0 && volumeAverage > 50000
        then
            TrendService.updateTrend("bullish");
            log("📈 TREND: Bullish trend detected - Avg Price: $" + priceAverage);
    }
    "#;

    // Rule 5: Market circuit breaker
    let circuit_breaker_rule = r#"
    rule "CircuitBreaker" salience 200 {
        when
            priceMax > 200.0 || priceMin < 50.0
        then
            MarketService.halt("circuit_breaker");
            log("🛑 CIRCUIT BREAKER: Extreme price movement detected");
    }
    "#;

    engine.add_rule(high_frequency_rule).await?;
    engine.add_rule(volatility_rule).await?;
    engine.add_rule(large_trade_rule).await?;
    engine.add_rule(trend_rule).await?;
    engine.add_rule(circuit_breaker_rule).await?;

    println!("✅ Added 5 sophisticated trading rules");
    Ok(())
}

#[cfg(feature = "streaming")]
async fn register_action_handlers(engine: &StreamRuleEngine) {
    // Alert service handler
    engine
        .register_action_handler("AlertService", |action| {
            if let Some(message) = action.parameters.get("message") {
                match message {
                    Value::String(msg) => {
                        println!("🚨 ALERT [{}]: {}", action.rule_name, msg);
                    }
                    _ => println!("🚨 ALERT [{}]: Unknown message type", action.rule_name),
                }
            }
        })
        .await;

    // Trend service handler
    engine
        .register_action_handler("TrendService", |action| {
            if let Some(trend) = action.parameters.get("trend") {
                match trend {
                    Value::String(t) => {
                        println!(
                            "📈 TREND UPDATE [{}]: Market trend is {}",
                            action.rule_name, t
                        );
                    }
                    _ => println!("📈 TREND [{}]: Trend data received", action.rule_name),
                }
            }
        })
        .await;

    // Market service handler
    engine
        .register_action_handler("MarketService", |action| {
            if let Some(action_type) = action.parameters.get("action") {
                match action_type {
                    Value::String(a) => {
                        println!(
                            "🛑 MARKET ACTION [{}]: {}",
                            action.rule_name,
                            a.to_uppercase()
                        );
                    }
                    _ => println!("🛑 MARKET [{}]: Action triggered", action.rule_name),
                }
            }
        })
        .await;

    // Log handler
    engine
        .register_action_handler("log", |action| {
            if let Some(message) = action.parameters.get("message") {
                match message {
                    Value::String(msg) => {
                        println!("📝 LOG [{}]: {}", action.rule_name, msg);
                    }
                    _ => println!("📝 LOG [{}]: Event logged", action.rule_name),
                }
            }
        })
        .await;

    println!("✅ Registered action handlers for alerts, trends, and market actions");
}

#[cfg(feature = "streaming")]
async fn simulate_trading_events(engine: std::sync::Arc<StreamRuleEngine>) {
    let mut interval = interval(Duration::from_millis(100)); // 10 events per second
    let mut event_count = 0;
    let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN"];

    for _ in 0..1200 {
        // Run for 2 minutes (1200 events)
        interval.tick().await;

        // Create realistic trading event
        let symbol = &symbols[fastrand::usize(0..symbols.len())];
        let base_price = match *symbol {
            "AAPL" => 150.0,
            "GOOGL" => 2800.0,
            "MSFT" => 300.0,
            "TSLA" => 800.0,
            "AMZN" => 3200.0,
            _ => 100.0,
        };

        // Add some volatility
        let price_variation = (fastrand::f64() - 0.5) * 20.0; // ±$10 variation
        let price = base_price + price_variation;

        // Occasional large trades
        let volume = if fastrand::f64() < 0.05 {
            // 5% chance of large trade
            fastrand::u32(500000..2000000) as f64
        } else {
            fastrand::u32(1000..100000) as f64
        };

        let mut data = HashMap::new();
        data.insert("symbol".to_string(), Value::String(symbol.to_string()));
        data.insert("price".to_string(), Value::Number(price));
        data.insert("volume".to_string(), Value::Number(volume));
        data.insert(
            "side".to_string(),
            Value::String(if fastrand::bool() { "buy" } else { "sell" }.to_string()),
        );

        let event = StreamEvent::new("TradeEvent", data, "trading_exchange");

        if let Err(e) = engine.send_event(event).await {
            eprintln!("Error sending event: {}", e);
            break;
        }

        event_count += 1;

        // Occasional burst of activity (simulating market news)
        if fastrand::f64() < 0.02 {
            // 2% chance of burst
            for _ in 0..fastrand::usize(5..20) {
                let mut burst_data = HashMap::new();
                burst_data.insert("symbol".to_string(), Value::String(symbol.to_string()));
                burst_data.insert(
                    "price".to_string(),
                    Value::Number(price + fastrand::f64() * 5.0),
                );
                burst_data.insert(
                    "volume".to_string(),
                    Value::Number(fastrand::u32(10000..200000) as f64),
                );
                burst_data.insert("side".to_string(), Value::String("buy".to_string()));

                let burst_event = StreamEvent::new("TradeEvent", burst_data, "trading_exchange");
                let _ = engine.send_event(burst_event).await;

                sleep(Duration::from_millis(10)).await;
            }
        }
    }

    println!("📊 Generated {} trading events", event_count);
}

#[cfg(feature = "streaming")]
async fn monitor_engine_metrics(engine: std::sync::Arc<StreamRuleEngine>) {
    let mut interval = interval(Duration::from_secs(10));

    for i in 0..12 {
        // Monitor for 2 minutes
        interval.tick().await;

        // Execute rules and get results
        match engine.execute_rules().await {
            Ok(result) => {
                println!("\n📊 METRICS UPDATE {} ({}0s):", i + 1, i + 1);
                println!("   Rules Fired: {}", result.rules_fired);
                println!("   Events Processed: {}", result.events_processed);
                println!("   Processing Time: {}ms", result.processing_time_ms);
                println!("   Actions Triggered: {}", result.actions.len());

                // Display analytics
                for (key, value) in &result.analytics {
                    match value {
                        Value::Number(n) => println!("   {}: {:.2}", key, n),
                        Value::String(s) => println!("   {}: {}", key, s),
                        _ => println!("   {}: {:?}", key, value),
                    }
                }
            }
            Err(e) => {
                eprintln!("Error executing rules: {}", e);
            }
        }

        // Get window statistics
        let stats = engine.get_window_statistics().await;
        println!("   Active Windows: {}", stats.total_windows);
        println!(
            "   Avg Events/Window: {:.1}",
            stats.average_events_per_window
        );

        // Get field-specific analytics
        let price_analytics = engine.get_field_analytics("price").await;
        if let Some(Value::Number(avg)) = price_analytics.get("overall_average") {
            println!("   Avg Price: ${:.2}", avg);
        }
        if let Some(Value::Number(min)) = price_analytics.get("global_min") {
            if let Some(Value::Number(max)) = price_analytics.get("global_max") {
                println!("   Price Range: ${:.2} - ${:.2}", min, max);
            }
        }

        println!("   ────────────────────────────────");
    }
}

// Fallback for non-streaming builds
#[cfg(not(feature = "streaming"))]
fn main() {
    println!("❌ Streaming feature not enabled!");
    println!("   To run this example, use: cargo run --example realtime_trading_stream --features streaming");
}

# 🌊 Streaming Rule Engine

Real-time event processing with sophisticated rule evaluation capabilities.

## Features

- **🔄 Continuous Processing**: Non-stop rule evaluation on streaming data
- **⏰ Time Windows**: Sliding and tumbling window aggregations  
- **📊 Stream Analytics**: Count, sum, average, min/max over time windows
- **🎯 Event Filtering**: Pattern matching and event correlation
- **⚡ High Throughput**: Async processing with backpressure handling
- **🚨 Real-time Alerts**: Immediate action triggering based on conditions

## Quick Start

### Enable Streaming Feature

Add to your `Cargo.toml`:

```toml
[dependencies]
rust-rule-engine = { version = "0.1.4", features = ["streaming"] }
```

### Basic Usage

```rust
use rust_rule_engine::streaming::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create streaming engine
    let mut engine = StreamRuleEngine::new();
    
    // Add streaming rule
    let rule = r#"
    rule "HighVolumeAlert" salience 10 {
        when
            WindowEventCount > 100 && volumeSum > 1000000
        then
            AlertService.trigger("High volume detected");
    }
    "#;
    
    engine.add_rule(rule).await?;
    
    // Register action handler
    engine.register_action_handler("AlertService", |action| {
        println!("🚨 Alert: {:?}", action.parameters);
    }).await;
    
    // Start processing
    engine.start().await?;
    
    // Send events
    let event = StreamEvent::new("TradeEvent", data, "exchange");
    engine.send_event(event).await?;
    
    Ok(())
}
```

## Advanced Configuration

```rust
let config = StreamConfig {
    buffer_size: 10000,                          // Event buffer size
    window_duration: Duration::from_secs(60),    // 60-second windows
    max_events_per_window: 1000,                 // Max events per window
    max_windows: 100,                            // Keep 100 windows
    window_type: WindowType::Sliding,            // Sliding windows
    analytics_cache_ttl_ms: 30000,               // 30s cache TTL
    processing_interval: Duration::from_millis(100), // Process every 100ms
};

let engine = StreamRuleEngine::with_config(config);
```

## Window Types

### Sliding Windows
```rust
// Continuously moving windows
WindowType::Sliding
```

### Tumbling Windows  
```rust
// Non-overlapping fixed intervals
WindowType::Tumbling
```

### Session Windows
```rust
// Based on inactivity gaps
WindowType::Session { 
    timeout: Duration::from_secs(300) 
}
```

## Stream Aggregations

The engine automatically provides these aggregations in rule conditions:

### Window Statistics
- `WindowEventCount` - Number of events in window
- `WindowStartTime` - Window start timestamp
- `WindowEndTime` - Window end timestamp  
- `WindowDurationMs` - Window duration in milliseconds

### Field Aggregations
For any numeric field `price`:
- `priceSum` - Sum of all price values
- `priceAverage` - Average price
- `priceMin` - Minimum price
- `priceMax` - Maximum price

Example rule using aggregations:
```rust
rule "PriceVolatility" {
    when
        priceMax - priceMin > 10.0 && WindowEventCount > 20
    then
        AlertService.trigger("High price volatility");
}
```

## Event Processing

### Creating Events
```rust
use std::collections::HashMap;

let mut data = HashMap::new();
data.insert("symbol".to_string(), Value::String("AAPL".to_string()));
data.insert("price".to_string(), Value::Number(150.50));
data.insert("volume".to_string(), Value::Number(10000.0));

let event = StreamEvent::new("TradeEvent", data, "nasdaq");
```

### Event Metadata
```rust
// Access event metadata
println!("Event ID: {}", event.id);
println!("Timestamp: {}", event.metadata.timestamp);
println!("Source: {}", event.metadata.source);
println!("Age: {}ms", event.age_ms());
```

### Event Pattern Matching
```rust
let pattern = EventPattern::new()
    .with_event_type("TradeEvent")
    .with_field("symbol", Value::String("AAPL".to_string()))
    .with_source("nasdaq");

if event.matches_pattern(&pattern) {
    println!("Event matches trading pattern");
}
```

## Action Handlers

Register custom handlers for rule actions:

```rust
// Alert handler
engine.register_action_handler("AlertService", |action| {
    match action.parameters.get("level") {
        Some(Value::String(level)) => {
            println!("🚨 {} Alert: {}", level.to_uppercase(), action.rule_name);
        }
        _ => println!("🚨 Alert triggered: {}", action.rule_name),
    }
}).await;

// Trading handler
engine.register_action_handler("TradingService", |action| {
    if let Some(Value::String(action_type)) = action.parameters.get("action") {
        match action_type.as_str() {
            "buy" => println!("📈 Executing BUY order"),
            "sell" => println!("📉 Executing SELL order"), 
            "halt" => println!("🛑 Halting trading"),
            _ => println!("🔄 Unknown trading action"),
        }
    }
}).await;
```

## Real-world Example

See `examples/realtime_trading_stream.rs` for a complete trading system:

```bash
# Run with streaming feature
cargo run --example realtime_trading_stream --features streaming
```

This example demonstrates:
- High-frequency trading detection
- Price volatility monitoring
- Large trade alerts
- Trend analysis
- Circuit breaker triggers
- Real-time metrics and monitoring

## Performance Tips

1. **Batch Processing**: Events are automatically batched for efficiency
2. **Window Limits**: Set appropriate `max_events_per_window` to prevent memory issues
3. **Cache TTL**: Use analytics cache for expensive calculations
4. **Buffer Size**: Increase `buffer_size` for high-throughput scenarios
5. **Processing Interval**: Balance latency vs. throughput with `processing_interval`

## Monitoring

```rust
// Get execution metrics
let result = engine.execute_rules().await?;
println!("Rules fired: {}", result.rules_fired);
println!("Events processed: {}", result.events_processed);
println!("Processing time: {}ms", result.processing_time_ms);

// Get window statistics
let stats = engine.get_window_statistics().await;
println!("Active windows: {}", stats.total_windows);
println!("Total events: {}", stats.total_events);

// Get field analytics
let analytics = engine.get_field_analytics("price").await;
if let Some(Value::Number(avg)) = analytics.get("overall_average") {
    println!("Average price: ${:.2}", avg);
}
```

## Integration Examples

### Financial Trading
- Real-time trade monitoring
- Risk management alerts
- Market volatility detection
- Regulatory compliance

### IoT Monitoring  
- Sensor data analysis
- Anomaly detection
- Predictive maintenance
- Resource optimization

### Log Processing
- Error rate monitoring
- Performance tracking
- Security event detection
- System health checks

### E-commerce
- Fraud detection
- Inventory alerts
- Customer behavior analysis
- Promotional triggers

The streaming rule engine provides powerful real-time capabilities while maintaining the simplicity and performance of the core rule engine.

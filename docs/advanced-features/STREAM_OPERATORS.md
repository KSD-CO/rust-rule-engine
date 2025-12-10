# 🌊 Stream Operators - Fluent API Guide

A powerful, composable API for building stream processing pipelines in Rust. Inspired by Apache Flink, Kafka Streams, and functional programming patterns.

## Table of Contents

- [Overview](#overview)
- [Core Concepts](#core-concepts)
- [Basic Operators](#basic-operators)
- [Advanced Operators](#advanced-operators)
- [Windowing](#windowing)
- [Aggregations](#aggregations)
- [Real-World Examples](#real-world-examples)
- [Performance Tips](#performance-tips)

## Overview

Stream Operators provide a fluent, chainable API for processing event streams with functional-style transformations, aggregations, and windowing operations.

### Key Features

- ✅ **Fluent API**: Chain operations naturally like Rust iterators
- ✅ **Type-Safe**: Leverages Rust's type system for compile-time safety
- ✅ **Zero-Copy**: Efficient event processing with minimal allocations
- ✅ **Composable**: Build complex pipelines from simple operators
- ✅ **Functional**: Map, filter, reduce, and more
- ✅ **Windowing**: Time-based aggregations (sliding, tumbling, session)
- ✅ **Key-By**: Partition streams by key for parallel processing

## Core Concepts

### DataStream

The fundamental abstraction for a stream of events:

```rust
use rust_rule_engine::streaming::*;

// Create from events
let stream = DataStream::from_events(events);

// Create empty and build up
let mut stream = DataStream::new();
stream.push(event);
```

### KeyedStream

A stream partitioned by a key field:

```rust
let keyed = stream.key_by(|e| e.get_string("user_id").unwrap_or("").to_string());
```

### WindowedStream

A stream with time-based windowing applied:

```rust
let windowed = stream.window(WindowConfig::tumbling(Duration::from_secs(60)));
```

## Basic Operators

### Filter

Keep only events matching a predicate:

```rust
let filtered = stream
    .filter(|e| e.get_numeric("amount").unwrap_or(0.0) > 100.0);
```

**Use Cases:**
- Remove invalid data
- Focus on specific event types
- Apply business rules

### Map

Transform each event:

```rust
let transformed = stream
    .map(|mut e| {
        // Add computed field
        if let Some(price) = e.get_numeric("price") {
            let tax = price * 0.1;
            e.data.insert("tax".to_string(), Value::Number(tax));
        }
        e
    });
```

**Use Cases:**
- Enrich events
- Format data
- Add derived fields

### FlatMap

Transform each event into multiple events:

```rust
let expanded = stream
    .flat_map(|e| {
        // Split one event into multiple
        let mut events = Vec::new();
        if let Some(items) = e.get_array("items") {
            for item in items {
                let mut new_event = e.clone();
                new_event.data.insert("item".to_string(), item.clone());
                events.push(new_event);
            }
        }
        events
    });
```

**Use Cases:**
- Unnesting arrays
- Event explosion
- Normalization

### ForEach

Execute side effects without modifying the stream:

```rust
let stream = stream
    .for_each(|e| {
        println!("Processing: {:?}", e);
        // Log, send metrics, etc.
    });
```

**Use Cases:**
- Logging
- Metrics collection
- Debugging

## Advanced Operators

### Key-By

Partition stream by a key for grouped operations:

```rust
let keyed = stream
    .key_by(|e| e.get_string("user_id").unwrap_or("unknown").to_string());

// Then aggregate per key
let totals = keyed.aggregate(Sum::new("amount"));
```

**Use Cases:**
- Per-user analytics
- Session tracking
- Grouped aggregations

### Reduce

Combine events into a single result:

```rust
let total = stream
    .reduce(|mut acc, e| {
        let acc_val = acc.get_numeric("total").unwrap_or(0.0);
        let e_val = e.get_numeric("amount").unwrap_or(0.0);
        acc.data.insert("total".to_string(), Value::Number(acc_val + e_val));
        acc
    });
```

**Use Cases:**
- Running totals
- Accumulation
- Finding max/min

### Group-By

Group events by a key and apply operations:

```rust
let grouped = stream
    .group_by(|e| e.get_string("category").unwrap_or("").to_string());

// Count per group
let counts = grouped.count();

// Aggregate per group
let averages = grouped.aggregate(Average::new("price"));
```

**Use Cases:**
- Category analysis
- Cohort grouping
- Distribution analysis

### Union

Combine two streams:

```rust
let combined = stream1.union(stream2);
```

**Use Cases:**
- Merge multiple sources
- Combine filtered results
- Data consolidation

### Take / Skip

Limit stream size:

```rust
let first_10 = stream.take(10);
let skip_first_5 = stream.skip(5);
```

**Use Cases:**
- Sampling
- Pagination
- Testing

### Sort

Order events by a key:

```rust
let sorted = stream
    .sort_by(|e| e.metadata.timestamp);
```

**Use Cases:**
- Time ordering
- Priority sorting
- Ranking

## Windowing

Apply time-based windows for aggregations over time ranges.

### Tumbling Windows

Non-overlapping fixed-size windows:

```rust
let windowed = stream
    .window(WindowConfig::tumbling(Duration::from_secs(60)));

// Aggregate within each window
let results = windowed.aggregate(Sum::new("amount"));
```

**Use Cases:**
- Hourly/daily summaries
- Batch processing
- Report generation

**Example:**
```
Events: |--A--B--|--C--D--|--E--F--|
Window:    [W1]     [W2]     [W3]
```

### Sliding Windows

Overlapping windows that slide forward:

```rust
let windowed = stream
    .window(WindowConfig::sliding(Duration::from_secs(60)));
```

**Use Cases:**
- Moving averages
- Trend detection
- Smoothing

**Example:**
```
Events: |--A--B--C--D--E--F--|
Window:    [----W1----]
              [----W2----]
                 [----W3----]
```

### Session Windows

Windows based on inactivity gaps:

```rust
let windowed = stream
    .window(WindowConfig::session(Duration::from_secs(300)));
```

**Use Cases:**
- User sessions
- Activity bursts
- Click streams

### Window Configuration

Customize window behavior:

```rust
let config = WindowConfig::tumbling(Duration::from_secs(60))
    .with_max_events(10000);  // Limit events per window
```

### Window Operations

```rust
// Count per window
let counts = windowed.counts();

// Aggregate per window
let sums = windowed.aggregate(Sum::new("amount"));

// Reduce per window
let results = windowed.reduce(|acc, e| /* ... */);

// Flatten back to stream
let flattened = windowed.flatten();
```

## Aggregations

Built-in aggregation functions for stream analysis.

### Count

Count events:

```rust
let count = stream.count();

// Or as aggregation
let result = stream.aggregate(Count);
```

### Sum

Sum numeric values:

```rust
let total = stream.aggregate(Sum::new("amount"));
```

### Average

Calculate average:

```rust
let avg = stream.aggregate(Average::new("price"));
```

### Min / Max

Find minimum or maximum:

```rust
let min_price = stream.aggregate(Min::new("price"));
let max_price = stream.aggregate(Max::new("price"));
```

### Custom Aggregator

Create custom aggregation logic:

```rust
let custom = CustomAggregator::new(|events: &[StreamEvent]| {
    // Your custom logic
    let values: Vec<f64> = events
        .iter()
        .filter_map(|e| e.get_numeric("value"))
        .collect();
    
    let sum: f64 = values.iter().sum();
    let avg = sum / values.len() as f64;
    
    let mut result = HashMap::new();
    result.insert("sum".to_string(), Value::Number(sum));
    result.insert("avg".to_string(), Value::Number(avg));
    
    AggregateResult::Map(result)
});

let result = stream.aggregate(custom);
```

## Real-World Examples

### Example 1: E-Commerce Analytics

Calculate revenue per category with discounts:

```rust
let revenue_by_category = DataStream::from_events(transactions)
    .filter(|e| e.get_string("status") == Some("completed"))
    .map(|mut e| {
        // Apply discount
        if let Some(amount) = e.get_numeric("amount") {
            let discount = e.get_numeric("discount").unwrap_or(0.0);
            let final_amount = amount * (1.0 - discount);
            e.data.insert("final_amount".to_string(), Value::Number(final_amount));
        }
        e
    })
    .key_by(|e| e.get_string("category").unwrap_or("").to_string())
    .aggregate(Sum::new("final_amount"));

for (category, total) in revenue_by_category {
    if let Some(amount) = total.as_number() {
        println!("Category {}: ${:.2}", category, amount);
    }
}
```

### Example 2: IoT Sensor Monitoring

Detect temperature anomalies in real-time:

```rust
let alerts = DataStream::from_events(sensor_readings)
    .window(WindowConfig::tumbling(Duration::from_secs(60)))
    .aggregate(CustomAggregator::new(|events| {
        let temps: Vec<f64> = events
            .iter()
            .filter_map(|e| e.get_numeric("temperature"))
            .collect();
        
        let avg = temps.iter().sum::<f64>() / temps.len() as f64;
        let max = temps.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        let anomaly = max > avg + 20.0;  // 20°C spike
        
        let mut result = HashMap::new();
        result.insert("has_anomaly".to_string(), Value::Boolean(anomaly));
        result.insert("max_temp".to_string(), Value::Number(max));
        
        AggregateResult::Map(result)
    }));
```

### Example 3: User Session Analysis

Track user behavior patterns:

```rust
let session_stats = DataStream::from_events(clickstream)
    .filter(|e| e.event_type == "PageView")
    .key_by(|e| e.get_string("user_id").unwrap_or("").to_string())
    .window(WindowConfig::session(Duration::from_secs(1800)))  // 30 min timeout
    .aggregate(CustomAggregator::new(|events| {
        let pages_viewed = events.len();
        let duration = if !events.is_empty() {
            let first = events.first().unwrap().metadata.timestamp;
            let last = events.last().unwrap().metadata.timestamp;
            last - first
        } else {
            0
        };
        
        let mut result = HashMap::new();
        result.insert("pages".to_string(), Value::Number(pages_viewed as f64));
        result.insert("duration_ms".to_string(), Value::Number(duration as f64));
        
        AggregateResult::Map(result)
    }));
```

### Example 4: Fraud Detection

Detect suspicious transaction patterns:

```rust
let suspicious = DataStream::from_events(transactions)
    .key_by(|e| e.get_string("user_id").unwrap_or("").to_string())
    .window(WindowConfig::sliding(Duration::from_secs(300)))  // 5 min window
    .aggregate(CustomAggregator::new(|events| {
        let count = events.len();
        let total: f64 = events
            .iter()
            .filter_map(|e| e.get_numeric("amount"))
            .sum();
        
        // Flag if: >10 transactions OR total > $5000 in 5 minutes
        let is_suspicious = count > 10 || total > 5000.0;
        
        let mut result = HashMap::new();
        result.insert("suspicious".to_string(), Value::Boolean(is_suspicious));
        result.insert("tx_count".to_string(), Value::Number(count as f64));
        result.insert("total".to_string(), Value::Number(total));
        
        AggregateResult::Map(result)
    }));
```

## Performance Tips

### 1. Use Filter Early

Filter unwanted events as early as possible in the pipeline:

```rust
// ✅ Good - filter first
stream
    .filter(|e| e.event_type == "Purchase")
    .map(|e| expensive_transformation(e))
    .aggregate(Sum::new("amount"));

// ❌ Bad - transform everything
stream
    .map(|e| expensive_transformation(e))
    .filter(|e| e.event_type == "Purchase")
    .aggregate(Sum::new("amount"));
```

### 2. Avoid Unnecessary Clones

Use references when possible:

```rust
// ✅ Good - no clone in filter
stream.filter(|e| e.get_numeric("amount").unwrap_or(0.0) > 100.0)

// ❌ Bad - unnecessary clone
stream.filter(|e| {
    let cloned = e.clone();  // Unnecessary!
    cloned.get_numeric("amount").unwrap_or(0.0) > 100.0
})
```

### 3. Batch Operations

Process events in batches when possible:

```rust
// Use windowing to batch events
stream
    .window(WindowConfig::tumbling(Duration::from_secs(1)))
    .aggregate(custom_batch_aggregator)
```

### 4. Limit Data Size

Use `take()` for testing or sampling:

```rust
stream
    .take(1000)  // Process only first 1000
    .aggregate(Sum::new("amount"))
```

### 5. Choose Appropriate Window Types

- **Tumbling**: Best for non-overlapping summaries (lowest memory)
- **Sliding**: For moving averages (higher memory, overlapping results)
- **Session**: For user behavior (variable window size)

### 6. Key-By Cardinality

Be mindful of key cardinality when using `key_by()`:

```rust
// ✅ Good - low cardinality (user_ids)
stream.key_by(|e| e.get_string("user_id").unwrap_or("").to_string())

// ⚠️  Caution - high cardinality (unique event IDs)
stream.key_by(|e| e.id.clone())  // Creates too many groups!
```

## API Reference

### DataStream Methods

| Method | Description | Returns |
|--------|-------------|---------|
| `filter(predicate)` | Keep events matching predicate | `DataStream` |
| `map(mapper)` | Transform each event | `DataStream` |
| `flat_map(mapper)` | Transform to multiple events | `DataStream` |
| `key_by(selector)` | Partition by key | `KeyedStream<K>` |
| `window(config)` | Apply time window | `WindowedStream` |
| `reduce(reducer)` | Combine to single result | `Option<StreamEvent>` |
| `group_by(selector)` | Group events | `GroupedStream<K>` |
| `aggregate(aggregator)` | Apply aggregation | `AggregateResult` |
| `for_each(action)` | Execute side effect | `DataStream` |
| `union(other)` | Combine streams | `DataStream` |
| `take(n)` | Take first n events | `DataStream` |
| `skip(n)` | Skip first n events | `DataStream` |
| `sort_by(key_fn)` | Sort events | `DataStream` |
| `count()` | Count events | `usize` |
| `collect()` | Collect to Vec | `Vec<StreamEvent>` |

### KeyedStream Methods

| Method | Description | Returns |
|--------|-------------|---------|
| `reduce(reducer)` | Reduce per key | `HashMap<K, StreamEvent>` |
| `aggregate(aggregator)` | Aggregate per key | `HashMap<K, AggregateResult>` |
| `window(config)` | Window per key | `KeyedWindowedStream<K>` |
| `count()` | Count per key | `HashMap<K, usize>` |
| `flatten()` | Back to DataStream | `DataStream` |

### WindowedStream Methods

| Method | Description | Returns |
|--------|-------------|---------|
| `aggregate(aggregator)` | Aggregate per window | `Vec<AggregateResult>` |
| `reduce(reducer)` | Reduce per window | `Vec<StreamEvent>` |
| `counts()` | Count per window | `Vec<usize>` |
| `flatten()` | Back to DataStream | `DataStream` |

## Examples

See full examples:
- `examples/03-advanced-features/stream_operators_demo.rs` - Basic operators
- `examples/06-use-cases/iot_monitoring_demo.rs` - IoT monitoring
- `examples/06-use-cases/fraud_detection_stream.rs` - Fraud detection

## Next Steps

- Explore [State Management](./STATE_MANAGEMENT.md) for stateful operators
- Learn about [Watermarks](./WATERMARKS.md) for handling late data
- Check out [Complex Event Processing](./CEP.md) for pattern detection

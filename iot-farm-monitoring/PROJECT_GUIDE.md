# IoT Farm Monitoring - Project Guide

## 🎯 Overview

This is a **production-ready, standalone project** that demonstrates the power of Rust Rule Engine's stream processing capabilities for real-world IoT applications. It was created as a separate project to avoid bloating the main rule engine repository.

## 📁 Project Structure

```
iot-farm-monitoring/
├── Cargo.toml              # Dependencies and configuration
├── Makefile                # Build and run shortcuts
├── README.md               # User documentation
├── PROJECT_GUIDE.md        # This file - development guide
├── docker-compose.yml      # Kafka setup
├── config.toml             # Runtime configuration
│
├── src/
│   ├── lib.rs              # Library entry point
│   ├── main.rs             # Main binary (requires Kafka)
│   ├── events.rs           # Event types and parsing
│   ├── monitor.rs          # Farm monitoring core logic
│   ├── config.rs           # Configuration management
│   └── kafka/
│       ├── mod.rs
│       └── consumer.rs     # Kafka consumer integration
│
├── examples/
│   ├── basic_demo.rs       # Standalone demo (no Kafka)
│   └── kafka_consumer.rs   # Kafka integration demo
│
├── tests/
│   └── integration_tests.rs
│
└── scripts/
    ├── setup_kafka.sh      # Initialize Kafka
    └── produce_events.sh   # Send test events
```

## 🚀 Quick Start

### 1. Run Without Kafka (Recommended First Step)

```bash
cd iot-farm-monitoring
cargo run --example basic_demo
```

This demonstrates all use cases with simulated sensor data.

### 2. Run With Kafka

```bash
# Install cmake (required for rdkafka)
brew install cmake      # macOS
# sudo apt-get install cmake  # Linux

# Build with Kafka support
cargo build --features kafka

# Setup Kafka
make kafka-setup

# Run the consumer (in one terminal)
cargo run --features kafka --example kafka_consumer

# Produce test events (in another terminal)
make kafka-produce
```

## 🏗️ Architecture

### Use Cases Implemented

1. **Automatic Irrigation Control**
   - Joins: `soil-sensors` ⨝ `temperature`
   - Condition: moisture < 30% AND temp > 25°C
   - Action: Trigger irrigation

2. **Frost Alert System**
   - Joins: `temperature` ⨝ `weather`
   - Condition: temp < 2°C AND condition includes "frost"
   - Action: Send frost alert

3. **Irrigation Efficiency Analysis**
   - Joins: `irrigation` ⨝ `soil-sensors`
   - Condition: soil reading AFTER irrigation stop
   - Action: Log efficiency metrics

4. **Sensor Anomaly Detection**
   - Validates sensor readings against thresholds
   - Detects out-of-range values

### Stream Join Patterns

```rust
// Inner Join - Match events from both streams
StreamJoinNode::new(
    "soil-sensors",     // Left stream
    "temperature",      // Right stream
    JoinType::Inner,
    JoinStrategy::TimeWindow { duration: 10min },
    // Key extractors (join on zone_id)
    Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
    Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
    // Custom condition
    Box::new(|left, right| right.timestamp >= left.timestamp),
)
```

### Event Format

**Soil Sensor:**
```json
{
  "zone_id": "zone_1",
  "moisture_level": 25.0,
  "timestamp": 1000
}
```

**Temperature Sensor:**
```json
{
  "zone_id": "zone_1",
  "temperature": 28.0,
  "sensor_type": "soil",
  "timestamp": 1010
}
```

**Irrigation Event:**
```json
{
  "zone_id": "zone_1",
  "action": "stop",
  "water_volume_ml": 50000,
  "timestamp": 1300
}
```

**Weather Station:**
```json
{
  "location": "farm",
  "condition": "frost_risk",
  "temperature": 0.5,
  "timestamp": 1005
}
```

## 🔧 Development

### Building

```bash
# Build without Kafka (default)
cargo build

# Build with Kafka support
cargo build --features kafka

# Release build
cargo build --release --features kafka
```

### Testing

```bash
# Run tests
cargo test

# Run specific test
cargo test test_irrigation_control_triggered

# Run with logs
RUST_LOG=info cargo test -- --nocapture
```

### Configuration

Edit `config.toml` to customize:
- Kafka brokers and topics
- Monitoring thresholds
- Optimization settings
- Alerting configuration

### Adding New Use Cases

1. **Define Event Type** in `src/events.rs`:
```rust
pub struct MyNewEvent {
    pub zone_id: String,
    pub value: f64,
    pub timestamp: i64,
}
```

2. **Create Helper Functions**:
```rust
pub fn create_my_new_event(zone_id: &str, value: f64, timestamp: i64) -> StreamEvent {
    // ... implementation
}

pub fn parse_my_new_event(event: &StreamEvent) -> Option<MyNewEvent> {
    // ... implementation
}
```

3. **Register Join in Monitor** (`src/monitor.rs`):
```rust
pub fn register_my_new_use_case(&mut self) {
    let join = StreamJoinNode::new(
        "my-stream-a",
        "my-stream-b",
        JoinType::Inner,
        JoinStrategy::TimeWindow { duration: Duration::from_secs(300) },
        Box::new(|e| e.data.get("join_key").and_then(|v| v.as_string())),
        Box::new(|e| e.data.get("join_key").and_then(|v| v.as_string())),
        Box::new(|a, b| /* your condition */),
    );

    self.join_manager.register_join(
        "my_use_case",
        join,
        Box::new(|joined| {
            // Handle joined events
        }),
    );
}
```

4. **Add Test** in `tests/integration_tests.rs`:
```rust
#[test]
fn test_my_new_use_case() {
    let mut monitor = FarmMonitor::with_defaults();
    monitor.register_my_new_use_case();

    // Send test events
    // Assert results
}
```

## 📊 Performance

### Memory Usage (10-minute window)

```
Soil sensors:   100 sensors × 0.1 Hz × 600s = 6,000 events × 200B = 1.2 MB
Temperature:    100 sensors × 0.2 Hz × 600s = 12,000 events × 200B = 2.4 MB
Irrigation:     1 event/min × 10 min = 10 events × 150B = 1.5 KB
Weather:        1 station × 0.05 Hz × 600s = 30 events × 180B = 5.4 KB

Total: ~5 MB (with 50% overhead for hash tables and indices)
```

### Optimization Strategies Applied

- **BuildSmaller**: Use irrigation stream (smallest) as hash table build side
- **PrePartition**: Partition by zone_id (100 zones → 10 partitions)
- **BloomFilter**: For sparse joins (< 10% selectivity)
- **IndexJoinKey**: Index zone_id for O(1) lookups
- **MergeWindows**: Combine adjacent time windows

### Benchmarking

```bash
# Run with timing
time cargo run --example basic_demo

# With profiling
cargo build --release --features kafka
cargo flamegraph --example kafka_consumer
```

## 🐛 Troubleshooting

### "cmake not found"

Kafka requires cmake to build:
```bash
brew install cmake      # macOS
sudo apt-get install cmake  # Linux
```

### "Failed to connect to Kafka"

Ensure Kafka is running:
```bash
docker-compose ps
docker-compose up -d
```

### Topics don't exist

Create topics:
```bash
./scripts/setup_kafka.sh
```

### No events received

Check topic names match:
```bash
docker exec -it kafka kafka-topics.sh --list --bootstrap-server localhost:9092
```

## 📚 Further Reading

- [Rust Rule Engine Docs](../README.md)
- [Stream Processing Guide](../docs/STREAMING.md)
- [RETE Algorithm](../docs/RETE.md)
- [Kafka Documentation](https://kafka.apache.org/documentation/)

## 🤝 Contributing

This is a demonstration project. For production use:
1. Add proper error handling and retries
2. Implement alerting system (email, SMS, webhooks)
3. Add metrics collection (Prometheus, Grafana)
4. Store events to database for analytics
5. Add authentication and authorization
6. Implement dashboard UI

## 📝 License

MIT License - same as Rust Rule Engine

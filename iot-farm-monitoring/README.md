# IoT Farm Monitoring System

A production-ready IoT monitoring system for smart agriculture using Rust Rule Engine's stream processing capabilities with Kafka integration.

## Features

- **Real-time Stream Processing**: Process sensor data from multiple sources using stream joins
- **Kafka Integration**: Consume events from Apache Kafka topics
- **Smart Irrigation Control**: Automatically trigger watering based on soil moisture and temperature
- **Frost Alert System**: Early warning system for crop protection
- **Irrigation Efficiency Analysis**: Track water usage effectiveness
- **Sensor Anomaly Detection**: Detect missing or malfunctioning sensors
- **Optimization**: Cost-based join optimization for high-volume streams

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Kafka Topics                              │
├─────────────────────────────────────────────────────────────────┤
│  soil-sensors  │  temperature  │  irrigation  │  weather        │
└────────┬───────┴───────┬───────┴──────┬───────┴─────────┬───────┘
         │               │              │                 │
         └───────────────┴──────────────┴─────────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │   Stream Join Manager        │
          │  (Rust Rule Engine)          │
          ├──────────────────────────────┤
          │  • Automatic Irrigation      │
          │  • Frost Alert               │
          │  • Efficiency Analysis       │
          │  • Anomaly Detection         │
          └──────────────────────────────┘
                         │
                         ▼
          ┌──────────────────────────────┐
          │    Action Handlers           │
          ├──────────────────────────────┤
          │  • Trigger irrigation        │
          │  • Send alerts               │
          │  • Log analytics             │
          │  • Update dashboards         │
          └──────────────────────────────┘
```

## Prerequisites

- Rust 1.70+
- Apache Kafka 2.8+ (optional, for Kafka integration)
- librdkafka (for Kafka consumer)

### Installing librdkafka

**macOS:**
```bash
brew install librdkafka
```

**Ubuntu/Debian:**
```bash
sudo apt-get install librdkafka-dev
```

**Windows:**
Follow the [rdkafka-sys installation guide](https://github.com/fede1024/rust-rdkafka#installation)

## Quick Start

### 1. Run Basic Demo (No Kafka Required)

```bash
cargo run --example basic_demo
```

This runs the farm monitoring system with simulated sensor data, demonstrating:
- Automatic irrigation control
- Frost alert system
- Irrigation efficiency analysis
- Sensor anomaly detection

### 2. Run with Kafka

First, start Kafka (using Docker):

```bash
# Start Kafka and Zookeeper
docker-compose up -d

# Create topics
docker exec -it kafka kafka-topics.sh --create --topic soil-sensors --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1
docker exec -it kafka kafka-topics.sh --create --topic temperature --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1
docker exec -it kafka kafka-topics.sh --create --topic irrigation --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1
docker exec -it kafka kafka-topics.sh --create --topic weather --bootstrap-server localhost:9092 --partitions 3 --replication-factor 1
```

Then run the Kafka consumer:

```bash
cargo run --example kafka_consumer
```

Or run the main application:

```bash
cargo run --bin farm-monitor
```

## Usage Examples

### Basic Demo

```rust
use iot_farm_monitoring::*;

#[tokio::main]
async fn main() {
    // Initialize the farm monitor
    let mut monitor = FarmMonitor::new();

    // Register use cases
    monitor.register_irrigation_control();
    monitor.register_frost_alert();
    monitor.register_efficiency_analysis();
    monitor.register_anomaly_detection();

    // Process events
    let soil_event = create_soil_sensor_reading("zone_1", 25.0, 1000);
    let temp_event = create_temperature_reading("zone_1", 28.0, 1010);

    monitor.process_event(soil_event).await;
    monitor.process_event(temp_event).await;
}
```

### Kafka Consumer

```rust
use iot_farm_monitoring::kafka::KafkaFarmConsumer;

#[tokio::main]
async fn main() {
    let consumer = KafkaFarmConsumer::new(
        "localhost:9092",
        vec!["soil-sensors", "temperature", "irrigation", "weather"]
    ).await?;

    consumer.start_consuming().await?;
}
```

## Configuration

Configuration can be set via environment variables or `config.toml`:

```toml
[kafka]
brokers = "localhost:9092"
group_id = "farm-monitor-group"
topics = ["soil-sensors", "temperature", "irrigation", "weather"]

[monitoring]
irrigation_moisture_threshold = 30.0
irrigation_temp_threshold = 25.0
frost_alert_temperature = 2.0
time_window_seconds = 600  # 10 minutes

[optimization]
enable_partitioning = true
enable_bloom_filter = true
max_memory_mb = 100
```

## Use Cases

### 1. Automatic Irrigation Control

Joins soil moisture and temperature sensors to automatically trigger irrigation when:
- Soil moisture < 30%
- Temperature > 25°C
- Within 10-minute time window

### 2. Frost Alert System

Joins temperature sensors with weather station data to send alerts when:
- Temperature drops below 2°C
- Weather conditions indicate frost risk
- Multiple sensors confirm the reading

### 3. Irrigation Efficiency Analysis

Joins irrigation events with subsequent moisture readings to:
- Measure water absorption rates
- Calculate irrigation effectiveness
- Optimize watering schedules

### 4. Sensor Anomaly Detection

Uses left outer joins to detect:
- Missing sensor readings
- Out-of-range values
- Sensor malfunctions

## Project Structure

```
iot-farm-monitoring/
├── Cargo.toml
├── README.md
├── config.toml
├── docker-compose.yml
├── src/
│   ├── main.rs              # Main application
│   ├── lib.rs               # Library exports
│   ├── monitor.rs           # Farm monitor core
│   ├── events.rs            # Event definitions
│   ├── use_cases/
│   │   ├── mod.rs
│   │   ├── irrigation.rs    # Irrigation control
│   │   ├── frost_alert.rs   # Frost alert system
│   │   ├── efficiency.rs    # Efficiency analysis
│   │   └── anomaly.rs       # Anomaly detection
│   ├── kafka/
│   │   ├── mod.rs
│   │   ├── consumer.rs      # Kafka consumer
│   │   └── producer.rs      # Kafka producer (for actions)
│   └── config.rs            # Configuration management
├── examples/
│   ├── basic_demo.rs        # Basic demo without Kafka
│   └── kafka_consumer.rs    # Kafka integration demo
└── tests/
    ├── integration_tests.rs
    └── kafka_tests.rs
```

## Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run with Kafka (requires running Kafka)
cargo test --test kafka_tests
```

## Performance

### Stream Statistics

Based on typical farm deployment:
- **Soil Sensors**: 100 sensors @ 0.1 Hz = 10 events/sec
- **Temperature Sensors**: 100 sensors @ 0.2 Hz = 20 events/sec
- **Irrigation Events**: ~1 event/minute = 0.017 events/sec
- **Weather Station**: 1 station @ 0.05 Hz = 0.05 events/sec

### Memory Usage

With 10-minute time windows:
- Soil sensor buffer: ~6,000 events × 200 bytes = 1.2 MB
- Temperature buffer: ~12,000 events × 200 bytes = 2.4 MB
- Total estimated memory: ~5 MB (with overhead)

### Optimization

The system uses several optimization strategies:
- **BuildSmaller**: Use irrigation stream (smallest) as hash table
- **PrePartition**: Partition by zone_id (100 zones → 10 partitions)
- **BloomFilter**: Skip non-matching events early (sparse joins)
- **IndexJoinKey**: Index zone_id for fast lookups

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Rust Rule Engine](https://github.com/your-org/rust-rule-engine)
- Kafka integration via [rdkafka](https://github.com/fede1024/rust-rdkafka)
- Inspired by real-world smart agriculture deployments

## Support

For issues and questions:
- GitHub Issues: [https://github.com/your-org/iot-farm-monitoring/issues](https://github.com/your-org/iot-farm-monitoring/issues)
- Documentation: [https://docs.rs/iot-farm-monitoring](https://docs.rs/iot-farm-monitoring)

# IoT Farm Monitoring System

A production-ready IoT monitoring system for smart agriculture using Rust Rule Engine's GRL-based stream processing with multi-stream joins and Kafka integration.

## Features

- **GRL Declarative Rules**: Define complex multi-stream joins using simple GRL syntax
- **Real-time Stream Processing**: Process sensor data from multiple Kafka topics with RETE Beta nodes
- **Multi-Stream Joins**: Automatic correlation across soil, temperature, irrigation, and weather streams
- **Time Windows**: Sliding and tumbling windows (2-10 minutes) for temporal correlation
- **Kafka Integration**: Consume events from Apache Kafka topics with millisecond-precision timestamps
- **Smart Alerts**: Critical irrigation, frost warnings, drought detection, optimal conditions monitoring
- **Statistics Tracking**: Real-time counters for irrigation triggers, frost alerts, and efficiency reports

## System Architecture

```
┌────────────────────────────────────────────────────────────────────────────┐
│                          Kafka Topics (4 streams)                          │
├────────────────────────────────────────────────────────────────────────────┤
│  soil-sensors  │  temperature  │  irrigation  │  weather                   │
└───────┬────────┴───────┬───────┴──────┬───────┴──────┬─────────────────────┘
        │                │              │              │
        ▼                ▼              ▼              ▼
┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐
│  Alpha1    │   │  Alpha2    │   │  Alpha3    │   │  Alpha4    │
├────────────┤   ├────────────┤   ├────────────┤   ├────────────┤
│ • Filter   │   │ • Filter   │   │ • Filter   │   │ • Filter   │
│   stream   │   │   stream   │   │   stream   │   │   stream   │
│   name     │   │   name     │   │   name     │   │   name     │
│ • Time     │   │ • Time     │   │ • Time     │   │ • Time     │
│   window   │   │   window   │   │   window   │   │   window   │
│   check    │   │   check    │   │   check    │   │   check    │
└─────┬──────┘   └─────┬──────┘   └─────┬──────┘   └─────┬──────┘
      │                │                │                │
      │    (filtered & windowed facts)  │                │
      └────────┬───────┴────────┬───────┴────────┬───────┘
               │                │                │
               ▼                ▼                ▼
        ┌─────────────────────────────────────────────┐
        │         Working Memory (Fact Store)         │
        ├─────────────────────────────────────────────┤
        │  Stores facts from all Alpha nodes          │
        │  • VecDeque<StreamEvent> per Alpha          │
        │  • Time-windowed buffers (2-10 min)         │
        │  • Auto-expires old events                  │
        │  • Indexed by zone_id for fast lookups      │
        └───────────────┬─────────────────────────────┘
                        │ (facts available for joining)
                        ▼
        ┌───────────────────────────────────────────┐
        │         Beta Nodes (8 join rules)         │
        ├───────────────────────────────────────────┤
        │  Read from Alpha buffers, join on zone_id │
        ├───────────────────────────────────────────┤
        │  • CriticalIrrigationNeeded (soil+temp)   │
        │  • OptimalConditions (soil+temp)          │
        │  • FrostAlert (temp+weather)              │
        │  • DroughtStress (soil+temp)              │
        │  • IrrigationEfficiency (soil+irr+temp)   │
        │  • RainDetected (soil+weather+irr)        │
        │  • ExtremeWeather (soil+weather+temp)     │
        │  • OptimalHarvest (soil+temp+weather)     │
        └───────────────┬───────────────────────────┘
                        │ (joined facts)
                        ▼
        ┌───────────────────────────────────────────┐
        │    Rule Evaluation Layer (Conditions)     │
        ├───────────────────────────────────────────┤
        │  evaluate_filter_conditions()             │
        │  • Parse field values from merged data    │
        │  • Check: moisture < 25.0                 │
        │  • Check: temperature > 30.0              │
        │  • Check: condition == "frost"            │
        │  • ALL conditions must pass               │
        └───────────────┬───────────────────────────┘
                        │ (if all pass)
                        ▼
        ┌───────────────────────────────────────────┐
        │         Actions & Side Effects            │
        ├───────────────────────────────────────────┤
        │  ✅ Rule FIRED - execute actions          │
        │  🚰 Log messages with context             │
        │  📊 Increment statistics counters         │
        │  � Trigger alerts/notifications           │
        └───────────────────────────────────────────┘
```

**Flow:**
1. **Kafka Topics** → Stream events (JSON with timestamp_ms, zone_id, sensor data)
2. **Alpha Nodes** → Filter events by stream name and time window
3. **Working Memory** → Store filtered facts (VecDeque buffers per stream, 2-10 min retention)
4. **Beta Nodes** → Join facts from Working Memory on zone_id (correlate sensors by zone)
5. **Rule Evaluation** → Check filter conditions on joined facts (moisture < 25%, temp > 30°C)
6. **Actions** → If all conditions pass: log messages, update statistics, trigger alerts

**Notes:** 
- **Architecture**: Working Memory is a separate layer between Alpha and Beta nodes (RETE standard)
- **Implementation**: Working Memory is embedded in each Alpha node as `VecDeque<StreamEvent>` buffers
- Rule Evaluation uses `evaluate_filter_conditions()` to check business logic after successful joins

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

### 1. Setup Kafka

Run the setup script to start Kafka and create topics:

```bash
./scripts/setup_kafka.sh
```

This will:
- Start Kafka and Zookeeper via Docker Compose
- Create 4 topics: `soil-sensors`, `temperature`, `irrigation`, `weather`
- Verify topic creation

### 2. Start the Farm Monitor

```bash
cargo run --bin farm-monitor --features kafka
```

This will:
- Load GRL rules from `grl_rules/farm_monitoring_multistream.grl`
- Create 4 Alpha nodes (one per stream)
- Create 8 Beta nodes (one per multi-stream join rule)
- Start consuming from Kafka topics
- Display rule fires with statistics

### 3. Produce Test Events

In another terminal, generate test sensor data:

```bash
./scripts/produce_events.sh
```

This produces events covering all scenarios:
- Critical irrigation needs (low moisture + high temp)
- Optimal growing conditions
- Frost alerts
- Drought stress
- Irrigation efficiency tracking
- Rain detection
- Extreme weather conditions
- Optimal harvest timing

## GRL Rules

The system uses 8 declarative multi-stream join rules defined in `grl_rules/farm_monitoring_multistream.grl`:

1. **CriticalIrrigationNeeded** - Joins soil + temperature (moisture < 25%, temp > 30°C)
2. **OptimalConditions** - Joins soil + temperature (40-60% moisture, 22-28°C)
3. **FrostAlert** - Joins temperature + weather (temp < 0°C, frost condition)
4. **DroughtStress** - Joins soil + temperature (moisture < 20%, temp > 35°C)
5. **IrrigationEfficiency** - Joins soil + irrigation + temperature (moisture change analysis)
6. **RainDetectedSkipIrrigation** - Joins soil + weather + irrigation (skip if rain detected)
7. **ExtremeWeatherIrrigation** - Joins soil + weather + temperature (extreme heat response)
8. **OptimalHarvestConditions** - Joins soil + temperature + weather (harvest readiness)

### Example GRL Rule

```grl
rule "CriticalIrrigationNeeded" salience 100 {
    when
        moisture: SoilSensorReading from stream("soil-sensors") over window(5 min, sliding) &&
        temp: TemperatureReading from stream("temperature") over window(5 min, sliding) &&
        moisture.zone_id == temp.zone_id &&
        moisture.moisture_level < 25.0 &&
        temp.temperature > 30.0
    then
        log("🚰 CRITICAL IRRIGATION: Zone {} - Moisture: {:.1}%, Temp: {:.1}°C",
            moisture.zone_id, moisture.moisture_level, temp.temperature);
}
```

## Configuration

Configuration in `config.toml`:

```toml
[kafka]
brokers = "localhost:9092"
group_id = "farm-monitor-group"
topics = ["soil-sensors", "temperature", "irrigation", "weather"]

[monitoring]
# GRL rules file
grl_rules_file = "grl_rules/farm_monitoring_multistream.grl"

# Statistics output interval
stats_interval_seconds = 10
```

## Output Example

```
✅ Created 4 alpha nodes (1 per stream)
✅ Created 8 beta nodes (multi-stream joins)
✅ Created stream processor from GRL rules

✅ Rule 'CriticalIrrigationNeeded' FIRED for zone zone_1
✅ Rule 'FrostAlert' FIRED for zone zone_2
✅ Rule 'IrrigationEfficiency' FIRED for zone zone_3

📊 === Farm Monitor Statistics ===
   Events Processed: 4069
   Irrigation Triggered: 76
   Frost Alerts: 28
   Efficiency Reports: 56
   Anomalies Detected: 0
```

## Project Structure

```
iot-farm-monitoring/
├── Cargo.toml                      # Dependencies: tokio, rdkafka, serde
├── README.md                       # This file
├── config.toml                     # Kafka and monitoring configuration
├── docker-compose.yml              # Kafka + Zookeeper setup
├── src/
│   ├── main.rs                     # Binary entry point
│   ├── lib.rs                      # Public API exports
│   ├── monitor.rs                  # FarmMonitor + MonitorStats
│   ├── events.rs                   # StreamEvent definitions
│   ├── config.rs                   # Config loader
│   ├── stream_rule_processor.rs    # GRL → RETE processor (Alpha + Beta nodes)
│   ├── working_memory.rs           # Working Memory (fact storage layer)
│   └── kafka/
│       ├── mod.rs
│       └── consumer.rs             # Kafka → StreamEvent consumer
├── grl_rules/
│   ├── farm_monitoring_multistream.grl  # 8 declarative join rules
│   └── README.md
├── scripts/
│   ├── setup_kafka.sh              # Docker + topic creation
│   └── produce_events.sh           # Test data generator
└── docs/
    ├── GETTING_STARTED.md
    ├── MULTI_STREAM_JOINS.md
    ├── STREAM_TIME_WINDOW_GUIDE.md
    └── PROJECT_GUIDE.md
```

## Architecture

### RETE-Based Stream Processing

```
Kafka Topics     Alpha Nodes      Working Memory       Beta Nodes       Rule Evaluation      Actions
─────────────   ───────────      ──────────────       ──────────       ───────────────      ─────────

soil-sensors ──> Alpha1 ──┐
                          │
temperature  ──> Alpha2 ──┤      Working Memory       Beta1 (join)     Evaluate:            ✅ Rule
                          ├───>  (Fact Store)    ──>  soil + temp  ──> moisture < 25%  ──>  FIRED
weather      ──> Alpha3 ──┤      • VecDeque/stream                     temp > 30°           🚰 Log
                          │      • 2-10 min window                                          📊 Stats
irrigation   ──> Alpha4 ──┘      • Auto-expire        Beta2 (join)     Evaluate:            ✅ Rule
                                                  ──>  temp + weather ─> temp < 0°     ──>  FIRED
                                                       (multi-stream)   condition="frost"    ❄️ Alert
```

### Key Components

- **GRL Parser**: Converts declarative rules to Alpha + Beta node network
- **Alpha Nodes**: Per-stream filtering and time window checking
- **Working Memory**: Central fact storage layer (facade over Alpha node buffers)
- **Beta Nodes**: Multi-stream joins on zone_id reading from Working Memory
- **Rule Evaluation**: Filter condition checks after successful joins
- **StreamRuleProcessor**: Orchestrates event routing and rule firing
- **MonitorStats**: Real-time statistics tracking

## Performance

### Real-World Results

From production testing:
- **Events Processed**: ~4000 events in 8 seconds (~500 events/sec)
- **Rules Fired**: 160 total fires across 8 rules
- **Latency**: Sub-millisecond join evaluation
- **Memory**: ~5 MB for 10-minute windows across 4 streams

### Optimization Features

- **Time Windows**: Automatic expiry of old events
- **Zone-based Correlation**: Join on zone_id for spatial correlation
- **Millisecond Timestamps**: Precise temporal alignment
- **Statistics Integration**: Zero-overhead counter tracking

## Troubleshooting

### Kafka Connection Issues

If you see "Failed to connect to broker":
```bash
# Check if Kafka is running
docker ps | grep kafka

# Restart Kafka
docker-compose down
docker-compose up -d
```

### No Rules Firing

Check timestamps:
- Events must have `timestamp_ms` in milliseconds (not seconds)
- Events must fall within the time window
- Join keys (zone_id) must match exactly

### High Memory Usage

Adjust time windows in GRL rules:
```grl
# Reduce from 10 min to 5 min
over window(5 min, sliding)
```

## Documentation

- [Getting Started Guide](docs/GETTING_STARTED.md)
- [Multi-Stream Joins](MULTI_STREAM_JOINS.md)
- [Stream Time Windows](STREAM_TIME_WINDOW_GUIDE.md)
- [Project Guide](PROJECT_GUIDE.md)

## License

This project is part of the Rust Rule Engine ecosystem.

## Acknowledgments

- Built with custom RETE algorithm implementation
- Kafka integration via [rdkafka](https://github.com/fede1024/rust-rdkafka)
- GRL parser with multi-stream join support
- Inspired by real-world smart agriculture IoT deployments

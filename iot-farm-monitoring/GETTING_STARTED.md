# Getting Started with IoT Farm Monitoring

## 🚀 5-Minute Quick Start

### Step 1: Navigate to Project
```bash
cd iot-farm-monitoring
```

### Step 2: Run the Demo
```bash
cargo run --example basic_demo
```

That's it! You'll see:
```
🚜 IoT Farm Monitoring System - Basic Demo
=============================================

=== Use Case 1: Automatic Irrigation Control ===
🚰 IRRIGATION TRIGGERED for zone zone_1 - Moisture: 25.0%, Temp: 28.0°C

=== Use Case 2: Frost Alert System ===
❄️ FROST ALERT for zone zone_3 - Temp: 1.0°C, Weather: frost_risk

=== Use Case 3: Irrigation Efficiency Analysis ===
📊 EFFICIENCY REPORT for zone zone_5 - Moisture after 5 min: 65.0%

=== Use Case 4: Optimization Analysis ===
  Estimated Cost: 0.72
  Optimizations: 2
  Strategy: Using left stream as build side (smaller)...
  Estimated Memory: 5 MB

✅ Demo completed successfully!
```

## 🎓 What You Just Saw

The demo simulated:
1. **Sensor readings** from 3 zones
2. **Stream joins** between soil moisture and temperature
3. **Automatic decisions** (trigger irrigation, send alerts)
4. **Efficiency analysis** of irrigation
5. **Performance optimization** of join strategies

## 📚 Next Steps

### Option A: Explore the Code
```bash
# Look at event definitions
cat src/events.rs

# See the monitoring logic
cat src/monitor.rs

# Check the demo
cat examples/basic_demo.rs
```

### Option B: Run Tests
```bash
cargo test

# Expected output:
# running 7 tests
# test test_config_default ... ok
# test test_event_parsing ... ok
# test test_frost_alert_triggered ... ok
# test test_irrigation_control_triggered ... ok
# ...
# test result: ok. 7 passed
```

### Option C: Customize Configuration
```bash
# Edit thresholds
vim config.toml

# Change these values:
irrigation_moisture_threshold = 35.0  # Was 30.0
irrigation_temp_threshold = 20.0      # Was 25.0

# Run again to see different behavior
cargo run --example basic_demo
```

### Option D: Add Your Own Use Case

Create `examples/my_use_case.rs`:
```rust
use iot_farm_monitoring::*;

fn main() {
    let mut monitor = FarmMonitor::with_defaults();

    // Your custom monitoring logic here!

    monitor.process_event(create_soil_sensor_reading("zone_1", 40.0, 1000));
    // ... more events

    println!("Stats: {:?}", monitor.get_stats());
}
```

Run it:
```bash
cargo run --example my_use_case
```

## 🐳 Advanced: Kafka Integration

Want to try with real Kafka?

### Prerequisites
```bash
# macOS
brew install cmake
brew install docker

# Linux
sudo apt-get install cmake docker.io docker-compose
```

### Setup
```bash
# Start Kafka (takes ~30 seconds)
make kafka-setup

# In terminal 1: Start consumer
cargo run --features kafka --example kafka_consumer

# In terminal 2: Send test events
make kafka-produce
```

### Expected Output (Terminal 1)
```
🔌 Connecting to Kafka...
✅ Consumer is ready! Waiting for events...

🚰 IRRIGATION TRIGGERED for zone zone_1 - Moisture: 25.0%, Temp: 28.0°C
❄️ FROST ALERT for zone zone_3 - Temp: 1.0°C, Weather: frost_risk
...
```

### Cleanup
```bash
# Stop Kafka
make kafka-stop
```

## 🎯 Understanding the Code

### Event Flow
```
Sensor → StreamEvent → JoinNode → Handler → Action
  |          |            |          |         |
 Raw       Parsed      Matched   Process   Alert
 Data      Event       Events    Logic    System
```

### Key Components

**1. Events** (`src/events.rs`)
- Define sensor data structures
- Parse from/to StreamEvent
- 4 event types: Soil, Temperature, Irrigation, Weather

**2. Monitor** (`src/monitor.rs`)
- Register use cases (join definitions)
- Process incoming events
- Track statistics

**3. Join Strategy**
```rust
StreamJoinNode::new(
    "soil-sensors",        // Left stream
    "temperature",         // Right stream
    JoinType::Inner,       // Match both sides
    JoinStrategy::TimeWindow { duration: 10min },
    // Join on zone_id
    Box::new(|e| e.data.get("zone_id")),
    Box::new(|e| e.data.get("zone_id")),
    // Custom condition
    Box::new(|left, right| right.timestamp >= left.timestamp),
)
```

## 💡 Common Tasks

### Add a New Sensor Type

1. Define in `src/events.rs`:
```rust
pub struct MySensorReading {
    pub zone_id: String,
    pub my_value: f64,
    pub timestamp: i64,
}

pub fn create_my_sensor_reading(zone_id: &str, value: f64, ts: i64) -> StreamEvent {
    // ... implementation
}
```

2. Register join in `src/monitor.rs`:
```rust
pub fn register_my_use_case(&mut self) {
    let join = StreamJoinNode::new(/* ... */);
    self.join_manager.register_join("my_use_case", join, handler);
}
```

3. Add test in `tests/integration_tests.rs`:
```rust
#[test]
fn test_my_use_case() {
    let mut monitor = FarmMonitor::with_defaults();
    monitor.register_my_use_case();
    // ... test logic
}
```

### Modify Thresholds

Edit `config.toml`:
```toml
[monitoring]
irrigation_moisture_threshold = 35.0  # Changed from 30.0
frost_alert_temperature = 5.0          # Changed from 2.0
```

Or in code:
```rust
let mut config = Config::default();
config.monitoring.irrigation_moisture_threshold = 35.0;
let monitor = FarmMonitor::new(config);
```

### Add Logging

```rust
use log::{info, warn, error};

info!("Processing event: {:?}", event);
warn!("Threshold exceeded: {}", value);
error!("Failed to process: {}", err);
```

Run with logs:
```bash
RUST_LOG=info cargo run --example basic_demo
RUST_LOG=debug cargo run --example basic_demo
```

## 📖 Documentation Files

- `README.md` - User guide (start here)
- `GETTING_STARTED.md` - This file (quick start)
- `PROJECT_GUIDE.md` - Architecture & development
- `INTEGRATION.md` - Integration patterns
- `SUMMARY.md` - Project overview

## 🆘 Troubleshooting

### "command not found: cargo"
Install Rust: https://rustup.rs/

### "cmake not found"
```bash
brew install cmake      # macOS
sudo apt-get install cmake  # Linux
```

### "Failed to connect to Kafka"
Make sure Kafka is running:
```bash
docker-compose ps
make kafka-setup
```

### Tests fail
```bash
# Clean and rebuild
cargo clean
cargo test
```

## 🎉 Success Criteria

You've successfully set up the project when:

✅ Demo runs without errors
```bash
cargo run --example basic_demo
# Should show: ✅ Demo completed successfully!
```

✅ Tests pass
```bash
cargo test
# Should show: test result: ok. 7 passed
```

✅ You understand the event flow
- Events created → Joins matched → Handlers executed

## 🚀 Ready to Build?

You now have a working IoT monitoring system with:
- ✅ Stream processing
- ✅ Real-time joins
- ✅ Event correlation
- ✅ Performance optimization
- ✅ Production patterns

**Next:** Adapt it for your use case! 🎯

See `PROJECT_GUIDE.md` for architecture details or `INTEGRATION.md` for integration patterns.

---

**Questions?** Check the other documentation files or explore the example code!

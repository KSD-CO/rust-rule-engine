# GRL Rules Directory

## Active Rules

### `farm_monitoring_hybrid.grl`
**STATUS: ✅ ACTIVE - Currently loaded and running**

This is the ONLY GRL rule file currently in use by the IoT Farm Monitoring system.

**Architecture:**
- **Hybrid Approach**: Rust handles stream processing, GRL handles business logic
- **Integration Point**: `src/monitor.rs` line 38
- **Loading**: `GrlReteLoader::load_from_file("grl_rules/farm_monitoring_hybrid.grl")`

**How It Works:**
1. **Rust StreamJoinNode** processes Kafka events and time windows
2. **Rust callbacks** create TypedFacts (IrrigationDecision, FrostAlert, etc.)
3. **Facts inserted** into GRL RETE engine via `engine.insert()`
4. **GRL rules fire** automatically with incremental propagation
5. **GRL log()** statements output via `log::info!()` macro

**Rule Categories:**
- **Irrigation Decisions** (salience 80-100):
  - IrrigationRequired: Moisture < 30% AND Temp > 25°C
  - PreemptiveMonitoring: Early warning (score 25-40)
  - NormalConditions: Everything OK
  
- **Drought Stress** (salience 110):
  - Moisture < 20% AND Temp > 32°C → Critical alert

- **Frost Alerts** (salience 90-100):
  - CriticalFrostAlert: Temp < 0°C
  - FrostWarning: 0°C ≤ Temp < 5°C

- **Efficiency Analysis** (salience 90-100):
  - HighEfficiency: Moisture improvement > 20%
  - NormalEfficiency: 10% ≤ improvement ≤ 20%
  - LowEfficiency: improvement < 10% (possible leak/blockage)

- **Anomaly Detection** (salience 95-100):
  - CriticalAnomaly: Sensor readings out of valid range
  - HighSeverityAnomaly: Unusual patterns
  - ModerateAnomaly: Minor deviations

**Fact Structures:**
```rust
IrrigationDecision {
    zone_id: String,
    moisture_level: f64,
    temperature: f64,
    recommendation: String,  // "IRRIGATE", "MONITOR", "OK"
    score: f64
}

FrostAlert {
    zone_id: String,
    temperature: f64,
    weather_condition: String
}

EfficiencyReport {
    zone_id: String,
    moisture_before: f64,
    moisture_after: f64,
    improvement: f64
}

AnomalyDetection {
    zone_id: String,
    sensor_type: String,
    value: f64,
    severity: String  // "CRITICAL", "HIGH", "MODERATE"
}
```

## Why Hybrid Approach?

### ❌ What Doesn't Work (Yet)
**Stream Syntax in GRL:**
```grl
// This syntax exists but is NOT integrated into the parser
rule "Example" {
    when
        soil from stream("soil-sensors") over window(1 min, sliding)
    then
        log("Value: {}", soil.moisture);  // Format strings not supported
}
```

**Not Supported:**
- ❌ `from stream("...")` syntax - parser exists but not integrated
- ❌ `over window(...)` syntax - requires major parser changes
- ❌ if-else in `then` blocks - use multiple rules instead
- ❌ `let` variable declarations in rules
- ❌ Format strings in log: `log("text {}", var)` - use multiple args instead

### ✅ What Works (Current Implementation)
**Rust Stream Processing + GRL Business Logic:**
```rust
// Rust: Handle streams and time windows
StreamJoinNode::new()
    .join_strategy(TimeWindow::new(Duration::from_secs(60)))
    .on_match(|soil, temp| {
        // Create fact for GRL
        let decision = IrrigationDecision {
            zone_id: soil.zone_id.clone(),
            moisture_level: soil.moisture_level,
            temperature: temp.temperature,
            recommendation: calculate_recommendation(...),
            score: calculate_score(...)
        };
        
        // Insert into GRL engine
        engine.insert(TypedFacts::from_struct(&decision)?)?;
    });
```

```grl
// GRL: Handle business rules
rule "IrrigationRequired" salience 100 {
    when
        decision: IrrigationDecision(decision.recommendation == "IRRIGATE")
    then
        log("🚰 IRRIGATION REQUIRED: Zone", decision.zone_id);
        log("   Moisture:", decision.moisture_level, "%, Temp:", decision.temperature, "°C");
        Retract(decision);
}

rule "LowEfficiency" salience 95 {
    when
        report: EfficiencyReport(report.improvement < 10.0)
    then
        log("⚠️ LOW EFFICIENCY: Zone", report.zone_id);
        log("   🔧 Check irrigation system: possible leak or blockage");
        Retract(report);
}
```

## Testing

**Start farm-monitor:**
```bash
cd iot-farm-monitoring
RUST_LOG=info cargo run --release --features kafka --bin farm-monitor
```

**Generate test data:**
```bash
# Basic test
./scripts/produce_test_data.sh

# Stress test with 6 scenarios
./scripts/produce_stress_test_data.sh
```

**Expected Output:**
```
INFO  [iot_farm_monitoring::monitor] ✅ Loaded 13 GRL rules from grl_rules/farm_monitoring_hybrid.grl
INFO  [rust_rule_engine::engine::engine] 📋 🚰 IRRIGATION REQUIRED: Zone zone_1
INFO  [rust_rule_engine::engine::engine] 📋    Moisture: 25.5 %, Temp: 30.2 °C
INFO  [iot_farm_monitoring::monitor] ✅ Irrigation trigger #1: zone_1 (Score: 45.8/100)
```

## Rule Design Guidelines

### ✅ DO: Use Multiple Rules for Conditional Logic
```grl
rule "HighEfficiency" {
    when report: EfficiencyReport(report.improvement > 20.0)
    then log("📈 HIGH EFFICIENCY"); Retract(report);
}

rule "LowEfficiency" {
    when report: EfficiencyReport(report.improvement < 10.0)
    then log("⚠️ LOW EFFICIENCY"); Retract(report);
}
```

### ❌ DON'T: Use if-else (Not Supported)
```grl
rule "Efficiency" {
    when report: EfficiencyReport()
    then
        if report.improvement > 20.0 {  // ❌ NOT SUPPORTED
            log("HIGH");
        }
}
```

### ✅ DO: Use Salience for Priority
```grl
rule "Critical" salience 110 { ... }  // Fires first
rule "High" salience 100 { ... }      // Fires second
rule "Normal" salience 90 { ... }     // Fires last
```

### ✅ DO: Use Simple Log Statements
```grl
log("Text");                           // ✅ Single string
log("Value:", var);                    // ✅ Multiple args (joined by space)
log("Zone", zone.id, "at", temp, "°C"); // ✅ Multiple values
```

### ❌ DON'T: Use Format Strings (Not Yet)
```grl
log("Zone {} at {:.1}°C", zone, temp); // ❌ Format strings not supported
```

## File History

**Removed Files (2024-12-18):**
- `farm_monitoring_streams.grl` - Stream syntax prototype (not functional)
- `stream_greenhouse_monitoring.grl` - Example with unsupported syntax
- `stream_demo_WORKING.grl` - Old demo file
- `aquaculture_monitoring.grl` - Unrelated use case
- `vegetable_monitoring.grl` - Unrelated use case
- `integrated_farm_rules.grl` - Old version
- `stream_aquaculture.grl` - Unrelated use case
- `stream_climate_control.grl` - Unrelated use case

**Reason for Removal:** These files contained stream syntax (`from stream(...)`) which looks nice but cannot be parsed by the current GRL parser. They were causing confusion about what syntax is actually supported.

## Future Roadmap

### Phase 1: Current (Hybrid Approach) ✅
- Rust handles Kafka streams and time windows
- GRL handles business rule logic
- Facts passed via TypedFacts interface

### Phase 2: Stream Syntax Integration (Future)
- Integrate `parse_stream_source()` from `src/parser/grl/stream_syntax.rs`
- Support `from stream("...")` and `over window(...)` in parser
- Enable direct stream processing in GRL
- Add format string support to log()
- Support if-else and let in `then` blocks

### Phase 3: Full Stream Processing (Long-term)
- Native GRL stream operators
- Complex event processing in GRL
- Pattern detection across streams
- Temporal reasoning

## Support

**Issues:**
- GRL rules not loading? Check `RUST_LOG=info` output
- Rules not firing? Check that Facts match the expected structure
- Log statements not appearing? Verify `log::info!()` is used (not `println!()`)

**Documentation:**
- Main docs: `../docs/GRL_LOGGING_GUIDE.md`
- Farm layout: `../docs/FARM_LAYOUT_DETAILED.md`
- Stream guide: `../STREAM_TIME_WINDOW_GUIDE.md`

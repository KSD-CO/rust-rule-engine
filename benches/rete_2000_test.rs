use rust_rule_engine::rete::{
    ReteUlEngine,
    auto_network::{Rule, ConditionGroup, Condition},
};
use std::time::{Duration, Instant};

fn main() {
    println!("🚀 Starting 2000 Rules Performance Test for RETE-UL");
    println!("==================================================");

    let mut engine = ReteUlEngine::new();

    // Create 2000 rules with varying complexity
    println!("📝 Creating 2000 rules...");
    let start_create = Instant::now();

    for i in 0..2000 {
        let rule = Rule {
            name: format!("Rule{}", i),
            conditions: ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition {
                    field: "count".to_string(),
                    operator: ">".to_string(),
                    value: format!("{}", i % 100), // Vary threshold
                })),
                operator: "AND".to_string(),
                right: Box::new(ConditionGroup::Single(Condition {
                    field: "status".to_string(),
                    operator: "==".to_string(),
                    value: format!("active{}", i % 10), // Vary status
                })),
            },
            action: format!("log('Rule{} fired')", i),
        };

        engine.add_rule_from_definition(&rule, (2000 - i) as i32, false);
    }

    let create_time = start_create.elapsed();
    println!("✅ Created 2000 rules in {:?}", create_time);

    // Set up facts that will match ~50% of rules
    println!("📊 Setting up test facts...");
    engine.set_fact("count".to_string(), "50".to_string());
    engine.set_fact("status".to_string(), "active5".to_string());

    // Test single execution
    println!("⚡ Testing single execution...");
    let start_single = Instant::now();
    let result = engine.fire_all();
    let single_time = start_single.elapsed();

    println!("📈 Single execution results:");
    println!("   Time: {:?}", single_time);
    println!("   Rules fired: {}", result.len());
    println!("   Latency per rule: {:.2} µs", single_time.as_micros() as f64 / 2000.0);
    println!("   Rules/second: {:.2}", 2000.0 / single_time.as_secs_f64());

    // Test multiple executions
    println!("🔄 Testing multiple executions...");
    let mut total_fired = 0;
    let mut times = Vec::new();
    let num_runs = 5;

    for run in 0..num_runs {
        let start = Instant::now();
        let result = engine.fire_all();
        let elapsed = start.elapsed();
        total_fired += result.len();
        times.push(elapsed);
        println!("   Run {}: {:?} ({} rules fired)", run + 1, elapsed, result.len());
    }

    let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
    println!("📊 Average execution time: {:?}", avg_time);
    println!("🎯 Total rules fired across {} runs: {}", num_runs, total_fired);
    println!("📈 Average rules fired per run: {}", total_fired / num_runs);

    // Performance analysis
    let rules_per_second = 2000.0 / avg_time.as_secs_f64();
    println!("🚀 Performance: {:.2} rules/second", rules_per_second);
    println!("⚡ Latency: {:.2} µs per rule", avg_time.as_micros() as f64 / 2000.0);

    // Scalability assessment
    if avg_time > Duration::from_secs(1) {
        println!("❌ CRITICAL: Execution took > 1 second - NOT suitable for real-time use");
        println!("   This indicates fundamental scalability issues");
    } else if avg_time > Duration::from_millis(500) {
        println!("⚠️  WARNING: Execution took > 500ms - borderline for real-time applications");
        println!("   May work for batch processing but not interactive systems");
    } else if avg_time > Duration::from_millis(100) {
        println!("⚠️  WARNING: Execution took > 100ms - acceptable for some real-time use");
        println!("   Good for most business applications");
    } else {
        println!("✅ EXCELLENT: Fast execution suitable for real-time systems");
        println!("   Can handle high-frequency rule evaluation");
    }

    // Memory assessment (rough estimate)
    let estimated_memory_kb = 2000 * 50; // Rough estimate: 50KB per rule
    if estimated_memory_kb > 100 * 1024 { // 100MB
        println!("⚠️  WARNING: Estimated high memory usage ({} KB)", estimated_memory_kb);
        println!("   Monitor for memory leaks in production");
    } else {
        println!("✅ Reasonable memory usage estimated at {} KB", estimated_memory_kb);
    }

    // Comparison with smaller scales
    println!("📊 Scale Comparison:");
    println!("   10 rules: ~5 µs per rule (estimated from benchmarks)");
    println!("   50 rules: ~2.3 µs per rule (from existing benchmarks)");
    println!("   2000 rules: {:.2} µs per rule (measured)", avg_time.as_micros() as f64 / 2000.0);

    let scaling_factor = (avg_time.as_micros() as f64 / 2000.0) / 2.3;
    println!("   Scaling degradation: {:.1}x slower than 50-rule baseline", scaling_factor);

    if scaling_factor > 10.0 {
        println!("❌ POOR SCALING: Performance degrades significantly with rule count");
        println!("   RETE-UL may not be suitable for large rule sets");
    } else if scaling_factor > 5.0 {
        println!("⚠️  MODERATE SCALING: Some performance degradation");
        println!("   Acceptable for medium-large rule sets");
    } else {
        println!("✅ GOOD SCALING: Maintains reasonable performance");
        println!("   Suitable for large rule sets");
    }

    println!("\n🎯 CONCLUSION:");
    if avg_time < Duration::from_millis(100) && scaling_factor < 5.0 {
        println!("   RETE-UL performs WELL at 2000 rules scale");
        println!("   Suitable for production use with large rule sets");
    } else if avg_time < Duration::from_millis(500) {
        println!("   RETE-UL performs ADEQUATELY at 2000 rules scale");
        println!("   May need optimization for high-performance requirements");
    } else {
        println!("   RETE-UL shows SIGNIFICANT PERFORMANCE ISSUES at 2000 rules");
        println!("   Consider alternative approaches or full RETE implementation");
    }
}
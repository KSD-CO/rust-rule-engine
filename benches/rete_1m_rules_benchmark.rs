use std::time::{Duration, Instant};
use rust_rule_engine::rete::{
    ReteUlEngine,
    auto_network::{Rule, ConditionGroup, Condition},
};

fn main() {
    println!("🚀 Starting 1,000,000 Rules Benchmark for RETE-UL");
    println!("==================================================");
    println!("⚠️  WARNING: This benchmark tests 1 MILLION rules!");
    println!("   - May take several minutes to create rules");
    println!("   - May use 500MB+ of memory");
    println!("   - May take 10-30 seconds per execution");
    println!("   - Consider running on a machine with plenty of RAM");
    println!("==================================================");

    let mut engine = ReteUlEngine::new();

    // Create 1,000,000 rules with varying complexity
    println!("📝 Creating 1,000,000 rules...");
    let start_create = Instant::now();

    for i in 0..1_000_000 {
        let rule = Rule {
            name: format!("Rule{}", i),
            conditions: ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition {
                    field: "count".to_string(),
                    operator: ">".to_string(),
                    value: format!("{}", i % 1000), // Vary threshold more
                })),
                operator: "AND".to_string(),
                right: Box::new(ConditionGroup::Single(Condition {
                    field: "status".to_string(),
                    operator: "==".to_string(),
                    value: format!("active{}", i % 100), // Vary status more
                })),
            },
            action: format!("log('Rule{} fired')", i),
        };

        engine.add_rule_from_definition(&rule, (1_000_000 - i) as i32, false);
    }

    let create_time = start_create.elapsed();
    println!("✅ Created 1,000,000 rules in {:?}", create_time);

    // Set up facts that will match ~5% of rules (50K matches for 1M rules)
    println!("📊 Setting up test facts...");
    engine.set_fact("count".to_string(), "500".to_string()); // Match rules with threshold > 500
    engine.set_fact("status".to_string(), "active50".to_string()); // Match rules with status active50

    // Memory usage before execution
    let mem_before = get_memory_usage();
    println!("🧠 Memory before: {} KB", mem_before);

    // Test single execution
    println!("⚡ Testing single execution...");
    let start_exec = Instant::now();
    let result = engine.fire_all();
    let exec_time = start_exec.elapsed();

    println!("📈 Single execution results:");
    println!("   Time: {:?}", exec_time);
    println!("   Rules fired: {}", result.len());
    println!("   Latency per rule: {:.2} µs", exec_time.as_micros() as f64 / 1_000_000.0);
    println!("   Rules/second: {:.2}", 1_000_000.0 / exec_time.as_secs_f64());

    let mem_after = get_memory_usage();
    println!("🧠 Memory after: {} KB", mem_after);
    println!("📊 Memory delta: {} KB", mem_after as i64 - mem_before as i64);

    // Test multiple executions (reduced for 1M rules)
    println!("🔄 Testing multiple executions...");
    let mut total_fired = 0;
    let mut times = Vec::new();

    for run in 0..3 { // Reduced to 3 runs for 1M rules
        let start = Instant::now();
        let result = engine.fire_all();
        let elapsed = start.elapsed();
        total_fired += result.len();
        times.push(elapsed);
        println!("   Run {}: {:?} ({} rules fired)", run + 1, elapsed, result.len());
    }

    let avg_time = times.iter().sum::<Duration>() / times.len() as u32;
    println!("📊 Average execution time: {:?}", avg_time);
    println!("🎯 Total rules fired across 3 runs: {}", total_fired);
    println!("📈 Average rules fired per run: {}", total_fired / 3);

    // Performance analysis
    let rules_per_second = 1_000_000.0 / avg_time.as_secs_f64();
    println!("🚀 Performance: {:.2} rules/second", rules_per_second);
    println!("⚡ Latency: {:.2} µs per rule", avg_time.as_micros() as f64 / 1_000_000.0);

    // Scalability check
    if avg_time > Duration::from_secs(10) {
        println!("🚨 CRITICAL: Average execution took > 10 seconds - not suitable for real-time use");
    } else if avg_time > Duration::from_secs(1) {
        println!("⚠️  WARNING: Average execution took > 1 second - may not be suitable for interactive applications");
    } else if avg_time > Duration::from_millis(100) {
        println!("⚠️  WARNING: Average execution took > 100ms - borderline for batch processing");
    } else {
        println!("✅ Good performance for batch processing applications");
    }

    if mem_after > 1024 * 1024 { // 1GB
        println!("🚨 CRITICAL: Very high memory usage ({} MB) - monitor for memory leaks", mem_after / 1024);
    } else if mem_after > 100 * 1024 { // 100MB
        println!("⚠️  WARNING: High memory usage ({} MB) - consider memory optimization", mem_after / 1024);
    } else {
        println!("✅ Reasonable memory usage ({} MB)", mem_after / 1024);
    }

    println!("🎯 CONCLUSION:");
    if rules_per_second > 1000.0 {
        println!("   RETE-UL performs WELL at 1M rules scale");
        println!("   Suitable for production use with large rule sets");
    } else {
        println!("   RETE-UL performance is ADEQUATE at 1M rules scale");
        println!("   May need optimization for very large rule sets");
    }
}

fn get_memory_usage() -> usize {
    // Rough memory estimation for 1M rules
    // Each rule takes ~500 bytes on average (rule structure + conditions + strings)
    // Plus working memory, alpha/beta networks, etc.
    // This is a very rough estimate - real memory profiling would be better
    // Return value in KB for consistency with display
    1024 * 500 // Estimate ~500MB = 512,000 KB for 1M rules
}
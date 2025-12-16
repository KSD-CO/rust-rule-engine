use rust_rule_engine::parser::grl::GRLParser;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Purchasing Rules Parse Benchmark");
    println!("====================================\n");

    let rules_path = "examples/rules/05-performance/purchasing_rules.grl";
    println!("📄 File: {}", rules_path);
    let rules_content = fs::read_to_string(rules_path)?;

    // Analyze file
    let line_count = rules_content.lines().count();
    let char_count = rules_content.chars().count();

    println!("   Size: {} bytes", rules_content.len());
    println!("   Lines: {}", line_count);
    println!("   Characters: {}\n", char_count);

    // Quick parse to get rule count
    let parsed = GRLParser::parse_rules(&rules_content)?;
    println!("✅ Rules in file: {}\n", parsed.len());

    // Benchmark 1: Warmup and cold start
    println!("🔥 Benchmark 1: Cold vs Warm Parse");
    println!("-----------------------------------");

    let cold_start = Instant::now();
    let _ = GRLParser::parse_rules(&rules_content)?;
    let cold_time = cold_start.elapsed();
    println!("   Cold start: {:?}", cold_time);

    // Warm up
    for _ in 0..10 {
        let _ = GRLParser::parse_rules(&rules_content)?;
    }

    let warm_start = Instant::now();
    let _ = GRLParser::parse_rules(&rules_content)?;
    let warm_time = warm_start.elapsed();
    println!("   Warm parse: {:?}", warm_time);
    println!(
        "   Improvement: {:.1}x faster\n",
        cold_time.as_nanos() as f64 / warm_time.as_nanos() as f64
    );

    // Benchmark 2: Small sample size for precision
    println!("📈 Benchmark 2: Precision Test (n=100)");
    println!("--------------------------------------");
    benchmark_parse(&rules_content, 100)?;
    println!();

    // Benchmark 3: Medium sample for reliability
    println!("📈 Benchmark 3: Reliability Test (n=1000)");
    println!("-----------------------------------------");
    benchmark_parse(&rules_content, 1000)?;
    println!();

    // Benchmark 4: Large sample for throughput
    println!("📈 Benchmark 4: Throughput Test (n=5000)");
    println!("----------------------------------------");
    benchmark_parse(&rules_content, 5000)?;
    println!();

    // Benchmark 5: Stress test with memory pressure
    println!("💪 Benchmark 5: Memory Stress Test");
    println!("-----------------------------------");
    stress_test(&rules_content, 10000)?;
    println!();

    // Benchmark 6: Parallel-like scenario (sequential but rapid)
    println!("⚡ Benchmark 6: Rapid Sequential Parse");
    println!("--------------------------------------");
    rapid_parse_test(&rules_content, 1000)?;
    println!();

    println!("✅ All benchmarks completed!");

    Ok(())
}

fn benchmark_parse(content: &str, iterations: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = GRLParser::parse_rules(content)?;
        durations.push(start.elapsed());
    }

    let total: std::time::Duration = durations.iter().sum();
    let avg = total / iterations as u32;

    // Calculate percentiles
    durations.sort();
    let p50 = durations[iterations / 2];
    let p90 = durations[iterations * 90 / 100];
    let p95 = durations[iterations * 95 / 100];
    let p99 = durations[iterations * 99 / 100];
    let min = durations.first().unwrap();
    let max = durations.last().unwrap();

    println!("   Iterations: {}", iterations);
    println!("   Average: {:?}", avg);
    println!("   Median (P50): {:?}", p50);
    println!("   P90: {:?}", p90);
    println!("   P95: {:?}", p95);
    println!("   P99: {:?}", p99);
    println!("   Min: {:?}", min);
    println!("   Max: {:?}", max);
    println!("   Total time: {:?}", total);
    println!(
        "   Throughput: {:.2} parses/sec",
        iterations as f64 / total.as_secs_f64()
    );

    // Calculate standard deviation
    let mean_nanos = avg.as_nanos() as f64;
    let variance: f64 = durations
        .iter()
        .map(|d| {
            let diff = d.as_nanos() as f64 - mean_nanos;
            diff * diff
        })
        .sum::<f64>()
        / iterations as f64;
    let std_dev = variance.sqrt();
    let std_dev_duration = std::time::Duration::from_nanos(std_dev as u64);

    println!("   Std Dev: {:?}", std_dev_duration);
    println!(
        "   Coefficient of Variation: {:.2}%",
        (std_dev / mean_nanos) * 100.0
    );

    Ok(())
}

fn stress_test(content: &str, iterations: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut all_parsed = Vec::with_capacity(iterations / 100);
    let mut durations = Vec::with_capacity(iterations);

    let start = Instant::now();

    for i in 0..iterations {
        let parse_start = Instant::now();
        let parsed = GRLParser::parse_rules(content)?;
        durations.push(parse_start.elapsed());

        // Keep every 100th result to simulate memory retention
        if i % 100 == 0 {
            all_parsed.push(parsed);
        }
    }

    let total_time = start.elapsed();
    let avg = total_time / iterations as u32;

    println!("   Iterations: {}", iterations);
    println!("   Total time: {:?}", total_time);
    println!("   Average: {:?}", avg);
    println!(
        "   Throughput: {:.2} parses/sec",
        iterations as f64 / total_time.as_secs_f64()
    );
    println!(
        "   Retained results: {} (simulating memory pressure)",
        all_parsed.len()
    );
    println!(
        "   Memory retained: ~{} rules",
        all_parsed.len() * all_parsed.first().map_or(0, |v| v.len())
    );

    Ok(())
}

fn rapid_parse_test(content: &str, iterations: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("   Running {} rapid parses with no delays...", iterations);

    let start = Instant::now();
    let mut success_count = 0;

    for _ in 0..iterations {
        match GRLParser::parse_rules(content) {
            Ok(_) => success_count += 1,
            Err(e) => eprintln!("   Parse error: {}", e),
        }
    }

    let total_time = start.elapsed();
    let avg = total_time / iterations as u32;

    println!("   Successful parses: {}/{}", success_count, iterations);
    println!("   Total time: {:?}", total_time);
    println!("   Average: {:?}", avg);
    println!(
        "   Throughput: {:.2} parses/sec",
        iterations as f64 / total_time.as_secs_f64()
    );

    // Calculate parses per millisecond
    let parses_per_ms = iterations as f64 / total_time.as_millis() as f64;
    println!("   Rate: {:.2} parses/ms", parses_per_ms);

    Ok(())
}

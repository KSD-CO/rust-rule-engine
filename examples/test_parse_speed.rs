use rust_rule_engine::parser::grl::GRLParser;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules_content = fs::read_to_string("examples/rules/purchasing_rules.grl")?;
    println!("Testing GRL parsing performance");
    println!("File size: {} bytes\n", rules_content.len());
    
    // Warm up
    let _ = GRLParser::parse_rules(&rules_content)?;
    
    // Test with 100 iterations (smaller than original 1000 for faster feedback)
    let iterations = 100;
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _rules = GRLParser::parse_rules(&rules_content)?;
    }
    
    let elapsed = start.elapsed();
    let avg_per_parse = elapsed.as_micros() as f64 / iterations as f64;
    
    println!("Iterations: {}", iterations);
    println!("Total time: {:?}", elapsed);
    println!("Average per parse: {:.2} µs", avg_per_parse);
    println!("Throughput: {:.2} parses/sec", iterations as f64 / elapsed.as_secs_f64());
    
    Ok(())
}

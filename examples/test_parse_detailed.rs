use rust_rule_engine::parser::grl::GRLParser;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules_content = fs::read_to_string("examples/rules/purchasing_rules.grl")?;
    println!("Testing GRL parsing performance with detailed timing\n");
    println!("File size: {} bytes", rules_content.len());
    
    // Count rules in file
    let rule_count = rules_content.matches("rule ").count();
    println!("Rules in file: {}\n", rule_count);
    
    // First parse - includes potential lazy initialization
    println!("=== First Parse (cold) ===");
    let start = Instant::now();
    let result1 = GRLParser::parse_rules(&rules_content)?;
    let elapsed1 = start.elapsed();
    println!("Time: {:?}", elapsed1);
    println!("Rules parsed: {}\n", result1.len());
    
    // Second parse - should be faster if lazy statics already initialized
    println!("=== Second Parse (warm) ===");
    let start = Instant::now();
    let result2 = GRLParser::parse_rules(&rules_content)?;
    let elapsed2 = start.elapsed();
    println!("Time: {:?}", elapsed2);
    println!("Rules parsed: {}\n", result2.len());
    
    // Third parse
    println!("=== Third Parse (warm) ===");
    let start = Instant::now();
    let result3 = GRLParser::parse_rules(&rules_content)?;
    let elapsed3 = start.elapsed();
    println!("Time: {:?}", elapsed3);
    println!("Rules parsed: {}\n", result3.len());
    
    // Average of warm parses
    let warm_avg = (elapsed2.as_micros() + elapsed3.as_micros()) as f64 / 2.0 / 1000.0;
    println!("Average warm parse time: {:.3} ms", warm_avg);
    
    Ok(())
}

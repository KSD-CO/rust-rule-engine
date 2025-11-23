use rust_rule_engine::parser::grl::GRLParser;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rules_content = fs::read_to_string("examples/rules/purchasing_rules.grl")?;
    
    // Parse just one rule
    let single_rule = r#"
rule "CalculateShortage" salience 120 no-loop {
    when
        required_qty > 0
    then
        Log("Calculating shortage...");
        shortage = required_qty - available_qty;
        Log("Shortage calculated");
}
"#;
    
    println!("Testing single rule parse performance\n");
    println!("Rule size: {} bytes\n", single_rule.len());
    
    // Warm up
    let _ = GRLParser::parse_rule(single_rule)?;
    
    // Measure many single parses
    let iterations = 10000;
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = GRLParser::parse_rule(single_rule)?;
    }
    let elapsed = start.elapsed();
    
    let micros_per_parse = elapsed.as_micros() as f64 / iterations as f64;
    println!("Iterations: {}", iterations);
    println!("Total time: {:?}", elapsed);
    println!("Avg per parse: {:.2} µs ({:.4} ms)", micros_per_parse, micros_per_parse / 1000.0);
    println!("Throughput: {:.2} parses/sec", iterations as f64 / elapsed.as_secs_f64());
    
    Ok(())
}

use rust_rule_engine::parser::grl::GRLParser;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Complex Rule Parsing...\n");

    // Read the test file
    let grl_content = fs::read_to_string("examples/rules/test_complex_rule.grl")?;
    println!("📄 Input GRL content:");
    println!("{}", grl_content);
    println!("{}", "=".repeat(50));

    // Parse rules
    match GRLParser::parse_rules(&grl_content) {
        Ok(rules) => {
            println!("✅ Successfully parsed {} rules!\n", rules.len());

            for (i, rule) in rules.iter().enumerate() {
                println!("🔧 Rule {} Details:", i + 1);
                println!("   Name: {}", rule.name);
                println!("   Salience: {}", rule.salience);
                println!("   Conditions: {:?}", rule.conditions);
                println!("   Actions: {:?}", rule.actions);
                println!();
            }
        }
        Err(e) => {
            println!("❌ Parse error: {}", e);
            return Err(Box::new(e));
        }
    }

    println!("🎉 Complex rule parsing test completed successfully!");
    Ok(())
}

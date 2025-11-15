///! Test Accumulate Parser
///!
///! This example tests parsing accumulate() syntax from GRL files

use rust_rule_engine::GRLParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Accumulate Parser");
    println!("===========================\n");

    let grl_path = "examples/rules/accumulate_test.grl";
    println!("📄 Reading GRL file: {}", grl_path);

    let grl_content = std::fs::read_to_string(grl_path)?;
    println!("✓ File read successfully\n");

    println!("📋 Parsing rules...");
    match GRLParser::parse_rules(&grl_content) {
        Ok(rules) => {
            println!("✅ Successfully parsed {} rules\n", rules.len());

            for rule in &rules {
                println!("───────────────────────────────");
                println!("Rule: {}", rule.name);
                println!("Salience: {}", rule.salience);
                println!("Conditions: {:?}", rule.conditions);
                println!("Actions: {:?}", rule.actions);
                println!();
            }

            println!("✅ All rules parsed successfully!");
        }
        Err(e) => {
            println!("❌ Parsing failed: {:?}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

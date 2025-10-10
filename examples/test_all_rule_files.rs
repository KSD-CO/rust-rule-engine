use rust_rule_engine::parser::grl::GRLParser;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing All Rule Files in examples/rules/\n");

    let rules_dir = "examples/rules";
    let rule_files = [
        "test_complex_rule.grl",
        "simple_business_rules.grl",
        "advanced_nested_rules.grl",
        "legacy_format_rules.grl",
    ];

    let mut total_rules = 0;

    for rule_file in &rule_files {
        let file_path = Path::new(rules_dir).join(rule_file);
        println!("📄 Testing: {}", rule_file);
        println!("{}", "=".repeat(50));

        match fs::read_to_string(&file_path) {
            Ok(content) => match GRLParser::parse_rules(&content) {
                Ok(rules) => {
                    println!("✅ Successfully parsed {} rules", rules.len());
                    total_rules += rules.len();

                    for (i, rule) in rules.iter().enumerate() {
                        println!(
                            "   🔧 Rule {}: \"{}\" (salience: {})",
                            i + 1,
                            rule.name,
                            rule.salience
                        );
                    }
                }
                Err(e) => {
                    println!("❌ Parse error: {}", e);
                    return Err(Box::new(e));
                }
            },
            Err(e) => {
                println!("❌ Failed to read file: {}", e);
                return Err(Box::new(e));
            }
        }
        println!();
    }

    println!("🎉 All rule files tested successfully!");
    println!("📊 Total rules parsed: {}", total_rules);

    Ok(())
}

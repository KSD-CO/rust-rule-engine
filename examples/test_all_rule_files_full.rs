use rust_rule_engine::parser::grl::GRLParser;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing ALL .grl files in examples/rules/ (dynamic)");

    let rules_dir = Path::new("examples/rules");
    let mut total_rules = 0usize;
    let mut files_tested = 0usize;

    for entry in rules_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "grl" {
                    files_tested += 1;
                    let fname = path.file_name().unwrap().to_string_lossy().to_string();
                    println!("\n📄 Testing: {}", fname);
                    println!("{}", "=".repeat(60));
                    let content = fs::read_to_string(&path)?;
                    match GRLParser::parse_rules(&content) {
                        Ok(rules) => {
                            println!("✅ Parsed {} rule(s)", rules.len());
                            total_rules += rules.len();
                        }
                        Err(e) => {
                            println!("❌ Parse error for {}: {}", fname, e);
                        }
                    }
                }
            }
        }
    }

    println!("\n🎉 Done. Files tested: {}. Total rules parsed: {}", files_tested, total_rules);
    Ok(())
}

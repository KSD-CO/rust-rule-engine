// Helper: convert parser Rule/ConditionGroup to auto_network Rule/ConditionGroup
fn convert_condition_group(src: &rust_rule_engine::ConditionGroup) -> rust_rule_engine::rete::auto_network::ConditionGroup {
    use rust_rule_engine::rete::auto_network::{ConditionGroup as AutoGroup, Condition as AutoCond};
    match src {
        rust_rule_engine::ConditionGroup::Single(cond) => {
            AutoGroup::Single(AutoCond {
                field: cond.field.clone(),
                operator: format!("{:?}", cond.operator),
                value: cond.value.to_string(),
            })
        }
        rust_rule_engine::ConditionGroup::Compound { left, operator, right } => {
            AutoGroup::Compound {
                left: Box::new(convert_condition_group(left)),
                operator: format!("{:?}", operator),
                right: Box::new(convert_condition_group(right)),
            }
        }
        rust_rule_engine::ConditionGroup::Not(inner) => {
            AutoGroup::Not(Box::new(convert_condition_group(inner)))
        }
        rust_rule_engine::ConditionGroup::Exists(inner) => {
            AutoGroup::Exists(Box::new(convert_condition_group(inner)))
        }
        rust_rule_engine::ConditionGroup::Forall(inner) => {
            AutoGroup::Forall(Box::new(convert_condition_group(inner)))
        }
        rust_rule_engine::ConditionGroup::Accumulate { .. } => {
            // Accumulate is not supported in auto_network yet
            AutoGroup::Single(AutoCond {
                field: "true".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            })
        }
    }
}

fn convert_rule(src: &rust_rule_engine::Rule) -> rust_rule_engine::rete::auto_network::Rule {
    rust_rule_engine::rete::auto_network::Rule {
        name: src.name.clone(),
        conditions: convert_condition_group(&src.conditions),
        action: src.actions.get(0).map(|a| format!("{:?}", a)).unwrap_or_default(),
    }
}

// Helper: collect all leaf conditions from ConditionGroup
fn collect_conditions<'a>(group: &'a rust_rule_engine::ConditionGroup, out: &mut Vec<&'a rust_rule_engine::Condition>) {
    match group {
        rust_rule_engine::ConditionGroup::Single(cond) => out.push(cond),
        rust_rule_engine::ConditionGroup::Compound { left, right, .. } => {
            collect_conditions(left, out);
            collect_conditions(right, out);
        }
        rust_rule_engine::ConditionGroup::Not(inner)
        | rust_rule_engine::ConditionGroup::Exists(inner)
        | rust_rule_engine::ConditionGroup::Forall(inner) => {
            collect_conditions(inner, out);
        }
        rust_rule_engine::ConditionGroup::Accumulate { .. } => {
            // Accumulate doesn't have simple conditions to collect
        }
    }
}
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

                    use rust_rule_engine::rete::auto_network::{build_rete_ul_from_rule, Rule as AutoRule};
                    use rust_rule_engine::rete::evaluate_rete_ul_node;
                    use std::collections::HashMap;

                    for (i, rule) in rules.iter().enumerate() {
                        println!(
                            "   🔧 Rule {}: \"{}\" (salience: {})",
                            i + 1,
                            rule.name,
                            rule.salience
                        );

                        // Chuyển rule sang mạng RETE tự động
                        let auto_rule = convert_rule(rule);
                        let rete_node = build_rete_ul_from_rule(&auto_rule);

                            // Tạo facts khớp với điều kiện của rule để test fire
                            let mut facts = HashMap::new();
                            let mut conds = Vec::new();
                            collect_conditions(&rule.conditions, &mut conds);
                            for cond in &conds {
                                let op_str = format!("{:?}", cond.operator);
                                let value_str = cond.value.to_string();
                                let value = match op_str.as_str() {
                                    ">" | ">=" => {
                                        match value_str.parse::<i64>() {
                                            Ok(v) => (v + 1).to_string(),
                                            Err(_) => value_str.clone(),
                                        }
                                    }
                                    "<" | "<=" => {
                                        match value_str.parse::<i64>() {
                                            Ok(v) => (v - 1).to_string(),
                                            Err(_) => value_str.clone(),
                                        }
                                    }
                                    "==" | "=" => value_str.clone(),
                                    "Contains" => value_str.clone(),
                                    _ => value_str.clone(),
                                };
                                facts.insert(cond.field.clone(), value);
                            }
                            // In ra facts để debug
                            println!("      📝 Facts for test: {:?}", facts);
                            let matched = evaluate_rete_ul_node(&rete_node, &facts);
                            println!("      ➡ RETE match with sample facts: {}", matched);
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

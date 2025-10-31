use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::{
    engine::{EngineConfig, RustRuleEngine},
    errors::Result,
    types::Value,
    Facts, KnowledgeBase,
};
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("🔄 GRL No-Loop Parsing Demo");
    println!("============================");

    // Test parsing GRL with no-loop
    test_grl_no_loop_parsing()?;

    println!();

    // Test execution with parsed no-loop rules
    test_no_loop_execution()?;

    Ok(())
}

fn test_grl_no_loop_parsing() -> Result<()> {
    println!("📋 Testing GRL No-Loop Parsing");
    println!("-------------------------------");

    let grl_content = std::fs::read_to_string("examples/rules/no_loop_test.grl")
        .expect("Failed to read no_loop_test.grl");

    let rules = GRLParser::parse_rules(&grl_content)?;

    println!("✅ Parsed {} rules from GRL file:", rules.len());

    for rule in &rules {
        println!(
            "  📝 Rule: '{}' (salience: {}, no-loop: {})",
            rule.name, rule.salience, rule.no_loop
        );
    }

    // Verify no-loop flags are correctly parsed
    let safe_incrementer = rules.iter().find(|r| r.name == "SafeScoreIncrementer");
    let bonus_applier = rules.iter().find(|r| r.name == "BonusApplier");
    let score_incrementer = rules.iter().find(|r| r.name == "ScoreIncrementer");

    if let Some(rule) = safe_incrementer {
        assert!(
            rule.no_loop,
            "SafeScoreIncrementer should have no-loop=true"
        );
        println!("  ✅ SafeScoreIncrementer correctly parsed with no-loop=true");
    }

    if let Some(rule) = bonus_applier {
        assert!(rule.no_loop, "BonusApplier should have no-loop=true");
        println!("  ✅ BonusApplier correctly parsed with no-loop=true");
    }

    if let Some(rule) = score_incrementer {
        assert!(!rule.no_loop, "ScoreIncrementer should have no-loop=false");
        println!("  ✅ ScoreIncrementer correctly parsed with no-loop=false");
    }

    Ok(())
}

fn test_no_loop_execution() -> Result<()> {
    println!("📋 Testing No-Loop Execution");
    println!("-----------------------------");

    use rust_rule_engine::engine::coverage::RuleCoverage;

    // Khởi tạo coverage
    let mut coverage = RuleCoverage::new();

    // Khởi tạo engine
    let config = EngineConfig {
        max_cycles: 10,
        debug_mode: false,
        ..Default::default()
    };
    let kb = KnowledgeBase::new("NoLoopTest");
    let grl_content = std::fs::read_to_string("examples/rules/no_loop_test.grl")
        .expect("Failed to read no_loop_test.grl");
    let rules = GRLParser::parse_rules(&grl_content)?;
    for rule in rules {
        let _ = kb.add_rule(rule);
    }
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Đăng ký function và action handler
    engine.register_function("set", |_, _| Ok(Value::Boolean(true)));
    engine.register_function("log", |_, _| Ok(Value::Boolean(true)));
    engine.register_action_handler("set", |_, _| Ok(()));

    // Sinh test case tự động cho từng rule
    let rules = engine.knowledge_base().get_rules().clone();
    let mut total_run = 0;
    for rule in &rules {
        let test_facts_list = rust_rule_engine::engine::coverage::generate_test_facts_for_rule(rule);
        for (i, facts) in test_facts_list.iter().enumerate() {
            let _ = engine.execute_with_callback(facts, |rule_name, _facts_id| {
                coverage.record_hit(rule_name, &format!("{}_{}_auto", rule_name, i));
            });
            total_run += 1;
        }
    }

    let all_rule_names: Vec<String> = rules.iter().map(|r| r.name.clone()).collect();
    println!("\n=== Auto Test Coverage ===");
    println!("Test cases auto-generated: {}", total_run);
    println!("{}", coverage.report(&all_rule_names));

    Ok(())
}

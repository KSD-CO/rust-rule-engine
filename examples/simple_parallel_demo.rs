use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::ParallelConfig;
use rust_rule_engine::engine::ParallelRuleEngine;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Simple Parallel Rule Engine Demo");
    println!("====================================\n");

    // Create test data
    let facts = create_simple_facts();
    let kb = create_simple_kb()?;

    println!("🔧 Created {} rules for testing", kb.get_rules().len());

    // Test parallel execution with different configurations
    test_parallel_configs(&kb, &facts)?;

    println!("\n✅ Parallel execution concept demonstrated!");
    println!("🎯 Next steps for full implementation:");
    println!("   - 🔧 Integrate with actual condition evaluation");
    println!("   - 🚀 Add real action execution");
    println!("   - 📊 Optimize threading strategy");
    println!("   - 🔍 Add dependency analysis");

    Ok(())
}

fn create_simple_facts() -> Facts {
    let facts = Facts::new();
    facts.set("User", {
        let mut user = HashMap::new();
        user.insert("Age".to_string(), Value::Number(25.0));
        user.insert("Country".to_string(), Value::String("US".to_string()));
        user.insert("IsActive".to_string(), Value::Boolean(true));
        Value::Object(user)
    });
    facts
}

fn create_simple_kb() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let mut kb = KnowledgeBase::new("SimpleTestKB");

    let rules = vec![
        r#"rule "Rule1" salience 10 {
            when User.Age >= 18
            then log("Rule1 fired");
        }"#,
        r#"rule "Rule2" salience 10 {
            when User.Country == "US"
            then log("Rule2 fired");
        }"#,
        r#"rule "Rule3" salience 9 {
            when User.IsActive == true
            then log("Rule3 fired");
        }"#,
        r#"rule "Rule4" salience 8 {
            when User.Age > 20
            then log("Rule4 fired");
        }"#,
        r#"rule "Rule5" salience 8 {
            when User.Country != "UK"
            then log("Rule5 fired");
        }"#,
    ];

    for rule_str in rules {
        kb.add_rules_from_grl(rule_str)?;
    }

    Ok(kb)
}

fn test_parallel_configs(
    kb: &KnowledgeBase, 
    facts: &Facts
) -> Result<(), Box<dyn std::error::Error>> {
    let configs = vec![
        ("Single Thread", ParallelConfig {
            enabled: true,
            max_threads: 1,
            min_rules_per_thread: 1,
            dependency_analysis: false,
        }),
        ("Multi Thread", ParallelConfig {
            enabled: true,
            max_threads: 4,
            min_rules_per_thread: 1,
            dependency_analysis: false,
        }),
        ("Default Config", ParallelConfig::default()),
        ("Disabled", ParallelConfig {
            enabled: false,
            ..Default::default()
        }),
    ];

    for (name, config) in configs {
        println!("\n🧪 Testing {} configuration:", name);
        println!("   Enabled: {}", config.enabled);
        println!("   Max threads: {}", config.max_threads);
        
        let start = Instant::now();
        let engine = ParallelRuleEngine::new(config);
        let result = engine.execute_parallel(kb, facts, true)?;
        let execution_time = start.elapsed();

        println!("   ⏱️  Execution time: {:?}", execution_time);
        println!("   📊 Rules evaluated: {}", result.total_rules_evaluated);
        println!("   🔥 Rules fired: {}", result.total_rules_fired);
        println!("   📈 Theoretical speedup: {:.2}x", result.parallel_speedup);
        
        // Show per-salience execution
        let mut salience_counts = HashMap::new();
        for context in &result.execution_contexts {
            *salience_counts.entry(context.rule.salience).or_insert(0) += 1;
        }
        
        for (&salience, &count) in &salience_counts {
            println!("     📋 Salience {}: {} rules", salience, count);
        }
    }

    Ok(())
}

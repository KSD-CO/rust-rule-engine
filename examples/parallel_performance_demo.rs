use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::parallel::{ParallelConfig, ParallelRuleEngine};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Parallel vs Sequential Performance Demo");
    println!("==========================================");
    println!();

    // Create test data
    let rule_count = 50;
    let user_count = 100;

    let facts = setup_facts(user_count);
    let rules_str = create_rules(rule_count);

    println!("📊 Test Configuration:");
    println!("   Rules: {}", rule_count);
    println!("   Users: {}", user_count);
    println!();

    // Test Sequential Execution
    println!("🔄 Testing Sequential Execution...");
    let sequential_time = test_sequential(&rules_str, &facts)?;
    println!("   ⏱️  Sequential time: {:?}", sequential_time);
    println!();

    // Test Parallel Execution with different thread counts
    for threads in [2, 4, 8] {
        println!("⚡ Testing Parallel Execution ({} threads)...", threads);
        let parallel_time = test_parallel(&rules_str, &facts, threads)?;
        let speedup = sequential_time.as_nanos() as f64 / parallel_time.as_nanos() as f64;
        println!("   ⏱️  Parallel time: {:?}", parallel_time);
        println!("   📈 Speedup: {:.2}x", speedup);
        println!();
    }

    println!("✅ Performance comparison completed!");
    println!();
    println!("📝 Notes:");
    println!("• Speedup depends on rule complexity and system resources");
    println!("• Optimal thread count varies by workload");
    println!("• Parallel overhead may reduce gains for small rule sets");

    Ok(())
}

fn setup_facts(user_count: usize) -> Facts {
    let facts = Facts::new();

    for i in 0..user_count {
        let mut user = HashMap::new();
        user.insert("Id".to_string(), Value::String(format!("USER{:03}", i)));
        user.insert("Age".to_string(), Value::Integer(20 + (i % 50) as i64));
        user.insert(
            "SpendingTotal".to_string(),
            Value::Number(100.0 + (i as f64 * 50.0)),
        );
        user.insert("IsAdult".to_string(), Value::Boolean(true));

        facts
            .add_value(&format!("User{}", i), Value::Object(user))
            .unwrap();
    }

    facts
}

fn create_rules(rule_count: usize) -> String {
    let mut rules = String::new();

    for i in 0..rule_count {
        let user_idx = i % 100;
        let rule = format!(
            r#"
            rule "Rule{}" salience {} {{
                when
                    User{}.Age > {} && User{}.SpendingTotal > {}
                then
                    log("Rule {} executed for User{}");
            }}
        "#,
            i,
            100 - (i % 20),
            user_idx,
            18 + (i % 10),
            user_idx,
            200.0 + (i as f64 * 10.0),
            i,
            user_idx
        );
        rules.push_str(&rule);
    }

    rules
}

fn test_sequential(
    rules_str: &str,
    facts: &Facts,
) -> Result<std::time::Duration, Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new("SequentialTest");
    let parsed_rules = GRLParser::parse_rules(rules_str)?;

    for rule in parsed_rules {
        kb.add_rule(rule)?;
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 5,
        ..Default::default()
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    let start = Instant::now();
    let _result = engine.execute(facts)?;
    Ok(start.elapsed())
}

fn test_parallel(
    rules_str: &str,
    facts: &Facts,
    threads: usize,
) -> Result<std::time::Duration, Box<dyn std::error::Error>> {
    let config = ParallelConfig {
        enabled: true,
        max_threads: threads,
        min_rules_per_thread: 2,
        dependency_analysis: false,
    };

    let engine = ParallelRuleEngine::new(config);

    let kb = KnowledgeBase::new("ParallelTest");
    let parsed_rules = GRLParser::parse_rules(rules_str)?;

    for rule in parsed_rules {
        kb.add_rule(rule)?;
    }

    let start = Instant::now();
    let _result = engine.execute_parallel(&kb, facts, false)?;
    Ok(start.elapsed())
}

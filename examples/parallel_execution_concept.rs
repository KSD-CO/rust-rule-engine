use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Parallel Rule Execution Concept Demo");
    println!("========================================\n");

    demo_sequential_vs_parallel()?;

    Ok(())
}

fn demo_sequential_vs_parallel() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Demo: Sequential vs Parallel Rule Execution");
    println!("----------------------------------------------");

    // Create facts for testing
    let facts = Facts::new();
    facts.set("User", {
        let mut user = HashMap::new();
        user.insert("Age".to_string(), Value::Number(30.0));
        user.insert("Country".to_string(), Value::String("US".to_string()));
        user.insert("SpendingTotal".to_string(), Value::Number(1500.0));
        user.insert("IsVIP".to_string(), Value::Boolean(false));
        user.insert("Category".to_string(), Value::String("unknown".to_string()));
        Value::Object(user)
    });

    // Create multiple independent rules with artificial delays
    let kb = create_test_knowledge_base()?;
    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 1,
        ..Default::default()
    };

    println!("🔧 Created {} rules for testing", kb.get_rules().len());

    // Test 1: Sequential Execution (Current Implementation)
    println!("\n🐌 Sequential Execution:");
    let start = Instant::now();
    let mut engine = RustRuleEngine::with_config(kb.clone(), config.clone());

    // Register simple task function
    engine.register_function(
        "simpleTask",
        Box::new(|args: &[Value], _facts: &Facts| {
            if let Some(Value::String(msg)) = args.first() {
                println!("     📝 {}", msg);
            }
            Ok(Value::Null)
        }),
    );

    let result = engine.execute(&facts)?;
    let sequential_time = start.elapsed();

    println!("   ⏱️  Time: {:?}", sequential_time);
    println!("   🔥 Rules fired: {}", result.rules_fired);
    println!("   📊 Rules evaluated: {}", result.rules_evaluated);

    // Test 2: Parallel Execution Concept (Demo implementation)
    println!("\n⚡ Parallel Execution Concept:");
    let start = Instant::now();
    let parallel_result = simulate_parallel_execution(&kb, &facts)?;
    let parallel_time = start.elapsed();

    println!("   ⏱️  Time: {:?}", parallel_time);
    println!("   🔥 Rules fired: {}", parallel_result.rules_fired);
    println!("   📊 Rules evaluated: {}", parallel_result.rules_evaluated);

    // Performance comparison
    println!("\n📈 Performance Comparison:");
    if sequential_time > parallel_time {
        let speedup = sequential_time.as_millis() as f64 / parallel_time.as_millis() as f64;
        println!("   🚀 Parallel is {:.2}x faster!", speedup);
    } else {
        println!("   ⚠️  Sequential was faster (overhead from threading)");
    }

    println!("\n💡 Parallel Rule Execution Benefits:");
    println!("   ✅ Better performance with many independent rules");
    println!("   ✅ CPU utilization on multi-core systems");
    println!("   ✅ Reduced latency for large rule sets");
    println!("   ⚠️  Need to handle data dependencies carefully");
    println!("   ⚠️  Memory synchronization overhead");

    Ok(())
}

fn create_test_knowledge_base() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let mut kb = KnowledgeBase::new("ParallelTestKB");

    // Add multiple independent rules with different priorities
    let rules = vec![
        r#"rule "Rule1" salience 10 {
            when
                User.Age >= 18
            then
                simpleTask("Rule1 executed");
        }"#,
        r#"rule "Rule2" salience 9 {
            when
                User.Country == "US"
            then
                simpleTask("Rule2 executed");
        }"#,
        r#"rule "Rule3" salience 8 {
            when
                User.SpendingTotal > 1000.0
            then
                simpleTask("Rule3 executed");
        }"#,
        r#"rule "Rule4" salience 7 {
            when
                User.IsVIP == false
            then
                simpleTask("Rule4 executed");
        }"#,
        r#"rule "Rule5" salience 6 {
            when
                User.Category == "unknown"
            then
                simpleTask("Rule5 executed");
        }"#,
    ];

    for rule_str in rules {
        kb.add_rules_from_grl(rule_str)?;
    }

    Ok(kb)
}

// Simulated parallel execution concept
fn simulate_parallel_execution(
    kb: &KnowledgeBase,
    facts: &Facts,
) -> Result<ParallelExecutionResult, Box<dyn std::error::Error>> {
    let rules = kb.get_rules().clone();
    let _handles: Vec<thread::JoinHandle<()>> = vec![];
    let results = Arc::new(Mutex::new(Vec::new()));

    // Group rules by salience level for parallel execution within same priority
    let mut salience_groups: HashMap<i32, Vec<_>> = HashMap::new();
    for rule in rules {
        salience_groups
            .entry(rule.salience)
            .or_insert_with(Vec::new)
            .push(rule);
    }

    let mut total_evaluated = 0;
    let mut total_fired = 0;

    // Execute rules by salience level (highest first)
    let mut salience_levels: Vec<_> = salience_groups.keys().collect();
    salience_levels.sort_by(|a, b| b.cmp(a)); // Descending order

    for &salience in salience_levels {
        let rules_at_level = &salience_groups[&salience];
        println!(
            "   🔄 Processing {} rules at salience level {}",
            rules_at_level.len(),
            salience
        );

        // Parallel execution within same salience level
        let results_clone = Arc::clone(&results);
        let handles: Vec<_> = rules_at_level
            .iter()
            .enumerate()
            .map(|(i, rule)| {
                let rule_clone = rule.clone();
                let facts_clone = facts.clone();
                let results_clone = Arc::clone(&results_clone);

                thread::spawn(move || {
                    // Simulate rule evaluation time
                    thread::sleep(Duration::from_millis(10 + i as u64 * 5));

                    // Simple condition check (in real implementation, use actual engine logic)
                    let fired = simulate_rule_evaluation(&rule_clone, &facts_clone);

                    let mut results = results_clone.lock().unwrap();
                    results.push(RuleResult {
                        rule_name: rule_clone.name.clone(),
                        fired,
                        evaluated: true,
                    });

                    if fired {
                        println!("     🔥 {} fired", rule_clone.name);
                    }
                })
            })
            .collect();

        // Wait for all rules at this salience level to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Count results for this level
        let results_guard = results.lock().unwrap();
        for result in results_guard.iter() {
            if result.evaluated {
                total_evaluated += 1;
            }
            if result.fired {
                total_fired += 1;
            }
        }
    }

    Ok(ParallelExecutionResult {
        rules_evaluated: total_evaluated,
        rules_fired: total_fired,
    })
}

#[derive(Debug)]
struct RuleResult {
    rule_name: String,
    fired: bool,
    evaluated: bool,
}

#[derive(Debug)]
struct ParallelExecutionResult {
    rules_evaluated: usize,
    rules_fired: usize,
}

// Simplified rule evaluation for demo
fn simulate_rule_evaluation(rule: &rust_rule_engine::engine::rule::Rule, _facts: &Facts) -> bool {
    // For demo purposes, just return true if rule has a name
    // In real implementation, this would use the actual condition evaluation logic
    !rule.name.is_empty()
}

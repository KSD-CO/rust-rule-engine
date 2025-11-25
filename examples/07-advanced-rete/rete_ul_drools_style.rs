use rust_rule_engine::rete::auto_network::{Rule, build_rete_ul_from_rule};
use rust_rule_engine::rete::network::{ReteUlEngine, build_rete_ul_from_condition_group};
use rust_rule_engine::rete::evaluate_rete_ul_node;
use std::collections::HashMap;
use std::time::Instant;

/// OLD VERSION: Simulate Drools-style RETE: repeatedly fire rules and update facts until no more matches
/// WARNING: This version REBUILDS nodes every iteration - SLOW!
fn rete_ul_fire_all_rules_slow(rules: &[Rule], facts: &mut HashMap<String, String>) -> Vec<String> {
    let mut fired_rules = Vec::new();
    let mut loop_count = 0;
    loop {
        let mut any_fired = false;
        for rule in rules {
            let rete_node = build_rete_ul_from_rule(rule); // ❌ BAD: Rebuild every time!
            let matched = evaluate_rete_ul_node(&rete_node, facts);
            if matched && !fired_rules.contains(&rule.name) {
                fired_rules.push(rule.name.clone());
                any_fired = true;
                // Simulate action: set a flag for each rule
                facts.insert(format!("{}_fired", rule.name), "true".to_string());
                // You can extend here to support more complex actions
                println!("RETE-UL fired: {}", rule.name);
            }
        }
        loop_count += 1;
        if !any_fired || loop_count > 20 { break; }
    }
    fired_rules
}

/// NEW VERSION: Use ReteUlEngine with cached nodes - FAST!
fn rete_ul_fire_all_rules_fast(rules: &[Rule], facts: &mut HashMap<String, String>) -> Vec<String> {
    let mut engine = ReteUlEngine::new();

    // Build nodes ONCE and cache
    for rule in rules {
        let node = build_rete_ul_from_condition_group(&rule.conditions);
        let rule_name = rule.name.clone();
        engine.add_rule_with_action(
            rule.name.clone(),
            node,
            0,
            true,
            move |_facts| {
                println!("RETE-UL fired: {}", rule_name);
            },
        );
    }

    // Set facts
    for (k, v) in facts.iter() {
        engine.set_fact(k.clone(), v.clone());
    }

    // Fire all (using cached nodes!)
    let fired = engine.fire_all();

    // Update original facts
    for (k, v) in engine.get_all_facts().iter() {
        facts.insert(k.clone(), v.clone());
    }

    fired
}

fn main() {
    println!("\n🔬 RETE-UL Drools-style Example (Performance Comparison)");
    println!("=========================================================\n");

    // Simple rules
    let simple_rules = vec![
        Rule {
            name: "IsAdult".to_string(),
            conditions: rust_rule_engine::rete::auto_network::ConditionGroup::Single(
                rust_rule_engine::rete::auto_network::Condition {
                    field: "age".to_string(),
                    operator: ">=".to_string(),
                    value: "18".to_string(),
                }
            ),
            action: "set is_adult_fired".to_string(),
        },
        Rule {
            name: "IsActive".to_string(),
            conditions: rust_rule_engine::rete::auto_network::ConditionGroup::Single(
                rust_rule_engine::rete::auto_network::Condition {
                    field: "status".to_string(),
                    operator: "==".to_string(),
                    value: "active".to_string(),
                }
            ),
            action: "set is_active_fired".to_string(),
        },
        Rule {
            name: "HighScore".to_string(),
            conditions: rust_rule_engine::rete::auto_network::ConditionGroup::Single(
                rust_rule_engine::rete::auto_network::Condition {
                    field: "score".to_string(),
                    operator: ">".to_string(),
                    value: "80".to_string(),
                }
            ),
            action: "set high_score_fired".to_string(),
        },
    ];

    // Test 1: Fast version (cached nodes)
    println!("🚀 Test 1: FAST version (with cached nodes)");
    let mut facts1 = HashMap::new();
    facts1.insert("age".to_string(), "25".to_string());
    facts1.insert("status".to_string(), "active".to_string());
    facts1.insert("score".to_string(), "88.5".to_string());
    println!("Initial facts: age=25, status=active, score=88.5");

    let start1 = Instant::now();
    let fired1 = rete_ul_fire_all_rules_fast(&simple_rules, &mut facts1);
    let duration1 = start1.elapsed();
    println!("Fired rules: {:?}", fired1);
    println!("Time: {:?}\n", duration1);

    // Test 2: Slow version (rebuild nodes every time)
    println!("🐌 Test 2: SLOW version (rebuild nodes every iteration)");
    let mut facts2 = HashMap::new();
    facts2.insert("age".to_string(), "25".to_string());
    facts2.insert("status".to_string(), "active".to_string());
    facts2.insert("score".to_string(), "88.5".to_string());
    println!("Initial facts: age=25, status=active, score=88.5");

    let start2 = Instant::now();
    let fired2 = rete_ul_fire_all_rules_slow(&simple_rules, &mut facts2);
    let duration2 = start2.elapsed();
    println!("Fired rules: {:?}", fired2);
    println!("Time: {:?}\n", duration2);

    // Performance comparison
    println!("📊 Performance Comparison:");
    println!("   Fast version: {:?}", duration1);
    println!("   Slow version: {:?}", duration2);
    if duration2.as_nanos() > 0 {
        let speedup = duration2.as_nanos() as f64 / duration1.as_nanos() as f64;
        println!("   Speedup: {:.2}x faster with caching!", speedup);
    }
    println!("\n✅ Use ReteUlEngine with cached nodes for better performance!");
}

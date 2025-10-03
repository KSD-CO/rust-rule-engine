/// Demo: Parallel Rule Execution
/// This example shows parallel rule execution capabilities.

use rust_rule_engine::engine::{
    parallel::{ParallelRuleEngine, ParallelConfig},
    rule::{Rule, Condition, ConditionGroup},
    facts::Facts,
    knowledge_base::KnowledgeBase,
};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::time::Instant;

fn main() {
    println!("🛡️  PARALLEL RULE EXECUTION DEMO");
    println!("=================================\n");

    // Demo: Simple parallel execution
    demo_parallel_execution();
}

fn demo_parallel_execution() {
    println!("📊 Parallel Rule Execution Test\n");
    
    // Create knowledge base with test rules
    let mut kb = KnowledgeBase::new("ParallelDemo");
    
    // Add some independent rules
    let rules = vec![
        Rule::new(
            "CheckAge".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Age".to_string(),
                Operator::GreaterThanOrEqual,
                Value::Integer(18),
            )),
            vec![ActionType::Log { 
                message: "User is adult".to_string() 
            }],
        ),
        Rule::new(
            "CheckCountry".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Country".to_string(),
                Operator::Equal,
                Value::String("US".to_string()),
            )),
            vec![ActionType::Log { 
                message: "User is from US".to_string() 
            }],
        ),
        Rule::new(
            "CheckEmail".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Email".to_string(),
                Operator::Contains,
                Value::String("@test.com".to_string()),
            )),
            vec![ActionType::Log { 
                message: "User has test email".to_string() 
            }],
        ),
    ];

    for rule in rules {
        kb.add_rule(rule).unwrap();
    }
    
    // Create facts
    let facts = Facts::new();
    facts.set("User.Age", Value::Integer(25));
    facts.set("User.Country", Value::String("US".to_string()));
    facts.set("User.Email", Value::String("user@test.com".to_string()));
    
    // Configure parallel engine
    let config = ParallelConfig {
        enabled: true,
        max_threads: 4,
        min_rules_per_thread: 1,
        dependency_analysis: true,
    };
    
    let engine = ParallelRuleEngine::new(config);
    
    // Execute rules
    let start = Instant::now();
    match engine.execute_parallel(&kb, &facts, true) {
        Ok(result) => {
            let duration = start.elapsed();
            println!("✅ Parallel execution completed!");
            println!("   Rules evaluated: {}", result.total_rules_evaluated);
            println!("   Rules fired: {}", result.total_rules_fired);
            println!("   Execution time: {:?}", duration);
            println!("   Parallel speedup: {:.2}x", result.parallel_speedup);
        }
        Err(e) => {
            println!("❌ Execution failed: {}", e);
        }
    }
}
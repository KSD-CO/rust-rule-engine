/// Demo: Memoization for RETE-UL Evaluation
///
/// This example demonstrates how memoization can dramatically improve
/// performance by caching evaluation results and avoiding redundant computations.
use rust_rule_engine::rete::auto_network::{Condition, ConditionGroup, Rule};
use rust_rule_engine::rete::facts::TypedFacts;
use rust_rule_engine::rete::memoization::MemoizedEvaluator;
use rust_rule_engine::rete::network::{build_rete_ul_from_condition_group, ReteUlNode};
use std::time::Instant;

fn main() {
    println!("\n🧠 RETE-UL Memoization Demo");
    println!("==========================\n");

    // Create complex rules
    let rules = [
        Rule {
            name: "ComplexRule1".to_string(),
            conditions: ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition {
                    field: "age".to_string(),
                    operator: ">=".to_string(),
                    value: "18".to_string(),
                })),
                operator: "AND".to_string(),
                right: Box::new(ConditionGroup::Compound {
                    left: Box::new(ConditionGroup::Single(Condition {
                        field: "score".to_string(),
                        operator: ">".to_string(),
                        value: "80".to_string(),
                    })),
                    operator: "AND".to_string(),
                    right: Box::new(ConditionGroup::Single(Condition {
                        field: "active".to_string(),
                        operator: "==".to_string(),
                        value: "true".to_string(),
                    })),
                }),
            },
            action: "complex_action_1".to_string(),
        },
        Rule {
            name: "ComplexRule2".to_string(),
            conditions: ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition {
                    field: "name".to_string(),
                    operator: "contains".to_string(),
                    value: "John".to_string(),
                })),
                operator: "OR".to_string(),
                right: Box::new(ConditionGroup::Single(Condition {
                    field: "email".to_string(),
                    operator: "endsWith".to_string(),
                    value: "@example.com".to_string(),
                })),
            },
            action: "complex_action_2".to_string(),
        },
    ];

    // Build nodes once
    let nodes: Vec<ReteUlNode> = rules
        .iter()
        .map(|r| build_rete_ul_from_condition_group(&r.conditions))
        .collect();

    // Create facts
    let mut facts = TypedFacts::new();
    facts.set("age", 25i64);
    facts.set("score", 95.5);
    facts.set("active", true);
    facts.set("name", "John Smith");
    facts.set("email", "john@example.com");

    println!("📊 Scenario: Evaluating {} complex rules", rules.len());
    println!(
        "Facts: age=25, score=95.5, active=true, name=\"John Smith\", email=\"john@example.com\"\n"
    );

    // Test 1: Without memoization
    println!("🐌 Test 1: WITHOUT Memoization");
    println!("------------------------------");
    let iterations = 10000;

    let start = Instant::now();
    let mut without_memo_results = Vec::new();
    for _ in 0..iterations {
        for node in &nodes {
            without_memo_results.push(node.evaluate_typed(&facts));
        }
    }
    let duration_without = start.elapsed();

    println!("Iterations: {}", iterations);
    println!("Total evaluations: {}", iterations * nodes.len());
    println!("Time: {:?}", duration_without);
    let total_evals = (iterations as u128) * (nodes.len() as u128);
    println!(
        "Avg per evaluation: {:?}ns",
        duration_without.as_nanos() / total_evals
    );

    // Test 2: With memoization
    println!("\n🚀 Test 2: WITH Memoization");
    println!("---------------------------");

    let mut evaluator = MemoizedEvaluator::new();
    let start = Instant::now();
    let mut with_memo_results = Vec::new();
    for _ in 0..iterations {
        for node in &nodes {
            let result = evaluator.evaluate(node, &facts, |n, f| n.evaluate_typed(f));
            with_memo_results.push(result);
        }
    }
    let duration_with = start.elapsed();

    println!("Iterations: {}", iterations);
    println!("Total evaluations: {}", iterations * nodes.len());
    println!("Time: {:?}", duration_with);
    println!(
        "Avg per evaluation: {:?}ns",
        duration_with.as_nanos() / total_evals
    );

    // Show cache statistics
    let stats = evaluator.stats();
    println!("\n📈 Cache Statistics:");
    println!("   {}", stats);
    println!("   Cache entries: {}", stats.cache_size);
    println!(
        "   Cache hits: {} ({:.2}%)",
        stats.hits,
        stats.hit_rate * 100.0
    );
    println!("   Cache misses: {}", stats.misses);

    // Performance comparison
    println!("\n⚡ Performance Comparison:");
    println!("-------------------------");
    let speedup = duration_without.as_nanos() as f64 / duration_with.as_nanos() as f64;
    println!("Without memoization: {:?}", duration_without);
    println!("With memoization: {:?}", duration_with);

    if speedup > 1.0 {
        println!("Speedup: {:.2}x faster!", speedup);
        println!("Time saved: {:?}", duration_without - duration_with);
    } else {
        println!("Overhead: {:.2}x slower", 1.0 / speedup);
        println!("Time overhead: {:?}", duration_with - duration_without);
        println!("\n⚠️  Note: For simple rules with few iterations,");
        println!("   memoization overhead (hashing + HashMap lookup) can exceed benefits.");
        println!("   Memoization is most effective with:");
        println!("   - Complex nested conditions");
        println!("   - Many repeated evaluations (100k+)");
        println!("   - Expensive custom operators");
    }

    // Verify correctness
    assert_eq!(
        without_memo_results, with_memo_results,
        "Results should be identical!"
    );
    println!("\n✅ Correctness verified: Both methods produce identical results");

    // Test 3: Memoization with changing facts
    println!("\n🔄 Test 3: Memoization with Changing Facts");
    println!("------------------------------------------");

    let mut evaluator2 = MemoizedEvaluator::new();
    let test_ages = vec![15, 20, 25, 30, 15, 20, 25, 30]; // Repeating values

    println!("Testing with repeating age values: {:?}", test_ages);

    for age in &test_ages {
        facts.set("age", *age as i64);
        for node in &nodes {
            evaluator2.evaluate(node, &facts, |n, f| n.evaluate_typed(f));
        }
    }

    let stats2 = evaluator2.stats();
    println!("\nCache statistics:");
    println!("   {}", stats2);
    println!(
        "   Hit rate: {:.2}% (cache reused for repeated age values)",
        stats2.hit_rate * 100.0
    );

    // Summary
    println!("\n✨ Memoization Summary");
    println!("=====================");
    println!(
        "✅ Cache hit rate: {:.2}% (excellent!)",
        stats.hit_rate * 100.0
    );
    println!(
        "✅ Memory efficient: Only {} cache entries for {} evaluations",
        stats.cache_size,
        iterations * nodes.len()
    );
    println!("✅ Transparent: Same API, automatic caching");
    println!("✅ Correctness: Results are identical");
    println!("\n💡 Memoization works best when:");
    println!("   - Same facts evaluated repeatedly (high cache hits)");
    println!("   - Complex nested conditions (expensive to evaluate)");
    println!("   - Large rule networks with many rules");
    println!("   - Facts change infrequently between evaluations");
    println!("\n⚠️  May have overhead when:");
    println!("   - Simple conditions (fast to evaluate anyway)");
    println!("   - Facts change frequently (low cache hits)");
    println!("   - Few iterations (amortization not reached)");
}

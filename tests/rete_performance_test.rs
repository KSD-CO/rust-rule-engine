use rust_rule_engine::rete::{
    auto_network::{Condition, ConditionGroup, Rule},
    ReteUlEngine,
};
use std::time::{Duration, Instant};

#[test]
fn test_rete_basic_execution() {
    let mut engine = ReteUlEngine::new();

    // Add simple rule
    let rule = Rule {
        name: "TestRule".to_string(),
        conditions: ConditionGroup::Single(Condition {
            field: "age".to_string(),
            operator: ">".to_string(),
            value: "25".to_string(),
        }),
        action: "log('matched')".to_string(),
    };

    engine.add_rule_from_definition(&rule, 100, false);
    engine.set_fact("age".to_string(), "30".to_string());

    // Test with timeout
    let start = Instant::now();
    let result = engine.fire_all();
    let elapsed = start.elapsed();

    println!("RETE execution took: {:?}", elapsed);
    println!("Fired rules: {:?}", result);

    // Should complete in reasonable time (< 1 second for 1 rule)
    assert!(
        elapsed < Duration::from_secs(1),
        "RETE took too long: {:?}",
        elapsed
    );
    assert!(!result.is_empty(), "No rules fired");
}

#[test]
fn test_rete_multiple_rules() {
    let mut engine = ReteUlEngine::new();

    // Add 3 rules
    for i in 0..3 {
        let rule = Rule {
            name: format!("Rule{}", i),
            conditions: ConditionGroup::Single(Condition {
                field: "count".to_string(),
                operator: ">".to_string(),
                value: format!("{}", i * 10),
            }),
            action: format!("log('Rule{} fired')", i),
        };
        engine.add_rule_from_definition(&rule, 100 - i, false);
    }

    engine.set_fact("count".to_string(), "50".to_string());

    // Test with timeout
    let start = Instant::now();
    let result = engine.fire_all();
    let elapsed = start.elapsed();

    println!("RETE with 3 rules took: {:?}", elapsed);
    println!("Fired rules: {:?}", result);

    // Should complete in reasonable time
    assert!(
        elapsed < Duration::from_secs(2),
        "RETE took too long: {:?}",
        elapsed
    );
    assert_eq!(result.len(), 3, "Expected 3 rules to fire");
}

#[test]
fn test_rete_and_condition() {
    let mut engine = ReteUlEngine::new();

    let rule = Rule {
        name: "AndRule".to_string(),
        conditions: ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition {
                field: "age".to_string(),
                operator: ">".to_string(),
                value: "25".to_string(),
            })),
            operator: "AND".to_string(),
            right: Box::new(ConditionGroup::Single(Condition {
                field: "country".to_string(),
                operator: "==".to_string(),
                value: "US".to_string(),
            })),
        },
        action: "log('matched')".to_string(),
    };

    engine.add_rule_from_definition(&rule, 100, false);
    engine.set_fact("age".to_string(), "30".to_string());
    engine.set_fact("country".to_string(), "US".to_string());

    let start = Instant::now();
    let result = engine.fire_all();
    let elapsed = start.elapsed();

    println!("RETE AND condition took: {:?}", elapsed);
    println!("Fired rules: {:?}", result);

    assert!(
        elapsed < Duration::from_secs(1),
        "RETE took too long: {:?}",
        elapsed
    );
    assert_eq!(result.len(), 1, "Expected 1 rule to fire");
}

#[test]
fn test_rete_no_loop() {
    let mut engine = ReteUlEngine::new();

    let rule = Rule {
        name: "NoLoopRule".to_string(),
        conditions: ConditionGroup::Single(Condition {
            field: "trigger".to_string(),
            operator: "==".to_string(),
            value: "true".to_string(),
        }),
        action: "log('fired')".to_string(),
    };

    // Add with no_loop = true
    engine.add_rule_from_definition(&rule, 100, true);
    engine.set_fact("trigger".to_string(), "true".to_string());

    let start = Instant::now();
    let result = engine.fire_all();
    let elapsed = start.elapsed();

    println!("RETE no_loop took: {:?}", elapsed);
    println!("Fired rules: {:?}", result);

    // With no_loop, should fire only once and not hang
    assert!(
        elapsed < Duration::from_secs(1),
        "RETE took too long: {:?}",
        elapsed
    );
    assert_eq!(result.len(), 1, "Expected 1 rule to fire exactly once");
}

#[test]
fn test_rete_performance_scaling() {
    for rule_count in [1, 5, 10, 20] {
        let mut engine = ReteUlEngine::new();

        // Add rules
        for i in 0..rule_count {
            let rule = Rule {
                name: format!("Rule{}", i),
                conditions: ConditionGroup::Single(Condition {
                    field: "value".to_string(),
                    operator: ">".to_string(),
                    value: format!("{}", i * 5),
                }),
                action: format!("log('Rule{}')", i),
            };
            engine.add_rule_from_definition(&rule, (100 - i) as i32, false);
        }

        engine.set_fact("value".to_string(), "100".to_string());

        let start = Instant::now();
        let result = engine.fire_all();
        let elapsed = start.elapsed();

        println!("\nRete with {} rules:", rule_count);
        println!("  Time: {:?}", elapsed);
        println!("  Fired: {} rules", result.len());
        println!("  Per rule: {:?}", elapsed / rule_count);

        // Should scale reasonably - max 100ms per rule
        let max_time = Duration::from_millis(100 * rule_count as u64);
        assert!(
            elapsed < max_time,
            "RETE with {} rules took {:?}, expected < {:?}",
            rule_count,
            elapsed,
            max_time
        );
    }
}

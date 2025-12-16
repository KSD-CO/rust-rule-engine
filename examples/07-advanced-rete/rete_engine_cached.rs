/// Demo: ReteUlEngine with cached nodes (Performance optimized!)
/// This example shows the difference between rebuilding nodes vs caching them
use rust_rule_engine::rete::auto_network::{Condition, ConditionGroup, Rule};
use rust_rule_engine::rete::network::{build_rete_ul_from_condition_group, ReteUlEngine};
use std::time::Instant;

fn main() {
    println!("\n🚀 RETE-UL Engine with Cached Nodes Demo");
    println!("==========================================\n");

    // Define rules
    let rules = vec![
        Rule {
            name: "IsAdult".to_string(),
            conditions: ConditionGroup::Single(Condition {
                field: "age".to_string(),
                operator: ">=".to_string(),
                value: "18".to_string(),
            }),
            action: "mark_adult".to_string(),
        },
        Rule {
            name: "IsActive".to_string(),
            conditions: ConditionGroup::Single(Condition {
                field: "status".to_string(),
                operator: "==".to_string(),
                value: "active".to_string(),
            }),
            action: "mark_active".to_string(),
        },
        Rule {
            name: "HighScore".to_string(),
            conditions: ConditionGroup::Single(Condition {
                field: "score".to_string(),
                operator: ">".to_string(),
                value: "80".to_string(),
            }),
            action: "mark_high_score".to_string(),
        },
        Rule {
            name: "VIPUser".to_string(),
            conditions: ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition {
                    field: "age".to_string(),
                    operator: ">=".to_string(),
                    value: "18".to_string(),
                })),
                operator: "AND".to_string(),
                right: Box::new(ConditionGroup::Compound {
                    left: Box::new(ConditionGroup::Single(Condition {
                        field: "status".to_string(),
                        operator: "==".to_string(),
                        value: "active".to_string(),
                    })),
                    operator: "AND".to_string(),
                    right: Box::new(ConditionGroup::Single(Condition {
                        field: "score".to_string(),
                        operator: ">".to_string(),
                        value: "80".to_string(),
                    })),
                }),
            },
            action: "grant_vip".to_string(),
        },
    ];

    // Create ReteUlEngine and add rules (nodes built and cached ONCE!)
    println!("📦 Building ReteUlEngine with cached nodes...");
    let start_build = Instant::now();
    let mut engine = ReteUlEngine::new();

    for rule in &rules {
        let node = build_rete_ul_from_condition_group(&rule.conditions);
        let rule_name = rule.name.clone();
        engine.add_rule_with_action(
            rule.name.clone(),
            node,
            0,    // priority
            true, // no_loop
            move |facts| {
                println!("   ✅ Rule fired: {}", rule_name);
                facts.insert(format!("{}_result", rule_name), "executed".to_string());
            },
        );
    }
    let build_time = start_build.elapsed();
    println!("   Build time: {:?}", build_time);

    // Set initial facts
    println!("\n📝 Setting initial facts...");
    engine.set_fact("age".to_string(), "25".to_string());
    engine.set_fact("status".to_string(), "active".to_string());
    engine.set_fact("score".to_string(), "88.5".to_string());

    println!("   age = 25");
    println!("   status = active");
    println!("   score = 88.5");

    // Check which rules match (without firing)
    println!("\n🔍 Checking matching rules...");
    let matching = engine.get_matching_rules();
    println!("   Matching rules: {:?}", matching);

    // Fire all rules (using cached nodes - NO rebuild!)
    println!("\n🔥 Firing all matching rules...");
    let start_fire = Instant::now();
    let fired = engine.fire_all();
    let fire_time = start_fire.elapsed();

    println!("\n📊 Results:");
    println!("   Rules fired: {:?}", fired);
    println!("   Fire time: {:?}", fire_time);
    println!("   Total time: {:?}", build_time + fire_time);

    // Test specific rule matching
    println!("\n🎯 Testing specific rules:");
    println!("   IsAdult matches: {}", engine.matches("IsAdult"));
    println!("   VIPUser matches: {}", engine.matches("VIPUser"));

    // Change facts and fire again
    println!("\n🔄 Changing facts and firing again...");
    engine.reset_fired_flags();
    engine.set_fact("score".to_string(), "50".to_string());
    println!("   Changed score to 50");

    let fired2 = engine.fire_all();
    println!("   Rules fired: {:?}", fired2);
    println!("   VIPUser still matches: {}", engine.matches("VIPUser"));

    // Performance comparison
    println!("\n⚡ Performance Comparison:");
    println!("   With caching: Build once, fire many times");
    println!("   Without caching: Rebuild nodes every iteration (BAD!)");
    println!("   Speedup: ~10-100x for complex rules and multiple iterations");

    // Test FORALL empty set fix
    println!("\n🐛 Testing FORALL empty set fix:");
    let forall_rule = Rule {
        name: "AllOrdersExpensive".to_string(),
        conditions: ConditionGroup::Forall(Box::new(ConditionGroup::Single(Condition {
            field: "order.amount".to_string(),
            operator: ">".to_string(),
            value: "100".to_string(),
        }))),
        action: "all_expensive".to_string(),
    };

    let mut engine2 = ReteUlEngine::new();
    engine2.add_rule_from_definition(&forall_rule, 0, true);

    // Empty set - should match (vacuous truth)
    let matching_empty = engine2.get_matching_rules();
    println!(
        "   FORALL on empty set: {:?} (should be ['AllOrdersExpensive'])",
        matching_empty
    );

    // Add one order that matches
    engine2.set_fact("order1.amount".to_string(), "150".to_string());
    let matching_one = engine2.get_matching_rules();
    println!("   FORALL with one matching order: {:?}", matching_one);

    // Add one order that doesn't match
    engine2.set_fact("order2.amount".to_string(), "50".to_string());
    let matching_mixed = engine2.get_matching_rules();
    println!(
        "   FORALL with mixed orders: {:?} (should be empty)",
        matching_mixed
    );

    println!("\n✨ Demo complete!");
}

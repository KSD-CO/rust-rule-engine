use rust_rule_engine::{
    engine::{
        rule::{Condition, ConditionGroup, Rule},
        EngineConfig, RustRuleEngine,
    },
    errors::Result,
    types::{ActionType, Operator, Value},
    Facts, KnowledgeBase,
};
use std::collections::HashMap;

fn main() -> Result<()> {
    println!("🔄 No-Loop Feature Demo");
    println!("=======================");

    // Demo 1: Without no-loop (infinite loop protection via max_cycles)
    demo_without_no_loop()?;

    println!();

    // Demo 2: With no-loop (rule fires only once per cycle)
    demo_with_no_loop()?;

    Ok(())
}

fn demo_without_no_loop() -> Result<()> {
    println!("📋 Demo 1: Without no-loop");
    println!("---------------------------");

    let kb = KnowledgeBase::new("WithoutNoLoop");

    // Rule that modifies its own condition - would cause infinite loop without max_cycles
    let rule = Rule::new(
        "ScoreBooster".to_string(),
        ConditionGroup::single(Condition::new(
            "Player.score".to_string(),
            Operator::LessThan,
            Value::Integer(100),
        )),
        vec![
            ActionType::Set {
                field: "Player.score".to_string(),
                value: Value::String("Player.score + 10".to_string()),
            },
            ActionType::Log {
                message: "Boosting score by 10".to_string(),
            },
        ],
    )
    .with_no_loop(false); // Explicitly disable no-loop

    let _ = kb.add_rule(rule);

    // Configure engine with limited cycles to prevent infinite loop
    let config = EngineConfig {
        max_cycles: 5, // Limit to 5 cycles
        debug_mode: true,
        ..Default::default()
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register function to simulate score increment
    engine.register_function("increment_score", |_args: &[Value], facts: &Facts| {
        if let Some(Value::Object(player)) = facts.get("Player") {
            if let Some(Value::Integer(current_score)) = player.get("score") {
                let new_score = current_score + 10;
                Ok(Value::Integer(new_score))
            } else {
                Ok(Value::Integer(10))
            }
        } else {
            Ok(Value::Integer(10))
        }
    });

    let facts = Facts::new();
    facts.set(
        "Player",
        Value::Object({
            let mut player = HashMap::new();
            player.insert("name".to_string(), Value::String("John".to_string()));
            player.insert("score".to_string(), Value::Integer(50));
            player
        }),
    );

    println!("🏁 Initial state: Player.score = 50");

    let result = engine.execute(&facts)?;

    println!(
        "📊 Result: {} rules fired in {} cycles",
        result.rules_fired, result.cycle_count
    );
    println!("   Max cycles prevented infinite loop!");

    if let Some(Value::Object(player)) = facts.get("Player") {
        if let Some(Value::Integer(final_score)) = player.get("score") {
            println!("🏁 Final score: {}", final_score);
        }
    }

    Ok(())
}

fn demo_with_no_loop() -> Result<()> {
    println!("📋 Demo 2: With no-loop");
    println!("------------------------");

    let kb = KnowledgeBase::new("WithNoLoop");

    // Same rule but with no-loop enabled
    let rule = Rule::new(
        "ScoreBooster".to_string(),
        ConditionGroup::single(Condition::new(
            "Player.score".to_string(),
            Operator::LessThan,
            Value::Integer(100),
        )),
        vec![
            ActionType::Set {
                field: "Player.score".to_string(),
                value: Value::String("Player.score + 10".to_string()),
            },
            ActionType::Log {
                message: "Boosting score by 10 (no-loop)".to_string(),
            },
        ],
    )
    .with_no_loop(true); // Enable no-loop

    let _ = kb.add_rule(rule);

    let config = EngineConfig {
        max_cycles: 10, // Higher limit since no-loop prevents infinite firing
        debug_mode: true,
        ..Default::default()
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    let facts = Facts::new();
    facts.set(
        "Player",
        Value::Object({
            let mut player = HashMap::new();
            player.insert("name".to_string(), Value::String("Jane".to_string()));
            player.insert("score".to_string(), Value::Integer(50));
            player
        }),
    );

    println!("🏁 Initial state: Player.score = 50");

    let result = engine.execute(&facts)?;

    println!(
        "📊 Result: {} rules fired in {} cycles",
        result.rules_fired, result.cycle_count
    );
    println!("   No-loop prevented the rule from firing multiple times in same cycle!");

    if let Some(Value::Object(player)) = facts.get("Player") {
        if let Some(Value::Integer(final_score)) = player.get("score") {
            println!("🏁 Final score: {}", final_score);
        }
    }

    Ok(())
}

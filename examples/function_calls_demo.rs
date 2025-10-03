use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Demo: Generic Function Calls ===\n");

    // Create test data
    let mut user_props = HashMap::new();
    user_props.insert("Name".to_string(), Value::String("John Doe".to_string()));
    user_props.insert("Score".to_string(), Value::Number(85.5));
    user_props.insert("Active".to_string(), Value::Boolean(true));

    let numbers = vec![
        Value::Number(10.0),
        Value::Number(20.0),
        Value::Number(30.0),
        Value::Number(15.5),
        Value::Number(25.2),
    ];

    // Create facts
    let facts = Facts::new();
    facts.add_value("User", Value::Object(user_props))?;
    facts.add_value("Numbers", Value::Array(numbers))?;
    facts.add_value("Message", Value::String("Hello World".to_string()))?;

    println!("🏁 Initial state:");
    if let Some(user) = facts.get("User") {
        println!("   User = {user:?}");
    }
    if let Some(numbers) = facts.get("Numbers") {
        println!("   Numbers = {numbers:?}");
    }
    if let Some(message) = facts.get("Message") {
        println!("   Message = {message:?}");
    }
    println!();

    // Create knowledge base
    let kb = KnowledgeBase::new("FunctionCallDemo");

    // Rule 1: Log message if user is active
    let log_rule = Rule::new(
        "LogUserInfo".to_string(),
        ConditionGroup::single(Condition::new(
            "User.Active".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Call {
            function: "log".to_string(),
            args: vec![
                Value::String("User is active!".to_string()),
                Value::String("Processing user data...".to_string()),
            ],
        }],
    )
    .with_salience(10);

    // Rule 2: Calculate statistics if score > 80
    let stats_rule = Rule::new(
        "CalculateStats".to_string(),
        ConditionGroup::single(Condition::new(
            "User.Score".to_string(),
            Operator::GreaterThan,
            Value::Number(80.0),
        )),
        vec![
            ActionType::Call {
                function: "sum".to_string(),
                args: vec![
                    Value::Number(10.0),
                    Value::Number(20.0),
                    Value::Number(30.0),
                ],
            },
            ActionType::Call {
                function: "max".to_string(),
                args: vec![
                    Value::Number(85.5),
                    Value::Number(90.0),
                    Value::Number(78.2),
                ],
            },
            ActionType::Call {
                function: "average".to_string(),
                args: vec![
                    Value::Number(85.5),
                    Value::Number(90.0),
                    Value::Number(78.2),
                ],
            },
        ],
    )
    .with_salience(8);

    // Rule 3: String manipulation
    let string_rule = Rule::new(
        "StringManipulation".to_string(),
        ConditionGroup::single(Condition::new(
            "Message".to_string(),
            Operator::Contains,
            Value::String("Hello".to_string()),
        )),
        vec![
            ActionType::Call {
                function: "uppercase".to_string(),
                args: vec![Value::String("hello world".to_string())],
            },
            ActionType::Call {
                function: "contains".to_string(),
                args: vec![
                    Value::String("Hello World".to_string()),
                    Value::String("World".to_string()),
                ],
            },
            ActionType::Call {
                function: "format".to_string(),
                args: vec![
                    Value::String("User {0} has score {1}".to_string()),
                    Value::String("John Doe".to_string()),
                    Value::Number(85.5),
                ],
            },
        ],
    )
    .with_salience(6);

    // Rule 4: Mathematical functions
    let math_rule = Rule::new(
        "MathFunctions".to_string(),
        ConditionGroup::single(Condition::new(
            "User.Score".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Number(80.0),
        )),
        vec![
            ActionType::Call {
                function: "round".to_string(),
                args: vec![Value::Number(85.7)],
            },
            ActionType::Call {
                function: "floor".to_string(),
                args: vec![Value::Number(85.7)],
            },
            ActionType::Call {
                function: "ceil".to_string(),
                args: vec![Value::Number(85.2)],
            },
            ActionType::Call {
                function: "abs".to_string(),
                args: vec![Value::Number(-42.5)],
            },
        ],
    )
    .with_salience(4);

    // Rule 5: Utility functions
    let utility_rule = Rule::new(
        "UtilityFunctions".to_string(),
        ConditionGroup::single(Condition::new(
            "User.Name".to_string(),
            Operator::NotEqual,
            Value::String("".to_string()),
        )),
        vec![
            ActionType::Call {
                function: "timestamp".to_string(),
                args: vec![],
            },
            ActionType::Call {
                function: "random".to_string(),
                args: vec![Value::Number(100.0)],
            },
            ActionType::Call {
                function: "length".to_string(),
                args: vec![Value::String("Hello World".to_string())],
            },
            ActionType::Call {
                function: "split".to_string(),
                args: vec![
                    Value::String("apple,banana,cherry".to_string()),
                    Value::String(",".to_string()),
                ],
            },
        ],
    )
    .with_salience(2);

    // Add rules to knowledge base
    let _ = kb.add_rule(log_rule);
    let _ = kb.add_rule(stats_rule);
    let _ = kb.add_rule(string_rule);
    let _ = kb.add_rule(math_rule);
    let _ = kb.add_rule(utility_rule);

    // Create engine with debug mode
    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 5,
        ..Default::default()
    };
    let engine = RustRuleEngine::with_config(kb, config);

    // Execute rules
    println!("🚀 Executing rules with generic function calls...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Execution Results:");
    println!("   Cycles: {}", result.cycle_count);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    println!("\n🎯 Function call examples demonstrated:");
    println!("   📋 Logging: log(), print()");
    println!("   🔢 Math: sum(), max(), average(), round(), floor(), ceil(), abs()");
    println!("   📝 String: uppercase(), contains(), format(), length(), split()");
    println!("   🛠️ Utility: timestamp(), random(), update()");
    println!("   🎨 Custom: Any user-defined function names");

    Ok(())
}

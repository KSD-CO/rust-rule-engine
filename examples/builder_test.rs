use rust_rule_engine::{Facts, RuleEngineBuilder, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Testing RuleEngineBuilder with inline rules");

    let grl_rules = r#"
        rule "SimpleTest" salience 10 {
            when
                User.Age >= 18
            then
                log("User is adult");
        }
    "#;

    // Test inline rules
    let engine = RuleEngineBuilder::new().with_inline_grl(grl_rules)?.build();

    // Create facts
    let facts = Facts::new();
    let mut user = HashMap::new();
    user.insert("Age".to_string(), Value::Integer(25));
    facts.add_value("User", Value::Object(user))?;

    // Execute
    let result = engine.execute(&facts)?;
    println!("✅ Rules fired: {}", result.rules_fired);

    Ok(())
}

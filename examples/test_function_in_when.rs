/// Demo: Why function calls don't work in WHEN clause
///
/// This demonstrates the parser limitation

use rust_rule_engine::{
    engine::knowledge_base::KnowledgeBase,
    engine::{EngineConfig, RustRuleEngine},
    types::Value,
    GRLParser,
};

fn main() {
    println!("=== Testing Function Calls in WHEN vs THEN ===\n");

    // Test 1: Function in THEN (should work)
    println!("✅ Test 1: Function call in THEN clause");
    println!("─────────────────────────────────────────");
    test_function_in_then();

    println!("\n❌ Test 2: Function call in WHEN clause");
    println!("─────────────────────────────────────────");
    test_function_in_when();
}

fn test_function_in_then() {
    let rule = r#"
rule "FunctionInThen" {
    when
        User.needsCheck == true
    then
        set(User.sentiment, aiSentiment(User.text));
        Log("AI function called successfully");
}
    "#;

    match GRLParser::parse_rules(rule) {
        Ok(rules) => {
            println!("✅ Parser SUCCESS!");
            println!("   Rule name: {}", rules[0].name);
            println!("   Conditions: {:?}", rules[0].conditions);
            println!("   Actions: {} actions", rules[0].actions.len());
            println!("\n   💡 Function 'aiSentiment' can be called in THEN clause");
        }
        Err(e) => {
            println!("❌ Parser FAILED: {:?}", e);
        }
    }
}

fn test_function_in_when() {
    let rule = r#"
rule "FunctionInWhen" {
    when
        aiSentiment(User.text) == "negative"
    then
        set(User.flagged, true);
}
    "#;

    match GRLParser::parse_rules(rule) {
        Ok(rules) => {
            println!("✅ Parser SUCCESS (unexpected!)");
            println!("   Rule name: {}", rules[0].name);
        }
        Err(e) => {
            println!("❌ Parser FAILED (expected):");
            println!("   Error: {:?}", e);
            println!("\n   💡 Parser regex only matches:");
            println!("      field operator value");
            println!("      Example: User.age >= 18");
            println!("\n   ❌ Parser does NOT support:");
            println!("      function(args) operator value");
            println!("      Example: aiSentiment(text) == \"negative\"");
        }
    }
}

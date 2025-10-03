use rust_rule_engine::*;
use std::collections::HashMap;

#[test]
fn test_basic_grule_rules() {
    // Test very simple GRL parsing
    let grl_content = r#"rule "TestRule" {
when
User.Age > 18
then
User.setStatus("adult");
}"#;

    let rules = GRLParser::parse_rules(grl_content).unwrap();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name, "TestRule");
}

#[test]
fn test_multiple_rules_parsing() {
    // Test two simple rules
    let grl_content = r#"rule "Rule1" {
when
User.Age > 18
then
User.setStatus("adult");
}
rule "Rule2" {
when
User.Score >= 90
then
User.setGrade("A");
}"#;

    let rules = GRLParser::parse_rules(grl_content).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn test_facts_manipulation() {
    // Test basic facts handling
    let mut facts = HashMap::new();
    facts.insert("user_age".to_string(), Value::Integer(25));
    facts.insert(
        "user_status".to_string(),
        Value::String("pending".to_string()),
    );

    assert_eq!(facts.get("user_age"), Some(&Value::Integer(25)));
    assert_eq!(
        facts.get("user_status"),
        Some(&Value::String("pending".to_string()))
    );
}

#[test]
fn test_knowledge_base_management() {
    let kb = KnowledgeBase::new("TestKB");
    assert_eq!(kb.name(), "TestKB");

    // Test simple GRL rule parsing
    let simple_rule = r#"rule "SimpleRule" {
when
Car.Speed > 50
then
Car.setAlert("speeding");
}"#;

    let rules = GRLParser::parse_rules(simple_rule).unwrap();
    assert_eq!(rules.len(), 1);
}

#[test]
fn test_engine_configuration() {
    let kb = KnowledgeBase::new("ConfigTests");
    let _engine = RustRuleEngine::new(kb);

    // Test engine can be created
    assert!(true);
}

use rust_rule_engine::backward::backward_engine::{BackwardConfig, BackwardEngine};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, Rule};
use rust_rule_engine::types::{ActionType, Operator, Value};
use rust_rule_engine::ConditionGroup;

fn main() {
    let kb = KnowledgeBase::new("solution_limit");

    // Simple rule
    kb.add_rule(Rule::new(
        "SetValue".to_string(),
        ConditionGroup::single(Condition::new(
            "Input.Ready".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        )),
        vec![ActionType::Set {
            field: "Output.Value".to_string(),
            value: Value::Number(42.0),
        }],
    ))
    .unwrap();

    // Test with max_solutions = 1
    let config1 = BackwardConfig {
        max_solutions: 1,
        ..Default::default()
    };

    let mut engine1 = BackwardEngine::with_config(kb.clone(), config1);
    let mut facts1 = Facts::new();
    facts1.set("Input.Ready", Value::Boolean(true));

    let result1 = engine1.query("Output.Value == 42", &mut facts1).unwrap();
    
    println!("Provable: {}", result1.provable);
    println!("Proof trace: {:?}", result1.proof_trace);
    
    if !result1.provable {
        panic!("Test failed: result should be provable!");
    } else {
        println!("✅ Test passed!");
    }
}

use rust_rule_engine::rete::ReteUlEngine;

#[test]
fn test_rete_ul_engine_basic() {
    let mut engine = ReteUlEngine::new();
    engine.set_fact("age".to_string(), "20".to_string());
    engine.set_fact("status".to_string(), "active".to_string());
    engine.set_fact("age2".to_string(), "15".to_string());
    engine.set_fact("status2".to_string(), "inactive".to_string());

    // No rules added, just check facts API
    assert_eq!(engine.get_fact("age"), Some(&"20".to_string()));
    assert_eq!(engine.get_fact("status"), Some(&"active".to_string()));
    assert_eq!(engine.get_fact("age2"), Some(&"15".to_string()));
    assert_eq!(engine.get_fact("status2"), Some(&"inactive".to_string()));
}

use rust_rule_engine::rete::{AlphaNode, AlphaMemory, BetaNode, ReteNetwork};

#[test]
fn test_rete_network_basic() {
    let alpha1 = AlphaNode { field: "age".to_string(), operator: "==".to_string(), value: "20".to_string() };
    let alpha2 = AlphaNode { field: "status".to_string(), operator: "==".to_string(), value: "active".to_string() };
    let alpha_mem1 = AlphaMemory { matches: vec![] };
    let alpha_mem2 = AlphaMemory { matches: vec![] };
    let beta = BetaNode { left: alpha_mem1.clone(), right: alpha_mem2.clone() };

    let mut rete = ReteNetwork {
        alpha_nodes: vec![alpha1, alpha2],
        alpha_memories: vec![alpha_mem1, alpha_mem2],
        beta_nodes: vec![beta],
    };

    let facts = vec![
        ("age".to_string(), "20".to_string()),
        ("status".to_string(), "active".to_string()),
        ("age".to_string(), "15".to_string()),
        ("status".to_string(), "inactive".to_string()),
    ];

    rete.propagate(facts);
    // Check that alpha memories are populated
    assert_eq!(rete.alpha_memories[0].matches.len(), 1);
    assert_eq!(rete.alpha_memories[1].matches.len(), 1);
}

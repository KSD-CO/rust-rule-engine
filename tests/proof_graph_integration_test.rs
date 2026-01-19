//! Integration test for ProofGraph caching with backward chaining
//!
//! This test demonstrates how ProofGraph caches proven facts across
//! multiple queries, avoiding re-exploration when facts are already known.

#[cfg(feature = "backward-chaining")]
use rust_rule_engine::backward::{BackwardEngine, FactKey, ProofGraph};
use rust_rule_engine::{Facts, KnowledgeBase, Value};

#[test]
#[cfg(feature = "backward-chaining")]
fn test_proof_graph_caching_basic() {
    // Create knowledge base with rules
    let kb = KnowledgeBase::new("test_kb");

    // Create facts
    let mut facts = Facts::new();
    facts.set("User.Age", Value::Integer(25));

    // Create backward engine
    let mut engine = BackwardEngine::new(kb);

    // First query - should explore and prove
    let _result1 = engine.query("User.Age >= 18", &mut facts).unwrap();

    // Note: ProofGraph is internal to search; this test verifies behavior
    // In a real scenario with IncrementalEngine, cache would be populated
}

#[test]
#[cfg(feature = "backward-chaining")]
fn test_proof_graph_invalidation() {
    // Test that demonstrates proof graph structure
    let mut graph = ProofGraph::new();

    use rust_rule_engine::rete::FactHandle;

    // Create fact handles
    let premise_handle = FactHandle::new(1);
    let conclusion_handle = FactHandle::new(2);

    // Create fact keys
    let premise_key = FactKey::from_pattern("User.Age >= 18");
    let conclusion_key = FactKey::from_pattern("User.CanVote == true");

    // Insert premise proof
    graph.insert_proof(
        premise_handle,
        premise_key.clone(),
        "AgeFactRule".to_string(),
        vec![],
        vec![],
    );

    // Insert conclusion proof depending on premise
    graph.insert_proof(
        conclusion_handle,
        conclusion_key.clone(),
        "VotingRule".to_string(),
        vec![premise_handle],
        vec!["User.Age >= 18".to_string()],
    );

    // Both should be proven
    assert!(graph.is_proven(&premise_key));
    assert!(graph.is_proven(&conclusion_key));

    // Invalidate premise (simulating retraction by TMS)
    graph.invalidate_handle(&premise_handle);

    // Conclusion should now be invalid
    let conclusion_node = graph.get_node(&conclusion_handle).unwrap();
    assert!(!conclusion_node.valid);

    // Stats should show invalidations
    assert_eq!(graph.stats.invalidations, 2); // premise + dependent
}

#[test]
#[cfg(feature = "backward-chaining")]
fn test_proof_graph_multiple_justifications() {
    let mut graph = ProofGraph::new();
    use rust_rule_engine::rete::FactHandle;

    let handle = FactHandle::new(1);
    let key = FactKey::from_pattern("User.IsVIP == true");

    // Add first justification (high spender)
    graph.insert_proof(
        handle,
        key.clone(),
        "HighSpenderRule".to_string(),
        vec![],
        vec![],
    );

    // Add second justification (loyalty points)
    graph.insert_proof(
        handle,
        key.clone(),
        "LoyaltyRule".to_string(),
        vec![],
        vec![],
    );

    let node = graph.get_node(&handle).unwrap();
    assert_eq!(node.justifications.len(), 2);
    assert!(node.valid);

    // Fact should remain valid even if one justification is removed
    // (in a full TMS, this would be tracked)
}

#[test]
#[cfg(feature = "backward-chaining")]
fn test_proof_graph_cache_statistics() {
    let mut graph = ProofGraph::new();
    let key = FactKey::from_pattern("User.Active == true");

    // First lookup - cache miss
    assert!(!graph.is_proven(&key));
    assert_eq!(graph.stats.cache_misses, 1);
    assert_eq!(graph.stats.cache_hits, 0);

    // Insert proof
    use rust_rule_engine::rete::FactHandle;
    let handle = FactHandle::new(1);
    graph.insert_proof(
        handle,
        key.clone(),
        "ActiveRule".to_string(),
        vec![],
        vec![],
    );

    // Second lookup - cache hit
    assert!(graph.is_proven(&key));
    assert_eq!(graph.stats.cache_hits, 1);
    assert_eq!(graph.stats.cache_misses, 1);

    // Third lookup - another hit
    assert!(graph.is_proven(&key));
    assert_eq!(graph.stats.cache_hits, 2);

    // Calculate hit rate
    let total = graph.stats.cache_hits + graph.stats.cache_misses;
    let hit_rate = (graph.stats.cache_hits as f64) / (total as f64);
    assert!(hit_rate > 0.66); // 2 hits out of 3 lookups
}

#[test]
#[cfg(feature = "backward-chaining")]
fn test_fact_key_parsing() {
    // Test various pattern formats
    let key1 = FactKey::from_pattern("User.Score >= 80");
    assert_eq!(key1.fact_type, "User");
    assert_eq!(key1.field, Some("Score".to_string()));

    let key2 = FactKey::from_pattern("Order.Status == \"shipped\"");
    assert_eq!(key2.fact_type, "Order");
    assert_eq!(key2.field, Some("Status".to_string()));

    let key3 = FactKey::from_pattern("SimplePattern");
    assert_eq!(key3.fact_type, "SimplePattern");
    assert_eq!(key3.field, None);
}

#[test]
#[cfg(feature = "backward-chaining")]
fn test_proof_graph_dependency_propagation() {
    let mut graph = ProofGraph::new();
    use rust_rule_engine::rete::FactHandle;

    // Create a chain: A -> B -> C
    let handle_a = FactHandle::new(1);
    let handle_b = FactHandle::new(2);
    let handle_c = FactHandle::new(3);

    let key_a = FactKey::from_pattern("A == true");
    let key_b = FactKey::from_pattern("B == true");
    let key_c = FactKey::from_pattern("C == true");

    // Insert A
    graph.insert_proof(handle_a, key_a.clone(), "RuleA".to_string(), vec![], vec![]);

    // Insert B (depends on A)
    graph.insert_proof(
        handle_b,
        key_b.clone(),
        "RuleB".to_string(),
        vec![handle_a],
        vec!["A == true".to_string()],
    );

    // Insert C (depends on B)
    graph.insert_proof(
        handle_c,
        key_c.clone(),
        "RuleC".to_string(),
        vec![handle_b],
        vec!["B == true".to_string()],
    );

    // All should be valid
    assert!(graph.is_proven(&key_a));
    assert!(graph.is_proven(&key_b));
    assert!(graph.is_proven(&key_c));

    // Invalidate A - should cascade to B and C
    graph.invalidate_handle(&handle_a);

    let node_b = graph.get_node(&handle_b).unwrap();
    let node_c = graph.get_node(&handle_c).unwrap();

    assert!(!node_b.valid);
    assert!(!node_c.valid);

    // Should have invalidated 3 nodes total
    assert_eq!(graph.stats.invalidations, 3);
}

//! Memoization support for RETE-UL evaluation
//!
//! This module provides caching mechanisms to avoid re-evaluating the same
//! node with the same facts multiple times, significantly improving performance
//! for complex rule networks.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use super::network::ReteUlNode;
use super::facts::TypedFacts;

/// Compute hash for facts (for memoization key)
fn compute_facts_hash(facts: &TypedFacts) -> u64 {
    let mut hasher = DefaultHasher::new();
    let mut sorted_facts: Vec<_> = facts.get_all().iter().collect();
    sorted_facts.sort_by_key(|(k, _)| *k);

    for (key, value) in sorted_facts {
        key.hash(&mut hasher);
        value.as_string().hash(&mut hasher);
    }

    hasher.finish()
}

/// Compute hash for a node (for memoization key)
fn compute_node_hash(node: &ReteUlNode) -> u64 {
    let mut hasher = DefaultHasher::new();
    // Simple hash based on node type and structure
    format!("{:?}", node).hash(&mut hasher);
    hasher.finish()
}

/// Memoization cache for RETE-UL evaluation
pub struct MemoizedEvaluator {
    cache: HashMap<(u64, u64), bool>,
    hits: usize,
    misses: usize,
}

impl MemoizedEvaluator {
    /// Create new memoized evaluator
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Evaluate node with memoization
    pub fn evaluate(
        &mut self,
        node: &ReteUlNode,
        facts: &TypedFacts,
        eval_fn: impl FnOnce(&ReteUlNode, &TypedFacts) -> bool,
    ) -> bool {
        let node_hash = compute_node_hash(node);
        let facts_hash = compute_facts_hash(facts);
        let key = (node_hash, facts_hash);

        if let Some(&result) = self.cache.get(&key) {
            self.hits += 1;
            return result;
        }

        self.misses += 1;
        let result = eval_fn(node, facts);
        self.cache.insert(key, result);
        result
    }

    /// Get cache statistics
    pub fn stats(&self) -> MemoStats {
        MemoStats {
            cache_size: self.cache.len(),
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for MemoizedEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Memoization statistics
#[derive(Debug, Clone, Copy)]
pub struct MemoStats {
    pub cache_size: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
}

impl std::fmt::Display for MemoStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Memo Stats: {} entries, {} hits, {} misses, {:.2}% hit rate",
            self.cache_size,
            self.hits,
            self.misses,
            self.hit_rate * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rete::alpha::AlphaNode;
    use crate::rete::network::ReteUlNode;
    use crate::rete::facts::{TypedFacts, FactValue};

    #[test]
    fn test_memoization() {
        let mut evaluator = MemoizedEvaluator::new();
        let mut facts = TypedFacts::new();
        facts.set("age", 25i64);

        let node = ReteUlNode::UlAlpha(AlphaNode {
            field: "age".to_string(),
            operator: ">".to_string(),
            value: "18".to_string(),
        });

        // First evaluation - cache miss
        let mut eval_count = 0;
        let result1 = evaluator.evaluate(&node, &facts, |n, f| {
            eval_count += 1;
            n.evaluate_typed(f)
        });
        assert!(result1);
        assert_eq!(eval_count, 1);

        // Second evaluation - cache hit
        let result2 = evaluator.evaluate(&node, &facts, |n, f| {
            eval_count += 1;
            n.evaluate_typed(f)
        });
        assert!(result2);
        assert_eq!(eval_count, 1); // Should not re-evaluate!

        let stats = evaluator.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }
}

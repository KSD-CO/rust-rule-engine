//! Proof Graph for Incremental Caching and TMS Integration
//!
//! This module provides a global cache of proven facts with dependency tracking,
//! enabling reuse across multiple queries and incremental updates when facts change.
//!
//! Architecture:
//! - ProofGraph: maintains mapping from FactKeys to proven facts with justifications
//! - ProofGraphNode: represents a proven fact with its supporting premises and rules
//! - Integration with IncrementalEngine for TMS-aware retraction propagation

use crate::rete::FactHandle;
use crate::types::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Canonical key for identifying a fact (type + field values)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactKey {
    /// Fact type (e.g., "User", "Order")
    pub fact_type: String,

    /// Field name if specific field query (e.g., "User.Score")
    pub field: Option<String>,

    /// Expected value for field (e.g., "Score >= 80")
    pub pattern: String,
}

impl FactKey {
    /// Create a new fact key from a pattern string
    pub fn from_pattern(pattern: &str) -> Self {
        // Parse pattern like "User.Score >= 80" into components
        if let Some(dot_pos) = pattern.find('.') {
            let fact_type = pattern[..dot_pos].trim().to_string();
            let rest = &pattern[dot_pos + 1..];

            // Extract field name (before operator)
            let field = if let Some(op_pos) = rest.find(|c: char| !c.is_alphanumeric() && c != '_')
            {
                Some(rest[..op_pos].trim().to_string())
            } else {
                Some(rest.trim().to_string())
            };

            Self {
                fact_type,
                field,
                pattern: pattern.to_string(),
            }
        } else {
            // Simple pattern without dot notation
            Self {
                fact_type: pattern.to_string(),
                field: None,
                pattern: pattern.to_string(),
            }
        }
    }

    /// Create from explicit components
    pub fn new(fact_type: String, field: Option<String>, pattern: String) -> Self {
        Self {
            fact_type,
            field,
            pattern,
        }
    }
}

/// A justification for a proven fact (one way it was derived)
#[derive(Debug, Clone)]
pub struct Justification {
    /// Rule that produced this fact
    pub rule_name: String,

    /// Premise fact handles that were used
    pub premises: Vec<FactHandle>,

    /// Premise keys (for human-readable tracing)
    pub premise_keys: Vec<String>,

    /// When this justification was created (generation/timestamp)
    pub generation: u64,
}

/// A node in the proof graph representing a proven fact
#[derive(Debug, Clone)]
pub struct ProofGraphNode {
    /// Unique key for this fact
    pub key: FactKey,

    /// Fact handle from IncrementalEngine (if inserted logically)
    pub handle: Option<FactHandle>,

    /// All justifications (ways this fact was proven)
    pub justifications: Vec<Justification>,

    /// Dependents (facts that depend on this fact as premise)
    pub dependents: HashSet<FactHandle>,

    /// Whether this fact is currently valid
    pub valid: bool,

    /// Generation when last validated
    pub generation: u64,

    /// Variable bindings (if any) associated with this proof
    pub bindings: HashMap<String, Value>,
}

impl ProofGraphNode {
    /// Create a new proof graph node
    pub fn new(key: FactKey) -> Self {
        Self {
            key,
            handle: None,
            justifications: Vec::new(),
            dependents: HashSet::new(),
            valid: true,
            generation: 0,
            bindings: HashMap::new(),
        }
    }

    /// Add a justification
    pub fn add_justification(
        &mut self,
        rule_name: String,
        premises: Vec<FactHandle>,
        premise_keys: Vec<String>,
        generation: u64,
    ) {
        self.justifications.push(Justification {
            rule_name,
            premises,
            premise_keys,
            generation,
        });
        self.valid = true;
        self.generation = generation;
    }

    /// Check if this node has any valid justifications
    pub fn has_valid_justifications(&self) -> bool {
        !self.justifications.is_empty()
    }

    /// Remove a justification involving a retracted premise
    pub fn remove_justifications_with_premise(&mut self, premise_handle: &FactHandle) -> bool {
        let before = self.justifications.len();
        self.justifications
            .retain(|j| !j.premises.contains(premise_handle));
        let after = self.justifications.len();

        // If no justifications left, mark invalid
        if self.justifications.is_empty() {
            self.valid = false;
        }

        before != after
    }
}

/// Global proof graph cache
pub struct ProofGraph {
    /// Nodes indexed by fact handle
    nodes_by_handle: HashMap<FactHandle, ProofGraphNode>,

    /// Index from fact key to handles (for pattern lookup)
    index_by_key: HashMap<FactKey, Vec<FactHandle>>,

    /// Reverse dependency index (premise -> dependents)
    dependencies: HashMap<FactHandle, HashSet<FactHandle>>,

    /// Generation counter for tracking updates
    generation: u64,

    /// Statistics
    pub stats: ProofGraphStats,
}

/// Statistics about proof graph usage
#[derive(Debug, Clone, Default)]
pub struct ProofGraphStats {
    pub total_nodes: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub invalidations: usize,
    pub justifications_added: usize,
}

impl ProofGraph {
    /// Create a new proof graph
    pub fn new() -> Self {
        Self {
            nodes_by_handle: HashMap::new(),
            index_by_key: HashMap::new(),
            dependencies: HashMap::new(),
            generation: 0,
            stats: ProofGraphStats::default(),
        }
    }

    /// Insert a proof into the graph
    pub fn insert_proof(
        &mut self,
        handle: FactHandle,
        key: FactKey,
        rule_name: String,
        premises: Vec<FactHandle>,
        premise_keys: Vec<String>,
    ) {
        self.generation += 1;

        // Get or create node
        let node = self.nodes_by_handle.entry(handle).or_insert_with(|| {
            let mut node = ProofGraphNode::new(key.clone());
            node.handle = Some(handle);
            self.stats.total_nodes += 1;
            node
        });

        // Add justification
        node.add_justification(rule_name, premises.clone(), premise_keys, self.generation);
        self.stats.justifications_added += 1;

        // Update key index
        self.index_by_key
            .entry(key.clone())
            .or_default()
            .push(handle);

        // Update dependency edges
        for premise in &premises {
            self.dependencies
                .entry(*premise)
                .or_default()
                .insert(handle);

            // Also update the premise node's dependents
            if let Some(premise_node) = self.nodes_by_handle.get_mut(premise) {
                premise_node.dependents.insert(handle);
            }
        }
    }

    /// Lookup proven facts by key pattern
    pub fn lookup_by_key(&mut self, key: &FactKey) -> Option<Vec<&ProofGraphNode>> {
        if let Some(handles) = self.index_by_key.get(key) {
            let nodes: Vec<&ProofGraphNode> = handles
                .iter()
                .filter_map(|h| self.nodes_by_handle.get(h))
                .filter(|n| n.valid)
                .collect();

            if !nodes.is_empty() {
                self.stats.cache_hits += 1;
                Some(nodes)
            } else {
                self.stats.cache_misses += 1;
                None
            }
        } else {
            self.stats.cache_misses += 1;
            None
        }
    }

    /// Check if a fact key has been proven
    pub fn is_proven(&mut self, key: &FactKey) -> bool {
        self.lookup_by_key(key).is_some()
    }

    /// Invalidate a fact handle (e.g., when retracted by TMS)
    pub fn invalidate_handle(&mut self, handle: &FactHandle) {
        self.stats.invalidations += 1;

        // Get dependents before removing
        let dependents = self.dependencies.get(handle).cloned();

        // Mark node invalid
        if let Some(node) = self.nodes_by_handle.get_mut(handle) {
            node.valid = false;
        }

        // Propagate to dependents
        if let Some(deps) = dependents {
            for dep_handle in deps {
                self.propagate_invalidation(&dep_handle, handle);
            }
        }
    }

    /// Propagate invalidation to a dependent fact
    fn propagate_invalidation(
        &mut self,
        dependent_handle: &FactHandle,
        premise_handle: &FactHandle,
    ) {
        if let Some(node) = self.nodes_by_handle.get_mut(dependent_handle) {
            // Remove justifications that depend on this premise
            let changed = node.remove_justifications_with_premise(premise_handle);

            // If node became invalid (no justifications left), propagate further
            if changed && !node.valid {
                self.stats.invalidations += 1;

                // Get dependents and propagate recursively
                let further_deps = node.dependents.clone();
                for further_dep in further_deps {
                    self.propagate_invalidation(&further_dep, dependent_handle);
                }
            }
        }
    }

    /// Get a node by handle
    pub fn get_node(&self, handle: &FactHandle) -> Option<&ProofGraphNode> {
        self.nodes_by_handle.get(handle)
    }

    /// Clear all cached proofs (reset graph)
    pub fn clear(&mut self) {
        self.nodes_by_handle.clear();
        self.index_by_key.clear();
        self.dependencies.clear();
        self.generation = 0;
        self.stats = ProofGraphStats::default();
    }

    /// Get current generation counter
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Print statistics
    pub fn print_stats(&self) {
        println!("ProofGraph Statistics:");
        println!("  Total nodes: {}", self.stats.total_nodes);
        println!("  Cache hits: {}", self.stats.cache_hits);
        println!("  Cache misses: {}", self.stats.cache_misses);
        println!("  Invalidations: {}", self.stats.invalidations);
        println!(
            "  Justifications added: {}",
            self.stats.justifications_added
        );

        if self.stats.cache_hits + self.stats.cache_misses > 0 {
            let hit_rate = (self.stats.cache_hits as f64)
                / ((self.stats.cache_hits + self.stats.cache_misses) as f64)
                * 100.0;
            println!("  Cache hit rate: {:.1}%", hit_rate);
        }
    }
}

impl Default for ProofGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe wrapper for ProofGraph
pub type SharedProofGraph = Arc<std::sync::Mutex<ProofGraph>>;

/// Create a new shared proof graph
pub fn new_shared() -> SharedProofGraph {
    Arc::new(std::sync::Mutex::new(ProofGraph::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fact_key_from_pattern() {
        let key = FactKey::from_pattern("User.Score >= 80");
        assert_eq!(key.fact_type, "User");
        assert_eq!(key.field, Some("Score".to_string()));
        assert_eq!(key.pattern, "User.Score >= 80");
    }

    #[test]
    fn test_proof_graph_insert_and_lookup() {
        let mut graph = ProofGraph::new();
        let handle = FactHandle::new(1);
        let key = FactKey::from_pattern("User.Score >= 80");

        graph.insert_proof(handle, key.clone(), "ScoreRule".to_string(), vec![], vec![]);

        assert!(graph.is_proven(&key));
        assert_eq!(graph.stats.total_nodes, 1);
    }

    #[test]
    fn test_dependency_tracking() {
        let mut graph = ProofGraph::new();
        let premise_handle = FactHandle::new(1);
        let conclusion_handle = FactHandle::new(2);

        let premise_key = FactKey::from_pattern("User.Age >= 18");
        let conclusion_key = FactKey::from_pattern("User.CanVote == true");

        // Insert premise
        graph.insert_proof(
            premise_handle,
            premise_key.clone(),
            "AgeRule".to_string(),
            vec![],
            vec![],
        );

        // Insert conclusion depending on premise
        graph.insert_proof(
            conclusion_handle,
            conclusion_key.clone(),
            "VotingRule".to_string(),
            vec![premise_handle],
            vec!["User.Age >= 18".to_string()],
        );

        assert!(graph.is_proven(&premise_key));
        assert!(graph.is_proven(&conclusion_key));

        // Invalidate premise
        graph.invalidate_handle(&premise_handle);

        // Conclusion should now be invalid
        let conclusion_node = graph.get_node(&conclusion_handle).unwrap();
        assert!(!conclusion_node.valid);
        assert_eq!(graph.stats.invalidations, 2); // premise + dependent
    }

    #[test]
    fn test_multiple_justifications() {
        let mut graph = ProofGraph::new();
        let handle = FactHandle::new(1);
        let key = FactKey::from_pattern("User.IsVIP == true");

        // Add first justification
        graph.insert_proof(
            handle,
            key.clone(),
            "HighSpenderRule".to_string(),
            vec![],
            vec![],
        );

        // Add second justification for same fact
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
    }

    #[test]
    fn test_cache_statistics() {
        let mut graph = ProofGraph::new();
        let key = FactKey::from_pattern("User.Active == true");

        // Miss
        assert!(!graph.is_proven(&key));
        assert_eq!(graph.stats.cache_misses, 1);

        // Insert
        let handle = FactHandle::new(1);
        graph.insert_proof(
            handle,
            key.clone(),
            "ActiveRule".to_string(),
            vec![],
            vec![],
        );

        // Hit
        assert!(graph.is_proven(&key));
        assert_eq!(graph.stats.cache_hits, 1);
    }

    #[test]
    fn test_clear() {
        let mut graph = ProofGraph::new();
        let handle = FactHandle::new(1);
        let key = FactKey::from_pattern("Test.Value == 42");

        graph.insert_proof(handle, key.clone(), "TestRule".to_string(), vec![], vec![]);
        assert!(graph.is_proven(&key));

        graph.clear();
        assert!(!graph.is_proven(&key));
        assert_eq!(graph.stats.total_nodes, 0);
    }
}

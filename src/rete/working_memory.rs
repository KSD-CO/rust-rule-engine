//! Working Memory for RETE-UL (Drools-style)
//!
//! This module implements a Working Memory system similar to Drools, providing:
//! - FactHandle for tracking inserted objects
//! - Insert, update, retract operations
//! - Type indexing for fast lookups
//! - Change tracking for incremental updates

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use super::facts::{FactValue, TypedFacts};

/// Unique handle for a fact in working memory (similar to Drools FactHandle)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FactHandle(u64);

impl FactHandle {
    /// Create a new fact handle with a unique ID
    fn new(id: u64) -> Self {
        Self(id)
    }

    /// Get the handle ID
    pub fn id(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for FactHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FactHandle({})", self.0)
    }
}

/// A fact stored in working memory
#[derive(Debug, Clone)]
pub struct WorkingMemoryFact {
    /// The fact handle
    pub handle: FactHandle,
    /// Fact type (e.g., "Person", "Order")
    pub fact_type: String,
    /// The actual fact data
    pub data: TypedFacts,
    /// Metadata
    pub metadata: FactMetadata,
}

/// Metadata for a fact
#[derive(Debug, Clone)]
pub struct FactMetadata {
    /// When the fact was inserted
    pub inserted_at: std::time::Instant,
    /// When the fact was last updated
    pub updated_at: std::time::Instant,
    /// Number of updates
    pub update_count: usize,
    /// Is this fact retracted?
    pub retracted: bool,
}

impl Default for FactMetadata {
    fn default() -> Self {
        let now = std::time::Instant::now();
        Self {
            inserted_at: now,
            updated_at: now,
            update_count: 0,
            retracted: false,
        }
    }
}

/// Working Memory - stores and manages facts (Drools-style)
pub struct WorkingMemory {
    /// All facts by handle
    facts: HashMap<FactHandle, WorkingMemoryFact>,
    /// Type index: fact_type -> set of handles
    type_index: HashMap<String, HashSet<FactHandle>>,
    /// Next fact ID
    next_id: AtomicU64,
    /// Modified handles since last propagation
    modified_handles: HashSet<FactHandle>,
    /// Retracted handles since last propagation
    retracted_handles: HashSet<FactHandle>,
}

impl WorkingMemory {
    /// Create a new empty working memory
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            type_index: HashMap::new(),
            next_id: AtomicU64::new(1),
            modified_handles: HashSet::new(),
            retracted_handles: HashSet::new(),
        }
    }

    /// Insert a fact into working memory (returns FactHandle)
    pub fn insert(&mut self, fact_type: String, data: TypedFacts) -> FactHandle {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let handle = FactHandle::new(id);

        let fact = WorkingMemoryFact {
            handle,
            fact_type: fact_type.clone(),
            data,
            metadata: FactMetadata::default(),
        };

        self.facts.insert(handle, fact);
        self.type_index
            .entry(fact_type)
            .or_insert_with(HashSet::new)
            .insert(handle);
        self.modified_handles.insert(handle);

        handle
    }

    /// Update a fact in working memory
    pub fn update(&mut self, handle: FactHandle, data: TypedFacts) -> Result<(), String> {
        let fact = self.facts.get_mut(&handle)
            .ok_or_else(|| format!("FactHandle {} not found", handle))?;

        if fact.metadata.retracted {
            return Err(format!("FactHandle {} is retracted", handle));
        }

        fact.data = data;
        fact.metadata.updated_at = std::time::Instant::now();
        fact.metadata.update_count += 1;
        self.modified_handles.insert(handle);

        Ok(())
    }

    /// Retract (delete) a fact from working memory
    pub fn retract(&mut self, handle: FactHandle) -> Result<(), String> {
        let fact = self.facts.get_mut(&handle)
            .ok_or_else(|| format!("FactHandle {} not found", handle))?;

        if fact.metadata.retracted {
            return Err(format!("FactHandle {} already retracted", handle));
        }

        fact.metadata.retracted = true;
        self.retracted_handles.insert(handle);

        // Remove from type index
        if let Some(handles) = self.type_index.get_mut(&fact.fact_type) {
            handles.remove(&handle);
        }

        Ok(())
    }

    /// Get a fact by handle
    pub fn get(&self, handle: &FactHandle) -> Option<&WorkingMemoryFact> {
        self.facts.get(handle).filter(|f| !f.metadata.retracted)
    }

    /// Get all facts of a specific type
    pub fn get_by_type(&self, fact_type: &str) -> Vec<&WorkingMemoryFact> {
        if let Some(handles) = self.type_index.get(fact_type) {
            handles
                .iter()
                .filter_map(|h| self.facts.get(h))
                .filter(|f| !f.metadata.retracted)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all facts
    pub fn get_all_facts(&self) -> Vec<&WorkingMemoryFact> {
        self.facts
            .values()
            .filter(|f| !f.metadata.retracted)
            .collect()
    }

    /// Get all fact handles
    pub fn get_all_handles(&self) -> Vec<FactHandle> {
        self.facts
            .values()
            .filter(|f| !f.metadata.retracted)
            .map(|f| f.handle)
            .collect()
    }

    /// Get modified handles since last clear
    pub fn get_modified_handles(&self) -> &HashSet<FactHandle> {
        &self.modified_handles
    }

    /// Get retracted handles since last clear
    pub fn get_retracted_handles(&self) -> &HashSet<FactHandle> {
        &self.retracted_handles
    }

    /// Clear modification tracking (after propagation)
    pub fn clear_modification_tracking(&mut self) {
        self.modified_handles.clear();
        self.retracted_handles.clear();
    }

    /// Get statistics
    pub fn stats(&self) -> WorkingMemoryStats {
        let active_facts = self.facts.values().filter(|f| !f.metadata.retracted).count();
        let retracted_facts = self.facts.values().filter(|f| f.metadata.retracted).count();

        WorkingMemoryStats {
            total_facts: self.facts.len(),
            active_facts,
            retracted_facts,
            types: self.type_index.len(),
            modified_pending: self.modified_handles.len(),
            retracted_pending: self.retracted_handles.len(),
        }
    }

    /// Clear all facts
    pub fn clear(&mut self) {
        self.facts.clear();
        self.type_index.clear();
        self.modified_handles.clear();
        self.retracted_handles.clear();
    }

    /// Flatten all facts into a single TypedFacts for evaluation
    /// Each fact's fields are prefixed with "type.handle."
    pub fn to_typed_facts(&self) -> TypedFacts {
        let mut result = TypedFacts::new();

        for fact in self.get_all_facts() {
            let prefix = format!("{}.{}", fact.fact_type, fact.handle.id());
            for (key, value) in fact.data.get_all() {
                result.set(format!("{}.{}", prefix, key), value.clone());
            }

            // Also add without prefix for simple access
            for (key, value) in fact.data.get_all() {
                result.set(format!("{}.{}", fact.fact_type, key), value.clone());
            }
        }

        result
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Working memory statistics
#[derive(Debug, Clone)]
pub struct WorkingMemoryStats {
    pub total_facts: usize,
    pub active_facts: usize,
    pub retracted_facts: usize,
    pub types: usize,
    pub modified_pending: usize,
    pub retracted_pending: usize,
}

impl std::fmt::Display for WorkingMemoryStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WM Stats: {} active, {} retracted, {} types, {} modified, {} pending retraction",
            self.active_facts, self.retracted_facts, self.types,
            self.modified_pending, self.retracted_pending
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut wm = WorkingMemory::new();
        let mut person_data = TypedFacts::new();
        person_data.set("name", "John");
        person_data.set("age", 25i64);

        let handle = wm.insert("Person".to_string(), person_data);

        let fact = wm.get(&handle).unwrap();
        assert_eq!(fact.fact_type, "Person");
        assert_eq!(fact.data.get("name").unwrap().as_string(), "John");
    }

    #[test]
    fn test_update() {
        let mut wm = WorkingMemory::new();
        let mut data = TypedFacts::new();
        data.set("age", 25i64);

        let handle = wm.insert("Person".to_string(), data);

        let mut updated_data = TypedFacts::new();
        updated_data.set("age", 26i64);
        wm.update(handle, updated_data).unwrap();

        let fact = wm.get(&handle).unwrap();
        assert_eq!(fact.data.get("age").unwrap().as_integer(), Some(26));
        assert_eq!(fact.metadata.update_count, 1);
    }

    #[test]
    fn test_retract() {
        let mut wm = WorkingMemory::new();
        let data = TypedFacts::new();
        let handle = wm.insert("Person".to_string(), data);

        wm.retract(handle).unwrap();

        assert!(wm.get(&handle).is_none());
        assert_eq!(wm.get_all_facts().len(), 0);
    }

    #[test]
    fn test_type_index() {
        let mut wm = WorkingMemory::new();

        for i in 0..5 {
            let mut data = TypedFacts::new();
            data.set("id", i as i64);
            wm.insert("Person".to_string(), data);
        }

        for i in 0..3 {
            let mut data = TypedFacts::new();
            data.set("id", i as i64);
            wm.insert("Order".to_string(), data);
        }

        assert_eq!(wm.get_by_type("Person").len(), 5);
        assert_eq!(wm.get_by_type("Order").len(), 3);
        assert_eq!(wm.get_by_type("Unknown").len(), 0);
    }

    #[test]
    fn test_modification_tracking() {
        let mut wm = WorkingMemory::new();
        let data = TypedFacts::new();
        let h1 = wm.insert("Person".to_string(), data.clone());
        let h2 = wm.insert("Person".to_string(), data.clone());

        assert_eq!(wm.get_modified_handles().len(), 2);

        wm.clear_modification_tracking();
        assert_eq!(wm.get_modified_handles().len(), 0);

        wm.update(h1, data.clone()).unwrap();
        assert_eq!(wm.get_modified_handles().len(), 1);

        wm.retract(h2).unwrap();
        assert_eq!(wm.get_retracted_handles().len(), 1);
    }
}

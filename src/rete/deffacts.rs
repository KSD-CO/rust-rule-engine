//! Initial Facts System (inspired by CLIPS deffacts)
//!
//! Provides pre-defined fact sets that are automatically loaded into working memory.
//! Similar to CLIPS deffacts and Drools declared facts.

use crate::errors::{Result, RuleEngineError};
use crate::rete::facts::TypedFacts;
use std::collections::HashMap;

/// A single fact instance with its type
#[derive(Debug, Clone)]
pub struct FactInstance {
    /// The fact type (e.g., "Person", "Order", "Config")
    pub fact_type: String,
    /// The fact data
    pub data: TypedFacts,
}

impl FactInstance {
    /// Create a new fact instance
    pub fn new(fact_type: impl Into<String>, data: TypedFacts) -> Self {
        Self {
            fact_type: fact_type.into(),
            data,
        }
    }
}

/// A named set of initial facts (deffacts)
#[derive(Debug, Clone)]
pub struct Deffacts {
    /// Name of this deffacts set
    pub name: String,
    /// Collection of initial facts
    pub facts: Vec<FactInstance>,
    /// Optional description
    pub description: Option<String>,
}

impl Deffacts {
    /// Create a new deffacts set
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            facts: Vec::new(),
            description: None,
        }
    }

    /// Add a fact to this deffacts set
    pub fn add_fact(&mut self, fact_type: impl Into<String>, data: TypedFacts) {
        self.facts.push(FactInstance::new(fact_type, data));
    }

    /// Set the description
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = Some(description.into());
    }

    /// Get the number of facts
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    /// Check if this deffacts is empty
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
}

/// Registry for managing deffacts
/// Stores named sets of initial facts that can be loaded into working memory
#[derive(Debug, Clone)]
pub struct DeffactsRegistry {
    deffacts: HashMap<String, Deffacts>,
}

impl DeffactsRegistry {
    /// Create a new deffacts registry
    pub fn new() -> Self {
        Self {
            deffacts: HashMap::new(),
        }
    }

    /// Register a deffacts set
    pub fn register(&mut self, deffacts: Deffacts) -> Result<()> {
        let name = deffacts.name.clone();

        if self.deffacts.contains_key(&name) {
            return Err(RuleEngineError::EvaluationError {
                message: format!("Deffacts '{}' already exists", name),
            });
        }

        self.deffacts.insert(name, deffacts);
        Ok(())
    }

    /// Register a deffacts set, replacing if it exists
    pub fn register_or_replace(&mut self, deffacts: Deffacts) {
        let name = deffacts.name.clone();
        self.deffacts.insert(name, deffacts);
    }

    /// Get a deffacts set by name
    pub fn get(&self, name: &str) -> Option<&Deffacts> {
        self.deffacts.get(name)
    }

    /// Get a mutable reference to a deffacts set
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Deffacts> {
        self.deffacts.get_mut(name)
    }

    /// Check if a deffacts set exists
    pub fn exists(&self, name: &str) -> bool {
        self.deffacts.contains_key(name)
    }

    /// Remove a deffacts set
    pub fn remove(&mut self, name: &str) -> Result<Deffacts> {
        self.deffacts
            .remove(name)
            .ok_or_else(|| RuleEngineError::EvaluationError {
                message: format!("Deffacts '{}' not found", name),
            })
    }

    /// List all deffacts names
    pub fn list_deffacts(&self) -> Vec<String> {
        self.deffacts.keys().cloned().collect()
    }

    /// Get all facts from all deffacts sets
    pub fn get_all_facts(&self) -> Vec<(String, FactInstance)> {
        let mut all_facts = Vec::new();

        for (deffacts_name, deffacts) in &self.deffacts {
            for fact in &deffacts.facts {
                all_facts.push((deffacts_name.clone(), fact.clone()));
            }
        }

        all_facts
    }

    /// Get total count of all facts across all deffacts
    pub fn total_fact_count(&self) -> usize {
        self.deffacts.values().map(|d| d.fact_count()).sum()
    }

    /// Clear all deffacts
    pub fn clear(&mut self) {
        self.deffacts.clear();
    }

    /// Get the number of deffacts sets
    pub fn len(&self) -> usize {
        self.deffacts.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.deffacts.is_empty()
    }
}

impl Default for DeffactsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating deffacts with a fluent API
pub struct DeffactsBuilder {
    name: String,
    facts: Vec<FactInstance>,
    description: Option<String>,
}

impl DeffactsBuilder {
    /// Create a new deffacts builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            facts: Vec::new(),
            description: None,
        }
    }

    /// Add a fact to this deffacts
    pub fn add_fact(mut self, fact_type: impl Into<String>, data: TypedFacts) -> Self {
        self.facts.push(FactInstance::new(fact_type, data));
        self
    }

    /// Add multiple facts of the same type
    pub fn add_facts(mut self, fact_type: impl Into<String>, facts: Vec<TypedFacts>) -> Self {
        let fact_type_str = fact_type.into();
        for data in facts {
            self.facts
                .push(FactInstance::new(fact_type_str.clone(), data));
        }
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Build the deffacts
    pub fn build(self) -> Deffacts {
        Deffacts {
            name: self.name,
            facts: self.facts,
            description: self.description,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rete::facts::FactValue;

    #[test]
    fn test_create_fact_instance() {
        let mut data = TypedFacts::new();
        data.set("name", FactValue::String("John".to_string()));
        data.set("age", FactValue::Integer(30));

        let fact = FactInstance::new("Person", data);
        assert_eq!(fact.fact_type, "Person");
        assert_eq!(
            fact.data.get("name"),
            Some(&FactValue::String("John".to_string()))
        );
    }

    #[test]
    fn test_deffacts_basic() {
        let mut deffacts = Deffacts::new("initial-data");

        let mut person_data = TypedFacts::new();
        person_data.set("name", FactValue::String("Alice".to_string()));

        deffacts.add_fact("Person", person_data);
        deffacts.set_description("Initial person data");

        assert_eq!(deffacts.name, "initial-data");
        assert_eq!(deffacts.fact_count(), 1);
        assert!(!deffacts.is_empty());
        assert_eq!(
            deffacts.description,
            Some("Initial person data".to_string())
        );
    }

    #[test]
    fn test_registry_register() {
        let mut registry = DeffactsRegistry::new();

        let mut deffacts = Deffacts::new("test");
        let mut data = TypedFacts::new();
        data.set("value", FactValue::Integer(42));
        deffacts.add_fact("Config", data);

        registry.register(deffacts).unwrap();

        assert!(registry.exists("test"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn test_registry_duplicate_error() {
        let mut registry = DeffactsRegistry::new();

        let deffacts1 = Deffacts::new("test");
        let deffacts2 = Deffacts::new("test");

        registry.register(deffacts1).unwrap();
        let result = registry.register(deffacts2);

        assert!(result.is_err());
    }

    #[test]
    fn test_registry_register_or_replace() {
        let mut registry = DeffactsRegistry::new();

        let mut deffacts1 = Deffacts::new("test");
        let mut data1 = TypedFacts::new();
        data1.set("version", FactValue::Integer(1));
        deffacts1.add_fact("Config", data1);

        let mut deffacts2 = Deffacts::new("test");
        let mut data2 = TypedFacts::new();
        data2.set("version", FactValue::Integer(2));
        deffacts2.add_fact("Config", data2);

        registry.register_or_replace(deffacts1);
        registry.register_or_replace(deffacts2);

        assert_eq!(registry.len(), 1);
        let deffacts = registry.get("test").unwrap();
        assert_eq!(deffacts.fact_count(), 1);
    }

    #[test]
    fn test_registry_get_all_facts() {
        let mut registry = DeffactsRegistry::new();

        // First deffacts with 2 facts
        let mut deffacts1 = Deffacts::new("set1");
        let mut data1 = TypedFacts::new();
        data1.set("name", FactValue::String("Alice".to_string()));
        deffacts1.add_fact("Person", data1);

        let mut data2 = TypedFacts::new();
        data2.set("name", FactValue::String("Bob".to_string()));
        deffacts1.add_fact("Person", data2);

        // Second deffacts with 1 fact
        let mut deffacts2 = Deffacts::new("set2");
        let mut data3 = TypedFacts::new();
        data3.set("debug", FactValue::Boolean(true));
        deffacts2.add_fact("Config", data3);

        registry.register(deffacts1).unwrap();
        registry.register(deffacts2).unwrap();

        let all_facts = registry.get_all_facts();
        assert_eq!(all_facts.len(), 3);
        assert_eq!(registry.total_fact_count(), 3);
    }

    #[test]
    fn test_registry_remove() {
        let mut registry = DeffactsRegistry::new();

        let deffacts = Deffacts::new("temp");
        registry.register(deffacts).unwrap();

        assert!(registry.exists("temp"));

        let removed = registry.remove("temp").unwrap();
        assert_eq!(removed.name, "temp");
        assert!(!registry.exists("temp"));
    }

    #[test]
    fn test_registry_list_deffacts() {
        let mut registry = DeffactsRegistry::new();

        registry.register(Deffacts::new("set1")).unwrap();
        registry.register(Deffacts::new("set2")).unwrap();
        registry.register(Deffacts::new("set3")).unwrap();

        let list = registry.list_deffacts();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&"set1".to_string()));
        assert!(list.contains(&"set2".to_string()));
        assert!(list.contains(&"set3".to_string()));
    }

    #[test]
    fn test_registry_clear() {
        let mut registry = DeffactsRegistry::new();

        registry.register(Deffacts::new("set1")).unwrap();
        registry.register(Deffacts::new("set2")).unwrap();

        assert_eq!(registry.len(), 2);

        registry.clear();

        assert_eq!(registry.len(), 0);
        assert!(registry.is_empty());
    }

    #[test]
    fn test_builder_basic() {
        let mut data = TypedFacts::new();
        data.set("name", FactValue::String("Charlie".to_string()));
        data.set("age", FactValue::Integer(25));

        let deffacts = DeffactsBuilder::new("people")
            .add_fact("Person", data)
            .with_description("Initial people data")
            .build();

        assert_eq!(deffacts.name, "people");
        assert_eq!(deffacts.fact_count(), 1);
        assert_eq!(
            deffacts.description,
            Some("Initial people data".to_string())
        );
    }

    #[test]
    fn test_builder_multiple_facts() {
        let mut person1 = TypedFacts::new();
        person1.set("name", FactValue::String("Alice".to_string()));

        let mut person2 = TypedFacts::new();
        person2.set("name", FactValue::String("Bob".to_string()));

        let mut config = TypedFacts::new();
        config.set("debug", FactValue::Boolean(true));

        let deffacts = DeffactsBuilder::new("startup")
            .add_fact("Person", person1)
            .add_fact("Person", person2)
            .add_fact("Config", config)
            .build();

        assert_eq!(deffacts.fact_count(), 3);
    }

    #[test]
    fn test_builder_add_facts_batch() {
        let mut person1 = TypedFacts::new();
        person1.set("name", FactValue::String("Alice".to_string()));

        let mut person2 = TypedFacts::new();
        person2.set("name", FactValue::String("Bob".to_string()));

        let people = vec![person1, person2];

        let deffacts = DeffactsBuilder::new("batch-people")
            .add_facts("Person", people)
            .build();

        assert_eq!(deffacts.fact_count(), 2);
        assert_eq!(deffacts.facts[0].fact_type, "Person");
        assert_eq!(deffacts.facts[1].fact_type, "Person");
    }

    #[test]
    fn test_integration_with_registry() {
        let mut registry = DeffactsRegistry::new();

        // Create deffacts using builder
        let mut person_data = TypedFacts::new();
        person_data.set("name", FactValue::String("Admin".to_string()));
        person_data.set("role", FactValue::String("administrator".to_string()));

        let mut config_data = TypedFacts::new();
        config_data.set("max_users", FactValue::Integer(1000));
        config_data.set("debug_mode", FactValue::Boolean(false));

        let deffacts = DeffactsBuilder::new("system-startup")
            .add_fact("User", person_data)
            .add_fact("SystemConfig", config_data)
            .with_description("System initialization facts")
            .build();

        // Register it
        registry.register(deffacts).unwrap();

        // Verify
        assert!(registry.exists("system-startup"));
        let retrieved = registry.get("system-startup").unwrap();
        assert_eq!(retrieved.fact_count(), 2);
        assert_eq!(
            retrieved.description,
            Some("System initialization facts".to_string())
        );
    }
}

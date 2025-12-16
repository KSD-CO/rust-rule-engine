//! Global Variables System (inspired by CLIPS defglobal)
//!
//! Provides persistent global variables that can be accessed across rule firings.
//! Similar to CLIPS defglobal and Drools globals.

use crate::errors::{Result, RuleEngineError};
use crate::rete::facts::FactValue;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Global variable definition
#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub name: String,
    pub value: FactValue,
    pub read_only: bool,
}

impl GlobalVar {
    /// Create a new global variable
    pub fn new(name: impl Into<String>, value: FactValue) -> Self {
        Self {
            name: name.into(),
            value,
            read_only: false,
        }
    }

    /// Create a read-only global variable
    pub fn read_only(name: impl Into<String>, value: FactValue) -> Self {
        Self {
            name: name.into(),
            value,
            read_only: true,
        }
    }

    /// Update the value (fails if read-only)
    pub fn set(&mut self, value: FactValue) -> Result<()> {
        if self.read_only {
            return Err(RuleEngineError::EvaluationError {
                message: format!("Cannot modify read-only global '{}'", self.name),
            });
        }
        self.value = value;
        Ok(())
    }

    /// Get the current value
    pub fn get(&self) -> &FactValue {
        &self.value
    }
}

/// Global variables registry
/// Thread-safe storage for global variables shared across rules
#[derive(Debug, Clone)]
pub struct GlobalsRegistry {
    globals: Arc<RwLock<HashMap<String, GlobalVar>>>,
}

impl GlobalsRegistry {
    /// Create a new globals registry
    pub fn new() -> Self {
        Self {
            globals: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Define a global variable
    pub fn define(&self, name: impl Into<String>, value: FactValue) -> Result<()> {
        let var_name = name.into();
        let mut globals = self.globals.write().map_err(|e| {
            RuleEngineError::ExecutionError(format!("Failed to acquire write lock: {}", e))
        })?;

        globals.insert(var_name.clone(), GlobalVar::new(var_name, value));
        Ok(())
    }

    /// Define a read-only global variable
    pub fn define_readonly(&self, name: impl Into<String>, value: FactValue) -> Result<()> {
        let var_name = name.into();
        let mut globals = self.globals.write().map_err(|e| {
            RuleEngineError::ExecutionError(format!("Failed to acquire write lock: {}", e))
        })?;

        globals.insert(var_name.clone(), GlobalVar::read_only(var_name, value));
        Ok(())
    }

    /// Get a global variable value
    pub fn get(&self, name: &str) -> Result<FactValue> {
        let globals = self.globals.read().map_err(|e| {
            RuleEngineError::ExecutionError(format!("Failed to acquire read lock: {}", e))
        })?;

        globals
            .get(name)
            .map(|var| var.value.clone())
            .ok_or_else(|| RuleEngineError::EvaluationError {
                message: format!("Global variable '{}' not found", name),
            })
    }

    /// Set a global variable value
    pub fn set(&self, name: &str, value: FactValue) -> Result<()> {
        let mut globals = self.globals.write().map_err(|e| {
            RuleEngineError::ExecutionError(format!("Failed to acquire write lock: {}", e))
        })?;

        let var = globals
            .get_mut(name)
            .ok_or_else(|| RuleEngineError::EvaluationError {
                message: format!("Global variable '{}' not found", name),
            })?;

        var.set(value)
    }

    /// Check if a global variable exists
    pub fn exists(&self, name: &str) -> bool {
        if let Ok(globals) = self.globals.read() {
            globals.contains_key(name)
        } else {
            false
        }
    }

    /// Remove a global variable
    pub fn remove(&self, name: &str) -> Result<()> {
        let mut globals = self.globals.write().map_err(|e| {
            RuleEngineError::ExecutionError(format!("Failed to acquire write lock: {}", e))
        })?;

        globals
            .remove(name)
            .ok_or_else(|| RuleEngineError::EvaluationError {
                message: format!("Global variable '{}' not found", name),
            })?;

        Ok(())
    }

    /// List all global variable names
    pub fn list_globals(&self) -> Vec<String> {
        if let Ok(globals) = self.globals.read() {
            globals.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Get all globals as a HashMap
    pub fn get_all(&self) -> HashMap<String, FactValue> {
        if let Ok(globals) = self.globals.read() {
            globals
                .iter()
                .map(|(k, v)| (k.clone(), v.value.clone()))
                .collect()
        } else {
            HashMap::new()
        }
    }

    /// Clear all global variables
    pub fn clear(&self) {
        if let Ok(mut globals) = self.globals.write() {
            globals.clear();
        }
    }

    /// Increment a numeric global variable
    pub fn increment(&self, name: &str, delta: f64) -> Result<()> {
        let mut globals = self.globals.write().map_err(|e| {
            RuleEngineError::ExecutionError(format!("Failed to acquire write lock: {}", e))
        })?;

        let var = globals
            .get_mut(name)
            .ok_or_else(|| RuleEngineError::EvaluationError {
                message: format!("Global variable '{}' not found", name),
            })?;

        if var.read_only {
            return Err(RuleEngineError::EvaluationError {
                message: format!("Cannot modify read-only global '{}'", name),
            });
        }

        let new_value = match &var.value {
            FactValue::Integer(i) => FactValue::Integer(i + delta as i64),
            FactValue::Float(f) => FactValue::Float(f + delta),
            _ => {
                return Err(RuleEngineError::EvaluationError {
                    message: format!("Global '{}' is not numeric", name),
                })
            }
        };

        var.value = new_value;
        Ok(())
    }
}

impl Default for GlobalsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for defining multiple globals at once
pub struct GlobalsBuilder {
    registry: GlobalsRegistry,
}

impl GlobalsBuilder {
    /// Create a new globals builder
    pub fn new() -> Self {
        Self {
            registry: GlobalsRegistry::new(),
        }
    }

    /// Add a global variable
    pub fn define(self, name: impl Into<String>, value: FactValue) -> Self {
        let _ = self.registry.define(name, value);
        self
    }

    /// Add a read-only global variable
    pub fn define_readonly(self, name: impl Into<String>, value: FactValue) -> Self {
        let _ = self.registry.define_readonly(name, value);
        self
    }

    /// Build and return the registry
    pub fn build(self) -> GlobalsRegistry {
        self.registry
    }
}

impl Default for GlobalsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_define_and_get() {
        let registry = GlobalsRegistry::new();
        registry.define("counter", FactValue::Integer(0)).unwrap();

        let value = registry.get("counter").unwrap();
        assert_eq!(value, FactValue::Integer(0));
    }

    #[test]
    fn test_set_global() {
        let registry = GlobalsRegistry::new();
        registry
            .define("status", FactValue::String("active".to_string()))
            .unwrap();

        registry
            .set("status", FactValue::String("inactive".to_string()))
            .unwrap();

        let value = registry.get("status").unwrap();
        assert_eq!(value, FactValue::String("inactive".to_string()));
    }

    #[test]
    fn test_readonly_global() {
        let registry = GlobalsRegistry::new();
        registry
            .define_readonly("PI", FactValue::Float(std::f64::consts::PI))
            .unwrap();

        // Should fail to modify
        let result = registry.set("PI", FactValue::Float(3.0));
        assert!(result.is_err());

        // Value should remain unchanged
        let value = registry.get("PI").unwrap();
        assert_eq!(value, FactValue::Float(std::f64::consts::PI));
    }

    #[test]
    fn test_increment() {
        let registry = GlobalsRegistry::new();
        registry.define("counter", FactValue::Integer(10)).unwrap();

        registry.increment("counter", 5.0).unwrap();

        let value = registry.get("counter").unwrap();
        assert_eq!(value, FactValue::Integer(15));
    }

    #[test]
    fn test_list_globals() {
        let registry = GlobalsRegistry::new();
        registry.define("var1", FactValue::Integer(1)).unwrap();
        registry.define("var2", FactValue::Integer(2)).unwrap();

        let list = registry.list_globals();
        assert_eq!(list.len(), 2);
        assert!(list.contains(&"var1".to_string()));
        assert!(list.contains(&"var2".to_string()));
    }

    #[test]
    fn test_remove_global() {
        let registry = GlobalsRegistry::new();
        registry.define("temp", FactValue::Boolean(true)).unwrap();

        assert!(registry.exists("temp"));

        registry.remove("temp").unwrap();

        assert!(!registry.exists("temp"));
    }

    #[test]
    fn test_builder() {
        let registry = GlobalsBuilder::new()
            .define("max_retries", FactValue::Integer(3))
            .define("timeout", FactValue::Float(30.0))
            .define_readonly("VERSION", FactValue::String("1.0.0".to_string()))
            .build();

        assert_eq!(registry.get("max_retries").unwrap(), FactValue::Integer(3));
        assert_eq!(registry.get("timeout").unwrap(), FactValue::Float(30.0));
        assert_eq!(
            registry.get("VERSION").unwrap(),
            FactValue::String("1.0.0".to_string())
        );
    }

    #[test]
    fn test_get_all() {
        let registry = GlobalsRegistry::new();
        registry.define("a", FactValue::Integer(1)).unwrap();
        registry.define("b", FactValue::Integer(2)).unwrap();

        let all = registry.get_all();
        assert_eq!(all.len(), 2);
        assert_eq!(all.get("a"), Some(&FactValue::Integer(1)));
        assert_eq!(all.get("b"), Some(&FactValue::Integer(2)));
    }

    #[test]
    fn test_thread_safety() {
        use std::thread;

        let registry = GlobalsRegistry::new();
        registry
            .define("shared_counter", FactValue::Integer(0))
            .unwrap();

        let registry_clone = registry.clone();
        let handle = thread::spawn(move || {
            registry_clone.increment("shared_counter", 1.0).unwrap();
        });

        handle.join().unwrap();

        let value = registry.get("shared_counter").unwrap();
        assert_eq!(value, FactValue::Integer(1));
    }
}

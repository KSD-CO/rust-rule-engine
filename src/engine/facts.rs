use crate::errors::{Result, RuleEngineError};
use crate::types::{Context, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::collections::HashSet;

/// Facts - represents the working memory of data objects
/// Similar to Grule's DataContext concept
#[derive(Debug, Clone)]
pub struct Facts {
    data: Arc<RwLock<HashMap<String, Value>>>,
    fact_types: Arc<RwLock<HashMap<String, String>>>,
    /// Undo log frames for lightweight snapshots (stack of frames)
    /// Each frame records per-key previous values so rollback can restore only
    /// changed keys instead of cloning the whole facts map.
    undo_frames: Arc<RwLock<Vec<Vec<UndoEntry>>>>,
}

impl Facts {
    /// Create a generic object from key-value pairs
    pub fn create_object(pairs: Vec<(String, Value)>) -> Value {
        let mut map = HashMap::new();
        for (key, value) in pairs {
            map.insert(key, value);
        }
        Value::Object(map)
    }

    /// Create a user object
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            fact_types: Arc::new(RwLock::new(HashMap::new())),
            undo_frames: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add a fact object to the working memory
    pub fn add<T>(&self, name: &str, fact: T) -> Result<()>
    where
        T: Serialize + std::fmt::Debug,
    {
        let value =
            serde_json::to_value(&fact).map_err(|e| RuleEngineError::SerializationError {
                message: e.to_string(),
            })?;

        let fact_value = Value::from(value);

        let mut data = self.data.write().unwrap();
        let mut types = self.fact_types.write().unwrap();

        data.insert(name.to_string(), fact_value);
        types.insert(name.to_string(), std::any::type_name::<T>().to_string());

        Ok(())
    }

    /// Add a simple value fact
    pub fn add_value(&self, name: &str, value: Value) -> Result<()> {
        let mut data = self.data.write().unwrap();
        let mut types = self.fact_types.write().unwrap();

        data.insert(name.to_string(), value);
        types.insert(name.to_string(), "Value".to_string());

        Ok(())
    }

    /// Get a fact by name
    pub fn get(&self, name: &str) -> Option<Value> {
        let data = self.data.read().unwrap();
        data.get(name).cloned()
    }

    /// Get a nested fact property (e.g., "User.Profile.Age")
    pub fn get_nested(&self, path: &str) -> Option<Value> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return None;
        }

        let data = self.data.read().unwrap();
        let mut current = data.get(parts[0])?;

        for part in parts.iter().skip(1) {
            match current {
                Value::Object(ref obj) => {
                    current = obj.get(*part)?;
                }
                _ => return None,
            }
        }

        Some(current.clone())
    }

    /// Set a fact value
    pub fn set(&self, name: &str, value: Value) {
        // Record previous value for undo if an undo frame is active
        self.record_undo_for_key(name);

        let mut data = self.data.write().unwrap();
        data.insert(name.to_string(), value);
    }

    /// Set a nested fact property
    pub fn set_nested(&self, path: &str, value: Value) -> Result<()> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err(RuleEngineError::FieldNotFound {
                field: path.to_string(),
            });
        }

    // Record previous top-level key for undo semantics
    self.record_undo_for_key(parts[0]);

    let mut data = self.data.write().unwrap();

        if parts.len() == 1 {
            data.insert(parts[0].to_string(), value);
            return Ok(());
        }

        // Navigate to parent and set the nested value
        let root_key = parts[0];
        let root_value = data
            .get_mut(root_key)
            .ok_or_else(|| RuleEngineError::FieldNotFound {
                field: root_key.to_string(),
            })?;

        self.set_nested_in_value(root_value, &parts[1..], value)?;
        Ok(())
    }

    #[allow(clippy::only_used_in_recursion)]
    fn set_nested_in_value(&self, current: &mut Value, path: &[&str], value: Value) -> Result<()> {
        if path.is_empty() {
            return Ok(());
        }

        if path.len() == 1 {
            // We're at the target field
            match current {
                Value::Object(ref mut obj) => {
                    obj.insert(path[0].to_string(), value);
                    Ok(())
                }
                _ => Err(RuleEngineError::TypeMismatch {
                    expected: "Object".to_string(),
                    actual: format!("{:?}", current),
                }),
            }
        } else {
            // Continue navigating
            match current {
                Value::Object(ref mut obj) => {
                    let next_value =
                        obj.get_mut(path[0])
                            .ok_or_else(|| RuleEngineError::FieldNotFound {
                                field: path[0].to_string(),
                            })?;
                    self.set_nested_in_value(next_value, &path[1..], value)
                }
                _ => Err(RuleEngineError::TypeMismatch {
                    expected: "Object".to_string(),
                    actual: format!("{:?}", current),
                }),
            }
        }
    }

    /// Remove a fact
    pub fn remove(&self, name: &str) -> Option<Value> {
        // Record undo before removing
        self.record_undo_for_key(name);

        let mut data = self.data.write().unwrap();
        let mut types = self.fact_types.write().unwrap();

        types.remove(name);
        data.remove(name)
    }

    /// Clear all facts
    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        let mut types = self.fact_types.write().unwrap();

        data.clear();
        types.clear();
    }

    /// Get all fact names
    pub fn get_fact_names(&self) -> Vec<String> {
        let data = self.data.read().unwrap();
        data.keys().cloned().collect()
    }

    /// Get fact count
    pub fn count(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }

    /// Check if a fact exists
    pub fn contains(&self, name: &str) -> bool {
        let data = self.data.read().unwrap();
        data.contains_key(name)
    }

    /// Get all facts as a HashMap (for pattern matching evaluation)
    pub fn get_all_facts(&self) -> HashMap<String, Value> {
        let data = self.data.read().unwrap();
        data.clone()
    }

    /// Get the type name of a fact
    pub fn get_fact_type(&self, name: &str) -> Option<String> {
        let types = self.fact_types.read().unwrap();
        types.get(name).cloned()
    }

    /// Convert to Context for rule evaluation
    pub fn to_context(&self) -> Context {
        let data = self.data.read().unwrap();
        data.clone()
    }

    /// Create Facts from Context
    pub fn from_context(context: Context) -> Self {
        let facts = Facts::new();
        {
            let mut data = facts.data.write().unwrap();
            *data = context;
        }
        facts
    }

    /// Merge another Facts instance into this one
    pub fn merge(&self, other: &Facts) {
        let other_data = other.data.read().unwrap();
        let other_types = other.fact_types.read().unwrap();

        let mut data = self.data.write().unwrap();
        let mut types = self.fact_types.write().unwrap();

        for (key, value) in other_data.iter() {
            data.insert(key.clone(), value.clone());
        }

        for (key, type_name) in other_types.iter() {
            types.insert(key.clone(), type_name.clone());
        }
    }

    /// Get a snapshot of all facts
    pub fn snapshot(&self) -> FactsSnapshot {
        let data = self.data.read().unwrap();
        let types = self.fact_types.read().unwrap();

        FactsSnapshot {
            data: data.clone(),
            fact_types: types.clone(),
        }
    }

    /// Restore from a snapshot
    pub fn restore(&self, snapshot: FactsSnapshot) {
        let mut data = self.data.write().unwrap();
        let mut types = self.fact_types.write().unwrap();

        *data = snapshot.data;
        *types = snapshot.fact_types;
    }
}

impl Default for Facts {
    fn default() -> Self {
        Self::new()
    }
}

/// A snapshot of Facts state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactsSnapshot {
    /// The fact data stored as key-value pairs
    pub data: HashMap<String, Value>,
    /// Type information for each fact
    pub fact_types: HashMap<String, String>,
}

/// Undo entry for a single key
#[derive(Debug, Clone)]
struct UndoEntry {
    key: String,
    prev_value: Option<Value>,
    prev_type: Option<String>,
}

impl Facts {
    /// Start a new undo frame. Call `rollback_undo_frame` to revert or
    /// `commit_undo_frame` to discard recorded changes.
    pub fn begin_undo_frame(&self) {
        let mut frames = self.undo_frames.write().unwrap();
        frames.push(Vec::new());
    }

    /// Commit (discard) the top-most undo frame
    pub fn commit_undo_frame(&self) {
        let mut frames = self.undo_frames.write().unwrap();
        frames.pop();
    }

    /// Rollback the top-most undo frame, restoring prior values
    pub fn rollback_undo_frame(&self) {
        let mut frames = self.undo_frames.write().unwrap();
        if let Some(frame) = frames.pop() {
            // Restore in reverse order
            let mut data = self.data.write().unwrap();
            let mut types = self.fact_types.write().unwrap();

            for entry in frame.into_iter().rev() {
                match entry.prev_value {
                    Some(v) => { data.insert(entry.key.clone(), v); }
                    None => { data.remove(&entry.key); }
                }

                match entry.prev_type {
                    Some(t) => { types.insert(entry.key.clone(), t); }
                    None => { types.remove(&entry.key); }
                }
            }
        }
    }

    /// Record prior state for a top-level key if an undo frame is active
    fn record_undo_for_key(&self, key: &str) {
        let mut frames = self.undo_frames.write().unwrap();
        if let Some(frame) = frames.last_mut() {
            // capture previous value & type
            let data = self.data.read().unwrap();
            let types = self.fact_types.read().unwrap();

            // Only record once per key in this frame
            if frame.iter().any(|e: &UndoEntry| e.key == key) {
                return;
            }

            let prev_value = data.get(key).cloned();
            let prev_type = types.get(key).cloned();

            frame.push(UndoEntry {
                key: key.to_string(),
                prev_value,
                prev_type,
            });
        }
    }
}

/// Trait for objects that can be used as facts
pub trait Fact: Serialize + std::fmt::Debug {
    /// Get the name of this fact type
    fn fact_name() -> &'static str;
}

/// Macro to implement Fact trait easily
#[macro_export]
macro_rules! impl_fact {
    ($type:ty, $name:expr) => {
        impl Fact for $type {
            fn fact_name() -> &'static str {
                $name
            }
        }
    };
}

/// Helper functions for working with fact objects
pub struct FactHelper;

impl FactHelper {
    /// Create a generic object with key-value pairs
    pub fn create_object(pairs: Vec<(&str, Value)>) -> Value {
        let mut object = HashMap::new();
        for (key, value) in pairs {
            object.insert(key.to_string(), value);
        }
        Value::Object(object)
    }

    /// Create a User fact from common fields
    pub fn create_user(name: &str, age: i64, email: &str, country: &str, is_vip: bool) -> Value {
        let mut user = HashMap::new();
        user.insert("Name".to_string(), Value::String(name.to_string()));
        user.insert("Age".to_string(), Value::Integer(age));
        user.insert("Email".to_string(), Value::String(email.to_string()));
        user.insert("Country".to_string(), Value::String(country.to_string()));
        user.insert("IsVIP".to_string(), Value::Boolean(is_vip));

        Value::Object(user)
    }

    /// Create a Product fact
    pub fn create_product(
        name: &str,
        price: f64,
        category: &str,
        in_stock: bool,
        stock_count: i64,
    ) -> Value {
        let mut product = HashMap::new();
        product.insert("Name".to_string(), Value::String(name.to_string()));
        product.insert("Price".to_string(), Value::Number(price));
        product.insert("Category".to_string(), Value::String(category.to_string()));
        product.insert("InStock".to_string(), Value::Boolean(in_stock));
        product.insert("StockCount".to_string(), Value::Integer(stock_count));

        Value::Object(product)
    }

    /// Create an Order fact
    pub fn create_order(
        id: &str,
        user_id: &str,
        total: f64,
        item_count: i64,
        status: &str,
    ) -> Value {
        let mut order = HashMap::new();
        order.insert("ID".to_string(), Value::String(id.to_string()));
        order.insert("UserID".to_string(), Value::String(user_id.to_string()));
        order.insert("Total".to_string(), Value::Number(total));
        order.insert("ItemCount".to_string(), Value::Integer(item_count));
        order.insert("Status".to_string(), Value::String(status.to_string()));

        Value::Object(order)
    }

    /// Create a TestCar object for method call demo
    pub fn create_test_car(
        speed_up: bool,
        speed: f64,
        max_speed: f64,
        speed_increment: f64,
    ) -> Value {
        let mut car = HashMap::new();
        car.insert("speedUp".to_string(), Value::Boolean(speed_up));
        car.insert("speed".to_string(), Value::Number(speed));
        car.insert("maxSpeed".to_string(), Value::Number(max_speed));
        car.insert("Speed".to_string(), Value::Number(speed));
        car.insert("SpeedIncrement".to_string(), Value::Number(speed_increment));
        car.insert(
            "_type".to_string(),
            Value::String("TestCarClass".to_string()),
        );

        Value::Object(car)
    }

    /// Create a DistanceRecord object for method call demo  
    pub fn create_distance_record(total_distance: f64) -> Value {
        let mut record = HashMap::new();
        record.insert("TotalDistance".to_string(), Value::Number(total_distance));
        record.insert(
            "_type".to_string(),
            Value::String("DistanceRecordClass".to_string()),
        );

        Value::Object(record)
    }

    /// Create a Transaction fact for fraud detection
    pub fn create_transaction(
        id: &str,
        amount: f64,
        location: &str,
        timestamp: i64,
        user_id: &str,
    ) -> Value {
        let mut transaction = HashMap::new();
        transaction.insert("ID".to_string(), Value::String(id.to_string()));
        transaction.insert("Amount".to_string(), Value::Number(amount));
        transaction.insert("Location".to_string(), Value::String(location.to_string()));
        transaction.insert("Timestamp".to_string(), Value::Integer(timestamp));
        transaction.insert("UserID".to_string(), Value::String(user_id.to_string()));

        Value::Object(transaction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facts_basic_operations() {
        let facts = Facts::new();

        // Add facts
        facts.add_value("age", Value::Integer(25)).unwrap();
        facts
            .add_value("name", Value::String("John".to_string()))
            .unwrap();

        // Get facts
        assert_eq!(facts.get("age"), Some(Value::Integer(25)));
        assert_eq!(facts.get("name"), Some(Value::String("John".to_string())));

        // Count
        assert_eq!(facts.count(), 2);

        // Contains
        assert!(facts.contains("age"));
        assert!(!facts.contains("email"));
    }

    #[test]
    fn test_nested_facts() {
        let facts = Facts::new();
        let user = FactHelper::create_user("John", 25, "john@example.com", "US", true);

        facts.add_value("User", user).unwrap();

        // Get nested values
        assert_eq!(facts.get_nested("User.Age"), Some(Value::Integer(25)));
        assert_eq!(
            facts.get_nested("User.Name"),
            Some(Value::String("John".to_string()))
        );

        // Set nested values
        facts.set_nested("User.Age", Value::Integer(26)).unwrap();
        assert_eq!(facts.get_nested("User.Age"), Some(Value::Integer(26)));
    }

    #[test]
    fn test_facts_snapshot() {
        let facts = Facts::new();
        facts
            .add_value("test", Value::String("value".to_string()))
            .unwrap();

        let snapshot = facts.snapshot();

        facts.clear();
        assert_eq!(facts.count(), 0);

        facts.restore(snapshot);
        assert_eq!(facts.count(), 1);
        assert_eq!(facts.get("test"), Some(Value::String("value".to_string())));
    }
}

//! State Management for Stream Processing
//!
//! Provides stateful operators with persistence, checkpointing, and recovery capabilities.
//! Essential for production streaming applications that need fault tolerance.
//!
//! ## Features
//!
//! - **Stateful Operators**: Maintain state across events (counters, aggregations, etc.)
//! - **Checkpointing**: Periodic state snapshots for fault tolerance
//! - **Recovery**: Restore state after failures
//! - **Multiple Backends**: Memory, File, and extensible to RocksDB/Redis
//! - **TTL Support**: Automatic state expiration
//!
//! ## Example
//!
//! ```rust,ignore
//! use rust_rule_engine::streaming::state::*;
//!
//! // Create state store
//! let mut state = StateStore::new(StateBackend::Memory);
//!
//! // Store and retrieve state
//! state.put("user_count", Value::Integer(42))?;
//! let count = state.get("user_count")?;
//!
//! // Checkpoint for fault tolerance
//! state.checkpoint("checkpoint_1")?;
//! ```

use crate::types::Value;
use crate::RuleEngineError;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(feature = "streaming-redis")]
use redis::{Commands, Client, Connection};

/// Result type for state operations
pub type StateResult<T> = Result<T, RuleEngineError>;

/// Backend type for state storage
#[derive(Debug, Clone, PartialEq)]
pub enum StateBackend {
    /// In-memory state (not persistent across restarts)
    Memory,
    /// File-based state (persistent)
    File { path: PathBuf },
    /// Redis backend (distributed, scalable)
    #[cfg(feature = "streaming-redis")]
    Redis {
        /// Redis connection URL (e.g., "redis://127.0.0.1:6379")
        url: String,
        /// Key prefix for namespacing
        key_prefix: String,
    },
    /// Custom backend (extensible)
    Custom { name: String },
}

/// Configuration for state management
#[derive(Debug, Clone)]
pub struct StateConfig {
    /// State backend type
    pub backend: StateBackend,
    /// Enable automatic checkpointing
    pub auto_checkpoint: bool,
    /// Checkpoint interval
    pub checkpoint_interval: Duration,
    /// Maximum checkpoint history to keep
    pub max_checkpoints: usize,
    /// Enable state TTL (time-to-live)
    pub enable_ttl: bool,
    /// Default TTL for state entries
    pub default_ttl: Duration,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            backend: StateBackend::Memory,
            auto_checkpoint: false,
            checkpoint_interval: Duration::from_secs(60),
            max_checkpoints: 10,
            enable_ttl: false,
            default_ttl: Duration::from_secs(3600),
        }
    }
}

/// Entry in the state store with metadata
#[derive(Debug, Clone)]
struct StateEntry {
    /// The actual value
    value: Value,
    /// When this entry was created (milliseconds since epoch)
    created_at: u64,
    /// When this entry was last updated
    updated_at: u64,
    /// TTL for this entry (if any)
    ttl: Option<Duration>,
}

impl StateEntry {
    fn new(value: Value, ttl: Option<Duration>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            value,
            created_at: now,
            updated_at: now,
            ttl,
        }
    }

    fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let ttl_ms = ttl.as_millis() as u64;
            now > self.created_at + ttl_ms
        } else {
            false
        }
    }

    fn update(&mut self, value: Value) {
        self.value = value;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }
}

/// Main state store for managing stateful operations
pub struct StateStore {
    /// Configuration
    config: StateConfig,
    /// Internal state storage
    state: Arc<RwLock<HashMap<String, StateEntry>>>,
    /// Checkpoint metadata
    checkpoints: Arc<RwLock<Vec<CheckpointMetadata>>>,
    /// Last checkpoint time
    last_checkpoint: Arc<RwLock<u64>>,
    /// Redis connection (if using Redis backend)
    #[cfg(feature = "streaming-redis")]
    redis_client: Option<Arc<RwLock<Client>>>,
}

impl StateStore {
    /// Create a new state store with default config
    pub fn new(backend: StateBackend) -> Self {
        let config = StateConfig {
            backend,
            ..Default::default()
        };
        Self::with_config(config)
    }

    /// Create a state store with custom configuration
    pub fn with_config(config: StateConfig) -> Self {
        #[cfg(feature = "streaming-redis")]
        let redis_client = if let StateBackend::Redis { url, .. } = &config.backend {
            Client::open(url.as_str())
                .ok()
                .map(|client| Arc::new(RwLock::new(client)))
        } else {
            None
        };

        Self {
            config,
            state: Arc::new(RwLock::new(HashMap::new())),
            checkpoints: Arc::new(RwLock::new(Vec::new())),
            last_checkpoint: Arc::new(RwLock::new(0)),
            #[cfg(feature = "streaming-redis")]
            redis_client,
        }
    }

    // Helper methods for Redis operations
    #[cfg(feature = "streaming-redis")]
    fn get_redis_key(&self, key: &str) -> String {
        if let StateBackend::Redis { key_prefix, .. } = &self.config.backend {
            format!("{}:{}", key_prefix, key)
        } else {
            key.to_string()
        }
    }

    #[cfg(feature = "streaming-redis")]
    fn redis_put(&self, key: &str, value: &Value, ttl: Option<Duration>) -> StateResult<()> {
        if let Some(client) = &self.redis_client {
            let client = client.read().unwrap();
            let mut conn = client.get_connection().map_err(|e| {
                RuleEngineError::ExecutionError(format!("Redis connection error: {}", e))
            })?;

            let redis_key = self.get_redis_key(key);
            let json = serde_json::to_string(value).map_err(|e| {
                RuleEngineError::ExecutionError(format!("Failed to serialize value: {}", e))
            })?;

            if let Some(ttl) = ttl {
                let ttl_secs = ttl.as_secs();
                conn.set_ex(&redis_key, json, ttl_secs).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Redis SET error: {}", e))
                })?;
            } else {
                conn.set(&redis_key, json).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Redis SET error: {}", e))
                })?;
            }

            Ok(())
        } else {
            Err(RuleEngineError::ExecutionError(
                "Redis client not initialized".to_string(),
            ))
        }
    }

    #[cfg(feature = "streaming-redis")]
    fn redis_get(&self, key: &str) -> StateResult<Option<Value>> {
        if let Some(client) = &self.redis_client {
            let client = client.read().unwrap();
            let mut conn = client.get_connection().map_err(|e| {
                RuleEngineError::ExecutionError(format!("Redis connection error: {}", e))
            })?;

            let redis_key = self.get_redis_key(key);
            let result: Option<String> = conn.get(&redis_key).map_err(|e| {
                RuleEngineError::ExecutionError(format!("Redis GET error: {}", e))
            })?;

            if let Some(json) = result {
                let value: Value = serde_json::from_str(&json).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Failed to deserialize value: {}", e))
                })?;
                Ok(Some(value))
            } else {
                Ok(None)
            }
        } else {
            Err(RuleEngineError::ExecutionError(
                "Redis client not initialized".to_string(),
            ))
        }
    }

    #[cfg(feature = "streaming-redis")]
    fn redis_delete(&self, key: &str) -> StateResult<()> {
        if let Some(client) = &self.redis_client {
            let client = client.read().unwrap();
            let mut conn = client.get_connection().map_err(|e| {
                RuleEngineError::ExecutionError(format!("Redis connection error: {}", e))
            })?;

            let redis_key = self.get_redis_key(key);
            conn.del(&redis_key).map_err(|e| {
                RuleEngineError::ExecutionError(format!("Redis DEL error: {}", e))
            })?;

            Ok(())
        } else {
            Err(RuleEngineError::ExecutionError(
                "Redis client not initialized".to_string(),
            ))
        }
    }

    #[cfg(feature = "streaming-redis")]
    fn redis_keys(&self) -> StateResult<Vec<String>> {
        if let Some(client) = &self.redis_client {
            let client = client.read().unwrap();
            let mut conn = client.get_connection().map_err(|e| {
                RuleEngineError::ExecutionError(format!("Redis connection error: {}", e))
            })?;

            let pattern = self.get_redis_key("*");
            let keys: Vec<String> = conn.keys(&pattern).map_err(|e| {
                RuleEngineError::ExecutionError(format!("Redis KEYS error: {}", e))
            })?;

            // Remove prefix from keys
            if let StateBackend::Redis { key_prefix, .. } = &self.config.backend {
                let prefix_len = key_prefix.len() + 1; // +1 for ':'
                Ok(keys.iter()
                    .map(|k| k[prefix_len..].to_string())
                    .collect())
            } else {
                Ok(keys)
            }
        } else {
            Err(RuleEngineError::ExecutionError(
                "Redis client not initialized".to_string(),
            ))
        }
    }

    /// Put a value into state
    pub fn put(&mut self, key: impl Into<String>, value: Value) -> StateResult<()> {
        let key = key.into();
        let ttl = if self.config.enable_ttl {
            Some(self.config.default_ttl)
        } else {
            None
        };

        #[cfg(feature = "streaming-redis")]
        if matches!(self.config.backend, StateBackend::Redis { .. }) {
            return self.redis_put(&key, &value, ttl);
        }

        let entry = StateEntry::new(value, ttl);
        let mut state = self.state.write().unwrap();
        state.insert(key, entry);

        Ok(())
    }

    /// Put a value with custom TTL
    pub fn put_with_ttl(
        &mut self,
        key: impl Into<String>,
        value: Value,
        ttl: Duration,
    ) -> StateResult<()> {
        let key = key.into();

        #[cfg(feature = "streaming-redis")]
        if matches!(self.config.backend, StateBackend::Redis { .. }) {
            return self.redis_put(&key, &value, Some(ttl));
        }

        let entry = StateEntry::new(value, Some(ttl));
        let mut state = self.state.write().unwrap();
        state.insert(key, entry);

        Ok(())
    }

    /// Get a value from state
    pub fn get(&self, key: &str) -> StateResult<Option<Value>> {
        #[cfg(feature = "streaming-redis")]
        if matches!(self.config.backend, StateBackend::Redis { .. }) {
            return self.redis_get(key);
        }

        let state = self.state.read().unwrap();

        if let Some(entry) = state.get(key) {
            if entry.is_expired() {
                Ok(None)
            } else {
                Ok(Some(entry.value.clone()))
            }
        } else {
            Ok(None)
        }
    }

    /// Update an existing value
    pub fn update(&mut self, key: &str, value: Value) -> StateResult<()> {
        #[cfg(feature = "streaming-redis")]
        if matches!(self.config.backend, StateBackend::Redis { .. }) {
            // For Redis, update is same as put (will overwrite with same TTL behavior)
            let ttl = if self.config.enable_ttl {
                Some(self.config.default_ttl)
            } else {
                None
            };
            return self.redis_put(key, &value, ttl);
        }

        let mut state = self.state.write().unwrap();

        if let Some(entry) = state.get_mut(key) {
            if entry.is_expired() {
                return Err(RuleEngineError::ExecutionError(
                    "State entry has expired".to_string(),
                ));
            }
            entry.update(value);
            Ok(())
        } else {
            Err(RuleEngineError::ExecutionError(format!(
                "State key '{}' not found",
                key
            )))
        }
    }

    /// Delete a value from state
    pub fn delete(&mut self, key: &str) -> StateResult<()> {
        #[cfg(feature = "streaming-redis")]
        if matches!(self.config.backend, StateBackend::Redis { .. }) {
            return self.redis_delete(key);
        }

        let mut state = self.state.write().unwrap();
        state.remove(key);
        Ok(())
    }

    /// Check if a key exists
    pub fn contains(&self, key: &str) -> bool {
        #[cfg(feature = "streaming-redis")]
        if matches!(self.config.backend, StateBackend::Redis { .. }) {
            return self.get(key).ok().flatten().is_some();
        }

        let state = self.state.read().unwrap();
        if let Some(entry) = state.get(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Get all keys in state
    pub fn keys(&self) -> Vec<String> {
        #[cfg(feature = "streaming-redis")]
        if matches!(self.config.backend, StateBackend::Redis { .. }) {
            return self.redis_keys().unwrap_or_else(|_| Vec::new());
        }

        let state = self.state.read().unwrap();
        state
            .iter()
            .filter(|(_, entry)| !entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Clear all state
    pub fn clear(&mut self) -> StateResult<()> {
        let mut state = self.state.write().unwrap();
        state.clear();
        Ok(())
    }

    /// Get the number of entries in state
    pub fn len(&self) -> usize {
        let state = self.state.read().unwrap();
        state.iter().filter(|(_, entry)| !entry.is_expired()).count()
    }

    /// Check if state is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&mut self) -> usize {
        let mut state = self.state.write().unwrap();
        let expired_keys: Vec<String> = state
            .iter()
            .filter(|(_, entry)| entry.is_expired())
            .map(|(key, _)| key.clone())
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            state.remove(&key);
        }

        count
    }

    /// Create a checkpoint of current state
    pub fn checkpoint(&mut self, name: impl Into<String>) -> StateResult<String> {
        let checkpoint_id = format!(
            "checkpoint_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let state = self.state.read().unwrap();
        let snapshot: HashMap<String, Value> = state
            .iter()
            .filter(|(_, entry)| !entry.is_expired())
            .map(|(key, entry)| (key.clone(), entry.value.clone()))
            .collect();

        match &self.config.backend {
            StateBackend::Memory => {
                // Store checkpoint metadata only
                let metadata = CheckpointMetadata {
                    id: checkpoint_id.clone(),
                    name: name.into(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    entry_count: snapshot.len(),
                    size_bytes: 0, // Not tracked for memory
                };

                let mut checkpoints = self.checkpoints.write().unwrap();
                checkpoints.push(metadata);

                // Keep only max_checkpoints
                if checkpoints.len() > self.config.max_checkpoints {
                    checkpoints.remove(0);
                }
            }
            StateBackend::File { path } => {
                // Serialize and save to file
                let checkpoint_path = path.join(&checkpoint_id);
                fs::create_dir_all(&checkpoint_path).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Failed to create checkpoint dir: {}", e))
                })?;

                let data_path = checkpoint_path.join("state.json");
                let json = serde_json::to_string_pretty(&snapshot).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Failed to serialize state: {}", e))
                })?;

                let mut file = fs::File::create(&data_path).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Failed to create checkpoint file: {}", e))
                })?;

                file.write_all(json.as_bytes()).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Failed to write checkpoint: {}", e))
                })?;

                let metadata = CheckpointMetadata {
                    id: checkpoint_id.clone(),
                    name: name.into(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    entry_count: snapshot.len(),
                    size_bytes: json.len(),
                };

                let mut checkpoints = self.checkpoints.write().unwrap();
                checkpoints.push(metadata);

                // Clean old checkpoints
                if checkpoints.len() > self.config.max_checkpoints {
                    let old_checkpoint = checkpoints.remove(0);
                    let old_path = path.join(&old_checkpoint.id);
                    let _ = fs::remove_dir_all(old_path);
                }
            }
            #[cfg(feature = "streaming-redis")]
            StateBackend::Redis { .. } => {
                // For Redis, checkpointing is handled by Redis persistence (RDB/AOF)
                // We just store metadata
                let metadata = CheckpointMetadata {
                    id: checkpoint_id.clone(),
                    name: name.into(),
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    entry_count: snapshot.len(),
                    size_bytes: 0,
                };

                let mut checkpoints = self.checkpoints.write().unwrap();
                checkpoints.push(metadata);

                if checkpoints.len() > self.config.max_checkpoints {
                    checkpoints.remove(0);
                }
            }
            StateBackend::Custom { .. } => {
                return Err(RuleEngineError::ExecutionError(
                    "Custom backend checkpointing not implemented".to_string(),
                ));
            }
        }

        let mut last = self.last_checkpoint.write().unwrap();
        *last = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Ok(checkpoint_id)
    }

    /// Restore state from a checkpoint
    pub fn restore(&mut self, checkpoint_id: &str) -> StateResult<()> {
        match &self.config.backend {
            StateBackend::Memory => {
                Err(RuleEngineError::ExecutionError(
                    "Cannot restore from memory backend (checkpoints not persisted)".to_string(),
                ))
            }
            StateBackend::File { path } => {
                let checkpoint_path = path.join(checkpoint_id);
                let data_path = checkpoint_path.join("state.json");

                if !data_path.exists() {
                    return Err(RuleEngineError::ExecutionError(format!(
                        "Checkpoint '{}' not found",
                        checkpoint_id
                    )));
                }

                let mut file = fs::File::open(&data_path).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Failed to open checkpoint file: {}", e))
                })?;

                let mut json = String::new();
                file.read_to_string(&mut json).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Failed to read checkpoint: {}", e))
                })?;

                let snapshot: HashMap<String, Value> = serde_json::from_str(&json).map_err(|e| {
                    RuleEngineError::ExecutionError(format!("Failed to deserialize checkpoint: {}", e))
                })?;

                // Clear current state and restore
                let mut state = self.state.write().unwrap();
                state.clear();

                for (key, value) in snapshot {
                    let entry = StateEntry::new(value, None);
                    state.insert(key, entry);
                }

                Ok(())
            }
            #[cfg(feature = "streaming-redis")]
            StateBackend::Redis { .. } => {
                // Redis persistence is automatic (RDB/AOF)
                // State is already in Redis, no restore needed
                Ok(())
            }
            StateBackend::Custom { .. } => {
                Err(RuleEngineError::ExecutionError(
                    "Custom backend restore not implemented".to_string(),
                ))
            }
        }
    }

    /// List all checkpoints
    pub fn list_checkpoints(&self) -> Vec<CheckpointMetadata> {
        let checkpoints = self.checkpoints.read().unwrap();
        checkpoints.clone()
    }

    /// Get the latest checkpoint
    pub fn latest_checkpoint(&self) -> Option<CheckpointMetadata> {
        let checkpoints = self.checkpoints.read().unwrap();
        checkpoints.last().cloned()
    }

    /// Get state statistics
    pub fn statistics(&self) -> StateStatistics {
        let state = self.state.read().unwrap();
        let total_entries = state.len();
        let expired_entries = state.iter().filter(|(_, e)| e.is_expired()).count();
        let active_entries = total_entries - expired_entries;

        let checkpoints = self.checkpoints.read().unwrap();
        let last_checkpoint = self.last_checkpoint.read().unwrap();

        StateStatistics {
            total_entries,
            active_entries,
            expired_entries,
            checkpoint_count: checkpoints.len(),
            last_checkpoint_time: *last_checkpoint,
        }
    }
}

/// Metadata about a checkpoint
#[derive(Debug, Clone)]
pub struct CheckpointMetadata {
    /// Unique checkpoint ID
    pub id: String,
    /// User-provided name
    pub name: String,
    /// Timestamp when checkpoint was created
    pub timestamp: u64,
    /// Number of entries in checkpoint
    pub entry_count: usize,
    /// Size in bytes (for file-based checkpoints)
    pub size_bytes: usize,
}

/// Statistics about state store
#[derive(Debug, Clone)]
pub struct StateStatistics {
    /// Total number of entries (including expired)
    pub total_entries: usize,
    /// Number of active (non-expired) entries
    pub active_entries: usize,
    /// Number of expired entries
    pub expired_entries: usize,
    /// Number of checkpoints
    pub checkpoint_count: usize,
    /// Time of last checkpoint
    pub last_checkpoint_time: u64,
}

/// Stateful operator that maintains state across events
pub struct StatefulOperator<F>
where
    F: Fn(&mut StateStore, &crate::streaming::event::StreamEvent) -> StateResult<Option<Value>>,
{
    /// State store
    state: StateStore,
    /// Processing function
    process_fn: F,
}

impl<F> StatefulOperator<F>
where
    F: Fn(&mut StateStore, &crate::streaming::event::StreamEvent) -> StateResult<Option<Value>>,
{
    /// Create a new stateful operator
    pub fn new(state: StateStore, process_fn: F) -> Self {
        Self { state, process_fn }
    }

    /// Process an event through the stateful operator
    pub fn process(
        &mut self,
        event: &crate::streaming::event::StreamEvent,
    ) -> StateResult<Option<Value>> {
        (self.process_fn)(&mut self.state, event)
    }

    /// Get reference to state store
    pub fn state(&self) -> &StateStore {
        &self.state
    }

    /// Get mutable reference to state store
    pub fn state_mut(&mut self) -> &mut StateStore {
        &mut self.state
    }

    /// Create a checkpoint
    pub fn checkpoint(&mut self, name: impl Into<String>) -> StateResult<String> {
        self.state.checkpoint(name)
    }

    /// Restore from checkpoint
    pub fn restore(&mut self, checkpoint_id: &str) -> StateResult<()> {
        self.state.restore(checkpoint_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::event::StreamEvent;
    use std::collections::HashMap;

    #[test]
    fn test_state_store_basic_operations() {
        let mut store = StateStore::new(StateBackend::Memory);

        // Put and get
        store.put("counter", Value::Integer(42)).unwrap();
        let value = store.get("counter").unwrap();
        assert_eq!(value, Some(Value::Integer(42)));

        // Update
        store.update("counter", Value::Integer(100)).unwrap();
        let value = store.get("counter").unwrap();
        assert_eq!(value, Some(Value::Integer(100)));

        // Contains
        assert!(store.contains("counter"));
        assert!(!store.contains("missing"));

        // Delete
        store.delete("counter").unwrap();
        assert!(!store.contains("counter"));
    }

    #[test]
    fn test_state_ttl() {
        let mut config = StateConfig::default();
        config.enable_ttl = true;
        config.default_ttl = Duration::from_millis(100);

        let mut store = StateStore::with_config(config);

        store.put("temp", Value::String("expires".to_string())).unwrap();
        assert!(store.contains("temp"));

        // Wait for TTL
        std::thread::sleep(Duration::from_millis(150));

        // Should be expired now
        assert!(!store.contains("temp"));
        let value = store.get("temp").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_checkpoint_memory() {
        let mut store = StateStore::new(StateBackend::Memory);

        store.put("key1", Value::Integer(1)).unwrap();
        store.put("key2", Value::Integer(2)).unwrap();

        let checkpoint_id = store.checkpoint("test_checkpoint").unwrap();
        assert!(!checkpoint_id.is_empty());

        let checkpoints = store.list_checkpoints();
        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].entry_count, 2);
    }

    #[test]
    fn test_stateful_operator() {
        let store = StateStore::new(StateBackend::Memory);

        // Counter operator: increments counter for each event
        let mut operator = StatefulOperator::new(store, |state, event| {
            let key = format!("counter_{}", event.event_type);
            let current = state.get(&key)?.unwrap_or(Value::Integer(0));

            if let Value::Integer(count) = current {
                let new_count = count + 1;
                state.put(&key, Value::Integer(new_count))?;
                Ok(Some(Value::Integer(new_count)))
            } else {
                Ok(None)
            }
        });

        // Process events
        let mut data = HashMap::new();
        data.insert("test".to_string(), Value::String("data".to_string()));

        for _ in 0..5 {
            let event = StreamEvent::new("TestEvent", data.clone(), "test");
            operator.process(&event).unwrap();
        }

        // Check counter
        let count = operator.state().get("counter_TestEvent").unwrap();
        assert_eq!(count, Some(Value::Integer(5)));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut config = StateConfig::default();
        config.enable_ttl = true;
        config.default_ttl = Duration::from_millis(50);

        let mut store = StateStore::with_config(config);

        store.put("key1", Value::Integer(1)).unwrap();
        store.put("key2", Value::Integer(2)).unwrap();
        store.put("key3", Value::Integer(3)).unwrap();

        assert_eq!(store.len(), 3);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(100));

        // Cleanup
        let expired = store.cleanup_expired();
        assert_eq!(expired, 3);
        assert_eq!(store.len(), 0);
    }
}

use crate::engine::RustRuleEngine;
use crate::errors::Result;
use std::collections::HashMap;
use std::sync::Arc;

/// Plugin state tracking
#[derive(Debug, Clone, PartialEq)]
pub enum PluginState {
    Loading,
    Loaded,
    Unloaded,
    Error,
}

/// Plugin health status
#[derive(Debug, Clone, PartialEq)]
pub enum PluginHealth {
    Healthy,
    Warning(String),
    Error(String),
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub state: PluginState,
    pub health: PluginHealth,
    pub actions: Vec<String>,
    pub functions: Vec<String>,
    pub dependencies: Vec<String>,
}

/// Plugin information for external queries
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub state: PluginState,
    pub health: PluginHealth,
}

/// Core trait that all plugins must implement
pub trait RulePlugin: Send + Sync {
    /// Get plugin metadata
    fn get_metadata(&self) -> &PluginMetadata;

    /// Register custom actions with the engine
    fn register_actions(&self, engine: &mut RustRuleEngine) -> Result<()>;

    /// Register custom functions with the engine
    fn register_functions(&self, engine: &mut RustRuleEngine) -> Result<()> {
        // Default: no functions to register
        Ok(())
    }

    /// Called when plugin is unloaded
    fn unload(&mut self) -> Result<()> {
        Ok(())
    }

    /// Health check for the plugin
    fn health_check(&mut self) -> PluginHealth {
        PluginHealth::Healthy
    }
}

/// Plugin statistics
#[derive(Debug, Clone)]
pub struct PluginStats {
    pub total_plugins: usize,
    pub loaded_plugins: usize,
    pub failed_plugins: usize,
    pub warnings: usize,
}

/// Plugin configuration
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub max_plugins: usize,
    pub enable_hot_reload: bool,
    pub plugin_timeout_ms: u64,
    pub safety_checks: bool,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            max_plugins: 50,
            enable_hot_reload: true,
            plugin_timeout_ms: 5000,
            safety_checks: true,
        }
    }
}

/// Main plugin manager
pub struct PluginManager {
    plugins: HashMap<String, Arc<dyn RulePlugin>>,
    config: PluginConfig,
    load_order: Vec<String>,
}

impl PluginManager {
    pub fn new(config: PluginConfig) -> Self {
        Self {
            plugins: HashMap::new(),
            config,
            load_order: Vec::new(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(PluginConfig::default())
    }

    /// Load a plugin into the manager
    pub fn load_plugin(&mut self, plugin: Arc<dyn RulePlugin>) -> Result<()> {
        let metadata = plugin.get_metadata();
        let name = metadata.name.clone();

        // Check if already loaded
        if self.plugins.contains_key(&name) {
            return Err(crate::errors::RuleEngineError::PluginError {
                message: format!("Plugin '{}' is already loaded", name),
            });
        }

        // Check plugin limit
        if self.plugins.len() >= self.config.max_plugins {
            return Err(crate::errors::RuleEngineError::PluginError {
                message: format!("Maximum plugin limit ({}) reached", self.config.max_plugins),
            });
        }

        // Check dependencies
        if self.config.safety_checks {
            self.validate_dependencies(&metadata.dependencies)?;
        }

        // Store plugin
        self.plugins.insert(name.clone(), plugin);
        self.load_order.push(name.clone());

        Ok(())
    }

    /// Unload a plugin
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        let plugin = self.plugins.get_mut(name).ok_or_else(|| {
            crate::errors::RuleEngineError::PluginError {
                message: format!("Plugin '{}' not found", name),
            }
        })?;

        // Note: We need a mutable reference to call unload
        // This is a design limitation - we'll work around it
        self.plugins.remove(name);
        self.load_order.retain(|n| n != name);

        Ok(())
    }

    /// Hot reload a plugin
    pub fn hot_reload_plugin(&mut self, name: &str, new_plugin: Arc<dyn RulePlugin>) -> Result<()> {
        // Remove old plugin
        self.unload_plugin(name)?;

        // Load new plugin (registration happens in engine)
        self.load_plugin(new_plugin)?;

        Ok(())
    }

    /// Get plugin metadata
    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginMetadata> {
        self.plugins.get(name).map(|p| p.get_metadata())
    }

    /// List all loaded plugins  
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins
            .values()
            .map(|plugin| {
                let metadata = plugin.get_metadata();
                PluginInfo {
                    name: metadata.name.clone(),
                    version: metadata.version.clone(),
                    description: metadata.description.clone(),
                    state: metadata.state.clone(),
                    health: metadata.health.clone(),
                }
            })
            .collect()
    }

    /// Health check all plugins
    pub fn plugin_health_check(&mut self) -> HashMap<String, PluginHealth> {
        let mut results = HashMap::new();

        // For this demo, we'll return the current health from metadata
        for plugin in self.plugins.values() {
            let metadata = plugin.get_metadata();
            results.insert(metadata.name.clone(), metadata.health.clone());
        }

        results
    }

    /// Validate plugin dependencies
    fn validate_dependencies(&self, dependencies: &[String]) -> Result<()> {
        for dep in dependencies {
            if !self.plugins.contains_key(dep) {
                return Err(crate::errors::RuleEngineError::PluginError {
                    message: format!("Dependency '{}' is not loaded", dep),
                });
            }
        }
        Ok(())
    }

    /// Get plugin statistics
    pub fn get_stats(&self) -> PluginStats {
        let mut loaded_count = 0;
        let mut failed_count = 0;
        let mut warning_count = 0;

        for plugin in self.plugins.values() {
            let metadata = plugin.get_metadata();
            match metadata.health {
                PluginHealth::Healthy => loaded_count += 1,
                PluginHealth::Warning(_) => warning_count += 1,
                PluginHealth::Error(_) => failed_count += 1,
            }
        }

        PluginStats {
            total_plugins: self.plugins.len(),
            loaded_plugins: loaded_count,
            failed_plugins: failed_count,
            warnings: warning_count,
        }
    }
}

impl std::fmt::Display for PluginStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Plugins: {} total (✅ {} loaded, ⚠️ {} warnings, ❌ {} failed)",
            self.total_plugins, self.loaded_plugins, self.warnings, self.failed_plugins
        )
    }
}

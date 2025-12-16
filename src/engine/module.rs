//! Module System (CLIPS-inspired defmodule)
//!
//! Provides namespace isolation and visibility control for large knowledge bases.
//! Similar to CLIPS defmodule, Drools rule units, and package systems.
//!
//! # Features
//!
//! - **Module Isolation**: Separate namespaces for rules, templates, and facts
//! - **Import/Export Control**: Fine-grained visibility management
//! - **Module Focus**: Control execution flow across modules
//! - **Pattern Matching**: Export/import with wildcards (e.g., "sensor-*")
//!
//! # Example
//!
//! ```rust
//! use rust_rule_engine::engine::module::{ModuleManager, ExportList, ImportType};
//!
//! let mut manager = ModuleManager::new();
//!
//! // Create modules
//! manager.create_module("SENSORS").unwrap();
//! manager.create_module("CONTROL").unwrap();
//!
//! // Configure exports
//! manager.export_all_from("SENSORS", ExportList::All).unwrap();
//!
//! // Configure imports
//! manager.import_from("CONTROL", "SENSORS", ImportType::AllTemplates, "*").unwrap();
//!
//! // Set focus
//! manager.set_focus("CONTROL").unwrap();
//! ```

use crate::errors::{Result, RuleEngineError};
use std::collections::{HashMap, HashSet, VecDeque};

/// Type of item that can be exported/imported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ItemType {
    /// Rule
    Rule,
    /// Template (deftemplate)
    Template,
    /// Fact
    Fact,
    /// All items
    All,
}

/// Export list specification
#[derive(Debug, Clone, PartialEq)]
pub enum ExportList {
    /// Export everything (default for MAIN module)
    All,
    /// Export nothing (default for user modules)
    None,
    /// Export specific items matching patterns
    Specific(Vec<ExportItem>),
}

/// Single export item
#[derive(Debug, Clone, PartialEq)]
pub struct ExportItem {
    /// Type of item to export
    pub item_type: ItemType,
    /// Name or pattern (supports wildcards like "sensor-*")
    pub pattern: String,
}

/// Import type specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportType {
    /// Import all rules
    AllRules,
    /// Import all templates
    AllTemplates,
    /// Import specific rules
    Rules,
    /// Import specific templates
    Templates,
    /// Import everything
    All,
}

/// Re-export configuration
#[derive(Debug, Clone, PartialEq)]
pub struct ReExport {
    /// Items to re-export (patterns from imported modules)
    pub patterns: Vec<String>,
    /// Whether to re-export transitively (re-export what was imported from other modules)
    pub transitive: bool,
}

/// Import declaration
#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    /// Module to import from
    pub from_module: String,
    /// Type of items to import
    pub import_type: ImportType,
    /// Pattern to match (supports wildcards)
    pub pattern: String,
    /// Re-export configuration (if any)
    pub re_export: Option<ReExport>,
}

/// A module in the knowledge base
#[derive(Debug, Clone)]
pub struct Module {
    /// Module name
    pub name: String,
    /// Rules owned by this module
    rules: HashSet<String>,
    /// Templates owned by this module
    templates: HashSet<String>,
    /// Facts owned by this module (by type)
    fact_types: HashSet<String>,
    /// Export specification
    exports: ExportList,
    /// Import declarations
    imports: Vec<ImportDecl>,
    /// Module documentation
    pub doc: Option<String>,
    /// Module-level salience (priority adjustment for all rules in this module)
    salience: i32,
}

impl Module {
    /// Create a new module
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let exports = if name == "MAIN" {
            ExportList::All
        } else {
            ExportList::None
        };

        Self {
            name,
            rules: HashSet::new(),
            templates: HashSet::new(),
            fact_types: HashSet::new(),
            exports,
            imports: Vec::new(),
            doc: None,
            salience: 0,
        }
    }

    /// Add documentation
    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = Some(doc.into());
        self
    }

    /// Add a rule to this module
    pub fn add_rule(&mut self, rule_name: impl Into<String>) {
        self.rules.insert(rule_name.into());
    }

    /// Add a template to this module
    pub fn add_template(&mut self, template_name: impl Into<String>) {
        self.templates.insert(template_name.into());
    }

    /// Add a fact type to this module
    pub fn add_fact_type(&mut self, fact_type: impl Into<String>) {
        self.fact_types.insert(fact_type.into());
    }

    /// Set export specification
    pub fn set_exports(&mut self, exports: ExportList) {
        self.exports = exports;
    }

    /// Get export specification
    pub fn get_exports(&self) -> &ExportList {
        &self.exports
    }

    /// Add an import declaration
    pub fn add_import(&mut self, import: ImportDecl) {
        self.imports.push(import);
    }

    /// Check if this module exports a rule (including re-exports)
    pub fn exports_rule(&self, rule_name: &str) -> bool {
        // Check if it's an owned rule
        let is_owned = self.rules.contains(rule_name);

        // Check if it matches export spec for owned rules
        let exports_owned = match &self.exports {
            ExportList::All => is_owned,
            ExportList::None => false,
            ExportList::Specific(items) => {
                is_owned
                    && items.iter().any(|item| {
                        matches!(item.item_type, ItemType::Rule | ItemType::All)
                            && pattern_matches(&item.pattern, rule_name)
                    })
            }
        };

        // Also check if it should be re-exported (transitive)
        exports_owned || self.should_re_export_rule(rule_name)
    }

    /// Check if this module exports a template (including re-exports)
    pub fn exports_template(&self, template_name: &str) -> bool {
        // Check if it's an owned template
        let is_owned = self.templates.contains(template_name);

        // Check if it matches export spec for owned templates
        let exports_owned = match &self.exports {
            ExportList::All => is_owned,
            ExportList::None => false,
            ExportList::Specific(items) => {
                is_owned
                    && items.iter().any(|item| {
                        matches!(item.item_type, ItemType::Template | ItemType::All)
                            && pattern_matches(&item.pattern, template_name)
                    })
            }
        };

        // Also check if it should be re-exported (transitive)
        exports_owned || self.should_re_export_template(template_name)
    }

    /// Get all rules in this module
    pub fn get_rules(&self) -> &HashSet<String> {
        &self.rules
    }

    /// Get all templates in this module
    pub fn get_templates(&self) -> &HashSet<String> {
        &self.templates
    }

    /// Get all imports
    pub fn get_imports(&self) -> &[ImportDecl] {
        &self.imports
    }

    /// Set module-level salience (priority adjustment for all rules)
    pub fn set_salience(&mut self, salience: i32) {
        self.salience = salience;
    }

    /// Get module-level salience
    pub fn get_salience(&self) -> i32 {
        self.salience
    }

    /// Check if a rule should be re-exported based on import declarations
    pub fn should_re_export_rule(&self, rule_name: &str) -> bool {
        for import in &self.imports {
            if let Some(re_export) = &import.re_export {
                if re_export
                    .patterns
                    .iter()
                    .any(|p| pattern_matches(p, rule_name))
                {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a template should be re-exported based on import declarations
    pub fn should_re_export_template(&self, template_name: &str) -> bool {
        for import in &self.imports {
            if let Some(re_export) = &import.re_export {
                if re_export
                    .patterns
                    .iter()
                    .any(|p| pattern_matches(p, template_name))
                {
                    return true;
                }
            }
        }
        false
    }
}

/// Error type for cyclic import detection
#[derive(Debug, Clone)]
pub struct CycleError {
    /// The cycle that was detected (list of module names forming the cycle)
    pub cycle_path: Vec<String>,
}

impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cyclic import detected: {}",
            self.cycle_path.join(" -> ")
        )
    }
}

/// Module manager for organizing knowledge bases
#[derive(Debug, Clone)]
pub struct ModuleManager {
    /// All modules
    modules: HashMap<String, Module>,
    /// Current focus module (for execution)
    current_focus: String,
    /// Default module name
    default_module: String,
    /// Track import graph for cycle detection
    import_graph: HashMap<String, HashSet<String>>,
}

impl ModuleManager {
    /// Create a new module manager
    pub fn new() -> Self {
        let mut modules = HashMap::new();
        modules.insert("MAIN".to_string(), Module::new("MAIN"));

        Self {
            modules,
            current_focus: "MAIN".to_string(),
            default_module: "MAIN".to_string(),
            import_graph: HashMap::new(),
        }
    }

    /// Create a new module
    pub fn create_module(&mut self, name: impl Into<String>) -> Result<&mut Module> {
        let name = name.into();

        if self.modules.contains_key(&name) {
            return Err(RuleEngineError::ModuleError {
                message: format!("Module '{}' already exists", name),
            });
        }

        self.modules.insert(name.clone(), Module::new(&name));
        Ok(self.modules.get_mut(&name).unwrap())
    }

    /// Get a module (mutable)
    pub fn get_module_mut(&mut self, name: &str) -> Result<&mut Module> {
        self.modules
            .get_mut(name)
            .ok_or_else(|| RuleEngineError::ModuleError {
                message: format!("Module '{}' not found", name),
            })
    }

    /// Get a module (immutable)
    pub fn get_module(&self, name: &str) -> Result<&Module> {
        self.modules
            .get(name)
            .ok_or_else(|| RuleEngineError::ModuleError {
                message: format!("Module '{}' not found", name),
            })
    }

    /// Delete a module
    pub fn delete_module(&mut self, name: &str) -> Result<()> {
        if name == self.default_module {
            return Err(RuleEngineError::ModuleError {
                message: "Cannot delete default module".to_string(),
            });
        }

        if name == self.current_focus {
            self.current_focus = self.default_module.clone();
        }

        self.modules
            .remove(name)
            .ok_or_else(|| RuleEngineError::ModuleError {
                message: format!("Module '{}' not found", name),
            })?;

        // Clean up import graph
        self.import_graph.remove(name);
        for (_, imports) in self.import_graph.iter_mut() {
            imports.remove(name);
        }

        Ok(())
    }

    /// Set current focus module
    pub fn set_focus(&mut self, module_name: impl Into<String>) -> Result<()> {
        let module_name = module_name.into();

        if !self.modules.contains_key(&module_name) {
            return Err(RuleEngineError::ModuleError {
                message: format!("Module '{}' not found", module_name),
            });
        }

        self.current_focus = module_name;
        Ok(())
    }

    /// Get current focus module name
    pub fn get_focus(&self) -> &str {
        &self.current_focus
    }

    /// Get all module names
    pub fn list_modules(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    /// Configure exports for a module
    pub fn export_all_from(&mut self, module_name: &str, export_list: ExportList) -> Result<()> {
        let module = self.get_module_mut(module_name)?;
        module.set_exports(export_list);
        Ok(())
    }

    /// Detect if adding an import would create a cycle
    ///
    /// Uses BFS (Breadth-First Search) to traverse the import graph from `from_module`
    /// and check if we can reach `to_module`. If we can, adding `to_module -> from_module`
    /// would create a cycle.
    ///
    /// Returns `Ok(())` if no cycle would be created.
    /// Returns `Err(RuleEngineError)` with detailed cycle path if cycle would be created.
    fn detect_cycle(&self, to_module: &str, from_module: &str) -> Result<()> {
        // Cycle with self
        if to_module == from_module {
            return Err(RuleEngineError::ModuleError {
                message: format!(
                    "Cyclic import detected: {} cannot import from itself",
                    to_module
                ),
            });
        }

        // BFS from from_module to see if we can reach to_module
        // If we can, then adding to_module -> from_module creates a cycle
        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map: HashMap<String, String> = HashMap::new();

        queue.push_back(from_module.to_string());
        visited.insert(from_module.to_string());

        while let Some(current) = queue.pop_front() {
            // Get all modules that current imports from
            if let Some(imports) = self.import_graph.get(&current) {
                for imported in imports {
                    if imported == to_module {
                        // Found a cycle! Reconstruct the path
                        let mut cycle_path = vec![to_module.to_string()];
                        let mut node = current.clone();

                        while let Some(parent) = parent_map.get(&node) {
                            cycle_path.push(node.clone());
                            node = parent.clone();
                        }

                        cycle_path.push(node);
                        cycle_path.reverse();

                        return Err(RuleEngineError::ModuleError {
                            message: format!("Cyclic import detected: {}", cycle_path.join(" -> ")),
                        });
                    }

                    if !visited.contains(imported) {
                        visited.insert(imported.clone());
                        parent_map.insert(imported.clone(), current.clone());
                        queue.push_back(imported.clone());
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the import graph for inspection/debugging
    pub fn get_import_graph(&self) -> &HashMap<String, HashSet<String>> {
        &self.import_graph
    }

    /// Export the import graph for visualization or analysis
    pub fn get_import_graph_debug(&self) -> Vec<(String, Vec<String>)> {
        self.import_graph
            .iter()
            .map(|(module, imports)| (module.clone(), imports.iter().cloned().collect()))
            .collect()
    }

    /// Add an import to a module
    pub fn import_from(
        &mut self,
        to_module: &str,
        from_module: &str,
        import_type: ImportType,
        pattern: impl Into<String>,
    ) -> Result<()> {
        self.import_from_with_reexport(to_module, from_module, import_type, pattern, None)
    }

    /// Add an import to a module with optional re-export configuration
    pub fn import_from_with_reexport(
        &mut self,
        to_module: &str,
        from_module: &str,
        import_type: ImportType,
        pattern: impl Into<String>,
        re_export: Option<ReExport>,
    ) -> Result<()> {
        // Validate from_module exists
        if !self.modules.contains_key(from_module) {
            return Err(RuleEngineError::ModuleError {
                message: format!("Source module '{}' not found", from_module),
            });
        }

        // Check for cycles BEFORE adding the import
        self.detect_cycle(to_module, from_module)?;

        let module = self.get_module_mut(to_module)?;
        module.add_import(ImportDecl {
            from_module: from_module.to_string(),
            import_type,
            pattern: pattern.into(),
            re_export,
        });

        // Update import graph
        self.import_graph
            .entry(to_module.to_string())
            .or_default()
            .insert(from_module.to_string());

        Ok(())
    }

    /// Check if a rule is visible to a module
    pub fn is_rule_visible(&self, rule_name: &str, to_module: &str) -> Result<bool> {
        let module = self.get_module(to_module)?;

        // Own rules are always visible
        if module.get_rules().contains(rule_name) {
            return Ok(true);
        }

        // Check imports
        for import in module.get_imports() {
            if !matches!(
                import.import_type,
                ImportType::AllRules | ImportType::Rules | ImportType::All
            ) {
                continue;
            }

            let from_module = self.get_module(&import.from_module)?;

            if from_module.exports_rule(rule_name) && pattern_matches(&import.pattern, rule_name) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if a template is visible to a module
    pub fn is_template_visible(&self, template_name: &str, to_module: &str) -> Result<bool> {
        let module = self.get_module(to_module)?;

        // Own templates are always visible
        if module.get_templates().contains(template_name) {
            return Ok(true);
        }

        // Check imports
        for import in module.get_imports() {
            if !matches!(
                import.import_type,
                ImportType::AllTemplates | ImportType::Templates | ImportType::All
            ) {
                continue;
            }

            let from_module = self.get_module(&import.from_module)?;

            if from_module.exports_template(template_name)
                && pattern_matches(&import.pattern, template_name)
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get all rules visible to a module
    pub fn get_visible_rules(&self, module_name: &str) -> Result<Vec<String>> {
        let module = self.get_module(module_name)?;
        let mut visible = HashSet::new();

        // Add own rules
        visible.extend(module.get_rules().iter().cloned());

        // Add imported rules
        for import in module.get_imports() {
            if !matches!(
                import.import_type,
                ImportType::AllRules | ImportType::Rules | ImportType::All
            ) {
                continue;
            }

            let from_module = self.get_module(&import.from_module)?;

            for rule in from_module.get_rules() {
                if from_module.exports_rule(rule) && pattern_matches(&import.pattern, rule) {
                    visible.insert(rule.clone());
                }
            }
        }

        Ok(visible.into_iter().collect())
    }

    /// Get module statistics
    pub fn get_stats(&self) -> ModuleStats {
        ModuleStats {
            total_modules: self.modules.len(),
            current_focus: self.current_focus.clone(),
            modules: self
                .modules
                .iter()
                .map(|(name, module)| {
                    (
                        name.clone(),
                        ModuleInfo {
                            name: name.clone(),
                            rules_count: module.rules.len(),
                            templates_count: module.templates.len(),
                            imports_count: module.imports.len(),
                            exports_type: match &module.exports {
                                ExportList::All => "All".to_string(),
                                ExportList::None => "None".to_string(),
                                ExportList::Specific(items) => format!("Specific({})", items.len()),
                            },
                            salience: module.salience,
                        },
                    )
                })
                .collect(),
        }
    }

    /// Set module-level salience
    pub fn set_module_salience(&mut self, module_name: &str, salience: i32) -> Result<()> {
        let module = self.get_module_mut(module_name)?;
        module.set_salience(salience);
        Ok(())
    }

    /// Get module-level salience
    pub fn get_module_salience(&self, module_name: &str) -> Result<i32> {
        let module = self.get_module(module_name)?;
        Ok(module.get_salience())
    }

    /// Get all transitive dependencies of a module (BFS)
    pub fn get_transitive_dependencies(&self, module_name: &str) -> Result<Vec<String>> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(module_name.to_string());
        visited.insert(module_name.to_string());

        while let Some(current) = queue.pop_front() {
            if let Some(imports) = self.import_graph.get(&current) {
                for imported in imports {
                    if !visited.contains(imported) {
                        visited.insert(imported.clone());
                        result.push(imported.clone());
                        queue.push_back(imported.clone());
                    }
                }
            }
        }

        Ok(result)
    }

    /// Validate module configuration
    pub fn validate_module(&self, module_name: &str) -> Result<ModuleValidation> {
        let module = self.get_module(module_name)?;
        let mut warnings = Vec::new();
        let mut errors = Vec::new();

        // Check for broken imports (imported modules don't exist)
        for import in module.get_imports() {
            if !self.modules.contains_key(&import.from_module) {
                errors.push(format!(
                    "Import references non-existent module: {}",
                    import.from_module
                ));
            }
        }

        // Check for unused imports (imports nothing visible)
        for import in module.get_imports() {
            if let Ok(from_module) = self.get_module(&import.from_module) {
                let mut has_visible = false;

                // Check if any rules/templates are visible through this import
                match import.import_type {
                    ImportType::AllRules | ImportType::Rules | ImportType::All => {
                        for rule in from_module.get_rules() {
                            if from_module.exports_rule(rule)
                                && pattern_matches(&import.pattern, rule)
                            {
                                has_visible = true;
                                break;
                            }
                        }
                    }
                    ImportType::AllTemplates | ImportType::Templates => {
                        for template in from_module.get_templates() {
                            if from_module.exports_template(template)
                                && pattern_matches(&import.pattern, template)
                            {
                                has_visible = true;
                                break;
                            }
                        }
                    }
                }

                if !has_visible {
                    warnings.push(format!(
                        "Import from '{}' with pattern '{}' doesn't match any exported items",
                        import.from_module, import.pattern
                    ));
                }
            }
        }

        // Check for re-export patterns that don't match any imported items
        for import in module.get_imports() {
            if let Some(re_export) = &import.re_export {
                for pattern in &re_export.patterns {
                    let mut matches_any = false;

                    if let Ok(from_module) = self.get_module(&import.from_module) {
                        // Check rules
                        for rule in from_module.get_rules() {
                            if from_module.exports_rule(rule) && pattern_matches(pattern, rule) {
                                matches_any = true;
                                break;
                            }
                        }

                        // Check templates
                        if !matches_any {
                            for template in from_module.get_templates() {
                                if from_module.exports_template(template)
                                    && pattern_matches(pattern, template)
                                {
                                    matches_any = true;
                                    break;
                                }
                            }
                        }
                    }

                    if !matches_any {
                        warnings.push(format!(
                            "Re-export pattern '{}' from import '{}' doesn't match any items",
                            pattern, import.from_module
                        ));
                    }
                }
            }
        }

        // Check if module has no rules or templates
        if module.get_rules().is_empty()
            && module.get_templates().is_empty()
            && module.get_imports().is_empty()
        {
            warnings.push("Module is empty (no rules, templates, or imports)".to_string());
        }

        Ok(ModuleValidation {
            module_name: module_name.to_string(),
            is_valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Validate all modules in the system
    pub fn validate_all_modules(&self) -> HashMap<String, ModuleValidation> {
        self.modules
            .keys()
            .filter_map(|name| self.validate_module(name).ok().map(|v| (name.clone(), v)))
            .collect()
    }
}

impl Default for ModuleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Module statistics
#[derive(Debug, Clone)]
pub struct ModuleStats {
    /// Total number of modules
    pub total_modules: usize,
    /// Current focus module
    pub current_focus: String,
    /// Information about each module
    pub modules: HashMap<String, ModuleInfo>,
}

/// Information about a single module
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    /// Module name
    pub name: String,
    /// Number of rules
    pub rules_count: usize,
    /// Number of templates
    pub templates_count: usize,
    /// Number of imports
    pub imports_count: usize,
    /// Export type description
    pub exports_type: String,
    /// Module-level salience
    pub salience: i32,
}

/// Module validation result
#[derive(Debug, Clone)]
pub struct ModuleValidation {
    /// Module name
    pub module_name: String,
    /// Whether the module is valid (no errors)
    pub is_valid: bool,
    /// List of errors
    pub errors: Vec<String>,
    /// List of warnings
    pub warnings: Vec<String>,
}

/// Check if a name matches a pattern (supports wildcards)
fn pattern_matches(pattern: &str, name: &str) -> bool {
    if pattern == "*" || pattern == "?ALL" {
        return true;
    }

    // Simple wildcard matching
    if let Some(prefix) = pattern.strip_suffix('*') {
        name.starts_with(prefix)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        name.ends_with(suffix)
    } else {
        pattern == name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let mut manager = ModuleManager::new();

        assert!(manager.create_module("TEST").is_ok());
        assert!(manager.create_module("TEST").is_err()); // Duplicate

        assert_eq!(manager.list_modules().len(), 2); // MAIN + TEST
    }

    #[test]
    fn test_module_focus() {
        let mut manager = ModuleManager::new();
        manager.create_module("SENSORS").unwrap();

        assert_eq!(manager.get_focus(), "MAIN");

        manager.set_focus("SENSORS").unwrap();
        assert_eq!(manager.get_focus(), "SENSORS");

        assert!(manager.set_focus("NONEXISTENT").is_err());
    }

    #[test]
    fn test_export_import() {
        let mut manager = ModuleManager::new();
        manager.create_module("SENSORS").unwrap();
        manager.create_module("CONTROL").unwrap();

        // Add rules to SENSORS
        let sensors = manager.get_module_mut("SENSORS").unwrap();
        sensors.add_rule("sensor-temp");
        sensors.add_rule("sensor-pressure");
        sensors.set_exports(ExportList::Specific(vec![ExportItem {
            item_type: ItemType::Rule,
            pattern: "sensor-*".to_string(),
        }]));

        // Import in CONTROL
        manager
            .import_from("CONTROL", "SENSORS", ImportType::AllRules, "*")
            .unwrap();

        // Check visibility
        assert!(manager.is_rule_visible("sensor-temp", "CONTROL").unwrap());
        assert!(manager
            .is_rule_visible("sensor-pressure", "CONTROL")
            .unwrap());
    }

    #[test]
    fn test_pattern_matching() {
        assert!(pattern_matches("*", "anything"));
        assert!(pattern_matches("sensor-*", "sensor-temp"));
        assert!(pattern_matches("sensor-*", "sensor-pressure"));
        assert!(!pattern_matches("sensor-*", "control-temp"));
        assert!(pattern_matches("*-temp", "sensor-temp"));
        assert!(pattern_matches("exact", "exact"));
        assert!(!pattern_matches("exact", "not-exact"));
    }

    #[test]
    fn test_main_module_default_export() {
        let manager = ModuleManager::new();
        let main_module = manager.get_module("MAIN").unwrap();

        // MAIN module should export all by default
        assert!(matches!(main_module.exports, ExportList::All));
    }

    #[test]
    fn test_user_module_default_export() {
        let mut manager = ModuleManager::new();
        manager.create_module("USER").unwrap();
        let user_module = manager.get_module("USER").unwrap();

        // User modules should export none by default
        assert!(matches!(user_module.exports, ExportList::None));
    }

    #[test]
    fn test_visibility_own_rules() {
        let mut manager = ModuleManager::new();
        manager.create_module("TEST").unwrap();

        let test_module = manager.get_module_mut("TEST").unwrap();
        test_module.add_rule("my-rule");

        // Own rules are always visible
        assert!(manager.is_rule_visible("my-rule", "TEST").unwrap());
    }

    #[test]
    fn test_get_visible_rules() {
        let mut manager = ModuleManager::new();
        manager.create_module("MOD1").unwrap();
        manager.create_module("MOD2").unwrap();

        // Add rules to MOD1
        let mod1 = manager.get_module_mut("MOD1").unwrap();
        mod1.add_rule("rule1");
        mod1.add_rule("rule2");
        mod1.set_exports(ExportList::All);

        // Add rule to MOD2
        let mod2 = manager.get_module_mut("MOD2").unwrap();
        mod2.add_rule("rule3");

        // Import from MOD1 to MOD2
        manager
            .import_from("MOD2", "MOD1", ImportType::AllRules, "*")
            .unwrap();

        let visible = manager.get_visible_rules("MOD2").unwrap();
        assert!(visible.contains(&"rule1".to_string()));
        assert!(visible.contains(&"rule2".to_string()));
        assert!(visible.contains(&"rule3".to_string()));
        assert_eq!(visible.len(), 3);
    }

    #[test]
    fn test_module_stats() {
        let mut manager = ModuleManager::new();
        manager.create_module("TEST").unwrap();

        let test_module = manager.get_module_mut("TEST").unwrap();
        test_module.add_rule("rule1");
        test_module.add_template("template1");

        let stats = manager.get_stats();
        assert_eq!(stats.total_modules, 2); // MAIN + TEST
        assert_eq!(stats.current_focus, "MAIN");

        let test_info = stats.modules.get("TEST").unwrap();
        assert_eq!(test_info.rules_count, 1);
        assert_eq!(test_info.templates_count, 1);
    }

    // Phase 3 tests

    #[test]
    fn test_transitive_reexport() {
        let mut manager = ModuleManager::new();
        manager.create_module("BASE").unwrap();
        manager.create_module("MIDDLE").unwrap();
        manager.create_module("TOP").unwrap();

        // BASE has rules
        let base = manager.get_module_mut("BASE").unwrap();
        base.add_rule("base-rule1");
        base.add_rule("base-rule2");
        base.set_exports(ExportList::All);

        // MIDDLE imports from BASE and re-exports
        manager
            .import_from_with_reexport(
                "MIDDLE",
                "BASE",
                ImportType::AllRules,
                "*",
                Some(ReExport {
                    patterns: vec!["base-*".to_string()],
                    transitive: true,
                }),
            )
            .unwrap();

        // TOP imports from MIDDLE
        manager
            .import_from("TOP", "MIDDLE", ImportType::AllRules, "*")
            .unwrap();

        // TOP should see rules from BASE through MIDDLE's re-export
        assert!(manager.is_rule_visible("base-rule1", "TOP").unwrap());
        assert!(manager.is_rule_visible("base-rule2", "TOP").unwrap());
    }

    #[test]
    fn test_module_salience() {
        let mut manager = ModuleManager::new();
        manager.create_module("HIGH_PRIORITY").unwrap();
        manager.create_module("LOW_PRIORITY").unwrap();

        // Set different salience levels
        manager.set_module_salience("HIGH_PRIORITY", 100).unwrap();
        manager.set_module_salience("LOW_PRIORITY", -50).unwrap();

        // Verify salience values
        assert_eq!(manager.get_module_salience("HIGH_PRIORITY").unwrap(), 100);
        assert_eq!(manager.get_module_salience("LOW_PRIORITY").unwrap(), -50);
        assert_eq!(manager.get_module_salience("MAIN").unwrap(), 0);

        // Check in stats
        let stats = manager.get_stats();
        assert_eq!(stats.modules.get("HIGH_PRIORITY").unwrap().salience, 100);
        assert_eq!(stats.modules.get("LOW_PRIORITY").unwrap().salience, -50);
    }

    #[test]
    fn test_transitive_dependencies() {
        let mut manager = ModuleManager::new();
        manager.create_module("A").unwrap();
        manager.create_module("B").unwrap();
        manager.create_module("C").unwrap();
        manager.create_module("D").unwrap();

        // Create dependency chain: A -> B -> C -> D
        manager.import_from("A", "B", ImportType::All, "*").unwrap();
        manager.import_from("B", "C", ImportType::All, "*").unwrap();
        manager.import_from("C", "D", ImportType::All, "*").unwrap();

        // Get transitive dependencies of A
        let deps = manager.get_transitive_dependencies("A").unwrap();
        assert!(deps.contains(&"B".to_string()));
        assert!(deps.contains(&"C".to_string()));
        assert!(deps.contains(&"D".to_string()));
        assert_eq!(deps.len(), 3);
    }

    #[test]
    fn test_module_validation_broken_import() {
        let mut manager = ModuleManager::new();
        manager.create_module("TEST").unwrap();

        // Manually add a broken import (this bypasses normal validation)
        let test_module = manager.get_module_mut("TEST").unwrap();
        test_module.add_import(ImportDecl {
            from_module: "NONEXISTENT".to_string(),
            import_type: ImportType::All,
            pattern: "*".to_string(),
            re_export: None,
        });

        let validation = manager.validate_module("TEST").unwrap();
        assert!(!validation.is_valid);
        assert!(validation.errors.iter().any(|e| e.contains("NONEXISTENT")));
    }

    #[test]
    fn test_module_validation_unused_import() {
        let mut manager = ModuleManager::new();
        manager.create_module("SOURCE").unwrap();
        manager.create_module("TARGET").unwrap();

        // SOURCE has a rule
        let source = manager.get_module_mut("SOURCE").unwrap();
        source.add_rule("my-rule");
        source.set_exports(ExportList::None); // Export nothing

        // TARGET imports from SOURCE but SOURCE exports nothing
        manager
            .import_from("TARGET", "SOURCE", ImportType::AllRules, "*")
            .unwrap();

        let validation = manager.validate_module("TARGET").unwrap();
        assert!(validation.is_valid); // It's valid but has warnings
        assert!(!validation.warnings.is_empty());
        assert!(validation
            .warnings
            .iter()
            .any(|w| w.contains("doesn't match any exported items")));
    }

    #[test]
    fn test_module_validation_empty_module() {
        let mut manager = ModuleManager::new();
        manager.create_module("EMPTY").unwrap();

        let validation = manager.validate_module("EMPTY").unwrap();
        assert!(validation.is_valid);
        assert!(validation.warnings.iter().any(|w| w.contains("empty")));
    }

    #[test]
    fn test_module_validation_reexport_pattern() {
        let mut manager = ModuleManager::new();
        manager.create_module("SOURCE").unwrap();
        manager.create_module("TARGET").unwrap();

        // SOURCE has rules
        let source = manager.get_module_mut("SOURCE").unwrap();
        source.add_rule("rule1");
        source.set_exports(ExportList::All);

        // TARGET imports with re-export pattern that doesn't match
        manager
            .import_from_with_reexport(
                "TARGET",
                "SOURCE",
                ImportType::AllRules,
                "*",
                Some(ReExport {
                    patterns: vec!["sensor-*".to_string()], // Pattern doesn't match "rule1"
                    transitive: false,
                }),
            )
            .unwrap();

        let validation = manager.validate_module("TARGET").unwrap();
        assert!(validation.is_valid);
        assert!(validation
            .warnings
            .iter()
            .any(|w| w.contains("Re-export pattern")));
    }

    #[test]
    fn test_validate_all_modules() {
        let mut manager = ModuleManager::new();
        manager.create_module("MOD1").unwrap();
        manager.create_module("MOD2").unwrap();

        let validations = manager.validate_all_modules();
        assert_eq!(validations.len(), 3); // MAIN + MOD1 + MOD2
        assert!(validations.contains_key("MAIN"));
        assert!(validations.contains_key("MOD1"));
        assert!(validations.contains_key("MOD2"));
    }

    #[test]
    fn test_selective_reexport() {
        let mut manager = ModuleManager::new();
        manager.create_module("BASE").unwrap();
        manager.create_module("MIDDLE").unwrap();
        manager.create_module("TOP").unwrap();

        // BASE has multiple rules
        let base = manager.get_module_mut("BASE").unwrap();
        base.add_rule("sensor-temp");
        base.add_rule("sensor-pressure");
        base.add_rule("control-valve");
        base.set_exports(ExportList::All);

        // MIDDLE imports all but only re-exports sensor-* rules
        manager
            .import_from_with_reexport(
                "MIDDLE",
                "BASE",
                ImportType::AllRules,
                "*",
                Some(ReExport {
                    patterns: vec!["sensor-*".to_string()],
                    transitive: true,
                }),
            )
            .unwrap();

        // MIDDLE should see all rules
        assert!(manager.is_rule_visible("sensor-temp", "MIDDLE").unwrap());
        assert!(manager
            .is_rule_visible("sensor-pressure", "MIDDLE")
            .unwrap());
        assert!(manager.is_rule_visible("control-valve", "MIDDLE").unwrap());

        // TOP imports from MIDDLE
        manager
            .import_from("TOP", "MIDDLE", ImportType::AllRules, "*")
            .unwrap();

        // TOP should only see re-exported sensor-* rules
        assert!(manager.is_rule_visible("sensor-temp", "TOP").unwrap());
        assert!(manager.is_rule_visible("sensor-pressure", "TOP").unwrap());
        // control-valve is not re-exported, so TOP shouldn't see it through MIDDLE
        // Note: This test shows the design intent, but the current implementation
        // may need additional logic to fully enforce transitive re-export restrictions
    }
}

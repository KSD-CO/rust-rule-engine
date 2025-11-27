//! Integration tests for Module System
//!
//! Tests CLIPS-inspired defmodule functionality including:
//! - Module creation and management
//! - Import/Export mechanisms
//! - Visibility rules
//! - Module focus

use rust_rule_engine::engine::module::{ModuleManager, ExportList, ExportItem, ItemType, ImportType};

#[test]
fn test_basic_module_operations() {
    let mut manager = ModuleManager::new();

    // MAIN module should exist by default
    assert!(manager.get_module("MAIN").is_ok());
    assert_eq!(manager.get_focus(), "MAIN");

    // Create new modules
    assert!(manager.create_module("SENSORS").is_ok());
    assert!(manager.create_module("CONTROL").is_ok());

    // Duplicate creation should fail
    assert!(manager.create_module("SENSORS").is_err());

    // List modules
    let modules = manager.list_modules();
    assert_eq!(modules.len(), 3);
    assert!(modules.contains(&"MAIN".to_string()));
    assert!(modules.contains(&"SENSORS".to_string()));
    assert!(modules.contains(&"CONTROL".to_string()));
}

#[test]
fn test_module_focus() {
    let mut manager = ModuleManager::new();
    manager.create_module("TEST1").unwrap();
    manager.create_module("TEST2").unwrap();

    // Default focus is MAIN
    assert_eq!(manager.get_focus(), "MAIN");

    // Change focus
    manager.set_focus("TEST1").unwrap();
    assert_eq!(manager.get_focus(), "TEST1");

    manager.set_focus("TEST2").unwrap();
    assert_eq!(manager.get_focus(), "TEST2");

    // Invalid module should fail
    assert!(manager.set_focus("NONEXISTENT").is_err());
}

#[test]
fn test_module_deletion() {
    let mut manager = ModuleManager::new();
    manager.create_module("TEMP").unwrap();

    assert!(manager.get_module("TEMP").is_ok());

    // Delete module
    manager.delete_module("TEMP").unwrap();
    assert!(manager.get_module("TEMP").is_err());

    // Cannot delete MAIN module
    assert!(manager.delete_module("MAIN").is_err());
}

#[test]
fn test_export_all() {
    let mut manager = ModuleManager::new();
    manager.create_module("SENSORS").unwrap();

    // Add rules to SENSORS
    let sensors = manager.get_module_mut("SENSORS").unwrap();
    sensors.add_rule("sensor-temp");
    sensors.add_rule("sensor-pressure");

    // Export all
    manager.export_all_from("SENSORS", ExportList::All).unwrap();

    // Check that rules are exported
    let sensors = manager.get_module("SENSORS").unwrap();
    assert!(sensors.exports_rule("sensor-temp"));
    assert!(sensors.exports_rule("sensor-pressure"));
}

#[test]
fn test_export_specific() {
    let mut manager = ModuleManager::new();
    manager.create_module("SENSORS").unwrap();

    // Add rules
    let sensors = manager.get_module_mut("SENSORS").unwrap();
    sensors.add_rule("sensor-temp");
    sensors.add_rule("sensor-pressure");
    sensors.add_rule("control-fan");

    // Export only sensor-* rules
    manager.export_all_from("SENSORS", ExportList::Specific(vec![
        ExportItem {
            item_type: ItemType::Rule,
            pattern: "sensor-*".to_string(),
        },
    ])).unwrap();

    let sensors = manager.get_module("SENSORS").unwrap();
    assert!(sensors.exports_rule("sensor-temp"));
    assert!(sensors.exports_rule("sensor-pressure"));
    assert!(!sensors.exports_rule("control-fan")); // Not exported
}

#[test]
fn test_export_none() {
    let mut manager = ModuleManager::new();
    manager.create_module("PRIVATE").unwrap();

    // Add rules
    let private = manager.get_module_mut("PRIVATE").unwrap();
    private.add_rule("internal-rule");

    // Export none (default for non-MAIN modules)
    assert!(!private.exports_rule("internal-rule"));
}

#[test]
fn test_import_from_module() {
    let mut manager = ModuleManager::new();
    manager.create_module("SENSORS").unwrap();
    manager.create_module("CONTROL").unwrap();

    // Setup SENSORS module
    let sensors = manager.get_module_mut("SENSORS").unwrap();
    sensors.add_rule("sensor-temp");
    sensors.add_rule("sensor-pressure");
    sensors.set_exports(ExportList::All);

    // Import in CONTROL
    manager.import_from("CONTROL", "SENSORS", ImportType::AllRules, "*").unwrap();

    // Verify visibility
    assert!(manager.is_rule_visible("sensor-temp", "CONTROL").unwrap());
    assert!(manager.is_rule_visible("sensor-pressure", "CONTROL").unwrap());
}

#[test]
fn test_import_with_pattern() {
    let mut manager = ModuleManager::new();
    manager.create_module("SENSORS").unwrap();
    manager.create_module("CONTROL").unwrap();

    // Setup SENSORS module
    let sensors = manager.get_module_mut("SENSORS").unwrap();
    sensors.add_rule("sensor-temp");
    sensors.add_rule("sensor-pressure");
    sensors.add_rule("calibrate-sensor");
    sensors.set_exports(ExportList::All);

    // Import only sensor-* rules
    manager.import_from("CONTROL", "SENSORS", ImportType::Rules, "sensor-*").unwrap();

    // Check visibility
    assert!(manager.is_rule_visible("sensor-temp", "CONTROL").unwrap());
    assert!(manager.is_rule_visible("sensor-pressure", "CONTROL").unwrap());
    assert!(!manager.is_rule_visible("calibrate-sensor", "CONTROL").unwrap()); // Not matching pattern
}

#[test]
fn test_visibility_own_rules_always_visible() {
    let mut manager = ModuleManager::new();
    manager.create_module("TEST").unwrap();

    let test = manager.get_module_mut("TEST").unwrap();
    test.add_rule("my-rule");
    test.set_exports(ExportList::None); // Even with no export

    // Own rules are always visible
    assert!(manager.is_rule_visible("my-rule", "TEST").unwrap());
}

#[test]
fn test_visibility_imported_rules() {
    let mut manager = ModuleManager::new();
    manager.create_module("MOD_A").unwrap();
    manager.create_module("MOD_B").unwrap();

    // MOD_A has rules
    let mod_a = manager.get_module_mut("MOD_A").unwrap();
    mod_a.add_rule("rule-a1");
    mod_a.add_rule("rule-a2");
    mod_a.set_exports(ExportList::All);

    // MOD_B imports from MOD_A
    manager.import_from("MOD_B", "MOD_A", ImportType::AllRules, "*").unwrap();

    // MOD_B should see MOD_A's rules
    assert!(manager.is_rule_visible("rule-a1", "MOD_B").unwrap());
    assert!(manager.is_rule_visible("rule-a2", "MOD_B").unwrap());

    // MAIN should NOT see MOD_A's rules (no import)
    assert!(!manager.is_rule_visible("rule-a1", "MAIN").unwrap());
}

#[test]
fn test_get_visible_rules() {
    let mut manager = ModuleManager::new();
    manager.create_module("MOD1").unwrap();
    manager.create_module("MOD2").unwrap();

    // MOD1 has rules
    let mod1 = manager.get_module_mut("MOD1").unwrap();
    mod1.add_rule("rule1");
    mod1.add_rule("rule2");
    mod1.set_exports(ExportList::All);

    // MOD2 has rules and imports from MOD1
    let mod2 = manager.get_module_mut("MOD2").unwrap();
    mod2.add_rule("rule3");

    manager.import_from("MOD2", "MOD1", ImportType::AllRules, "*").unwrap();

    // Get all visible rules in MOD2
    let visible = manager.get_visible_rules("MOD2").unwrap();
    assert_eq!(visible.len(), 3);
    assert!(visible.contains(&"rule1".to_string()));
    assert!(visible.contains(&"rule2".to_string()));
    assert!(visible.contains(&"rule3".to_string()));
}

#[test]
fn test_template_visibility() {
    let mut manager = ModuleManager::new();
    manager.create_module("SENSORS").unwrap();
    manager.create_module("CONTROL").unwrap();

    // SENSORS has templates
    let sensors = manager.get_module_mut("SENSORS").unwrap();
    sensors.add_template("temperature");
    sensors.add_template("pressure");
    sensors.set_exports(ExportList::All);

    // CONTROL imports templates
    manager.import_from("CONTROL", "SENSORS", ImportType::AllTemplates, "*").unwrap();

    // Check visibility
    assert!(manager.is_template_visible("temperature", "CONTROL").unwrap());
    assert!(manager.is_template_visible("pressure", "CONTROL").unwrap());
}

#[test]
fn test_main_module_exports_all_by_default() {
    let manager = ModuleManager::new();
    let main = manager.get_module("MAIN").unwrap();

    // MAIN should export all by default
    assert!(matches!(main.get_exports(), ExportList::All));
}

#[test]
fn test_user_module_exports_none_by_default() {
    let mut manager = ModuleManager::new();
    manager.create_module("USER").unwrap();

    let user = manager.get_module("USER").unwrap();

    // User modules should export none by default
    assert!(matches!(user.get_exports(), ExportList::None));
}

#[test]
fn test_module_stats() {
    let mut manager = ModuleManager::new();
    manager.create_module("TEST1").unwrap();
    manager.create_module("TEST2").unwrap();

    // Add some rules
    let test1 = manager.get_module_mut("TEST1").unwrap();
    test1.add_rule("rule1");
    test1.add_rule("rule2");
    test1.add_template("template1");

    manager.set_focus("TEST1").unwrap();

    let stats = manager.get_stats();

    assert_eq!(stats.total_modules, 3); // MAIN + TEST1 + TEST2
    assert_eq!(stats.current_focus, "TEST1");

    let test1_info = stats.modules.get("TEST1").unwrap();
    assert_eq!(test1_info.rules_count, 2);
    assert_eq!(test1_info.templates_count, 1);
}

#[test]
fn test_complex_import_scenario() {
    // Scenario: MOD_A -> MOD_B -> MOD_C (transitive imports)
    let mut manager = ModuleManager::new();
    manager.create_module("MOD_A").unwrap();
    manager.create_module("MOD_B").unwrap();
    manager.create_module("MOD_C").unwrap();

    // MOD_A has rules
    let mod_a = manager.get_module_mut("MOD_A").unwrap();
    mod_a.add_rule("a-rule");
    mod_a.set_exports(ExportList::All);

    // MOD_B imports from MOD_A
    manager.import_from("MOD_B", "MOD_A", ImportType::AllRules, "*").unwrap();

    // MOD_B has its own rules and exports them
    let mod_b = manager.get_module_mut("MOD_B").unwrap();
    mod_b.add_rule("b-rule");
    mod_b.set_exports(ExportList::All);

    // MOD_C imports from MOD_B
    manager.import_from("MOD_C", "MOD_B", ImportType::AllRules, "*").unwrap();

    // MOD_C should see MOD_B's rules
    assert!(manager.is_rule_visible("b-rule", "MOD_C").unwrap());

    // MOD_C should NOT see MOD_A's rules (no direct import)
    assert!(!manager.is_rule_visible("a-rule", "MOD_C").unwrap());
}

#[test]
fn test_module_with_documentation() {
    let mut manager = ModuleManager::new();
    manager.create_module("DOCUMENTED").unwrap();

    let documented = manager.get_module_mut("DOCUMENTED").unwrap();
    *documented = documented.clone().with_doc("This is a test module");

    assert!(documented.doc.is_some());
    assert_eq!(documented.doc.as_ref().unwrap(), "This is a test module");
}

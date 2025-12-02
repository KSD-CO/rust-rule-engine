//! Module System - Phase 3 Features Demo
//!
//! Demonstrates Phase 3 features:
//! 1. Transitive import support (re-export)
//! 2. Module-level salience configuration
//! 3. Module validation tools

use rust_rule_engine::engine::module::{
    ModuleManager, ExportList, ImportType, ReExport, ExportItem, ItemType
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Module System - Phase 3 Features Demo\n");
    println!("{}", "=".repeat(80));

    // Create module manager
    let mut manager = ModuleManager::new();

    // =========================================================================
    // Part 1: Transitive Imports (Re-export)
    // =========================================================================
    println!("\n📦 Part 1: Transitive Imports (Re-export)\n");
    println!("Creating a 3-tier module hierarchy with re-exports:");
    println!("  BASE → MIDDLEWARE → APPLICATION\n");

    // Create modules
    manager.create_module("BASE")?;
    manager.create_module("MIDDLEWARE")?;
    manager.create_module("APPLICATION")?;

    // BASE module: Foundation rules and templates
    {
        let base = manager.get_module_mut("BASE")?;
        base.add_rule("sensor-temperature");
        base.add_rule("sensor-pressure");
        base.add_rule("sensor-humidity");
        base.add_rule("validate-reading");
        base.add_template("SensorReading");
        base.add_template("ValidationResult");
        base.set_exports(ExportList::All);
        println!("✓ BASE module created with 4 rules and 2 templates");
    }

    // MIDDLEWARE module: Imports from BASE and re-exports selectively
    {
        println!("✓ MIDDLEWARE imports from BASE");
        println!("  - Imports all rules and templates");
        println!("  - Re-exports only sensor-* rules (not validate-reading)");

        manager.import_from_with_reexport(
            "MIDDLEWARE",
            "BASE",
            ImportType::All,
            "*",
            Some(ReExport {
                patterns: vec!["sensor-*".to_string()],
                transitive: true,
            }),
        )?;

        // Add middleware-specific rules
        let middleware = manager.get_module_mut("MIDDLEWARE")?;
        middleware.add_rule("process-sensor-data");
        middleware.add_rule("aggregate-readings");
        middleware.set_exports(ExportList::All);
        println!("  - Added 2 middleware-specific rules");
    }

    // APPLICATION module: Imports from MIDDLEWARE
    {
        println!("✓ APPLICATION imports from MIDDLEWARE");
        manager.import_from("APPLICATION", "MIDDLEWARE", ImportType::All, "*")?;

        let app = manager.get_module_mut("APPLICATION")?;
        app.add_rule("display-dashboard");
        app.add_rule("send-alerts");
    }

    // Verify transitive visibility
    println!("\n📊 Visibility Analysis:");
    println!("  MIDDLEWARE can see:");
    let middleware_rules = manager.get_visible_rules("MIDDLEWARE")?;
    for rule in &middleware_rules {
        println!("    ✓ {}", rule);
    }

    println!("\n  APPLICATION can see:");
    let app_rules = manager.get_visible_rules("APPLICATION")?;
    for rule in &app_rules {
        println!("    ✓ {}", rule);
    }

    // Check what APPLICATION CANNOT see (validate-reading)
    let can_see_validate = manager.is_rule_visible("validate-reading", "APPLICATION")?;
    println!("\n  APPLICATION can see 'validate-reading' (not re-exported): {}",
             if can_see_validate { "❌ YES (unexpected)" } else { "✓ NO (correct)" });

    let can_see_sensor = manager.is_rule_visible("sensor-temperature", "APPLICATION")?;
    println!("  APPLICATION can see 'sensor-temperature' (re-exported): {}",
             if can_see_sensor { "✓ YES" } else { "❌ NO (unexpected)" });

    // =========================================================================
    // Part 2: Module-Level Salience
    // =========================================================================
    println!("\n\n🎯 Part 2: Module-Level Salience Configuration\n");
    println!("Creating modules with different priority levels:");

    manager.create_module("CRITICAL_ALERTS")?;
    manager.create_module("STANDARD_PROCESSING")?;
    manager.create_module("BACKGROUND_TASKS")?;

    // Set different salience levels
    manager.set_module_salience("CRITICAL_ALERTS", 1000)?;
    manager.set_module_salience("STANDARD_PROCESSING", 0)?;
    manager.set_module_salience("BACKGROUND_TASKS", -500)?;

    println!("  ✓ CRITICAL_ALERTS: salience = {}",
             manager.get_module_salience("CRITICAL_ALERTS")?);
    println!("  ✓ STANDARD_PROCESSING: salience = {}",
             manager.get_module_salience("STANDARD_PROCESSING")?);
    println!("  ✓ BACKGROUND_TASKS: salience = {}",
             manager.get_module_salience("BACKGROUND_TASKS")?);

    println!("\n  💡 Use case: Rules in CRITICAL_ALERTS will fire before others");
    println!("     when combined with rule-level salience:");
    println!("     - Module salience: base priority for all rules in module");
    println!("     - Rule salience: fine-tuned adjustment per rule");
    println!("     - Total priority = module_salience + rule_salience");

    // =========================================================================
    // Part 3: Module Validation Tools
    // =========================================================================
    println!("\n\n🔍 Part 3: Module Validation Tools\n");

    // Create some modules with issues for demonstration
    manager.create_module("VALID_MODULE")?;
    manager.create_module("EMPTY_MODULE")?;
    manager.create_module("MODULE_WITH_UNUSED_IMPORT")?;

    // Add content to VALID_MODULE
    {
        let valid = manager.get_module_mut("VALID_MODULE")?;
        valid.add_rule("rule1");
        valid.add_rule("rule2");
        valid.add_template("template1");
        valid.set_exports(ExportList::All);
    }

    // MODULE_WITH_UNUSED_IMPORT imports but nothing matches
    {
        let source = manager.get_module_mut("VALID_MODULE")?;
        source.set_exports(ExportList::Specific(vec![
            ExportItem {
                item_type: ItemType::Rule,
                pattern: "rule1".to_string(),
            }
        ]));

        manager.import_from("MODULE_WITH_UNUSED_IMPORT", "VALID_MODULE",
                           ImportType::AllRules, "nonexistent-*")?;
    }

    println!("Validating individual modules:\n");

    // Validate VALID_MODULE
    let validation = manager.validate_module("VALID_MODULE")?;
    println!("  📋 VALID_MODULE:");
    println!("     Valid: {}", if validation.is_valid { "✅ YES" } else { "❌ NO" });
    println!("     Errors: {}", validation.errors.len());
    println!("     Warnings: {}", validation.warnings.len());

    // Validate EMPTY_MODULE
    let validation = manager.validate_module("EMPTY_MODULE")?;
    println!("\n  📋 EMPTY_MODULE:");
    println!("     Valid: {}", if validation.is_valid { "✅ YES" } else { "❌ NO" });
    println!("     Errors: {}", validation.errors.len());
    println!("     Warnings: {}", validation.warnings.len());
    if !validation.warnings.is_empty() {
        for warning in &validation.warnings {
            println!("       ⚠️  {}", warning);
        }
    }

    // Validate MODULE_WITH_UNUSED_IMPORT
    let validation = manager.validate_module("MODULE_WITH_UNUSED_IMPORT")?;
    println!("\n  📋 MODULE_WITH_UNUSED_IMPORT:");
    println!("     Valid: {}", if validation.is_valid { "✅ YES" } else { "❌ NO" });
    println!("     Errors: {}", validation.errors.len());
    println!("     Warnings: {}", validation.warnings.len());
    if !validation.warnings.is_empty() {
        for warning in &validation.warnings {
            println!("       ⚠️  {}", warning);
        }
    }

    // Validate all modules
    println!("\n\nValidating all modules in the system:\n");
    let all_validations = manager.validate_all_modules();

    let valid_count = all_validations.values().filter(|v| v.is_valid).count();
    let warning_count: usize = all_validations.values().map(|v| v.warnings.len()).sum();
    let error_count: usize = all_validations.values().map(|v| v.errors.len()).sum();

    println!("  📊 Summary:");
    println!("     Total modules: {}", all_validations.len());
    println!("     Valid: {}", valid_count);
    println!("     Total warnings: {}", warning_count);
    println!("     Total errors: {}", error_count);

    // =========================================================================
    // Part 4: Transitive Dependencies Analysis
    // =========================================================================
    println!("\n\n🔗 Part 4: Transitive Dependencies Analysis\n");

    // Get transitive dependencies for APPLICATION
    let deps = manager.get_transitive_dependencies("APPLICATION")?;
    println!("  APPLICATION depends on {} modules:", deps.len());
    for dep in &deps {
        println!("    → {}", dep);
    }

    // Get transitive dependencies for MIDDLEWARE
    let deps = manager.get_transitive_dependencies("MIDDLEWARE")?;
    println!("\n  MIDDLEWARE depends on {} modules:", deps.len());
    for dep in &deps {
        println!("    → {}", dep);
    }

    // =========================================================================
    // Part 5: Complete System Statistics
    // =========================================================================
    println!("\n\n📊 Part 5: Complete System Statistics\n");

    let stats = manager.get_stats();
    println!("  Total modules: {}", stats.total_modules);
    println!("  Current focus: {}", stats.current_focus);
    println!("\n  Module details:");

    let mut module_names: Vec<_> = stats.modules.keys().collect();
    module_names.sort();

    for name in module_names {
        if let Some(info) = stats.modules.get(name) {
            println!("\n    📦 {}:", name);
            println!("       Rules: {}", info.rules_count);
            println!("       Templates: {}", info.templates_count);
            println!("       Imports: {}", info.imports_count);
            println!("       Exports: {}", info.exports_type);
            println!("       Salience: {}", info.salience);
        }
    }

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n\n{}", "=".repeat(80));
    println!("✅ Phase 3 Features Demonstration Complete!\n");
    println!("Key Features Demonstrated:");
    println!("  1. ✓ Transitive re-export (MIDDLEWARE re-exports from BASE)");
    println!("  2. ✓ Module-level salience (priority levels for modules)");
    println!("  3. ✓ Module validation (detect issues and warnings)");
    println!("  4. ✓ Transitive dependency analysis (BFS traversal)");
    println!("  5. ✓ Complete system statistics and introspection");
    println!("\n🎉 All Phase 3 features are working correctly!");

    Ok(())
}

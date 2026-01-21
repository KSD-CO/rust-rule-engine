/// Smart Home System with Module System
///
/// This example demonstrates the module system in action with a realistic
/// smart home control system that organizes rules and templates across
/// multiple modules with controlled visibility.
///
/// Module Architecture:
/// ```
///   SENSORS (collects data)
///      ↓
///   CONTROL (makes decisions)  ← also imports from SENSORS
///      ↓
///   ALERT (sends notifications) ← imports from both
///      ↓
///   LOGGER (logs everything)
/// ```
///
/// Run with:
/// ```
/// cargo run --example smart_home_modules
/// ```
use rust_rule_engine::engine::module::{ExportList, ImportType, ModuleManager};
use rust_rule_engine::parser::GRLParser;
use std::fs;

#[allow(dead_code)]
#[allow(unused_must_use)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏠 Smart Home System with Module Architecture\n");
    println!("{}\n", "=".repeat(70));

    // Load GRL file
    let grl_path = "examples/10-module-system/smart_home.grl";
    let grl_content =
        fs::read_to_string(grl_path).map_err(|e| format!("Failed to read {}: {}", grl_path, e))?;

    // Parse GRL with modules
    let parsed = GRLParser::parse_with_modules(&grl_content)?;
    let mut manager = parsed.module_manager;
    let rules = parsed.rules;
    let _rule_modules = parsed.rule_modules;

    println!("📂 Loaded from: {}\n", grl_path);
    println!(
        "✅ Parsed {} rules into {} modules\n",
        rules.len(),
        manager.list_modules().len()
    );

    // Alternative: Set up module hierarchy programmatically (if file not found)
    // setup_modules(&mut manager)?;

    // Display module structure
    display_module_structure(&manager)?;

    // Demonstrate visibility
    demonstrate_visibility(&manager)?;

    // Show module statistics
    show_statistics(&manager);

    // Demonstrate execution flow
    demonstrate_execution_flow(&mut manager)?;

    println!("\n{}\n", "=".repeat(70));
    println!("✅ Smart Home Module System Demo Complete!");

    Ok(())
}

/// Set up the module hierarchy for smart home system
#[allow(dead_code)]
fn setup_modules(manager: &mut ModuleManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Setting up Module Hierarchy...\n");

    // Create SENSORS module
    manager.create_module("SENSORS")?;
    {
        let sensors = manager.get_module_mut("SENSORS")?;
        sensors.add_template("temperature");
        sensors.add_template("humidity");
        sensors.add_template("motion");
        sensors.add_rule("check-temperature");
        sensors.add_rule("check-humidity");
        sensors.set_exports(ExportList::All);
        println!("  ✓ SENSORS module created");
        println!("    - Templates: temperature, humidity, motion");
        println!("    - Rules: check-temperature, check-humidity");
        println!("    - Exports: All");
    }

    // Create CONTROL module
    manager.create_module("CONTROL")?;
    {
        let control = manager.get_module_mut("CONTROL")?;
        control.add_template("hvac-state");
        control.add_template("light-state");
        control.add_rule("activate-cooling");
        control.add_rule("activate-heating");
        control.add_rule("turn-on-lights-on-motion");
        control.set_exports(ExportList::All);

        // Import from SENSORS
        manager.import_from("CONTROL", "SENSORS", ImportType::AllRules, "*")?;
        manager.import_from("CONTROL", "SENSORS", ImportType::AllTemplates, "*")?;

        println!("  ✓ CONTROL module created");
        println!("    - Templates: hvac-state, light-state");
        println!("    - Rules: activate-cooling, activate-heating, turn-on-lights-on-motion");
        println!("    - Exports: All");
        println!("    - Imports: All from SENSORS");
    }

    // Create ALERT module
    manager.create_module("ALERT")?;
    {
        let alert = manager.get_module_mut("ALERT")?;
        alert.add_template("alert-log");
        alert.add_rule("log-high-temp-alert");
        alert.add_rule("log-control-action");
        alert.set_exports(ExportList::All);

        // Import from SENSORS and CONTROL
        manager.import_from("ALERT", "SENSORS", ImportType::AllTemplates, "*")?;
        manager.import_from("ALERT", "CONTROL", ImportType::AllRules, "*")?;
        manager.import_from("ALERT", "CONTROL", ImportType::AllTemplates, "*")?;

        println!("  ✓ ALERT module created");
        println!("    - Templates: alert-log");
        println!("    - Rules: log-high-temp-alert, log-control-action");
        println!("    - Exports: All");
        println!("    - Imports: All templates from SENSORS, all from CONTROL");
    }

    // Create LOGGER module
    manager.create_module("LOGGER")?;
    {
        let logger = manager.get_module_mut("LOGGER")?;
        logger.add_template("system-log");
        logger.add_rule("log-all-temps");
        logger.add_rule("log-all-humidity");
        logger.add_rule("log-alerts");

        // Import from all other modules
        manager.import_from("LOGGER", "SENSORS", ImportType::AllTemplates, "*")?;
        manager.import_from("LOGGER", "CONTROL", ImportType::AllTemplates, "*")?;
        manager.import_from("LOGGER", "ALERT", ImportType::AllTemplates, "*")?;

        println!("  ✓ LOGGER module created");
        println!("    - Templates: system-log");
        println!("    - Rules: log-all-temps, log-all-humidity, log-alerts");
        println!("    - Imports: All templates from SENSORS, CONTROL, ALERT");
    }

    Ok(())
}

/// Display the module structure
fn display_module_structure(manager: &ModuleManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}\n", "=".repeat(70));
    println!("📋 Module Structure:\n");

    let modules = manager.list_modules();
    for module_name in modules {
        let module = manager.get_module(&module_name)?;

        println!(
            "  ┌─ Module: {} ({})",
            module_name,
            if module_name == manager.get_focus() {
                "FOCUSED"
            } else {
                "idle"
            }
        );

        // Show templates
        let templates = module.get_templates();
        if !templates.is_empty() {
            println!("  │  Templates:");
            for template in templates {
                println!("  │    - {}", template);
            }
        }

        // Show rules
        let rules = module.get_rules();
        if !rules.is_empty() {
            println!("  │  Rules:");
            for rule in rules {
                println!("  │    - {}", rule);
            }
        }

        // Show imports
        let imports = module.get_imports();
        if !imports.is_empty() {
            println!("  │  Imports:");
            for import in imports {
                println!(
                    "  │    - from {} (pattern: {})",
                    import.from_module, import.pattern
                );
            }
        }

        println!("  └");
    }

    Ok(())
}

/// Demonstrate visibility rules
fn demonstrate_visibility(manager: &ModuleManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n{}\n", "=".repeat(70));
    println!("🔍 Visibility Analysis:\n");

    let test_cases = vec![
        ("SENSORS", "check-temperature", true, "Own rule"),
        (
            "SENSORS",
            "activate-cooling",
            false,
            "Rule from other module",
        ),
        (
            "CONTROL",
            "check-temperature",
            true,
            "Imported from SENSORS",
        ),
        ("CONTROL", "activate-cooling", true, "Own rule"),
        (
            "CONTROL",
            "log-high-temp-alert",
            false,
            "Rule from ALERT (no import)",
        ),
        ("ALERT", "check-temperature", true, "Imported from SENSORS"),
        ("ALERT", "activate-cooling", true, "Imported from CONTROL"),
        ("ALERT", "log-high-temp-alert", true, "Own rule"),
        (
            "LOGGER",
            "temperature",
            true,
            "Template imported from SENSORS",
        ),
        (
            "LOGGER",
            "hvac-state",
            true,
            "Template imported from CONTROL",
        ),
        ("LOGGER", "alert-log", true, "Template imported from ALERT"),
    ];

    for (module, rule, expected, reason) in test_cases {
        let visible = manager.is_rule_visible(rule, module).unwrap_or(false)
            || manager.is_template_visible(rule, module).unwrap_or(false);

        let status = if visible == expected { "✓" } else { "✗" };
        let symbol = if visible { "👁️ " } else { "🔒" };

        println!(
            "  {} {} {:<25} visible to {:<10} | {}",
            status, symbol, rule, module, reason
        );
    }

    Ok(())
}

/// Show module statistics
fn show_statistics(manager: &ModuleManager) {
    println!("\n{}\n", "=".repeat(70));
    println!("📊 Module Statistics:\n");

    let stats = manager.get_stats();

    println!("  Total Modules: {}", stats.total_modules);
    println!("  Current Focus: {}", stats.current_focus);
    println!();

    let mut module_list: Vec<_> = stats.modules.iter().collect();
    module_list.sort_by_key(|&(name, _)| name);

    for (name, info) in module_list {
        println!("  Module: {} ({})", name, info.name);
        println!("    - Rules:     {}", info.rules_count);
        println!("    - Templates: {}", info.templates_count);
        println!("    - Imports:   {}", info.imports_count);
        println!("    - Exports:   {}", info.exports_type);
        println!();
    }
}

/// Demonstrate execution flow by changing module focus
fn demonstrate_execution_flow(
    manager: &mut ModuleManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}\n", "=".repeat(70));
    println!("🔄 Module Execution Flow Simulation:\n");

    let flow_sequence = vec![
        ("SENSORS", "Collecting sensor data..."),
        ("CONTROL", "Processing sensor data and making decisions..."),
        ("ALERT", "Generating alerts..."),
        ("LOGGER", "Logging all events..."),
    ];

    for (module_name, description) in flow_sequence {
        manager.set_focus(module_name)?;

        println!("  Step: Set focus to {}", module_name);
        println!("  └─ {}", description);

        // Show visible rules in this module
        let visible_rules = manager.get_visible_rules(module_name)?;
        println!("  └─ Available rules: {:?}", visible_rules);

        println!();
    }

    Ok(())
}

//! Module System Demo (CLIPS-inspired defmodule)
//!
//! This example demonstrates the module system for organizing large knowledge bases
//! into isolated, manageable modules with controlled visibility.
//!
//! Features demonstrated:
//! - Module creation and management
//! - Import/Export mechanisms
//! - Visibility rules
//! - Module focus for execution control
//!
//! Run with:
//! ```
//! cargo run --example module_demo
//! ```

use rust_rule_engine::engine::module::{
    ExportItem, ExportList, ImportType, ItemType, ModuleManager,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Module System Demo ===\n");

    demo_basic_modules()?;
    println!("\n{}\n", "=".repeat(60));
    demo_iot_system()?;

    Ok(())
}

/// Demo 1: Basic module operations
fn demo_basic_modules() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Demo 1: Basic Module Operations\n");

    let mut manager = ModuleManager::new();

    // Create modules
    println!("Creating modules...");
    manager.create_module("SENSORS")?;
    manager.create_module("CONTROL")?;
    manager.create_module("ALERT")?;

    println!("✓ Created modules: {:?}\n", manager.list_modules());

    // Add rules to SENSORS module
    let sensors = manager.get_module_mut("SENSORS")?;
    sensors.add_rule("sensor-temperature");
    sensors.add_rule("sensor-pressure");
    sensors.add_rule("sensor-humidity");
    sensors.set_exports(ExportList::All);

    println!("SENSORS module:");
    println!("  Rules: {:?}", sensors.get_rules());
    println!("  Exports: All\n");

    // Setup CONTROL module with imports
    manager.import_from("CONTROL", "SENSORS", ImportType::AllRules, "sensor-*")?;

    let control = manager.get_module_mut("CONTROL")?;
    control.add_rule("control-fan");
    control.add_rule("control-heater");

    println!("CONTROL module:");
    println!("  Own rules: {:?}", control.get_rules());
    println!("  Imports from: SENSORS (sensor-*)\n");

    // Check visibility
    println!("Visibility checks:");
    println!(
        "  CONTROL can see 'sensor-temperature': {}",
        manager.is_rule_visible("sensor-temperature", "CONTROL")?
    );
    println!(
        "  CONTROL can see 'control-fan': {}",
        manager.is_rule_visible("control-fan", "CONTROL")?
    );
    println!(
        "  ALERT can see 'sensor-temperature': {}",
        manager.is_rule_visible("sensor-temperature", "ALERT")?
    );

    // Get all visible rules
    let visible = manager.get_visible_rules("CONTROL")?;
    println!("\nAll rules visible to CONTROL: {:?}", visible);

    // Module focus
    println!("\nModule Focus:");
    println!("  Current focus: {}", manager.get_focus());
    manager.set_focus("CONTROL")?;
    println!("  Switched to: {}", manager.get_focus());

    Ok(())
}

/// Demo 2: Complete IoT system with multiple modules
fn demo_iot_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏭 Demo 2: IoT System with Modular Architecture\n");

    let mut manager = ModuleManager::new();

    // ========================================
    // SENSORS Module - Data collection
    // ========================================
    println!("Creating SENSORS module...");
    manager.create_module("SENSORS")?;

    let sensors = manager.get_module_mut("SENSORS")?;
    *sensors = sensors
        .clone()
        .with_doc("Handles all sensor data collection and initial processing");

    // Add sensor rules
    sensors.add_rule("read-temperature-sensor");
    sensors.add_rule("read-pressure-sensor");
    sensors.add_rule("read-humidity-sensor");
    sensors.add_rule("validate-sensor-data");

    // Add templates
    sensors.add_template("temperature-reading");
    sensors.add_template("pressure-reading");
    sensors.add_template("humidity-reading");

    // Export all templates and sensor-* rules
    sensors.set_exports(ExportList::Specific(vec![
        ExportItem {
            item_type: ItemType::Template,
            pattern: "*".to_string(), // Export all templates
        },
        ExportItem {
            item_type: ItemType::Rule,
            pattern: "read-*".to_string(), // Export read rules
        },
    ]));

    println!("✓ SENSORS module configured");
    println!("  Doc: {}", sensors.doc.as_ref().unwrap());
    println!("  Rules: {}", sensors.get_rules().len());
    println!("  Templates: {}", sensors.get_templates().len());
    println!("  Exports: Templates(*), Rules(read-*)\n");

    // ========================================
    // CONTROL Module - Decision making
    // ========================================
    println!("Creating CONTROL module...");
    manager.create_module("CONTROL")?;

    // Import templates and rules from SENSORS
    manager.import_from("CONTROL", "SENSORS", ImportType::AllTemplates, "*")?;
    manager.import_from("CONTROL", "SENSORS", ImportType::Rules, "read-*")?;

    let control = manager.get_module_mut("CONTROL")?;
    *control = control
        .clone()
        .with_doc("Makes control decisions based on sensor data");

    // Add control rules
    control.add_rule("control-temperature");
    control.add_rule("control-pressure");
    control.add_rule("control-fan-speed");
    control.add_rule("control-heater-power");

    // Export control decisions
    control.set_exports(ExportList::Specific(vec![ExportItem {
        item_type: ItemType::Rule,
        pattern: "control-*".to_string(),
    }]));

    println!("✓ CONTROL module configured");
    println!("  Doc: {}", control.doc.as_ref().unwrap());
    println!("  Rules: {}", control.get_rules().len());
    println!("  Imports: SENSORS(templates + rules)");
    println!("  Exports: Rules(control-*)\n");

    // ========================================
    // ALERT Module - Alert generation
    // ========================================
    println!("Creating ALERT module...");
    manager.create_module("ALERT")?;

    // Import from both SENSORS and CONTROL
    manager.import_from("ALERT", "SENSORS", ImportType::AllTemplates, "*")?;
    manager.import_from("ALERT", "CONTROL", ImportType::Rules, "control-*")?;

    let alert = manager.get_module_mut("ALERT")?;
    *alert = alert
        .clone()
        .with_doc("Generates alerts based on sensor data and control decisions");

    // Add alert rules
    alert.add_rule("alert-high-temperature");
    alert.add_rule("alert-low-pressure");
    alert.add_rule("alert-system-failure");
    alert.add_rule("alert-maintenance-required");

    alert.set_exports(ExportList::All);

    println!("✓ ALERT module configured");
    println!("  Doc: {}", alert.doc.as_ref().unwrap());
    println!("  Rules: {}", alert.get_rules().len());
    println!("  Imports: SENSORS(templates), CONTROL(rules)");
    println!("  Exports: All\n");

    // ========================================
    // Display system statistics
    // ========================================
    println!("{}\n", "=".repeat(60));
    println!("📊 System Statistics:\n");

    let stats = manager.get_stats();
    println!("Total modules: {}", stats.total_modules);
    println!("Current focus: {}\n", stats.current_focus);

    for (name, info) in stats.modules.iter() {
        if name == "MAIN" {
            continue; // Skip MAIN module
        }

        println!("Module: {}", info.name);
        println!("  Rules: {}", info.rules_count);
        println!("  Templates: {}", info.templates_count);
        println!("  Imports: {}", info.imports_count);
        println!("  Exports: {}", info.exports_type);
    }

    // ========================================
    // Demonstrate visibility
    // ========================================
    println!("\n{}\n", "=".repeat(60));
    println!("🔍 Visibility Analysis:\n");

    // CONTROL module visibility
    println!("CONTROL module can see:");
    let control_visible = manager.get_visible_rules("CONTROL")?;
    for rule in control_visible.iter().take(5) {
        println!("  ✓ {}", rule);
    }
    println!(
        "  ... and {} more rules",
        control_visible.len().saturating_sub(5)
    );

    // ALERT module visibility
    println!("\nALERT module can see:");
    let alert_visible = manager.get_visible_rules("ALERT")?;
    for rule in alert_visible.iter().take(5) {
        println!("  ✓ {}", rule);
    }
    println!(
        "  ... and {} more rules",
        alert_visible.len().saturating_sub(5)
    );

    // Template visibility
    println!("\nTemplate visibility:");
    println!(
        "  CONTROL can see 'temperature-reading': {}",
        manager.is_template_visible("temperature-reading", "CONTROL")?
    );
    println!(
        "  ALERT can see 'temperature-reading': {}",
        manager.is_template_visible("temperature-reading", "ALERT")?
    );
    println!(
        "  MAIN can see 'temperature-reading': {}",
        manager.is_template_visible("temperature-reading", "MAIN")?
    );

    // ========================================
    // Execution flow simulation
    // ========================================
    println!("\n{}\n", "=".repeat(60));
    println!("🔄 Execution Flow Simulation:\n");

    println!("Step 1: Focus on SENSORS - Process sensor data");
    manager.set_focus("SENSORS")?;
    println!("  Current focus: {}", manager.get_focus());
    println!(
        "  Rules active: {:?}",
        manager.get_visible_rules("SENSORS")?.len()
    );

    println!("\nStep 2: Focus on CONTROL - Make decisions");
    manager.set_focus("CONTROL")?;
    println!("  Current focus: {}", manager.get_focus());
    println!(
        "  Rules active: {:?}",
        manager.get_visible_rules("CONTROL")?.len()
    );

    println!("\nStep 3: Focus on ALERT - Generate alerts");
    manager.set_focus("ALERT")?;
    println!("  Current focus: {}", manager.get_focus());
    println!(
        "  Rules active: {:?}",
        manager.get_visible_rules("ALERT")?.len()
    );

    // ========================================
    // Benefits summary
    // ========================================
    println!("\n{}\n", "=".repeat(60));
    println!("💡 Benefits of Module System:\n");
    println!("✓ Organization: Clear separation of concerns");
    println!("✓ Namespace: No name conflicts between modules");
    println!("✓ Encapsulation: Hide implementation details");
    println!("✓ Reusability: Import/reuse modules across systems");
    println!("✓ Scalability: Better structure for large knowledge bases");
    println!("✓ Maintainability: Easier to understand and modify");

    Ok(())
}

//! Cyclic Import Detection Example
//!
//! Demonstrates the module system's cycle detection feature which prevents
//! circular dependencies between modules.
//!
//! This is important for:
//! - Preventing infinite loops during module loading
//! - Maintaining clean module architecture
//! - Detecting configuration errors early
//! - Preventing memory issues from recursive imports
//!
//! # Example Scenarios
//!
//! 1. Self-import detection (A -> A)
//! 2. Simple cycles (A -> B -> A)
//! 3. Complex cycles (A -> B -> C -> A)
//! 4. Valid chains that don't cycle
//! 5. Diamond dependencies (multiple paths, no cycle)

use rust_rule_engine::engine::module::{ImportType, ModuleManager};

fn main() {
    println!("=== Cyclic Import Detection Demo ===\n");

    example_1_self_import();
    println!("\n{}\n", "=".repeat(50));

    example_2_simple_cycle();
    println!("\n{}\n", "=".repeat(50));

    example_3_complex_cycle();
    println!("\n{}\n", "=".repeat(50));

    example_4_valid_chains();
    println!("\n{}\n", "=".repeat(50));

    example_5_diamond_dependency();
    println!("\n{}\n", "=".repeat(50));

    example_6_real_world_system();
}

/// Example 1: Self-import detection
fn example_1_self_import() {
    println!("EXAMPLE 1: Self-Import Detection (A -> A)\n");

    let mut manager = ModuleManager::new();
    manager.create_module("DATABASE").unwrap();

    println!("Attempting: DATABASE imports from DATABASE");
    match manager.import_from("DATABASE", "DATABASE", ImportType::AllRules, "*") {
        Ok(_) => println!("  ✓ Allowed (shouldn't happen)"),
        Err(e) => println!("  ✗ Rejected: {}", e),
    }
}

/// Example 2: Simple two-module cycle
fn example_2_simple_cycle() {
    println!("EXAMPLE 2: Simple Cycle Detection (A -> B -> A)\n");

    let mut manager = ModuleManager::new();
    manager.create_module("PARSER").unwrap();
    manager.create_module("TOKENIZER").unwrap();

    println!("Step 1: PARSER imports from TOKENIZER");
    match manager.import_from("PARSER", "TOKENIZER", ImportType::AllRules, "*") {
        Ok(_) => println!("  ✓ Allowed"),
        Err(e) => println!("  ✗ Rejected: {}", e),
    }

    println!("\nStep 2: TOKENIZER imports from PARSER (creates cycle)");
    match manager.import_from("TOKENIZER", "PARSER", ImportType::AllRules, "*") {
        Ok(_) => println!("  ✓ Allowed (shouldn't happen)"),
        Err(e) => println!("  ✗ Rejected: {}", e),
    }

    println!("\nImport graph:");
    for (module, imports) in manager.get_import_graph_debug() {
        if !imports.is_empty() {
            println!("  {} imports from: {:?}", module, imports);
        }
    }
}

/// Example 3: Longer cycle chain
fn example_3_complex_cycle() {
    println!("EXAMPLE 3: Complex Cycle (A -> B -> C -> A)\n");

    let mut manager = ModuleManager::new();
    manager.create_module("COMPILER").unwrap();
    manager.create_module("OPTIMIZER").unwrap();
    manager.create_module("CODEGEN").unwrap();

    println!("Building valid chain: COMPILER -> OPTIMIZER -> CODEGEN");
    manager
        .import_from("COMPILER", "OPTIMIZER", ImportType::AllRules, "*")
        .unwrap();
    println!("  ✓ COMPILER imports from OPTIMIZER");

    manager
        .import_from("OPTIMIZER", "CODEGEN", ImportType::AllRules, "*")
        .unwrap();
    println!("  ✓ OPTIMIZER imports from CODEGEN");

    println!("\nAttempting to close the loop: CODEGEN -> COMPILER");
    match manager.import_from("CODEGEN", "COMPILER", ImportType::AllRules, "*") {
        Ok(_) => println!("  ✓ Allowed (shouldn't happen)"),
        Err(e) => println!("  ✗ Rejected: {}", e),
    }

    println!("\nFinal import graph:");
    for (module, imports) in manager.get_import_graph_debug() {
        if !imports.is_empty() {
            println!("  {} -> {:?}", module, imports);
        }
    }
}

/// Example 4: Valid chains without cycles
fn example_4_valid_chains() {
    println!("EXAMPLE 4: Valid Chain (No Cycles)\n");

    let mut manager = ModuleManager::new();
    manager.create_module("SENSORS").unwrap();
    manager.create_module("PROCESSOR").unwrap();
    manager.create_module("ACTUATORS").unwrap();

    println!("Building valid chain: SENSORS -> PROCESSOR -> ACTUATORS");

    println!("  SENSORS imports from PROCESSOR");
    manager
        .import_from("SENSORS", "PROCESSOR", ImportType::AllRules, "*")
        .unwrap();
    println!("    ✓ Success");

    println!("  PROCESSOR imports from ACTUATORS");
    manager
        .import_from("PROCESSOR", "ACTUATORS", ImportType::AllRules, "*")
        .unwrap();
    println!("    ✓ Success");

    println!("\nAll imports successful (no cycles detected)!");

    println!("\nFinal import graph:");
    for (module, imports) in manager.get_import_graph_debug() {
        if !imports.is_empty() {
            println!("  {} -> {:?}", module, imports);
        }
    }
}

/// Example 5: Diamond dependency (valid, no cycles)
fn example_5_diamond_dependency() {
    println!("EXAMPLE 5: Diamond Dependency Pattern (No Cycle)\n");

    let mut manager = ModuleManager::new();
    manager.create_module("CORE").unwrap();
    manager.create_module("SERVICE_A").unwrap();
    manager.create_module("SERVICE_B").unwrap();

    println!("Building diamond: SERVICE_A -> CORE <- SERVICE_B\n");

    println!("  SERVICE_A imports from CORE");
    manager
        .import_from("SERVICE_A", "CORE", ImportType::AllRules, "*")
        .unwrap();
    println!("    ✓ Success");

    println!("  SERVICE_B imports from CORE");
    manager
        .import_from("SERVICE_B", "CORE", ImportType::AllRules, "*")
        .unwrap();
    println!("    ✓ Success");

    println!("\n  SERVICE_A imports from SERVICE_B (allowed, no cycle)");
    match manager.import_from("SERVICE_A", "SERVICE_B", ImportType::AllRules, "*") {
        Ok(_) => println!("    ✓ Success"),
        Err(e) => println!("    ✗ Rejected: {}", e),
    }

    println!("\n  But SERVICE_B -> SERVICE_A creates cycle:");
    match manager.import_from("SERVICE_B", "SERVICE_A", ImportType::AllRules, "*") {
        Ok(_) => println!("    ✓ Allowed"),
        Err(e) => println!("    ✗ Rejected: {}", e),
    }

    println!("\nFinal import graph:");
    for (module, imports) in manager.get_import_graph_debug() {
        if !imports.is_empty() {
            println!("  {} -> {:?}", module, imports);
        }
    }
}

/// Example 6: Real-world smart home system
fn example_6_real_world_system() {
    println!("EXAMPLE 6: Real-World Smart Home System\n");

    let mut manager = ModuleManager::new();

    // Create module hierarchy
    manager.create_module("HARDWARE_DRIVERS").unwrap();
    manager.create_module("SENSORS").unwrap();
    manager.create_module("CONTROL_LOGIC").unwrap();
    manager.create_module("AUTOMATION").unwrap();
    manager.create_module("FEEDBACK").unwrap();
    manager.create_module("UTILITIES").unwrap();

    println!("Building smart home module architecture:\n");

    // Valid architecture
    println!("Layer 1: SENSORS imports from HARDWARE_DRIVERS");
    manager
        .import_from("SENSORS", "HARDWARE_DRIVERS", ImportType::AllRules, "*")
        .unwrap();
    println!("  ✓ Success\n");

    println!("Layer 2: CONTROL_LOGIC imports from SENSORS");
    manager
        .import_from("CONTROL_LOGIC", "SENSORS", ImportType::AllRules, "*")
        .unwrap();
    println!("  ✓ Success\n");

    println!("Layer 3: AUTOMATION imports from CONTROL_LOGIC");
    manager
        .import_from("AUTOMATION", "CONTROL_LOGIC", ImportType::AllRules, "*")
        .unwrap();
    println!("  ✓ Success\n");

    println!("Layer 4: FEEDBACK imports from AUTOMATION");
    manager
        .import_from("FEEDBACK", "AUTOMATION", ImportType::AllRules, "*")
        .unwrap();
    println!("  ✓ Success\n");

    println!("Utility: CONTROL_LOGIC also imports from UTILITIES");
    manager
        .import_from("CONTROL_LOGIC", "UTILITIES", ImportType::AllRules, "*")
        .unwrap();
    println!("  ✓ Success\n");

    println!("Utility: AUTOMATION also imports from UTILITIES");
    manager
        .import_from("AUTOMATION", "UTILITIES", ImportType::AllRules, "*")
        .unwrap();
    println!("  ✓ Success\n");

    println!("Attempting invalid feedback loop: FEEDBACK -> SENSORS");
    match manager.import_from("FEEDBACK", "SENSORS", ImportType::AllRules, "*") {
        Ok(_) => println!("  ✓ Allowed"),
        Err(e) => println!("  ✗ Rejected (correctly): {}\n", e),
    }

    println!("Module import graph:");
    println!("{}", "─".repeat(60));
    let mut modules: Vec<_> = manager.get_import_graph_debug().into_iter().collect();
    modules.sort_by(|a, b| a.0.cmp(&b.0));

    for (module, imports) in modules {
        if !imports.is_empty() {
            println!("{:20} -> {:?}", module, imports);
        }
    }
    println!("{}\n", "─".repeat(60));

    println!("✓ System architecture is valid with no circular dependencies!");
}

use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::plugin::RulePlugin;
use rust_rule_engine::engine::RustRuleEngine;
use rust_rule_engine::errors::Result;
use rust_rule_engine::plugins::{
    CollectionUtilsPlugin, DateUtilsPlugin, MathUtilsPlugin, StringUtilsPlugin, ValidationPlugin,
};

fn main() -> Result<()> {
    println!("=== Rust Rule Engine - Built-in Plugin Suite Demo ===\n");

    let kb = KnowledgeBase::new("PluginDemo");
    let mut engine = RustRuleEngine::new(kb);
    let facts = Facts::new();

    // Load all built-in plugins
    println!("🔌 Loading built-in plugins...");
    let string_plugin = StringUtilsPlugin::new();
    let math_plugin = MathUtilsPlugin::new();
    let date_plugin = DateUtilsPlugin::new();
    let validation_plugin = ValidationPlugin::new();
    let collection_plugin = CollectionUtilsPlugin::new();

    // Register plugins
    string_plugin.register_actions(&mut engine)?;
    string_plugin.register_functions(&mut engine)?;

    math_plugin.register_actions(&mut engine)?;
    math_plugin.register_functions(&mut engine)?;

    date_plugin.register_actions(&mut engine)?;
    date_plugin.register_functions(&mut engine)?;

    validation_plugin.register_actions(&mut engine)?;
    validation_plugin.register_functions(&mut engine)?;

    collection_plugin.register_actions(&mut engine)?;
    collection_plugin.register_functions(&mut engine)?;

    println!("✅ All plugins loaded successfully!\n");

    // Demo plugin metadata
    println!("� === PLUGIN INFORMATION ===");
    println!("String Utils Plugin: {}", string_plugin.get_metadata().name);
    println!("  Version: {}", string_plugin.get_metadata().version);
    println!(
        "  Actions: {:?}",
        string_plugin.get_metadata().actions.len()
    );
    println!(
        "  Functions: {:?}",
        string_plugin.get_metadata().functions.len()
    );

    println!("Math Utils Plugin: {}", math_plugin.get_metadata().name);
    println!("  Version: {}", math_plugin.get_metadata().version);
    println!("  Actions: {:?}", math_plugin.get_metadata().actions.len());
    println!(
        "  Functions: {:?}",
        math_plugin.get_metadata().functions.len()
    );

    println!("Date Utils Plugin: {}", date_plugin.get_metadata().name);
    println!("  Version: {}", date_plugin.get_metadata().version);
    println!("  Actions: {:?}", date_plugin.get_metadata().actions.len());
    println!(
        "  Functions: {:?}",
        date_plugin.get_metadata().functions.len()
    );

    println!(
        "Validation Plugin: {}",
        validation_plugin.get_metadata().name
    );
    println!("  Version: {}", validation_plugin.get_metadata().version);
    println!(
        "  Actions: {:?}",
        validation_plugin.get_metadata().actions.len()
    );
    println!(
        "  Functions: {:?}",
        validation_plugin.get_metadata().functions.len()
    );

    println!(
        "Collection Utils Plugin: {}",
        collection_plugin.get_metadata().name
    );
    println!("  Version: {}", collection_plugin.get_metadata().version);
    println!(
        "  Actions: {:?}",
        collection_plugin.get_metadata().actions.len()
    );
    println!(
        "  Functions: {:?}",
        collection_plugin.get_metadata().functions.len()
    );

    // Test function availability
    println!("\n� === FUNCTION AVAILABILITY TEST ===");
    let string_functions = vec!["concat", "repeat", "substring"];
    let math_functions = vec!["max", "min", "sqrt", "sum"];
    let date_functions = vec!["now", "today", "dayOfWeek"];
    let validation_functions = vec!["isEmail", "isPhone", "isUrl"];
    let collection_functions = vec!["length", "contains", "first", "last"];

    println!("String functions registered:");
    for func in &string_functions {
        println!("  {} : {}", func, engine.has_function(func));
    }

    println!("Math functions registered:");
    for func in &math_functions {
        println!("  {} : {}", func, engine.has_function(func));
    }

    println!("Date functions registered:");
    for func in &date_functions {
        println!("  {} : {}", func, engine.has_function(func));
    }

    println!("Validation functions registered:");
    for func in &validation_functions {
        println!("  {} : {}", func, engine.has_function(func));
    }

    println!("Collection functions registered:");
    for func in &collection_functions {
        println!("  {} : {}", func, engine.has_function(func));
    }

    // Test action availability
    println!("\n⚡ === ACTION AVAILABILITY TEST ===");
    let string_actions = vec!["ToUpperCase", "ToLowerCase", "StringTrim"];
    let math_actions = vec!["Add", "Subtract", "Multiply", "Divide"];
    let date_actions = vec!["CurrentDate", "CurrentTime", "AddDays"];
    let validation_actions = vec!["ValidateEmail", "ValidatePhone", "ValidateUrl"];
    let collection_actions = vec!["ArrayLength", "ArrayPush", "ArrayPop"];

    println!("String actions registered:");
    for action in &string_actions {
        println!("  {} : {}", action, engine.has_action_handler(action));
    }

    println!("Math actions registered:");
    for action in &math_actions {
        println!("  {} : {}", action, engine.has_action_handler(action));
    }

    println!("Date actions registered:");
    for action in &date_actions {
        println!("  {} : {}", action, engine.has_action_handler(action));
    }

    println!("Validation actions registered:");
    for action in &validation_actions {
        println!("  {} : {}", action, engine.has_action_handler(action));
    }

    println!("Collection actions registered:");
    for action in &collection_actions {
        println!("  {} : {}", action, engine.has_action_handler(action));
    }

    // Test some basic functions that don't require rule execution
    println!("\n🧪 === BASIC FUNCTION TESTS ===");

    // Test string concat function
    if engine.has_function("concat") {
        // Note: We would need a public API to call functions directly
        // For now, just show that they're registered
        println!("✅ concat function is available");
    }

    // Test math max function
    if engine.has_function("max") {
        println!("✅ max function is available");
    }

    // Test validation isEmail function
    if engine.has_function("isEmail") {
        println!("✅ isEmail function is available");
    }

    // Test collection length function
    if engine.has_function("length") {
        println!("✅ length function is available");
    }

    println!("\n🎉 Built-in Plugin Suite Demo completed successfully!");
    println!("📊 Summary:");
    println!("  - Total plugins loaded: 5");
    println!(
        "  - String utils: {} actions, {} functions",
        string_plugin.get_metadata().actions.len(),
        string_plugin.get_metadata().functions.len()
    );
    println!(
        "  - Math utils: {} actions, {} functions",
        math_plugin.get_metadata().actions.len(),
        math_plugin.get_metadata().functions.len()
    );
    println!(
        "  - Date utils: {} actions, {} functions",
        date_plugin.get_metadata().actions.len(),
        date_plugin.get_metadata().functions.len()
    );
    println!(
        "  - Validation: {} actions, {} functions",
        validation_plugin.get_metadata().actions.len(),
        validation_plugin.get_metadata().functions.len()
    );
    println!(
        "  - Collection utils: {} actions, {} functions",
        collection_plugin.get_metadata().actions.len(),
        collection_plugin.get_metadata().functions.len()
    );

    let total_actions = string_plugin.get_metadata().actions.len()
        + math_plugin.get_metadata().actions.len()
        + date_plugin.get_metadata().actions.len()
        + validation_plugin.get_metadata().actions.len()
        + collection_plugin.get_metadata().actions.len();

    let total_functions = string_plugin.get_metadata().functions.len()
        + math_plugin.get_metadata().functions.len()
        + date_plugin.get_metadata().functions.len()
        + validation_plugin.get_metadata().functions.len()
        + collection_plugin.get_metadata().functions.len();

    println!("  - Total actions available: {}", total_actions);
    println!("  - Total functions available: {}", total_functions);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rule_engine::engine::plugin::RulePlugin;

    #[test]
    fn test_all_plugins_load() -> Result<()> {
        let kb = KnowledgeBase::new("TestKB");
        let mut engine = RustRuleEngine::new(kb);

        // Load all plugins
        let string_plugin = StringUtilsPlugin::new();
        let math_plugin = MathUtilsPlugin::new();
        let date_plugin = DateUtilsPlugin::new();
        let validation_plugin = ValidationPlugin::new();
        let collection_plugin = CollectionUtilsPlugin::new();

        // Register all plugins - should not error
        string_plugin.register_actions(&mut engine)?;
        string_plugin.register_functions(&mut engine)?;
        math_plugin.register_actions(&mut engine)?;
        math_plugin.register_functions(&mut engine)?;
        date_plugin.register_actions(&mut engine)?;
        date_plugin.register_functions(&mut engine)?;
        validation_plugin.register_actions(&mut engine)?;
        validation_plugin.register_functions(&mut engine)?;
        collection_plugin.register_actions(&mut engine)?;
        collection_plugin.register_functions(&mut engine)?;

        // Test that key functions are registered
        assert!(engine.has_function("concat"));
        assert!(engine.has_function("max"));
        assert!(engine.has_function("isEmail"));
        assert!(engine.has_function("length"));

        // Test that key actions are registered
        assert!(engine.has_action_handler("ToUpperCase"));
        assert!(engine.has_action_handler("Add"));
        assert!(engine.has_action_handler("CurrentDate"));
        assert!(engine.has_action_handler("ValidateEmail"));
        assert!(engine.has_action_handler("ArrayLength"));

        Ok(())
    }

    #[test]
    fn test_plugin_metadata() {
        let string_plugin = StringUtilsPlugin::new();
        let metadata = string_plugin.get_metadata();

        assert_eq!(metadata.name, "string_utils");
        assert_eq!(metadata.version, "1.0.0");
        assert!(!metadata.actions.is_empty());
        assert!(!metadata.functions.is_empty());
    }

    #[test]
    fn test_plugin_health() {
        let mut string_plugin = StringUtilsPlugin::new();
        let health = string_plugin.health_check();

        // Should be healthy by default
        assert!(matches!(
            health,
            rust_rule_engine::engine::plugin::PluginHealth::Healthy
        ));
    }
}

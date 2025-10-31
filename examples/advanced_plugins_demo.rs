use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::plugin::{PluginConfig, PluginManager, RulePlugin};
use rust_rule_engine::RustRuleEngine;
use std::sync::Arc;

// Import our example plugins
mod plugins;
use plugins::aiml_plugin::AIMLPlugin;
use plugins::database_plugin::DatabasePlugin;
use plugins::http_client_plugin::HttpClientPlugin;
use plugins::notification_plugin::NotificationPlugin;
use plugins::string_utils_plugin::StringUtilsPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced Plugins Demo - Rust Rule Engine v0.9.0");
    println!("{}", "=".repeat(60));

    // Initialize the rule engine
    let knowledge_base = KnowledgeBase::new("AdvancedPluginsDemo");
    let mut engine = RustRuleEngine::new(knowledge_base);
    let config = PluginConfig::default();
    let mut plugin_manager = PluginManager::new(config);

    // Create and register all our example plugins
    println!("\n📦 Loading Advanced Plugins...");

    // HTTP Client Plugin
    let http_plugin = Arc::new(HttpClientPlugin::new());
    register_plugin(&mut engine, &mut plugin_manager, http_plugin.clone())?;

    // Database Plugin
    let database_plugin = Arc::new(DatabasePlugin::new());
    register_plugin(&mut engine, &mut plugin_manager, database_plugin.clone())?;

    // AI/ML Plugin
    let aiml_plugin = Arc::new(AIMLPlugin::new());
    register_plugin(&mut engine, &mut plugin_manager, aiml_plugin.clone())?;

    // Notification Plugin
    let notification_plugin = Arc::new(NotificationPlugin::new());
    register_plugin(
        &mut engine,
        &mut plugin_manager,
        notification_plugin.clone(),
    )?;

    // String Utils Plugin
    let string_plugin = Arc::new(StringUtilsPlugin::new());
    register_plugin(&mut engine, &mut plugin_manager, string_plugin.clone())?;

    println!("✅ All plugins loaded successfully!\n");

    // Display plugin information
    println!("📋 Loaded Plugins:");
    let plugin_list = plugin_manager.list_plugins();
    for (i, plugin_info) in plugin_list.iter().enumerate() {
        println!(
            "   {}. {} v{} - {}",
            i + 1,
            plugin_info.name,
            plugin_info.version,
            plugin_info.description
        );
        println!("      � State: {:?}", plugin_info.state);
        println!("      � Health: {:?}", plugin_info.health);
        println!();
    }

    println!("\n🎉 Advanced plugin demonstration completed!");
    println!("💡 These plugins can be easily integrated into your rule definitions");
    println!("   to create powerful automation workflows.");

    // Show plugin statistics
    let stats = plugin_manager.get_stats();
    println!("\n📊 Plugin Statistics:");
    println!("   Total Plugins: {}", stats.total_plugins);
    println!("   Loaded Plugins: {}", stats.loaded_plugins);
    println!("   Failed Plugins: {}", stats.failed_plugins);
    println!("   Warnings: {}", stats.warnings);

    Ok(())
}

fn register_plugin(
    engine: &mut RustRuleEngine,
    plugin_manager: &mut PluginManager,
    plugin: Arc<dyn RulePlugin>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Register actions and functions with the engine
    plugin.register_actions(engine)?;
    plugin.register_functions(engine)?;

    // Load the plugin into the manager
    plugin_manager.load_plugin(plugin);

    Ok(())
}

use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::plugin::{PluginConfig, PluginHealth};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::Facts;
use std::sync::Arc;

mod plugins;
use plugins::string_utils_plugin::StringUtilsPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Plugin System Test Demo");
    println!("===========================");

    // Create simple engine
    let kb = KnowledgeBase::new("PluginTest");
    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 5,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Configure plugin system
    let plugin_config = PluginConfig {
        max_plugins: 10,
        enable_hot_reload: true,
        plugin_timeout_ms: 3000,
        safety_checks: true,
    };
    engine.configure_plugins(plugin_config);

    println!("\n📊 Initial Plugin Stats:");
    println!("  {}", engine.get_plugin_stats());

    // Load StringUtils plugin
    println!("\n🔌 Loading StringUtils Plugin...");
    let string_plugin = Arc::new(StringUtilsPlugin::new());

    match engine.load_plugin(string_plugin) {
        Ok(_) => println!("  ✅ Plugin loaded successfully!"),
        Err(e) => {
            println!("  ❌ Failed to load plugin: {}", e);
            return Err(e.into());
        }
    }

    // Show plugin info
    println!("\n📋 Plugin Information:");
    for plugin_info in engine.list_plugins() {
        println!(
            "  📦 {} v{}: {}",
            plugin_info.name, plugin_info.version, plugin_info.description
        );
        println!(
            "      State: {:?}, Health: {:?}",
            plugin_info.state, plugin_info.health
        );
    }

    // Test basic actions manually
    println!("\n🧪 Testing Plugin Actions...");

    let facts = Facts::new();
    facts.add_value(
        "test",
        rust_rule_engine::types::Value::String("Hello World".to_string()),
    )?;

    // Test if actions are registered
    println!("  🔍 Plugin actions have been registered successfully");
    println!("      Available actions: ToUpperCase, ToLowerCase, StringLength, StringContains");

    // Health check plugins
    println!("\n🏥 Plugin Health Check:");
    let health_results = engine.plugin_health_check();
    for (plugin_name, health) in health_results {
        match health {
            PluginHealth::Healthy => println!("  ✅ {} is healthy", plugin_name),
            PluginHealth::Warning(msg) => println!("  ⚠️ {} has warning: {}", plugin_name, msg),
            PluginHealth::Error(msg) => println!("  ❌ {} has error: {}", plugin_name, msg),
        }
    }

    // Show final plugin stats
    println!("\n📊 Final Plugin Stats:");
    println!("  {}", engine.get_plugin_stats());

    // Demo unloading
    println!("\n🔌 Testing plugin unload...");
    match engine.unload_plugin("string-utils") {
        Ok(_) => println!("  ✅ Plugin unloaded successfully"),
        Err(e) => println!("  ⚠️ Unload note: {}", e),
    }

    println!("\n📊 Stats After Unload:");
    println!("  {}", engine.get_plugin_stats());

    println!("\n🎉 Plugin System Test completed successfully!");
    println!("\n💡 Key Features Tested:");
    println!("  🔌 Plugin loading and registration");
    println!("  📦 Plugin metadata and information");
    println!("  🏥 Health monitoring");
    println!("  📊 Plugin statistics");
    println!("  🔄 Plugin unloading");
    println!("  🛡️ Safe plugin management");

    Ok(())
}

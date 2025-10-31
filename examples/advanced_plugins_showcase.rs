use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::plugin::{PluginManager, RulePlugin};
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

    // Create plugin manager with disabled safety checks for demo
    let config = rust_rule_engine::engine::plugin::PluginConfig {
        max_plugins: 50,
        enable_hot_reload: true,
        plugin_timeout_ms: 5000,
        safety_checks: false, // Disable dependency validation for demo
    };
    let mut plugin_manager = PluginManager::new(config);

    // Create and register all our example plugins
    println!("\n📦 Loading Advanced Plugins...");

    let http_plugin = Arc::new(HttpClientPlugin::new());
    let database_plugin = Arc::new(DatabasePlugin::new());
    let aiml_plugin = Arc::new(AIMLPlugin::new());
    let notification_plugin = Arc::new(NotificationPlugin::new());
    let string_plugin = Arc::new(StringUtilsPlugin::new());

    // Register plugins with engine and plugin manager
    register_plugin(&mut engine, &mut plugin_manager, http_plugin.clone())?;
    register_plugin(&mut engine, &mut plugin_manager, database_plugin.clone())?;
    register_plugin(&mut engine, &mut plugin_manager, aiml_plugin.clone())?;
    register_plugin(
        &mut engine,
        &mut plugin_manager,
        notification_plugin.clone(),
    )?;
    register_plugin(&mut engine, &mut plugin_manager, string_plugin.clone())?;

    println!("✅ All plugins loaded successfully!\n");

    // Display plugin information
    println!("📋 Loaded Plugins:");
    for (i, plugin_info) in plugin_manager.list_plugins().iter().enumerate() {
        println!("{}. {} v{}", i + 1, plugin_info.name, plugin_info.version);
        println!("   Description: {}", plugin_info.description);
        println!(
            "   State: {:?} | Health: {:?}",
            plugin_info.state, plugin_info.health
        );
    }

    // Display detailed plugin capabilities
    println!("\n📋 Plugin Capabilities:");
    display_plugin_capabilities(http_plugin.as_ref());
    display_plugin_capabilities(database_plugin.as_ref());
    display_plugin_capabilities(aiml_plugin.as_ref());
    display_plugin_capabilities(notification_plugin.as_ref());
    display_plugin_capabilities(string_plugin.as_ref());

    // Show how these plugins could be used in real scenarios
    println!("\n🎯 Real-World Use Case Examples:");
    println!("{}", "=".repeat(60));

    demo_ecommerce_scenario();
    demo_content_moderation_scenario();
    demo_system_monitoring_scenario();
    demo_customer_support_scenario();

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
    // Register actions and functions with engine
    plugin.register_actions(engine)?;
    plugin.register_functions(engine)?;

    // Load plugin into manager
    plugin_manager.load_plugin(plugin)?;

    Ok(())
}

fn display_plugin_capabilities(plugin: &dyn RulePlugin) {
    let metadata = plugin.get_metadata();
    println!("\n🔌 {} v{}", metadata.name, metadata.version);
    println!("   📝 {}", metadata.description);
    println!("   👤 Author: {}", metadata.author);

    if !metadata.actions.is_empty() {
        println!("   ⚡ Actions ({}):", metadata.actions.len());
        for (i, action) in metadata.actions.iter().enumerate() {
            println!("      {}. {}", i + 1, action);
        }
    }

    if !metadata.functions.is_empty() {
        println!("   🔧 Functions ({}):", metadata.functions.len());
        for (i, function) in metadata.functions.iter().enumerate() {
            println!("      {}. {}", i + 1, function);
        }
    }

    if !metadata.dependencies.is_empty() {
        println!("   📦 Dependencies: {}", metadata.dependencies.join(", "));
    }
}

fn demo_ecommerce_scenario() {
    println!("\n🛒 Scenario 1: E-commerce Order Processing");
    println!("{}", "-".repeat(50));
    println!("When a high-value order is placed:");
    println!("  1. 📧 validateEmail() - Verify customer email format");
    println!("  2. 🤖 PredictValue(fraud_detection) - Run ML fraud detection");
    println!("  3. 💾 ExecuteSQL() - Check product inventory in database");
    println!("  4. 🌐 HttpPost() - Process payment via payment gateway");
    println!("  5. 📧 SendEmail() - Send order confirmation to customer");
    println!("  6. 💾 InsertRecord() - Create order record in database");
    println!("  7. 🪝 SendWebhook() - Notify fulfillment system");
    println!("  Result: ✅ Fully automated order processing with fraud protection");
}

fn demo_content_moderation_scenario() {
    println!("\n🛡️  Scenario 2: Content Moderation & AI Analysis");
    println!("{}", "-".repeat(50));
    println!("When user-generated content is submitted:");
    println!("  1. 😊 SentimentAnalysis() - Analyze emotional tone");
    println!("  2. 🏷️  ClassifyText() - Categorize content type");
    println!("  3. 🔍 extractKeywords() - Extract important terms");
    println!("  4. 📊 calculateSimilarity() - Compare with known spam");
    println!("  5. 💾 InsertRecord() - Store analysis results");
    println!("  6. 🚨 CreateAlert() - Generate moderation alerts");
    println!("  7. 💬 SendSlackMessage() - Notify moderation team");
    println!("  Result: ✅ AI-powered content moderation with real-time alerts");
}

fn demo_system_monitoring_scenario() {
    println!("\n📊 Scenario 3: System Monitoring & Alerting");
    println!("{}", "-".repeat(50));
    println!("When system metrics exceed thresholds:");
    println!("  1. 🌐 HttpGet() - Fetch system health status");
    println!("  2. 💾 InsertRecord() - Store metrics in database");
    println!("  3. 🚨 DetectAnomalies() - ML-based anomaly detection");
    println!("  4. 📊 aggregateData() - Compare with historical averages");
    println!("  5. 🔴 RedisSet() - Cache current metrics for fast access");
    println!("  6. 🚨 CreateAlert() - Generate high-priority system alert");
    println!("  7. 📱 SendSMS() - Immediate SMS to on-call engineer");
    println!("  8. 📧 SendEmail() - Detailed report to DevOps team");
    println!("  Result: ✅ Proactive system monitoring with multi-channel alerts");
}

fn demo_customer_support_scenario() {
    println!("\n🎧 Scenario 4: Customer Support Automation");
    println!("{}", "-".repeat(50));
    println!("When a support ticket is received:");
    println!("  1. 🏷️  ClassifyText() - Categorize the support request");
    println!("  2. 🔍 extractKeywords() - Identify key issues");
    println!("  3. 💾 ExecuteSQL() - Look up customer information");
    println!("  4. 🤖 ChatCompletion() - Generate AI-powered response");
    println!("  5. 🍃 MongoFind() - Find similar resolved tickets");
    println!("  6. 💾 InsertRecord() - Create support ticket record");
    println!("  7. 📧 SendEmail() - Auto-response to customer");
    println!("  8. 💬 SendSlackMessage() - Notify support team");
    println!("  9. ⏰ ScheduleNotification() - Schedule follow-up");
    println!("  10. 🎯 RecommendItems() - Suggest helpful articles");
    println!("  Result: ✅ Intelligent support automation with context awareness");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registration() {
        let knowledge_base = KnowledgeBase::new("TestPlugins");
        let mut engine = RustRuleEngine::new(knowledge_base);
        let mut plugin_manager = PluginManager::with_default_config();

        let http_plugin = Arc::new(HttpClientPlugin::new());

        // Test plugin registration
        assert!(register_plugin(&mut engine, &mut plugin_manager, http_plugin).is_ok());

        // Test that actions and functions are registered
        assert!(engine.has_action_handler("HttpGet"));
        assert!(engine.has_action_handler("SendWebhook"));
        assert!(engine.has_function("apiCall"));
        assert!(engine.has_function("parseJson"));

        // Test plugin manager
        let plugins = plugin_manager.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "http-client");
    }

    #[test]
    fn test_all_plugins_load() {
        let knowledge_base = KnowledgeBase::new("TestAllPlugins");
        let mut engine = RustRuleEngine::new(knowledge_base);
        let mut plugin_manager = PluginManager::with_default_config();

        // Create all plugins
        let plugins: Vec<Arc<dyn RulePlugin>> = vec![
            Arc::new(HttpClientPlugin::new()),
            Arc::new(DatabasePlugin::new()),
            Arc::new(AIMLPlugin::new()),
            Arc::new(NotificationPlugin::new()),
            Arc::new(StringUtilsPlugin::new()),
        ];

        // Register all plugins
        for plugin in plugins {
            assert!(register_plugin(&mut engine, &mut plugin_manager, plugin).is_ok());
        }

        // Check all plugins are loaded
        let loaded_plugins = plugin_manager.list_plugins();
        assert_eq!(loaded_plugins.len(), 5);

        // Test some key actions/functions from each plugin
        assert!(engine.has_action_handler("HttpGet")); // HTTP
        assert!(engine.has_action_handler("ExecuteSQL")); // Database
        assert!(engine.has_action_handler("PredictValue")); // AI/ML
        assert!(engine.has_action_handler("SendEmail")); // Notification
        assert!(engine.has_action_handler("ToUpperCase")); // String Utils

        assert!(engine.has_function("apiCall")); // HTTP
        assert!(engine.has_function("sqlQuery")); // Database
        assert!(engine.has_function("calculateSimilarity")); // AI/ML
        assert!(engine.has_function("formatMessage")); // Notification
        assert!(engine.has_function("concatenate")); // String Utils
    }
}

use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::plugin::{PluginConfig, PluginHealth};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use rust_rule_engine::Facts;
use std::collections::HashMap;
use std::sync::Arc;

mod plugins;
use plugins::string_utils_plugin::StringUtilsPlugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 Plugin System Demo v0.9.0");
    println!("==============================");

    // Create engine with plugin support
    let kb = KnowledgeBase::new("PluginDemo");
    let config = EngineConfig {
        debug_mode: true,
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
    engine.load_plugin(string_plugin)?;

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

    // Create rules that use plugin actions
    let plugin_rules = [
        r#"
        rule "ProcessUserInput" salience 20 {
            when
                User.name != "" && User.message != ""
            then
                ToUpperCase(User.name, "User.nameUpper");
                ToLowerCase(User.email, "User.emailLower"); 
                StringLength(User.message, "User.messageLength");
                StringContains(User.message, "Hello", "User.isGreeting");
                log("User input processed with plugins");
        }
        "#,
        r#"
        rule "StringValidation" salience 15 {
            when
                User.messageLength > 10 && User.isGreeting == true
            then
                log("Valid greeting message detected");
                User.status = "validated";
        }
        "#,
    ];

    // Parse and add rules
    for (i, rule_grl) in plugin_rules.iter().enumerate() {
        let rules = GRLParser::parse_rules(rule_grl)?;
        for rule in rules {
            engine.knowledge_base_mut().add_rule(rule)?;
            println!("  ✅ Added rule {}", i + 1);
        }
    }

    // Create test facts
    println!("\n📊 Setting up test facts...");
    let facts = Facts::new();

    let mut user = HashMap::new();
    user.insert("name".to_string(), Value::String("John Doe".to_string()));
    user.insert(
        "email".to_string(),
        Value::String("JOHN.DOE@EXAMPLE.COM".to_string()),
    );
    user.insert(
        "message".to_string(),
        Value::String("Hello there! How are you doing today?".to_string()),
    );
    user.insert("status".to_string(), Value::String("pending".to_string()));

    facts.add_value("User", Value::Object(user))?;

    println!("  📋 User.name = \"John Doe\"");
    println!("  📋 User.email = \"JOHN.DOE@EXAMPLE.COM\"");
    println!("  📋 User.message = \"Hello there! How are you doing today?\"");

    // Execute rules with plugin actions
    println!("\n🚀 Executing rules with plugin actions...");
    let result = engine.execute(&facts)?;

    println!("\n📊 Execution Results:");
    println!("  Rules fired: {}", result.rules_fired);
    println!("  Cycles: {}", result.cycle_count);
    println!("  Duration: {:?}", result.execution_time);

    // Show processed results
    println!("\n📋 Processed Results:");
    if let Some(name_upper) = facts.get("User.nameUpper") {
        println!("  📤 Uppercase name: {:?}", name_upper);
    }
    if let Some(email_lower) = facts.get("User.emailLower") {
        println!("  📤 Lowercase email: {:?}", email_lower);
    }
    if let Some(length) = facts.get("User.messageLength") {
        println!("  📏 Message length: {:?}", length);
    }
    if let Some(is_greeting) = facts.get("User.isGreeting") {
        println!("  👋 Is greeting: {:?}", is_greeting);
    }
    if let Some(status) = facts.get("User.status") {
        println!("  ✅ Status: {:?}", status);
    }

    // Test plugin functions in rules
    println!("\n🧪 Testing plugin functions...");
    engine.register_function("testConcatFunction", |_args, facts| {
        // Use plugin function
        facts.set_nested(
            "Test.concatenated",
            Value::String("This is a test of plugin functions".to_string()),
        )?;
        Ok(Value::Boolean(true))
    });

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

    // Demo hot reload (simulate)
    println!("\n🔄 Simulating hot reload...");
    let new_string_plugin = Arc::new(StringUtilsPlugin::new());
    engine.hot_reload_plugin("string-utils", new_string_plugin)?;

    // Demo unloading
    println!("\n🔌 Unloading plugin...");
    engine.unload_plugin("string-utils")?;

    println!("\n📊 Final Plugin Stats After Unload:");
    println!("  {}", engine.get_plugin_stats());

    println!("\n🎉 Plugin System Demo completed successfully!");
    println!("\n💡 Key Features Demonstrated:");
    println!("  🔌 Plugin loading and management");
    println!("  🎯 Custom actions from plugins");
    println!("  🔧 Plugin functions in rules");
    println!("  🏥 Health monitoring");
    println!("  🔄 Hot reloading");
    println!("  📊 Plugin statistics");
    println!("  🛡️ Safe plugin isolation");

    Ok(())
}

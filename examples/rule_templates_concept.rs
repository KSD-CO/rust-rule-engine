use rust_rule_engine::engine::{ParameterType, RuleTemplate, TemplateManager};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Rule Templates Concept Demo");
    println!("===============================");

    // Demo 1: Template Creation and Parameter Substitution
    demo_template_creation()?;

    // Demo 2: Template Manager
    demo_template_manager()?;

    // Demo 3: Parameter Validation
    demo_parameter_validation()?;

    println!("\n✅ Rule Templates concept demonstrated successfully!");
    println!("📋 Key Features:");
    println!("   - ✅ Template creation with parameters");
    println!("   - ✅ Parameter substitution");
    println!("   - ✅ JSON serialization/deserialization");
    println!("   - ✅ Template management");
    println!("   - ✅ Parameter validation");
    println!("   - ⚠️  GRL generation (needs parser fixes for complex rules)");

    Ok(())
}

fn demo_template_creation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 1: Template Creation & Parameter Substitution");
    println!("----------------------------------------------------");

    // Create a VIP status template
    let vip_template = RuleTemplate::new("VIPStatusCheck")
        .with_description("Check if user qualifies for VIP status based on country and spending")
        .with_parameter("country", ParameterType::String)
        .with_parameter("threshold", ParameterType::Number)
        .with_parameter("vip_level", ParameterType::String)
        .with_condition("User.Country == \"{{country}}\" && User.SpendingTotal >= {{threshold}}")
        .with_action("User.setVIPLevel(\"{{vip_level}}\")");

    println!("✅ Created VIP template with parameters:");
    println!("   - country: String");
    println!("   - threshold: Number");
    println!("   - vip_level: String");

    // Test parameter substitution
    let params = {
        let mut params = HashMap::new();
        params.insert("country".to_string(), "US".to_string());
        params.insert("threshold".to_string(), "1000".to_string());
        params.insert("vip_level".to_string(), "Gold".to_string());
        params
    };

    let condition_result =
        vip_template.substitute_placeholders(&vip_template.condition_template, &params);

    let action_result =
        vip_template.substitute_placeholders(&vip_template.action_template, &params);

    println!("✅ Parameter substitution example:");
    println!("   Condition Template: {}", vip_template.condition_template);
    println!("   Condition Result:   {}", condition_result);
    println!("   Action Template:    {}", vip_template.action_template);
    println!("   Action Result:      {}", action_result);

    Ok(())
}

fn demo_template_manager() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📁 Demo 2: Template Manager & JSON Serialization");
    println!("------------------------------------------------");

    let mut manager = TemplateManager::new();

    // Create multiple templates
    let age_template = RuleTemplate::new("AgeBasedRule")
        .with_description("Rules based on user age")
        .with_parameter("min_age", ParameterType::Number)
        .with_parameter("action", ParameterType::String)
        .with_condition("User.Age >= {{min_age}}")
        .with_action("{{action}}");

    let location_template = RuleTemplate::new("LocationBasedRule")
        .with_description("Rules based on user location")
        .with_parameter("country", ParameterType::String)
        .with_parameter("region", ParameterType::String)
        .with_parameter("bonus", ParameterType::Number)
        .with_condition("User.Country == \"{{country}}\" && User.Region == \"{{region}}\"")
        .with_action("User.addBonus({{bonus}})");

    manager.register_template(age_template);
    manager.register_template(location_template);

    println!("✅ Registered {} templates", manager.list_templates().len());
    println!("   Templates: {:?}", manager.list_templates());

    // Serialize to JSON
    let json = manager.save_to_json()?;
    println!("\n📄 Templates serialized to JSON (first 200 chars):");
    let preview = if json.len() > 200 {
        format!("{}...", &json[..200])
    } else {
        json.clone()
    };
    println!("{}", preview);

    // Test loading from JSON
    let mut new_manager = TemplateManager::new();
    new_manager.load_from_json(&json)?;

    println!(
        "✅ Successfully loaded {} templates from JSON",
        new_manager.list_templates().len()
    );

    Ok(())
}

fn demo_parameter_validation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Demo 3: Parameter Validation");
    println!("-------------------------------");

    // Create a template with required parameters
    let security_template = RuleTemplate::new("SecurityAlert")
        .with_parameter("event_type", ParameterType::String)
        .with_parameter("severity", ParameterType::String)
        .with_parameter("threshold", ParameterType::Number)
        .with_condition(
            "SecurityEvent.Type == \"{{event_type}}\" && SecurityEvent.Count >= {{threshold}}",
        )
        .with_action("Alert.create(\"{{severity}}\", \"Security event detected\")");

    println!("✅ Created security template with required parameters:");
    for param in &security_template.parameters {
        println!("   - {}: {:?}", param.name, param.param_type);
    }

    // Test validation with complete parameters
    let complete_params = {
        let mut params = HashMap::new();
        params.insert("event_type".to_string(), "failed_login".to_string());
        params.insert("severity".to_string(), "HIGH".to_string());
        params.insert("threshold".to_string(), "5".to_string());
        params
    };

    match security_template.validate_parameters(&complete_params) {
        Ok(_) => println!("✅ Validation passed with complete parameters"),
        Err(e) => println!("❌ Validation failed: {}", e),
    }

    // Test validation with missing parameters
    let incomplete_params = {
        let mut params = HashMap::new();
        params.insert("event_type".to_string(), "failed_login".to_string());
        // Missing severity and threshold
        params
    };

    match security_template.validate_parameters(&incomplete_params) {
        Ok(_) => println!("✅ Validation passed (unexpected)"),
        Err(e) => println!("✅ Validation correctly failed: {}", e),
    }

    // Test bulk configuration generation
    let security_configs = vec![
        ("Failed_Login_Alert".to_string(), {
            let mut params = HashMap::new();
            params.insert("event_type".to_string(), "failed_login".to_string());
            params.insert("severity".to_string(), "HIGH".to_string());
            params.insert("threshold".to_string(), "5".to_string());
            params
        }),
        ("Suspicious_Access_Alert".to_string(), {
            let mut params = HashMap::new();
            params.insert("event_type".to_string(), "suspicious_access".to_string());
            params.insert("severity".to_string(), "CRITICAL".to_string());
            params.insert("threshold".to_string(), "3".to_string());
            params
        }),
    ];

    println!(
        "✅ Generated {} security rule configurations",
        security_configs.len()
    );
    for (name, params) in &security_configs {
        println!("   - {}: {} parameters", name, params.len());
    }

    Ok(())
}

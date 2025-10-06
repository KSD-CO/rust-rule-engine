use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::engine::{ParameterType, RuleTemplate, TemplateManager};
use rust_rule_engine::types::Value;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Rule Templates Concept Demo");
    println!("===============================");

    // Demo 1: VIP Template
    demo_vip_template()?;

    // Demo 2: Discount Template
    demo_discount_template()?;

    // Demo 3: Template Manager
    demo_template_manager()?;

    // Demo 4: Bulk Rule Generation
    demo_bulk_rule_generation()?;

    println!("\n✅ Rule Templates concept demonstrated successfully!");
    println!("📋 Key Features:");
    println!("   - ✅ VIP customer template");
    println!("   - ✅ Discount template");
    println!("   - ✅ Template management");
    println!("   - ✅ Bulk rule generation");
    println!("   - ✅ Parameter substitution");
    println!("   - ✅ JSON serialization/deserialization");

    Ok(())
}

fn demo_vip_template() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 1: VIP Status Template");
    println!("------------------------------");

    // Create a VIP status template
    let vip_template = RuleTemplate::new("VIPStatusCheck")
        .with_description("Check if user qualifies for VIP status based on country and spending")
        .with_parameter("country", ParameterType::String)
        .with_parameter("threshold", ParameterType::Number)
        .with_parameter("vip_level", ParameterType::String)
        .with_condition("User.Country == \"{{country}}\"")
        .with_action("User.setIsVIP(\"{{vip_level}}\")");

    println!("✅ Created VIP template with parameters: country, threshold, vip_level");

    // Generate rules for different countries
    let us_rule = vip_template
        .instantiate("VIP_US_Gold")
        .with_param("country", "US")
        .with_param("threshold", "1000")
        .with_param("vip_level", "Gold")
        .build()?;

    let uk_rule = vip_template
        .instantiate("VIP_UK_Silver")
        .with_param("country", "UK")
        .with_param("threshold", "800")
        .with_param("vip_level", "Silver")
        .build()?;

    let jp_rule = vip_template
        .instantiate("VIP_JP_Platinum")
        .with_param("country", "JP")
        .with_param("threshold", "1500")
        .with_param("vip_level", "Platinum")
        .build()?;

    println!("✅ Generated 3 rules from template:");
    println!("   - {} (threshold: $1000)", us_rule.name);
    println!("   - {} (threshold: $800)", uk_rule.name);
    println!("   - {} (threshold: $1500)", jp_rule.name);

    // Test the generated rules
    let kb = KnowledgeBase::new("VIPTest");
    kb.add_rule(us_rule)?;
    kb.add_rule(uk_rule)?;
    kb.add_rule(jp_rule)?;

    let config = EngineConfig {
        debug_mode: true,
        max_cycles: 3,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register custom functions
    engine.register_function(
        "setIsVIP",
        Box::new(|args: &[Value], _facts: &Facts| {
            if let Some(Value::Boolean(is_vip)) = args.first() {
                println!(
                    "  🔧 Called setIsVIP([{}]) -> Set IsVIP to {}",
                    is_vip, is_vip
                );
                Ok(Value::Boolean(*is_vip))
            } else {
                Ok(Value::Null)
            }
        }),
    );

    engine.register_function(
        "setVIPLevel",
        Box::new(|args: &[Value], _facts: &Facts| {
            if let Some(Value::String(level)) = args.first() {
                println!(
                    "  🔧 Called setVIPLevel([{}]) -> Set VIPLevel to {}",
                    level, level
                );
                Ok(Value::String(level.clone()))
            } else {
                Ok(Value::Null)
            }
        }),
    );

    // Test US user
    let facts = Facts::new();
    facts.set("User", {
        let mut user = HashMap::new();
        user.insert("Country".to_string(), Value::String("US".to_string()));
        user.insert("SpendingTotal".to_string(), Value::Number(1200.0));
        user.insert("IsVIP".to_string(), Value::Boolean(false));
        user.insert("VIPLevel".to_string(), Value::String("None".to_string()));
        Value::Object(user)
    });

    println!("\n🧪 Testing US user with $1200 spending:");
    let result = engine.execute(&facts)?;
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Execution time: {:?}", result.execution_time);

    if let Some(Value::Object(user)) = facts.get("User") {
        if let Some(Value::String(level)) = user.get("VIPLevel") {
            println!("   VIP Level: {}", level);
        }
    }

    Ok(())
}

fn demo_discount_template() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🏷️ Demo 2: Discount Template");
    println!("----------------------------");

    // Create a discount template with multiple conditions
    let discount_template = RuleTemplate::new("ProductDiscount")
        .with_description("Apply discount based on product category and customer status")
        .with_parameter("category", ParameterType::String)
        .with_parameter("customer_type", ParameterType::String)
        .with_parameter("discount_percent", ParameterType::Number)
        .with_parameter("min_amount", ParameterType::Number)
        .with_condition("Order.Category == \"{{category}}\" && Customer.Type == \"{{customer_type}}\" && Order.Amount >= {{min_amount}}")
        .with_action("Order.setDiscountPercent({{discount_percent}})")
        .with_salience(15);

    println!("✅ Created discount template for category-based promotions");

    // Generate specific discount rules
    let electronics_vip = discount_template
        .instantiate("Electronics_VIP_Discount")
        .with_param("category", "Electronics")
        .with_param("customer_type", "VIP")
        .with_param("discount_percent", "25")
        .with_param("min_amount", "500")
        .build()?;

    let books_student = discount_template
        .instantiate("Books_Student_Discount")
        .with_param("category", "Books")
        .with_param("customer_type", "Student")
        .with_param("discount_percent", "15")
        .with_param("min_amount", "50")
        .build()?;

    println!("✅ Generated discount rules:");
    println!(
        "   - {} (25% for VIP on Electronics $500+)",
        electronics_vip.name
    );
    println!(
        "   - {} (15% for Students on Books $50+)",
        books_student.name
    );

    Ok(())
}

fn demo_template_manager() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📁 Demo 3: Template Manager & JSON Serialization");
    println!("------------------------------------------------");

    let mut manager = TemplateManager::new();

    // Create and register multiple templates
    let age_template = RuleTemplate::new("AgeBasedRule")
        .with_parameter("min_age", ParameterType::Number)
        .with_parameter("action", ParameterType::String)
        .with_condition("User.Age >= {{min_age}}")
        .with_action("{{action}}")
        .with_salience(10);

    let location_template = RuleTemplate::new("LocationBasedRule")
        .with_parameter("country", ParameterType::String)
        .with_parameter("region", ParameterType::String)
        .with_parameter("bonus", ParameterType::Number)
        .with_condition("User.Country == \"{{country}}\" && User.Region == \"{{region}}\"")
        .with_action("User.addBonus({{bonus}}); log(\"Location bonus applied\")")
        .with_salience(5);

    manager.register_template(age_template);
    manager.register_template(location_template);

    println!("✅ Registered {} templates", manager.list_templates().len());
    println!("   Templates: {:?}", manager.list_templates());

    // Serialize to JSON
    let json = manager.save_to_json()?;
    println!("\n📄 Templates serialized to JSON:");
    println!("{}", json);

    // Test loading from JSON
    let mut new_manager = TemplateManager::new();
    new_manager.load_from_json(&json)?;

    println!(
        "✅ Successfully loaded {} templates from JSON",
        new_manager.list_templates().len()
    );

    Ok(())
}

fn demo_bulk_rule_generation() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Demo 4: Bulk Rule Generation");
    println!("-------------------------------");

    let mut manager = TemplateManager::new();

    // Create a security alert template
    let security_template = RuleTemplate::new("SecurityAlert")
        .with_parameter("event_type", ParameterType::String)
        .with_parameter("severity", ParameterType::String)
        .with_parameter("threshold", ParameterType::Number)
        .with_condition(
            "SecurityEvent.Type == \"{{event_type}}\" && SecurityEvent.Count >= {{threshold}}",
        )
        .with_action("Alert.create(\"{{severity}}\", \"Multiple {{event_type}} events detected\")")
        .with_salience(100);

    manager.register_template(security_template);

    // Define multiple configurations for bulk generation
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
        ("Rate_Limit_Alert".to_string(), {
            let mut params = HashMap::new();
            params.insert("event_type".to_string(), "rate_limit_exceeded".to_string());
            params.insert("severity".to_string(), "MEDIUM".to_string());
            params.insert("threshold".to_string(), "10".to_string());
            params
        }),
    ];

    // Generate all security rules at once
    let security_rules = manager.generate_rules("SecurityAlert", security_configs)?;

    println!(
        "✅ Generated {} security alert rules from single template:",
        security_rules.len()
    );
    for rule in &security_rules {
        println!("   - {} (salience: {:?})", rule.name, rule.salience);
    }

    // Add to knowledge base and test
    let kb = KnowledgeBase::new("SecurityRules");
    for rule in security_rules {
        kb.add_rule(rule)?;
    }

    println!("✅ All security rules added to knowledge base");
    println!("   Total rules in KB: {}", kb.get_statistics().total_rules);

    Ok(())
}

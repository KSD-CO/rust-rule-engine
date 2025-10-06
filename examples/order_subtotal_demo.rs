use rust_rule_engine::*;
use std::collections::HashMap;

/// 🛒 E-commerce Order Processing with SubTotal Calculations
/// 
/// This example demonstrates:
/// 1. Product-level SubTotal calculation: SubTotal = Price * Quantity
/// 2. Order-level SubTotal calculation: Sum of all product SubTotals
/// 3. Distributed processing across different rule nodes

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    println!("🛒 === E-commerce Order Processing Demo ===");
    println!("Demonstrating SubTotal calculations across product list and order");

    // Create distributed nodes for different calculation phases
    let mut product_calc_node = create_node("ProductCalculator", "Individual product SubTotal calculations")?;
    let mut order_calc_node = create_node("OrderCalculator", "Order-level aggregation calculations")?;
    let mut validation_node = create_node("ValidationNode", "Order validation and business rules")?;

    // Create order with multiple products
    let order_facts = create_order_with_products()?;

    println!("\n📦 Initial Order Data:");
    display_order_details(&order_facts);

    // Phase 1: Calculate SubTotal for each product
    println!("\n🔨 Phase 1: Product-level SubTotal calculations...");
    add_product_calculation_rules(&mut product_calc_node)?;
    let result1 = product_calc_node.execute(&order_facts)?;
    println!("   ✅ {} product calculation rules fired", result1.rules_fired);

    // Phase 2: Calculate order total
    println!("\n🔨 Phase 2: Order-level SubTotal aggregation...");
    add_order_calculation_rules(&mut order_calc_node)?;
    let result2 = order_calc_node.execute(&order_facts)?;
    println!("   ✅ {} order calculation rules fired", result2.rules_fired);

    // Phase 3: Apply business rules and validation
    println!("\n🔨 Phase 3: Business rules and validation...");
    add_business_validation_rules(&mut validation_node)?;
    let result3 = validation_node.execute(&order_facts)?;
    println!("   ✅ {} validation rules fired", result3.rules_fired);

    println!("\n📊 Final Order After Processing:");
    display_order_details(&order_facts);

    println!("\n📈 Processing Summary:");
    println!("   Product calculations: {} rules", result1.rules_fired);
    println!("   Order aggregations: {} rules", result2.rules_fired);
    println!("   Business validations: {} rules", result3.rules_fired);
    println!("   Total distributed processing: {} rules across 3 nodes", 
        result1.rules_fired + result2.rules_fired + result3.rules_fired);

    println!("\n🎯 Distributed Benefits:");
    println!("   🧮 Specialized Calculations: Each node handles specific math operations");
    println!("   📊 Parallel Processing: Product calculations can run in parallel");
    println!("   🔄 Sequential Dependencies: Order total depends on product subtotals");
    println!("   ✅ Validation Pipeline: Business rules applied after calculations");

    Ok(())
}

fn create_node(name: &str, description: &str) -> std::result::Result<RustRuleEngine, Box<dyn std::error::Error>> {
    println!("🏗️ Creating {}: {}", name, description);
    
    let kb = KnowledgeBase::new(name);
    let config = EngineConfig {
        max_cycles: 5,
        debug_mode: true,
        enable_stats: true,
        ..Default::default()
    };
    
    Ok(RustRuleEngine::with_config(kb, config))
}

fn create_order_with_products() -> std::result::Result<Facts, Box<dyn std::error::Error>> {
    let facts = Facts::new();

    // Create Order object
    let mut order_props = HashMap::new();
    order_props.insert("OrderID".to_string(), Value::String("ORD-2025-001".to_string()));
    order_props.insert("CustomerID".to_string(), Value::String("CUST-12345".to_string()));
    order_props.insert("Status".to_string(), Value::String("PROCESSING".to_string()));
    order_props.insert("SubTotal".to_string(), Value::Number(0.0)); // Will be calculated
    order_props.insert("Tax".to_string(), Value::Number(0.0)); // Will be calculated
    order_props.insert("Total".to_string(), Value::Number(0.0)); // Will be calculated
    order_props.insert("DiscountPercent".to_string(), Value::Number(0.0));
    facts.add_value("Order", Value::Object(order_props))?;

    // Create Product 1: Laptop
    let mut product1_props = HashMap::new();
    product1_props.insert("ProductID".to_string(), Value::String("LAPTOP-001".to_string()));
    product1_props.insert("Name".to_string(), Value::String("Gaming Laptop".to_string()));
    product1_props.insert("Price".to_string(), Value::Number(1500.0));
    product1_props.insert("Quantity".to_string(), Value::Integer(2));
    product1_props.insert("SubTotal".to_string(), Value::Number(0.0)); // Will be calculated
    product1_props.insert("Category".to_string(), Value::String("Electronics".to_string()));
    facts.add_value("Product1", Value::Object(product1_props))?;

    // Create Product 2: Mouse
    let mut product2_props = HashMap::new();
    product2_props.insert("ProductID".to_string(), Value::String("MOUSE-002".to_string()));
    product2_props.insert("Name".to_string(), Value::String("Wireless Mouse".to_string()));
    product2_props.insert("Price".to_string(), Value::Number(50.0));
    product2_props.insert("Quantity".to_string(), Value::Integer(3));
    product2_props.insert("SubTotal".to_string(), Value::Number(0.0)); // Will be calculated
    product2_props.insert("Category".to_string(), Value::String("Accessories".to_string()));
    facts.add_value("Product2", Value::Object(product2_props))?;

    // Create Product 3: Keyboard
    let mut product3_props = HashMap::new();
    product3_props.insert("ProductID".to_string(), Value::String("KEYBOARD-003".to_string()));
    product3_props.insert("Name".to_string(), Value::String("Mechanical Keyboard".to_string()));
    product3_props.insert("Price".to_string(), Value::Number(120.0));
    product3_props.insert("Quantity".to_string(), Value::Integer(1));
    product3_props.insert("SubTotal".to_string(), Value::Number(0.0)); // Will be calculated
    product3_props.insert("Category".to_string(), Value::String("Accessories".to_string()));
    facts.add_value("Product3", Value::Object(product3_props))?;

    // Create Customer for validation rules
    let mut customer_props = HashMap::new();
    customer_props.insert("CustomerID".to_string(), Value::String("CUST-12345".to_string()));
    customer_props.insert("Name".to_string(), Value::String("John Doe".to_string()));
    customer_props.insert("Tier".to_string(), Value::String("Gold".to_string()));
    customer_props.insert("TotalSpent".to_string(), Value::Number(5000.0));
    facts.add_value("Customer", Value::Object(customer_props))?;

    Ok(facts)
}

fn add_product_calculation_rules(engine: &mut RustRuleEngine) -> Result<(), Box<dyn Error>> {
    println!("   📝 Adding product SubTotal calculation rules...");
    
    let rules = vec![
        // Calculate SubTotal for Product1: SubTotal = Price * Quantity
        r#"rule "CalculateProduct1SubTotal" salience 20 {
            when Product1.SubTotal == 0.0 && Product1.Price > 0.0 && Product1.Quantity > 0
            then Product1.SubTotal = Product1.Price * Product1.Quantity;
        }"#,
        
        // Calculate SubTotal for Product2
        r#"rule "CalculateProduct2SubTotal" salience 20 {
            when Product2.SubTotal == 0.0 && Product2.Price > 0.0 && Product2.Quantity > 0
            then Product2.SubTotal = Product2.Price * Product2.Quantity;
        }"#,
        
        // Calculate SubTotal for Product3
        r#"rule "CalculateProduct3SubTotal" salience 20 {
            when Product3.SubTotal == 0.0 && Product3.Price > 0.0 && Product3.Quantity > 0
            then Product3.SubTotal = Product3.Price * Product3.Quantity;
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base.add_rule(rule)?;
        }
    }

    Ok(())
}

fn add_order_calculation_rules(engine: &mut RustRuleEngine) -> Result<(), Box<dyn Error>> {
    println!("   📝 Adding order-level aggregation rules...");
    
    let rules = vec![
        // Calculate Order SubTotal = Sum of all product SubTotals
        r#"rule "CalculateOrderSubTotal" salience 15 {
            when Product1.SubTotal > 0.0 && Product2.SubTotal > 0.0 && Product3.SubTotal > 0.0 && Order.SubTotal == 0.0
            then Order.SubTotal = Product1.SubTotal + Product2.SubTotal + Product3.SubTotal;
        }"#,
        
        // Calculate Tax (8% of SubTotal)
        r#"rule "CalculateTax" salience 10 {
            when Order.SubTotal > 0.0 && Order.Tax == 0.0
            then Order.Tax = Order.SubTotal * 0.08;
        }"#,
        
        // Calculate Final Total
        r#"rule "CalculateTotal" salience 5 {
            when Order.SubTotal > 0.0 && Order.Tax > 0.0
            then Order.Total = Order.SubTotal + Order.Tax;
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base.add_rule(rule)?;
        }
    }

    Ok(())
}

fn add_business_validation_rules(engine: &mut RustRuleEngine) -> Result<(), Box<dyn Error>> {
    println!("   📝 Adding business validation rules...");
    
    let rules = vec![
        // Gold customer discount
        r#"rule "GoldCustomerDiscount" salience 15 {
            when Customer.Tier == "Gold" && Order.SubTotal > 1000.0 && Order.DiscountPercent == 0.0
            then Order.DiscountPercent = 10.0;
        }"#,
        
        // Apply discount to total
        r#"rule "ApplyDiscount" salience 10 {
            when Order.DiscountPercent > 0.0 && Order.Total > 0.0
            then Order.Total = Order.Total * (1.0 - Order.DiscountPercent / 100.0);
        }"#,
        
        // Large order validation
        r#"rule "LargeOrderValidation" salience 5 {
            when Order.SubTotal > 2000.0
            then Order.RequiresApproval = true;
        }"#,
        
        // Final order status
        r#"rule "FinalizeOrder" salience 1 {
            when Order.Total > 0.0
            then Order.Status = "CALCULATED";
        }"#,
    ];

    for rule_str in rules {
        let parsed_rules = GRLParser::parse_rules(rule_str)?;
        for rule in parsed_rules {
            engine.knowledge_base.add_rule(rule)?;
        }
    }

    Ok(())
}

fn display_order_details(facts: &Facts) {
    // Display Order
    if let Some(order) = facts.get("Order") {
        println!("   📋 Order: {:?}", order);
    }

    // Display Products
    for i in 1..=3 {
        let product_key = format!("Product{}", i);
        if let Some(product) = facts.get(&product_key) {
            if let Value::Object(obj) = product {
                let name = obj.get("Name").unwrap_or(&Value::String("Unknown".to_string()));
                let price = obj.get("Price").unwrap_or(&Value::Number(0.0));
                let qty = obj.get("Quantity").unwrap_or(&Value::Integer(0));
                let subtotal = obj.get("SubTotal").unwrap_or(&Value::Number(0.0));
                
                println!("   📦 {}: {} x {} @ ${:.2} = ${:.2}", 
                    product_key, 
                    name.to_string().trim_matches('"'),
                    qty.to_integer(),
                    price.to_number(),
                    subtotal.to_number()
                );
            }
        }
    }

    // Display Customer
    if let Some(customer) = facts.get("Customer") {
        println!("   👤 Customer: {:?}", customer);
    }
}

//! Product Recommendation Engine using Backward Chaining
//!
//! This example demonstrates:
//! - Complex multi-factor decision making
//! - Customer segmentation and profiling
//! - Product matching based on multiple criteria
//! - Recommendation scoring and ranking

use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::types::Value;
use rust_rule_engine::backward::BackwardEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Product Recommendation - Backward Chaining              ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test Case 1: Premium tech enthusiast
    println!("📋 Test Case 1: Premium Tech Enthusiast");
    println!("─────────────────────────────────────────────────────────────");
    test_premium_tech_enthusiast()?;

    println!("\n");

    // Test Case 2: Budget-conscious family shopper
    println!("📋 Test Case 2: Budget-Conscious Family Shopper");
    println!("─────────────────────────────────────────────────────────────");
    test_budget_family_shopper()?;

    println!("\n");

    // Test Case 3: Business professional
    println!("📋 Test Case 3: Business Professional");
    println!("─────────────────────────────────────────────────────────────");
    test_business_professional()?;

    println!("\n✅ All recommendation tests completed!");
    Ok(())
}

fn create_recommendation_rules() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let rules = r#"
rule "IdentifyCustomerTier_Premium" {
    when
        Customer.LoyaltyPoints >= 10000
    then
        Customer.Tier = "Premium";
}

rule "IdentifyCustomerTier_Gold" {
    when
        Customer.LoyaltyPoints >= 5000 && Customer.LoyaltyPoints < 10000
    then
        Customer.Tier = "Gold";
}

rule "IdentifyCustomerTier_Silver" {
    when
        Customer.LoyaltyPoints >= 1000 && Customer.LoyaltyPoints < 5000
    then
        Customer.Tier = "Silver";
}

rule "IdentifyCustomerTier_Bronze" {
    when
        Customer.LoyaltyPoints < 1000
    then
        Customer.Tier = "Bronze";
}

rule "CalculatePurchasePower_High" {
    when
        Customer.AverageOrderValue >= 1000
    then
        Customer.PurchasePower = "High";
}

rule "CalculatePurchasePower_Medium" {
    when
        Customer.AverageOrderValue >= 300 && Customer.AverageOrderValue < 1000
    then
        Customer.PurchasePower = "Medium";
}

rule "CalculatePurchasePower_Low" {
    when
        Customer.AverageOrderValue < 300
    then
        Customer.PurchasePower = "Low";
}

rule "IdentifyShoppingPattern_Frequent" {
    when
        Customer.OrdersPerMonth >= 4
    then
        Customer.ShoppingPattern = "Frequent";
}

rule "IdentifyShoppingPattern_Regular" {
    when
        Customer.OrdersPerMonth >= 2 && Customer.OrdersPerMonth < 4
    then
        Customer.ShoppingPattern = "Regular";
}

rule "IdentifyShoppingPattern_Occasional" {
    when
        Customer.OrdersPerMonth < 2
    then
        Customer.ShoppingPattern = "Occasional";
}

rule "DetermineBudgetCategory_Luxury" {
    when
        Customer.PurchasePower == "High" && Customer.Tier == "Premium"
    then
        Customer.BudgetCategory = "Luxury";
}

rule "DetermineBudgetCategory_Premium" {
    when
        Customer.PurchasePower == "High" && Customer.Tier == "Gold"
    then
        Customer.BudgetCategory = "Premium";
}

rule "DetermineBudgetCategory_Standard" {
    when
        Customer.PurchasePower == "Medium"
    then
        Customer.BudgetCategory = "Standard";
}

rule "DetermineBudgetCategory_Economy" {
    when
        Customer.PurchasePower == "Low"
    then
        Customer.BudgetCategory = "Economy";
}

rule "ProfileSegment_TechEnthusiast" {
    when
        Customer.Category == "Electronics" && Customer.ShoppingPattern == "Frequent"
    then
        Customer.Segment = "TechEnthusiast";
}

rule "ProfileSegment_FamilyShopper" {
    when
        Customer.Category == "Household" && Customer.OrdersPerMonth >= 3
    then
        Customer.Segment = "FamilyShopper";
}

rule "ProfileSegment_BusinessUser" {
    when
        Customer.Category == "Business" && Customer.PurchasePower == "High"
    then
        Customer.Segment = "BusinessUser";
}

rule "ProfileSegment_Fashionista" {
    when
        Customer.Category == "Fashion" && Customer.ShoppingPattern == "Frequent"
    then
        Customer.Segment = "Fashionista";
}

rule "RecommendProduct_LatestGadget" {
    when
        Customer.Segment == "TechEnthusiast" && Customer.BudgetCategory == "Luxury"
    then
        Recommendation.ProductType = "LatestGadget";
        Recommendation.Category = "Electronics";
}

rule "RecommendProduct_SmartHome" {
    when
        Customer.Segment == "TechEnthusiast" && Customer.BudgetCategory == "Premium"
    then
        Recommendation.ProductType = "SmartHome";
        Recommendation.Category = "Electronics";
}

rule "RecommendProduct_FamilyPack" {
    when
        Customer.Segment == "FamilyShopper" && Customer.BudgetCategory == "Standard"
    then
        Recommendation.ProductType = "FamilyPack";
        Recommendation.Category = "Household";
}

rule "RecommendProduct_BulkEssentials" {
    when
        Customer.Segment == "FamilyShopper" && Customer.BudgetCategory == "Economy"
    then
        Recommendation.ProductType = "BulkEssentials";
        Recommendation.Category = "Household";
}

rule "RecommendProduct_BusinessSuite" {
    when
        Customer.Segment == "BusinessUser" && Customer.PurchasePower == "High"
    then
        Recommendation.ProductType = "BusinessSuite";
        Recommendation.Category = "Business";
}

rule "CalculateRecommendationScore_Excellent" {
    when
        Customer.Segment == "TechEnthusiast" && Customer.BudgetCategory == "Luxury" && Customer.Tier == "Premium"
    then
        Recommendation.Score = 95;
        Recommendation.Confidence = "Excellent";
}

rule "CalculateRecommendationScore_Good" {
    when
        Customer.Segment == "TechEnthusiast" && Customer.BudgetCategory == "Premium"
    then
        Recommendation.Score = 85;
        Recommendation.Confidence = "Good";
}

rule "CalculateRecommendationScore_Fair" {
    when
        Customer.Segment == "FamilyShopper" && Customer.BudgetCategory == "Standard"
    then
        Recommendation.Score = 75;
        Recommendation.Confidence = "Fair";
}

rule "ApplyDiscount_PremiumMember" {
    when
        Customer.Tier == "Premium" && Recommendation.Score >= 90
    then
        Recommendation.DiscountPercent = 20;
}

rule "ApplyDiscount_GoldMember" {
    when
        Customer.Tier == "Gold" && Recommendation.Score >= 80
    then
        Recommendation.DiscountPercent = 15;
}

rule "ApplyDiscount_SilverMember" {
    when
        Customer.Tier == "Silver" && Recommendation.Score >= 70
    then
        Recommendation.DiscountPercent = 10;
}

rule "IncludeUpsell_HighValue" {
    when
        Customer.PurchasePower == "High" && Recommendation.ProductType == "LatestGadget"
    then
        Recommendation.IncludeUpsell = true;
        Recommendation.UpsellType = "Accessories";
}

rule "IncludeCrossSell_TechUser" {
    when
        Customer.Segment == "TechEnthusiast" && Recommendation.ProductType == "SmartHome"
    then
        Recommendation.IncludeCrossSell = true;
        Recommendation.CrossSellType = "ConnectedDevices";
}

rule "PrioritizeRecommendation_TopTier" {
    when
        Recommendation.Score >= 90 && Customer.Tier == "Premium"
    then
        Recommendation.Priority = "High";
        Recommendation.SendEmail = true;
}

rule "PrioritizeRecommendation_MidTier" {
    when
        Recommendation.Score >= 75 && Recommendation.Score < 90
    then
        Recommendation.Priority = "Medium";
        Recommendation.SendNotification = true;
}

rule "FinalizeRecommendation" {
    when
        Recommendation.ProductType != "" && Recommendation.Score >= 70
    then
        Recommendation.Approved = true;
}
    "#;

    let mut kb = KnowledgeBase::new("ProductRecommendationSystem");
    for rule in rust_rule_engine::parser::grl::GRLParser::parse_rules(rules)? {
        kb.add_rule(rule)?;
    }
    Ok(kb)
}

fn test_premium_tech_enthusiast() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_recommendation_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("Customer.ID", Value::Number(1001.0));
    facts.set("Customer.Name", Value::String("Alex Tech".to_string()));
    facts.set("Customer.LoyaltyPoints", Value::Number(15000.0));
    facts.set("Customer.AverageOrderValue", Value::Number(1500.0));
    facts.set("Customer.OrdersPerMonth", Value::Number(5.0));
    facts.set("Customer.Category", Value::String("Electronics".to_string()));

    println!("Customer Profile:");
    println!("  Name: Alex Tech");
    println!("  Loyalty Points: 15,000");
    println!("  Average Order Value: $1,500");
    println!("  Orders Per Month: 5");
    println!("  Primary Category: Electronics");

    // Query: Should we recommend a product?
    println!("\n🔍 Analyzing customer for product recommendation...");
    facts.set("Recommendation.ProductType", Value::String("".to_string()));

    let result = bc_engine.query("Recommendation.Approved == true", &mut facts)?;

    if result.provable {
        println!("  ✓ RECOMMENDATION APPROVED!");

        println!("\n  Customer Segmentation:");
        if let Some(tier) = facts.get("Customer.Tier") {
            println!("    • Tier: {:?}", tier);
        }
        if let Some(power) = facts.get("Customer.PurchasePower") {
            println!("    • Purchase Power: {:?}", power);
        }
        if let Some(pattern) = facts.get("Customer.ShoppingPattern") {
            println!("    • Shopping Pattern: {:?}", pattern);
        }
        if let Some(budget) = facts.get("Customer.BudgetCategory") {
            println!("    • Budget Category: {:?}", budget);
        }
        if let Some(segment) = facts.get("Customer.Segment") {
            println!("    • Customer Segment: {:?}", segment);
        }

        println!("\n  Recommendation Details:");
        if let Some(product_type) = facts.get("Recommendation.ProductType") {
            println!("    • Product Type: {:?}", product_type);
        }
        if let Some(category) = facts.get("Recommendation.Category") {
            println!("    • Category: {:?}", category);
        }
        if let Some(score) = facts.get("Recommendation.Score") {
            println!("    • Confidence Score: {:?}", score);
        }
        if let Some(confidence) = facts.get("Recommendation.Confidence") {
            println!("    • Confidence Level: {:?}", confidence);
        }

        println!("\n  Special Offers:");
        if let Some(discount) = facts.get("Recommendation.DiscountPercent") {
            println!("    • Discount: {:?}%", discount);
        }
        if let Some(upsell) = facts.get("Recommendation.IncludeUpsell") {
            if let Value::Boolean(true) = upsell {
                if let Some(upsell_type) = facts.get("Recommendation.UpsellType") {
                    println!("    • Upsell Available: {:?}", upsell_type);
                }
            }
        }

        println!("\n  Marketing Actions:");
        if let Some(priority) = facts.get("Recommendation.Priority") {
            println!("    • Priority: {:?}", priority);
        }
        if let Some(email) = facts.get("Recommendation.SendEmail") {
            if let Value::Boolean(true) = email {
                println!("    • Action: Send personalized email");
            }
        }

        println!("\n  Statistics:");
        println!("    {} goals explored", result.stats.goals_explored);
        println!("    {} rules evaluated", result.stats.rules_evaluated);
        println!("    Max depth: {}", result.stats.max_depth);
    } else {
        println!("  ✗ No recommendation at this time");
    }

    Ok(())
}

fn test_budget_family_shopper() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_recommendation_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("Customer.ID", Value::Number(2002.0));
    facts.set("Customer.Name", Value::String("Sarah Family".to_string()));
    facts.set("Customer.LoyaltyPoints", Value::Number(3500.0));
    facts.set("Customer.AverageOrderValue", Value::Number(250.0));
    facts.set("Customer.OrdersPerMonth", Value::Number(4.0));
    facts.set("Customer.Category", Value::String("Household".to_string()));

    println!("Customer Profile:");
    println!("  Name: Sarah Family");
    println!("  Loyalty Points: 3,500");
    println!("  Average Order Value: $250");
    println!("  Orders Per Month: 4");
    println!("  Primary Category: Household");

    println!("\n🔍 Analyzing customer for product recommendation...");
    facts.set("Recommendation.ProductType", Value::String("".to_string()));

    let result = bc_engine.query("Recommendation.Approved == true", &mut facts)?;

    if result.provable {
        println!("  ✓ RECOMMENDATION APPROVED!");

        println!("\n  Customer Profile:");
        if let Some(tier) = facts.get("Customer.Tier") {
            println!("    • Tier: {:?}", tier);
        }
        if let Some(segment) = facts.get("Customer.Segment") {
            println!("    • Segment: {:?}", segment);
        }
        if let Some(budget) = facts.get("Customer.BudgetCategory") {
            println!("    • Budget Category: {:?}", budget);
        }

        println!("\n  Recommended Products:");
        if let Some(product_type) = facts.get("Recommendation.ProductType") {
            println!("    • Type: {:?}", product_type);
        }
        if let Some(score) = facts.get("Recommendation.Score") {
            println!("    • Score: {:?}", score);
        }

        println!("\n  Benefits:");
        println!("    ✓ Bulk savings for family needs");
        println!("    ✓ Value-focused recommendations");
        println!("    ✓ Frequent purchase rewards");

        if let Some(discount) = facts.get("Recommendation.DiscountPercent") {
            println!("    ✓ Member discount: {:?}%", discount);
        }
    } else {
        println!("  ✗ No recommendation approved");
    }

    Ok(())
}

fn test_business_professional() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_recommendation_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("Customer.ID", Value::Number(3003.0));
    facts.set("Customer.Name", Value::String("Mike Business".to_string()));
    facts.set("Customer.LoyaltyPoints", Value::Number(8000.0));
    facts.set("Customer.AverageOrderValue", Value::Number(2000.0));
    facts.set("Customer.OrdersPerMonth", Value::Number(3.0));
    facts.set("Customer.Category", Value::String("Business".to_string()));

    println!("Customer Profile:");
    println!("  Name: Mike Business");
    println!("  Loyalty Points: 8,000");
    println!("  Average Order Value: $2,000");
    println!("  Orders Per Month: 3");
    println!("  Primary Category: Business");

    println!("\n🔍 Analyzing customer for product recommendation...");
    facts.set("Recommendation.ProductType", Value::String("".to_string()));

    let result = bc_engine.query("Recommendation.Approved == true", &mut facts)?;

    if result.provable {
        println!("  ✓ RECOMMENDATION APPROVED!");

        println!("\n  Business Customer Profile:");
        if let Some(tier) = facts.get("Customer.Tier") {
            println!("    • Tier: {:?}", tier);
        }
        if let Some(power) = facts.get("Customer.PurchasePower") {
            println!("    • Purchase Power: {:?}", power);
        }
        if let Some(segment) = facts.get("Customer.Segment") {
            println!("    • Segment: {:?}", segment);
        }

        println!("\n  Business Recommendations:");
        if let Some(product_type) = facts.get("Recommendation.ProductType") {
            println!("    • Product Suite: {:?}", product_type);
        }
        if let Some(score) = facts.get("Recommendation.Score") {
            println!("    • Match Score: {:?}", score);
        }

        println!("\n  Business Value:");
        println!("    ✓ Enterprise-grade solutions");
        println!("    ✓ High purchase power recognition");
        println!("    ✓ Professional support included");

        if let Some(discount) = facts.get("Recommendation.DiscountPercent") {
            println!("    ✓ Corporate discount: {:?}%", discount);
        }

        println!("\n  Statistics:");
        println!("    {} goals explored", result.stats.goals_explored);
        println!("    Max depth: {}", result.stats.max_depth);
    } else {
        println!("  ✗ No recommendation approved");
    }

    Ok(())
}

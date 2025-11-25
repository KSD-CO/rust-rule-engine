//! Financial Loan Approval System using Backward Chaining
//!
//! This example demonstrates:
//! - Complex multi-level reasoning
//! - Depth-first search strategy
//! - Multiple rule chains
//! - Variable bindings for loan parameters

use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::types::Value;
use rust_rule_engine::backward::BackwardEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Financial Loan Approval - Backward Chaining Demo        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test Case 1: Prime customer with good credit
    println!("📋 Test Case 1: Prime Customer");
    println!("─────────────────────────────────────────────────────────────");
    test_prime_customer()?;

    println!("\n");

    // Test Case 2: Subprime customer - should be rejected
    println!("📋 Test Case 2: Subprime Customer");
    println!("─────────────────────────────────────────────────────────────");
    test_subprime_customer()?;

    println!("\n");

    // Test Case 3: Borderline case with collateral
    println!("📋 Test Case 3: Borderline with Collateral");
    println!("─────────────────────────────────────────────────────────────");
    test_borderline_with_collateral()?;

    println!("\n✅ All loan approval tests completed!");
    Ok(())
}

fn create_loan_rules() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let rules = r#"
rule "IdentifyCreditTier_Excellent" {
    when
        Customer.CreditScore >= 750
    then
        Customer.CreditTier = "Excellent";
}

rule "IdentifyCreditTier_Good" {
    when
        Customer.CreditScore >= 650 && Customer.CreditScore < 750
    then
        Customer.CreditTier = "Good";
}

rule "IdentifyCreditTier_Fair" {
    when
        Customer.CreditScore >= 550 && Customer.CreditScore < 650
    then
        Customer.CreditTier = "Fair";
}

rule "IdentifyCreditTier_Poor" {
    when
        Customer.CreditScore < 550
    then
        Customer.CreditTier = "Poor";
}

rule "CalculateDebtRatio" {
    when
        Customer.MonthlyDebt > 0 && Customer.MonthlyIncome > 0
    then
        Customer.DebtToIncomeRatio = Customer.MonthlyDebt / Customer.MonthlyIncome;
}

rule "CheckDebtRatioStatus_Healthy" {
    when
        Customer.DebtToIncomeRatio <= 0.36
    then
        Customer.DebtRatioStatus = "Healthy";
}

rule "CheckDebtRatioStatus_Concerning" {
    when
        Customer.DebtToIncomeRatio > 0.36 && Customer.DebtToIncomeRatio <= 0.50
    then
        Customer.DebtRatioStatus = "Concerning";
}

rule "CheckDebtRatioStatus_High" {
    when
        Customer.DebtToIncomeRatio > 0.50
    then
        Customer.DebtRatioStatus = "High";
}

rule "DetermineRiskCategory_Low" {
    when
        Customer.CreditTier == "Excellent" && Customer.DebtRatioStatus == "Healthy"
    then
        Loan.RiskCategory = "Low";
}

rule "DetermineRiskCategory_Medium" {
    when
        Customer.CreditTier == "Good" && Customer.DebtRatioStatus == "Healthy"
    then
        Loan.RiskCategory = "Medium";
}

rule "DetermineRiskCategory_Medium_Alt" {
    when
        Customer.CreditTier == "Excellent" && Customer.DebtRatioStatus == "Concerning"
    then
        Loan.RiskCategory = "Medium";
}

rule "DetermineRiskCategory_High" {
    when
        Customer.CreditTier == "Fair"
    then
        Loan.RiskCategory = "High";
}

rule "DetermineRiskCategory_VeryHigh" {
    when
        Customer.CreditTier == "Poor"
    then
        Loan.RiskCategory = "VeryHigh";
}

rule "CalculateInterestRate_Low" {
    when
        Loan.RiskCategory == "Low"
    then
        Loan.InterestRate = 3.5;
}

rule "CalculateInterestRate_Medium" {
    when
        Loan.RiskCategory == "Medium"
    then
        Loan.InterestRate = 5.5;
}

rule "CalculateInterestRate_High" {
    when
        Loan.RiskCategory == "High"
    then
        Loan.InterestRate = 8.5;
}

rule "CalculateInterestRate_VeryHigh" {
    when
        Loan.RiskCategory == "VeryHigh"
    then
        Loan.InterestRate = 12.5;
}

rule "ApproveLoan_LowRisk" {
    when
        Loan.RiskCategory == "Low" && Loan.Amount <= 500000
    then
        Loan.Approved = true;
        Loan.ApprovalReason = "Low risk profile";
}

rule "ApproveLoan_MediumRisk" {
    when
        Loan.RiskCategory == "Medium" && Loan.Amount <= 300000
    then
        Loan.Approved = true;
        Loan.ApprovalReason = "Medium risk with reasonable amount";
}

rule "ApproveLoan_HighRisk_WithCollateral" {
    when
        Loan.RiskCategory == "High" && Loan.HasCollateral == true && Loan.Amount <= 150000
    then
        Loan.Approved = true;
        Loan.ApprovalReason = "High risk mitigated by collateral";
}

rule "RejectLoan_VeryHighRisk" {
    when
        Loan.RiskCategory == "VeryHigh"
    then
        Loan.Approved = false;
        Loan.RejectionReason = "Credit score too low";
}

rule "RejectLoan_ExcessiveAmount" {
    when
        Loan.RiskCategory == "High" && Loan.Amount > 150000
    then
        Loan.Approved = false;
        Loan.RejectionReason = "Loan amount too high for risk category";
}
    "#;

    let mut kb = KnowledgeBase::new("LoanApprovalSystem");
    for rule in rust_rule_engine::parser::grl::GRLParser::parse_rules(rules)? {
        kb.add_rule(rule)?;
    }
    Ok(kb)
}

fn test_prime_customer() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_loan_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    // Setup customer facts
    let mut facts = Facts::new();
    facts.set("Customer.CreditScore", Value::Number(780.0));
    facts.set("Customer.MonthlyIncome", Value::Number(8000.0));
    facts.set("Customer.MonthlyDebt", Value::Number(2000.0));
    facts.set("Loan.Amount", Value::Number(250000.0));
    facts.set("Loan.HasCollateral", Value::Boolean(false));

    println!("Customer Profile:");
    println!("  Credit Score: 780");
    println!("  Monthly Income: $8,000");
    println!("  Monthly Debt: $2,000");
    println!("  Loan Amount: $250,000");

    // Query: Can the loan be approved?
    println!("\n🔍 Query: Loan.Approved == true");
    let result = bc_engine.query("Loan.Approved == true", &mut facts)?;

    if result.provable {
        println!("  ✓ Loan APPROVED!");

        // Show derived facts
        println!("\n  Derived Facts:");
        if let Some(tier) = facts.get("Customer.CreditTier") {
            println!("    • Credit Tier: {:?}", tier);
        }
        if let Some(ratio) = facts.get("Customer.DebtToIncomeRatio") {
            println!("    • Debt-to-Income Ratio: {:.2}",
                if let Value::Number(n) = ratio { n } else { 0.0 });
        }
        if let Some(status) = facts.get("Customer.DebtRatioStatus") {
            println!("    • Debt Ratio Status: {:?}", status);
        }
        if let Some(risk) = facts.get("Loan.RiskCategory") {
            println!("    • Risk Category: {:?}", risk);
        }
        if let Some(rate) = facts.get("Loan.InterestRate") {
            println!("    • Interest Rate: {:.1}%",
                if let Value::Number(n) = rate { n } else { 0.0 });
        }
        if let Some(reason) = facts.get("Loan.ApprovalReason") {
            println!("    • Reason: {:?}", reason);
        }

        // Show proof trace
        println!("\n  Proof Trace:");
        println!("    {} goals explored", result.stats.goals_explored);
        println!("    {} rules evaluated", result.stats.rules_evaluated);
        println!("    Max depth: {}", result.stats.max_depth);
    } else {
        println!("  ✗ Loan NOT approved");
    }

    Ok(())
}

fn test_subprime_customer() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_loan_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("Customer.CreditScore", Value::Number(480.0));
    facts.set("Customer.MonthlyIncome", Value::Number(3000.0));
    facts.set("Customer.MonthlyDebt", Value::Number(1800.0));
    facts.set("Loan.Amount", Value::Number(50000.0));
    facts.set("Loan.HasCollateral", Value::Boolean(false));

    println!("Customer Profile:");
    println!("  Credit Score: 480 (Poor)");
    println!("  Monthly Income: $3,000");
    println!("  Monthly Debt: $1,800");
    println!("  Loan Amount: $50,000");

    println!("\n🔍 Query: Loan.Approved == true");
    let result = bc_engine.query("Loan.Approved == true", &mut facts)?;

    if result.provable {
        println!("  ✓ Loan APPROVED (unexpected!)");
    } else {
        println!("  ✗ Loan NOT approved (expected)");

        // Check rejection reason
        println!("\n  Checking rejection reason...");
        let reject_result = bc_engine.query("Loan.Approved == false", &mut facts)?;

        if reject_result.provable {
            if let Some(reason) = facts.get("Loan.RejectionReason") {
                println!("    • Rejection Reason: {:?}", reason);
            }
            if let Some(tier) = facts.get("Customer.CreditTier") {
                println!("    • Credit Tier: {:?}", tier);
            }
            if let Some(risk) = facts.get("Loan.RiskCategory") {
                println!("    • Risk Category: {:?}", risk);
            }
        }

        println!("\n  Statistics:");
        println!("    {} goals explored", result.stats.goals_explored);
        println!("    Missing facts: {:?}", result.missing_facts);
    }

    Ok(())
}

fn test_borderline_with_collateral() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_loan_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("Customer.CreditScore", Value::Number(620.0));
    facts.set("Customer.MonthlyIncome", Value::Number(5000.0));
    facts.set("Customer.MonthlyDebt", Value::Number(2000.0));
    facts.set("Loan.Amount", Value::Number(120000.0));
    facts.set("Loan.HasCollateral", Value::Boolean(true));

    println!("Customer Profile:");
    println!("  Credit Score: 620 (Fair)");
    println!("  Monthly Income: $5,000");
    println!("  Monthly Debt: $2,000");
    println!("  Loan Amount: $120,000");
    println!("  Has Collateral: Yes");

    println!("\n🔍 Query: Loan.Approved == true");
    let result = bc_engine.query("Loan.Approved == true", &mut facts)?;

    if result.provable {
        println!("  ✓ Loan APPROVED!");

        println!("\n  Key Decision Factors:");
        if let Some(tier) = facts.get("Customer.CreditTier") {
            println!("    • Credit Tier: {:?}", tier);
        }
        if let Some(ratio) = facts.get("Customer.DebtToIncomeRatio") {
            println!("    • Debt-to-Income Ratio: {:.2}",
                if let Value::Number(n) = ratio { n } else { 0.0 });
        }
        if let Some(risk) = facts.get("Loan.RiskCategory") {
            println!("    • Risk Category: {:?}", risk);
        }
        if let Some(rate) = facts.get("Loan.InterestRate") {
            println!("    • Interest Rate: {:.1}%",
                if let Value::Number(n) = rate { n } else { 0.0 });
        }
        if let Some(reason) = facts.get("Loan.ApprovalReason") {
            println!("    • Approval Reason: {:?}", reason);
        }

        println!("\n  Analysis:");
        println!("    This case shows how collateral can help approve");
        println!("    borderline applications with fair credit scores.");
    } else {
        println!("  ✗ Loan NOT approved");
    }

    Ok(())
}

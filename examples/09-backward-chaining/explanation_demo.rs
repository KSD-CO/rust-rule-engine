// Explanation System Demo
//
// This example demonstrates the explanation system for backward chaining queries.
// It shows how to generate human-readable explanations of reasoning processes.

use rust_rule_engine::backward::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("\n🔍 Backward Chaining Explanation System Demo");
    println!("{}", "=".repeat(80));

    // Demo 1: Simple reasoning with proof tree
    demo_1_simple_proof_tree()?;

    // Demo 2: Complex multi-level reasoning
    demo_2_complex_reasoning()?;

    // Demo 3: Negation in reasoning
    demo_3_negation_reasoning()?;

    // Demo 4: Export to different formats
    demo_4_export_formats()?;

    println!("\n{}", "=".repeat(80));
    println!("✅ All explanation demos completed successfully!");
    println!("{}", "=".repeat(80));

    Ok(())
}

/// Demo 1: Simple proof tree with basic facts
fn demo_1_simple_proof_tree() -> Result<(), Box<dyn Error>> {
    println!("\n📊 Demo 1: Simple Proof Tree");
    println!("{}", "-".repeat(80));

    // Create a simple proof tree manually
    let mut root = ProofNode::rule("user.is_vip == true".to_string(), "vip_rule".to_string(), 0);

    let mut child1 = ProofNode::rule(
        "user.purchase_total > 1000".to_string(),
        "spending_rule".to_string(),
        1,
    );

    let fact1 = ProofNode::fact("user.purchase_total = 1500".to_string(), 2);
    child1.add_child(fact1);
    root.add_child(child1);

    let tree = ProofTree::new(root, "user.is_vip == true".to_string());

    println!("\nProof Tree:");
    tree.print();

    Ok(())
}

/// Demo 2: Complex multi-level reasoning (Loan Approval)
fn demo_2_complex_reasoning() -> Result<(), Box<dyn Error>> {
    println!("\n💰 Demo 2: Loan Approval System with Explanation");
    println!("{}", "-".repeat(80));

    println!("\n📋 Applicant: Alice");
    println!("  Credit Score: 750");
    println!("  Years Employed: 5");
    println!("  Debt Ratio: 0.25");

    // Create explanation builder (simulation - actual integration pending)
    let mut builder = ExplanationBuilder::new();
    builder.enable();

    // Simulate proof tree building
    let mut root = ProofNode::rule(
        "loan_status = approved".to_string(),
        "loan_approved".to_string(),
        0,
    );

    let mut credit_node = ProofNode::rule(
        "has_good_credit == true".to_string(),
        "good_credit".to_string(),
        1,
    );
    let credit_fact = ProofNode::fact("credit_score = 750".to_string(), 2);
    credit_node.add_child(credit_fact);
    root.add_child(credit_node);

    let mut income_node = ProofNode::rule(
        "has_stable_income == true".to_string(),
        "stable_income".to_string(),
        1,
    );
    let income_fact = ProofNode::fact("years_employed = 5".to_string(), 2);
    income_node.add_child(income_fact);
    root.add_child(income_node);

    let mut debt_node = ProofNode::rule(
        "has_low_debt == true".to_string(),
        "low_debt".to_string(),
        1,
    );
    let debt_fact = ProofNode::fact("debt_ratio = 0.25".to_string(), 2);
    debt_node.add_child(debt_fact);
    root.add_child(debt_node);

    let tree = ProofTree::new(root, "loan_status = approved".to_string());

    println!("\n🌳 Proof Tree:");
    tree.print();

    // Create full explanation
    let explanation = Explanation::new("Is Alice's loan approved?".to_string(), tree);

    println!("\n📝 Full Explanation:");
    explanation.print();

    Ok(())
}

/// Demo 3: Negation in reasoning
fn demo_3_negation_reasoning() -> Result<(), Box<dyn Error>> {
    println!("\n🚫 Demo 3: Negation in Reasoning");
    println!("{}", "-".repeat(80));

    // Build proof tree with negation
    let mut root = ProofNode::rule(
        "user.can_access == true".to_string(),
        "access_rule".to_string(),
        0,
    );

    let active_fact = ProofNode::fact("user.is_active = true".to_string(), 1);
    root.add_child(active_fact);

    // Negation node - user is NOT banned
    let not_banned = ProofNode::negation(
        "NOT user.is_banned == true".to_string(),
        1,
        true, // Negation succeeded
    );
    root.add_child(not_banned);

    let tree = ProofTree::new(root, "user.can_access == true".to_string());

    println!("\nUser Access Check:");
    println!("  ✓ User is active");
    println!("  ✓ User is NOT banned (closed-world assumption)");
    println!("  → Access GRANTED");

    println!("\n🌳 Proof Tree with Negation:");
    tree.print();

    Ok(())
}

/// Demo 4: Export to different formats
fn demo_4_export_formats() -> Result<(), Box<dyn Error>> {
    println!("\n📤 Demo 4: Export to Different Formats");
    println!("{}", "-".repeat(80));

    // Build a sample proof tree
    let mut root = ProofNode::rule(
        "eligible_for_discount == true".to_string(),
        "discount_rule".to_string(),
        0,
    );

    let vip_fact = ProofNode::fact("user.is_vip = true".to_string(), 1);
    root.add_child(vip_fact);

    let purchase_fact = ProofNode::fact("order.total = 500".to_string(), 1);
    root.add_child(purchase_fact);

    let tree = ProofTree::new(root, "Is customer eligible for discount?".to_string());

    // Export to JSON
    println!("\n1️⃣ JSON Export:");
    match tree.to_json() {
        Ok(json) => {
            println!("{}", &json[..200.min(json.len())]);
            if json.len() > 200 {
                println!("... (truncated)");
            }
            // Save to file
            std::fs::write("explanation.json", json)?;
            println!("\n✅ Saved to explanation.json");
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    // Export to Markdown
    println!("\n2️⃣ Markdown Export:");
    let md = tree.to_markdown();
    println!("{}", &md[..200.min(md.len())]);
    if md.len() > 200 {
        println!("... (truncated)");
    }
    std::fs::write("explanation.md", md)?;
    println!("\n✅ Saved to explanation.md");

    // Export to HTML
    println!("\n3️⃣ HTML Export:");
    let html = tree.to_html();
    println!("<html excerpt>");
    println!("  <title>Proof Explanation</title>");
    println!("  ... (full HTML document)");
    std::fs::write("explanation.html", html)?;
    println!("\n✅ Saved to explanation.html");

    println!("\n📁 Files generated:");
    println!("  • explanation.json - Machine-readable format");
    println!("  • explanation.md   - Human-readable markdown");
    println!("  • explanation.html - Interactive web view");

    Ok(())
}

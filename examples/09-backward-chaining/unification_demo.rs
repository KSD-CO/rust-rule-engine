//! Demonstration of Variable Unification in Backward Chaining
//!
//! This example shows how to use variable bindings and unification
//! for pattern matching in backward chaining queries.

use rust_rule_engine::backward::{BackwardEngine, Bindings, Expression, ExpressionParser, Unifier};
use rust_rule_engine::types::Value;
use rust_rule_engine::{Facts, KnowledgeBase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Backward Chaining - Variable Unification Demo           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    demo_1_basic_variable_binding()?;
    demo_2_pattern_matching_with_variables()?;
    demo_3_unification_algorithm()?;
    demo_4_complex_queries_with_variables()?;
    demo_5_conflict_detection()?;

    println!("\n✅ All unification demos completed successfully!");
    Ok(())
}

/// Demo 1: Basic Variable Binding
fn demo_1_basic_variable_binding() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 1: Basic Variable Binding");
    println!("─────────────────────────────────────────────────────────────");

    // Create a bindings set
    let mut bindings = Bindings::new();

    // Bind variables to values
    println!("Binding variables:");
    bindings.bind(
        "Customer".to_string(),
        Value::String("John Doe".to_string()),
    )?;
    println!("  ?Customer = \"John Doe\"");

    bindings.bind("Age".to_string(), Value::Number(35.0))?;
    println!("  ?Age = 35");

    bindings.bind("IsVIP".to_string(), Value::Boolean(true))?;
    println!("  ?IsVIP = true");

    // Retrieve bindings
    println!("\nRetrieving bindings:");
    if let Some(customer) = bindings.get("Customer") {
        println!("  ?Customer -> {:?}", customer);
    }
    if let Some(age) = bindings.get("Age") {
        println!("  ?Age -> {:?}", age);
    }
    if let Some(is_vip) = bindings.get("IsVIP") {
        println!("  ?IsVIP -> {:?}", is_vip);
    }

    println!("\n✓ Demo 1 complete\n");
    Ok(())
}

/// Demo 2: Pattern Matching with Variables
fn demo_2_pattern_matching_with_variables() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 2: Pattern Matching with Variables");
    println!("─────────────────────────────────────────────────────────────");

    // Setup facts
    let facts = Facts::new();
    facts.set("User.Name", Value::String("Alice".to_string()));
    facts.set("User.Age", Value::Number(28.0));
    facts.set("User.IsVIP", Value::Boolean(true));
    facts.set("Order.Amount", Value::Number(1500.0));

    println!("Facts:");
    println!("  User.Name = \"Alice\"");
    println!("  User.Age = 28");
    println!("  User.IsVIP = true");
    println!("  Order.Amount = 1500");

    // Create bindings
    let mut bindings = Bindings::new();

    // Match expressions against facts
    println!("\nPattern Matching:");

    // Match 1: User.IsVIP == true
    let expr1 = ExpressionParser::parse("User.IsVIP == true")?;
    let match1 = Unifier::match_expression(&expr1, &facts, &mut bindings)?;
    println!(
        "  User.IsVIP == true -> {}",
        if match1 { "✓ Match" } else { "✗ No match" }
    );

    // Match 2: Order.Amount > 1000
    let expr2 = ExpressionParser::parse("Order.Amount > 1000")?;
    let match2 = Unifier::match_expression(&expr2, &facts, &mut bindings)?;
    println!(
        "  Order.Amount > 1000 -> {}",
        if match2 { "✓ Match" } else { "✗ No match" }
    );

    // Match 3: User.Age < 25 (should fail)
    let expr3 = ExpressionParser::parse("User.Age < 25")?;
    let match3 = Unifier::match_expression(&expr3, &facts, &mut bindings)?;
    println!(
        "  User.Age < 25 -> {}",
        if match3 { "✓ Match" } else { "✗ No match" }
    );

    println!("\n✓ Demo 2 complete\n");
    Ok(())
}

/// Demo 3: Unification Algorithm
fn demo_3_unification_algorithm() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 3: Unification Algorithm");
    println!("─────────────────────────────────────────────────────────────");

    let mut bindings = Bindings::new();

    println!("Unifying expressions:");

    // Unify 1: Variable with Literal
    println!("\n1. Unifying ?X with 42:");
    let var_x = Expression::Variable("X".to_string());
    let lit_42 = Expression::Literal(Value::Number(42.0));

    let result1 = Unifier::unify(&var_x, &lit_42, &mut bindings)?;
    println!(
        "   Result: {}",
        if result1 { "Success ✓" } else { "Failed ✗" }
    );
    if let Some(x_val) = bindings.get("X") {
        println!("   ?X is now bound to: {:?}", x_val);
    }

    // Unify 2: Already bound variable with same value
    println!("\n2. Unifying ?X (already 42) with 42 again:");
    let result2 = Unifier::unify(&var_x, &lit_42, &mut bindings)?;
    println!(
        "   Result: {}",
        if result2 { "Success ✓" } else { "Failed ✗" }
    );

    // Unify 3: Two literals (same value)
    println!("\n3. Unifying literal 100 with literal 100:");
    let lit1 = Expression::Literal(Value::Number(100.0));
    let lit2 = Expression::Literal(Value::Number(100.0));
    let result3 = Unifier::unify(&lit1, &lit2, &mut bindings)?;
    println!(
        "   Result: {}",
        if result3 { "Success ✓" } else { "Failed ✗" }
    );

    // Unify 4: Two literals (different values)
    println!("\n4. Unifying literal 100 with literal 200:");
    let lit3 = Expression::Literal(Value::Number(200.0));
    let result4 = Unifier::unify(&lit1, &lit3, &mut bindings)?;
    println!(
        "   Result: {}",
        if result4 { "Success ✓" } else { "Failed ✗" }
    );

    println!("\n✓ Demo 3 complete\n");
    Ok(())
}

/// Demo 4: Complex Queries with Variables
fn demo_4_complex_queries_with_variables() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 4: Complex Queries with Variables");
    println!("─────────────────────────────────────────────────────────────");

    // Setup knowledge base with rules
    let rules = r#"
rule "IdentifyVIPCustomer" {
    when
        Customer.Points > 1000
    then
        Customer.Status = "VIP";
}

rule "CalculateDiscount" {
    when
        Customer.Status == "VIP" && Order.Amount > 500
    then
        Order.Discount = 0.15;
}

rule "ApproveOrder" {
    when
        Order.Discount == 0.15
    then
        Order.Approved = true;
}
    "#;

    let kb = KnowledgeBase::new("VariableDemo");
    for rule in rust_rule_engine::parser::grl::GRLParser::parse_rules(rules)? {
        kb.add_rule(rule)?;
    }

    // Setup facts
    let mut facts = Facts::new();
    facts.set("Customer.Points", Value::Number(1500.0));
    facts.set("Order.Amount", Value::Number(800.0));

    println!("Initial Facts:");
    println!("  Customer.Points = 1500");
    println!("  Order.Amount = 800");

    // Create backward chaining engine
    let mut bc_engine = BackwardEngine::new(kb);

    // Query 1: Is order approved?
    println!("\n🔍 Query 1: Order.Approved == true");
    let result1 = bc_engine.query("Order.Approved == true", &mut facts)?;

    if result1.provable {
        println!("   ✓ Goal is provable!");
        println!("   Bindings: {:?}", result1.bindings);

        // Show derived facts
        println!("\n   Derived facts:");
        if let Some(status) = facts.get("Customer.Status") {
            println!("     Customer.Status = {:?}", status);
        }
        if let Some(discount) = facts.get("Order.Discount") {
            println!("     Order.Discount = {:?}", discount);
        }
        if let Some(approved) = facts.get("Order.Approved") {
            println!("     Order.Approved = {:?}", approved);
        }
    } else {
        println!("   ✗ Goal is not provable");
    }

    println!("\n✓ Demo 4 complete\n");
    Ok(())
}

/// Demo 5: Conflict Detection
fn demo_5_conflict_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 5: Conflict Detection");
    println!("─────────────────────────────────────────────────────────────");

    let mut bindings = Bindings::new();

    // First binding
    println!("Binding ?Customer to \"Alice\":");
    bindings.bind("Customer".to_string(), Value::String("Alice".to_string()))?;
    println!("  ✓ Success");

    // Same binding (should succeed)
    println!("\nBinding ?Customer to \"Alice\" again:");
    let result1 = bindings.bind("Customer".to_string(), Value::String("Alice".to_string()));
    match result1 {
        Ok(_) => println!("  ✓ Success (same value)"),
        Err(e) => println!("  ✗ Failed: {}", e),
    }

    // Conflicting binding (should fail)
    println!("\nBinding ?Customer to \"Bob\" (conflict!):");
    let result2 = bindings.bind("Customer".to_string(), Value::String("Bob".to_string()));
    match result2 {
        Ok(_) => println!("  ✓ Success"),
        Err(e) => println!("  ✗ Failed (expected): {}", e),
    }

    // Merge bindings
    println!("\nMerging bindings:");
    let mut bindings2 = Bindings::new();
    bindings2.bind("Order".to_string(), Value::Number(1234.0))?;
    bindings2.bind("Amount".to_string(), Value::Number(500.0))?;

    println!("  Bindings1: Customer=Alice");
    println!("  Bindings2: Order=1234, Amount=500");

    bindings.merge(&bindings2)?;
    println!("  ✓ Merge successful");
    println!("  Final bindings: {:?}", bindings.as_map());

    // Merge with conflict
    println!("\nMerging with conflicting bindings:");
    let mut bindings3 = Bindings::new();
    bindings3.bind("Customer".to_string(), Value::String("Charlie".to_string()))?;

    let result3 = bindings.merge(&bindings3);
    match result3 {
        Ok(_) => println!("  ✓ Merge successful"),
        Err(e) => println!("  ✗ Merge failed (expected): {}", e),
    }

    println!("\n✓ Demo 5 complete\n");
    Ok(())
}

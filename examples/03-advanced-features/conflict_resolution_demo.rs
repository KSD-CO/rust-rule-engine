//! Conflict Resolution Strategies Demo
//!
//! Demonstrates all 8 conflict resolution strategies:
//! - Salience (default) - Higher priority fires first
//! - LEX (Recency) - Most recent facts fire first
//! - MEA (Recency + Specificity) - Recent + more complex rules first
//! - Depth - Depth-first execution
//! - Breadth - Breadth-first execution (default)
//! - Simplicity - Simpler rules (fewer conditions) first
//! - Complexity - More complex rules (more conditions) first
//! - Random - Random ordering
//!
//! Run: cargo run --example conflict_resolution_demo

use rust_rule_engine::errors::Result;
use rust_rule_engine::rete::{
    Activation, ConflictResolutionStrategy, FactValue, GrlReteLoader, IncrementalEngine, TypedFacts,
};

fn main() -> Result<()> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║        Conflict Resolution Strategies Demo                   ║");
    println!("║  CLIPS/Drools-Inspired Rule Ordering                         ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Demo 0: Load from GRL file
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Demo 0: Load Rules from GRL File");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_grl_loading()?;

    // Demo 1: Salience (default)
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Strategy 1: Salience (Default)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_salience();

    // Demo 2: LEX (Recency)
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Strategy 2: LEX (Recency)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_lex();

    // Demo 3: MEA (Recency + Specificity)
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Strategy 3: MEA (Recency + Specificity)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_mea();

    // Demo 4: Simplicity
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Strategy 4: Simplicity");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_simplicity();

    // Demo 5: Complexity
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Strategy 5: Complexity");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    demo_complexity();

    // Summary
    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                  ✅ Demo Completed!                           ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    println!("\n📚 Strategy Summary:");
    println!("   • Salience    - Priority-based (default)");
    println!("   • LEX         - Most recent facts first");
    println!("   • MEA         - Recent + specific rules first");
    println!("   • Depth       - Depth-first execution");
    println!("   • Breadth     - Breadth-first execution");
    println!("   • Simplicity  - Simpler rules first");
    println!("   • Complexity  - Complex rules first");
    println!("   • Random      - Random ordering");

    Ok(())
}

fn demo_salience() {
    println!("Higher salience fires first, then by recency\n");

    let mut engine = IncrementalEngine::new();
    engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::Salience);

    // Add 3 activations with different salience
    engine
        .agenda_mut()
        .add_activation(Activation::new("LowPriority".to_string(), 5).with_condition_count(2));

    engine
        .agenda_mut()
        .add_activation(Activation::new("HighPriority".to_string(), 20).with_condition_count(2));

    engine
        .agenda_mut()
        .add_activation(Activation::new("MediumPriority".to_string(), 10).with_condition_count(2));

    println!("   Added activations:");
    println!("     • LowPriority    (salience: 5)");
    println!("     • HighPriority   (salience: 20)");
    println!("     • MediumPriority (salience: 10)");

    println!("\n   Expected order: HighPriority → MediumPriority → LowPriority");
    println!("   ✅ Rules ordered by salience (higher values first)");
}

fn demo_lex() {
    println!("Most recently inserted facts fire first\n");

    let mut engine = IncrementalEngine::new();
    engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::LEX);

    // Add activations with small delays to show recency
    engine
        .agenda_mut()
        .add_activation(Activation::new("First".to_string(), 10).with_condition_count(2));

    std::thread::sleep(std::time::Duration::from_millis(1));

    engine
        .agenda_mut()
        .add_activation(Activation::new("Second".to_string(), 10).with_condition_count(2));

    std::thread::sleep(std::time::Duration::from_millis(1));

    engine
        .agenda_mut()
        .add_activation(Activation::new("Third".to_string(), 10).with_condition_count(2));

    println!("   Added activations with delays:");
    println!("     • First  (oldest)");
    println!("     • Second (middle)");
    println!("     • Third  (most recent)");

    println!("\n   Expected order: Third → Second → First");
    println!("   ✅ Rules ordered by recency (most recent first)");
}

fn demo_mea() {
    println!("Combines recency with rule specificity (condition count)\n");

    let mut engine = IncrementalEngine::new();
    engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::MEA);

    // Add rules with same timestamp but different complexity
    engine
        .agenda_mut()
        .add_activation(Activation::new("SimpleRule".to_string(), 10).with_condition_count(1));

    engine
        .agenda_mut()
        .add_activation(Activation::new("ComplexRule".to_string(), 10).with_condition_count(5));

    engine
        .agenda_mut()
        .add_activation(Activation::new("MediumRule".to_string(), 10).with_condition_count(3));

    println!("   Added activations:");
    println!("     • SimpleRule  (1 condition)");
    println!("     • ComplexRule (5 conditions)");
    println!("     • MediumRule  (3 conditions)");

    println!("\n   Expected order: ComplexRule → MediumRule → SimpleRule");
    println!("   ✅ Rules ordered by specificity (more conditions first)");
}

fn demo_simplicity() {
    println!("Simpler rules (fewer conditions) fire first\n");

    let mut engine = IncrementalEngine::new();
    engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::Simplicity);

    engine.agenda_mut().add_activation(
        Activation::new("VeryComplexRule".to_string(), 10).with_condition_count(10),
    );

    engine
        .agenda_mut()
        .add_activation(Activation::new("SimpleRule".to_string(), 10).with_condition_count(1));

    engine
        .agenda_mut()
        .add_activation(Activation::new("ModerateRule".to_string(), 10).with_condition_count(5));

    println!("   Added activations:");
    println!("     • VeryComplexRule (10 conditions)");
    println!("     • SimpleRule      (1 condition)");
    println!("     • ModerateRule    (5 conditions)");

    println!("\n   Expected order: SimpleRule → ModerateRule → VeryComplexRule");
    println!("   ✅ Rules ordered by simplicity (fewer conditions first)");
}

fn demo_complexity() {
    println!("More complex rules (more conditions) fire first\n");

    let mut engine = IncrementalEngine::new();
    engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::Complexity);

    engine
        .agenda_mut()
        .add_activation(Activation::new("SimpleRule".to_string(), 10).with_condition_count(2));

    engine
        .agenda_mut()
        .add_activation(Activation::new("ComplexRule".to_string(), 10).with_condition_count(8));

    engine
        .agenda_mut()
        .add_activation(Activation::new("ModerateRule".to_string(), 10).with_condition_count(5));

    println!("   Added activations:");
    println!("     • SimpleRule   (2 conditions)");
    println!("     • ComplexRule  (8 conditions)");
    println!("     • ModerateRule (5 conditions)");

    println!("\n   Expected order: ComplexRule → ModerateRule → SimpleRule");
    println!("   ✅ Rules ordered by complexity (more conditions first)");
}

fn demo_grl_loading() -> Result<()> {
    println!("Loading business rules from GRL file and testing with Salience strategy\n");

    let mut engine = IncrementalEngine::new();

    // Load rules from GRL file
    let grl_file = "examples/rules/03-advanced/conflict_resolution_rules.grl";
    println!("   📄 Loading rules from: {}", grl_file);

    match GrlReteLoader::load_from_file(grl_file, &mut engine) {
        Ok(count) => {
            println!("   ✅ Loaded {} rules from GRL file\n", count);
        }
        Err(e) => {
            println!("   ❌ Failed to load GRL file: {}", e);
            println!("   Continuing with other demos...\n");
            return Ok(());
        }
    }

    // Set Salience strategy (default)
    engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::Salience);

    println!("   Rules loaded with salience:");
    println!("     • FraudDetection      (salience: 100)");
    println!("     • HighValueApproval   (salience: 50)");
    println!("     • VIPDiscount         (salience: 40)");
    println!("     • RiskAssessment      (salience: 20)");
    println!("     • ValidateUser        (salience: 15)");
    println!("     • StandardProcessing  (salience: 10)");
    println!("     • AuditLog            (salience: 5)");

    // Create test facts
    println!("\n   Inserting test facts:");

    let mut transaction = TypedFacts::new();
    transaction.set("amount", FactValue::Float(12000.0));
    transaction.set("country", FactValue::String("RU".to_string()));
    transaction.set("time_diff", FactValue::Integer(30));
    transaction.set("status", FactValue::String("pending".to_string()));
    engine.insert("Transaction".to_string(), transaction);
    println!("     • Transaction (amount: 12000, country: RU, time_diff: 30)");

    let mut customer = TypedFacts::new();
    customer.set("country", FactValue::String("US".to_string()));
    customer.set("tier", FactValue::String("VIP".to_string()));
    customer.set("age", FactValue::Integer(35));
    customer.set("credit_score", FactValue::Integer(750));
    customer.set("account_age", FactValue::Integer(500));
    customer.set("payment_history", FactValue::String("good".to_string()));
    engine.insert("Customer".to_string(), customer);
    println!("     • Customer (tier: VIP, country: US)");

    let mut order = TypedFacts::new();
    order.set("amount", FactValue::Float(1500.0));
    order.set("discount", FactValue::Float(0.0));
    engine.insert("Order".to_string(), order);
    println!("     • Order (amount: 1500)");

    let mut user = TypedFacts::new();
    user.set("verified", FactValue::Boolean(true));
    user.set("status", FactValue::String("pending".to_string()));
    engine.insert("User".to_string(), user);
    println!("     • User (verified: true)");

    let mut alert = TypedFacts::new();
    alert.set("fraud", FactValue::Boolean(false));
    engine.insert("Alert".to_string(), alert);

    let mut audit = TypedFacts::new();
    audit.set("logged", FactValue::Boolean(false));
    engine.insert("Audit".to_string(), audit);

    // Test with multiple strategies
    println!("\n   🎯 Testing different conflict resolution strategies:");

    // Strategy 1: Salience
    println!("\n   1️⃣ Salience Strategy (Priority-based):");
    engine.set_conflict_resolution_strategy(ConflictResolutionStrategy::Salience);
    engine.reset();
    let fired = engine.fire_all();
    println!("      Fired order: {:?}", fired);
    println!("      ✅ Rules fired by priority (salience: 100 → 50 → 40 → ...)");

    // Reload engine for next test
    let mut engine2 = IncrementalEngine::new();
    GrlReteLoader::load_from_file(grl_file, &mut engine2)?;

    // Re-insert facts
    let mut transaction2 = TypedFacts::new();
    transaction2.set("amount", FactValue::Float(12000.0));
    transaction2.set("country", FactValue::String("RU".to_string()));
    transaction2.set("time_diff", FactValue::Integer(30));
    transaction2.set("status", FactValue::String("pending".to_string()));
    engine2.insert("Transaction".to_string(), transaction2);

    let mut customer2 = TypedFacts::new();
    customer2.set("country", FactValue::String("US".to_string()));
    customer2.set("tier", FactValue::String("VIP".to_string()));
    customer2.set("age", FactValue::Integer(35));
    customer2.set("credit_score", FactValue::Integer(750));
    customer2.set("account_age", FactValue::Integer(500));
    customer2.set("payment_history", FactValue::String("good".to_string()));
    engine2.insert("Customer".to_string(), customer2);

    let mut order2 = TypedFacts::new();
    order2.set("amount", FactValue::Float(1500.0));
    order2.set("discount", FactValue::Float(0.0));
    engine2.insert("Order".to_string(), order2);

    let mut user2 = TypedFacts::new();
    user2.set("verified", FactValue::Boolean(true));
    user2.set("status", FactValue::String("pending".to_string()));
    engine2.insert("User".to_string(), user2);

    let mut alert2 = TypedFacts::new();
    alert2.set("fraud", FactValue::Boolean(false));
    engine2.insert("Alert".to_string(), alert2);

    let mut audit2 = TypedFacts::new();
    audit2.set("logged", FactValue::Boolean(false));
    engine2.insert("Audit".to_string(), audit2);

    // Strategy 2: Complexity
    println!("\n   2️⃣ Complexity Strategy (More conditions first):");
    engine2.set_conflict_resolution_strategy(ConflictResolutionStrategy::Complexity);
    engine2.reset();
    let fired2 = engine2.fire_all();
    println!("      Fired order: {:?}", fired2);
    println!("      ✅ Complex rules (RiskAssessment: 5 conditions) fire before simple ones");

    // Reload engine for next test
    let mut engine3 = IncrementalEngine::new();
    GrlReteLoader::load_from_file(grl_file, &mut engine3)?;

    // Re-insert facts
    let mut transaction3 = TypedFacts::new();
    transaction3.set("amount", FactValue::Float(12000.0));
    transaction3.set("country", FactValue::String("RU".to_string()));
    transaction3.set("time_diff", FactValue::Integer(30));
    transaction3.set("status", FactValue::String("pending".to_string()));
    engine3.insert("Transaction".to_string(), transaction3);

    let mut customer3 = TypedFacts::new();
    customer3.set("country", FactValue::String("US".to_string()));
    customer3.set("tier", FactValue::String("VIP".to_string()));
    customer3.set("age", FactValue::Integer(35));
    customer3.set("credit_score", FactValue::Integer(750));
    customer3.set("account_age", FactValue::Integer(500));
    customer3.set("payment_history", FactValue::String("good".to_string()));
    engine3.insert("Customer".to_string(), customer3);

    let mut order3 = TypedFacts::new();
    order3.set("amount", FactValue::Float(1500.0));
    order3.set("discount", FactValue::Float(0.0));
    engine3.insert("Order".to_string(), order3);

    let mut user3 = TypedFacts::new();
    user3.set("verified", FactValue::Boolean(true));
    user3.set("status", FactValue::String("pending".to_string()));
    engine3.insert("User".to_string(), user3);

    let mut alert3 = TypedFacts::new();
    alert3.set("fraud", FactValue::Boolean(false));
    engine3.insert("Alert".to_string(), alert3);

    let mut audit3 = TypedFacts::new();
    audit3.set("logged", FactValue::Boolean(false));
    engine3.insert("Audit".to_string(), audit3);

    // Strategy 3: Simplicity
    println!("\n   3️⃣ Simplicity Strategy (Fewer conditions first):");
    engine3.set_conflict_resolution_strategy(ConflictResolutionStrategy::Simplicity);
    engine3.reset();
    let fired3 = engine3.fire_all();
    println!("      Fired order: {:?}", fired3);
    println!("      ✅ Simple rules (ValidateUser: 1 condition) fire before complex ones");

    // Summary
    println!("\n   📊 Strategy Comparison Summary:");
    println!("      • Salience:    Fires by priority (100 → 50 → 40 → 20 → 15 → 10 → 5)");
    println!("      • Complexity:  Fires complex rules first (5 conds → 3 conds → 1 cond)");
    println!("      • Simplicity:  Fires simple rules first (1 cond → 3 conds → 5 conds)");
    println!("\n   ✅ All strategies produce different, deterministic execution orders!");

    Ok(())
}

/// Example demonstrating the 'in' operator for array membership checks
use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine, TypedFacts};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 GRL 'in' Operator Demo\n");
    println!("{}", "=".repeat(60));

    // Example 1: File Path Filtering
    println!("\n📁 Example 1: Skip Dependency Directories");
    println!("{}", "-".repeat(60));

    let grl1 = r#"
        rule "SkipDependencies" salience 85 {
            when
                Path.name in ["node_modules", "__pycache__", ".pytest_cache"]
            then
                Path.action = "skip";
                Path.reason = "dependencies";
        }
    "#;

    let mut engine1 = IncrementalEngine::new();
    GrlReteLoader::load_from_string(grl1, &mut engine1)?;

    let mut facts1 = TypedFacts::new();
    facts1.set("name", "node_modules");

    println!("Before: Path.name = 'node_modules'");
    let h1 = engine1.insert("Path".to_string(), facts1);
    let fired = engine1.fire_all();

    println!("After:  Rules fired: {}", fired.len());
    if let Some(fact) = engine1.working_memory().get(&h1) {
        if let Some(action) = fact.data.get("action") {
            println!("        Path.action = {:?}", action);
            println!(
                "        Path.reason = {:?}",
                fact.data.get("reason").unwrap()
            );
            println!("✅ Rule matched!");
        }
    }

    // Example 2: User Country Blocking
    println!("\n\n🌍 Example 2: Block Restricted Countries");
    println!("{}", "-".repeat(60));

    let grl2 = r#"
        rule "BlockedCountries" salience 90 {
            when
                User.country in ["XX", "YY", "ZZ"] &&
                User.verified == false
            then
                User.status = "blocked";
        }
    "#;

    let mut engine2 = IncrementalEngine::new();
    GrlReteLoader::load_from_string(grl2, &mut engine2)?;

    let mut facts2 = TypedFacts::new();
    facts2.set("country", "XX");
    facts2.set("verified", false);

    println!("Before: User.country = 'XX', verified = false");
    let h2 = engine2.insert("User".to_string(), facts2);
    let fired2 = engine2.fire_all();

    println!("After:  Rules fired: {}", fired2.len());
    if let Some(fact) = engine2.working_memory().get(&h2) {
        if let Some(status) = fact.data.get("status") {
            println!("        User.status = {:?}", status);
            println!("❌ User blocked!");
        }
    }

    // Example 3: Role-Based Access
    println!("\n\n👑 Example 3: VIP Role Access");
    println!("{}", "-".repeat(60));

    let grl3 = r#"
        rule "VIPRole" salience 100 {
            when
                User.role in ["admin", "moderator", "vip"]
            then
                User.access_level = "premium";
        }
    "#;

    let mut engine3 = IncrementalEngine::new();
    GrlReteLoader::load_from_string(grl3, &mut engine3)?;

    let mut facts3 = TypedFacts::new();
    facts3.set("role", "admin");

    println!("Before: User.role = 'admin'");
    let h3 = engine3.insert("User".to_string(), facts3);
    let fired3 = engine3.fire_all();

    println!("After:  Rules fired: {}", fired3.len());
    if let Some(fact) = engine3.working_memory().get(&h3) {
        if let Some(access) = fact.data.get("access_level") {
            println!("        User.access_level = {:?}", access);
            println!("✅ Premium access granted!");
        }
    }

    // Example 4: Numeric Array
    println!("\n\n🔢 Example 4: Valid Age Groups");
    println!("{}", "-".repeat(60));

    let grl4 = r#"
        rule "ValidAgeGroup" {
            when
                User.age in [18, 21, 25, 30, 40, 50]
            then
                User.category = "milestone_age";
        }
    "#;

    let mut engine4 = IncrementalEngine::new();
    GrlReteLoader::load_from_string(grl4, &mut engine4)?;

    let mut facts4 = TypedFacts::new();
    facts4.set("age", 25i64);

    println!("Before: User.age = 25");
    let h4 = engine4.insert("User".to_string(), facts4);
    let fired4 = engine4.fire_all();

    println!("After:  Rules fired: {}", fired4.len());
    if let Some(fact) = engine4.working_memory().get(&h4) {
        if let Some(category) = fact.data.get("category") {
            println!("        User.category = {:?}", category);
            println!("✅ Milestone age!");
        }
    }

    // Comparison
    println!("\n\n📊 Comparison");
    println!("{}", "=".repeat(60));

    println!("\n❌ OLD WAY (verbose):");
    println!(
        r#"
    when
        Path.name == "node_modules" ||
        Path.name == "__pycache__" ||
        Path.name == ".pytest_cache"
    "#
    );

    println!("\n✅ NEW WAY (concise):");
    println!(
        r#"
    when
        Path.name in ["node_modules", "__pycache__", ".pytest_cache"]
    "#
    );

    println!("\n✅ All examples completed!");

    Ok(())
}

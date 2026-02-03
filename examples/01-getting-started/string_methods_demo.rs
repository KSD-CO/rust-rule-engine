/// Example demonstrating string methods: startsWith, endsWith, contains, matches
use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine, TypedFacts};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔤 GRL String Methods Demo\n");
    println!("{}", "=".repeat(70));

    // Example 1: Email Validation with startsWith
    println!("\n📧 Example 1: Email Domain Check (startsWith)");
    println!("{}", "-".repeat(70));

    let grl1 = r#"
        rule "AdminEmailCheck" salience 90 {
            when
                User.email startsWith "admin@" &&
                User.access != "granted"
            then
                User.access = "granted";
                User.role = "administrator";
        }
    "#;

    let mut engine1 = IncrementalEngine::new();
    GrlReteLoader::load_from_string(grl1, &mut engine1)?;

    let mut facts1 = TypedFacts::new();
    facts1.set("email", "admin@company.com");
    facts1.set("access", "pending");

    println!("Before: email = 'admin@company.com', access = 'pending'");
    let h1 = engine1.insert("User".to_string(), facts1);
    let fired = engine1.fire_all();

    if fired.len() == 1 {
        if let Some(fact) = engine1.working_memory().get(&h1) {
            println!("After:  Rules fired: {}", fired.len());
            println!(
                "        User.access = {:?}",
                fact.data.get("access").unwrap()
            );
            println!("        User.role = {:?}", fact.data.get("role").unwrap());
            println!("✅ Admin access granted based on email prefix!");
        }
    } else {
        println!("⚠️  Rules fired: {} times (infinite loop)", fired.len());
    }

    // Example 2: File Extension Check with endsWith
    println!("\n\n📁 Example 2: File Type Detection (endsWith)");
    println!("{}", "-".repeat(70));

    let grl2 = r#"
        rule "ImageFileDetection" salience 85 {
            when
                File.name endsWith ".jpg" ||
                File.name endsWith ".png" ||
                File.name endsWith ".gif"
            then
                File.type = "image";
                File.handler = "ImageProcessor";
        }
    "#;

    let mut engine2 = IncrementalEngine::new();
    GrlReteLoader::load_from_string(grl2, &mut engine2)?;

    let mut facts2 = TypedFacts::new();
    facts2.set("name", "profile-photo.png");

    println!("Before: File.name = 'profile-photo.png'");
    let h2 = engine2.insert("File".to_string(), facts2);
    let fired2 = engine2.fire_all();

    if fired2.len() == 1 {
        if let Some(fact) = engine2.working_memory().get(&h2) {
            println!("After:  Rules fired: {}", fired2.len());
            println!("        File.type = {:?}", fact.data.get("type").unwrap());
            println!(
                "        File.handler = {:?}",
                fact.data.get("handler").unwrap()
            );
            println!("✅ Image file detected based on extension!");
        }
    } else {
        println!("⚠️  Rules fired: {} times (infinite loop)", fired2.len());
    }

    // Example 3: Contains operator for substring search
    println!("\n\n🔍 Example 3: Keyword Detection (contains)");
    println!("{}", "-".repeat(70));

    let grl3 = r#"
        rule "SpamDetection" salience 95 {
            when
                Message.content contains "FREE MONEY" &&
                Message.flagged != true
            then
                Message.flagged = true;
                Message.category = "spam";
        }
    "#;

    let mut engine3 = IncrementalEngine::new();
    GrlReteLoader::load_from_string(grl3, &mut engine3)?;

    let mut facts3 = TypedFacts::new();
    facts3.set("content", "Click here for FREE MONEY today!");
    facts3.set("flagged", false);

    println!("Before: Message.content = 'Click here for FREE MONEY today!'");
    let h3 = engine3.insert("Message".to_string(), facts3);
    let fired3 = engine3.fire_all();

    if fired3.len() == 1 {
        if let Some(fact) = engine3.working_memory().get(&h3) {
            println!("After:  Rules fired: {}", fired3.len());
            println!(
                "        Message.flagged = {:?}",
                fact.data.get("flagged").unwrap()
            );
            println!(
                "        Message.category = {:?}",
                fact.data.get("category").unwrap()
            );
            println!("🚨 Spam detected based on keyword!");
        }
    } else {
        println!("⚠️  Rules fired: {} times (infinite loop)", fired3.len());
    }

    // Example 4: Wildcard Pattern Matching with matches
    println!("\n\n🎯 Example 4: Pattern Matching (matches)");
    println!("{}", "-".repeat(70));

    let grl4 = r#"
        rule "VersionPatternCheck" salience 80 {
            when
                Package.version matches "1.*.0" &&
                Package.status != "validated"
            then
                Package.status = "validated";
                Package.series = "1.x.0";
        }
    "#;

    let mut engine4 = IncrementalEngine::new();
    GrlReteLoader::load_from_string(grl4, &mut engine4)?;

    let mut facts4 = TypedFacts::new();
    facts4.set("version", "1.18.0");
    facts4.set("status", "pending");

    println!("Before: Package.version = '1.18.0', status = 'pending'");
    let h4 = engine4.insert("Package".to_string(), facts4);
    let fired4 = engine4.fire_all();

    if fired4.len() == 1 {
        if let Some(fact) = engine4.working_memory().get(&h4) {
            println!("After:  Rules fired: {}", fired4.len());
            println!(
                "        Package.status = {:?}",
                fact.data.get("status").unwrap()
            );
            println!(
                "        Package.series = {:?}",
                fact.data.get("series").unwrap()
            );
            println!("✅ Version pattern matched!");
        }
    } else {
        println!("⚠️  Rules fired: {} times (infinite loop)", fired4.len());
    }

    // Summary
    println!("\n\n📊 String Method Summary");
    println!("{}", "=".repeat(70));
    println!("✅ startsWith  - Check if string begins with prefix");
    println!("✅ endsWith    - Check if string ends with suffix");
    println!("✅ contains    - Check if string contains substring");
    println!("✅ matches     - Wildcard pattern matching (* and ?)");
    println!("\nAll string methods are now supported in GRL parser! 🎉");

    Ok(())
}

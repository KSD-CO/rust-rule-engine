use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, ParallelConfig, ParallelRuleEngine, RustRuleEngine};
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("⚠️  Rule Dependencies in Parallel Execution Analysis");
    println!("====================================================\n");

    // Demo 1: Safe parallel rules (independent)
    demo_safe_parallel_rules()?;

    // Demo 2: Dangerous parallel rules (dependent)
    demo_dangerous_parallel_rules()?;

    // Demo 3: Same salience dependency conflicts
    demo_same_salience_conflicts()?;

    println!("\n🎯 Key Takeaways:");
    println!("   ✅ Rules at DIFFERENT salience levels are safe (sequential execution)");
    println!("   ⚠️  Rules at SAME salience level can have hidden dependencies");
    println!("   🚨 Current implementation doesn't analyze dependencies");
    println!("   🔧 Need dependency analysis for safe parallel execution");

    Ok(())
}

fn demo_safe_parallel_rules() -> Result<(), Box<dyn std::error::Error>> {
    println!("✅ Demo 1: Safe Parallel Rules (Independent)");
    println!("---------------------------------------------");

    let facts = create_facts_for_demo();
    let kb = create_safe_parallel_kb()?;

    println!("🔧 Rules in this demo:");
    for rule in kb.get_rules() {
        println!("   📋 {}: salience {}", rule.name, rule.salience);
    }

    println!("\n🚀 These rules are SAFE to run in parallel because:");
    println!("   - Different data fields (User.Age vs User.Country vs Order.Amount)");
    println!("   - No shared state modifications");
    println!("   - Read-only operations");

    let engine = ParallelRuleEngine::new(ParallelConfig {
        enabled: true,
        max_threads: 4,
        min_rules_per_thread: 1,
        dependency_analysis: false,
    });

    let result = engine.execute_parallel(&kb, &facts, true)?;
    println!(
        "\n📊 Result: {} rules fired safely in parallel",
        result.total_rules_fired
    );

    Ok(())
}

fn demo_dangerous_parallel_rules() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚨 Demo 2: Dangerous Parallel Rules (Dependent)");
    println!("------------------------------------------------");

    let facts = create_facts_for_demo();
    let kb = create_dangerous_parallel_kb()?;

    println!("🔧 Rules in this demo:");
    for rule in kb.get_rules() {
        println!("   📋 {}: salience {}", rule.name, rule.salience);
    }

    println!("\n⚠️  These rules are DANGEROUS to run in parallel because:");
    println!("   - Rule1 calculates User.Score");
    println!("   - Rule2 depends on User.Score for VIP status");
    println!("   - Rule3 depends on User.IsVIP for discount");
    println!("   - Race condition: Rule2/Rule3 might read Score before Rule1 sets it!");

    // Test sequential first
    println!("\n🐌 Sequential execution (correct order):");
    let mut sequential_engine = RustRuleEngine::with_config(
        kb.clone(),
        EngineConfig {
            debug_mode: true,
            max_cycles: 1,
            ..Default::default()
        },
    );

    // Register functions for sequential engine
    register_sequential_functions(&mut sequential_engine);
    let _sequential_result = sequential_engine.execute(&facts)?;

    // Test parallel (potential issues)
    println!("\n⚡ Parallel execution (potential race conditions):");
    let mut parallel_engine = ParallelRuleEngine::new(ParallelConfig {
        enabled: true,
        max_threads: 4,
        min_rules_per_thread: 1,
        dependency_analysis: false,
    });

    // Register functions for parallel engine
    register_parallel_functions(&mut parallel_engine);
    let _parallel_result = parallel_engine.execute_parallel(&kb, &facts, true)?;

    println!("\n🔍 Analysis:");
    println!("   - Sequential: Rules execute in salience order → correct dependencies");
    println!("   - Parallel: Rules at same salience may execute out of order → race conditions");

    Ok(())
}

fn demo_same_salience_conflicts() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n💥 Demo 3: Same Salience Dependency Conflicts");
    println!("----------------------------------------------");

    let facts = create_facts_for_demo();
    let kb = create_conflict_kb()?;

    println!("🔧 Rules with SAME salience (will run in parallel):");
    for rule in kb.get_rules() {
        println!("   📋 {}: salience {}", rule.name, rule.salience);
    }

    println!("\n🚨 CRITICAL ISSUE:");
    println!("   - All rules have salience 10 (same priority)");
    println!("   - Step1: calculates total");
    println!("   - Step2: applies discount based on total");
    println!("   - Step3: calculates final price based on discounted total");
    println!("   - Running in parallel = UNDEFINED BEHAVIOR!");

    let engine = ParallelRuleEngine::new(ParallelConfig {
        enabled: true,
        max_threads: 3,
        min_rules_per_thread: 1,
        dependency_analysis: false,
    });

    let result = engine.execute_parallel(&kb, &facts, true)?;
    println!(
        "\n📊 Result: {} rules fired (but in what order?)",
        result.total_rules_fired
    );

    println!("\n💡 Solution needed:");
    println!("   1. 🔍 Dependency analysis to detect read/write conflicts");
    println!("   2. 📊 Graph analysis to find dependency chains");
    println!("   3. 🚦 Sequential execution for dependent rules");
    println!("   4. ⚡ Parallel execution only for independent rules");

    Ok(())
}

fn create_facts_for_demo() -> Facts {
    let facts = Facts::new();
    facts.set("User", {
        let mut user = HashMap::new();
        user.insert("Age".to_string(), Value::Number(25.0));
        user.insert("Country".to_string(), Value::String("US".to_string()));
        user.insert("SpendingTotal".to_string(), Value::Number(1500.0));
        user.insert("Score".to_string(), Value::Number(0.0)); // Will be calculated
        user.insert("IsVIP".to_string(), Value::Boolean(false)); // Will be set
        Value::Object(user)
    });

    facts.set("Order", {
        let mut order = HashMap::new();
        order.insert("Amount".to_string(), Value::Number(100.0));
        order.insert("Total".to_string(), Value::Number(0.0)); // Will be calculated
        order.insert("DiscountRate".to_string(), Value::Number(0.0)); // Will be set
        order.insert("FinalAmount".to_string(), Value::Number(0.0)); // Will be calculated
        Value::Object(order)
    });

    facts
}

fn create_safe_parallel_kb() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let mut kb = KnowledgeBase::new("SafeParallelKB");

    let rules = vec![
        // These rules are SAFE to run in parallel - no dependencies
        r#"rule "AgeValidation" salience 10 {
            when User.Age >= 18
            then validateAge("User is adult");
        }"#,
        r#"rule "CountryCheck" salience 10 {
            when User.Country == "US"
            then checkCountry("US customer");
        }"#,
        r#"rule "OrderAmountCheck" salience 10 {
            when Order.Amount > 50.0
            then validateOrder("Order amount is valid");
        }"#,
    ];

    for rule_str in rules {
        kb.add_rules_from_grl(rule_str)?;
    }

    Ok(kb)
}

fn create_dangerous_parallel_kb() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let mut kb = KnowledgeBase::new("DangerousParallelKB");

    let rules = vec![
        // These rules have DEPENDENCIES - dangerous to run in parallel
        r#"rule "CalculateScore" salience 10 {
            when User.SpendingTotal > 1000.0
            then calculateScore("Calculating user score");
        }"#,
        r#"rule "CheckVIPStatus" salience 10 {
            when User.Score >= 80.0
            then setVIPStatus("Setting VIP status");
        }"#,
        r#"rule "ApplyVIPDiscount" salience 10 {
            when User.IsVIP == true
            then applyDiscount("Applying VIP discount");
        }"#,
    ];

    for rule_str in rules {
        kb.add_rules_from_grl(rule_str)?;
    }

    Ok(kb)
}

fn create_conflict_kb() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let mut kb = KnowledgeBase::new("ConflictKB");

    let rules = vec![
        // ALL same salience - will run in parallel but have dependencies!
        r#"rule "Step1_CalculateTotal" salience 10 {
            when Order.Amount > 0.0
            then calculateTotal("Step 1: Calculate total");
        }"#,
        r#"rule "Step2_ApplyDiscount" salience 10 {
            when Order.Total > 100.0
            then applyOrderDiscount("Step 2: Apply discount");
        }"#,
        r#"rule "Step3_CalculateFinal" salience 10 {
            when Order.DiscountRate > 0.0
            then calculateFinalAmount("Step 3: Calculate final");
        }"#,
    ];

    for rule_str in rules {
        kb.add_rules_from_grl(rule_str)?;
    }

    Ok(kb)
}

fn register_sequential_functions(engine: &mut RustRuleEngine) {
    engine.register_function(
        "calculateScore",
        Box::new(|args: &[Value], facts: &Facts| {
            println!("     🧮 Calculating score...");
            // In real implementation, would update facts
            Ok(Value::Number(85.0))
        }),
    );

    engine.register_function(
        "setVIPStatus",
        Box::new(|args: &[Value], facts: &Facts| {
            println!("     ⭐ Setting VIP status...");
            // In real implementation, would update facts
            Ok(Value::Boolean(true))
        }),
    );

    engine.register_function(
        "applyDiscount",
        Box::new(|args: &[Value], facts: &Facts| {
            println!("     💰 Applying discount...");
            Ok(Value::Null)
        }),
    );
}

fn register_parallel_functions(engine: &mut ParallelRuleEngine) {
    engine.register_function("validateAge", |args: &[Value], _facts: &Facts| {
        println!("     ✅ Age validation");
        Ok(Value::Null)
    });

    engine.register_function("checkCountry", |args: &[Value], _facts: &Facts| {
        println!("     🌎 Country check");
        Ok(Value::Null)
    });

    engine.register_function("validateOrder", |args: &[Value], _facts: &Facts| {
        println!("     🛒 Order validation");
        Ok(Value::Null)
    });

    engine.register_function("calculateScore", |args: &[Value], _facts: &Facts| {
        println!("     🧮 Calculating score (parallel)");
        Ok(Value::Number(85.0))
    });

    engine.register_function("setVIPStatus", |args: &[Value], _facts: &Facts| {
        println!("     ⭐ Setting VIP status (parallel)");
        Ok(Value::Boolean(true))
    });

    engine.register_function("applyDiscount", |args: &[Value], _facts: &Facts| {
        println!("     💰 Applying discount (parallel)");
        Ok(Value::Null)
    });

    engine.register_function("calculateTotal", |args: &[Value], _facts: &Facts| {
        println!("     📊 Step 1: Calculating total");
        Ok(Value::Number(150.0))
    });

    engine.register_function("applyOrderDiscount", |args: &[Value], _facts: &Facts| {
        println!("     🏷️  Step 2: Applying discount");
        Ok(Value::Number(0.1))
    });

    engine.register_function("calculateFinalAmount", |args: &[Value], _facts: &Facts| {
        println!("     💵 Step 3: Calculating final amount");
        Ok(Value::Number(135.0))
    });
}

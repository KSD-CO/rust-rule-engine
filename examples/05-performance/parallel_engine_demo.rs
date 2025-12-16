use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, ParallelConfig, ParallelRuleEngine, RustRuleEngine};
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Parallel Rule Engine Demo");
    println!("=============================\n");

    // Demo 1: Performance comparison
    demo_performance_comparison()?;

    // Demo 2: Parallel configuration options
    demo_parallel_configuration()?;

    // Demo 3: Large scale parallel execution
    demo_large_scale_parallel()?;

    println!("\n✅ Parallel Rule Engine demonstrated successfully!");
    println!("🎯 Key Benefits:");
    println!("   - ⚡ Faster execution for large rule sets");
    println!("   - 🧵 Multi-core CPU utilization");
    println!("   - 🔧 Configurable parallelization");
    println!("   - 📊 Performance monitoring");

    Ok(())
}

fn demo_performance_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Demo 1: Performance Comparison");
    println!("----------------------------------");

    // Create test data
    let facts = create_test_facts();
    let kb = create_performance_test_kb()?;

    println!(
        "🔧 Created {} rules for performance testing",
        kb.get_rules().len()
    );

    // Test 1: Sequential execution
    println!("\n🐌 Sequential Execution:");
    let start = Instant::now();
    let mut sequential_engine = RustRuleEngine::with_config(
        kb.clone(),
        EngineConfig {
            debug_mode: false,
            max_cycles: 1,
            ..Default::default()
        },
    );
    register_test_functions(&mut sequential_engine);
    let sequential_result = sequential_engine.execute(&facts)?;
    let sequential_time = start.elapsed();

    println!("   ⏱️  Time: {:?}", sequential_time);
    println!("   🔥 Rules fired: {}", sequential_result.rules_fired);

    // Test 2: Parallel execution
    println!("\n⚡ Parallel Execution:");
    let start = Instant::now();
    let mut parallel_engine = ParallelRuleEngine::new(ParallelConfig::default());
    register_test_functions_parallel(&mut parallel_engine);
    let parallel_result = parallel_engine.execute_parallel(&kb, &facts, false)?;
    let parallel_time = start.elapsed();

    println!("   ⏱️  Time: {:?}", parallel_time);
    println!("   {}", parallel_result.get_stats());

    // Performance comparison
    if sequential_time > parallel_time {
        let speedup = sequential_time.as_millis() as f64 / parallel_time.as_millis() as f64;
        println!("\n🚀 Parallel execution is {:.2}x faster!", speedup);
    } else {
        println!("\n⚠️  Sequential was faster (threading overhead for small rule sets)");
    }

    Ok(())
}

fn demo_parallel_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 2: Parallel Configuration Options");
    println!("------------------------------------------");

    let facts = create_test_facts();
    let kb = create_performance_test_kb()?;

    // Test different configurations
    let configs = vec![
        ("Default", ParallelConfig::default()),
        (
            "High Parallelism",
            ParallelConfig {
                enabled: true,
                max_threads: 8,
                min_rules_per_thread: 1,
                dependency_analysis: true,
            },
        ),
        (
            "Conservative",
            ParallelConfig {
                enabled: true,
                max_threads: 2,
                min_rules_per_thread: 5,
                dependency_analysis: true,
            },
        ),
        (
            "Disabled",
            ParallelConfig {
                enabled: false,
                ..Default::default()
            },
        ),
    ];

    for (name, config) in configs {
        println!("\n🔧 Testing {} configuration:", name);
        println!("   Max threads: {}", config.max_threads);
        println!("   Min rules per thread: {}", config.min_rules_per_thread);
        println!("   Enabled: {}", config.enabled);

        let start = Instant::now();
        let mut engine = ParallelRuleEngine::new(config);
        register_test_functions_parallel(&mut engine);
        let result = engine.execute_parallel(&kb, &facts, false)?;
        let execution_time = start.elapsed();

        println!("   ⏱️  Execution time: {:?}", execution_time);
        println!("   🔥 Rules fired: {}", result.total_rules_fired);
        println!("   📈 Speedup: {:.2}x", result.parallel_speedup);
    }

    Ok(())
}

fn demo_large_scale_parallel() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📋 Demo 3: Large Scale Parallel Execution");
    println!("------------------------------------------");

    // Create a large knowledge base
    let kb = create_large_scale_kb(50)?; // 50 rules
    let facts = create_test_facts();

    println!(
        "🏗️  Created knowledge base with {} rules",
        kb.get_rules().len()
    );

    // Test with different thread counts
    let thread_counts = vec![1, 2, 4, 8];

    for thread_count in thread_counts {
        println!("\n🧵 Testing with {} threads:", thread_count);

        let config = ParallelConfig {
            enabled: true,
            max_threads: thread_count,
            min_rules_per_thread: 1,
            dependency_analysis: true,
        };

        let start = Instant::now();
        let mut engine = ParallelRuleEngine::new(config);
        register_test_functions_parallel(&mut engine);
        let result = engine.execute_parallel(&kb, &facts, false)?;
        let execution_time = start.elapsed();

        println!("   ⏱️  Time: {:?}", execution_time);
        println!("   🔥 Rules fired: {}", result.total_rules_fired);
        println!("   📈 Theoretical speedup: {:.2}x", result.parallel_speedup);
        println!(
            "   📊 Rules per second: {:.0}",
            result.total_rules_evaluated as f64 / execution_time.as_secs_f64()
        );
    }

    Ok(())
}

fn create_test_facts() -> Facts {
    let facts = Facts::new();
    facts.set("User", {
        let mut user = HashMap::new();
        user.insert("Age".to_string(), Value::Number(25.0));
        user.insert("Country".to_string(), Value::String("US".to_string()));
        user.insert("SpendingTotal".to_string(), Value::Number(1500.0));
        user.insert("IsVIP".to_string(), Value::Boolean(false));
        user.insert(
            "Category".to_string(),
            Value::String("standard".to_string()),
        );
        Value::Object(user)
    });

    facts.set("Order", {
        let mut order = HashMap::new();
        order.insert("Amount".to_string(), Value::Number(100.0));
        order.insert(
            "Category".to_string(),
            Value::String("electronics".to_string()),
        );
        order.insert("ItemCount".to_string(), Value::Number(3.0));
        Value::Object(order)
    });

    facts
}

fn create_performance_test_kb() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new("PerformanceTestKB");

    let rules = vec![
        r#"rule "AgeValidation" salience 10 {
            when User.Age >= 18
            then validateAge("adult");
        }"#,
        r#"rule "CountryCheck" salience 10 {
            when User.Country == "US"
            then processCountry("US processing");
        }"#,
        r#"rule "SpendingAnalysis" salience 10 {
            when User.SpendingTotal > 1000.0
            then analyzeSpending("high spender");
        }"#,
        r#"rule "VIPCheck" salience 9 {
            when User.IsVIP == false
            then checkVIPStatus("standard user");
        }"#,
        r#"rule "CategoryProcessing" salience 9 {
            when User.Category == "standard"
            then processCategory("standard processing");
        }"#,
        r#"rule "OrderValidation" salience 8 {
            when Order.Amount > 50.0
            then validateOrder("order valid");
        }"#,
        r#"rule "ItemCountCheck" salience 8 {
            when Order.ItemCount >= 2.0
            then checkItemCount("multiple items");
        }"#,
        r#"rule "ElectronicsRule" salience 7 {
            when Order.Category == "electronics"
            then processElectronics("electronics order");
        }"#,
    ];

    for rule_str in rules {
        kb.add_rules_from_grl(rule_str)?;
    }

    Ok(kb)
}

fn create_large_scale_kb(rule_count: usize) -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new("LargeScaleKB");

    for i in 0..rule_count {
        let salience = 10 - (i % 10) as i32; // Vary salience from 1-10
        let rule_str = format!(
            r#"rule "Rule{}" salience {} {{
                when User.Age >= {}
                then processRule("Rule {} executed");
            }}"#,
            i,
            salience,
            i % 30 + 18,
            i
        );
        kb.add_rules_from_grl(&rule_str)?;
    }

    Ok(kb)
}

fn register_test_functions(engine: &mut RustRuleEngine) {
    engine.register_function("validateAge", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     ✅ Age validation: {}", msg);
        }
        Ok(Value::Null)
    });

    // Also register as an action handler in case the GRL emits a Custom action
    engine.register_action_handler(
        "validateAge",
        |params: &std::collections::HashMap<String, Value>, _facts: &Facts| {
            if let Some(v) = params.get("0") {
                match v {
                    Value::String(msg) => println!("     ✅ Age validation (action): {}", msg),
                    _ => println!(
                        "     ✅ Age validation (action) with non-string param: {:?}",
                        v
                    ),
                }
            }
            Ok(())
        },
    );

    // Register other action handlers used by the performance KB rules
    engine.register_action_handler(
        "processCountry",
        |params: &std::collections::HashMap<String, Value>, _facts: &Facts| {
            if let Some(Value::String(msg)) = params.get("0") {
                println!("     🌎 Country processing (action): {}", msg);
            }
            Ok(())
        },
    );

    engine.register_action_handler(
        "analyzeSpending",
        |params: &std::collections::HashMap<String, Value>, _facts: &Facts| {
            if let Some(Value::String(msg)) = params.get("0") {
                println!("     💰 Spending analysis (action): {}", msg);
            }
            Ok(())
        },
    );

    engine.register_action_handler(
        "checkVIPStatus",
        |params: &std::collections::HashMap<String, Value>, _facts: &Facts| {
            if let Some(Value::String(msg)) = params.get("0") {
                println!("     ⭐ VIP check (action): {}", msg);
            }
            Ok(())
        },
    );

    engine.register_action_handler(
        "processCategory",
        |params: &std::collections::HashMap<String, Value>, _facts: &Facts| {
            if let Some(Value::String(msg)) = params.get("0") {
                println!("     📂 Category processing (action): {}", msg);
            }
            Ok(())
        },
    );

    engine.register_action_handler(
        "validateOrder",
        |params: &std::collections::HashMap<String, Value>, _facts: &Facts| {
            if let Some(Value::String(msg)) = params.get("0") {
                println!("     🛒 Order validation (action): {}", msg);
            }
            Ok(())
        },
    );

    engine.register_action_handler(
        "checkItemCount",
        |params: &std::collections::HashMap<String, Value>, _facts: &Facts| {
            if let Some(Value::String(msg)) = params.get("0") {
                println!("     📦 Item count check (action): {}", msg);
            }
            Ok(())
        },
    );

    engine.register_action_handler(
        "processElectronics",
        |params: &std::collections::HashMap<String, Value>, _facts: &Facts| {
            if let Some(Value::String(msg)) = params.get("0") {
                println!("     ⚡ Electronics processing (action): {}", msg);
            }
            Ok(())
        },
    );

    engine.register_action_handler(
        "processRule",
        |params: &std::collections::HashMap<String, Value>, _facts: &Facts| {
            if let Some(Value::String(msg)) = params.get("0") {
                println!("     🔧 {}", msg);
            }
            Ok(())
        },
    );

    engine.register_function("processCountry", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     🌎 Country processing: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("analyzeSpending", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     💰 Spending analysis: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("checkVIPStatus", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     ⭐ VIP check: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("processCategory", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     📂 Category processing: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("validateOrder", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     🛒 Order validation: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("checkItemCount", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     📦 Item count check: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("processElectronics", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     ⚡ Electronics processing: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("processRule", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     🔧 {}", msg);
        }
        Ok(Value::Null)
    });
}

fn register_test_functions_parallel(engine: &mut ParallelRuleEngine) {
    engine.register_function("validateAge", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     ✅ Age validation: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("processCountry", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     🌎 Country processing: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("analyzeSpending", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     💰 Spending analysis: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("checkVIPStatus", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     ⭐ VIP check: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("processCategory", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     📂 Category processing: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("validateOrder", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     🛒 Order validation: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("checkItemCount", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     📦 Item count check: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("processElectronics", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     ⚡ Electronics processing: {}", msg);
        }
        Ok(Value::Null)
    });

    engine.register_function("processRule", |args: &[Value], _facts| {
        if let Some(Value::String(msg)) = args.first() {
            println!("     🔧 {}", msg);
        }
        Ok(Value::Null)
    });
}

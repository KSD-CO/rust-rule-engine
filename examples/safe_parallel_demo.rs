/// Demo: Safe Parallel Rule Execution with Dependency Analysis
/// This example shows how the enhanced parallel engine automatically detects
/// rule dependencies and chooses the safest execution strategy.

use rust_rule_engine::engine::{
    SafeParallelRuleEngine, SafeParallelConfig, ExecutionStrategy,
    rule::{Rule, Condition, ConditionGroup},
    facts::Facts,
};
use rust_rule_engine::types::{ActionType, ComparisonOperator, LogicalOperator, Value};
use std::time::Instant;

fn main() {
    println!("🛡️  SAFE PARALLEL RULE EXECUTION DEMO");
    println!("=====================================\n");

    // Demo 1: Safe Independent Rules (Should parallelize)
    demo_safe_independent_rules();
    
    println!("\n" + "=".repeat(60).as_str() + "\n");
    
    // Demo 2: Dangerous Dependent Rules (Should stay sequential)
    demo_dangerous_dependent_rules();
    
    println!("\n" + "=".repeat(60).as_str() + "\n");
    
    // Demo 3: Mixed Rules (Should use hybrid execution)
    demo_mixed_rules();
    
    println!("\n" + "=".repeat(60).as_str() + "\n");
    
    // Demo 4: Configuration Options
    demo_configuration_options();
}

fn demo_safe_independent_rules() {
    println!("🟢 DEMO 1: Safe Independent Rules");
    println!("   These rules operate on different data fields and can safely run in parallel\n");
    
    let mut engine = SafeParallelRuleEngine::new(SafeParallelConfig {
        analyze_dependencies: true,
        enable_logging: true,
        max_threads: 4,
        ..Default::default()
    });
    
    // Create rules that don't interfere with each other
    let rules = vec![
        create_rule("AgeValidation", "User.Age", 18, 100),
        create_rule("CountryCheck", "User.Country", "US", 50),
        create_rule("OrderAmountCheck", "Order.Amount", 100.0, 30),
        create_rule("EmailValidation", "User.Email", "@", 20),
    ];
    
    let mut facts = Facts::new();
    facts.set("User.Age".to_string(), Value::Integer(25));
    facts.set("User.Country".to_string(), Value::String("US".to_string()));
    facts.set("Order.Amount".to_string(), Value::Float(150.0));
    facts.set("User.Email".to_string(), Value::String("user@test.com".to_string()));
    
    println!("📊 Rules to execute:");
    for rule in &rules {
        println!("   - {} (salience: {})", rule.name, rule.salience);
    }
    
    let start_time = Instant::now();
    let result = engine.execute_rules_safe_parallel(&rules, &mut facts);
    let total_time = start_time.elapsed();
    
    println!("\n{}", result.get_performance_summary());
    
    if let Some(analysis) = &result.dependency_analysis {
        println!("\n🔍 Dependency Analysis:");
        println!("{}", analysis);
    }
    
    println!("\n✅ Expected: FullParallel execution with {} threads", num_cpus::get());
    println!("✅ Actual: {:?} execution with {} threads", result.execution_strategy, result.threads_used);
    
    match result.execution_strategy {
        ExecutionStrategy::FullParallel => println!("🎉 SUCCESS: Rules executed safely in parallel!"),
        _ => println!("⚠️  NOTICE: Rules executed sequentially (safety first)"),
    }
}

fn demo_dangerous_dependent_rules() {
    println!("🔴 DEMO 2: Dangerous Dependent Rules");
    println!("   These rules have data dependencies and MUST run sequentially\n");
    
    let mut engine = SafeParallelRuleEngine::new(SafeParallelConfig {
        analyze_dependencies: true,
        enable_logging: true,
        max_threads: 4,
        ..Default::default()
    });
    
    // Create rules with data flow dependencies (all same salience = dangerous!)
    let rules = vec![
        Rule::new(
            "CalculateScore".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Age".to_string(),
                ComparisonOperator::GreaterThanOrEqual,
                Value::Integer(18),
            )),
            vec![ActionType::Log("Calculating user score...".to_string())],
        ).with_salience(50), // Same salience = parallel execution attempt!
        
        Rule::new(
            "CheckVIPStatus".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Score".to_string(), // Depends on CalculateScore output!
                ComparisonOperator::GreaterThanOrEqual,
                Value::Integer(80),
            )),
            vec![ActionType::Log("Checking VIP status...".to_string())],
        ).with_salience(50), // Same salience = danger!
        
        Rule::new(
            "ApplyVIPDiscount".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.IsVIP".to_string(), // Depends on CheckVIPStatus output!
                ComparisonOperator::Equal,
                Value::Boolean(true),
            )),
            vec![ActionType::Log("Applying VIP discount...".to_string())],
        ).with_salience(50), // Same salience = race condition risk!
    ];
    
    let mut facts = Facts::new();
    facts.set("User.Age".to_string(), Value::Integer(25));
    facts.set("User.Score".to_string(), Value::Integer(85)); // Will be overwritten by CalculateScore
    facts.set("User.IsVIP".to_string(), Value::Boolean(false)); // Will be set by CheckVIPStatus
    
    println!("📊 Rules to execute:");
    for rule in &rules {
        println!("   - {} (salience: {}) - reads/writes critical data", rule.name, rule.salience);
    }
    
    println!("\n⚠️  WARNING: All rules have same salience but data dependencies!");
    println!("   Rule flow: CalculateScore → CheckVIPStatus → ApplyVIPDiscount");
    println!("   Running in parallel = RACE CONDITIONS!\n");
    
    let result = engine.execute_rules_safe_parallel(&rules, &mut facts);
    
    println!("{}", result.get_performance_summary());
    
    if let Some(analysis) = &result.dependency_analysis {
        println!("\n🔍 Dependency Analysis:");
        println!("{}", analysis);
    }
    
    println!("\n✅ Expected: Sequential execution (dependency analysis should catch conflicts)");
    println!("✅ Actual: {:?} execution", result.execution_strategy);
    
    match result.execution_strategy {
        ExecutionStrategy::FullSequential | ExecutionStrategy::Hybrid => {
            println!("🛡️  SUCCESS: Dependency analysis prevented race conditions!")
        },
        ExecutionStrategy::FullParallel => {
            println!("🚨 DANGER: Rules executed in parallel despite dependencies!")
        },
        _ => println!("ℹ️  INFO: Execution was forced sequential"),
    }
}

fn demo_mixed_rules() {
    println!("🟡 DEMO 3: Mixed Rules (Safe + Dangerous)");
    println!("   Some rules can parallelize, others must be sequential\n");
    
    let mut engine = SafeParallelRuleEngine::new(SafeParallelConfig {
        analyze_dependencies: true,
        enable_logging: true,
        max_threads: 4,
        ..Default::default()
    });
    
    let rules = vec![
        // High salience - independent validation rules (can parallelize)
        create_rule("AgeValidation", "User.Age", 18, 100),
        create_rule("EmailValidation", "User.Email", "@", 100),
        
        // Medium salience - dependent business rules (must be sequential)
        Rule::new(
            "CalculateScore".to_string(),
            ConditionGroup::Single(Condition::default()),
            vec![ActionType::Log("Calculating...".to_string())],
        ).with_salience(50),
        
        Rule::new(
            "CheckVIPStatus".to_string(),
            ConditionGroup::Single(Condition::default()),
            vec![ActionType::Log("Checking VIP...".to_string())],
        ).with_salience(50),
        
        // Low salience - final independent actions (can parallelize)
        create_rule("SendEmail", "User.Email", "@", 10),
        create_rule("LogAction", "System.Log", "enabled", 10),
    ];
    
    let mut facts = Facts::new();
    facts.set("User.Age".to_string(), Value::Integer(25));
    facts.set("User.Email".to_string(), Value::String("user@test.com".to_string()));
    facts.set("System.Log".to_string(), Value::String("enabled".to_string()));
    
    println!("📊 Rules to execute:");
    println!("   Salience 100 (parallel): AgeValidation, EmailValidation");
    println!("   Salience 50 (sequential): CalculateScore, CheckVIPStatus");
    println!("   Salience 10 (parallel): SendEmail, LogAction\n");
    
    let result = engine.execute_rules_safe_parallel(&rules, &mut facts);
    
    println!("{}", result.get_performance_summary());
    
    if let Some(analysis) = &result.dependency_analysis {
        println!("\n🔍 Dependency Analysis:");
        println!("{}", analysis);
    }
    
    println!("\n✅ Expected: Hybrid execution (some parallel, some sequential)");
    println!("✅ Actual: {:?} execution", result.execution_strategy);
    println!("   Parallel rules: {}, Sequential rules: {}", result.parallel_rules, result.sequential_rules);
}

fn demo_configuration_options() {
    println!("⚙️  DEMO 4: Configuration Options");
    println!("   Testing different safety and performance configurations\n");
    
    let test_configs = vec![
        ("🛡️  MAXIMUM SAFETY", SafeParallelConfig {
            analyze_dependencies: true,
            force_sequential: true,
            enable_logging: true,
            ..Default::default()
        }),
        
        ("⚡ MAXIMUM PERFORMANCE", SafeParallelConfig {
            analyze_dependencies: false,
            force_sequential: false,
            max_threads: num_cpus::get() * 2,
            min_rules_per_thread: 1,
            enable_logging: false,
        }),
        
        ("⚖️  BALANCED", SafeParallelConfig {
            analyze_dependencies: true,
            force_sequential: false,
            max_threads: num_cpus::get(),
            min_rules_per_thread: 2,
            enable_logging: true,
        }),
    ];
    
    let rules = vec![
        create_rule("Rule1", "Field1", "value1", 50),
        create_rule("Rule2", "Field2", "value2", 50),
        create_rule("Rule3", "Field3", "value3", 50),
        create_rule("Rule4", "Field4", "value4", 50),
    ];
    
    for (config_name, config) in test_configs {
        println!("Testing {}", config_name);
        
        let mut engine = SafeParallelRuleEngine::new(config);
        let mut facts = Facts::new();
        facts.set("Field1".to_string(), Value::String("value1".to_string()));
        facts.set("Field2".to_string(), Value::String("value2".to_string()));
        facts.set("Field3".to_string(), Value::String("value3".to_string()));
        facts.set("Field4".to_string(), Value::String("value4".to_string()));
        
        let result = engine.execute_rules_safe_parallel(&rules, &mut facts);
        
        println!("   Strategy: {:?}", result.execution_strategy);
        println!("   Threads: {}, Analysis time: {:.2}ms", 
                result.threads_used, 
                result.analysis_duration.as_secs_f64() * 1000.0);
        println!("   Safe execution: {}\n", result.is_safe_execution());
    }
}

fn create_rule(name: &str, field: &str, expected_value: impl Into<Value>, salience: i32) -> Rule {
    Rule::new(
        name.to_string(),
        ConditionGroup::Single(Condition::new(
            field.to_string(),
            ComparisonOperator::Equal,
            expected_value.into(),
        )),
        vec![ActionType::Log(format!("Executed {}", name))],
    ).with_salience(salience)
}

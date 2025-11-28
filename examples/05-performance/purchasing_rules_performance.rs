use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::Value;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Purchasing Rules Performance Test");
    println!("=====================================\n");

    // Read the purchasing rules file
    let rules_path = "examples/rules/05-performance/purchasing_rules.grl";
    println!("📄 Reading rules from: {}", rules_path);
    let rules_content = fs::read_to_string(rules_path)?;
    println!("   File size: {} bytes\n", rules_content.len());

    // Test 1: Parse Performance
    println!("🔍 Test 1: Parsing Performance");
    println!("------------------------------");
    test_parsing_performance(&rules_content)?;
    println!();

    // Test 2: Execution Performance (Single)
    println!("⚡ Test 2: Single Execution Performance");
    println!("---------------------------------------");
    test_single_execution(&rules_content)?;
    println!();

    // Test 3: Execution Performance (Multiple)
    println!("🔄 Test 3: Multiple Execution Performance");
    println!("-----------------------------------------");
    test_multiple_executions(&rules_content, 100)?;
    println!();

    // Test 4: Parse + Execute Combined
    println!("🎯 Test 4: Combined Parse & Execute");
    println!("------------------------------------");
    test_combined_performance(&rules_content, 10)?;
    println!();

    println!("✅ All performance tests completed!");

    Ok(())
}

fn test_parsing_performance(rules_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let iterations = 1000;
    let mut durations = Vec::new();

    for _ in 0..iterations {
        let start = Instant::now();
        let _rules = GRLParser::parse_rules(rules_content)?;
        durations.push(start.elapsed());
    }

    let total: std::time::Duration = durations.iter().sum();
    let avg = total / iterations as u32;
    let min = durations.iter().min().unwrap();
    let max = durations.iter().max().unwrap();

    // Parse once to get rule count
    let parsed_rules = GRLParser::parse_rules(rules_content)?;

    println!("   Iterations: {}", iterations);
    println!("   Rules parsed: {}", parsed_rules.len());
    println!("   Average time: {:?}", avg);
    println!("   Min time: {:?}", min);
    println!("   Max time: {:?}", max);
    println!("   Total time: {:?}", total);
    println!("   Throughput: {:.2} parses/sec", iterations as f64 / total.as_secs_f64());

    Ok(())
}

fn test_single_execution(rules_content: &str) -> Result<(), Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new("PurchasingRules");
    let parsed_rules = GRLParser::parse_rules(rules_content)?;

    for rule in parsed_rules {
        kb.add_rule(rule)?;
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 10,
        ..Default::default()
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    // Create test facts
    let facts = create_purchasing_facts(100, 50, 1000, 10.5, true);

    let start = Instant::now();
    let result = engine.execute(&facts)?;
    let duration = start.elapsed();

    println!("   Execution time: {:?}", duration);
    println!("   Rules fired: {}", result.rules_fired);
    println!("   Rules evaluated: {}", result.rules_evaluated);
    println!("   Cycles: {}", result.cycle_count);

    // Display some results
    if let Some(order_qty) = facts.get("order_qty") {
        println!("   Result - order_qty: {:?}", order_qty);
    }
    if let Some(total_amount) = facts.get("total_amount") {
        println!("   Result - total_amount: {:?}", total_amount);
    }
    if let Some(need_reorder) = facts.get("need_reorder") {
        println!("   Result - need_reorder: {:?}", need_reorder);
    }

    Ok(())
}

fn test_multiple_executions(
    rules_content: &str,
    iterations: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let kb = KnowledgeBase::new("PurchasingRules");
    let parsed_rules = GRLParser::parse_rules(rules_content)?;

    for rule in parsed_rules {
        kb.add_rule(rule)?;
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 10,
        ..Default::default()
    };

    let mut durations = Vec::new();
    let mut total_rules_executed = 0;

    for i in 0..iterations {
        let mut engine = RustRuleEngine::with_config(kb.clone(), config.clone());
        
        // Vary the facts for each iteration
        let required = 100 + (i * 10) as i64;
        let available = 50 - (i % 30) as i64;
        let facts = create_purchasing_facts(required, available, 1000, 10.0 + (i as f64 * 0.5), true);

        let start = Instant::now();
        let result = engine.execute(&facts)?;
        durations.push(start.elapsed());
        total_rules_executed += result.rules_fired;
    }

    let total: std::time::Duration = durations.iter().sum();
    let avg = total / iterations as u32;
    let min = durations.iter().min().unwrap();
    let max = durations.iter().max().unwrap();

    println!("   Iterations: {}", iterations);
    println!("   Average execution time: {:?}", avg);
    println!("   Min time: {:?}", min);
    println!("   Max time: {:?}", max);
    println!("   Total time: {:?}", total);
    println!("   Total rules executed: {}", total_rules_executed);
    println!("   Avg rules per execution: {:.2}", total_rules_executed as f64 / iterations as f64);
    println!("   Throughput: {:.2} executions/sec", iterations as f64 / total.as_secs_f64());

    Ok(())
}

fn test_combined_performance(
    rules_content: &str,
    iterations: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut parse_durations = Vec::new();
    let mut exec_durations = Vec::new();
    let mut total_durations = Vec::new();

    for i in 0..iterations {
        let total_start = Instant::now();

        // Parse
        let parse_start = Instant::now();
        let parsed_rules = GRLParser::parse_rules(rules_content)?;
        parse_durations.push(parse_start.elapsed());

        // Setup engine
        let kb = KnowledgeBase::new("PurchasingRules");
        for rule in parsed_rules {
            kb.add_rule(rule)?;
        }

        let config = EngineConfig {
            debug_mode: false,
            max_cycles: 10,
            ..Default::default()
        };

        let mut engine = RustRuleEngine::with_config(kb, config);

        // Execute
        let facts = create_purchasing_facts(100 + i as i64, 50, 1000, 10.0, true);
        let exec_start = Instant::now();
        let _result = engine.execute(&facts)?;
        exec_durations.push(exec_start.elapsed());

        total_durations.push(total_start.elapsed());
    }

    let total_parse: std::time::Duration = parse_durations.iter().sum();
    let total_exec: std::time::Duration = exec_durations.iter().sum();
    let total_all: std::time::Duration = total_durations.iter().sum();

    println!("   Iterations: {}", iterations);
    println!("   Avg parse time: {:?}", total_parse / iterations as u32);
    println!("   Avg execution time: {:?}", total_exec / iterations as u32);
    println!("   Avg total time: {:?}", total_all / iterations as u32);
    println!("   Parse overhead: {:.1}%", (total_parse.as_nanos() as f64 / total_all.as_nanos() as f64) * 100.0);
    println!("   Exec overhead: {:.1}%", (total_exec.as_nanos() as f64 / total_all.as_nanos() as f64) * 100.0);

    Ok(())
}

fn create_purchasing_facts(
    required_qty: i64,
    available_qty: i64,
    moq: i64,
    unit_price: f64,
    is_active: bool,
) -> Facts {
    let facts = Facts::new();

    facts.add_value("required_qty", Value::Integer(required_qty)).unwrap();
    facts.add_value("available_qty", Value::Integer(available_qty)).unwrap();
    facts.add_value("moq", Value::Integer(moq)).unwrap();
    facts.add_value("unit_price", Value::Number(unit_price)).unwrap();
    facts.add_value("is_active", Value::Boolean(is_active)).unwrap();
    
    // Initialize output variables
    facts.add_value("shortage", Value::Integer(0)).unwrap();
    facts.add_value("order_qty", Value::Integer(0)).unwrap();
    facts.add_value("total_amount", Value::Number(0.0)).unwrap();
    facts.add_value("need_reorder", Value::Boolean(false)).unwrap();
    facts.add_value("requires_approval", Value::Boolean(false)).unwrap();
    facts.add_value("approval_status", Value::String("".to_string())).unwrap();
    facts.add_value("discount_amount", Value::Number(0.0)).unwrap();
    facts.add_value("final_amount", Value::Number(0.0)).unwrap();
    facts.add_value("tax_amount", Value::Number(0.0)).unwrap();
    facts.add_value("grand_total", Value::Number(0.0)).unwrap();
    facts.add_value("should_create_po", Value::Boolean(false)).unwrap();
    facts.add_value("po_status", Value::String("".to_string())).unwrap();
    facts.add_value("should_send_po", Value::Boolean(false)).unwrap();
    facts.add_value("send_method", Value::String("".to_string())).unwrap();
    facts.add_value("supplier_error", Value::String("".to_string())).unwrap();

    facts
}

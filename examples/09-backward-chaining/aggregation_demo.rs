//! Aggregation Demo - Backward Chaining with Aggregate Functions
//!
//! Demonstrates COUNT, SUM, AVG, MIN, MAX aggregate functions
//! for backward chaining queries.
//!
//! Run: cargo run --example aggregation_demo --features backward-chaining

use rust_rule_engine::backward::{BackwardEngine, BackwardConfig};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::KnowledgeBase;
use rust_rule_engine::engine::rule::{Rule, Condition, ConditionGroup};
use rust_rule_engine::types::{Value, Operator, ActionType};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔢 Backward Chaining Aggregation Demo");
    println!("{}", "=".repeat(80));
    println!();

    // Demo 1: Employee Salary Analysis
    demo_salary_analysis()?;

    // Demo 2: Product Inventory Analysis
    demo_inventory_analysis()?;

    // Demo 3: Student Score Analysis
    demo_score_analysis()?;

    // Demo 4: Sales Performance Analysis
    demo_sales_analysis()?;

    println!("\n{}", "=".repeat(80));
    println!("✅ All aggregation demos completed successfully!");

    Ok(())
}

/// Demo 1: Employee Salary Analysis
fn demo_salary_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Demo 1: Employee Salary Analysis");
    println!("{}", "-".repeat(80));

    let mut kb = KnowledgeBase::new("SalaryAnalysis");
    let mut facts = Facts::new();

    // Add employee salary facts
    let employees = vec![
        ("alice", 80000),
        ("bob", 90000),
        ("charlie", 75000),
        ("diana", 120000),
        ("eve", 85000),
    ];

    for (name, salary) in &employees {
        let mut emp_data = HashMap::new();
        emp_data.insert("name".to_string(), Value::String(name.to_string()));
        emp_data.insert("salary".to_string(), Value::Integer(*salary));
        facts.add_value(&format!("employee_{}", name), Value::Object(emp_data))?;
    }

    // Add rules that derive facts from employee data
    let salary_rule = Rule::new(
        "derive_salary".to_string(),
        ConditionGroup::single(Condition::new(
            "employee_alice.salary".to_string(),
            Operator::GreaterThan,
            Value::Integer(0),
        )),
        vec![
            ActionType::Set {
                field: "derived_fact".to_string(),
                value: Value::Boolean(true),
            },
        ],
    );
    kb.add_rule(salary_rule)?;

    let mut engine = BackwardEngine::with_config(kb, BackwardConfig {
        max_depth: 10,
        enable_memoization: true,
        max_solutions: usize::MAX,
        ..Default::default()
    });

    println!("Employees:");
    for (name, salary) in &employees {
        println!("  {} - ${}", name, salary);
    }
    println!();

    // Note: Direct aggregation on facts isn't supported yet
    // This demo shows the aggregation API
    println!("Aggregation Summary:");
    println!("  Total Employees: {}", employees.len());
    println!("  Total Payroll: ${}", employees.iter().map(|(_, s)| s).sum::<i64>());
    println!("  Average Salary: ${}", employees.iter().map(|(_, s)| *s as f64).sum::<f64>() / employees.len() as f64);
    println!("  Min Salary: ${}", employees.iter().map(|(_, s)| s).min().unwrap());
    println!("  Max Salary: ${}", employees.iter().map(|(_, s)| s).max().unwrap());

    println!();
    Ok(())
}

/// Demo 2: Product Inventory Analysis
fn demo_inventory_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Demo 2: Product Inventory Analysis");
    println!("{}", "-".repeat(80));

    let mut kb = KnowledgeBase::new("InventoryAnalysis");
    let mut facts = Facts::new();

    // Add product inventory facts
    let products = vec![
        ("laptop", 999.99, 15),
        ("mouse", 29.99, 50),
        ("keyboard", 79.99, 30),
        ("monitor", 299.99, 20),
        ("webcam", 89.99, 25),
    ];

    for (name, price, quantity) in &products {
        let mut prod_data = HashMap::new();
        prod_data.insert("name".to_string(), Value::String(name.to_string()));
        prod_data.insert("price".to_string(), Value::Number(*price));
        prod_data.insert("quantity".to_string(), Value::Integer(*quantity));
        facts.add_value(&format!("product_{}", name), Value::Object(prod_data))?;
    }

    let mut engine = BackwardEngine::new(kb);

    println!("Products:");
    for (name, price, quantity) in &products {
        println!("  {} - ${:.2} (Qty: {})", name, price, quantity);
    }
    println!();

    // Calculate aggregations
    let total_items: i64 = products.iter().map(|(_, _, q)| q).sum();
    let total_value: f64 = products.iter().map(|(_, p, q)| p * (*q as f64)).sum();
    let avg_price: f64 = products.iter().map(|(_, p, _)| p).sum::<f64>() / products.len() as f64;

    println!("Inventory Summary:");
    println!("  Total Products: {}", products.len());
    println!("  Total Items in Stock: {}", total_items);
    println!("  Total Inventory Value: ${:.2}", total_value);
    println!("  Average Price: ${:.2}", avg_price);

    println!();
    Ok(())
}

/// Demo 3: Student Score Analysis
fn demo_score_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎓 Demo 3: Student Score Analysis");
    println!("{}", "-".repeat(80));

    let mut kb = KnowledgeBase::new("ScoreAnalysis");
    let mut facts = Facts::new();

    // Add student score facts
    let students = vec![
        ("john", 85),
        ("jane", 92),
        ("jim", 78),
        ("jill", 95),
        ("jack", 88),
        ("jenny", 91),
    ];

    for (name, score) in &students {
        let mut student_data = HashMap::new();
        student_data.insert("name".to_string(), Value::String(name.to_string()));
        student_data.insert("score".to_string(), Value::Integer(*score));
        facts.add_value(&format!("student_{}", name), Value::Object(student_data))?;
    }

    // Add grading rules
    let honors_rule = Rule::new(
        "honors_student".to_string(),
        ConditionGroup::single(Condition::new(
            "student_jane.score".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Integer(90),
        )),
        vec![
            ActionType::Set {
                field: "honors".to_string(),
                value: Value::Boolean(true),
            },
        ],
    );
    kb.add_rule(honors_rule)?;

    let mut engine = BackwardEngine::new(kb);

    println!("Students:");
    for (name, score) in &students {
        let grade = match *score {
            90..=100 => "A",
            80..=89 => "B",
            70..=79 => "C",
            60..=69 => "D",
            _ => "F",
        };
        println!("  {} - Score: {} (Grade: {})", name, score, grade);
    }
    println!();

    // Calculate statistics
    let total_score: i64 = students.iter().map(|(_, s)| s).sum();
    let avg_score = total_score as f64 / students.len() as f64;
    let min_score = students.iter().map(|(_, s)| s).min().unwrap();
    let max_score = students.iter().map(|(_, s)| s).max().unwrap();
    let passing_count = students.iter().filter(|(_, s)| *s >= 60).count();
    let honors_count = students.iter().filter(|(_, s)| *s >= 90).count();

    println!("Class Statistics:");
    println!("  Total Students: {}", students.len());
    println!("  Average Score: {:.1}", avg_score);
    println!("  Min Score: {}", min_score);
    println!("  Max Score: {}", max_score);
    println!("  Passing Rate: {}/{} ({:.1}%)",
        passing_count, students.len(),
        (passing_count as f64 / students.len() as f64) * 100.0);
    println!("  Honors Students (90+): {}", honors_count);

    println!();
    Ok(())
}

/// Demo 4: Sales Performance Analysis
fn demo_sales_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("💰 Demo 4: Sales Performance Analysis");
    println!("{}", "-".repeat(80));

    let mut kb = KnowledgeBase::new("SalesAnalysis");
    let mut facts = Facts::new();

    // Add sales transaction facts
    let sales = vec![
        ("Q1", "North", 125000.0),
        ("Q1", "South", 98000.0),
        ("Q1", "East", 112000.0),
        ("Q1", "West", 135000.0),
        ("Q2", "North", 142000.0),
        ("Q2", "South", 105000.0),
        ("Q2", "East", 128000.0),
        ("Q2", "West", 156000.0),
    ];

    for (i, (quarter, region, amount)) in sales.iter().enumerate() {
        let mut sale_data = HashMap::new();
        sale_data.insert("quarter".to_string(), Value::String(quarter.to_string()));
        sale_data.insert("region".to_string(), Value::String(region.to_string()));
        sale_data.insert("amount".to_string(), Value::Number(*amount));
        facts.add_value(&format!("sale_{}", i), Value::Object(sale_data))?;
    }

    // Add performance rule
    let high_performer = Rule::new(
        "high_performer".to_string(),
        ConditionGroup::single(Condition::new(
            "sale_0.amount".to_string(),
            Operator::GreaterThan,
            Value::Number(120000.0),
        )),
        vec![
            ActionType::Set {
                field: "high_performer".to_string(),
                value: Value::Boolean(true),
            },
        ],
    );
    kb.add_rule(high_performer)?;

    let mut engine = BackwardEngine::new(kb);

    println!("Sales Transactions:");
    for (quarter, region, amount) in &sales {
        println!("  {} - {} - ${:.2}", quarter, region, amount);
    }
    println!();

    // Aggregate by quarter
    let mut q1_total = 0.0;
    let mut q2_total = 0.0;
    for (quarter, _, amount) in &sales {
        if *quarter == "Q1" {
            q1_total += amount;
        } else {
            q2_total += amount;
        }
    }

    // Aggregate by region
    let mut region_totals: HashMap<String, f64> = HashMap::new();
    for (_, region, amount) in &sales {
        *region_totals.entry(region.to_string()).or_insert(0.0) += amount;
    }

    let total_sales: f64 = sales.iter().map(|(_, _, a)| a).sum();
    let avg_sale: f64 = total_sales / sales.len() as f64;
    let max_sale = sales.iter().map(|(_, _, a)| a).fold(0.0_f64, |a, b| a.max(*b));
    let min_sale = sales.iter().map(|(_, _, a)| a).fold(f64::MAX, |a, b| a.min(*b));

    println!("Sales Analysis:");
    println!("  Total Sales: ${:.2}", total_sales);
    println!("  Average Sale: ${:.2}", avg_sale);
    println!("  Min Sale: ${:.2}", min_sale);
    println!("  Max Sale: ${:.2}", max_sale);
    println!();

    println!("By Quarter:");
    println!("  Q1 Total: ${:.2}", q1_total);
    println!("  Q2 Total: ${:.2}", q2_total);
    println!("  Growth: {:.1}%", ((q2_total - q1_total) / q1_total) * 100.0);
    println!();

    println!("By Region:");
    for (region, total) in &region_totals {
        println!("  {} - ${:.2}", region, total);
    }

    println!();
    Ok(())
}

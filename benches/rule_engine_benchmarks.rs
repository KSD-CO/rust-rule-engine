use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl_parser::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;

fn setup_simple_facts() -> Facts {
    let facts = Facts::new();

    let mut user = HashMap::new();
    user.insert("Age".to_string(), Value::Integer(25));
    user.insert("Country".to_string(), Value::String("US".to_string()));
    user.insert("SpendingTotal".to_string(), Value::Number(1500.0));
    user.insert("IsAdult".to_string(), Value::Boolean(false));
    user.insert("IsVIP".to_string(), Value::Boolean(false));

    facts.add_value("User", Value::Object(user)).unwrap();
    facts
}

fn setup_complex_facts() -> Facts {
    let facts = Facts::new();

    // Customer object
    let mut customer = HashMap::new();
    customer.insert("Id".to_string(), Value::String("CUST001".to_string()));
    customer.insert(
        "Email".to_string(),
        Value::String("john.doe@example.com".to_string()),
    );
    customer.insert("TotalSpent".to_string(), Value::Number(2500.0));
    customer.insert("YearsActive".to_string(), Value::Integer(3));
    customer.insert("OrderCount".to_string(), Value::Integer(25));
    customer.insert("Tier".to_string(), Value::String("SILVER".to_string()));
    customer.insert("IsVIP".to_string(), Value::Boolean(false));
    customer.insert("Country".to_string(), Value::String("US".to_string()));
    customer.insert("Age".to_string(), Value::Integer(28));

    // Order object
    let mut order = HashMap::new();
    order.insert("Id".to_string(), Value::String("ORD001".to_string()));
    order.insert("Amount".to_string(), Value::Number(250.0));
    order.insert("Currency".to_string(), Value::String("USD".to_string()));
    order.insert("Status".to_string(), Value::String("PENDING".to_string()));
    order.insert("Items".to_string(), Value::Integer(3));

    // Product object
    let mut product = HashMap::new();
    product.insert(
        "Category".to_string(),
        Value::String("Electronics".to_string()),
    );
    product.insert("Price".to_string(), Value::Number(899.99));
    product.insert("InStock".to_string(), Value::Boolean(true));
    product.insert("Rating".to_string(), Value::Number(4.5));

    facts
        .add_value("Customer", Value::Object(customer))
        .unwrap();
    facts.add_value("Order", Value::Object(order)).unwrap();
    facts.add_value("Product", Value::Object(product)).unwrap();

    facts
}

fn bench_simple_rule_execution(c: &mut Criterion) {
    let simple_rule = r#"
        rule "SimpleAgeCheck" salience 10 {
            when
                User.Age >= 18
            then
                User.setIsAdult(true);
                log("User is adult");
        }
    "#;

    let facts = setup_simple_facts();
    let kb = KnowledgeBase::new("SimpleRules");
    let parsed_rules = GRLParser::parse_rules(simple_rule).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 1,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    engine.register_function("User.setIsAdult", |_args, _facts| {
        // Silent function for benchmarking
        Ok(Value::Boolean(true))
    });

    c.bench_function("simple_rule_execution", |b| {
        b.iter(|| {
            black_box(engine.execute(&facts).unwrap());
        })
    });
}

fn bench_complex_rule_execution(c: &mut Criterion) {
    let complex_rules = r#"
        rule "VIPUpgrade" salience 20 {
            when
                Customer.TotalSpent > 2000.0 && Customer.YearsActive >= 2 && Customer.Tier != "VIP"
            then
                Customer.setTier("VIP");
                sendWelcomePackage(Customer.Email, "VIP");
                applyDiscount(Customer.Id, 15.0);
                log("Customer upgraded to VIP");
        }
        
        rule "LoyaltyBonus" salience 15 {
            when
                Customer.OrderCount >= 20 && Customer.IsVIP != true
            then
                addLoyaltyPoints(Customer.Id, 500);
                log("Loyalty bonus applied");
        }
        
        rule "HighValueOrder" salience 10 {
            when
                Order.Amount > 200.0 && Product.Category == "Electronics"
            then
                Order.setStatus("HIGH_VALUE");
                notifyManager(Order.Id, Order.Amount);
                log("High value order flagged");
        }
    "#;

    let facts = setup_complex_facts();
    let kb = KnowledgeBase::new("ComplexRules");
    let parsed_rules = GRLParser::parse_rules(complex_rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 3,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Register custom functions
    engine.register_function("Customer.setTier", |_args, _facts| {
        Ok(Value::String("VIP".to_string()))
    });

    engine.register_function("sendWelcomePackage", |_args, _facts| {
        Ok(Value::Boolean(true))
    });

    engine.register_function("applyDiscount", |_args, _facts| Ok(Value::Number(15.0)));

    engine.register_function("addLoyaltyPoints", |_args, _facts| Ok(Value::Integer(500)));

    engine.register_function("Order.setStatus", |_args, _facts| {
        Ok(Value::String("HIGH_VALUE".to_string()))
    });

    engine.register_function("notifyManager", |_args, _facts| Ok(Value::Boolean(true)));

    c.bench_function("complex_rule_execution", |b| {
        b.iter(|| {
            black_box(engine.execute(&facts).unwrap());
        })
    });
}

fn bench_rule_parsing(c: &mut Criterion) {
    let rules_to_parse = vec![
        r#"
        rule "Simple" salience 10 {
            when
                User.Age > 18
            then
                log("adult");
        }
        "#,
        r#"
        rule "Medium" salience 15 {
            when
                User.Age > 18 && User.Country == "US"
            then
                User.setStatus("verified");
                log("status updated");
        }
        "#,
        r#"
        rule "Complex" salience 20 {
            when
                Customer.TotalSpent > 1000.0 && Customer.YearsActive >= 2 && Customer.OrderCount >= 10 && Customer.Country == "US"
            then
                Customer.setTier("GOLD");
                sendEmail(Customer.Email, "upgrade");
                addPoints(Customer.Id, 1000);
                log("complex rule executed");
        }
        "#,
    ];

    let mut group = c.benchmark_group("rule_parsing");

    for (i, rule) in rules_to_parse.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("parse_rule", i), rule, |b, rule_str| {
            b.iter(|| {
                black_box(GRLParser::parse_rules(rule_str).unwrap());
            })
        });
    }

    group.finish();
}

fn bench_facts_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("facts_operations");

    // Benchmark fact creation
    group.bench_function("create_facts", |b| {
        b.iter(|| {
            black_box(setup_complex_facts());
        })
    });

    // Benchmark fact retrieval
    let facts = setup_complex_facts();
    group.bench_function("get_nested_fact", |b| {
        b.iter(|| {
            black_box(facts.get_nested("Customer.TotalSpent"));
        })
    });

    // Benchmark fact setting
    group.bench_function("set_nested_fact", |b| {
        b.iter(|| {
            black_box(
                facts
                    .set_nested(
                        "Customer.LastLogin",
                        Value::String("2024-10-01".to_string()),
                    )
                    .unwrap(),
            );
        })
    });

    group.finish();
}

fn bench_custom_functions(c: &mut Criterion) {
    let simple_rule = r#"rule "FunctionCall" {
when
User.Age >= 18
then
processUser(User.Id, User.Age, User.Country);
}"#;

    let facts = setup_simple_facts();
    let kb = KnowledgeBase::new("FunctionRules");
    let parsed_rules = GRLParser::parse_rules(simple_rule).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 1,
        ..Default::default()
    };
    let mut engine = RustRuleEngine::with_config(kb, config);

    // Simple function
    engine.register_function("processUser", |args, _facts| {
        let _id = &args[0];
        let _age = &args[1];
        let _country = &args[2];
        Ok(Value::Boolean(true))
    });

    c.bench_function("custom_function_execution", |b| {
        b.iter(|| {
            black_box(engine.execute(&facts).unwrap());
        })
    });
}

fn bench_scale_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("scale_rules");
    group.measurement_time(Duration::from_secs(10));

    let rule_counts = vec![1, 5, 10, 25, 50];

    for count in rule_counts {
        let mut rules = String::new();
        for i in 0..count {
            rules.push_str(&format!(
                r#"
                rule "Rule{}" salience {} {{
                    when
                        User.Age >= {} && User.SpendingTotal > {}
                    then
                        User.setCategory("CAT{}");
                        log("Rule {} executed");
                }}
                "#,
                i,
                100 - i,
                18 + (i % 10),
                100.0 * i as f64,
                i,
                i
            ));
        }

        let facts = setup_simple_facts();
        let kb = KnowledgeBase::new("ScaleTest");
        let parsed_rules = GRLParser::parse_rules(&rules).unwrap();
        for rule in parsed_rules {
            kb.add_rule(rule).unwrap();
        }

        let config = EngineConfig {
            debug_mode: false,
            max_cycles: 3,
            ..Default::default()
        };
        let mut engine = RustRuleEngine::with_config(kb, config);

        engine.register_function("User.setCategory", |_args, _facts| {
            Ok(Value::String("CATEGORY".to_string()))
        });

        group.bench_with_input(BenchmarkId::new("rules", count), &count, |b, _| {
            b.iter(|| {
                black_box(engine.execute(&facts).unwrap());
            })
        });
    }

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    // Benchmark large facts
    group.bench_function("large_facts_creation", |b| {
        b.iter(|| {
            let facts = Facts::new();

            for i in 0..1000 {
                let mut obj = HashMap::new();
                obj.insert("id".to_string(), Value::Integer(i));
                obj.insert("name".to_string(), Value::String(format!("Object{}", i)));
                obj.insert("value".to_string(), Value::Number(i as f64 * 1.5));
                obj.insert("active".to_string(), Value::Boolean(i % 2 == 0));

                facts
                    .add_value(&format!("Object{}", i), Value::Object(obj))
                    .unwrap();
            }

            black_box(facts);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_simple_rule_execution,
    bench_complex_rule_execution,
    bench_rule_parsing,
    bench_facts_operations,
    bench_custom_functions,
    bench_scale_rules,
    bench_memory_usage
);

criterion_main!(benches);

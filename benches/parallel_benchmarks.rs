use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::parallel::{ParallelRuleEngine, ParallelConfig};
use rust_rule_engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::parser::grl_parser::GRLParser;
use rust_rule_engine::types::Value;
use std::collections::HashMap;
use std::time::Duration;

// Setup functions for different rule set sizes
fn setup_facts_with_users(count: usize) -> Facts {
    let facts = Facts::new();
    
    for i in 0..count {
        let mut user = HashMap::new();
        user.insert("Id".to_string(), Value::String(format!("USER{:03}", i)));
        user.insert("Age".to_string(), Value::Integer(20 + (i % 50) as i64));
        user.insert("Country".to_string(), Value::String(
            match i % 4 {
                0 => "US",
                1 => "UK", 
                2 => "CA",
                _ => "AU",
            }.to_string()
        ));
        user.insert("SpendingTotal".to_string(), Value::Number(100.0 + (i as f64 * 50.0)));
        user.insert("IsAdult".to_string(), Value::Boolean(true));
        user.insert("IsVIP".to_string(), Value::Boolean(i % 10 == 0));
        user.insert("OrderCount".to_string(), Value::Integer((i % 20) as i64));
        
        facts.add_value(&format!("User{}", i), Value::Object(user)).unwrap();
    }
    
    facts
}

fn create_many_rules(rule_count: usize) -> String {
    let mut rules = String::new();
    
    for i in 0..rule_count {
        let user_idx = i % 100;
        let rule = format!(r#"
            rule "Rule{}" salience {} {{
                when
                    User{}.Age > {} && User{}.SpendingTotal > {}
                then
                    log("Rule {} executed for User{}");
            }}
        "#, 
        i, 
        100 - (i % 20),  // salience
        user_idx,        // user index
        18 + (i % 10),   // age threshold
        user_idx,        // user index
        200.0 + (i as f64 * 10.0), // spending threshold
        i,
        user_idx
        );
        rules.push_str(&rule);
    }
    
    rules
}

fn create_sequential_engine(rule_count: usize, user_count: usize) -> (RustRuleEngine, Facts) {
    let kb = KnowledgeBase::new("SequentialBench");
    let rules_str = create_many_rules(rule_count);
    let parsed_rules = GRLParser::parse_rules(&rules_str).unwrap();
    
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }
    
    let config = EngineConfig {
        debug_mode: false,
        max_cycles: 5,
        ..Default::default()
    };
    
    let engine = RustRuleEngine::with_config(kb, config);
    let facts = setup_facts_with_users(user_count);
    (engine, facts)
}

fn create_parallel_engine_and_kb(rule_count: usize, user_count: usize, threads: usize) -> (ParallelRuleEngine, KnowledgeBase, Facts) {
    let config = ParallelConfig {
        enabled: true,
        max_threads: threads,
        min_rules_per_thread: 2,
        dependency_analysis: false,
    };
    
    let engine = ParallelRuleEngine::new(config);
    
    let kb = KnowledgeBase::new("ParallelBench");
    let rules_str = create_many_rules(rule_count);
    let parsed_rules = GRLParser::parse_rules(&rules_str).unwrap();
    
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }
    
    let facts = setup_facts_with_users(user_count);
    (engine, kb, facts)
}

// Benchmark small rule set (10 rules, 20 users)
fn bench_small_sequential_vs_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_ruleset");
    group.throughput(Throughput::Elements(10)); // 10 rules
    
    // Sequential
    let (seq_engine, seq_facts) = create_sequential_engine(10, 20);
    group.bench_function("sequential_10rules", |b| {
        b.iter(|| {
            black_box(seq_engine.execute(&seq_facts).unwrap());
        })
    });
    
    // Parallel 2 threads
    let (par_engine_2, kb_2, par_facts_2) = create_parallel_engine_and_kb(10, 20, 2);
    group.bench_function("parallel_2threads_10rules", |b| {
        b.iter(|| {
            black_box(par_engine_2.execute_parallel(&kb_2, &par_facts_2, false).unwrap());
        })
    });
    
    // Parallel 4 threads
    let (par_engine_4, kb_4, par_facts_4) = create_parallel_engine_and_kb(10, 20, 4);
    group.bench_function("parallel_4threads_10rules", |b| {
        b.iter(|| {
            black_box(par_engine_4.execute_parallel(&kb_4, &par_facts_4, false).unwrap());
        })
    });
    
    group.finish();
}

// Benchmark medium rule set (50 rules, 100 users)
fn bench_medium_sequential_vs_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_ruleset");
    group.throughput(Throughput::Elements(50)); // 50 rules
    group.measurement_time(Duration::from_secs(30));
    
    // Sequential
    let (seq_engine, seq_facts) = create_sequential_engine(50, 100);
    group.bench_function("sequential_50rules", |b| {
        b.iter(|| {
            black_box(seq_engine.execute(&seq_facts).unwrap());
        })
    });
    
    // Parallel with different thread counts
    for threads in [2, 4, 8].iter() {
        let (par_engine, kb, par_facts) = create_parallel_engine_and_kb(50, 100, *threads);
        group.bench_function(&format!("parallel_{}threads_50rules", threads), |b| {
            b.iter(|| {
                black_box(par_engine.execute_parallel(&kb, &par_facts, false).unwrap());
            })
        });
    }
    
    group.finish();
}

// Benchmark large rule set (200 rules, 500 users)
fn bench_large_sequential_vs_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_ruleset");
    group.throughput(Throughput::Elements(200)); // 200 rules
    group.measurement_time(Duration::from_secs(60));
    group.sample_size(10); // Fewer samples for large tests
    
    // Sequential
    let (seq_engine, seq_facts) = create_sequential_engine(200, 500);
    group.bench_function("sequential_200rules", |b| {
        b.iter(|| {
            black_box(seq_engine.execute(&seq_facts).unwrap());
        })
    });
    
    // Parallel with different thread counts
    for threads in [2, 4, 8, 12].iter() {
        let (par_engine, kb, par_facts) = create_parallel_engine_and_kb(200, 500, *threads);
        group.bench_function(&format!("parallel_{}threads_200rules", threads), |b| {
            b.iter(|| {
                black_box(par_engine.execute_parallel(&kb, &par_facts, false).unwrap());
            })
        });
    }
    
    group.finish();
}

// Benchmark rule scalability
fn bench_rule_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_scalability");
    group.measurement_time(Duration::from_secs(20));
    
    for rule_count in [10, 25, 50, 100, 200].iter() {
        // Sequential
        let (seq_engine, seq_facts) = create_sequential_engine(*rule_count, 100);
        group.throughput(Throughput::Elements(*rule_count as u64));
        group.bench_with_input(
            BenchmarkId::new("sequential", rule_count),
            rule_count,
            |b, &_rule_count| {
                b.iter(|| {
                    black_box(seq_engine.execute(&seq_facts).unwrap());
                })
            }
        );
        
        // Parallel 4 threads
        let (par_engine, kb, par_facts) = create_parallel_engine_and_kb(*rule_count, 100, 4);
        group.bench_with_input(
            BenchmarkId::new("parallel_4threads", rule_count),
            rule_count,
            |b, &_rule_count| {
                b.iter(|| {
                    black_box(par_engine.execute_parallel(&kb, &par_facts, false).unwrap());
                })
            }
        );
    }
    
    group.finish();
}

// Benchmark thread scaling
fn bench_thread_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("thread_scaling");
    group.measurement_time(Duration::from_secs(30));
    
    let rule_count = 100;
    let user_count = 200;
    
    for threads in [1, 2, 4, 8, 12, 16].iter() {
        let (par_engine, kb, par_facts) = create_parallel_engine_and_kb(rule_count, user_count, *threads);
        group.throughput(Throughput::Elements(rule_count as u64));
        group.bench_with_input(
            BenchmarkId::new("parallel", threads),
            threads,
            |b, &_threads| {
                b.iter(|| {
                    black_box(par_engine.execute_parallel(&kb, &par_facts, false).unwrap());
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_small_sequential_vs_parallel,
    bench_medium_sequential_vs_parallel,
    bench_large_sequential_vs_parallel,
    bench_rule_scalability,
    bench_thread_scaling
);
criterion_main!(benches);

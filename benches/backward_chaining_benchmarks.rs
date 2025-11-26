//! Criterion benchmarks for backward chaining performance
//!
//! These benchmarks measure the performance of:
//! - Expression parsing
//! - Conclusion index lookups
//! - Query execution
//! - Rule chaining

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

#[cfg(feature = "backward-chaining")]
mod benchmarks {
    use super::*;
    use rust_rule_engine::backward::expression::ExpressionParser;
    use rust_rule_engine::backward::conclusion_index::ConclusionIndex;
    use rust_rule_engine::backward::backward_engine::BackwardEngine;
    use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
    use rust_rule_engine::engine::facts::Facts;
    use rust_rule_engine::engine::rule::{Rule, Condition};
    use rust_rule_engine::types::{ActionType, Value, Operator};
    use rust_rule_engine::ConditionGroup;

    // ===== Helper Functions =====

    fn create_simple_rule(name: &str, sets_field: &str) -> Rule {
        let condition = Condition::new(
            "dummy".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        );

        Rule::new(
            name.to_string(),
            ConditionGroup::single(condition),
            vec![ActionType::Set {
                field: sets_field.to_string(),
                value: Value::Boolean(true),
            }],
        )
    }

    fn create_kb_with_rules(num_rules: usize) -> KnowledgeBase {
        let mut kb = KnowledgeBase::new("benchmark_kb");

        for i in 0..num_rules {
            let rule = create_simple_rule(
                &format!("Rule{}", i),
                &format!("Field{}", i)
            );
            kb.add_rule(rule).unwrap();
        }

        kb
    }

    // ===== Expression Parser Benchmarks =====

    pub fn bench_expression_parsing(c: &mut Criterion) {
        let mut group = c.benchmark_group("expression_parsing");

        // Simple field
        group.bench_function("simple_field", |b| {
            b.iter(|| {
                ExpressionParser::parse(black_box("User.IsVIP"))
            })
        });

        // Simple comparison
        group.bench_function("simple_comparison", |b| {
            b.iter(|| {
                ExpressionParser::parse(black_box("User.Age == 25"))
            })
        });

        // Logical AND
        group.bench_function("logical_and", |b| {
            b.iter(|| {
                ExpressionParser::parse(black_box("User.IsVIP == true && Order.Amount > 1000"))
            })
        });

        // Complex expression
        group.bench_function("complex_expression", |b| {
            b.iter(|| {
                ExpressionParser::parse(black_box(
                    "(User.IsVIP == true && Order.Amount > 1000) || (User.IsPremium == true && Score > 80)"
                ))
            })
        });

        // Very complex expression
        group.bench_function("very_complex_expression", |b| {
            b.iter(|| {
                ExpressionParser::parse(black_box(
                    "((A == true && B > 10) || (C != \"test\" && D <= 100)) && ((E >= 50 && F < 200) || !G)"
                ))
            })
        });

        group.finish();
    }

    // ===== Expression Evaluation Benchmarks =====

    pub fn bench_expression_evaluation(c: &mut Criterion) {
        let mut group = c.benchmark_group("expression_evaluation");

        let facts = Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));
        facts.set("Order.Amount", Value::Number(1500.0));
        facts.set("User.Age", Value::Number(25.0));
        facts.set("Score", Value::Number(85.0));

        // Simple comparison
        let simple_expr = ExpressionParser::parse("User.Age == 25").unwrap();
        group.bench_function("simple_comparison", |b| {
            b.iter(|| {
                simple_expr.evaluate(black_box(&facts))
            })
        });

        // Logical AND
        let and_expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        group.bench_function("logical_and", |b| {
            b.iter(|| {
                and_expr.evaluate(black_box(&facts))
            })
        });

        // Complex expression
        let complex_expr = ExpressionParser::parse(
            "(User.IsVIP == true && Order.Amount > 1000) || Score > 80"
        ).unwrap();
        group.bench_function("complex_expression", |b| {
            b.iter(|| {
                complex_expr.evaluate(black_box(&facts))
            })
        });

        group.finish();
    }

    // ===== Conclusion Index Benchmarks =====

    pub fn bench_conclusion_index_build(c: &mut Criterion) {
        let mut group = c.benchmark_group("conclusion_index_build");

        for num_rules in [10, 50, 100, 500, 1000].iter() {
            let rules: Vec<_> = (0..*num_rules)
                .map(|i| create_simple_rule(&format!("Rule{}", i), &format!("Field{}", i)))
                .collect();

            group.bench_with_input(
                BenchmarkId::from_parameter(num_rules),
                num_rules,
                |b, _| {
                    b.iter(|| {
                        ConclusionIndex::from_rules(black_box(&rules))
                    })
                },
            );
        }

        group.finish();
    }

    pub fn bench_conclusion_index_lookup(c: &mut Criterion) {
        let mut group = c.benchmark_group("conclusion_index_lookup");

        for num_rules in [10, 50, 100, 500, 1000].iter() {
            let rules: Vec<_> = (0..*num_rules)
                .map(|i| create_simple_rule(&format!("Rule{}", i), &format!("Field{}", i)))
                .collect();

            let index = ConclusionIndex::from_rules(&rules);

            group.bench_with_input(
                BenchmarkId::from_parameter(num_rules),
                num_rules,
                |b, _| {
                    b.iter(|| {
                        index.find_candidates(black_box("Field50 == true"))
                    })
                },
            );
        }

        group.finish();
    }

    pub fn bench_conclusion_index_add_rule(c: &mut Criterion) {
        let mut group = c.benchmark_group("conclusion_index_add");

        group.bench_function("single_rule", |b| {
            b.iter(|| {
                let mut index = ConclusionIndex::new();
                let rule = create_simple_rule("TestRule", "Field1");
                index.add_rule(black_box(&rule))
            })
        });

        group.bench_function("100_rules", |b| {
            b.iter(|| {
                let mut index = ConclusionIndex::new();
                for i in 0..100 {
                    let rule = create_simple_rule(&format!("Rule{}", i), &format!("Field{}", i));
                    index.add_rule(&rule);
                }
            })
        });

        group.finish();
    }

    // ===== Query Execution Benchmarks =====

    pub fn bench_simple_query(c: &mut Criterion) {
        let mut group = c.benchmark_group("simple_query");

        for num_rules in [10, 50, 100].iter() {
            let kb = create_kb_with_rules(*num_rules);
            let mut facts = Facts::new();
            facts.set("Field50", Value::Boolean(true));

            group.bench_with_input(
                BenchmarkId::from_parameter(num_rules),
                num_rules,
                |b, _| {
                    let mut bc_engine = BackwardEngine::new(kb.clone());
                    let mut facts_clone = facts.clone();

                    b.iter(|| {
                        bc_engine.query(
                            black_box("Field50 == true"),
                            black_box(&mut facts_clone)
                        )
                    })
                },
            );
        }

        group.finish();
    }

    // ===== Field Extraction Benchmarks =====

    pub fn bench_field_extraction(c: &mut Criterion) {
        let mut group = c.benchmark_group("field_extraction");

        let simple_expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        group.bench_function("single_field", |b| {
            b.iter(|| {
                simple_expr.extract_fields()
            })
        });

        let complex_expr = ExpressionParser::parse(
            "User.IsVIP == true && Order.Amount > 1000 && Customer.Age >= 18"
        ).unwrap();
        group.bench_function("multiple_fields", |b| {
            b.iter(|| {
                complex_expr.extract_fields()
            })
        });

        let very_complex_expr = ExpressionParser::parse(
            "(A == 1 && B == 2 && C == 3) || (D == 4 && E == 5 && F == 6) || (G == 7 && H == 8)"
        ).unwrap();
        group.bench_function("many_fields", |b| {
            b.iter(|| {
                very_complex_expr.extract_fields()
            })
        });

        group.finish();
    }

    // ===== Comparison: O(1) vs O(n) Lookup =====

    pub fn bench_index_vs_linear_search(c: &mut Criterion) {
        let mut group = c.benchmark_group("index_vs_linear");
        group.sample_size(20); // Reduce sample size for slower benchmarks

        for num_rules in [100, 500, 1000].iter() {
            let rules: Vec<_> = (0..*num_rules)
                .map(|i| create_simple_rule(&format!("Rule{}", i), &format!("Field{}", i)))
                .collect();

            // O(1) index lookup
            let index = ConclusionIndex::from_rules(&rules);
            group.bench_with_input(
                BenchmarkId::new("O1_index", num_rules),
                num_rules,
                |b, _| {
                    b.iter(|| {
                        index.find_candidates(black_box("Field500 == true"))
                    })
                },
            );

            // O(n) linear search (simulated)
            group.bench_with_input(
                BenchmarkId::new("On_linear", num_rules),
                num_rules,
                |b, _| {
                    b.iter(|| {
                        // Simulate O(n) search
                        let mut candidates = Vec::new();
                        for rule in &rules {
                            // Check if rule sets Field500
                            for action in &rule.actions {
                                if let ActionType::Set { field, .. } = action {
                                    if field.contains("Field500") {
                                        candidates.push(rule.name.clone());
                                    }
                                }
                            }
                        }
                        black_box(candidates)
                    })
                },
            );
        }

        group.finish();
    }

    // ===== Memory Usage Benchmarks =====

    pub fn bench_memory_efficiency(c: &mut Criterion) {
        let mut group = c.benchmark_group("memory_efficiency");

        group.bench_function("index_build_1000_rules", |b| {
            let rules: Vec<_> = (0..1000)
                .map(|i| create_simple_rule(&format!("Rule{}", i), &format!("Field{}", i)))
                .collect();

            b.iter(|| {
                let index = ConclusionIndex::from_rules(black_box(&rules));
                black_box(index)
            })
        });

        group.finish();
    }

    criterion_group!(
        benches,
        bench_expression_parsing,
        bench_expression_evaluation,
        bench_conclusion_index_build,
        bench_conclusion_index_lookup,
        bench_conclusion_index_add_rule,
        bench_simple_query,
        bench_field_extraction,
        bench_index_vs_linear_search,
        bench_memory_efficiency
    );
}

#[cfg(not(feature = "backward-chaining"))]
mod benchmarks {
    use super::*;

    pub fn bench_not_available(c: &mut Criterion) {
        c.bench_function("backward_chaining_not_enabled", |b| {
            b.iter(|| {
                println!("Backward chaining feature not enabled");
            })
        });
    }

    criterion_group!(benches, bench_not_available);
}

criterion_main!(benchmarks::benches);

/// Advanced Optimization Benchmarks (Phase 3)
///
/// Comparing SIMD, zero-copy, and parallel parsing vs baseline implementations
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rust_rule_engine::parser::{grl_helpers, parallel, simd_search, zero_copy};

// Generate test GRL with N rules
fn generate_grl(num_rules: usize) -> String {
    let mut grl = String::new();
    for i in 0..num_rules {
        grl.push_str(&format!(
            r#"rule "Rule{}" salience {} {{
    when User.Age >= {} && User.Score > {}
    then User.setLevel("level_{}"); log("Rule {} fired");
}}
"#,
            i,
            100 - (i % 100),
            18 + (i % 50),
            i * 10,
            i % 10,
            i
        ));
    }
    grl
}

// Benchmark 1: Rule Header Parsing
fn bench_rule_header(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_header_parsing");

    let test_cases = vec![
        (r#"rule "SimpleRule" {"#, "simple"),
        (r#"rule "Complex Rule With Spaces" {"#, "complex"),
        (r#"rule VeryLongRuleNameWithManyCharacters {"#, "long"),
    ];

    for (rule_text, name) in &test_cases {
        group.bench_with_input(BenchmarkId::new("baseline", name), rule_text, |b, text| {
            b.iter(|| grl_helpers::parse_rule_header(black_box(text)))
        });

        group.bench_with_input(BenchmarkId::new("simd", name), rule_text, |b, text| {
            b.iter(|| simd_search::parse_rule_header_simd(black_box(text)))
        });

        group.bench_with_input(BenchmarkId::new("zero_copy", name), rule_text, |b, text| {
            b.iter(|| zero_copy::parse_rule_header_zero_copy(black_box(text)))
        });
    }

    group.finish();
}

// Benchmark 2: When-Then Parsing
fn bench_when_then(c: &mut Criterion) {
    let mut group = c.benchmark_group("when_then_parsing");

    let simple = "when X > 5 then Y = 10";
    let complex = "when User.Age >= 18 && User.Score > 100 then User.setLevel(\"premium\")";
    let nested = "when (X > 5 && Y < 10) || (A == B && C != D) then calculate(X, Y, Z)";

    group.bench_with_input(
        BenchmarkId::new("baseline", "simple"),
        &simple,
        |b, text| b.iter(|| grl_helpers::parse_when_then(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("zero_copy", "simple"),
        &simple,
        |b, text| b.iter(|| zero_copy::parse_when_then_zero_copy(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("baseline", "complex"),
        &complex,
        |b, text| b.iter(|| grl_helpers::parse_when_then(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("zero_copy", "complex"),
        &complex,
        |b, text| b.iter(|| zero_copy::parse_when_then_zero_copy(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("baseline", "nested"),
        &nested,
        |b, text| b.iter(|| grl_helpers::parse_when_then(black_box(text))),
    );

    group.bench_with_input(
        BenchmarkId::new("zero_copy", "nested"),
        &nested,
        |b, text| b.iter(|| zero_copy::parse_when_then_zero_copy(black_box(text))),
    );

    group.finish();
}

// Benchmark 3: Rule Splitting
fn bench_rule_splitting(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_splitting");

    let sizes = vec![10, 50, 100, 500];

    for size in sizes {
        let grl = generate_grl(size);
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("baseline", size), &grl, |b, text| {
            b.iter(|| grl_helpers::split_into_rules(black_box(text)))
        });

        group.bench_with_input(BenchmarkId::new("simd", size), &grl, |b, text| {
            b.iter(|| simd_search::split_into_rules_simd(black_box(text)))
        });

        group.bench_with_input(BenchmarkId::new("zero_copy", size), &grl, |b, text| {
            b.iter(|| zero_copy::split_into_rules_zero_copy(black_box(text)))
        });
    }

    group.finish();
}

// Benchmark 4: Full Rule Parsing (Parallel vs Sequential)
fn bench_full_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_rule_parsing");

    let sizes = vec![10, 50, 100, 500];

    for size in sizes {
        let grl = generate_grl(size);
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("adaptive", size), &grl, |b, text| {
            b.iter(|| parallel::parse_rules_adaptive(black_box(text)))
        });

        group.bench_with_input(BenchmarkId::new("parallel", size), &grl, |b, text| {
            b.iter(|| parallel::parse_rules_parallel(black_box(text)))
        });

        group.bench_with_input(BenchmarkId::new("parallel_simd", size), &grl, |b, text| {
            b.iter(|| parallel::parse_rules_parallel_simd(black_box(text)))
        });

        if size >= 100 {
            group.bench_with_input(
                BenchmarkId::new("chunked_parallel", size),
                &grl,
                |b, text| b.iter(|| parallel::parse_rules_chunked_parallel(black_box(text), 20)),
            );
        }
    }

    group.finish();
}

// Benchmark 5: Brace Matching
fn bench_brace_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("brace_matching");

    let simple = "{ simple body }";
    let nested = "{ outer { inner { deeply_nested } } }";
    let complex = "{ condition1 && condition2 { action1; action2 } || { fallback } }";

    for (text, name) in &[(simple, "simple"), (nested, "nested"), (complex, "complex")] {
        group.bench_with_input(BenchmarkId::new("baseline", name), text, |b, t| {
            b.iter(|| {
                use rust_rule_engine::parser::literal_search;
                literal_search::find_matching_brace(black_box(t), 0)
            })
        });

        group.bench_with_input(BenchmarkId::new("simd", name), text, |b, t| {
            b.iter(|| simd_search::find_matching_brace_simd(black_box(t), 0))
        });
    }

    group.finish();
}

// Benchmark 6: Line Counting (for error reporting)
fn bench_line_counting(c: &mut Criterion) {
    let mut group = c.benchmark_group("line_counting");

    let sizes = vec![100, 500, 1000];

    for size in sizes {
        let grl = generate_grl(size);
        let line_count = grl.lines().count();
        group.throughput(Throughput::Elements(line_count as u64));

        group.bench_with_input(BenchmarkId::new("baseline", size), &grl, |b, text| {
            b.iter(|| black_box(text).lines().count())
        });

        group.bench_with_input(BenchmarkId::new("simd", size), &grl, |b, text| {
            b.iter(|| simd_search::count_lines_simd(black_box(text)))
        });
    }

    group.finish();
}

// Benchmark 7: Keyword Search (multi-pattern)
fn bench_keyword_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("keyword_search");

    let grl = generate_grl(100);
    let keywords = vec!["when", "then", "rule", "salience", "log"];

    group.bench_function("aho_corasick", |b| {
        b.iter(|| simd_search::find_keywords_simd(black_box(&grl), black_box(&keywords)))
    });

    group.finish();
}

// Benchmark 8: Memory Allocation Comparison
fn bench_memory_allocations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocations");

    let grl = generate_grl(50);

    group.bench_function("baseline_with_allocations", |b| {
        b.iter(|| {
            let rules = grl_helpers::split_into_rules(black_box(&grl));
            black_box(rules)
        })
    });

    group.bench_function("zero_copy_no_allocations", |b| {
        b.iter(|| {
            let rules = zero_copy::split_into_rules_zero_copy(black_box(&grl));
            // Just count to avoid measuring Vec allocation
            black_box(rules.len())
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_rule_header,
    bench_when_then,
    bench_rule_splitting,
    bench_full_parsing,
    bench_brace_matching,
    bench_line_counting,
    bench_keyword_search,
    bench_memory_allocations,
);

criterion_main!(benches);

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_rule_engine::parser::{grl_helpers, literal_search};

#[cfg(feature = "legacy-regex-parser")]
use regex::Regex;

fn bench_email_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("email_validation");

    let emails = vec![
        "user@example.com",
        "john.doe+tag@company.co.uk",
        "admin_2024@test-domain.org",
        "test.user.name@sub.domain.com",
        "invalid@",
        "@missing.com",
    ];

    // Literal search benchmark
    group.bench_function("literal_search", |b| {
        b.iter(|| {
            for email in &emails {
                black_box(literal_search::is_valid_email_literal(email));
            }
        });
    });

    // Regex benchmark (if feature enabled)
    #[cfg(feature = "legacy-regex-parser")]
    {
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        group.bench_function("regex", |b| {
            b.iter(|| {
                for email in &emails {
                    black_box(email_regex.is_match(email));
                }
            });
        });
    }

    group.finish();
}

fn bench_rule_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_parsing");

    let rule_text = r#"rule "MyRule" salience 10 { when X > 5 && Y < 10 then Z = 15 }"#;

    // Literal search benchmark
    group.bench_function("literal_search", |b| {
        b.iter(|| {
            black_box(grl_helpers::parse_rule_header(rule_text));
            black_box(grl_helpers::extract_salience(rule_text));
        });
    });

    // Regex benchmark (if feature enabled)
    #[cfg(feature = "legacy-regex-parser")]
    {
        let rule_regex = Regex::new(r#"rule\s+(?:"([^"]+)"|([a-zA-Z_]\w*))"#).unwrap();
        let salience_regex = Regex::new(r"salience\s+(\d+)").unwrap();

        group.bench_function("regex", |b| {
            b.iter(|| {
                black_box(rule_regex.captures(rule_text));
                black_box(salience_regex.captures(rule_text));
            });
        });
    }

    group.finish();
}

fn bench_when_then_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("when_then_parsing");

    let when_then = "when X > 5 && Y < 10 && Z == 15 then A = 20; B = 30; C = 40";

    // Literal search benchmark
    group.bench_function("literal_search", |b| {
        b.iter(|| {
            black_box(grl_helpers::parse_when_then(when_then));
        });
    });

    // Regex benchmark (if feature enabled)
    #[cfg(feature = "legacy-regex-parser")]
    {
        let when_then_regex = Regex::new(r"when\s+(.+?)\s+then\s+(.+)").unwrap();

        group.bench_function("regex", |b| {
            b.iter(|| {
                black_box(when_then_regex.captures(when_then));
            });
        });
    }

    group.finish();
}

fn bench_multi_pattern_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_pattern_search");

    let text = r#"
    rule "Rule1" { when X > 5 then Y = 10 }
    rule "Rule2" { when A < 3 then B = 7 }
    rule "Rule3" { when C == 8 then D = 12 }
    "#;

    let patterns = ["rule", "when", "then", "salience", "agenda-group"];

    // Aho-Corasick benchmark
    group.bench_function("aho_corasick", |b| {
        b.iter(|| {
            black_box(literal_search::find_all_patterns(text, &patterns));
        });
    });

    // Sequential regex benchmark (if feature enabled)
    #[cfg(feature = "legacy-regex-parser")]
    {
        let regexes: Vec<Regex> = patterns.iter().map(|p| Regex::new(p).unwrap()).collect();

        group.bench_function("sequential_regex", |b| {
            b.iter(|| {
                for re in &regexes {
                    black_box(re.find_iter(text).count());
                }
            });
        });
    }

    group.finish();
}

fn bench_operator_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("operator_parsing");

    let operators = [
        ">= 10", "<= 5", "== 3", "!= 7", "> 2", "< 9", "contains", "matches",
    ];

    // Literal search benchmark
    group.bench_function("literal_search", |b| {
        b.iter(|| {
            for op in &operators {
                black_box(grl_helpers::parse_operator(op));
            }
        });
    });

    // Regex benchmark (if feature enabled)
    #[cfg(feature = "legacy-regex-parser")]
    {
        let op_regex = Regex::new(r"^(>=|<=|==|!=|>|<|contains|matches)").unwrap();

        group.bench_function("regex", |b| {
            b.iter(|| {
                for op in &operators {
                    black_box(op_regex.captures(op));
                }
            });
        });
    }

    group.finish();
}

fn bench_rule_splitting(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_splitting");

    let grl_text = r#"
    rule "Rule1" { when X > 5 then Y = 10 }
    rule "Rule2" { when A < 3 then B = 7 }
    rule "Rule3" { when C == 8 then D = 12 }
    rule "Rule4" { when E != 15 then F = 20 }
    rule "Rule5" { when G >= 25 then H = 30 }
    "#;

    // Literal search with brace matching
    group.bench_function("literal_search", |b| {
        b.iter(|| {
            black_box(grl_helpers::split_into_rules(grl_text));
        });
    });

    // Regex benchmark (if feature enabled)
    #[cfg(feature = "legacy-regex-parser")]
    {
        let rule_split_regex = Regex::new(r#"(?s)rule\s+(?:"[^"]+"|[a-zA-Z_]\w*).*?\}"#).unwrap();

        group.bench_function("regex", |b| {
            b.iter(|| {
                let matches: Vec<_> = rule_split_regex
                    .find_iter(grl_text)
                    .map(|m| m.as_str().to_string())
                    .collect();
                black_box(matches);
            });
        });
    }

    group.finish();
}

fn bench_large_text_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_text_parsing");

    // Generate large GRL text
    let mut large_text = String::new();
    for i in 0..100 {
        large_text.push_str(&format!(
            r#"rule "Rule{}" salience {} {{
                when X{} > {} && Y{} < {}
                then Z{} = {}; A{} = {}
            }}
            "#,
            i,
            i % 10,
            i,
            i * 2,
            i,
            i * 3,
            i,
            i * 4,
            i,
            i * 5
        ));
    }

    // Literal search
    group.bench_with_input(
        BenchmarkId::new("literal_search", "100_rules"),
        &large_text,
        |b, text| {
            b.iter(|| {
                black_box(grl_helpers::split_into_rules(text));
            });
        },
    );

    // Regex (if feature enabled)
    #[cfg(feature = "legacy-regex-parser")]
    {
        let rule_split_regex = Regex::new(r#"(?s)rule\s+(?:"[^"]+"|[a-zA-Z_]\w*).*?\}"#).unwrap();

        group.bench_with_input(
            BenchmarkId::new("regex", "100_rules"),
            &large_text,
            |b, text| {
                b.iter(|| {
                    let matches: Vec<_> = rule_split_regex
                        .find_iter(text)
                        .map(|m| m.as_str().to_string())
                        .collect();
                    black_box(matches);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_email_validation,
    bench_rule_parsing,
    bench_when_then_parsing,
    bench_multi_pattern_search,
    bench_operator_parsing,
    bench_rule_splitting,
    bench_large_text_parsing,
);

criterion_main!(benches);

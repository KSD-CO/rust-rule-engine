use rust_rule_engine::engine::{AnalyticsConfig, RuleAnalytics};
/// Advanced Analytics Demo
///
/// This example demonstrates the new analytics and performance monitoring capabilities
/// of the Rust Rule Engine. It shows how to:
///
/// 1. Enable analytics collection
/// 2. Configure sampling and retention policies  
/// 3. Execute rules while collecting metrics
/// 4. Analyze performance data and trends
/// 5. Get optimization recommendations
use rust_rule_engine::*;
use std::time::Duration;

fn main() -> Result<()> {
    println!("📊 Advanced Rule Engine Analytics Demo");
    println!("=====================================\n");

    // Create knowledge base with performance test rules
    let mut kb = KnowledgeBase::new("AnalyticsDemo");

    // Add rules with different performance characteristics
    create_test_rules(&mut kb)?;

    // Create engine configuration
    let config = EngineConfig {
        max_cycles: 10,
        timeout: Some(Duration::from_secs(5)),
        enable_stats: true,
        debug_mode: false,
    };

    let mut engine = RustRuleEngine::with_config(kb, config);

    // Configure analytics with production settings
    let analytics_config = AnalyticsConfig {
        track_execution_time: true,
        track_memory_usage: true,
        track_success_rate: true,
        sampling_rate: 0.8, // Sample 80% of executions
        retention_period: Duration::from_secs(30 * 24 * 60 * 60), // 30 days
        max_recent_samples: 100,
    };

    let analytics = RuleAnalytics::new(analytics_config);
    engine.enable_analytics(analytics);

    println!("🔧 Analytics Configuration:");
    if let Some(analytics) = engine.analytics() {
        println!(
            "   • Sampling Rate: {:.1}%",
            analytics.config().sampling_rate * 100.0
        );
        println!(
            "   • Retention: {} days",
            analytics.config().retention_period.as_secs() / (24 * 60 * 60)
        );
        println!(
            "   • Max Samples: {}",
            analytics.config().max_recent_samples
        );
        println!(
            "   • Track Timing: {}",
            analytics.config().track_execution_time
        );
    }
    println!();

    // Run multiple scenarios to collect analytics data
    run_analytics_scenarios(&mut engine)?;

    // Display comprehensive analytics report
    display_analytics_report(&engine);

    Ok(())
}

fn create_test_rules(kb: &mut KnowledgeBase) -> Result<()> {
    // Fast rule - simple condition
    let fast_rule = r#"
    rule "FastRule" salience 100 {
        when
            User.Age > 18
        then
            User.IsAdult = true;
    }
    "#;

    // Medium rule - moderate complexity
    let medium_rule = r#"
    rule "MediumRule" salience 50 {
        when
            User.Age >= 21 && User.Country == "US" && User.HasLicense == true
        then
            User.CanDrink = true;
            User.DiscountRate = 0.05;
    }
    "#;

    // Slow rule - complex conditions
    let slow_rule = r#"
    rule "SlowRule" salience 25 {
        when
            (User.Age >= 25 && User.Income > 50000 && User.CreditScore > 700) ||
            (User.Age >= 30 && User.Income > 40000) ||
            (User.IsVIP == true && User.TotalSpent > 10000)
        then
            User.IsEligibleForPremium = true;
            User.PremiumDiscount = 0.15;
            User.FreeShipping = true;
    }
    "#;

    // Variable rule - sometimes fires, sometimes doesn't
    let variable_rule = r#"
    rule "VariableRule" salience 75 {
        when
            User.Age % 3 == 0 && User.Country != "RESTRICTED"
        then
            User.LuckyNumber = User.Age * 7;
    }
    "#;

    // Failing rule - designed to rarely fire
    let failing_rule = r#"
    rule "RareRule" salience 10 {
        when
            User.Age == 42 && User.Country == "ATLANTIS"
        then
            User.IsSpecial = true;
    }
    "#;

    // Parse and add all rules
    for rule_grl in [
        fast_rule,
        medium_rule,
        slow_rule,
        variable_rule,
        failing_rule,
    ] {
        let rules = GRLParser::parse_rules(rule_grl)?;
        for rule in rules {
            kb.add_rule(rule)?;
        }
    }

    Ok(())
}

fn run_analytics_scenarios(engine: &mut RustRuleEngine) -> Result<()> {
    println!("🚀 Running Analytics Collection Scenarios");
    println!("==========================================\n");

    let scenarios = [
        ("Young Adult", 19, "US", true, 30000, 650, false, 500.0),
        ("College Student", 21, "US", true, 15000, 600, false, 200.0),
        ("Professional", 28, "US", true, 75000, 750, false, 2000.0),
        ("Senior", 35, "CA", true, 60000, 720, false, 3000.0),
        ("VIP Customer", 42, "US", true, 100000, 800, true, 15000.0),
        ("International", 30, "UK", true, 45000, 680, false, 1200.0),
        ("High Earner", 33, "US", true, 120000, 790, false, 5000.0),
        ("Lucky Customer", 27, "US", true, 55000, 700, false, 1800.0), // Age % 3 == 0
        (
            "Special Case",
            42,
            "ATLANTIS",
            true,
            80000,
            750,
            false,
            3000.0,
        ), // Should trigger rare rule
        (
            "Restricted",
            24,
            "RESTRICTED",
            true,
            40000,
            650,
            false,
            800.0,
        ), // Age % 3 == 0 but restricted
    ];

    for (i, (name, age, country, has_license, income, credit_score, is_vip, total_spent)) in
        scenarios.iter().enumerate()
    {
        println!("   {}. Testing scenario: {}", i + 1, name);

        // Create facts for this scenario
        let facts = Facts::new();
        let user = FactHelper::create_object(vec![
            ("Age", Value::Integer(*age)),
            ("Country", Value::String(country.to_string())),
            ("HasLicense", Value::Boolean(*has_license)),
            ("Income", Value::Number(*income as f64)),
            ("CreditScore", Value::Integer(*credit_score)),
            ("IsVIP", Value::Boolean(*is_vip)),
            ("TotalSpent", Value::Number(*total_spent)),
        ]);
        facts.add_value("User", user)?;

        // Execute rules and collect metrics
        let result = engine.execute(&facts)?;

        println!(
            "      ✓ Fired {} rules in {} cycles",
            result.rules_fired, result.cycle_count
        );
    }

    println!();
    Ok(())
}

fn display_analytics_report(engine: &RustRuleEngine) {
    println!("📈 Analytics Report");
    println!("==================\n");

    if let Some(analytics) = engine.analytics() {
        let stats = analytics.get_overall_stats();

        println!("📊 Overall Performance:");
        println!("   • Total Executions: {}", stats.total_evaluations);
        println!(
            "   • Average Duration: {:.2}ms",
            stats.avg_execution_time.as_secs_f64() * 1000.0
        );
        println!("   • Success Rate: {:.1}%", stats.success_rate * 100.0);
        println!(
            "   • Rules Analyzed: {}",
            analytics.get_all_rule_metrics().len()
        );
        println!();

        println!("🏆 Top Performing Rules:");
        let mut rule_metrics: Vec<_> = analytics.get_all_rule_metrics().iter().collect();
        rule_metrics.sort_by(|a, b| a.1.avg_execution_time().cmp(&b.1.avg_execution_time()));

        for (rule_name, metrics) in rule_metrics.iter().take(3) {
            println!(
                "   • {} ({:.2}ms avg, {:.1}% success)",
                rule_name,
                metrics.avg_execution_time().as_secs_f64() * 1000.0,
                metrics.success_rate()
            );
        }
        println!();

        println!("⚠️  Performance Concerns:");
        for (rule_name, metrics) in rule_metrics.iter().rev().take(2) {
            if metrics.avg_execution_time().as_millis() > 1 {
                println!(
                    "   • {} is slower than average ({:.2}ms)",
                    rule_name,
                    metrics.avg_execution_time().as_secs_f64() * 1000.0
                );
            }
        }
        println!();

        println!("🎯 Success Rate Analysis:");
        for (rule_name, metrics) in analytics.get_all_rule_metrics().iter() {
            let rate = metrics.success_rate();
            let status = if rate > 80.0 {
                "🟢"
            } else if rate > 50.0 {
                "🟡"
            } else {
                "🔴"
            };
            println!(
                "   {} {} ({:.1}% - {} executions)",
                status, rule_name, rate, metrics.total_evaluations
            );
        }
        println!();

        println!("🔮 Optimization Recommendations:");
        let recommendations = analytics.generate_recommendations();
        if recommendations.is_empty() {
            println!("   ✅ All rules are performing well!");
        } else {
            for recommendation in recommendations {
                println!("   💡 {}", recommendation);
            }
        }
        println!();

        println!("📅 Recent Activity:");
        let events = analytics.get_recent_events(5);
        if events.is_empty() {
            println!("   No recent events recorded");
        } else {
            for event in events {
                let status = if event.success { "✅" } else { "❌" };
                println!(
                    "   {} {} - {:.2}ms",
                    status,
                    event.rule_name,
                    event.duration.as_secs_f64() * 1000.0
                );
            }
        }
    } else {
        println!("   ❌ Analytics not enabled");
    }

    println!("\n🎉 Analytics demo completed!");
    println!("   The analytics system provides deep insights into rule performance,");
    println!("   helping you optimize your rule engine for production workloads.");
}

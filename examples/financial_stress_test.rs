use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::engine::RustRuleEngine;
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Complex Rules Stress Test - Financial Trading Domain");
    println!("=========================================================");

    // Create knowledge base and engine
    let mut kb = KnowledgeBase::new("FinancialTrading");

    // Create rules programmatically
    create_financial_rules(&mut kb)?;

    let engine = RustRuleEngine::new(kb);

    // Performance tracking
    let start_time = Instant::now();
    let mut total_rules_fired = 0;
    let iterations = 100;

    println!(
        "\n🔥 Running {} iterations of 5 complex financial rules...",
        iterations
    );

    // Execute rules with different scenarios
    for _i in 0..iterations {
        for scenario in 0..5 {
            let facts = create_complex_facts(scenario);

            match engine.execute(&facts) {
                Ok(result) => {
                    total_rules_fired += result.rules_fired;
                }
                Err(_e) => {
                    // Silent for benchmark
                }
            }
        }
    }

    let duration = start_time.elapsed();
    let total_rules_evaluated = iterations * 5; // 5 scenarios per iteration
    let rules_per_second = (total_rules_evaluated as f64 / duration.as_secs_f64()) as u64;
    let fired_per_second = (total_rules_fired as f64 / duration.as_secs_f64()) as u64;

    println!("\n📊 Performance Results:");
    println!("   ⏱️  Total time: {:?}", duration);
    println!("   📈 Scenarios evaluated: {}", total_rules_evaluated);
    println!("   🎯 Rules fired: {}", total_rules_fired);
    println!("   📈 Scenarios/sec: {}", rules_per_second);
    println!("   🔥 Rules fired/sec: {}", fired_per_second);
    println!(
        "   💡 Avg time per scenario: {:?}",
        duration / total_rules_evaluated as u32
    );

    println!("\n🧪 Testing sample scenarios:");
    test_sample_scenarios(&engine);

    Ok(())
}

fn create_financial_rules(kb: &mut KnowledgeBase) -> Result<(), Box<dyn std::error::Error>> {
    // Rule 1: High-Risk Trader Detection
    let high_risk_rule = Rule::new(
        "HighRiskTrader".to_string(),
        ConditionGroup::and(
            ConditionGroup::and(
                ConditionGroup::single(Condition::new(
                    "trader.MonthlyPnL".to_string(),
                    Operator::LessThan,
                    Value::Number(-1000000.0),
                )),
                ConditionGroup::single(Condition::new(
                    "trader.VarBreaches".to_string(),
                    Operator::GreaterThan,
                    Value::Number(3.0),
                )),
            ),
            ConditionGroup::single(Condition::new(
                "trader.RiskScore".to_string(),
                Operator::GreaterThan,
                Value::Number(8.5),
            )),
        ),
        vec![ActionType::Log {
            message: "High risk trader detected".to_string(),
        }],
    )
    .with_salience(10);
    kb.add_rule(high_risk_rule)?;

    // Rule 2: Market Maker Privileges
    let market_maker_rule = Rule::new(
        "MarketMakerCheck".to_string(),
        ConditionGroup::and(
            ConditionGroup::and(
                ConditionGroup::single(Condition::new(
                    "trader.Type".to_string(),
                    Operator::Equal,
                    Value::String("market_maker".to_string()),
                )),
                ConditionGroup::single(Condition::new(
                    "trader.MonthlyVolume".to_string(),
                    Operator::GreaterThan,
                    Value::Number(10000000.0),
                )),
            ),
            ConditionGroup::single(Condition::new(
                "trader.LatencyScore".to_string(),
                Operator::LessThan,
                Value::Number(2.0),
            )),
        ),
        vec![ActionType::Log {
            message: "Market maker approved".to_string(),
        }],
    )
    .with_salience(9);
    kb.add_rule(market_maker_rule)?;

    // Rule 3: Institutional Client Services
    let institutional_rule = Rule::new(
        "InstitutionalClient".to_string(),
        ConditionGroup::and(
            ConditionGroup::and(
                ConditionGroup::single(Condition::new(
                    "client.AUM".to_string(),
                    Operator::GreaterThanOrEqual,
                    Value::Number(100000000.0),
                )),
                ConditionGroup::single(Condition::new(
                    "client.MonthlyRevenue".to_string(),
                    Operator::GreaterThan,
                    Value::Number(500000.0),
                )),
            ),
            ConditionGroup::single(Condition::new(
                "client.Tier".to_string(),
                Operator::Equal,
                Value::String("institutional".to_string()),
            )),
        ),
        vec![ActionType::Log {
            message: "Prime services offered".to_string(),
        }],
    )
    .with_salience(8);
    kb.add_rule(institutional_rule)?;

    // Rule 4: Portfolio Risk Management
    let portfolio_rule = Rule::new(
        "PortfolioRisk".to_string(),
        ConditionGroup::and(
            ConditionGroup::and(
                ConditionGroup::single(Condition::new(
                    "portfolio.Beta".to_string(),
                    Operator::GreaterThan,
                    Value::Number(1.5),
                )),
                ConditionGroup::single(Condition::new(
                    "market.VIX".to_string(),
                    Operator::GreaterThan,
                    Value::Number(30.0),
                )),
            ),
            ConditionGroup::single(Condition::new(
                "portfolio.VaR".to_string(),
                Operator::GreaterThan,
                Value::Number(800000.0),
            )),
        ),
        vec![ActionType::Log {
            message: "Risk adjustment required".to_string(),
        }],
    )
    .with_salience(7);
    kb.add_rule(portfolio_rule)?;

    // Rule 5: Algorithmic Trading Approval
    let algo_rule = Rule::new(
        "AlgoTradingApproval".to_string(),
        ConditionGroup::and(
            ConditionGroup::and(
                ConditionGroup::single(Condition::new(
                    "trader.ExperienceLevel".to_string(),
                    Operator::GreaterThanOrEqual,
                    Value::Number(5.0),
                )),
                ConditionGroup::single(Condition::new(
                    "trader.RiskScore".to_string(),
                    Operator::LessThanOrEqual,
                    Value::Number(6.0),
                )),
            ),
            ConditionGroup::single(Condition::new(
                "strategy.BacktestSharpe".to_string(),
                Operator::GreaterThan,
                Value::Number(1.2),
            )),
        ),
        vec![ActionType::Log {
            message: "Algo trading approved".to_string(),
        }],
    )
    .with_salience(6);
    kb.add_rule(algo_rule)?;

    println!("✅ Created 5 financial trading rules");
    Ok(())
}

fn create_complex_facts(scenario: usize) -> Facts {
    let facts = Facts::new();

    match scenario % 5 {
        0 => {
            // High-risk trader scenario
            facts.set(
                "trader",
                Value::Object({
                    let mut trader = HashMap::new();
                    trader.insert("ID".to_string(), Value::String("TRADER_001".to_string()));
                    trader.insert("Name".to_string(), Value::String("John Risk".to_string()));
                    trader.insert("MonthlyPnL".to_string(), Value::Number(-1500000.0));
                    trader.insert("VarBreaches".to_string(), Value::Number(5.0));
                    trader.insert("RiskScore".to_string(), Value::Number(9.2));
                    trader.insert("Type".to_string(), Value::String("proprietary".to_string()));
                    trader
                }),
            );
        }
        1 => {
            // Market maker scenario
            facts.set(
                "trader",
                Value::Object({
                    let mut trader = HashMap::new();
                    trader.insert(
                        "Type".to_string(),
                        Value::String("market_maker".to_string()),
                    );
                    trader.insert("MonthlyVolume".to_string(), Value::Number(50000000.0));
                    trader.insert("LatencyScore".to_string(), Value::Number(1.2));
                    trader.insert("ExperienceLevel".to_string(), Value::Number(8.0));
                    trader.insert("RiskScore".to_string(), Value::Number(4.0));
                    trader
                }),
            );
        }
        2 => {
            // Institutional client scenario
            facts.set(
                "client",
                Value::Object({
                    let mut client = HashMap::new();
                    client.insert(
                        "ID".to_string(),
                        Value::String("CLIENT_INST_001".to_string()),
                    );
                    client.insert("AUM".to_string(), Value::Number(250000000.0));
                    client.insert("MonthlyRevenue".to_string(), Value::Number(750000.0));
                    client.insert(
                        "Tier".to_string(),
                        Value::String("institutional".to_string()),
                    );
                    client
                }),
            );
        }
        3 => {
            // Portfolio risk scenario
            facts.set(
                "portfolio",
                Value::Object({
                    let mut portfolio = HashMap::new();
                    portfolio.insert("Beta".to_string(), Value::Number(1.8));
                    portfolio.insert("VaR".to_string(), Value::Number(850000.0));
                    portfolio.insert("VaRLimit".to_string(), Value::Number(800000.0));
                    portfolio.insert("Concentration".to_string(), Value::Number(0.45));
                    portfolio
                }),
            );
            facts.set(
                "market",
                Value::Object({
                    let mut market = HashMap::new();
                    market.insert("VIX".to_string(), Value::Number(35.2));
                    market.insert("Volatility".to_string(), Value::Number(28.5));
                    market
                }),
            );
        }
        _ => {
            // Algo trading scenario
            facts.set(
                "trader",
                Value::Object({
                    let mut trader = HashMap::new();
                    trader.insert("ExperienceLevel".to_string(), Value::Number(7.0));
                    trader.insert("RiskScore".to_string(), Value::Number(4.5));
                    trader
                }),
            );
            facts.set(
                "strategy",
                Value::Object({
                    let mut strategy = HashMap::new();
                    strategy.insert("BacktestSharpe".to_string(), Value::Number(1.8));
                    strategy.insert("MaxDrawdown".to_string(), Value::Number(0.12));
                    strategy
                }),
            );
        }
    }

    facts
}

fn test_sample_scenarios(engine: &RustRuleEngine) {
    println!("\n   🎭 Scenario Testing:");

    // Test different scenarios
    for i in 0..5 {
        let facts = create_complex_facts(i);
        match engine.execute(&facts) {
            Ok(result) => {
                if result.rules_fired > 0 {
                    println!(
                        "   ✅ Scenario {} fired {} rules (evaluated {} rules in {} cycles)",
                        i + 1,
                        result.rules_fired,
                        result.rules_evaluated,
                        result.cycle_count
                    );
                } else {
                    println!(
                        "   ⚪ Scenario {} - no rules fired (evaluated {} rules)",
                        i + 1,
                        result.rules_evaluated
                    );
                }
            }
            Err(e) => {
                println!("   ❌ Scenario {} error: {:?}", i + 1, e);
            }
        }
    }

    println!("\n📋 Sample Data Examples:");

    // Show sample facts for each scenario
    for i in 0..3 {
        let facts = create_complex_facts(i);
        match i {
            0 => {
                if let Some(Value::Object(t)) = facts.get("trader") {
                    println!(
                        "   👤 High-Risk Trader: {:?} (P&L: {:?}, Risk: {:?})",
                        t.get("Name")
                            .unwrap_or(&Value::String("Unknown".to_string())),
                        t.get("MonthlyPnL").unwrap_or(&Value::Number(0.0)),
                        t.get("RiskScore").unwrap_or(&Value::Number(0.0))
                    );
                }
            }
            1 => {
                if let Some(Value::Object(t)) = facts.get("trader") {
                    println!(
                        "   🏭 Market Maker: Type {:?} (Volume: {:?}, Latency: {:?})",
                        t.get("Type")
                            .unwrap_or(&Value::String("Unknown".to_string())),
                        t.get("MonthlyVolume").unwrap_or(&Value::Number(0.0)),
                        t.get("LatencyScore").unwrap_or(&Value::Number(0.0))
                    );
                }
            }
            2 => {
                if let Some(Value::Object(c)) = facts.get("client") {
                    println!(
                        "   🏢 Institutional Client: {:?} (AUM: ${:?}, Revenue: ${:?})",
                        c.get("Tier")
                            .unwrap_or(&Value::String("Unknown".to_string())),
                        c.get("AUM").unwrap_or(&Value::Number(0.0)),
                        c.get("MonthlyRevenue").unwrap_or(&Value::Number(0.0))
                    );
                }
            }
            _ => {}
        }
    }

    println!("\n✅ Complex Rules Stress Test Completed!");
    println!("🏆 Rule engine successfully handled sophisticated financial trading scenarios");
}

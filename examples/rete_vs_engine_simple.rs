use rust_rule_engine::engine::engine::{EngineConfig, RustRuleEngine};
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::parser::grl::GRLParser;
use rust_rule_engine::types::ActionType;
use rust_rule_engine::types::Value;
use rust_rule_engine::Facts;
use std::time::Instant;

// Helper: convert parser ConditionGroup to auto_network ConditionGroup
fn convert_condition_group(src: &rust_rule_engine::ConditionGroup) -> rust_rule_engine::rete::auto_network::ConditionGroup {
    use rust_rule_engine::rete::auto_network::{ConditionGroup as AutoGroup, Condition as AutoCond};
    match src {
        rust_rule_engine::ConditionGroup::Single(cond) => {
            let op_str = match format!("{:?}", cond.operator).as_str() {
                "Eq" => "==",
                "Ne" => "!=",
                "Gt" => ">",
                "Lt" => "<",
                "Ge" => ">=",
                "Le" => "<=",
                _ => "==",
            };
            let val_str = match &cond.value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Integer(i) => i.to_string(),
                Value::Boolean(b) => b.to_string(),
                _ => cond.value.to_string(),
            };
            AutoGroup::Single(AutoCond {
                field: cond.field.clone(),
                operator: op_str.to_string(),
                value: val_str,
            })
        }
        rust_rule_engine::ConditionGroup::Compound { left, operator, right } => {
            let op_str = match format!("{:?}", operator).as_str() {
                "And" => "AND",
                "Or" => "OR",
                _ => "AND",
            };
            AutoGroup::Compound {
                left: Box::new(convert_condition_group(left)),
                operator: op_str.to_string(),
                right: Box::new(convert_condition_group(right)),
            }
        }
        rust_rule_engine::ConditionGroup::Not(inner) => {
            AutoGroup::Not(Box::new(convert_condition_group(inner)))
        }
        rust_rule_engine::ConditionGroup::Exists(inner) => {
            AutoGroup::Exists(Box::new(convert_condition_group(inner)))
        }
        rust_rule_engine::ConditionGroup::Forall(inner) => {
            AutoGroup::Forall(Box::new(convert_condition_group(inner)))
        }
        rust_rule_engine::ConditionGroup::Accumulate { .. } => {
            // Accumulate is not supported in auto_network yet
            // For now, convert to a simple true condition
            AutoGroup::Single(AutoCond {
                field: "true".to_string(),
                operator: "==".to_string(),
                value: "true".to_string(),
            })
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔬 RETE vs Engine Simple Example");
    println!("===============================");

    // Simple facts
    let mut facts = Facts::new();
    facts.set("age", Value::Integer(25));
    facts.set("status", Value::String("active".to_string()));
    facts.set("score", Value::Number(88.5));

    println!("Initial facts: age=25, status=active, score=88.5");

    // Simple rules
    let simple_rules = vec![
        r#"
        rule "IsAdult" {
            when age >= 18 then log("Is adult"); }
        "#,
        r#"
        rule "IsActive" {
            when status == "active" then log("Is active"); }
        "#,
        r#"
        rule "HighScore" {
            when score > 80 then log("High score"); }
        "#,
    ];

    // Parse and add rules to engine
    let mut engine = RustRuleEngine::with_config(KnowledgeBase::new("SimpleDemo"), EngineConfig::default());
    let mut rete_rules = Vec::new();
    for rule_grl in &simple_rules {
        let rule = GRLParser::parse_rule(rule_grl)?;
        engine.knowledge_base().add_rule(rule.clone())?;
        let auto_rule = rust_rule_engine::rete::auto_network::Rule {
            name: rule.name.clone(),
            conditions: convert_condition_group(&rule.conditions),
            action: rule.actions.get(0).map(|a| format!("{:?}", a)).unwrap_or_default(),
        };
        rete_rules.push(auto_rule);
    }

    // Run engine with Drools-style fired flag (each rule fires once)
    let mut facts_engine = facts.clone();
    let mut engine_fired = Vec::new();
    let start_engine = Instant::now();
    for rule_grl in &simple_rules {
        let rule = GRLParser::parse_rule(rule_grl)?;
        let fired_flag = format!("{}_fired", rule.name);
        if facts_engine.get(&fired_flag) == Some(Value::String("true".to_string())) {
            continue;
        }
        // Convert Facts to HashMap<String, Value> for evaluation
        let facts_map = facts_engine.get_all_facts();
        if rule.conditions.evaluate(&facts_map) {
            // Only fire if not fired before
            if let Some(action) = rule.actions.get(0) {
                match action {
                    ActionType::Log { message } => {
                        println!("📋 LOG: {}", message);
                    }
                    _ => {}
                }
            }
            facts_engine.set(&fired_flag, Value::String("true".to_string()));
            engine_fired.push(rule.name.clone());
        }
    }
    let duration_engine = start_engine.elapsed();
    println!("\nEngine results (Drools-style):");
    println!("  Rules fired: {}", engine_fired.len());
    println!("  Fired rules: {:?}", engine_fired);
    println!("  Time: {:?}", duration_engine);

    // Run RETE-UL with Drools-style fired flag (each rule fires once)
    use rust_rule_engine::rete::auto_network::build_rete_ul_from_rule;
    use rust_rule_engine::rete::network::{ReteUlRule, fire_rete_ul_rules_with_agenda};
    // Prepare fresh facts for RETE-UL (no fired flags)
    let mut facts_map = std::collections::HashMap::new();
    for (k, v) in facts.get_all_facts().iter() {
        let s = match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => format!("{:.1}", n),
            Value::Integer(i) => i.to_string(),
            Value::Boolean(b) => if *b { "true".to_string() } else { "false".to_string() },
            _ => v.to_string(),
        };
        // Only insert original fact keys, skip *_fired flags
        if !k.ends_with("_fired") {
            facts_map.insert(k.clone(), s);
        }
    }
    println!("RETE-UL initial facts: {:?}", facts_map);
    let mut rete_ul_rules = Vec::new();
    for rule in &rete_rules {
        let rete_node = build_rete_ul_from_rule(rule);
        let priority = match rule.name.as_str() {
            "HighScore" => 2,
            "IsAdult" => 1,
            "IsActive" => 1,
            _ => 0,
        };
        let no_loop = true; // Drools-style: only fire once per rule
        let rule_name_for_action = rule.name.clone();
        let rule_name_for_struct = rule.name.clone();
        let action = std::sync::Arc::new(move |facts: &mut std::collections::HashMap<String, String>| {
            let fired_flag = format!("{}_fired", rule_name_for_action);
            if facts.get(&fired_flag) == Some(&"true".to_string()) {
                return;
            }
            match rule_name_for_action.as_str() {
                "IsAdult" => { facts.insert("is_adult".to_string(), "true".to_string()); },
                "IsActive" => { facts.insert("is_active".to_string(), "true".to_string()); },
                "HighScore" => { facts.insert("high_score".to_string(), "true".to_string()); },
                _ => {}
            }
            facts.insert(fired_flag, "true".to_string());
        });
        rete_ul_rules.push(ReteUlRule {
            name: rule_name_for_struct,
            node: rete_node,
            priority,
            no_loop,
            action,
        });
    }
    let fired_rules = fire_rete_ul_rules_with_agenda(&mut rete_ul_rules, &mut facts_map);
    println!("\nRETE-UL Advanced (Drools-style): Fired {} / {}", fired_rules.len(), rete_ul_rules.len());
    println!("RETE-UL fired rules: {:?}", fired_rules);
    println!("Final facts: {:?}", facts_map);

    Ok(())
}

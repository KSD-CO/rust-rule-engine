/// Simple Dependency Analysis Test - No Safe Parallel Engine
/// Just testing the improved dependency analyzer directly
use rust_rule_engine::engine::{
    dependency::DependencyAnalyzer,
    rule::{Condition, ConditionGroup, Rule},
};
use rust_rule_engine::types::{ActionType, LogicalOperator, Operator, Value};

fn main() {
    println!("🔍 IMPROVED DEPENDENCY ANALYSIS TEST");
    println!("====================================\n");

    test_real_field_detection();
    println!("\n{}", "=".repeat(50));
    test_function_side_effects();
    println!("\n{}", "=".repeat(50));
    test_compound_conditions();
}

fn test_real_field_detection() {
    println!("📊 TEST 1: Real Field Detection from AST");

    let mut analyzer = DependencyAnalyzer::new();

    // Rules with actual field references
    let rules = vec![
        // Rule reads User.Age, writes nothing
        Rule::new(
            "AgeCheck".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Age".to_string(),
                Operator::GreaterThan,
                Value::Integer(18),
            )),
            vec![ActionType::Log {
                message: "Age validated".to_string(),
            }],
        ),
        // Rule reads User.Age, writes User.Status
        Rule::new(
            "SetStatus".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Age".to_string(),
                Operator::GreaterThan,
                Value::Integer(21),
            )),
            vec![ActionType::Set {
                field: "User.Status".to_string(),
                value: Value::String("adult".to_string()),
            }],
        ),
        // Rule reads User.Status, writes User.Benefits
        Rule::new(
            "AssignBenefits".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Status".to_string(),
                Operator::Equal,
                Value::String("adult".to_string()),
            )),
            vec![ActionType::Set {
                field: "User.Benefits".to_string(),
                value: Value::String("premium".to_string()),
            }],
        ),
    ];

    let result = analyzer.analyze(&rules);

    println!("📈 Analysis Results:");
    println!("{}", result.get_summary());

    if !result.can_parallelize_safely {
        println!("\n✅ CORRECTLY DETECTED dependencies!");
        println!("   SetStatus writes User.Status");
        println!("   AssignBenefits reads User.Status");
        println!("   → Data dependency detected → Sequential execution required");
    } else {
        println!("\n❌ MISSED dependencies!");
    }
}

fn test_function_side_effects() {
    println!("🛠️  TEST 2: Function Side Effect Detection");

    let mut analyzer = DependencyAnalyzer::new();

    let rules = vec![
        Rule::new(
            "CalculateUserScore".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Data".to_string(),
                Operator::Equal,
                Value::String("valid".to_string()),
            )),
            vec![ActionType::Call {
                function: "setUserScore".to_string(),
                args: vec![Value::Integer(85)],
            }],
        ),
        Rule::new(
            "CheckScore".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Score".to_string(),
                Operator::GreaterThan,
                Value::Integer(80),
            )),
            vec![ActionType::Set {
                field: "User.Grade".to_string(),
                value: Value::String("A".to_string()),
            }],
        ),
    ];

    let result = analyzer.analyze(&rules);

    println!("🔍 Function Analysis:");
    println!("{}", result.get_summary());

    if !result.can_parallelize_safely {
        println!("\n✅ SMART DETECTION!");
        println!("   setUserScore() function → inferred to write User.Score");
        println!("   CheckScore reads User.Score");
        println!("   → Function side effect dependency detected!");
    } else {
        println!("\n⚠️  Function side effects not detected (expected for basic implementation)");
    }
}

fn test_compound_conditions() {
    println!("🌳 TEST 3: Compound Condition Analysis");

    let mut analyzer = DependencyAnalyzer::new();

    // Complex nested condition
    let complex_condition = ConditionGroup::Compound {
        left: Box::new(ConditionGroup::Single(Condition::new(
            "User.Age".to_string(),
            Operator::GreaterThan,
            Value::Integer(21),
        ))),
        operator: LogicalOperator::And,
        right: Box::new(ConditionGroup::Not(Box::new(ConditionGroup::Single(
            Condition::new(
                "User.Blacklisted".to_string(),
                Operator::Equal,
                Value::Boolean(true),
            ),
        )))),
    };

    let rules = vec![Rule::new(
        "ComplexCheck".to_string(),
        complex_condition,
        vec![ActionType::Set {
            field: "User.Approved".to_string(),
            value: Value::Boolean(true),
        }],
    )];

    println!("🌲 Condition: (User.Age > 21) AND NOT(User.Blacklisted == true)");
    println!("   Expected reads: User.Age, User.Blacklisted");
    println!("   Expected writes: User.Approved");

    let result = analyzer.analyze(&rules);

    println!("\n📊 Compound Analysis:");
    println!("{}", result.get_detailed_report());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_extraction() {
        let mut analyzer = DependencyAnalyzer::new();

        let rule = Rule::new(
            "TestRule".to_string(),
            ConditionGroup::Single(Condition::new(
                "Input.Field".to_string(),
                Operator::Equal,
                Value::String("test".to_string()),
            )),
            vec![ActionType::Set {
                field: "Output.Field".to_string(),
                value: Value::String("result".to_string()),
            }],
        );

        let rules = vec![rule];
        let result = analyzer.analyze(&rules);

        // Should detect that this rule reads Input.Field and writes Output.Field
        assert_eq!(result.total_rules, 1);
        assert!(result.can_parallelize_safely); // Single rule should be safe
    }
}

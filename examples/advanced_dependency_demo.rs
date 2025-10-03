/// Demo: Advanced Dependency Analysis with Proper Field Detection
/// This example shows how the improved dependency analyzer properly detects
/// field reads and writes from actual rule conditions and actions.

use rust_rule_engine::engine::{
    dependency::{DependencyAnalyzer, DependencyAnalysisResult},
    rule::{Rule, Condition, ConditionGroup},
};
use rust_rule_engine::types::{ActionType, ComparisonOperator, Value};
use std::collections::HashMap;

fn main() {
    println!("🔍 ADVANCED DEPENDENCY ANALYSIS DEMO");
    println!("====================================\n");

    // Demo 1: Real Field Detection vs Old Hard-coded Detection
    demo_real_vs_hardcoded_detection();
    
    println!("\n" + "=".repeat(60).as_str() + "\n");
    
    // Demo 2: Complex Rule Dependencies
    demo_complex_dependencies();
    
    println!("\n" + "=".repeat(60).as_str() + "\n");
    
    // Demo 3: Function Call Analysis
    demo_function_call_analysis();
    
    println!("\n" + "=".repeat(60).as_str() + "\n");
    
    // Demo 4: Compound Condition Analysis
    demo_compound_conditions();
}

fn demo_real_vs_hardcoded_detection() {
    println!("📊 DEMO 1: Real Field Detection vs Hard-coded");
    println!("   Testing proper AST parsing vs name-based detection\n");
    
    let mut analyzer = DependencyAnalyzer::new();
    
    // Create rules with ACTUAL field references in conditions and actions
    let rules = vec![
        // Rule 1: Reads User.Age, writes nothing
        Rule::new(
            "ValidateUserAge".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Age".to_string(),
                ComparisonOperator::GreaterThanOrEqual,
                Value::Integer(18),
            )),
            vec![ActionType::Log {
                message: "User age validated".to_string(),
            }],
        ),
        
        // Rule 2: Reads User.Country, writes nothing
        Rule::new(
            "CheckCountryEligibility".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Country".to_string(),
                ComparisonOperator::Equal,
                Value::String("US".to_string()),
            )),
            vec![ActionType::Log {
                message: "Country eligibility checked".to_string(),
            }],
        ),
        
        // Rule 3: Reads User.Age and User.Country, writes User.EligibilityScore
        Rule::new(
            "CalculateEligibilityScore".to_string(),
            ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition::new(
                    "User.Age".to_string(),
                    ComparisonOperator::GreaterThan,
                    Value::Integer(21),
                ))),
                operator: rust_rule_engine::types::LogicalOperator::And,
                right: Box::new(ConditionGroup::Single(Condition::new(
                    "User.Country".to_string(),
                    ComparisonOperator::Equal,
                    Value::String("US".to_string()),
                ))),
            },
            vec![ActionType::Set {
                field: "User.EligibilityScore".to_string(),
                value: Value::Integer(100),
            }],
        ),
        
        // Rule 4: Reads User.EligibilityScore, writes User.IsVIP
        Rule::new(
            "DetermineVIPStatus".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.EligibilityScore".to_string(),
                ComparisonOperator::GreaterThanOrEqual,
                Value::Integer(80),
            )),
            vec![ActionType::Set {
                field: "User.IsVIP".to_string(),
                value: Value::Boolean(true),
            }],
        ),
    ];
    
    println!("🔍 Rules to analyze:");
    for (i, rule) in rules.iter().enumerate() {
        println!("   {}. {} - analyzing ACTUAL conditions and actions", i + 1, rule.name);
    }
    
    let result = analyzer.analyze(&rules);
    
    println!("\n{}", result.get_detailed_report());
    
    // Show the difference
    println!("\n✅ IMPROVEMENTS:");
    println!("   ❌ Old: Hard-coded rule name parsing (\"CalculateScore\" → User.Score)");
    println!("   ✅ New: Actual AST analysis (condition.field, action.field)");
    println!("   ❌ Old: Pattern matching on rule names");
    println!("   ✅ New: Recursive condition tree traversal");
    println!("   ❌ Old: False positives/negatives");
    println!("   ✅ New: Precise field dependency detection");
}

fn demo_complex_dependencies() {
    println!("🧩 DEMO 2: Complex Rule Dependencies");
    println!("   Testing real-world rule chains with multiple dependencies\n");
    
    let mut analyzer = DependencyAnalyzer::new();
    
    let rules = vec![
        // Step 1: Calculate base score (writes Order.BaseScore)
        Rule::new(
            "CalculateBaseScore".to_string(),
            ConditionGroup::Single(Condition::new(
                "Order.Amount".to_string(),
                ComparisonOperator::GreaterThan,
                Value::Number(0.0),
            )),
            vec![ActionType::Call {
                function: "calculateOrderScore".to_string(),
                args: vec![],
            }],
        ),
        
        // Step 2: Apply user multiplier (reads User.Level, Order.BaseScore, writes Order.AdjustedScore)
        Rule::new(
            "ApplyUserMultiplier".to_string(),
            ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition::new(
                    "User.Level".to_string(),
                    ComparisonOperator::GreaterThan,
                    Value::Integer(1),
                ))),
                operator: rust_rule_engine::types::LogicalOperator::And,
                right: Box::new(ConditionGroup::Single(Condition::new(
                    "Order.BaseScore".to_string(),
                    ComparisonOperator::GreaterThan,
                    Value::Number(0.0),
                ))),
            },
            vec![ActionType::Custom {
                action_type: "multiplyScore".to_string(),
                params: {
                    let mut params = HashMap::new();
                    params.insert("target_field".to_string(), Value::String("Order.AdjustedScore".to_string()));
                    params.insert("multiplier_field".to_string(), Value::String("User.Level".to_string()));
                    params
                },
            }],
        ),
        
        // Step 3: Calculate final discount (reads Order.AdjustedScore, writes Order.DiscountRate)
        Rule::new(
            "CalculateFinalDiscount".to_string(),
            ConditionGroup::Single(Condition::new(
                "Order.AdjustedScore".to_string(),
                ComparisonOperator::GreaterThanOrEqual,
                Value::Number(80.0),
            )),
            vec![ActionType::MethodCall {
                object: "Order".to_string(),
                method: "setDiscountRate".to_string(),
                args: vec![Value::Number(0.15)],
            }],
        ),
        
        // Step 4: Apply final discount (reads Order.Amount, Order.DiscountRate, writes Order.FinalAmount)
        Rule::new(
            "ApplyFinalDiscount".to_string(),
            ConditionGroup::Single(Condition::new(
                "Order.DiscountRate".to_string(),
                ComparisonOperator::GreaterThan,
                Value::Number(0.0),
            )),
            vec![ActionType::Set {
                field: "Order.FinalAmount".to_string(),
                value: Value::Number(0.0), // Would be calculated
            }],
        ),
    ];
    
    println!("🔗 Complex dependency chain:");
    println!("   Order.Amount → calculateOrderScore() → Order.BaseScore");
    println!("   User.Level + Order.BaseScore → Order.AdjustedScore");
    println!("   Order.AdjustedScore → Order.DiscountRate");
    println!("   Order.Amount + Order.DiscountRate → Order.FinalAmount\n");
    
    let result = analyzer.analyze(&rules);
    
    println!("{}", result.get_detailed_report());
    
    if !result.can_parallelize_safely {
        println!("\n🚨 CORRECTLY DETECTED: Rules have dependencies and cannot run in parallel!");
        println!("   Sequential execution required to maintain data flow integrity.");
    } else {
        println!("\n⚠️  MISSED DEPENDENCIES: Analysis may need improvement!");
    }
}

fn demo_function_call_analysis() {
    println!("🛠️  DEMO 3: Function Call Side Effect Analysis");
    println!("   Testing smart detection of function side effects\n");
    
    let mut analyzer = DependencyAnalyzer::new();
    
    let rules = vec![
        // Rule with function that modifies fields
        Rule::new(
            "ProcessUserData".to_string(),
            ConditionGroup::Single(Condition::new(
                "User.Status".to_string(),
                ComparisonOperator::Equal,
                Value::String("active".to_string()),
            )),
            vec![
                ActionType::Call {
                    function: "setUserScore".to_string(),
                    args: vec![Value::Integer(85)],
                },
                ActionType::Call {
                    function: "updateVIPStatus".to_string(),
                    args: vec![Value::Boolean(true)],
                },
                ActionType::Call {
                    function: "calculateOrderTotal".to_string(),
                    args: vec![],
                },
            ],
        ),
        
        // Rule that depends on function side effects
        Rule::new(
            "ApplyVIPBenefits".to_string(),
            ConditionGroup::Compound {
                left: Box::new(ConditionGroup::Single(Condition::new(
                    "User.Score".to_string(),
                    ComparisonOperator::GreaterThan,
                    Value::Integer(80),
                ))),
                operator: rust_rule_engine::types::LogicalOperator::And,
                right: Box::new(ConditionGroup::Single(Condition::new(
                    "VIP.Status".to_string(),
                    ComparisonOperator::Equal,
                    Value::Boolean(true),
                ))),
            },
            vec![ActionType::Set {
                field: "User.Benefits".to_string(),
                value: Value::String("VIP_PREMIUM".to_string()),
            }],
        ),
    ];
    
    println!("🔍 Function call analysis:");
    for rule in &rules {
        println!("   - {}", rule.name);
        for action in &rule.actions {
            match action {
                ActionType::Call { function, .. } => {
                    println!("     → Function: {} (analyzing side effects)", function);
                }
                _ => {}
            }
        }
    }
    
    let result = analyzer.analyze(&rules);
    
    println!("\n{}", result.get_detailed_report());
    
    println!("\n📈 FUNCTION ANALYSIS FEATURES:");
    println!("   ✅ Pattern matching on function names (setXxx, updateXxx, calculateXxx)");
    println!("   ✅ CamelCase to field name conversion");
    println!("   ✅ Side effect prediction for common patterns");
    println!("   ✅ Method call analysis with object modification detection");
}

fn demo_compound_conditions() {
    println!("🌳 DEMO 4: Compound Condition Tree Analysis");
    println!("   Testing recursive field extraction from complex condition trees\n");
    
    let mut analyzer = DependencyAnalyzer::new();
    
    // Create deeply nested compound conditions
    let complex_condition = ConditionGroup::Compound {
        left: Box::new(ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(Condition::new(
                "User.Age".to_string(),
                ComparisonOperator::GreaterThan,
                Value::Integer(21),
            ))),
            operator: rust_rule_engine::types::LogicalOperator::And,
            right: Box::new(ConditionGroup::Single(Condition::new(
                "User.Country".to_string(),
                ComparisonOperator::Equal,
                Value::String("US".to_string()),
            ))),
        }),
        operator: rust_rule_engine::types::LogicalOperator::Or,
        right: Box::new(ConditionGroup::Not(Box::new(ConditionGroup::Single(Condition::new(
            "User.IsBlacklisted".to_string(),
            ComparisonOperator::Equal,
            Value::Boolean(true),
        ))))),
    };
    
    let rules = vec![
        Rule::new(
            "ComplexEligibilityCheck".to_string(),
            complex_condition,
            vec![ActionType::Set {
                field: "User.Eligible".to_string(),
                value: Value::Boolean(true),
            }],
        ),
    ];
    
    println!("🌲 Complex condition tree:");
    println!("   ((User.Age > 21) AND (User.Country == 'US')) OR NOT(User.IsBlacklisted == true)");
    println!("   Expected reads: User.Age, User.Country, User.IsBlacklisted");
    println!("   Expected writes: User.Eligible\n");
    
    let result = analyzer.analyze(&rules);
    
    println!("{}", result.get_detailed_report());
    
    println!("\n🎯 CONDITION TREE ANALYSIS:");
    println!("   ✅ Recursive traversal of compound conditions");
    println!("   ✅ Handles AND, OR, NOT logical operators");
    println!("   ✅ Extracts fields from all nested levels");
    println!("   ✅ No hard-coded field assumptions");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_real_field_detection() {
        let mut analyzer = DependencyAnalyzer::new();
        
        let rule = Rule::new(
            "TestRule".to_string(),
            ConditionGroup::Single(Condition::new(
                "TestField".to_string(),
                ComparisonOperator::Equal,
                Value::String("test".to_string()),
            )),
            vec![ActionType::Set {
                field: "OutputField".to_string(),
                value: Value::String("result".to_string()),
            }],
        );
        
        let reads = analyzer.extract_condition_reads(&rule);
        let writes = analyzer.extract_action_writes(&rule);
        
        assert_eq!(reads, vec!["TestField"]);
        assert_eq!(writes, vec!["OutputField"]);
    }
}

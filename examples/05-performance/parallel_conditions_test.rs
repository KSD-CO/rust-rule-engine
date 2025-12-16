/// Test parallel executor với complex conditions (AND/OR/NOT)
use rust_rule_engine::engine::facts::Facts;
use rust_rule_engine::engine::knowledge_base::KnowledgeBase;
use rust_rule_engine::engine::parallel::{ParallelConfig, ParallelRuleEngine};
use rust_rule_engine::engine::rule::{Condition, ConditionGroup};
use rust_rule_engine::errors::Result;
use rust_rule_engine::types::{ActionType, Operator, Value};

fn main() -> Result<()> {
    println!("🧪 Testing Parallel Executor with Complex Conditions");
    println!("====================================================\n");

    // Test 1: Simple field comparison
    test_simple_condition()?;

    // Test 2: AND condition
    test_and_condition()?;

    // Test 3: OR condition
    test_or_condition()?;

    // Test 4: NOT condition
    test_not_condition()?;

    // Test 5: Nested conditions (AND + OR)
    test_nested_condition()?;

    // Test 6: Expression evaluation
    test_expression_evaluation()?;

    println!("\n✅ All parallel condition tests passed!");
    Ok(())
}

fn test_simple_condition() -> Result<()> {
    println!("📋 Test 1: Simple Field Comparison");

    let kb = KnowledgeBase::new("SimpleTest");
    let rule = rust_rule_engine::engine::rule::Rule::new(
        "AgeCheck".to_string(),
        ConditionGroup::Single(Condition::new(
            "User.age".to_string(),
            Operator::GreaterThan,
            Value::Integer(18),
        )),
        vec![ActionType::Log {
            message: "User is adult".to_string(),
        }],
    );
    kb.add_rule(rule);

    let facts = Facts::new();
    facts.set("User.age", Value::Integer(25));

    let config = ParallelConfig::default();
    let engine = ParallelRuleEngine::new(config);
    let result = engine.execute_parallel(&kb, &facts, false)?;

    assert_eq!(result.total_rules_fired, 1, "Rule should have fired");
    println!("   ✅ Simple condition works: age 25 > 18\n");
    Ok(())
}

fn test_and_condition() -> Result<()> {
    println!("📋 Test 2: AND Condition");

    let kb = KnowledgeBase::new("AndTest");

    // User.age >= 18 AND User.verified == true
    let condition = ConditionGroup::Compound {
        left: Box::new(ConditionGroup::Single(Condition::new(
            "User.age".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Integer(18),
        ))),
        operator: rust_rule_engine::types::LogicalOperator::And,
        right: Box::new(ConditionGroup::Single(Condition::new(
            "User.verified".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ))),
    };

    let rule = rust_rule_engine::engine::rule::Rule::new(
        "VerifiedAdultCheck".to_string(),
        condition,
        vec![ActionType::Log {
            message: "User is verified adult".to_string(),
        }],
    );
    kb.add_rule(rule);

    // Test case 1: Both conditions true
    let facts1 = Facts::new();
    facts1.set("User.age", Value::Integer(25));
    facts1.set("User.verified", Value::Boolean(true));

    let config = ParallelConfig::default();
    let engine = ParallelRuleEngine::new(config);
    let result1 = engine.execute_parallel(&kb, &facts1, false)?;
    assert_eq!(result1.total_rules_fired, 1, "AND: Both true should fire");

    // Test case 2: One condition false
    let facts2 = Facts::new();
    facts2.set("User.age", Value::Integer(25));
    facts2.set("User.verified", Value::Boolean(false));
    let result2 = engine.execute_parallel(&kb, &facts2, false)?;
    assert_eq!(
        result2.total_rules_fired, 0,
        "AND: One false should not fire"
    );

    println!("   ✅ AND condition works correctly\n");
    Ok(())
}

fn test_or_condition() -> Result<()> {
    println!("📋 Test 3: OR Condition");

    let kb = KnowledgeBase::new("OrTest");

    // User.isAdmin == true OR User.isModerator == true
    let condition = ConditionGroup::Compound {
        left: Box::new(ConditionGroup::Single(Condition::new(
            "User.isAdmin".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ))),
        operator: rust_rule_engine::types::LogicalOperator::Or,
        right: Box::new(ConditionGroup::Single(Condition::new(
            "User.isModerator".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ))),
    };

    let rule = rust_rule_engine::engine::rule::Rule::new(
        "StaffCheck".to_string(),
        condition,
        vec![ActionType::Log {
            message: "User is staff".to_string(),
        }],
    );
    kb.add_rule(rule);

    let config = ParallelConfig::default();
    let engine = ParallelRuleEngine::new(config);

    // Test case 1: First condition true
    let facts1 = Facts::new();
    facts1.set("User.isAdmin", Value::Boolean(true));
    facts1.set("User.isModerator", Value::Boolean(false));
    let result1 = engine.execute_parallel(&kb, &facts1, false)?;
    assert_eq!(result1.total_rules_fired, 1, "OR: First true should fire");

    // Test case 2: Second condition true
    let facts2 = Facts::new();
    facts2.set("User.isAdmin", Value::Boolean(false));
    facts2.set("User.isModerator", Value::Boolean(true));
    let result2 = engine.execute_parallel(&kb, &facts2, false)?;
    assert_eq!(result2.total_rules_fired, 1, "OR: Second true should fire");

    // Test case 3: Both false
    let facts3 = Facts::new();
    facts3.set("User.isAdmin", Value::Boolean(false));
    facts3.set("User.isModerator", Value::Boolean(false));
    let result3 = engine.execute_parallel(&kb, &facts3, false)?;
    assert_eq!(
        result3.total_rules_fired, 0,
        "OR: Both false should not fire"
    );

    println!("   ✅ OR condition works correctly\n");
    Ok(())
}

fn test_not_condition() -> Result<()> {
    println!("📋 Test 4: NOT Condition");

    let kb = KnowledgeBase::new("NotTest");

    // NOT (User.banned == true)
    let condition = ConditionGroup::Not(Box::new(ConditionGroup::Single(Condition::new(
        "User.banned".to_string(),
        Operator::Equal,
        Value::Boolean(true),
    ))));

    let rule = rust_rule_engine::engine::rule::Rule::new(
        "AllowAccessIfNotBanned".to_string(),
        condition,
        vec![ActionType::Log {
            message: "User can access".to_string(),
        }],
    );
    kb.add_rule(rule);

    let config = ParallelConfig::default();
    let engine = ParallelRuleEngine::new(config);

    // Test case 1: User not banned
    let facts1 = Facts::new();
    facts1.set("User.banned", Value::Boolean(false));
    let result1 = engine.execute_parallel(&kb, &facts1, false)?;
    assert_eq!(result1.total_rules_fired, 1, "NOT: false should fire");

    // Test case 2: User banned
    let facts2 = Facts::new();
    facts2.set("User.banned", Value::Boolean(true));
    let result2 = engine.execute_parallel(&kb, &facts2, false)?;
    assert_eq!(result2.total_rules_fired, 0, "NOT: true should not fire");

    println!("   ✅ NOT condition works correctly\n");
    Ok(())
}

fn test_nested_condition() -> Result<()> {
    println!("📋 Test 5: Nested Conditions (AND + OR)");

    let kb = KnowledgeBase::new("NestedTest");

    // (User.age >= 18 AND User.verified == true) OR User.isAdmin == true
    let left_and = ConditionGroup::Compound {
        left: Box::new(ConditionGroup::Single(Condition::new(
            "User.age".to_string(),
            Operator::GreaterThanOrEqual,
            Value::Integer(18),
        ))),
        operator: rust_rule_engine::types::LogicalOperator::And,
        right: Box::new(ConditionGroup::Single(Condition::new(
            "User.verified".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ))),
    };

    let condition = ConditionGroup::Compound {
        left: Box::new(left_and),
        operator: rust_rule_engine::types::LogicalOperator::Or,
        right: Box::new(ConditionGroup::Single(Condition::new(
            "User.isAdmin".to_string(),
            Operator::Equal,
            Value::Boolean(true),
        ))),
    };

    let rule = rust_rule_engine::engine::rule::Rule::new(
        "ComplexAccessCheck".to_string(),
        condition,
        vec![ActionType::Log {
            message: "User can access".to_string(),
        }],
    );
    kb.add_rule(rule);

    let config = ParallelConfig::default();
    let engine = ParallelRuleEngine::new(config);

    // Test case 1: Admin bypass (age < 18, not verified, but admin)
    let facts1 = Facts::new();
    facts1.set("User.age", Value::Integer(15));
    facts1.set("User.verified", Value::Boolean(false));
    facts1.set("User.isAdmin", Value::Boolean(true));
    let result1 = engine.execute_parallel(&kb, &facts1, false)?;
    assert_eq!(result1.total_rules_fired, 1, "Nested: Admin should bypass");

    // Test case 2: Verified adult (age >= 18, verified, not admin)
    let facts2 = Facts::new();
    facts2.set("User.age", Value::Integer(25));
    facts2.set("User.verified", Value::Boolean(true));
    facts2.set("User.isAdmin", Value::Boolean(false));
    let result2 = engine.execute_parallel(&kb, &facts2, false)?;
    assert_eq!(
        result2.total_rules_fired, 1,
        "Nested: Verified adult should pass"
    );

    // Test case 3: Fail all conditions
    let facts3 = Facts::new();
    facts3.set("User.age", Value::Integer(15));
    facts3.set("User.verified", Value::Boolean(false));
    facts3.set("User.isAdmin", Value::Boolean(false));
    let result3 = engine.execute_parallel(&kb, &facts3, false)?;
    assert_eq!(
        result3.total_rules_fired, 0,
        "Nested: All false should not fire"
    );

    println!("   ✅ Nested conditions work correctly\n");
    Ok(())
}

fn test_expression_evaluation() -> Result<()> {
    println!("📋 Test 6: Expression Evaluation");

    let kb = KnowledgeBase::new("ExpressionTest");

    // Compare two fact values: Order.amount > Order.limit
    let condition = ConditionGroup::Single(Condition::new(
        "Order.amount".to_string(),
        Operator::GreaterThan,
        Value::String("Order.limit".to_string()), // Reference to another fact
    ));

    let rule = rust_rule_engine::engine::rule::Rule::new(
        "CheckOrderLimit".to_string(),
        condition,
        vec![ActionType::Log {
            message: "Order exceeds limit".to_string(),
        }],
    );
    kb.add_rule(rule);

    let config = ParallelConfig::default();
    let engine = ParallelRuleEngine::new(config);

    // Test case 1: Amount exceeds limit
    let facts1 = Facts::new();
    facts1.set("Order.amount", Value::Number(1500.0));
    facts1.set("Order.limit", Value::Number(1000.0));
    let result1 = engine.execute_parallel(&kb, &facts1, false)?;
    assert_eq!(
        result1.total_rules_fired, 1,
        "Expression: 1500 > 1000 should fire"
    );

    // Test case 2: Amount within limit
    let facts2 = Facts::new();
    facts2.set("Order.amount", Value::Number(500.0));
    facts2.set("Order.limit", Value::Number(1000.0));
    let result2 = engine.execute_parallel(&kb, &facts2, false)?;
    assert_eq!(
        result2.total_rules_fired, 0,
        "Expression: 500 > 1000 should not fire"
    );

    println!("   ✅ Expression evaluation works correctly\n");
    Ok(())
}

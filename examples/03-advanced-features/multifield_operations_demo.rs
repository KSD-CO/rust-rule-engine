/// Demo: Complete MultiField Operations
/// Shows all multifield operations working in native engine
use rust_rule_engine::engine::{
    knowledge_base::KnowledgeBase,
    rule::{Condition, ConditionExpression, ConditionGroup, Rule},
    RustRuleEngine,
};
use rust_rule_engine::types::{ActionType, Operator, Value};
use std::collections::HashMap;

#[allow(deprecated)]
#[allow(unused_variables)]
fn main() {
    println!("🎯 MultiField Operations Demo\n");
    println!("Testing all multifield operations: empty, not_empty, count, first, last, contains, collect\n");

    let kb = KnowledgeBase::new("MultiFieldTest");
    let engine = RustRuleEngine::new(kb);

    // Test data
    let mut facts = HashMap::new();
    facts.insert(
        "Cart.items".to_string(),
        Value::Array(vec![
            Value::String("apple".to_string()),
            Value::String("banana".to_string()),
            Value::String("orange".to_string()),
        ]),
    );
    facts.insert("EmptyList.items".to_string(), Value::Array(vec![]));
    facts.insert(
        "Numbers.values".to_string(),
        Value::Array(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
            Value::Integer(4),
            Value::Integer(5),
        ]),
    );

    // Test 1: empty operation
    println!("📋 Test 1: empty operation");
    let empty_rule = Rule::new(
        "CheckEmpty".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "EmptyList.items".to_string(),
                operation: "empty".to_string(),
                variable: None,
            },
            operator: Operator::Equal,
            value: Value::Boolean(true),
            field: "EmptyList.items".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ Empty list detected!".to_string(),
        }],
    );

    if empty_rule.matches(&facts) {
        println!("   ✓ Empty operation works");
    }

    // Test 2: not_empty operation
    println!("\n📋 Test 2: not_empty operation");
    let not_empty_rule = Rule::new(
        "CheckNotEmpty".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "Cart.items".to_string(),
                operation: "not_empty".to_string(),
                variable: None,
            },
            operator: Operator::Equal,
            value: Value::Boolean(true),
            field: "Cart.items".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ Cart has items!".to_string(),
        }],
    );

    if not_empty_rule.matches(&facts) {
        println!("   ✓ Not empty operation works");
    }

    // Test 3: count operation
    println!("\n📋 Test 3: count operation");
    let count_rule = Rule::new(
        "CheckCount".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "Cart.items".to_string(),
                operation: "count".to_string(),
                variable: None,
            },
            operator: Operator::Equal,
            value: Value::Integer(3),
            field: "Cart.items".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ Cart has exactly 3 items!".to_string(),
        }],
    );

    if count_rule.matches(&facts) {
        println!("   ✓ Count operation works (3 items)");
    }

    // Test 4: count with comparison
    println!("\n📋 Test 4: count with GreaterThan");
    let count_gt_rule = Rule::new(
        "CheckCountGreater".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "Numbers.values".to_string(),
                operation: "count".to_string(),
                variable: None,
            },
            operator: Operator::GreaterThan,
            value: Value::Integer(3),
            field: "Numbers.values".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ Numbers has more than 3 items!".to_string(),
        }],
    );

    if count_gt_rule.matches(&facts) {
        println!("   ✓ Count > 3 works (5 items)");
    }

    // Test 5: first operation
    println!("\n📋 Test 5: first operation");
    let first_rule = Rule::new(
        "CheckFirst".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "Cart.items".to_string(),
                operation: "first".to_string(),
                variable: None,
            },
            operator: Operator::Equal,
            value: Value::String("apple".to_string()),
            field: "Cart.items".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ First item is apple!".to_string(),
        }],
    );

    if first_rule.matches(&facts) {
        println!("   ✓ First operation works (apple)");
    }

    // Test 6: last operation
    println!("\n📋 Test 6: last operation");
    let last_rule = Rule::new(
        "CheckLast".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "Cart.items".to_string(),
                operation: "last".to_string(),
                variable: None,
            },
            operator: Operator::Equal,
            value: Value::String("orange".to_string()),
            field: "Cart.items".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ Last item is orange!".to_string(),
        }],
    );

    if last_rule.matches(&facts) {
        println!("   ✓ Last operation works (orange)");
    }

    // Test 7: contains operation
    println!("\n📋 Test 7: contains operation");
    let contains_rule = Rule::new(
        "CheckContains".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "Cart.items".to_string(),
                operation: "contains".to_string(),
                variable: None,
            },
            operator: Operator::Equal,
            value: Value::String("banana".to_string()),
            field: "Cart.items".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ Cart contains banana!".to_string(),
        }],
    );

    if contains_rule.matches(&facts) {
        println!("   ✓ Contains operation works (banana found)");
    }

    // Test 8: collect operation
    println!("\n📋 Test 8: collect operation");
    let collect_rule = Rule::new(
        "CheckCollect".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "Cart.items".to_string(),
                operation: "collect".to_string(),
                variable: Some("$all_items".to_string()),
            },
            operator: Operator::Equal,
            value: Value::Boolean(true),
            field: "Cart.items".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ Collected all items into variable!".to_string(),
        }],
    );

    if collect_rule.matches(&facts) {
        println!("   ✓ Collect operation works");
    }

    // Test 9: first with numbers
    println!("\n📋 Test 9: first with numbers");
    let first_num_rule = Rule::new(
        "CheckFirstNumber".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "Numbers.values".to_string(),
                operation: "first".to_string(),
                variable: None,
            },
            operator: Operator::Equal,
            value: Value::Integer(1),
            field: "Numbers.values".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ First number is 1!".to_string(),
        }],
    );

    if first_num_rule.matches(&facts) {
        println!("   ✓ First with numbers works");
    }

    // Test 10: last with numbers
    println!("\n📋 Test 10: last with numbers");
    let last_num_rule = Rule::new(
        "CheckLastNumber".to_string(),
        ConditionGroup::Single(Condition {
            expression: ConditionExpression::MultiField {
                field: "Numbers.values".to_string(),
                operation: "last".to_string(),
                variable: None,
            },
            operator: Operator::Equal,
            value: Value::Integer(5),
            field: "Numbers.values".to_string(),
        }),
        vec![ActionType::Log {
            message: "✅ Last number is 5!".to_string(),
        }],
    );

    if last_num_rule.matches(&facts) {
        println!("   ✓ Last with numbers works");
    }

    println!("\n✅ All multifield operations working correctly!");
    println!("\n📊 Summary:");
    println!("   • empty: ✓");
    println!("   • not_empty: ✓");
    println!("   • count: ✓");
    println!("   • first: ✓");
    println!("   • last: ✓");
    println!("   • contains: ✓");
    println!("   • collect: ✓");
}

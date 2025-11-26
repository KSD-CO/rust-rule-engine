#![cfg(feature = "backward-chaining")]

use rust_rule_engine::backward::BackwardEngine;
use rust_rule_engine::engine::rule::{Condition, ConditionGroup, Rule};
use rust_rule_engine::types::{ActionType, Operator, Value};
use rust_rule_engine::{Facts, KnowledgeBase};
use std::sync::{Arc, Mutex};
use std::thread;

/// Test to verify current thread safety behavior
///
/// This test demonstrates that BackwardEngine requires external synchronization
/// for concurrent queries. The engine uses &mut self for queries, which prevents
/// concurrent access without wrapping in Arc<Mutex<>>.
#[test]
fn test_concurrent_queries_with_mutex() {
    // Setup knowledge base with simple rule
    let kb = KnowledgeBase::new("concurrent_kb");

    let condition = Condition::new(
        "User.Points".to_string(),
        Operator::GreaterThan,
        Value::Number(100.0),
    );

    let rule = Rule::new(
        "VIPRule".to_string(),
        ConditionGroup::Single(condition),
        vec![ActionType::Set {
            field: "User.IsVIP".to_string(),
            value: Value::Boolean(true),
        }],
    );

    let _ = kb.add_rule(rule);

    // Wrap engine in Arc<Mutex<>> for thread safety
    let engine = Arc::new(Mutex::new(BackwardEngine::new(kb)));

    let mut handles = vec![];

    // Spawn 10 threads to query concurrently
    for i in 0..10 {
        let engine_clone = Arc::clone(&engine);

        let handle = thread::spawn(move || {
            let mut facts = Facts::new();
            facts.set("User.Points", Value::Number(150.0 + i as f64));

            // Lock engine for this query
            let mut engine_guard = engine_clone.lock().unwrap();
            let result = engine_guard.query("User.IsVIP == true", &mut facts);

            assert!(result.is_ok());
            assert!(result.unwrap().provable);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test demonstrating that Facts need to be thread-local
///
/// Each thread must have its own Facts instance. Sharing Facts
/// across threads would require Arc<Mutex<Facts>>.
#[test]
fn test_thread_local_facts() {
    let kb = KnowledgeBase::new("test_kb");

    let condition = Condition::new(
        "Order.Total".to_string(),
        Operator::GreaterThanOrEqual,
        Value::Number(100.0),
    );

    let rule = Rule::new(
        "DiscountRule".to_string(),
        ConditionGroup::Single(condition),
        vec![ActionType::Set {
            field: "Order.Discount".to_string(),
            value: Value::Number(0.1),
        }],
    );

    let _ = kb.add_rule(rule);

    let engine = Arc::new(Mutex::new(BackwardEngine::new(kb)));

    let mut handles = vec![];

    // Each thread creates its own Facts
    for i in 0..5 {
        let engine_clone = Arc::clone(&engine);

        let handle = thread::spawn(move || {
            // Thread-local Facts
            let mut facts = Facts::new();
            facts.set("Order.Total", Value::Number(100.0 + (i * 50) as f64));
            facts.set("Order.ID", Value::Integer(i));

            let mut engine_guard = engine_clone.lock().unwrap();
            let result = engine_guard.query("Order.Discount == 0.1", &mut facts);

            assert!(result.is_ok());
            let qr = result.unwrap();
            assert!(qr.provable, "Order {} should get discount", i);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test to verify memoization cache behavior in concurrent scenarios
///
/// Memoization cache is stored in goal_manager which is accessed with &mut self.
/// This means cache updates are serialized by the Mutex lock.
#[test]
fn test_memoization_with_concurrent_queries() {
    let kb = KnowledgeBase::new("memo_kb");

    let condition = Condition::new(
        "User.Age".to_string(),
        Operator::GreaterThanOrEqual,
        Value::Integer(18),
    );

    let rule = Rule::new(
        "AdultRule".to_string(),
        ConditionGroup::Single(condition),
        vec![ActionType::Set {
            field: "User.IsAdult".to_string(),
            value: Value::Boolean(true),
        }],
    );

    let _ = kb.add_rule(rule);

    let engine = Arc::new(Mutex::new(BackwardEngine::new(kb)));

    let mut handles = vec![];

    // Multiple threads querying the same goal (should hit memoization)
    for i in 0..10 {
        let engine_clone = Arc::clone(&engine);

        let handle = thread::spawn(move || {
            let mut facts = Facts::new();
            facts.set("User.Age", Value::Integer(25));
            facts.set("User.ID", Value::Integer(i));

            let mut engine_guard = engine_clone.lock().unwrap();
            let result = engine_guard.query("User.IsAdult == true", &mut facts);

            assert!(result.is_ok());
            assert!(result.unwrap().provable);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test to verify that different queries don't interfere with each other
#[test]
fn test_different_queries_concurrent() {
    let kb = KnowledgeBase::new("multi_query_kb");

    // Rule 1: VIP qualification
    let vip_condition = Condition::new(
        "User.Points".to_string(),
        Operator::GreaterThan,
        Value::Number(1000.0),
    );
    let vip_rule = Rule::new(
        "VIPRule".to_string(),
        ConditionGroup::Single(vip_condition),
        vec![ActionType::Set {
            field: "User.IsVIP".to_string(),
            value: Value::Boolean(true),
        }],
    );

    // Rule 2: Premium qualification
    let premium_condition = Condition::new(
        "User.Subscription".to_string(),
        Operator::Equal,
        Value::String("premium".to_string()),
    );
    let premium_rule = Rule::new(
        "PremiumRule".to_string(),
        ConditionGroup::Single(premium_condition),
        vec![ActionType::Set {
            field: "User.IsPremium".to_string(),
            value: Value::Boolean(true),
        }],
    );

    let _ = kb.add_rule(vip_rule);
    let _ = kb.add_rule(premium_rule);

    let engine = Arc::new(Mutex::new(BackwardEngine::new(kb)));

    let mut handles = vec![];

    // Half threads query for VIP
    for i in 0..5 {
        let engine_clone = Arc::clone(&engine);

        let handle = thread::spawn(move || {
            let mut facts = Facts::new();
            facts.set("User.Points", Value::Number(1500.0));
            facts.set("User.ID", Value::Integer(i));

            let mut engine_guard = engine_clone.lock().unwrap();
            let result = engine_guard.query("User.IsVIP == true", &mut facts);

            assert!(result.is_ok());
            assert!(result.unwrap().provable);
        });

        handles.push(handle);
    }

    // Half threads query for Premium
    for i in 5..10 {
        let engine_clone = Arc::clone(&engine);

        let handle = thread::spawn(move || {
            let mut facts = Facts::new();
            facts.set("User.Subscription", Value::String("premium".to_string()));
            facts.set("User.ID", Value::Integer(i));

            let mut engine_guard = engine_clone.lock().unwrap();
            let result = engine_guard.query("User.IsPremium == true", &mut facts);

            assert!(result.is_ok());
            assert!(result.unwrap().provable);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Stress test with many concurrent queries
#[test]
fn test_stress_concurrent_queries() {
    let kb = KnowledgeBase::new("stress_kb");

    // Create a chain of rules
    for i in 0..5 {
        let condition = Condition::new(
            format!("Level{}.Complete", i),
            Operator::Equal,
            Value::Boolean(true),
        );

        let rule = Rule::new(
            format!("Level{}Rule", i + 1),
            ConditionGroup::Single(condition),
            vec![ActionType::Set {
                field: format!("Level{}.Complete", i + 1),
                value: Value::Boolean(true),
            }],
        );

        let _ = kb.add_rule(rule);
    }

    let engine = Arc::new(Mutex::new(BackwardEngine::new(kb)));

    let mut handles = vec![];

    // Spawn 50 threads
    for i in 0..50 {
        let engine_clone = Arc::clone(&engine);

        let handle = thread::spawn(move || {
            let mut facts = Facts::new();
            facts.set("Level0.Complete", Value::Boolean(true));
            facts.set("Thread.ID", Value::Integer(i));

            let mut engine_guard = engine_clone.lock().unwrap();
            let result = engine_guard.query("Level5.Complete == true", &mut facts);

            assert!(result.is_ok());
            assert!(result.unwrap().provable);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

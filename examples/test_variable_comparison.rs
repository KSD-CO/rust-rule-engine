/// Quick test to verify variable-to-variable comparison in RETE alpha nodes
use rust_rule_engine::rete::{AlphaNode, TypedFacts};

fn main() {
    println!("🧪 Testing Variable-to-Variable Comparison in AlphaNode");
    println!("========================================================\n");

    // Test 1: Compare with literal value
    println!("Test 1: L1 > 50 (literal comparison)");
    let mut facts1 = TypedFacts::new();
    facts1.set("L1", 100i64);

    let node1 = AlphaNode {
        field: "L1".to_string(),
        operator: ">".to_string(),
        value: "50".to_string(),
    };

    let result1 = node1.matches_typed(&facts1);
    println!("   L1=100, comparing with literal 50");
    println!("   Result: {} (expected: true)", result1);
    assert!(result1, "Should match when L1 > 50");

    // Test 2: Compare with variable (variable-to-variable)
    println!("\nTest 2: L1 > L1Min (variable comparison)");
    let mut facts2 = TypedFacts::new();
    facts2.set("L1", 100i64);
    facts2.set("L1Min", 50i64);

    let node2 = AlphaNode {
        field: "L1".to_string(),
        operator: ">".to_string(),
        value: "L1Min".to_string(),  // This should resolve to the value of L1Min
    };

    let result2 = node2.matches_typed(&facts2);
    println!("   L1=100, L1Min=50, comparing L1 > L1Min");
    println!("   Result: {} (expected: true)", result2);
    assert!(result2, "Should match when L1 > L1Min (100 > 50)");

    // Test 3: Variable comparison false case
    println!("\nTest 3: L1 > L1Min where L1 < L1Min");
    let mut facts3 = TypedFacts::new();
    facts3.set("L1", 40i64);
    facts3.set("L1Min", 50i64);

    let node3 = AlphaNode {
        field: "L1".to_string(),
        operator: ">".to_string(),
        value: "L1Min".to_string(),
    };

    let result3 = node3.matches_typed(&facts3);
    println!("   L1=40, L1Min=50, comparing L1 > L1Min");
    println!("   Result: {} (expected: false)", result3);
    assert!(!result3, "Should not match when L1 < L1Min (40 < 50)");

    // Test 4: Equal values
    println!("\nTest 4: L1 > L1Min where L1 == L1Min");
    let mut facts4 = TypedFacts::new();
    facts4.set("L1", 50i64);
    facts4.set("L1Min", 50i64);

    let node4 = AlphaNode {
        field: "L1".to_string(),
        operator: ">".to_string(),
        value: "L1Min".to_string(),
    };

    let result4 = node4.matches_typed(&facts4);
    println!("   L1=50, L1Min=50, comparing L1 > L1Min");
    println!("   Result: {} (expected: false)", result4);
    assert!(!result4, "Should not match when L1 == L1Min (50 == 50)");

    // Test 5: Ensure literal still works when variable doesn't exist
    println!("\nTest 5: Fallback to literal when variable doesn't exist");
    let mut facts5 = TypedFacts::new();
    facts5.set("L1", 100i64);
    // No L1Min fact

    let node5 = AlphaNode {
        field: "L1".to_string(),
        operator: ">".to_string(),
        value: "50".to_string(),  // Should be parsed as literal 50
    };

    let result5 = node5.matches_typed(&facts5);
    println!("   L1=100, no L1Min fact, comparing with '50' as literal");
    println!("   Result: {} (expected: true)", result5);
    assert!(result5, "Should match when parsing '50' as literal");

    println!("\n✅ All tests passed!");
    println!("\n📋 Summary:");
    println!("   ✅ Literal comparison works");
    println!("   ✅ Variable-to-variable comparison works");
    println!("   ✅ False cases handled correctly");
    println!("   ✅ Equal values handled correctly (not greater)");
    println!("   ✅ Fallback to literal when variable doesn't exist");
}

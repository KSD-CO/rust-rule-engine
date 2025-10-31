/// Demo: Working Memory with FactHandles (P2 Feature - Drools-style)
///
/// This example demonstrates the Working Memory system similar to Drools:
/// - Insert, update, retract operations with FactHandles
/// - Type indexing for fast lookups
/// - Change tracking for incremental updates
/// - Fact metadata and statistics

use rust_rule_engine::rete::working_memory::{WorkingMemory, FactHandle};
use rust_rule_engine::rete::facts::TypedFacts;

fn main() {
    println!("\n📦 Working Memory Demo (Drools-style)");
    println!("======================================\n");

    let mut wm = WorkingMemory::new();

    // Example 1: Insert facts
    println!("📋 Example 1: Inserting Facts");
    println!("-----------------------------");

    let mut person1 = TypedFacts::new();
    person1.set("name", "John Smith");
    person1.set("age", 25i64);
    person1.set("salary", 50000.0);

    let mut person2 = TypedFacts::new();
    person2.set("name", "Jane Doe");
    person2.set("age", 30i64);
    person2.set("salary", 75000.0);

    let handle1 = wm.insert("Person".to_string(), person1);
    let handle2 = wm.insert("Person".to_string(), person2);

    println!("Inserted Person 1: {}", handle1);
    println!("Inserted Person 2: {}", handle2);

    let mut order1 = TypedFacts::new();
    order1.set("id", 1001i64);
    order1.set("customer", "John Smith");
    order1.set("amount", 250.0);

    let handle3 = wm.insert("Order".to_string(), order1);
    println!("Inserted Order: {}", handle3);

    println!("\n📊 Statistics: {}", wm.stats());

    // Example 2: Query by type
    println!("\n📋 Example 2: Query by Type");
    println!("---------------------------");

    let persons = wm.get_by_type("Person");
    println!("Found {} persons:", persons.len());
    for person in persons {
        let name = person.data.get("name").unwrap().as_string();
        let age = person.data.get("age").unwrap().as_integer().unwrap();
        println!("  - {} ({}), age {}", person.handle, name, age);
    }

    let orders = wm.get_by_type("Order");
    println!("\nFound {} orders:", orders.len());
    for order in orders {
        let id = order.data.get("id").unwrap().as_integer().unwrap();
        let customer = order.data.get("customer").unwrap().as_string();
        println!("  - {} (Order #{}), customer: {}", order.handle, id, customer);
    }

    // Example 3: Update a fact
    println!("\n📋 Example 3: Updating Facts");
    println!("---------------------------");

    let fact1 = wm.get(&handle1).unwrap();
    println!("Before update:");
    println!("  {} - age: {}", handle1, fact1.data.get("age").unwrap().as_integer().unwrap());
    println!("  Update count: {}", fact1.metadata.update_count);

    let mut updated_person = TypedFacts::new();
    updated_person.set("name", "John Smith");
    updated_person.set("age", 26i64); // Birthday!
    updated_person.set("salary", 55000.0); // Raise!

    wm.update(handle1, updated_person).unwrap();

    let fact1_after = wm.get(&handle1).unwrap();
    println!("\nAfter update:");
    println!("  {} - age: {}", handle1, fact1_after.data.get("age").unwrap().as_integer().unwrap());
    println!("  Update count: {}", fact1_after.metadata.update_count);

    // Example 4: Change tracking
    println!("\n📋 Example 4: Change Tracking");
    println!("----------------------------");

    println!("Modified handles: {:?}", wm.get_modified_handles());
    println!("Retracted handles: {:?}", wm.get_retracted_handles());

    wm.clear_modification_tracking();
    println!("\nAfter clearing tracking:");
    println!("Modified handles: {:?}", wm.get_modified_handles());

    // Example 5: Retract a fact
    println!("\n📋 Example 5: Retracting Facts");
    println!("------------------------------");

    println!("Before retract: {} persons", wm.get_by_type("Person").len());

    wm.retract(handle2).unwrap();
    println!("Retracted: {}", handle2);

    println!("After retract: {} persons", wm.get_by_type("Person").len());
    println!("\n📊 Statistics: {}", wm.stats());

    // Verify fact is no longer accessible
    assert!(wm.get(&handle2).is_none(), "Retracted fact should not be accessible");
    println!("✅ Retracted fact is no longer accessible");

    // Example 6: Metadata
    println!("\n📋 Example 6: Fact Metadata");
    println!("--------------------------");

    let fact = wm.get(&handle1).unwrap();
    println!("Fact: {}", fact.handle);
    println!("  Type: {}", fact.fact_type);
    println!("  Inserted: {:?} ago", fact.metadata.inserted_at.elapsed());
    println!("  Updated: {:?} ago", fact.metadata.updated_at.elapsed());
    println!("  Update count: {}", fact.metadata.update_count);
    println!("  Retracted: {}", fact.metadata.retracted);

    // Example 7: Get all facts
    println!("\n📋 Example 7: Get All Facts");
    println!("---------------------------");

    let all_facts = wm.get_all_facts();
    println!("Total active facts: {}", all_facts.len());
    for fact in all_facts {
        println!("  - {} ({})", fact.handle, fact.fact_type);
    }

    // Example 8: Performance with many facts
    println!("\n📋 Example 8: Performance Test");
    println!("------------------------------");

    let mut wm_perf = WorkingMemory::new();
    let count = 1000;

    let start = std::time::Instant::now();
    let mut handles = Vec::new();
    for i in 0..count {
        let mut data = TypedFacts::new();
        data.set("id", i as i64);
        data.set("name", format!("Person{}", i));
        data.set("age", 20 + (i % 50) as i64);
        let h = wm_perf.insert("Person".to_string(), data);
        handles.push(h);
    }
    let insert_time = start.elapsed();

    println!("Inserted {} facts in {:?}", count, insert_time);
    println!("Avg per insert: {:?}", insert_time / count);

    // Query performance
    let start = std::time::Instant::now();
    let persons_perf = wm_perf.get_by_type("Person");
    let query_time = start.elapsed();

    println!("Queried {} facts in {:?}", persons_perf.len(), query_time);

    // Update performance
    let start = std::time::Instant::now();
    for handle in &handles[..100] {
        let mut data = TypedFacts::new();
        data.set("age", 99i64);
        wm_perf.update(*handle, data).unwrap();
    }
    let update_time = start.elapsed();

    println!("Updated 100 facts in {:?}", update_time);
    println!("Avg per update: {:?}", update_time / 100);

    println!("\n📊 Final Statistics: {}", wm_perf.stats());

    // Summary
    println!("\n✨ Working Memory Features");
    println!("=========================");
    println!("✅ FactHandle system for tracking objects");
    println!("✅ Insert, update, retract operations");
    println!("✅ Type indexing for fast lookups");
    println!("✅ Change tracking (modified/retracted)");
    println!("✅ Fact metadata (timestamps, update count)");
    println!("✅ Statistics and monitoring");
    println!("✅ Performance: 1000 inserts in ~{}µs", insert_time.as_micros());
    println!("\n🚀 Similar to Drools KieSession.insert/update/delete!");
}

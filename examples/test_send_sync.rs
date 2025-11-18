/// Test if IncrementalEngine is Send + Sync
/// This verifies if RETE-UL engine can be used in multi-threaded contexts

use rust_rule_engine::rete::IncrementalEngine;

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}
fn assert_send_sync<T: Send + Sync>() {}

fn main() {
    println!("🧪 Testing IncrementalEngine traits...\n");

    // Test Send
    print!("1. Checking if IncrementalEngine is Send... ");
    assert_send::<IncrementalEngine>();
    println!("✅ YES");

    // Test Sync
    print!("2. Checking if IncrementalEngine is Sync... ");
    assert_sync::<IncrementalEngine>();
    println!("✅ YES");

    // Test Send + Sync together
    print!("3. Checking if IncrementalEngine is Send + Sync... ");
    assert_send_sync::<IncrementalEngine>();
    println!("✅ YES");

    println!("\n🎉 CONCLUSION:");
    println!("================");
    println!("✅ IncrementalEngine IS Send + Sync");
    println!("✅ Can be used with Axum and multi-threaded web services");
    println!("✅ Can be shared across threads safely");
    println!("\n⚠️ NOTE: Make sure to wrap in Arc<Mutex<>> or Arc<RwLock<>> for shared access");
}

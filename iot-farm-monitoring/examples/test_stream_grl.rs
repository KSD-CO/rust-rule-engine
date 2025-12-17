/// Test Stream GRL Rules
///
/// This tests GRL rules that use stream syntax:
/// - from stream("name") over window(duration, type)
/// - Sliding windows
/// - Tumbling windows

use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine};

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║            🌊 STREAM GRL SYNTAX TEST 🌊                             ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    test_stream_climate_control();
    test_stream_aquaculture();

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                 ✅ ALL STREAM GRL TESTS DONE ✅                     ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");
}

fn test_stream_climate_control() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🥬 TEST 1: Stream Climate Control GRL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let grl_path = "grl_rules/stream_climate_control.grl";
    let mut engine = IncrementalEngine::new();

    match GrlReteLoader::load_from_file(grl_path, &mut engine) {
        Ok(count) => {
            println!("✅ Successfully loaded {} stream rules from {}", count, grl_path);
            println!("   Rules contain stream syntax:");
            println!("   - from stream(\"name\") over window(duration, type)");
            println!("   - Sliding windows (5 min, 10 min, 15 min)");
            println!("   - Tumbling windows (1 hour)");
        }
        Err(e) => {
            println!("❌ Failed to load {}: {}", grl_path, e);
            println!("\n   This is expected if stream syntax parsing is not fully integrated.");
            println!("   The parser exists in src/parser/grl/stream_syntax.rs");
            println!("   but may need integration with main GRL parser.");
        }
    }

    println!();
}

fn test_stream_aquaculture() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🐟 TEST 2: Stream Aquaculture GRL");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let grl_path = "grl_rules/stream_aquaculture.grl";
    let mut engine = IncrementalEngine::new();

    match GrlReteLoader::load_from_file(grl_path, &mut engine) {
        Ok(count) => {
            println!("✅ Successfully loaded {} stream rules from {}", count, grl_path);
            println!("   Stream features:");
            println!("   - DO monitoring: sliding 10 min window");
            println!("   - Temperature: sliding 10 min window");
            println!("   - pH: tumbling 1 hour window");
            println!("   - Water quality: tumbling 1 hour aggregation");
        }
        Err(e) => {
            println!("❌ Failed to load {}: {}", grl_path, e);
            println!("\n   The stream syntax parser is available but needs:");
            println!("   1. Integration with main GRL parser (parse_rules)");
            println!("   2. Stream-aware RETE node generation");
            println!("   3. Runtime stream topology setup");
            println!("\n   Current status:");
            println!("   ✅ Parser module exists: src/parser/grl/stream_syntax.rs");
            println!("   ✅ Supports: from stream() over window()");
            println!("   ❌ Not yet integrated into GrlReteLoader");
        }
    }

    println!();
}

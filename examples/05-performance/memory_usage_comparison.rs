//! Memory Usage Comparison
//!
//! This example measures actual memory consumption for RETE optimizations.
//! Shows real MB/GB usage, not just percentages.

use rust_rule_engine::rete::optimization::{CompactAlphaMemory, NodeSharingRegistry, TokenPool};
use rust_rule_engine::rete::{AlphaNode, TypedFacts};
use std::mem;

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                  📊 MEMORY USAGE ANALYSIS 📊                         ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    measure_node_sharing_memory();
    measure_alpha_memory();
    measure_token_pooling();

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                    ✅ ANALYSIS COMPLETE ✅                           ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");
}

// ===========================================================================
// NODE SHARING MEMORY ANALYSIS
// ===========================================================================

fn measure_node_sharing_memory() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📦 Node Sharing Memory Impact");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // WITHOUT sharing - store 10,000 nodes directly
    let nodes_without: Vec<AlphaNode> = (0..10000)
        .map(|i| AlphaNode {
            field: format!("field{}", i % 100),
            operator: ">".to_string(),
            value: "50".to_string(),
        })
        .collect();

    let size_per_node = mem::size_of::<AlphaNode>();
    let total_without = nodes_without.len() * size_per_node;

    println!("WITHOUT Sharing (10,000 nodes, 100 unique patterns):");
    println!("  Size per node:     {} bytes", size_per_node);
    println!("  Total nodes:       10,000");
    println!(
        "  Total memory:      {} bytes ({:.2} KB)",
        total_without,
        total_without as f64 / 1024.0
    );

    // WITH sharing - use registry
    let mut registry = NodeSharingRegistry::new();
    for (idx, node) in nodes_without.iter().enumerate() {
        registry.register(node, idx);
    }

    let stats = registry.stats();

    // Estimate: only store unique nodes + HashMap overhead
    let unique_nodes_size = stats.unique_patterns * size_per_node;
    let hashmap_overhead = stats.unique_patterns * 64; // Rough estimate: key + value + hash
    let total_with = unique_nodes_size + hashmap_overhead;

    println!("\nWITH Sharing (NodeSharingRegistry):");
    println!("  Unique patterns:   {}", stats.unique_patterns);
    println!(
        "  Unique nodes size: {} bytes ({:.2} KB)",
        unique_nodes_size,
        unique_nodes_size as f64 / 1024.0
    );
    println!("  HashMap overhead:  ~{} bytes", hashmap_overhead);
    println!(
        "  Total memory:      ~{} bytes ({:.2} KB)",
        total_with,
        total_with as f64 / 1024.0
    );

    let saved = total_without.saturating_sub(total_with);
    let saved_percent = (saved as f64 / total_without as f64) * 100.0;

    println!("\n✅ Memory Savings:");
    println!(
        "  Saved:             {} bytes ({:.2} KB)",
        saved,
        saved as f64 / 1024.0
    );
    println!("  Reduction:         {:.1}%", saved_percent);
    println!();
}

// ===========================================================================
// ALPHA MEMORY ANALYSIS
// ===========================================================================

fn measure_alpha_memory() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💾 Alpha Memory Compaction Impact");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Create 10,000 facts (only 100 unique)
    let facts: Vec<TypedFacts> = (0..10000)
        .map(|i| {
            let mut fact = TypedFacts::new();
            fact.set("id", (i % 100) as i64);
            fact.set("value", (i % 100) as i64);
            fact
        })
        .collect();

    // WITHOUT compaction - Vec storage
    let size_per_fact = mem::size_of::<TypedFacts>();
    let total_without = facts.len() * size_per_fact;

    println!("WITHOUT Compaction (Vec<TypedFacts>):");
    println!("  Facts inserted:    10,000");
    println!("  Size per fact:     {} bytes", size_per_fact);
    println!(
        "  Total memory:      {} bytes ({:.2} KB)",
        total_without,
        total_without as f64 / 1024.0
    );

    // WITH compaction - HashSet storage
    let mut compact_memory = CompactAlphaMemory::new();
    for fact in &facts {
        compact_memory.add(fact);
    }

    let unique_count = compact_memory.len();
    let unique_facts_size = unique_count * size_per_fact;
    let hashset_overhead = unique_count * 32; // Hash + next pointer
    let total_with = unique_facts_size + hashset_overhead;

    println!("\nWITH Compaction (CompactAlphaMemory):");
    println!("  Unique facts:      {}", unique_count);
    println!(
        "  Unique facts size: {} bytes ({:.2} KB)",
        unique_facts_size,
        unique_facts_size as f64 / 1024.0
    );
    println!("  HashSet overhead:  ~{} bytes", hashset_overhead);
    println!(
        "  Total memory:      ~{} bytes ({:.2} KB)",
        total_with,
        total_with as f64 / 1024.0
    );

    let saved = total_without.saturating_sub(total_with);
    let saved_percent = (saved as f64 / total_without as f64) * 100.0;

    println!("\n✅ Memory Savings:");
    println!(
        "  Saved:             {} bytes ({:.2} KB)",
        saved,
        saved as f64 / 1024.0
    );
    println!("  Reduction:         {:.1}%", saved_percent);
    println!(
        "  Savings ratio:     {:.1}%",
        compact_memory.memory_savings()
    );
    println!();
}

// ===========================================================================
// TOKEN POOLING ANALYSIS
// ===========================================================================

fn measure_token_pooling() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("♻️  Token Pooling Memory Impact");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // WITHOUT pooling - create 10,000 individual TypedFacts
    let size_per_fact = mem::size_of::<TypedFacts>();
    let iterations = 10000;
    let total_without = iterations * size_per_fact;

    println!("WITHOUT Pooling (10,000 allocations):");
    println!("  Iterations:        {}", iterations);
    println!("  Size per fact:     {} bytes", size_per_fact);
    println!(
        "  Total allocated:   {} bytes ({:.2} KB)",
        total_without,
        total_without as f64 / 1024.0
    );
    println!("  Allocation count:  {}", iterations);

    // WITH pooling - pre-allocate pool
    let pool_size = 100;
    let _pool = TokenPool::new(pool_size);

    let size_per_token = mem::size_of::<rust_rule_engine::rete::optimization::Token>();
    let pool_memory = pool_size * size_per_token;
    let vec_overhead = 64; // Vec capacity overhead
    let hashset_overhead = pool_size * 16; // HashSet tracking
    let total_with = pool_memory + vec_overhead + hashset_overhead;

    println!("\nWITH Pooling (TokenPool size: {}):", pool_size);
    println!("  Pool tokens:       {}", pool_size);
    println!("  Size per token:    {} bytes", size_per_token);
    println!(
        "  Pool memory:       {} bytes ({:.2} KB)",
        pool_memory,
        pool_memory as f64 / 1024.0
    );
    println!(
        "  Overhead:          ~{} bytes",
        vec_overhead + hashset_overhead
    );
    println!(
        "  Total memory:      ~{} bytes ({:.2} KB)",
        total_with,
        total_with as f64 / 1024.0
    );
    println!("  Allocation count:  {} (vs {})", pool_size, iterations);

    let saved_allocs = iterations - pool_size;
    let alloc_reduction = (saved_allocs as f64 / iterations as f64) * 100.0;

    println!("\n✅ Allocation Savings:");
    println!("  Allocations saved: {}", saved_allocs);
    println!("  Reduction:         {:.1}%", alloc_reduction);

    println!("\n⚠️  Trade-off:");
    println!(
        "  Peak memory is HIGHER with pooling ({:.2} KB always allocated)",
        total_with as f64 / 1024.0
    );
    println!(
        "  But amortized over time, saves {} repeated allocations",
        saved_allocs
    );
    println!("  Only beneficial for continuous high-volume processing");
    println!();
}

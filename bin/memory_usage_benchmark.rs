//! Memory Usage Benchmark
//!
//! Measures peak memory consumption for different optimization strategies.

use rust_rule_engine::rete::{AlphaMemoryIndex, BetaMemoryIndex, TypedFacts};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Memory tracking allocator
struct MemoryTracker;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for MemoryTracker {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            let size = layout.size();
            let current = ALLOCATED.fetch_add(size, Ordering::SeqCst) + size;
            let mut peak = PEAK.load(Ordering::SeqCst);
            while current > peak {
                match PEAK.compare_exchange_weak(peak, current, Ordering::SeqCst, Ordering::SeqCst)
                {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: MemoryTracker = MemoryTracker;

fn reset_memory_tracker() {
    ALLOCATED.store(0, Ordering::SeqCst);
    PEAK.store(0, Ordering::SeqCst);
}

fn get_peak_memory() -> usize {
    PEAK.load(Ordering::SeqCst)
}

fn format_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn benchmark_alpha_memory_usage() {
    println!("\n=== Alpha Memory Indexing - Memory Usage ===\n");

    for fact_count in [1_000, 10_000, 50_000, 100_000] {
        // Create facts
        let mut facts = Vec::new();
        for i in 0..fact_count {
            let mut fact = TypedFacts::new();
            fact.set("id", i as i64);
            fact.set("status", if i % 100 == 0 { "active" } else { "pending" });
            fact.set("priority", if i % 10 == 0 { "high" } else { "low" });
            fact.set("amount", (i * 100) as i64);
            facts.push(fact);
        }

        // Test 1: No index
        reset_memory_tracker();
        {
            let mut mem = AlphaMemoryIndex::new();
            for fact in &facts {
                mem.insert(fact.clone());
            }
            std::mem::forget(mem); // Keep it alive for peak measurement
        }
        let no_index_peak = get_peak_memory();

        // Test 2: Single index
        reset_memory_tracker();
        {
            let mut mem = AlphaMemoryIndex::new();
            mem.create_index("status".to_string());
            for fact in &facts {
                mem.insert(fact.clone());
            }
            std::mem::forget(mem);
        }
        let single_index_peak = get_peak_memory();

        // Test 3: Multiple indexes
        reset_memory_tracker();
        {
            let mut mem = AlphaMemoryIndex::new();
            mem.create_index("status".to_string());
            mem.create_index("priority".to_string());
            mem.create_index("amount".to_string());
            for fact in &facts {
                mem.insert(fact.clone());
            }
            std::mem::forget(mem);
        }
        let multi_index_peak = get_peak_memory();

        let overhead_single = ((single_index_peak as f64 / no_index_peak as f64) - 1.0) * 100.0;
        let overhead_multi = ((multi_index_peak as f64 / no_index_peak as f64) - 1.0) * 100.0;

        println!("📊 {} facts:", fact_count);
        println!(
            "  No index:        {} (baseline)",
            format_bytes(no_index_peak)
        );
        println!(
            "  Single index:    {} (+{:.1}%)",
            format_bytes(single_index_peak),
            overhead_single
        );
        println!(
            "  3 indexes:       {} (+{:.1}%)",
            format_bytes(multi_index_peak),
            overhead_multi
        );
        println!("  Overhead/index:  {:.1}%\n", overhead_multi / 3.0);
    }
}

fn benchmark_beta_memory_usage() {
    println!("\n=== Beta Memory Indexing - Memory Usage ===\n");

    for fact_count in [100, 1_000, 10_000] {
        // Create facts
        let mut left_facts = Vec::new();
        for i in 0..fact_count {
            let mut fact = TypedFacts::new();
            fact.set("order_id", i as i64);
            fact.set("customer_id", format!("C{}", i % 100));
            left_facts.push(fact);
        }

        let mut right_facts = Vec::new();
        for i in 0..fact_count {
            let mut fact = TypedFacts::new();
            fact.set("customer_id", format!("C{}", i % 100));
            fact.set("tier", if i % 10 == 0 { "gold" } else { "silver" });
            right_facts.push(fact);
        }

        // Test 1: No index (just store facts)
        reset_memory_tracker();
        {
            let _left = left_facts.clone();
            let _right = right_facts.clone();
            std::mem::forget(_left);
            std::mem::forget(_right);
        }
        let no_index_peak = get_peak_memory();

        // Test 2: With index
        reset_memory_tracker();
        {
            let mut index = BetaMemoryIndex::new("customer_id".to_string());
            for (idx, fact) in right_facts.iter().enumerate() {
                index.add(fact, idx);
            }
            std::mem::forget(index);
        }
        let index_peak = get_peak_memory();

        let overhead = ((index_peak as f64 / no_index_peak as f64) - 1.0) * 100.0;

        println!(
            "📊 {} facts per side ({} total joins):",
            fact_count,
            fact_count * fact_count
        );
        println!("  No index:  {} (facts only)", format_bytes(no_index_peak));
        println!(
            "  With index: {} (+{:.1}%)\n",
            format_bytes(index_peak),
            overhead
        );
    }
}

fn benchmark_combined_memory() {
    println!("\n=== Combined Optimization - Memory Usage ===\n");

    let fact_count = 10_000;

    // Create facts
    let mut facts = Vec::new();
    for i in 0..fact_count {
        let mut fact = TypedFacts::new();
        fact.set("id", i as i64);
        fact.set("status", if i % 10 == 0 { "active" } else { "pending" });
        fact.set("priority", if i % 5 == 0 { "high" } else { "low" });
        fact.set("category", format!("cat_{}", i % 20));
        fact.set("amount", (i * 100) as i64);
        facts.push(fact);
    }

    // Baseline: no optimization
    reset_memory_tracker();
    {
        let mut mem = AlphaMemoryIndex::new();
        for fact in &facts {
            mem.insert(fact.clone());
        }
        std::mem::forget(mem);
    }
    let baseline = get_peak_memory();

    // Light optimization: 1 index
    reset_memory_tracker();
    {
        let mut mem = AlphaMemoryIndex::new();
        mem.create_index("status".to_string());
        for fact in &facts {
            mem.insert(fact.clone());
        }
        std::mem::forget(mem);
    }
    let light = get_peak_memory();

    // Medium: 3 indexes
    reset_memory_tracker();
    {
        let mut mem = AlphaMemoryIndex::new();
        mem.create_index("status".to_string());
        mem.create_index("priority".to_string());
        mem.create_index("category".to_string());
        for fact in &facts {
            mem.insert(fact.clone());
        }
        std::mem::forget(mem);
    }
    let medium = get_peak_memory();

    // Heavy: 5 indexes
    reset_memory_tracker();
    {
        let mut mem = AlphaMemoryIndex::new();
        mem.create_index("status".to_string());
        mem.create_index("priority".to_string());
        mem.create_index("category".to_string());
        mem.create_index("amount".to_string());
        mem.create_index("id".to_string());
        for fact in &facts {
            mem.insert(fact.clone());
        }
        std::mem::forget(mem);
    }
    let heavy = get_peak_memory();

    println!("📊 10,000 facts with varying index counts:\n");
    println!("  Baseline (0 indexes):  {}", format_bytes(baseline));
    println!(
        "  Light (1 index):       {} (+{:.1}%)",
        format_bytes(light),
        ((light as f64 / baseline as f64) - 1.0) * 100.0
    );
    println!(
        "  Medium (3 indexes):    {} (+{:.1}%)",
        format_bytes(medium),
        ((medium as f64 / baseline as f64) - 1.0) * 100.0
    );
    println!(
        "  Heavy (5 indexes):     {} (+{:.1}%)",
        format_bytes(heavy),
        ((heavy as f64 / baseline as f64) - 1.0) * 100.0
    );

    println!(
        "\n💡 Memory per index: ~{:.1}%",
        ((heavy as f64 / baseline as f64) - 1.0) * 100.0 / 5.0
    );
}

fn main() {
    println!("🧪 Memory Usage Benchmark");
    println!("==========================");

    benchmark_alpha_memory_usage();
    benchmark_beta_memory_usage();
    benchmark_combined_memory();

    println!("\n✅ Benchmark complete!\n");
}

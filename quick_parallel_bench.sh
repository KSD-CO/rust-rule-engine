#!/bin/bash

# Quick Parallel vs Sequential Benchmark Comparison
echo "🚀 Quick Parallel vs Sequential Benchmark"
echo "=========================================="
echo ""

echo "📊 Running small benchmark comparison (10 rules)..."
cargo bench --bench parallel_benchmarks bench_small_sequential_vs_parallel -- --quiet

echo ""
echo "🔍 Key Comparisons:"
echo "• Sequential execution: baseline performance"
echo "• 2-thread parallel: should show ~1.5x improvement for small sets"
echo "• 4-thread parallel: may show diminishing returns for small sets"
echo ""
echo "📈 For better parallel gains, try larger rule sets:"
echo "• cargo bench --bench parallel_benchmarks bench_medium_sequential_vs_parallel"
echo "• cargo bench --bench parallel_benchmarks bench_large_sequential_vs_parallel"
echo ""
echo "📊 View detailed results:"
echo "• open target/criterion/reports/index.html"
echo ""

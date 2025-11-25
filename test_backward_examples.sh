#!/bin/bash

# Quick test script for backward chaining examples

echo "🧪 Testing Backward Chaining Examples"
echo "====================================="

# Build library with backward-chaining feature
echo "📦 Building library with backward-chaining feature..."
cargo build --lib --features backward-chaining --quiet
if [ $? -ne 0 ]; then
    echo "❌ Library build failed"
    exit 1
fi
echo "✅ Library built successfully"

# Test simple_query_demo
echo ""
echo "🔬 Testing simple_query_demo..."
rustc --edition 2021 \
    --crate-type bin \
    examples/09-backward-chaining/simple_query_demo.rs \
    --cfg 'feature="backward-chaining"' \
    -L target/debug/deps \
    --extern rust_rule_engine=target/debug/librust_rule_engine.rlib \
    -o target/debug/simple_query_demo \
    2>&1 | head -10

if [ $? -eq 0 ]; then
    echo "✅ simple_query_demo compiled"
    echo "   Running..."
    ./target/debug/simple_query_demo 2>&1 | head -50
else
    echo "❌ simple_query_demo failed to compile"
fi

# Test medical_diagnosis_demo
echo ""
echo "🏥 Testing medical_diagnosis_demo..."
rustc --edition 2021 \
    --crate-type bin \
    examples/09-backward-chaining/medical_diagnosis_demo.rs \
    --cfg 'feature="backward-chaining"' \
    -L target/debug/deps \
    --extern rust_rule_engine=target/debug/librust_rule_engine.rlib \
    -o target/debug/medical_diagnosis_demo \
    2>&1 | head -10

if [ $? -eq 0 ]; then
    echo "✅ medical_diagnosis_demo compiled"
    echo "   Running first demo..."
    ./target/debug/medical_diagnosis_demo 2>&1 | head -60
else
    echo "❌ medical_diagnosis_demo failed to compile"
fi

echo ""
echo "✨ Testing complete!"

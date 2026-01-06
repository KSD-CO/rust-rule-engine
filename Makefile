.PHONY: help all examples ci check fmt fmt-check clippy test build doc-test test-features
.PHONY: getting-started rete-engine advanced-features performance use-cases advanced-rete backward-chaining module-system

# Default target
help:
	@echo "Available targets:"
	@echo ""
	@echo "CI & Testing:"
	@echo "  make ci                     - Full CI check (matches GitHub Actions)"
	@echo "  make check                  - Quick check (fmt, clippy, test only)"
	@echo "  make fmt                    - Format code"
	@echo "  make fmt-check              - Check code formatting"
	@echo "  make clippy                 - Run clippy linter"
	@echo "  make test                   - Run tests"
	@echo "  make build                  - Build project"
	@echo "  make doc-test               - Run documentation tests"
	@echo ""
	@echo "Examples (streamlined from 108 to 26):"
	@echo "  make all                    - Run all examples"
	@echo "  make getting-started        - Run getting-started examples (4)"
	@echo "  make rete-engine            - Run RETE engine examples (5)"
	@echo "  make advanced-features      - Run advanced features examples (6)"
	@echo "  make performance            - Run performance examples (3)"
	@echo "  make advanced-rete          - Run advanced RETE examples (2)"
	@echo "  make backward-chaining      - Run backward-chaining examples (4)"
	@echo "  make module-system          - Run module-system examples (2)"

# =============================================================================
# CI CHECKS (matches GitHub Actions)
# =============================================================================

ci: fmt-check clippy build test test-features doc-test
	@echo "✅ All CI checks passed!"

# Quick check without build
check: fmt clippy test
	@echo "✅ All checks passed!"

fmt:
	@echo "🔧 Formatting code..."
	@cargo fmt

fmt-check:
	@echo "🔍 Checking code formatting..."
	@cargo fmt -- --check

clippy:
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

test:
	@echo "🧪 Running tests..."
	@cargo test --verbose --all-features

build:
	@echo "🔨 Building project..."
	@cargo build --verbose --all-features

doc-test:
	@echo "📚 Running doc tests..."
	@cargo test --doc --verbose

# Test different feature combinations
test-features:
	@echo "🧪 Testing feature combinations..."
	@echo "  Testing: no features (default)"
	@cargo test --no-default-features --lib
	@echo "  Testing: backward-chaining only"
	@cargo test --no-default-features --features backward-chaining --lib
	@echo "  Testing: streaming only"
	@cargo test --no-default-features --features streaming --lib
	@echo "  Testing: backward-chaining + streaming"
	@cargo test --no-default-features --features "backward-chaining,streaming" --lib
	@echo "  Testing: all features"
	@cargo test --all-features --lib
	@echo "✅ All feature combinations passed!"

# =============================================================================
# EXAMPLES (26 essential examples)
# =============================================================================

all: getting-started rete-engine advanced-features performance advanced-rete backward-chaining module-system

# 01 - Getting Started (4 examples)
getting-started:
	@echo "=== 01 - Getting Started (4 examples) ==="
	cargo run --example grule_demo
	cargo run --example fraud_detection
	cargo run --example expression_demo
	cargo run --example method_calls_demo

# 02 - RETE Engine (5 examples)
rete-engine:
	@echo "=== 02 - RETE Engine (5 examples) ==="
	cargo run --example rete_demo
	cargo run --example rete_grl_demo
	cargo run --example rete_typed_facts_demo
	cargo run --example rete_deffacts_demo
	cargo run --example tms_demo

# 03 - Advanced Features (6 examples)
advanced-features:
	@echo "=== 03 - Advanced Features (6 examples) ==="
	cargo run --example accumulate_grl_demo
	cargo run --example conflict_resolution_demo
	cargo run --example grl_no_loop_demo
	cargo run --example action_handlers_grl_demo
	cargo run --example rule_templates_demo
	cargo run --features streaming --example streaming_with_rules_demo

# 05 - Performance (3 examples)
performance:
	@echo "=== 05 - Performance (3 examples) ==="
	cargo run --release --example quick_engine_comparison
	cargo run --release --example parallel_engine_demo
	cargo run --release --example memory_usage_comparison

# 07 - Advanced RETE (2 examples)
advanced-rete:
	@echo "=== 07 - Advanced RETE (2 examples) ==="
	cargo run --example rete_p3_incremental
	cargo run --example rete_ul_drools_style

# 09 - Backward Chaining (4 examples)
backward-chaining:
	@echo "=== 09 - Backward Chaining (4 examples) ==="
	cargo run --features backward-chaining --example simple_query_demo
	cargo run --features backward-chaining --example ecommerce_approval_demo
	cargo run --features backward-chaining --example medical_diagnosis_demo
	cargo run --features backward-chaining --example grl_query_demo

# 10 - Module System (2 examples)
module-system:
	@echo "=== 10 - Module System (2 examples) ==="
	cargo run --example smart_home_modules
	cargo run --example phase3_demo

# =============================================================================
# INDIVIDUAL EXAMPLE SHORTCUTS
# =============================================================================

# Getting Started
grule_demo:
	cargo run --example grule_demo

fraud_detection:
	cargo run --example fraud_detection

expression_demo:
	cargo run --example expression_demo

method_calls_demo:
	cargo run --example method_calls_demo

# RETE Engine
rete_demo:
	cargo run --example rete_demo

rete_grl_demo:
	cargo run --example rete_grl_demo

rete_typed_facts_demo:
	cargo run --example rete_typed_facts_demo

rete_deffacts_demo:
	cargo run --example rete_deffacts_demo

tms_demo:
	cargo run --example tms_demo

# Advanced Features
accumulate_grl_demo:
	cargo run --example accumulate_grl_demo

conflict_resolution_demo:
	cargo run --example conflict_resolution_demo

grl_no_loop_demo:
	cargo run --example grl_no_loop_demo

action_handlers_grl_demo:
	cargo run --example action_handlers_grl_demo

rule_templates_demo:
	cargo run --example rule_templates_demo

streaming_with_rules_demo:
	cargo run --features streaming --example streaming_with_rules_demo

# Performance
quick_engine_comparison:
	cargo run --release --example quick_engine_comparison

parallel_engine_demo:
	cargo run --release --example parallel_engine_demo

memory_usage_comparison:
	cargo run --release --example memory_usage_comparison

# Advanced RETE
rete_p3_incremental:
	cargo run --example rete_p3_incremental

rete_ul_drools_style:
	cargo run --example rete_ul_drools_style

# Backward Chaining
simple_query_demo:
	cargo run --features backward-chaining --example simple_query_demo

ecommerce_approval_demo:
	cargo run --features backward-chaining --example ecommerce_approval_demo

medical_diagnosis_demo:
	cargo run --features backward-chaining --example medical_diagnosis_demo

grl_query_demo:
	cargo run --features backward-chaining --example grl_query_demo

# Module System
smart_home_modules:
	cargo run --example smart_home_modules

phase3_demo:
	cargo run --example phase3_demo

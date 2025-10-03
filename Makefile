# Makefile for rust-rule-engine crate
# =================================

.PHONY: help build test clean check format lint docs examples publish-dry publish-real release setup git-commit

# Default target
help:
	@echo "🚀 Rust Rule Engine - Makefile Commands"
	@echo "======================================="
	@echo ""
	@echo "📦 Build & Test:"
	@echo "  make build      - Build the project"
	@echo "  make test       - Run all tests"
	@echo "  make examples   - Run all examples"
	@echo "  make check      - Run cargo check"
	@echo ""
	@echo "🔧 Code Quality:"
	@echo "  make format     - Format code with rustfmt"
	@echo "  make lint       - Run clippy lints"
	@echo "  make docs       - Generate documentation"
	@echo ""
	@echo "📋 Git & Publishing:"
	@echo "  make git-commit - Add all files and commit"
	@echo "  make publish-dry - Dry run publish (test only)"
	@echo "  make publish - Publish to crates.io"
	@echo "  make release     - Full release process"
	@echo ""
	@echo "🧹 Cleanup:"
	@echo "  make clean      - Clean build artifacts"
	@echo ""

# Build the project
build:
	@echo "🔨 Building rust-rule-engine..."
	cargo build

# Build in release mode
build-release:
	@echo "🔨 Building rust-rule-engine (release)..."
	cargo build --release

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Run benchmarks
bench:
	@echo "⚡ Running benchmarks..."
	cargo bench

# Run benchmarks with baseline
bench-baseline:
	@echo "⚡ Running benchmarks with baseline..."
	cargo bench -- --save-baseline main

# Compare benchmarks
bench-compare:
	@echo "⚡ Comparing benchmarks..."
	cargo bench -- --baseline main

# Open benchmark report
bench-report:
	@echo "📊 Opening benchmark report..."
	@if [ -f target/criterion/index.html ]; then \
		echo "🌐 Opening benchmark report in browser..."; \
		xdg-open target/criterion/index.html 2>/dev/null || open target/criterion/index.html 2>/dev/null || echo "Please open target/criterion/index.html manually"; \
	else \
		echo "❌ No benchmark report found. Run 'make bench' first."; \
	fi

# Run all examples
examples:
	@echo "🎯 Running all examples..."
	@echo "================================"
	@for example in ecommerce fraud_detection grule_demo method_calls_demo rule_file_functions_demo custom_functions_demo; do \
		echo ""; \
		echo "🚀 Running example: $$example"; \
		echo "----------------------------"; \
		cargo run --example $$example || true; \
		echo ""; \
	done

# Check the project
check:
	@echo "✅ Checking project..."
	cargo check

# Format code
format:
	@echo "🎨 Formatting code..."
	cargo fmt

# Run clippy lints
lint:
	@echo "🔍 Running clippy lints..."
	cargo clippy -- -D warnings

# Generate documentation
docs:
	@echo "📚 Generating documentation..."
	cargo doc --open

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Git operations
git-status:
	@echo "📋 Git status:"
	git status --short

git-add:
	@echo "➕ Adding all files to git..."
	git add .

git-commit: git-add
	@echo "💾 Committing changes..."
	@read -p "Enter commit message: " msg; \
	git commit -m "$$msg"

# Pre-publish checks
pre-publish: format lint test
	@echo "🔍 Pre-publish checks completed successfully!"

# Dry run publish (test only)
publish-dry: pre-publish
	@echo "🧪 Dry run publish to crates.io..."
	@echo "⚠️  This will test the publish process without actually publishing"
	@echo ""
	@echo "📋 Package info:"
	@grep "^name\|^version\|^description" Cargo.toml
	@echo ""
	@echo "📁 Files to be included:"
	@cargo package --list | head -20
	@echo ""
	@read -p "Continue with dry run? (y/N): " confirm; \
	if [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]; then \
		cargo publish --dry-run --allow-dirty; \
	else \
		echo "❌ Dry run cancelled"; \
	fi

# Real publish to crates.io
publish: pre-publish git-status
	@echo "🚀 Publishing to crates.io..."
	@echo "⚠️  This will ACTUALLY publish the crate!"
	@echo ""
	@echo "📋 Package info:"
	@grep "^name\|^version\|^description" Cargo.toml
	@echo ""
	@echo "🔍 Git status:"
	@git status --short
	@echo ""
	@if [ -n "$$(git status --porcelain)" ]; then \
		echo "⚠️  Warning: You have uncommitted changes!"; \
		echo "Run 'make git-commit' first or use --allow-dirty"; \
		echo ""; \
		read -p "Publish with uncommitted changes? (y/N): " dirty; \
		if [ "$$dirty" = "y" ] || [ "$$dirty" = "Y" ]; then \
			echo "Publishing with --allow-dirty..."; \
			cargo publish --allow-dirty; \
		else \
			echo "❌ Publish cancelled. Commit your changes first."; \
			exit 1; \
		fi; \
	else \
		read -p "Continue with publish? (y/N): " confirm; \
		if [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]; then \
			cargo publish; \
		else \
			echo "❌ Publish cancelled"; \
		fi; \
	fi

# Full release process
release: 
	@echo "🎉 Starting full release process..."
	@echo "=================================="
	@echo ""
	@echo "This will:"
	@echo "1. 🎨 Format code"
	@echo "2. 🔍 Run lints"
	@echo "3. 🧪 Run tests"
	@echo "4. 🎯 Run examples"
	@echo "5. 💾 Commit changes (if needed)"
	@echo "6. 🚀 Publish to crates.io"
	@echo ""
	@read -p "Continue with full release? (y/N): " confirm; \
	if [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]; then \
		$(MAKE) format; \
		$(MAKE) lint; \
		$(MAKE) test; \
		$(MAKE) examples; \
		if [ -n "$$(git status --porcelain)" ]; then \
			echo ""; \
			echo "💾 Changes detected, committing..."; \
			$(MAKE) git-commit; \
		fi; \
		echo ""; \
		echo "🚀 Ready to publish!"; \
		$(MAKE) publish-real; \
	else \
		echo "❌ Release cancelled"; \
	fi

# Setup development environment
setup:
	@echo "🔧 Setting up development environment..."
	@echo "Installing required tools..."
	rustup component add rustfmt clippy
	@echo "✅ Setup complete!"

# Version bump helpers
version-patch:
	@echo "📈 Bumping patch version..."
	@current=$$(grep "^version" Cargo.toml | cut -d'"' -f2); \
	echo "Current version: $$current"; \
	read -p "Enter new patch version (e.g., 0.1.1): " new_version; \
	sed -i 's/^version = ".*"/version = "'$$new_version'"/' Cargo.toml; \
	echo "✅ Version updated to $$new_version"

version-minor:
	@echo "📈 Bumping minor version..."
	@current=$$(grep "^version" Cargo.toml | cut -d'"' -f2); \
	echo "Current version: $$current"; \
	read -p "Enter new minor version (e.g., 0.2.0): " new_version; \
	sed -i 's/^version = ".*"/version = "'$$new_version'"/' Cargo.toml; \
	echo "✅ Version updated to $$new_version"

version-major:
	@echo "📈 Bumping major version..."
	@current=$$(grep "^version" Cargo.toml | cut -d'"' -f2); \
	echo "Current version: $$current"; \
	read -p "Enter new major version (e.g., 1.0.0): " new_version; \
	sed -i 's/^version = ".*"/version = "'$$new_version'"/' Cargo.toml; \
	echo "✅ Version updated to $$new_version"

# Package information
package-info:
	@echo "📦 Package Information"
	@echo "====================="
	@echo ""
	@echo "📋 Basic Info:"
	@grep "^name\|^version\|^description\|^authors" Cargo.toml
	@echo ""
	@echo "📁 Files to be packaged:"
	@cargo package --list 2>/dev/null | head -20 || echo "Run 'cargo check' first"
	@echo ""
	@echo "📊 Package size:"
	@cargo package --list 2>/dev/null | wc -l || echo "Run 'cargo check' first" 
	@echo " files total"

# All quality checks
qa: format lint test
	@echo "✅ All quality assurance checks passed!"

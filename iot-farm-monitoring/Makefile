.PHONY: help build test run demo kafka-setup kafka-produce kafka-stop clean check

help:
	@echo "IoT Farm Monitoring System - Makefile"
	@echo "======================================"
	@echo ""
	@echo "Available targets:"
	@echo "  make build          - Build the project"
	@echo "  make test           - Run tests"
	@echo "  make demo           - Run basic demo (no Kafka)"
	@echo "  make run            - Run main application (requires Kafka)"
	@echo "  make kafka-setup    - Setup Kafka topics"
	@echo "  make kafka-produce  - Produce sample events"
	@echo "  make kafka-stop     - Stop Kafka containers"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make check          - Run clippy and fmt check"

build:
	@echo "🔨 Building project..."
	cargo build --release

test:
	@echo "🧪 Running tests..."
	cargo test --verbose

demo:
	@echo "🚜 Running basic demo..."
	cargo run --example basic_demo

run:
	@echo "🚜 Running farm monitoring system..."
	cargo run --bin farm-monitor

kafka-example:
	@echo "🔌 Running Kafka consumer example..."
	cargo run --example kafka_consumer

kafka-setup:
	@echo "⚙️  Setting up Kafka..."
	./scripts/setup_kafka.sh

kafka-produce:
	@echo "📤 Producing sample events..."
	./scripts/produce_events.sh

kafka-stop:
	@echo "🛑 Stopping Kafka..."
	docker-compose down

clean:
	@echo "🧹 Cleaning..."
	cargo clean

check:
	@echo "🔍 Running checks..."
	cargo fmt -- --check
	cargo clippy -- -D warnings

# Development targets
dev-demo: demo

dev-kafka: kafka-setup
	@echo ""
	@echo "Starting Kafka consumer in 5 seconds..."
	@sleep 5
	@make kafka-example

# Complete setup for first-time users
setup: build kafka-setup
	@echo ""
	@echo "✅ Setup complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Run demo: make demo"
	@echo "  2. Run with Kafka: make kafka-example"
	@echo "  3. Produce events: make kafka-produce"

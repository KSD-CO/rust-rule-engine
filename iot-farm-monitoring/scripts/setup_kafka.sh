#!/bin/bash
# Setup Kafka topics for IoT Farm Monitoring

echo "🚀 Setting up Kafka topics for IoT Farm Monitoring"
echo "=================================================="

# Start Kafka and Zookeeper if not running
echo ""
echo "1. Starting Kafka and Zookeeper..."
docker-compose up -d

# Wait for Kafka to be ready
echo ""
echo "2. Waiting for Kafka to be ready..."
sleep 10

# Create topics
echo ""
echo "3. Creating Kafka topics..."

docker exec -it kafka kafka-topics.sh \
  --create \
  --topic soil-sensors \
  --bootstrap-server localhost:9092 \
  --partitions 3 \
  --replication-factor 1 \
  --if-not-exists

docker exec -it kafka kafka-topics.sh \
  --create \
  --topic temperature \
  --bootstrap-server localhost:9092 \
  --partitions 3 \
  --replication-factor 1 \
  --if-not-exists

docker exec -it kafka kafka-topics.sh \
  --create \
  --topic irrigation \
  --bootstrap-server localhost:9092 \
  --partitions 3 \
  --replication-factor 1 \
  --if-not-exists

docker exec -it kafka kafka-topics.sh \
  --create \
  --topic weather \
  --bootstrap-server localhost:9092 \
  --partitions 3 \
  --replication-factor 1 \
  --if-not-exists

# List topics
echo ""
echo "4. Listing created topics:"
docker exec -it kafka kafka-topics.sh \
  --list \
  --bootstrap-server localhost:9092

echo ""
echo "✅ Kafka setup complete!"
echo ""
echo "Next steps:"
echo "  1. Run the consumer: cargo run --example kafka_consumer"
echo "  2. Produce test events: ./scripts/produce_events.sh"
echo "  3. View Kafka UI: http://localhost:8080"

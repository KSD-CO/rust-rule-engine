#!/bin/bash
# Produce sample IoT events to Kafka topics

echo "🌾 Producing sample IoT farm events to Kafka"
echo "==========================================="

# Soil sensor events
echo ""
echo "1. Producing soil sensor events..."
echo '{"zone_id":"zone_1","moisture_level":25.0,"timestamp":1000}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic soil-sensors \
    --bootstrap-server localhost:9092

echo '{"zone_id":"zone_2","moisture_level":45.0,"timestamp":2000}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic soil-sensors \
    --bootstrap-server localhost:9092

echo '{"zone_id":"zone_3","moisture_level":35.0,"timestamp":3000}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic soil-sensors \
    --bootstrap-server localhost:9092

# Temperature events
echo ""
echo "2. Producing temperature events..."
echo '{"zone_id":"zone_1","temperature":28.0,"sensor_type":"soil","timestamp":1010}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic temperature \
    --bootstrap-server localhost:9092

echo '{"zone_id":"zone_2","temperature":22.0,"sensor_type":"soil","timestamp":2010}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic temperature \
    --bootstrap-server localhost:9092

echo '{"zone_id":"zone_3","temperature":1.0,"sensor_type":"air","timestamp":3010}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic temperature \
    --bootstrap-server localhost:9092

# Irrigation events
echo ""
echo "3. Producing irrigation events..."
echo '{"zone_id":"zone_1","action":"start","timestamp":4000}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic irrigation \
    --bootstrap-server localhost:9092

echo '{"zone_id":"zone_1","action":"stop","water_volume_ml":50000,"timestamp":4300}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic irrigation \
    --bootstrap-server localhost:9092

# Weather events
echo ""
echo "4. Producing weather events..."
echo '{"location":"farm","condition":"frost_risk","temperature":0.5,"timestamp":3005}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic weather \
    --bootstrap-server localhost:9092

echo '{"location":"farm","condition":"clear","temperature":25.0,"timestamp":5000}' | \
  docker exec -i kafka kafka-console-producer.sh \
    --topic weather \
    --bootstrap-server localhost:9092

echo ""
echo "✅ Sample events produced!"
echo ""
echo "Expected outcomes:"
echo "  - Zone 1: Irrigation triggered (low moisture + high temp)"
echo "  - Zone 2: No irrigation (adequate moisture)"
echo "  - Zone 3: Frost alert (low temperature + frost risk)"
echo ""
echo "Check the consumer logs to see the results!"

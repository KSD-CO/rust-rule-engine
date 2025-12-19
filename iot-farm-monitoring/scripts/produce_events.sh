#!/bin/bash
# Produce sample IoT events to Kafka topics

echo "🌾 Producing sample IoT farm events to Kafka"
echo "==========================================="

# Get current timestamp in milliseconds
NOW=$(date +%s)000
T1=$((NOW))
T2=$((NOW + 1000))
T3=$((NOW + 2000))
T4=$((NOW + 3000))
T5=$((NOW + 4000))
T6=$((NOW + 5000))

echo "Using timestamps starting from: $NOW"
echo ""

# Soil sensor events
echo ""
echo "1. Producing soil sensor events..."
# Zone 1: Low moisture to trigger CriticalIrrigationNeeded
echo "{\"zone_id\":\"zone_1\",\"moisture_level\":20.0,\"timestamp\":$T1}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic soil-sensors \
    --bootstrap-server localhost:9092

# Zone 2: Optimal moisture for OptimalConditions
echo "{\"zone_id\":\"zone_2\",\"moisture_level\":50.0,\"timestamp\":$T2}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic soil-sensors \
    --bootstrap-server localhost:9092

# Zone 3: Very low moisture for DroughtStress
echo "{\"zone_id\":\"zone_3\",\"moisture_level\":15.0,\"timestamp\":$T3}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic soil-sensors \
    --bootstrap-server localhost:9092

# Zone 4: Perfect for OptimalHarvestConditions
echo "{\"zone_id\":\"zone_4\",\"moisture_level\":48.0,\"timestamp\":$T4}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic soil-sensors \
    --bootstrap-server localhost:9092

# Temperature events
echo ""
echo "2. Producing temperature events..."
# Zone 1: High temperature to trigger CriticalIrrigationNeeded (>30°C)
echo "{\"zone_id\":\"zone_1\",\"temperature\":33.0,\"sensor_type\":\"soil\",\"timestamp\":$((T1 + 100))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic temperature \
    --bootstrap-server localhost:9092

# Zone 2: Optimal temperature for OptimalConditions (22-28°C)
echo "{\"zone_id\":\"zone_2\",\"temperature\":25.0,\"sensor_type\":\"soil\",\"timestamp\":$((T2 + 100))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic temperature \
    --bootstrap-server localhost:9092

# Zone 3: High temp for DroughtStress (>32°C)
echo "{\"zone_id\":\"zone_3\",\"temperature\":35.0,\"sensor_type\":\"air\",\"timestamp\":$((T3 + 100))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic temperature \
    --bootstrap-server localhost:9092

# Zone 4: Perfect for OptimalHarvestConditions (20-25°C)
echo "{\"zone_id\":\"zone_4\",\"temperature\":22.0,\"sensor_type\":\"air\",\"timestamp\":$((T4 + 100))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic temperature \
    --bootstrap-server localhost:9092

# Zone 5: Below freezing for FrostAlert (<0°C)
echo "{\"zone_id\":\"zone_5\",\"temperature\":-3.0,\"sensor_type\":\"air\",\"timestamp\":$((T5 + 100))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic temperature \
    --bootstrap-server localhost:9092

# Irrigation events
echo ""
echo "3. Producing irrigation events..."
# Zone 1: Start irrigation
echo "{\"zone_id\":\"zone_1\",\"action\":\"start\",\"timestamp\":$((T1 + 200))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic irrigation \
    --bootstrap-server localhost:9092

# Zone 1: Stop irrigation (for IrrigationEfficiency rule)
echo "{\"zone_id\":\"zone_1\",\"action\":\"stop\",\"water_volume_ml\":45000,\"timestamp\":$((T1 + 500))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic irrigation \
    --bootstrap-server localhost:9092

# Zone 2: Stop irrigation event (moisture was <40 before stop)
echo "{\"zone_id\":\"zone_2\",\"action\":\"stop\",\"water_volume_ml\":30000,\"timestamp\":$((T2 + 200))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic irrigation \
    --bootstrap-server localhost:9092

# Weather events
echo ""
echo "4. Producing weather events..."
# Zone 5 has frost condition to trigger FrostAlert
echo "{\"location\":\"farm\",\"zone_id\":\"zone_5\",\"condition\":\"frost\",\"temperature\":-1.0,\"timestamp\":$((T5 + 200))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic weather \
    --bootstrap-server localhost:9092

# Zone 4 has clear weather for OptimalHarvestConditions
echo "{\"location\":\"farm\",\"zone_id\":\"zone_4\",\"condition\":\"clear\",\"temperature\":22.0,\"timestamp\":$((T4 + 200))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic weather \
    --bootstrap-server localhost:9092

# Zone 3 has heatwave for ExtremeWeatherIrrigation
echo "{\"location\":\"farm\",\"zone_id\":\"zone_3\",\"condition\":\"heatwave\",\"temperature\":38.0,\"timestamp\":$((T3 + 200))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic weather \
    --bootstrap-server localhost:9092

# Zone 2 has rain for RainDetectedSkipIrrigation
echo "{\"location\":\"farm\",\"zone_id\":\"zone_2\",\"condition\":\"rain\",\"temperature\":20.0,\"timestamp\":$((T2 + 300))}" | \
  docker exec -i kafka /usr/bin/kafka-console-producer \
    --topic weather \
    --bootstrap-server localhost:9092

echo ""
echo "✅ Sample events produced!"
echo ""
echo "Expected rule matches:"
echo "  - Zone 1: 🚰 CriticalIrrigationNeeded (moisture: 20% < 25, temp: 33°C > 30)"
echo "  - Zone 1: 💧 IrrigationEfficiency (stop irrigation, moisture < 40)"
echo "  - Zone 2: ✅ OptimalConditions (moisture: 50%, temp: 25°C)"
echo "  - Zone 2: 🌧️  RainDetectedSkipIrrigation (rain + irrigation stop)"
echo "  - Zone 3: 🔥 DroughtStress (moisture: 15% < 20, temp: 35°C > 32)"
echo "  - Zone 3: 🌡️  ExtremeWeatherIrrigation (low moisture + heatwave)"
echo "  - Zone 4: 🌾 OptimalHarvestConditions (moisture: 48%, temp: 22°C, clear)"
echo "  - Zone 5: ❄️  FrostAlert (temp: -3°C < 0, weather: frost)"
echo ""
echo "Check the consumer logs to see the results!"

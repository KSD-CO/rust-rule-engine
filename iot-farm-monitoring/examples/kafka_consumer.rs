/// Kafka Consumer Example
///
/// This example demonstrates how to consume sensor events from Kafka topics
/// and process them through the farm monitoring system.
///
/// Prerequisites:
/// 1. Start Kafka and Zookeeper: docker-compose up -d
/// 2. Create topics (see README.md)
/// 3. Run this example: cargo run --example kafka_consumer

use anyhow::Result;
use iot_farm_monitoring::{kafka::KafkaFarmConsumer, Config, FarmMonitor};
use log::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚜 IoT Farm Monitoring - Kafka Consumer Example");
    info!("================================================\n");

    // Create configuration
    let config = Config::default();

    // Create farm monitor
    let mut monitor = FarmMonitor::new(config.clone());

    // Register stream statistics for optimization
    monitor.register_stream_stats();

    // Register all use cases
    info!("Registering monitoring use cases...");
    monitor.register_irrigation_control();
    monitor.register_frost_alert();
    monitor.register_efficiency_analysis();
    monitor.register_anomaly_detection();

    info!("\n📊 Optimization Analysis:");
    monitor.print_optimization_analysis();

    info!("\n🔌 Connecting to Kafka...");
    info!("  Brokers: {}", config.kafka.brokers);
    info!("  Topics: {:?}", config.kafka.topics);

    // Create Kafka consumer
    let mut consumer = match KafkaFarmConsumer::new(config.kafka, monitor).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create Kafka consumer: {}", e);
            eprintln!("\nMake sure Kafka is running:");
            eprintln!("  docker-compose up -d");
            return Err(e);
        }
    };

    info!("\n✅ Consumer is ready! Waiting for events...");
    info!("   Press Ctrl+C to stop\n");

    // Print instructions for producing test events
    info!("💡 To produce test events, open another terminal and run:");
    info!("   docker exec -it kafka kafka-console-producer.sh \\");
    info!("     --topic soil-sensors --bootstrap-server localhost:9092\n");
    info!("   Then paste this JSON:");
    info!(r#"   {{"zone_id":"zone_1","moisture_level":25.0,"timestamp":1000}}"#);
    info!("\n   Or use the producer script: ./scripts/produce_events.sh\n");

    // Start consuming (runs indefinitely)
    consumer.start_consuming().await?;

    Ok(())
}

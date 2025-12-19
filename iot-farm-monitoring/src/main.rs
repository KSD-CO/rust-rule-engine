use anyhow::Result;
use iot_farm_monitoring::{kafka::KafkaFarmConsumer, Config, FarmMonitor};
use log::info;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("🚜 IoT Farm Monitoring System");
    info!("==============================\n");

    // Load configuration
    let config_path = env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    let config = Config::from_file(&config_path).unwrap_or_else(|e| {
        info!("Failed to load config file: {}. Using defaults.", e);
        Config::default()
    });

    // Create farm monitor
    let mut monitor = FarmMonitor::new(config.clone());

    // Register stream statistics
    monitor.register_stream_stats();

    // Register all use cases
    info!("Registering use cases...");
    monitor.register_irrigation_control();
    monitor.register_predictive_irrigation(); // New sophisticated rule
    monitor.register_frost_alert();
    monitor.register_efficiency_analysis();
    monitor.register_anomaly_detection();

    // Print optimization analysis
    monitor.print_optimization_analysis();

    info!("\n🔌 Connecting to Kafka...");

    // Create and start Kafka consumer
    let mut consumer = KafkaFarmConsumer::new(config.kafka, monitor).await?;

    info!("\n✅ Farm monitoring system is running!");
    info!("Listening for events from Kafka topics...\n");

    // Clone monitor for statistics thread
    let stats_handle = {
        let monitor_stats = consumer.get_monitor_stats();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let stats = monitor_stats.lock().unwrap();
                info!("\n📊 === STATISTICS REPORT ===");
                info!("Events Processed: {}", stats.events_processed);
                info!("🚰 Irrigation Triggered: {}", stats.irrigation_triggered);
                info!("❄️  Frost Alerts: {}", stats.frost_alerts);
                info!("📈 Efficiency Reports: {}", stats.efficiency_reports);
                info!("⚠️  Anomalies Detected: {}", stats.anomalies_detected);
                info!("==========================\n");
            }
        })
    };

    // Start consuming (this will run indefinitely)
    let consume_result = consumer.start_consuming().await;

    // Clean up
    stats_handle.abort();
    
    consume_result
}

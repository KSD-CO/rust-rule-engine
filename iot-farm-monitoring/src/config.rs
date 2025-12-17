use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub kafka: KafkaConfig,
    pub monitoring: MonitoringConfig,
    pub optimization: OptimizationConfig,
    pub logging: LoggingConfig,
    pub alerting: Option<AlertingConfig>,
    pub database: Option<DatabaseConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub group_id: String,
    pub topics: Vec<String>,
    #[serde(default = "default_auto_offset_reset")]
    pub auto_offset_reset: String,
    #[serde(default = "default_true")]
    pub enable_auto_commit: bool,
    #[serde(default = "default_session_timeout")]
    pub session_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub irrigation_moisture_threshold: f64,
    pub irrigation_temp_threshold: f64,
    pub frost_alert_temperature: f64,
    pub time_window_seconds: u64,
    #[serde(default = "default_zero")]
    pub anomaly_moisture_min: f64,
    #[serde(default = "default_hundred")]
    pub anomaly_moisture_max: f64,
    #[serde(default = "default_minus_twenty")]
    pub anomaly_temp_min: f64,
    #[serde(default = "default_sixty")]
    pub anomaly_temp_max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    #[serde(default = "default_true")]
    pub enable_partitioning: bool,
    #[serde(default = "default_true")]
    pub enable_bloom_filter: bool,
    #[serde(default = "default_true")]
    pub enable_indexing: bool,
    #[serde(default = "default_max_memory")]
    pub max_memory_mb: usize,
    #[serde(default = "default_true")]
    pub collect_stats: bool,
    #[serde(default = "default_stats_interval")]
    pub stats_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    pub smtp_server: Option<String>,
    pub smtp_port: Option<u16>,
    pub from_email: Option<String>,
    pub to_emails: Option<Vec<String>>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default)]
    pub enabled: bool,
    pub connection_string: Option<String>,
}

// Default value functions
fn default_auto_offset_reset() -> String {
    "earliest".to_string()
}
fn default_true() -> bool {
    true
}
fn default_session_timeout() -> u64 {
    6000
}
fn default_zero() -> f64 {
    0.0
}
fn default_hundred() -> f64 {
    100.0
}
fn default_minus_twenty() -> f64 {
    -20.0
}
fn default_sixty() -> f64 {
    60.0
}
fn default_max_memory() -> usize {
    100
}
fn default_stats_interval() -> u64 {
    60
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_format() -> String {
    "json".to_string()
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Create default configuration
    pub fn default() -> Self {
        Config {
            kafka: KafkaConfig {
                brokers: "localhost:9092".to_string(),
                group_id: "farm-monitor-group".to_string(),
                topics: vec![
                    "soil-sensors".to_string(),
                    "temperature".to_string(),
                    "irrigation".to_string(),
                    "weather".to_string(),
                ],
                auto_offset_reset: "earliest".to_string(),
                enable_auto_commit: true,
                session_timeout_ms: 6000,
            },
            monitoring: MonitoringConfig {
                irrigation_moisture_threshold: 30.0,
                irrigation_temp_threshold: 25.0,
                frost_alert_temperature: 2.0,
                time_window_seconds: 600,
                anomaly_moisture_min: 0.0,
                anomaly_moisture_max: 100.0,
                anomaly_temp_min: -20.0,
                anomaly_temp_max: 60.0,
            },
            optimization: OptimizationConfig {
                enable_partitioning: true,
                enable_bloom_filter: true,
                enable_indexing: true,
                max_memory_mb: 100,
                collect_stats: true,
                stats_interval_seconds: 60,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
            alerting: None,
            database: None,
        }
    }
}

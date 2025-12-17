use crate::config::Config;
use crate::events::*;
use log::{info, warn};
use rust_rule_engine::rete::stream_join_node::{JoinStrategy, JoinType, StreamJoinNode};
use rust_rule_engine::streaming::event::StreamEvent;
use rust_rule_engine::streaming::join_manager::StreamJoinManager;
use rust_rule_engine::streaming::join_optimizer::{JoinOptimizer, StreamStats};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Main farm monitoring system
pub struct FarmMonitor {
    config: Config,
    join_manager: StreamJoinManager,
    optimizer: JoinOptimizer,
    stats: Arc<Mutex<MonitorStats>>,
}

#[derive(Debug, Default)]
pub struct MonitorStats {
    pub irrigation_triggered: u64,
    pub frost_alerts: u64,
    pub efficiency_reports: u64,
    pub anomalies_detected: u64,
    pub events_processed: u64,
}

impl FarmMonitor {
    /// Create a new farm monitor with configuration
    pub fn new(config: Config) -> Self {
        Self {
            config,
            join_manager: StreamJoinManager::new(),
            optimizer: JoinOptimizer::new(),
            stats: Arc::new(Mutex::new(MonitorStats::default())),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(Config::default())
    }

    /// Register stream statistics for optimization
    pub fn register_stream_stats(&mut self) {
        // Soil sensors: 100 sensors @ 0.1 Hz = 10 events/sec
        self.optimizer.register_stream_stats(StreamStats {
            stream_name: "soil-sensors".to_string(),
            estimated_event_rate: 10.0,
            estimated_cardinality: 100,
            average_event_size: 200,
        });

        // Temperature sensors: 100 sensors @ 0.2 Hz = 20 events/sec
        self.optimizer.register_stream_stats(StreamStats {
            stream_name: "temperature".to_string(),
            estimated_event_rate: 20.0,
            estimated_cardinality: 100,
            average_event_size: 200,
        });

        // Irrigation events: ~1 event/minute
        self.optimizer.register_stream_stats(StreamStats {
            stream_name: "irrigation".to_string(),
            estimated_event_rate: 0.017,
            estimated_cardinality: 100,
            average_event_size: 150,
        });

        // Weather station: 1 station @ 0.05 Hz
        self.optimizer.register_stream_stats(StreamStats {
            stream_name: "weather".to_string(),
            estimated_event_rate: 0.05,
            estimated_cardinality: 1,
            average_event_size: 180,
        });
    }

    /// Register automatic irrigation control use case
    pub fn register_irrigation_control(&mut self) {
        info!("Registering irrigation control use case");

        let stats = self.stats.clone();
        let moisture_threshold = self.config.monitoring.irrigation_moisture_threshold;
        let temp_threshold = self.config.monitoring.irrigation_temp_threshold;
        let time_window = Duration::from_secs(self.config.monitoring.time_window_seconds);

        let join = StreamJoinNode::new(
            "soil-sensors".to_string(),
            "temperature".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: time_window,
            },
            Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
            Box::new(move |soil, temp| {
                // Temperature reading must come after or within window of soil reading
                temp.metadata.timestamp >= soil.metadata.timestamp
            }),
        );

        self.join_manager.register_join(
            "irrigation_control".to_string(),
            join,
            Box::new(move |joined| {
                if let (Some(soil), Some(temp)) = (&joined.left, &joined.right) {
                    if let (Some(soil_data), Some(temp_data)) =
                        (parse_soil_sensor(soil), parse_temperature(temp))
                    {
                        // Check thresholds
                        if soil_data.moisture_level < moisture_threshold
                            && temp_data.temperature > temp_threshold
                        {
                            info!(
                                "🚰 IRRIGATION TRIGGERED for zone {} - Moisture: {:.1}%, Temp: {:.1}°C",
                                soil_data.zone_id, soil_data.moisture_level, temp_data.temperature
                            );

                            let mut stats_lock = stats.lock().unwrap();
                            stats_lock.irrigation_triggered += 1;

                            // In production: Trigger actual irrigation system
                            // irrigation_api.start_watering(&soil_data.zone_id);
                        }
                    }
                }
            }),
        );
    }

    /// Register frost alert system use case
    pub fn register_frost_alert(&mut self) {
        info!("Registering frost alert system");

        let stats = self.stats.clone();
        let frost_threshold = self.config.monitoring.frost_alert_temperature;
        let time_window = Duration::from_secs(self.config.monitoring.time_window_seconds);

        let join = StreamJoinNode::new(
            "temperature".to_string(),
            "weather".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: time_window,
            },
            Box::new(|e| {
                // Extract location from zone_id (e.g., "zone_1" -> "farm")
                e.data
                    .get("zone_id")
                    .and_then(|v| v.as_string())
                    .map(|_| "farm".to_string())
            }),
            Box::new(|e| e.data.get("location").and_then(|v| v.as_string())),
            Box::new(|_temp, _weather| true),
        );

        self.join_manager.register_join(
            "frost_alert".to_string(),
            join,
            Box::new(move |joined| {
                if let (Some(temp), Some(weather)) = (&joined.left, &joined.right) {
                    if let (Some(temp_data), Some(weather_data)) =
                        (parse_temperature(temp), parse_weather(weather))
                    {
                        if temp_data.temperature < frost_threshold
                            && weather_data.condition.contains("frost")
                        {
                            warn!(
                                "❄️ FROST ALERT for zone {} - Temp: {:.1}°C, Weather: {}",
                                temp_data.zone_id, temp_data.temperature, weather_data.condition
                            );

                            let mut stats_lock = stats.lock().unwrap();
                            stats_lock.frost_alerts += 1;

                            // In production: Send alert notifications
                            // alert_system.send_frost_alert(&temp_data.zone_id);
                        }
                    }
                }
            }),
        );
    }

    /// Register irrigation efficiency analysis use case
    pub fn register_efficiency_analysis(&mut self) {
        info!("Registering irrigation efficiency analysis");

        let stats = self.stats.clone();
        let time_window = Duration::from_secs(self.config.monitoring.time_window_seconds);

        let join = StreamJoinNode::new(
            "irrigation".to_string(),
            "soil-sensors".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: time_window,
            },
            Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
            Box::new(|irrigation, soil| {
                // Soil reading must come AFTER irrigation event
                soil.metadata.timestamp > irrigation.metadata.timestamp
            }),
        );

        self.join_manager.register_join(
            "efficiency_analysis".to_string(),
            join,
            Box::new(move |joined| {
                if let (Some(irrigation), Some(soil)) = (&joined.left, &joined.right) {
                    if let (Some(irr_data), Some(soil_data)) =
                        (parse_irrigation(irrigation), parse_soil_sensor(soil))
                    {
                        if irr_data.action == "stop" {
                            let time_delta =
                                (soil_data.timestamp - irr_data.timestamp) / 60; // minutes

                            info!(
                                "📊 EFFICIENCY REPORT for zone {} - Moisture after {} min: {:.1}%",
                                irr_data.zone_id, time_delta, soil_data.moisture_level
                            );

                            let mut stats_lock = stats.lock().unwrap();
                            stats_lock.efficiency_reports += 1;

                            // In production: Store analytics data
                            // analytics_db.store_efficiency_data(&irr_data, &soil_data);
                        }
                    }
                }
            }),
        );
    }

    /// Register sensor anomaly detection use case
    pub fn register_anomaly_detection(&mut self) {
        info!("Registering sensor anomaly detection");

        let _stats = self.stats.clone();
        let min_moisture = self.config.monitoring.anomaly_moisture_min;
        let max_moisture = self.config.monitoring.anomaly_moisture_max;
        let min_temp = self.config.monitoring.anomaly_temp_min;
        let max_temp = self.config.monitoring.anomaly_temp_max;

        // For anomaly detection, we'll check values directly on event processing
        // In a more complex system, this could use left outer joins to detect missing sensors
        let check_soil_anomaly = move |event: &StreamEvent| {
            if let Some(soil_data) = parse_soil_sensor(event) {
                if soil_data.moisture_level < min_moisture
                    || soil_data.moisture_level > max_moisture
                {
                    warn!(
                        "⚠️ ANOMALY DETECTED - Soil sensor in zone {} reports invalid moisture: {:.1}%",
                        soil_data.zone_id, soil_data.moisture_level
                    );
                    return true;
                }
            }
            false
        };

        let check_temp_anomaly = move |event: &StreamEvent| {
            if let Some(temp_data) = parse_temperature(event) {
                if temp_data.temperature < min_temp || temp_data.temperature > max_temp {
                    warn!(
                        "⚠️ ANOMALY DETECTED - Temperature sensor in zone {} reports invalid temp: {:.1}°C",
                        temp_data.zone_id, temp_data.temperature
                    );
                    return true;
                }
            }
            false
        };

        // Store closures for later use in process_event
        // (In a real implementation, we'd have a proper anomaly detection system)
        let _ = (check_soil_anomaly, check_temp_anomaly);
    }

    /// Process an incoming stream event
    pub fn process_event(&mut self, event: StreamEvent) {
        let mut stats = self.stats.lock().unwrap();
        stats.events_processed += 1;
        drop(stats);

        self.join_manager.process_event(event);
    }

    /// Get monitoring statistics
    pub fn get_stats(&self) -> MonitorStats {
        let stats = self.stats.lock().unwrap();
        MonitorStats {
            irrigation_triggered: stats.irrigation_triggered,
            frost_alerts: stats.frost_alerts,
            efficiency_reports: stats.efficiency_reports,
            anomalies_detected: stats.anomalies_detected,
            events_processed: stats.events_processed,
        }
    }

    /// Print optimization analysis
    pub fn print_optimization_analysis(&self) {
        info!("=== Stream Join Optimization Analysis ===");

        let plan = self.optimizer.optimize_join(
            "soil-sensors",
            "temperature",
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(self.config.monitoring.time_window_seconds),
            },
        );

        info!("Irrigation Control Join:");
        info!("  Estimated Cost: {:.2}", plan.estimated_cost);
        info!("  Optimizations: {}", plan.optimizations.len());
        info!("  Strategy: {}", plan.explanation);

        let memory = self.optimizer.estimate_memory_usage(
            "soil-sensors",
            "temperature",
            Duration::from_secs(self.config.monitoring.time_window_seconds),
        );
        info!("  Estimated Memory: {} MB", memory / 1_000_000);
    }
}

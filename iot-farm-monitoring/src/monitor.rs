use crate::config::Config;
use crate::stream_rule_processor::StreamRuleProcessor;
use log::{info, warn};
use rust_rule_engine::rete::IncrementalEngine;
use rust_rule_engine::streaming::event::StreamEvent;
use std::sync::{Arc, Mutex};

/// Main farm monitoring system
pub struct FarmMonitor {
    #[allow(dead_code)]
    config: Config,
    stats: Arc<Mutex<MonitorStats>>,
    #[allow(dead_code)]
    grl_engine: Arc<Mutex<IncrementalEngine>>,
    stream_processor: Option<StreamRuleProcessor>,
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
        let mut grl_engine = IncrementalEngine::new();
        
        // Load GRL rules (multi-stream approach with declarative stream joins)
        let grl_path = "grl_rules/farm_monitoring_multistream.grl";
        match rust_rule_engine::rete::GrlReteLoader::load_from_file(grl_path, &mut grl_engine) {
            Ok(rules_count) => {
                info!("✅ Loaded {} GRL rules from {}", rules_count, grl_path);
                info!("   📋 Multi-stream rules with declarative joins");
            }
            Err(e) => {
                warn!("⚠️ Failed to load GRL rules from {}: {}", grl_path, e);
                warn!("   Continuing with Rust-only logging...");
            }
        }

        // Try to create stream processor with beta node support
        let stats = Arc::new(Mutex::new(MonitorStats::default()));
        let stream_processor = match StreamRuleProcessor::from_grl_file(grl_path) {
            Ok(mut processor) => {
                processor.set_stats(stats.clone());
                info!("✅ Created stream processor (with Beta nodes) from GRL rules");
                Some(processor)
            }
            Err(e) => {
                warn!("⚠️ Failed to create stream processor: {}", e);
                None
            }
        };
        
        Self {
            config,
            stats,
            grl_engine: Arc::new(Mutex::new(grl_engine)),
            stream_processor,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(Config::default())
    }

    /// Register stream statistics for optimization (deprecated - GRL handles this now)
    pub fn register_stream_stats(&mut self) {
        info!("📊 Stream stats registration - now handled by GRL engine");
        // GRL engine with native stream support handles optimization internally
    }

    /// Register automatic irrigation control use case (deprecated - now in GRL)
    pub fn register_irrigation_control(&mut self) {
        info!("✅ Irrigation control - handled by GRL rule 'CriticalIrrigationNeeded'");
        // Rule defined in farm_monitoring_multistream.grl with native stream joins
    }

    /// Register frost alert system use case (deprecated - now in GRL)
    pub fn register_frost_alert(&mut self) {
        info!("✅ Frost alert - handled by GRL rule 'FrostAlert'");
        // Rule defined in farm_monitoring_multistream.grl
    }

    /// Register irrigation efficiency analysis use case (deprecated - now in GRL)
    pub fn register_efficiency_analysis(&mut self) {
        info!("✅ Efficiency analysis - handled by GRL rule 'IrrigationEfficiency'");
        // Rule defined in farm_monitoring_multistream.grl
    }

    /// Register sensor anomaly detection use case (deprecated - now in GRL)
    pub fn register_anomaly_detection(&mut self) {
        info!("✅ Anomaly detection - can be added to GRL rules");
        // Can be implemented in GRL with threshold checks
    }

    /// Register predictive irrigation rule (deprecated - now in GRL)
    pub fn register_predictive_irrigation(&mut self) {
        info!("✅ Predictive irrigation - handled by GRL rules 'DroughtStress' and 'ExtremeWeatherIrrigation'");
        // Rules defined in farm_monitoring_multistream.grl with multi-stream joins
    }

    /// Process an incoming stream event through GRL stream processor
    pub fn process_event(&mut self, event: StreamEvent) {
        let event_count = {
            let mut stats = self.stats.lock().unwrap();
            stats.events_processed += 1;
            stats.events_processed
        };

        // Process through stream processor if available
        if let Some(processor) = &mut self.stream_processor {
            // Use metadata.source as stream name (e.g., "soil-sensors", "temperature")
            let stream_name = &event.metadata.source;
            
            // Debug: log every 10th event
            if event_count % 10 == 0 {
                info!("📍 Processing event #{}: stream={}, type={}, data_keys={:?}", 
                    event_count, stream_name, event.event_type, 
                    event.data.keys().collect::<Vec<_>>());
            }
            
            let processed = processor.process_event(stream_name, &event);
            
            if !processed && event_count % 20 == 0 {
                info!("   ⏭️ Event not matched by any stream rules");
            }
        } else {
            // Fallback: just log
            info!(
                "⚠️ No stream processor - event not processed: id={}, type={}",
                event.id, event.event_type
            );
        }
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

    /// Get reference to statistics for external monitoring
    pub fn get_stats_ref(&self) -> Arc<Mutex<MonitorStats>> {
        self.stats.clone()
    }

    /// Print optimization analysis (deprecated - GRL engine handles optimization)
    pub fn print_optimization_analysis(&self) {
        info!("=== GRL Stream Engine Configuration ===");
        info!("✅ Native stream joins configured in GRL rules");
        info!("✅ Alpha nodes: Filter + Window per stream");
        info!("✅ Beta nodes: Multi-stream joins with time windows");
        info!("✅ RETE network: Incremental rule propagation");
        info!("========================================");
    }
}

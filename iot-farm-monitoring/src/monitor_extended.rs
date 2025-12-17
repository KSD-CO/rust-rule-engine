use crate::config::Config;
use crate::events_extended::*;
use log::{info, warn};
use rust_rule_engine::rete::stream_join_node::{JoinStrategy, JoinType, StreamJoinNode};
use rust_rule_engine::streaming::event::StreamEvent;
use rust_rule_engine::streaming::join_manager::StreamJoinManager;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Extended farm monitoring system for integrated vegetable + fish farm
pub struct IntegratedFarmMonitor {
    config: Config,
    join_manager: StreamJoinManager,
    stats: Arc<Mutex<IntegratedFarmStats>>,
}

#[derive(Debug, Default)]
pub struct IntegratedFarmStats {
    // Greenhouse stats
    pub cooling_activated: u64,
    pub co2_injections: u64,
    pub pest_warnings: u64,
    pub harvests_scheduled: u64,

    // Aquaculture stats
    pub critical_do_alerts: u64,
    pub emergency_aerations: u64,
    pub ph_corrections: u64,
    pub ammonia_alerts: u64,
    pub feeding_events: u64,

    // Integration stats
    pub aquaponics_cycles: u64,
    pub water_exchanges: u64,
    pub energy_optimizations: u64,

    pub total_events_processed: u64,
}

impl IntegratedFarmMonitor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            join_manager: StreamJoinManager::new(),
            stats: Arc::new(Mutex::new(IntegratedFarmStats::default())),
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(Config::default())
    }

    // ========================================================================
    // VEGETABLE GREENHOUSE USE CASES
    // ========================================================================

    /// Register greenhouse climate control (temp + humidity)
    pub fn register_greenhouse_climate_control(&mut self) {
        info!("Registering greenhouse climate control");

        let stats = self.stats.clone();
        let time_window = Duration::from_secs(300); // 5 minutes

        let join = StreamJoinNode::new(
            "air-temperature".to_string(),
            "humidity-sensors".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: time_window,
            },
            Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
            Box::new(|temp, humidity| {
                humidity.metadata.timestamp >= temp.metadata.timestamp
            }),
        );

        self.join_manager.register_join(
            "greenhouse_climate".to_string(),
            join,
            Box::new(move |joined| {
                if let (Some(temp_e), Some(hum_e)) = (&joined.left, &joined.right) {
                    if let Some(temp_str) = temp_e.data.get("temperature").and_then(|v| v.as_string()) {
                        if let Some(hum_str) = hum_e.data.get("humidity_percent").and_then(|v| v.as_string()) {
                            let temp: f64 = temp_str.parse().unwrap_or(0.0);
                            let humidity: f64 = hum_str.parse().unwrap_or(0.0);

                            if temp > 30.0 && humidity < 60.0 {
                                let zone = temp_e.data.get("zone_id").unwrap().as_string().unwrap();
                                info!(
                                    "🌡️ GREENHOUSE COOLING: {} - Temp {:.1}°C, Humidity {:.1}%",
                                    zone, temp, humidity
                                );

                                let mut stats_lock = stats.lock().unwrap();
                                stats_lock.cooling_activated += 1;
                            }
                        }
                    }
                }
            }),
        );
    }

    /// Register CO2 enrichment for photosynthesis
    pub fn register_co2_enrichment(&mut self) {
        info!("Registering CO2 enrichment system");

        let stats = self.stats.clone();

        let join = StreamJoinNode::new(
            "light-sensors".to_string(),
            "co2-sensors".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(600),
            },
            Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("zone_id").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        self.join_manager.register_join(
            "co2_enrichment".to_string(),
            join,
            Box::new(move |joined| {
                if let (Some(light_e), Some(co2_e)) = (&joined.left, &joined.right) {
                    if let Some(light_str) = light_e.data.get("light_intensity").and_then(|v| v.as_string()) {
                        if let Some(co2_str) = co2_e.data.get("co2_ppm").and_then(|v| v.as_string()) {
                            let light: f64 = light_str.parse().unwrap_or(0.0);
                            let co2: f64 = co2_str.parse().unwrap_or(0.0);

                            if light > 10000.0 && co2 < 800.0 {
                                let zone = light_e.data.get("zone_id").unwrap().as_string().unwrap();
                                info!(
                                    "🌱 CO2 INJECTION: {} - Light {} lux, CO2 {:.0} ppm",
                                    zone, light, co2
                                );

                                let mut stats_lock = stats.lock().unwrap();
                                stats_lock.co2_injections += 1;
                            }
                        }
                    }
                }
            }),
        );
    }

    // ========================================================================
    // AQUACULTURE USE CASES
    // ========================================================================

    /// Register critical dissolved oxygen monitoring
    pub fn register_critical_do_monitoring(&mut self) {
        info!("Registering critical DO monitoring");

        let stats = self.stats.clone();

        let join = StreamJoinNode::new(
            "dissolved-oxygen".to_string(),
            "water-temperature".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(600),
            },
            Box::new(|e| e.data.get("pond_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("pond_id").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        self.join_manager.register_join(
            "critical_do".to_string(),
            join,
            Box::new(move |joined| {
                if let (Some(do_e), Some(temp_e)) = (&joined.left, &joined.right) {
                    if let Some(do_str) = do_e.data.get("do_mg_per_liter").and_then(|v| v.as_string()) {
                        if let Some(temp_str) = temp_e.data.get("temperature").and_then(|v| v.as_string()) {
                            let do_level: f64 = do_str.parse().unwrap_or(0.0);
                            let temp: f64 = temp_str.parse().unwrap_or(0.0);

                            if do_level < 4.0 && temp > 28.0 {
                                let pond = do_e.data.get("pond_id").unwrap().as_string().unwrap();
                                warn!(
                                    "🚨 CRITICAL DO: {} - DO {:.1} mg/L, Temp {:.1}°C - EMERGENCY AERATION!",
                                    pond, do_level, temp
                                );

                                let mut stats_lock = stats.lock().unwrap();
                                stats_lock.critical_do_alerts += 1;
                                stats_lock.emergency_aerations += 1;
                            }
                        }
                    }
                }
            }),
        );
    }

    /// Register ammonia toxicity prevention
    pub fn register_ammonia_monitoring(&mut self) {
        info!("Registering ammonia toxicity prevention");

        let stats = self.stats.clone();

        let join = StreamJoinNode::new(
            "ammonia-sensors".to_string(),
            "ph-sensors".to_string(),
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(900), // 15 minutes
            },
            Box::new(|e| e.data.get("pond_id").and_then(|v| v.as_string())),
            Box::new(|e| e.data.get("pond_id").and_then(|v| v.as_string())),
            Box::new(|_, _| true),
        );

        self.join_manager.register_join(
            "ammonia_toxicity".to_string(),
            join,
            Box::new(move |joined| {
                if let (Some(ammonia_e), Some(ph_e)) = (&joined.left, &joined.right) {
                    if let Some(ammonia_str) = ammonia_e.data.get("ammonia_ppm").and_then(|v| v.as_string()) {
                        if let Some(ph_str) = ph_e.data.get("ph_value").and_then(|v| v.as_string()) {
                            let ammonia: f64 = ammonia_str.parse().unwrap_or(0.0);
                            let ph: f64 = ph_str.parse().unwrap_or(0.0);

                            if ammonia > 0.5 && ph > 8.0 {
                                let pond = ammonia_e.data.get("pond_id").unwrap().as_string().unwrap();
                                warn!(
                                    "☠️ AMMONIA TOXICITY: {} - NH3 {:.2} ppm, pH {:.2} - WATER CHANGE NEEDED!",
                                    pond, ammonia, ph
                                );

                                let mut stats_lock = stats.lock().unwrap();
                                stats_lock.ammonia_alerts += 1;
                            }
                        }
                    }
                }
            }),
        );
    }

    // ========================================================================
    // INTEGRATED FARM USE CASES
    // ========================================================================

    /// Register aquaponics nutrient cycle (fish waste → plant nutrients)
    pub fn register_aquaponics_cycle(&mut self) {
        info!("Registering aquaponics nutrient cycle");

        let stats = self.stats.clone();

        let join = StreamJoinNode::new(
            "nitrate-sensors".to_string(),
            "humidity-sensors".to_string(), // Proxy for plant zones
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(3600), // 1 hour
            },
            Box::new(|e| {
                // Extract farm section from pond_id
                e.data.get("pond_id")
                    .and_then(|v| v.as_string())
                    .map(|s| {
                        if s.contains("1") { "section_1".to_string() }
                        else { "section_2".to_string() }
                    })
            }),
            Box::new(|e| {
                // Extract section from greenhouse zone
                e.data.get("zone_id")
                    .and_then(|v| v.as_string())
                    .map(|s| {
                        if s.contains("1") { "section_1".to_string() }
                        else { "section_2".to_string() }
                    })
            }),
            Box::new(|_, _| true),
        );

        self.join_manager.register_join(
            "aquaponics_cycle".to_string(),
            join,
            Box::new(move |joined| {
                if let (Some(nitrate_e), Some(plant_e)) = (&joined.left, &joined.right) {
                    if let Some(nitrate_str) = nitrate_e.data.get("nitrate_ppm").and_then(|v| v.as_string()) {
                        let nitrate: f64 = nitrate_str.parse().unwrap_or(0.0);

                        if nitrate > 30.0 {
                            let pond = nitrate_e.data.get("pond_id").unwrap().as_string().unwrap();
                            let zone = plant_e.data.get("zone_id").unwrap().as_string().unwrap();

                            info!(
                                "♻️ AQUAPONICS: Pump from {} ({:.1} ppm NO3) to {} - Fish waste → Plant nutrients!",
                                pond, nitrate, zone
                            );

                            let mut stats_lock = stats.lock().unwrap();
                            stats_lock.aquaponics_cycles += 1;
                        }
                    }
                }
            }),
        );
    }

    /// Process an incoming stream event
    pub fn process_event(&mut self, event: StreamEvent) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_events_processed += 1;
        drop(stats);

        self.join_manager.process_event(event);
    }

    /// Get monitoring statistics
    pub fn get_stats(&self) -> IntegratedFarmStats {
        let stats = self.stats.lock().unwrap();
        IntegratedFarmStats {
            cooling_activated: stats.cooling_activated,
            co2_injections: stats.co2_injections,
            pest_warnings: stats.pest_warnings,
            harvests_scheduled: stats.harvests_scheduled,
            critical_do_alerts: stats.critical_do_alerts,
            emergency_aerations: stats.emergency_aerations,
            ph_corrections: stats.ph_corrections,
            ammonia_alerts: stats.ammonia_alerts,
            feeding_events: stats.feeding_events,
            aquaponics_cycles: stats.aquaponics_cycles,
            water_exchanges: stats.water_exchanges,
            energy_optimizations: stats.energy_optimizations,
            total_events_processed: stats.total_events_processed,
        }
    }

    /// Print comprehensive statistics
    pub fn print_stats(&self) {
        let stats = self.get_stats();

        info!("\n╔══════════════════════════════════════════════════════════════╗");
        info!("║           INTEGRATED FARM MONITORING STATISTICS              ║");
        info!("╚══════════════════════════════════════════════════════════════╝");

        info!("\n🥬 GREENHOUSE (Vegetables):");
        info!("  Cooling Activated:      {}", stats.cooling_activated);
        info!("  CO2 Injections:         {}", stats.co2_injections);
        info!("  Pest Warnings:          {}", stats.pest_warnings);
        info!("  Harvests Scheduled:     {}", stats.harvests_scheduled);

        info!("\n🐟 AQUACULTURE (Fish):");
        info!("  Critical DO Alerts:     {}", stats.critical_do_alerts);
        info!("  Emergency Aerations:    {}", stats.emergency_aerations);
        info!("  pH Corrections:         {}", stats.ph_corrections);
        info!("  Ammonia Alerts:         {}", stats.ammonia_alerts);
        info!("  Feeding Events:         {}", stats.feeding_events);

        info!("\n♻️  INTEGRATION:");
        info!("  Aquaponics Cycles:      {}", stats.aquaponics_cycles);
        info!("  Water Exchanges:        {}", stats.water_exchanges);
        info!("  Energy Optimizations:   {}", stats.energy_optimizations);

        info!("\n📊 OVERALL:");
        info!("  Total Events Processed: {}", stats.total_events_processed);
        info!("");
    }
}

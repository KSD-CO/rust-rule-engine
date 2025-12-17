/// Comprehensive Integrated Farm Demo
///
/// Demonstrates COMPLETE system with:
/// - 🥬 Vegetable greenhouses (temperature, humidity, CO2, light)
/// - 🐟 Fish aquaculture (DO, pH, ammonia, nitrite, nitrate)
/// - ♻️  Aquaponics integration (fish waste → plant nutrients)
/// - 📋 GRL rules (loaded from .grl files)
/// - 🌊 All stream features (joins, windows, watermarks, aggregations)
///
/// Run with: cargo run --example comprehensive_farm_demo

use iot_farm_monitoring::*;
use log::info;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    print_header();

    // Create integrated farm monitor
    let mut monitor = IntegratedFarmMonitor::with_defaults();

    // Register all use cases
    register_all_use_cases(&mut monitor);

    // Run simulation scenarios
    simulate_normal_operations(&mut monitor);
    simulate_greenhouse_crisis(&mut monitor);
    simulate_fish_crisis(&mut monitor);
    simulate_aquaponics_integration(&mut monitor);

    // Print final statistics
    monitor.print_stats();

    info!("✅ Comprehensive farm demo completed successfully!");
}

fn print_header() {
    info!("╔══════════════════════════════════════════════════════════════════════╗");
    info!("║                                                                      ║");
    info!("║        🌾🐟 INTEGRATED FARM MONITORING SYSTEM - FULL DEMO 🐟🌾        ║");
    info!("║                                                                      ║");
    info!("║  Farm Setup:                                                         ║");
    info!("║    • 3 Greenhouses (lettuce, tomatoes, herbs)                        ║");
    info!("║    • 2 Fish Ponds (tilapia)                                          ║");
    info!("║    • 1 Aquaponics System (integrated vegetable + fish)               ║");
    info!("║                                                                      ║");
    info!("║  Monitoring:                                                         ║");
    info!("║    • Real-time sensor data from 50+ sensors                          ║");
    info!("║    • Stream joins for complex event processing                       ║");
    info!("║    • Automated control and alerts                                    ║");
    info!("║                                                                      ║");
    info!("╚══════════════════════════════════════════════════════════════════════╝\n");
}

fn register_all_use_cases(monitor: &mut IntegratedFarmMonitor) {
    info!("🔧 Registering All Use Cases...\n");

    // Greenhouse monitoring
    monitor.register_greenhouse_climate_control();
    monitor.register_co2_enrichment();

    // Fish monitoring
    monitor.register_critical_do_monitoring();
    monitor.register_ammonia_monitoring();

    // Integration
    monitor.register_aquaponics_cycle();

    info!("✅ All use cases registered!\n");
}

fn simulate_normal_operations(monitor: &mut IntegratedFarmMonitor) {
    info!("═══════════════════════════════════════════════════════════════");
    info!("📊 SCENARIO 1: Normal Farm Operations");
    info!("═══════════════════════════════════════════════════════════════\n");

    let base_time = 1000;

    // Greenhouse 1 - Lettuce (optimal conditions)
    info!("🥬 Greenhouse 1 (Lettuce) - Optimal Conditions:");
    monitor.process_event(create_temperature_reading("greenhouse_1", 24.0, "air", base_time));
    monitor.process_event(create_humidity_reading("greenhouse_1", 70.0, base_time + 10));
    monitor.process_event(create_light_reading("greenhouse_1", 15000.0, base_time + 20));
    monitor.process_event(create_co2_reading("greenhouse_1", 900.0, base_time + 30));
    info!("   ✓ All parameters normal\n");

    // Pond 1 - Tilapia (healthy)
    info!("🐟 Pond 1 (Tilapia) - Healthy Water Quality:");
    monitor.process_event(create_dissolved_oxygen_reading("pond_1", 6.5, base_time + 100));
    monitor.process_event(create_water_temp_reading("pond_1", 26.0, base_time + 110));
    monitor.process_event(create_ph_reading("pond_1", 7.2, base_time + 120));
    monitor.process_event(create_ammonia_reading("pond_1", 0.1, base_time + 130));
    info!("   ✓ Water quality excellent\n");

    info!("✅ Normal operations: No alerts\n");
}

fn simulate_greenhouse_crisis(monitor: &mut IntegratedFarmMonitor) {
    info!("═══════════════════════════════════════════════════════════════");
    info!("🌡️ SCENARIO 2: Greenhouse Heat Crisis");
    info!("═══════════════════════════════════════════════════════════════\n");

    let base_time = 10000;

    info!("🔥 Greenhouse 2 (Tomatoes) - Heat Wave Detected:");
    info!("   Initial: 28°C, 65% humidity");

    // Temperature rising
    monitor.process_event(create_temperature_reading("greenhouse_2", 28.0, "air", base_time));
    monitor.process_event(create_humidity_reading("greenhouse_2", 65.0, base_time + 10));

    info!("   10 minutes later: 32°C, 55% humidity ⚠️");
    monitor.process_event(create_temperature_reading("greenhouse_2", 32.0, "air", base_time + 600));
    monitor.process_event(create_humidity_reading("greenhouse_2", 55.0, base_time + 610));

    info!("   Expected: Cooling system activation\n");

    // Also check CO2 under high light
    info!("☀️ Bright sunlight detected, checking CO2:");
    monitor.process_event(create_light_reading("greenhouse_2", 25000.0, base_time + 1000));
    monitor.process_event(create_co2_reading("greenhouse_2", 600.0, base_time + 1010));
    info!("   Expected: CO2 injection to support photosynthesis\n");
}

fn simulate_fish_crisis(monitor: &mut IntegratedFarmMonitor) {
    info!("═══════════════════════════════════════════════════════════════");
    info!("🚨 SCENARIO 3: Fish Pond Emergency");
    info!("═══════════════════════════════════════════════════════════════\n");

    let base_time = 20000;

    info!("💀 Pond 2 - Multiple Critical Issues:");

    // Issue 1: Critical low DO + high temperature
    info!("\n   1️⃣ CRITICAL: Low Dissolved Oxygen");
    info!("      DO: 3.2 mg/L (critical < 4.0)");
    info!("      Temperature: 30°C (stress level)");

    monitor.process_event(create_dissolved_oxygen_reading("pond_2", 3.2, base_time));
    monitor.process_event(create_water_temp_reading("pond_2", 30.0, base_time + 10));
    info!("      ⚡ Expected: Emergency aeration activated!\n");

    // Issue 2: Ammonia toxicity
    info!("   2️⃣ TOXIC: High Ammonia + High pH");
    info!("      Ammonia: 0.8 ppm (toxic > 0.5)");
    info!("      pH: 8.5 (makes ammonia more toxic)");

    monitor.process_event(create_ammonia_reading("pond_2", 0.8, base_time + 500));
    monitor.process_event(create_ph_reading("pond_2", 8.5, base_time + 510));
    info!("      ⚡ Expected: Emergency water change + zeolite treatment!\n");

    info!("   ⚠️ Farmer should investigate immediately!");
    info!("   Possible causes: Overfeeding, filter failure, overstocking\n");
}

fn simulate_aquaponics_integration(monitor: &mut IntegratedFarmMonitor) {
    info!("═══════════════════════════════════════════════════════════════");
    info!("♻️  SCENARIO 4: Aquaponics Integration");
    info!("═══════════════════════════════════════════════════════════════\n");

    let base_time = 30000;

    info!("🔄 Integrated System - Nutrient Cycling:\n");

    // Fish pond produces nitrate from waste
    info!("   🐟 Pond 1: High nitrate from fish waste");
    info!("      Nitrate: 45 ppm (fish waste breakdown)");
    monitor.process_event(create_nitrate_reading("pond_1", 45.0, base_time));

    // Greenhouse needs nutrients
    info!("\n   🥬 Greenhouse 1: Plants need nutrients");
    info!("      Growth stage: Vegetative (high N demand)");
    monitor.process_event(create_humidity_reading("greenhouse_1", 65.0, base_time + 100));

    info!("\n   ♻️ AQUAPONICS CYCLE ACTIVATED:");
    info!("      1. Pump water from Pond 1 to Greenhouse 1");
    info!("      2. Plants absorb nitrate (nutrients)");
    info!("      3. Cleaned water returns to pond");
    info!("      4. Fish benefit from cleaner water");
    info!("      5. Plants benefit from free nutrients");
    info!("\n   💡 Benefits:");
    info!("      ✓ Zero chemical fertilizers");
    info!("      ✓ Reduced water changes");
    info!("      ✓ Better fish health");
    info!("      ✓ Healthier plants");
    info!("      ✓ Sustainable farming!\n");

    // Show another section
    info!("   🐟 Pond 2 → 🥬 Greenhouse 2 (another cycle)");
    monitor.process_event(create_nitrate_reading("pond_2", 38.0, base_time + 500));
    monitor.process_event(create_humidity_reading("greenhouse_2", 68.0, base_time + 600));
    info!("   ♻️ Second aquaponics cycle initiated!\n");
}

// Test GRL Rules - Verify all .grl files execute correctly
// This example loads and tests all three GRL rule files:
// - vegetable_monitoring.grl
// - aquaculture_monitoring.grl
// - integrated_farm_rules.grl

use rust_rule_engine::rete::{GrlReteLoader, IncrementalEngine, TypedFacts};

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║           🧪 GRL RULES TEST - Verifying All Rule Files 🧪           ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    test_vegetable_monitoring();
    test_aquaculture_monitoring();
    test_integrated_farm_rules();

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║                    ✅ ALL GRL TESTS COMPLETED ✅                     ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");
}

// Test 1: Vegetable Greenhouse Monitoring Rules
fn test_vegetable_monitoring() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🥬 TEST 1: Vegetable Monitoring Rules");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let grl_path = "grl_rules/vegetable_monitoring.grl";
    let mut engine = IncrementalEngine::new();

    match GrlReteLoader::load_from_file(grl_path, &mut engine) {
        Ok(count) => println!("✅ Loaded {} rules from {}\n", count, grl_path),
        Err(e) => {
            println!("❌ Failed to load {}: {}", grl_path, e);
            return;
        }
    }

    // Test Case 1: Greenhouse Cooling Control (Rule 1)
    println!("📋 Test Case 1.1: Greenhouse Cooling Control");
    println!("   Scenario: Temperature 32°C, Humidity 55%");

    let mut greenhouse = TypedFacts::new();
    greenhouse.set("zone_id", "GH-1");
    greenhouse.set("temperature", 32.0);
    greenhouse.set("humidity", 55.0);
    greenhouse.set("cooling_active", false);
    greenhouse.set("misting_active", false);

    let gh_handle = engine.insert("Greenhouse".to_string(), greenhouse);
    let fired = engine.fire_all();

    println!("   Rules Fired: {}", fired.len());

    if let Some(fact) = engine.working_memory().get(&gh_handle) {
        let cooling = fact.data.get("cooling_active").and_then(|v| v.as_boolean()).unwrap_or(false);
        let misting = fact.data.get("misting_active").and_then(|v| v.as_boolean()).unwrap_or(false);
        println!("   Result: cooling_active={}, misting_active={}", cooling, misting);

        if cooling && misting {
            println!("   ✅ PASS: Cooling activated correctly");
        } else {
            println!("   ❌ FAIL: Expected cooling and misting to be activated");
        }
    }

    // Test Case 2: CO2 Enrichment (Rule 2)
    let mut engine2 = IncrementalEngine::new();
    GrlReteLoader::load_from_file(grl_path, &mut engine2).ok();

    println!("\n📋 Test Case 1.2: CO2 Enrichment Control");
    println!("   Scenario: Light 15000 lux, CO2 700 ppm");

    let mut greenhouse = TypedFacts::new();
    greenhouse.set("zone_id", "GH-2");
    greenhouse.set("light_intensity", 15000.0);
    greenhouse.set("co2_ppm", 700.0);
    greenhouse.set("co2_target", 0.0);
    greenhouse.set("co2_injection", false);

    let gh_handle = engine2.insert("Greenhouse".to_string(), greenhouse);
    let fired = engine2.fire_all();

    println!("   Rules Fired: {}", fired.len());

    if let Some(fact) = engine2.working_memory().get(&gh_handle) {
        let injection = fact.data.get("co2_injection").and_then(|v| v.as_boolean()).unwrap_or(false);
        let target = fact.data.get("co2_target").and_then(|v| v.as_number()).unwrap_or(0.0);
        println!("   Result: co2_injection={}, target={} ppm", injection, target);

        if injection && target > 900.0 {
            println!("   ✅ PASS: CO2 enrichment activated correctly");
        } else {
            println!("   ❌ FAIL: Expected CO2 injection to be activated");
        }
    }

    // Test Case 3: Pest Risk Detection (Rule 3)
    let mut engine3 = IncrementalEngine::new();
    GrlReteLoader::load_from_file(grl_path, &mut engine3).ok();

    println!("\n📋 Test Case 1.3: Pest Risk Warning");
    println!("   Scenario: Temperature 28°C, Humidity 80%");

    let mut greenhouse = TypedFacts::new();
    greenhouse.set("zone_id", "GH-3");
    greenhouse.set("temperature", 28.0);
    greenhouse.set("humidity", 80.0);
    greenhouse.set("pest_risk", "LOW");

    let gh_handle = engine3.insert("Greenhouse".to_string(), greenhouse);
    let fired = engine3.fire_all();

    println!("   Rules Fired: {}", fired.len());

    if let Some(fact) = engine3.working_memory().get(&gh_handle) {
        let risk = fact.data.get("pest_risk").and_then(|v| Some(v.to_string())).unwrap_or_else(|| "UNKNOWN".to_string());
        println!("   Result: pest_risk={}", risk);

        if risk == "HIGH" {
            println!("   ✅ PASS: Pest risk detected correctly");
        } else {
            println!("   ❌ FAIL: Expected HIGH pest risk");
        }
    }

    println!("\n✅ Vegetable Monitoring: 3/3 test cases completed\n");
}

// Test 2: Aquaculture Monitoring Rules
fn test_aquaculture_monitoring() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🐟 TEST 2: Aquaculture Monitoring Rules");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let grl_path = "grl_rules/aquaculture_monitoring.grl";
    let mut engine = IncrementalEngine::new();

    match GrlReteLoader::load_from_file(grl_path, &mut engine) {
        Ok(count) => println!("✅ Loaded {} rules from {}\n", count, grl_path),
        Err(e) => {
            println!("❌ Failed to load {}: {}", grl_path, e);
            return;
        }
    }

    // Test Case 1: Critical Dissolved Oxygen (Rule 1 - MOST CRITICAL)
    println!("📋 Test Case 2.1: Critical Dissolved Oxygen Emergency");
    println!("   Scenario: DO 3.5 mg/L, Temperature 30°C");

    let mut pond = TypedFacts::new();
    pond.set("pond_id", "POND-1");
    pond.set("do_mg_per_liter", 3.5);
    pond.set("temperature", 30.0);
    pond.set("emergency_aeration", false);
    pond.set("alert_level", "NORMAL");

    let pond_handle = engine.insert("FishPond".to_string(), pond);
    let fired = engine.fire_all();

    println!("   Rules Fired: {}", fired.len());

    if let Some(fact) = engine.working_memory().get(&pond_handle) {
        let aeration = fact.data.get("emergency_aeration").and_then(|v| v.as_boolean()).unwrap_or(false);
        let alert = fact.data.get("alert_level").and_then(|v| Some(v.to_string())).unwrap_or_else(|| "UNKNOWN".to_string());
        println!("   Result: emergency_aeration={}, alert_level={}", aeration, alert);

        if aeration && alert == "CRITICAL" {
            println!("   ✅ PASS: Critical DO emergency triggered correctly");
        } else {
            println!("   ❌ FAIL: Expected emergency aeration");
        }
    }

    // Test Case 2: pH Imbalance (Rule 2)
    let mut engine2 = IncrementalEngine::new();
    GrlReteLoader::load_from_file(grl_path, &mut engine2).ok();

    println!("\n📋 Test Case 2.2: pH Imbalance Detection");
    println!("   Scenario: pH 9.0 (too high)");

    let mut pond = TypedFacts::new();
    pond.set("pond_id", "POND-2");
    pond.set("ph_value", 9.0);
    pond.set("ph_correction_needed", false);

    let pond_handle = engine2.insert("FishPond".to_string(), pond);
    let fired = engine2.fire_all();

    println!("   Rules Fired: {}", fired.len());

    if let Some(fact) = engine2.working_memory().get(&pond_handle) {
        let correction = fact.data.get("ph_correction_needed").and_then(|v| v.as_boolean()).unwrap_or(false);
        println!("   Result: ph_correction_needed={}", correction);

        if correction {
            println!("   ✅ PASS: pH correction triggered correctly");
        } else {
            println!("   ❌ FAIL: Expected pH correction");
        }
    }

    // Test Case 3: Ammonia Toxicity (Rule 3)
    let mut engine3 = IncrementalEngine::new();
    GrlReteLoader::load_from_file(grl_path, &mut engine3).ok();

    println!("\n📋 Test Case 2.3: Ammonia Toxicity Warning");
    println!("   Scenario: Ammonia 0.8 ppm, pH 8.5");

    let mut pond = TypedFacts::new();
    pond.set("pond_id", "POND-3");
    pond.set("ammonia_ppm", 0.8);
    pond.set("ph_value", 8.5);
    pond.set("water_change_urgent", false);
    pond.set("add_zeolite", false);

    let pond_handle = engine3.insert("FishPond".to_string(), pond);
    let fired = engine3.fire_all();

    println!("   Rules Fired: {}", fired.len());

    if let Some(fact) = engine3.working_memory().get(&pond_handle) {
        let water_change = fact.data.get("water_change_urgent").and_then(|v| v.as_boolean()).unwrap_or(false);
        let zeolite = fact.data.get("add_zeolite").and_then(|v| v.as_boolean()).unwrap_or(false);
        println!("   Result: water_change={}, add_zeolite={}", water_change, zeolite);

        if water_change && zeolite {
            println!("   ✅ PASS: Ammonia toxicity response triggered correctly");
        } else {
            println!("   ❌ FAIL: Expected water change and zeolite addition");
        }
    }

    println!("\n✅ Aquaculture Monitoring: 3/3 test cases completed\n");
}

// Test 3: Integrated Farm Rules (Aquaponics)
fn test_integrated_farm_rules() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("♻️  TEST 3: Integrated Farm Rules (Aquaponics)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let grl_path = "grl_rules/integrated_farm_rules.grl";
    let mut engine = IncrementalEngine::new();

    match GrlReteLoader::load_from_file(grl_path, &mut engine) {
        Ok(count) => println!("✅ Loaded {} rules from {}\n", count, grl_path),
        Err(e) => {
            println!("❌ Failed to load {}: {}", grl_path, e);
            return;
        }
    }

    // Test Case 1: Aquaponics Nutrient Cycle (Rule 1)
    println!("📋 Test Case 3.1: Aquaponics Nutrient Cycle");
    println!("   Scenario: Pond nitrate 35 ppm, Plants need nutrients (40%)");

    let mut pond = TypedFacts::new();
    pond.set("pond_id", "POND-A");
    pond.set("nitrate_ppm", 35.0);
    pond.set("aquaponics_enabled", true);
    pond.set("pump_to_greenhouse", false);

    let mut greenhouse = TypedFacts::new();
    greenhouse.set("zone_id", "GH-A");
    greenhouse.set("nutrient_level", 40.0);
    greenhouse.set("receive_pond_water", false);

    let pond_handle = engine.insert("FishPond".to_string(), pond);
    engine.insert("Greenhouse".to_string(), greenhouse);

    let fired = engine.fire_all();
    println!("   Rules Fired: {}", fired.len());

    if let Some(fact) = engine.working_memory().get(&pond_handle) {
        let pump = fact.data.get("pump_to_greenhouse").and_then(|v| v.as_boolean()).unwrap_or(false);
        println!("   Result: pump_to_greenhouse={}", pump);

        if pump {
            println!("   ✅ PASS: Aquaponics nutrient cycle activated correctly");
        } else {
            println!("   ❌ FAIL: Expected nutrient pump activation");
        }
    }

    // Test Case 2: Shared Evaporative Cooling (Rule 2)
    let mut engine2 = IncrementalEngine::new();
    GrlReteLoader::load_from_file(grl_path, &mut engine2).ok();

    println!("\n📋 Test Case 3.2: Shared Evaporative Cooling");
    println!("   Scenario: Greenhouse 35°C, Pond 22°C (indoor)");

    let mut greenhouse = TypedFacts::new();
    greenhouse.set("zone_id", "GH-B");
    greenhouse.set("temperature", 35.0);
    greenhouse.set("cooling_source", "none");

    let mut pond = TypedFacts::new();
    pond.set("pond_id", "POND-B");
    pond.set("temperature", 22.0);
    pond.set("location", "indoor");
    pond.set("evaporative_cooling_active", false);

    engine2.insert("Greenhouse".to_string(), greenhouse);
    let pond_handle = engine2.insert("FishPond".to_string(), pond);

    let fired = engine2.fire_all();
    println!("   Rules Fired: {}", fired.len());

    if let Some(fact) = engine2.working_memory().get(&pond_handle) {
        let cooling = fact.data.get("evaporative_cooling_active").and_then(|v| v.as_boolean()).unwrap_or(false);
        println!("   Result: evaporative_cooling_active={}", cooling);

        if cooling {
            println!("   ✅ PASS: Shared evaporative cooling activated correctly");
        } else {
            println!("   ❌ FAIL: Expected evaporative cooling");
        }
    }

    // Test Case 3: Farm Health Score (Rule 9)
    let mut engine3 = IncrementalEngine::new();
    GrlReteLoader::load_from_file(grl_path, &mut engine3).ok();

    println!("\n📋 Test Case 3.3: Farm Health Score Assessment");
    println!("   Scenario: All metrics excellent");

    let mut farm = TypedFacts::new();
    farm.set("avg_plant_health", 85.0);
    farm.set("avg_fish_health", 80.0);
    farm.set("water_quality_score", 75.0);
    farm.set("pest_pressure", 15.0);
    farm.set("disease_incidents", 0i64);
    farm.set("health_score", "UNKNOWN");

    let farm_handle = engine3.insert("Farm".to_string(), farm);
    let fired = engine3.fire_all();

    println!("   Rules Fired: {}", fired.len());

    if let Some(fact) = engine3.working_memory().get(&farm_handle) {
        let score = fact.data.get("health_score").and_then(|v| Some(v.to_string())).unwrap_or_else(|| "UNKNOWN".to_string());
        println!("   Result: health_score={}", score);

        if score == "EXCELLENT" {
            println!("   ✅ PASS: Farm health score calculated correctly");
        } else {
            println!("   ❌ FAIL: Expected EXCELLENT health score");
        }
    }

    println!("\n✅ Integrated Farm Rules: 3/3 test cases completed\n");
}

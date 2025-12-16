//! Medical Diagnostic System with Backward Chaining
//!
//! Demonstrates backward chaining for medical diagnosis:
//! - Start with symptoms (observations)
//! - Query possible diseases (goals)
//! - Trace back through causes
//! - Explain reasoning path
//!
//! Run with: cargo run --example medical_diagnosis_demo --features backward-chaining

#![cfg(feature = "backward-chaining")]

use rust_rule_engine::backward::{BackwardConfig, BackwardEngine, SearchStrategy};
use rust_rule_engine::types::Value;
use rust_rule_engine::{Facts, KnowledgeBase};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏥 Medical Diagnostic System - Backward Chaining Demo");
    println!("====================================================\n");

    demo_simple_diagnosis()?;
    demo_complex_diagnosis()?;
    demo_differential_diagnosis()?;
    demo_explain_reasoning()?;

    Ok(())
}

fn demo_simple_diagnosis() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 1: Simple Diagnosis - Flu Detection");
    println!("-------------------------------------------");

    let kb = KnowledgeBase::new("SimpleDiagnosis");

    // Medical knowledge base
    let rules = r#"
    rule "DiagnoseFlu" salience 100 {
        when
            Patient.HasFever == true && 
            Patient.HasCough == true && 
            Patient.HasFatigue == true
        then
            Diagnosis.Disease = "Influenza";
            Diagnosis.Confidence = 0.85;
            Diagnosis.Recommendation = "Rest, fluids, and antiviral medication";
    }
    
    rule "FeverFromInfection" salience 90 {
        when
            Patient.WhiteBloodCellCount > 11000
        then
            Patient.HasFever = true;
            Patient.HasInfection = true;
    }
    
    rule "CoughFromRespiratoryIssue" salience 90 {
        when
            Patient.LungCongestion == true
        then
            Patient.HasCough = true;
    }
    
    rule "FatigueFromFever" salience 85 {
        when
            Patient.HasFever == true && Patient.BodyTemperature > 38.5
        then
            Patient.HasFatigue = true;
    }
    "#;

    kb.add_rules_from_grl(rules)?;

    println!("📋 Medical Knowledge Base loaded:");
    println!("   • {} rules", kb.get_rules().len());
    for rule in kb.get_rules() {
        println!("     - {}", rule.name);
    }

    // Patient observations (lab results and symptoms)
    let mut facts = Facts::new();
    facts.set(
        "Patient",
        Value::Object({
            let mut patient = HashMap::new();
            patient.insert("WhiteBloodCellCount".to_string(), Value::Number(12500.0));
            patient.insert("BodyTemperature".to_string(), Value::Number(39.2));
            patient.insert("LungCongestion".to_string(), Value::Boolean(true));
            patient
        }),
    );

    println!("\n🔬 Patient Observations:");
    if let Some(Value::Object(patient)) = facts.get("Patient") {
        for (key, value) in patient {
            println!("   • {}: {:?}", key, value);
        }
    }

    // Create diagnostic engine
    let mut bc_engine = BackwardEngine::new(kb);

    // Query: Does patient have Influenza?
    println!("\n🔍 Query: Can we diagnose \"Influenza\"?");
    println!("   (Checking if Diagnosis.Disease == \"Influenza\")");

    let result = bc_engine.query(r#"Diagnosis.Disease == "Influenza""#, &mut facts)?;

    if result.provable {
        println!("\n✅ DIAGNOSIS CONFIRMED: Influenza");
        println!("\n📊 Diagnostic Statistics:");
        println!("   • Reasoning steps: {}", result.stats.goals_explored);
        println!("   • Rules evaluated: {}", result.stats.rules_evaluated);
        println!("   • Reasoning depth: {}", result.stats.max_depth);

        println!("\n📜 Reasoning Chain:");
        result.proof_trace.print();
    } else {
        println!("\n❌ Cannot confirm Influenza");
        println!("⚠️  Missing information:");
        for missing in &result.missing_facts {
            println!("   • {}", missing);
        }
    }

    Ok(())
}

fn demo_complex_diagnosis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📝 Demo 2: Complex Diagnosis - Multiple Conditions");
    println!("--------------------------------------------------");

    let kb = KnowledgeBase::new("ComplexDiagnosis");

    let rules = r#"
    rule "DiagnoseDiabetes" salience 100 {
        when
            Patient.BloodGlucose > 126 && Patient.HbA1c > 6.5
        then
            Diagnosis.Disease = "Type 2 Diabetes";
            Diagnosis.Severity = "Moderate";
    }
    
    rule "DiagnoseHypertension" salience 95 {
        when
            Patient.SystolicBP > 140 && Patient.DiastolicBP > 90
        then
            Diagnosis.HasHypertension = true;
            Risk.Cardiovascular = "High";
    }
    
    rule "MetabolicSyndrome" salience 90 {
        when
            Diagnosis.Disease == "Type 2 Diabetes" && 
            Diagnosis.HasHypertension == true
        then
            Diagnosis.MetabolicSyndrome = true;
            Risk.StrokeRisk = "Very High";
            Treatment.Lifestyle = "Diet, Exercise, Medication";
    }
    
    rule "CheckBloodSugar" salience 80 {
        when
            Patient.FastingHours >= 8 && Patient.RecentGlucoseTest == true
        then
            Patient.BloodGlucose = 145;
            Patient.HbA1c = 7.2;
    }
    "#;

    kb.add_rules_from_grl(rules)?;

    let mut facts = Facts::new();
    facts.set(
        "Patient",
        Value::Object({
            let mut patient = HashMap::new();
            patient.insert("FastingHours".to_string(), Value::Number(10.0));
            patient.insert("RecentGlucoseTest".to_string(), Value::Boolean(true));
            patient.insert("SystolicBP".to_string(), Value::Number(155.0));
            patient.insert("DiastolicBP".to_string(), Value::Number(95.0));
            patient
        }),
    );

    println!("🔬 Patient Test Results:");
    if let Some(Value::Object(patient)) = facts.get("Patient") {
        for (key, value) in patient {
            println!("   • {}: {:?}", key, value);
        }
    }

    let mut bc_engine = BackwardEngine::new(kb);

    // Query for Metabolic Syndrome
    println!("\n🔍 Query: Does patient have Metabolic Syndrome?");
    let result = bc_engine.query(r#"Diagnosis.MetabolicSyndrome == true"#, &mut facts)?;

    if result.provable {
        println!("\n⚠️  CRITICAL DIAGNOSIS: Metabolic Syndrome Detected");
        println!("\n🧬 Underlying Conditions:");
        println!("   • Type 2 Diabetes");
        println!("   • Hypertension");
        println!("\n⚡ Risk Assessment:");
        println!("   • Cardiovascular Risk: HIGH");
        println!("   • Stroke Risk: VERY HIGH");
        println!("\n💊 Recommended Treatment:");
        println!("   • Lifestyle: Diet, Exercise, Medication");
    } else {
        println!("\n✅ No Metabolic Syndrome detected");
    }

    Ok(())
}

fn demo_differential_diagnosis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📝 Demo 3: Differential Diagnosis - Chest Pain");
    println!("---------------------------------------------");

    let kb = KnowledgeBase::new("ChestPainDiagnosis");

    let rules = r#"
    rule "DiagnoseHeartAttack" salience 100 {
        when
            Symptoms.ChestPain == "Crushing" && 
            Symptoms.RadiatingPain == true &&
            Tests.TroponinElevated == true
        then
            Diagnosis.Critical = "Myocardial Infarction";
            Action.Emergency = "Call 911 immediately";
    }
    
    rule "DiagnoseAngina" salience 95 {
        when
            Symptoms.ChestPain == "Squeezing" && 
            Symptoms.ExertionalPain == true &&
            Tests.ECGAbnormal == false
        then
            Diagnosis.Condition = "Stable Angina";
            Action.Recommended = "Nitroglycerin, cardiology consult";
    }
    
    rule "DiagnoseGERD" salience 90 {
        when
            Symptoms.ChestPain == "Burning" && 
            Symptoms.AfterMeals == true &&
            Tests.ECGNormal == true
        then
            Diagnosis.Condition = "GERD";
            Action.Treatment = "Antacids, lifestyle changes";
    }
    
    rule "DiagnoseCostochondritis" salience 85 {
        when
            Symptoms.ChestPain == "Sharp" && 
            Symptoms.TenderToTouch == true &&
            Tests.CardiacMarkersNormal == true
        then
            Diagnosis.Condition = "Costochondritis";
            Action.Treatment = "NSAIDs, rest";
    }
    "#;

    kb.add_rules_from_grl(rules)?;

    println!("🏥 Differential Diagnosis Knowledge Base:");
    println!("   Possible conditions:");
    println!("   • Myocardial Infarction (Heart Attack)");
    println!("   • Stable Angina");
    println!("   • GERD (Acid Reflux)");
    println!("   • Costochondritis (Chest Wall Inflammation)");

    // Scenario 1: Heart Attack
    println!("\n📋 SCENARIO 1: Emergency Case");
    println!("------------------------------");

    let mut facts1 = Facts::new();
    facts1.set(
        "Symptoms",
        Value::Object({
            let mut symptoms = HashMap::new();
            symptoms.insert(
                "ChestPain".to_string(),
                Value::String("Crushing".to_string()),
            );
            symptoms.insert("RadiatingPain".to_string(), Value::Boolean(true));
            symptoms
        }),
    );
    facts1.set(
        "Tests",
        Value::Object({
            let mut tests = HashMap::new();
            tests.insert("TroponinElevated".to_string(), Value::Boolean(true));
            tests
        }),
    );

    let mut bc_engine = BackwardEngine::new(kb.clone());
    let result = bc_engine.query(
        r#"Diagnosis.Critical == "Myocardial Infarction""#,
        &mut facts1,
    )?;

    if result.provable {
        println!("🚨 CRITICAL: Myocardial Infarction (Heart Attack)");
        println!("⚡ ACTION REQUIRED: Call 911 immediately");
        println!("\n📜 Diagnostic Reasoning:");
        result.proof_trace.print();
    }

    // Scenario 2: GERD
    println!("\n\n📋 SCENARIO 2: Non-Emergency Case");
    println!("----------------------------------");

    let mut facts2 = Facts::new();
    facts2.set(
        "Symptoms",
        Value::Object({
            let mut symptoms = HashMap::new();
            symptoms.insert(
                "ChestPain".to_string(),
                Value::String("Burning".to_string()),
            );
            symptoms.insert("AfterMeals".to_string(), Value::Boolean(true));
            symptoms
        }),
    );
    facts2.set(
        "Tests",
        Value::Object({
            let mut tests = HashMap::new();
            tests.insert("ECGNormal".to_string(), Value::Boolean(true));
            tests
        }),
    );

    let mut bc_engine2 = BackwardEngine::new(kb);
    let result2 = bc_engine2.query(r#"Diagnosis.Condition == "GERD""#, &mut facts2)?;

    if result2.provable {
        println!("✅ DIAGNOSIS: GERD (Gastroesophageal Reflux Disease)");
        println!("💊 TREATMENT: Antacids, lifestyle changes");
        println!("\n📜 Diagnostic Reasoning:");
        result2.proof_trace.print();
    }

    Ok(())
}

fn demo_explain_reasoning() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📝 Demo 4: Explain Medical Reasoning");
    println!("------------------------------------");

    let kb = KnowledgeBase::new("ExplainReasoning");

    let rules = r#"
    rule "PneumoniaDiagnosis" salience 100 {
        when
            Clinical.HasFever == true && 
            Clinical.HasCough == true &&
            Imaging.ChestXRayInfiltrate == true &&
            Lab.ElevatedWBC == true
        then
            Diagnosis.Disease = "Pneumonia";
            Diagnosis.Type = "Community-Acquired";
    }
    
    rule "FeverIndicatesInfection" salience 90 {
        when
            Patient.Temperature > 38.0
        then
            Clinical.HasFever = true;
    }
    
    rule "ProductiveCough" salience 90 {
        when
            Patient.CoughType == "Productive" && 
            Patient.SputumColor == "Yellow-Green"
        then
            Clinical.HasCough = true;
            Clinical.BacterialInfection = "Likely";
    }
    
    rule "ChestXRayFindings" salience 85 {
        when
            Imaging.Consolidation == true || Imaging.Opacity == true
        then
            Imaging.ChestXRayInfiltrate = true;
    }
    
    rule "WBCElevation" salience 85 {
        when
            Lab.WhiteBloodCells > 10000
        then
            Lab.ElevatedWBC = true;
    }
    "#;

    kb.add_rules_from_grl(rules)?;

    let mut facts = Facts::new();
    facts.set(
        "Patient",
        Value::Object({
            let mut patient = HashMap::new();
            patient.insert("Temperature".to_string(), Value::Number(39.5));
            patient.insert(
                "CoughType".to_string(),
                Value::String("Productive".to_string()),
            );
            patient.insert(
                "SputumColor".to_string(),
                Value::String("Yellow-Green".to_string()),
            );
            patient
        }),
    );
    facts.set(
        "Imaging",
        Value::Object({
            let mut imaging = HashMap::new();
            imaging.insert("Consolidation".to_string(), Value::Boolean(true));
            imaging
        }),
    );
    facts.set(
        "Lab",
        Value::Object({
            let mut lab = HashMap::new();
            lab.insert("WhiteBloodCells".to_string(), Value::Number(15000.0));
            lab
        }),
    );

    println!("🔬 Clinical Data:");
    println!("   Temperature: 39.5°C");
    println!("   Cough: Productive with yellow-green sputum");
    println!("   Chest X-Ray: Consolidation present");
    println!("   WBC: 15,000/μL");

    let config = BackwardConfig {
        max_depth: 10,
        strategy: SearchStrategy::DepthFirst,
        enable_memoization: true,
        max_solutions: 1,
    };

    let mut bc_engine = BackwardEngine::with_config(kb, config);

    println!("\n🔍 Query: Can we diagnose Pneumonia?");
    let explanation = bc_engine.explain_why(r#"Diagnosis.Disease == "Pneumonia""#, &mut facts)?;

    println!("\n📖 Medical Reasoning Explanation:");
    println!("{}", explanation);

    println!("\n💡 Clinical Interpretation:");
    println!("   1. ✓ Patient has fever (>38°C) → Indicates infection");
    println!("   2. ✓ Productive cough with purulent sputum → Bacterial infection likely");
    println!("   3. ✓ Chest X-ray shows consolidation → Lung infiltrate present");
    println!("   4. ✓ Elevated WBC (>10,000) → Immune response to infection");
    println!("   ");
    println!("   CONCLUSION: All criteria met for Community-Acquired Pneumonia");
    println!("   ");
    println!("   📋 Recommended Actions:");
    println!("   • Start empiric antibiotic therapy");
    println!("   • Monitor oxygen saturation");
    println!("   • Follow-up chest X-ray in 4-6 weeks");

    Ok(())
}

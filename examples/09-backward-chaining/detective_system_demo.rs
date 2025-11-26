//! Detective Investigation System - Crime Solving with Backward Chaining
//!
//! A Sherlock Holmes-style investigation system that uses backward chaining
//! to solve crimes by working backwards from a suspect to find supporting evidence.
//!
//! Demonstrates:
//! - Goal-driven investigation
//! - Evidence chain reasoning
//! - Multiple hypothesis testing
//! - Proof by contradiction
//!
//! Run with: cargo run --example detective_system_demo --features backward-chaining

#![cfg(feature = "backward-chaining")]

use rust_rule_engine::backward::{BackwardEngine, BackwardConfig, SearchStrategy};
use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::types::Value;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🕵️  Detective Investigation System - Backward Chaining");
    println!("=====================================================\n");

    demo_murder_mystery()?;
    demo_alibi_checking()?;
    demo_motive_analysis()?;
    demo_sherlock_mode()?;

    Ok(())
}

fn demo_murder_mystery() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Demo 1: The Mansion Murder Mystery");
    println!("--------------------------------------");

    let mut kb = KnowledgeBase::new("MurderInvestigation");
    
    let rules = r#"
    rule "SuspectIsGuilty" salience 100 {
        when
            Suspect.HasMotive == true && 
            Suspect.HadOpportunity == true && 
            Suspect.HasIncriminatingEvidence == true
        then
            Investigation.Guilty = true;
            Investigation.Confidence = 0.95;
    }
    
    rule "FinancialMotive" salience 90 {
        when
            Suspect.InheritsMoney == true && 
            Suspect.HasDebts == true
        then
            Suspect.HasMotive = true;
            Motive.Type = "Financial Gain";
    }
    
    rule "OpportunityAtCrimeScene" salience 90 {
        when
            Evidence.FingerprintsMatch == true && 
            Timeline.WasPresent == true
        then
            Suspect.HadOpportunity = true;
    }
    
    rule "IncriminatingEvidence" salience 85 {
        when
            Evidence.DNAMatch == true || 
            Evidence.WeaponOwnership == true
        then
            Suspect.HasIncriminatingEvidence = true;
    }
    
    rule "FingerprintAnalysis" salience 80 {
        when
            Forensics.PrintsFound == true && 
            Suspect.PrintsOnFile == true
        then
            Evidence.FingerprintsMatch = true;
    }
    
    rule "TimelineAnalysis" salience 80 {
        when
            Witness.SawSuspect == true && 
            TimeOfDeath.Window == "10PM-11PM"
        then
            Timeline.WasPresent = true;
    }
    "#;
    
    kb.add_rules_from_grl(rules)?;

    println!("🏛️  THE CASE:");
    println!("   Victim: Lord Blackwood");
    println!("   Location: Blackwood Manor");
    println!("   Time: 10:30 PM, November 24th");
    println!("   Method: Blunt force trauma");

    println!("\n👤 PRIME SUSPECT:");
    println!("   Name: Richard Blackwood (nephew)");
    println!("   Status: Under investigation");

    // Evidence collected
    let mut facts = Facts::new();
    facts.set("Suspect", Value::Object({
        let mut suspect = HashMap::new();
        suspect.insert("InheritsMoney".to_string(), Value::Boolean(true));
        suspect.insert("HasDebts".to_string(), Value::Boolean(true));
        suspect.insert("PrintsOnFile".to_string(), Value::Boolean(true));
        suspect
    }));
    
    facts.set("Forensics", Value::Object({
        let mut forensics = HashMap::new();
        forensics.insert("PrintsFound".to_string(), Value::Boolean(true));
        forensics
    }));
    
    facts.set("Witness", Value::Object({
        let mut witness = HashMap::new();
        witness.insert("SawSuspect".to_string(), Value::Boolean(true));
        witness
    }));
    
    facts.set("TimeOfDeath", Value::Object({
        let mut tod = HashMap::new();
        tod.insert("Window".to_string(), Value::String("10PM-11PM".to_string()));
        tod
    }));
    
    facts.set("Evidence", Value::Object({
        let mut evidence = HashMap::new();
        evidence.insert("DNAMatch".to_string(), Value::Boolean(true));
        evidence
    }));

    println!("\n🔍 EVIDENCE COLLECTED:");
    println!("   ✓ Suspect inherits £5 million");
    println!("   ✓ Suspect has £200,000 in gambling debts");
    println!("   ✓ Fingerprints found at crime scene");
    println!("   ✓ Witness saw suspect at 10:15 PM");
    println!("   ✓ DNA match on murder weapon");

    let mut bc_engine = BackwardEngine::new(kb);

    println!("\n🎯 INVESTIGATION QUERY:");
    println!("   Can we prove Richard Blackwood is guilty?");
    
    let result = bc_engine.query("Investigation.Guilty == true", &mut facts)?;

    if result.provable {
        println!("\n⚖️  VERDICT: GUILTY");
        println!("\n📊 Case Strength:");
        println!("   • Evidence chain depth: {}", result.stats.max_depth);
        println!("   • Facts verified: {}", result.stats.goals_explored);
        println!("   • Confidence: 95%");
        
        println!("\n🔗 EVIDENCE CHAIN:");
        result.proof_trace.print();
        
        println!("\n📋 SUMMARY:");
        println!("   ✓ MOTIVE: Financial gain (inheritance + debts)");
        println!("   ✓ OPPORTUNITY: Present at crime scene (fingerprints + witness)");
        println!("   ✓ EVIDENCE: DNA match on weapon");
        println!("\n   🚔 RECOMMENDATION: Arrest warrant approved");
    } else {
        println!("\n⚖️  VERDICT: INSUFFICIENT EVIDENCE");
        println!("\n❌ Missing critical evidence:");
        for missing in &result.missing_facts {
            println!("   • {}", missing);
        }
    }

    Ok(())
}

fn demo_alibi_checking() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📝 Demo 2: Alibi Verification");
    println!("-----------------------------");

    let mut kb = KnowledgeBase::new("AlibilChecking");
    
    let rules = r#"
    rule "AlibliValid" salience 100 {
        when
            Alibi.HasWitness == true && 
            Alibi.CorroboratedByEvidence == true && 
            Alibi.TimelineConsistent == true
        then
            Suspect.Innocent = true;
            Investigation.Status = "Cleared";
    }
    
    rule "WitnessCorroboration" salience 90 {
        when
            Witness.Count >= 2 && 
            Witness.StatementsMatch == true
        then
            Alibi.HasWitness = true;
    }
    
    rule "EvidenceSupportsAlibi" salience 90 {
        when
            Evidence.PhoneRecords == "Different Location" && 
            Evidence.CCTVFootage == "Confirmed"
        then
            Alibi.CorroboratedByEvidence = true;
    }
    
    rule "TimelineCheck" salience 85 {
        when
            Alibi.ClaimedLocation == "Restaurant" && 
            Evidence.CreditCardTransaction == true &&
            Transaction.Time == "Crime Time"
        then
            Alibi.TimelineConsistent = true;
    }
    "#;
    
    kb.add_rules_from_grl(rules)?;

    println!("👤 SUSPECT: Sarah Thompson");
    println!("   Claim: 'I was at Giovanni's Restaurant'");
    println!("   Time: 10:00 PM - 11:30 PM");

    let mut facts = Facts::new();
    facts.set("Witness", Value::Object({
        let mut witness = HashMap::new();
        witness.insert("Count".to_string(), Value::Number(3.0));
        witness.insert("StatementsMatch".to_string(), Value::Boolean(true));
        witness
    }));
    
    facts.set("Evidence", Value::Object({
        let mut evidence = HashMap::new();
        evidence.insert("PhoneRecords".to_string(), Value::String("Different Location".to_string()));
        evidence.insert("CCTVFootage".to_string(), Value::String("Confirmed".to_string()));
        evidence.insert("CreditCardTransaction".to_string(), Value::Boolean(true));
        evidence
    }));
    
    facts.set("Alibi", Value::Object({
        let mut alibi = HashMap::new();
        alibi.insert("ClaimedLocation".to_string(), Value::String("Restaurant".to_string()));
        alibi
    }));
    
    facts.set("Transaction", Value::Object({
        let mut transaction = HashMap::new();
        transaction.insert("Time".to_string(), Value::String("Crime Time".to_string()));
        transaction
    }));

    println!("\n🔍 VERIFICATION:");
    println!("   ✓ 3 witnesses confirm presence at restaurant");
    println!("   ✓ Phone location data: 2 miles from crime scene");
    println!("   ✓ CCTV footage confirms alibi");
    println!("   ✓ Credit card used at restaurant at time of murder");

    let mut bc_engine = BackwardEngine::new(kb);
    let result = bc_engine.query("Suspect.Innocent == true", &mut facts)?;

    if result.provable {
        println!("\n✅ ALIBI CONFIRMED: VALID");
        println!("   Suspect cleared of all charges");
        println!("\n📜 Verification Chain:");
        result.proof_trace.print();
    } else {
        println!("\n❌ ALIBI: QUESTIONABLE");
    }

    Ok(())
}

fn demo_motive_analysis() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📝 Demo 3: Motive Analysis - Multiple Suspects");
    println!("-----------------------------------------------");

    let mut kb = KnowledgeBase::new("MotiveAnalysis");
    
    let rules = r#"
    rule "StrongFinancialMotive" salience 100 {
        when
            Suspect.BenefitsFinancially == true && 
            Financial.Amount > 100000 &&
            Suspect.FinancialDesperation == true
        then
            Motive.Strength = "Strong";
            Motive.Type = "Financial";
    }
    
    rule "RevengeMotlive" salience 95 {
        when
            Relationship.History == "Conflict" && 
            PastIncident.Severity == "High"
        then
            Motive.Strength = "Strong";
            Motive.Type = "Revenge";
    }
    
    rule "JealousyMotive" salience 90 {
        when
            Relationship.Type == "Love Triangle" && 
            Suspect.EmotionalState == "Jealous"
        then
            Motive.Strength = "Moderate";
            Motive.Type = "Passion";
    }
    "#;
    
    kb.add_rules_from_grl(rules)?;

    println!("🎭 THREE SUSPECTS, THREE MOTIVES\n");

    // Suspect 1: Financial
    println!("👤 SUSPECT 1: Business Partner");
    let mut facts1 = Facts::new();
    facts1.set("Suspect", Value::Object({
        let mut s = HashMap::new();
        s.insert("BenefitsFinancially".to_string(), Value::Boolean(true));
        s.insert("FinancialDesperation".to_string(), Value::Boolean(true));
        s
    }));
    facts1.set("Financial", Value::Object({
        let mut f = HashMap::new();
        f.insert("Amount".to_string(), Value::Number(500000.0));
        f
    }));

    let mut bc1 = BackwardEngine::new(kb.clone());
    let r1 = bc1.query("Motive.Type == 'Financial'", &mut facts1)?;
    
    if r1.provable {
        println!("   ✓ Motive: FINANCIAL (Strong)");
        println!("   Stands to gain: £500,000");
    }

    // Suspect 2: Revenge
    println!("\n👤 SUSPECT 2: Ex-Employee");
    let mut facts2 = Facts::new();
    facts2.set("Relationship", Value::Object({
        let mut r = HashMap::new();
        r.insert("History".to_string(), Value::String("Conflict".to_string()));
        r
    }));
    facts2.set("PastIncident", Value::Object({
        let mut p = HashMap::new();
        p.insert("Severity".to_string(), Value::String("High".to_string()));
        p
    }));

    let mut bc2 = BackwardEngine::new(kb.clone());
    let r2 = bc2.query("Motive.Type == 'Revenge'", &mut facts2)?;
    
    if r2.provable {
        println!("   ✓ Motive: REVENGE (Strong)");
        println!("   History: Fired under controversial circumstances");
    }

    // Suspect 3: Jealousy
    println!("\n👤 SUSPECT 3: Romantic Rival");
    let mut facts3 = Facts::new();
    facts3.set("Relationship", Value::Object({
        let mut r = HashMap::new();
        r.insert("Type".to_string(), Value::String("Love Triangle".to_string()));
        r
    }));
    facts3.set("Suspect", Value::Object({
        let mut s = HashMap::new();
        s.insert("EmotionalState".to_string(), Value::String("Jealous".to_string()));
        s
    }));

    let mut bc3 = BackwardEngine::new(kb);
    let r3 = bc3.query("Motive.Type == 'Passion'", &mut facts3)?;
    
    if r3.provable {
        println!("   ✓ Motive: PASSION/JEALOUSY (Moderate)");
        println!("   Relationship: Love triangle with victim");
    }

    println!("\n🎯 INVESTIGATION PRIORITY:");
    println!("   1. Business Partner (Financial - Strong motive)");
    println!("   2. Ex-Employee (Revenge - Strong motive)");
    println!("   3. Romantic Rival (Passion - Moderate motive)");

    Ok(())
}

fn demo_sherlock_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n\n📝 Demo 4: Sherlock Holmes Mode - Deductive Reasoning");
    println!("-----------------------------------------------------");

    let mut kb = KnowledgeBase::new("SherlockMode");
    
    let rules = r#"
    rule "DeduceIdentity" salience 100 {
        when
            Observation.Muddy == true && 
            Observation.SpecificMudType == true &&
            Knowledge.MudSourceIdentified == true
        then
            Deduction.Location = "Crime Scene Identified";
    }
    
    rule "IdentifyMudSource" salience 90 {
        when
            Lab.MudComposition == "Red Clay with Iron Oxide" &&
            Geographic.RareInCity == true
        then
            Knowledge.MudSourceIdentified = true;
            Location.Specific = "Old Brickworks, East End";
    }
    
    rule "ObservationMudBoots" salience 85 {
        when
            Physical.BootsWorn == true &&
            Physical.MudColor == "Reddish"
        then
            Observation.Muddy = true;
            Observation.SpecificMudType = true;
    }
    
    rule "GeographicConstraint" salience 80 {
        when
            Location.Specific == "Old Brickworks, East End"
        then
            Geographic.RareInCity = true;
            Suspects.Narrowed = "3 people work there";
    }
    "#;
    
    kb.add_rules_from_grl(rules)?;

    println!("🎩 SHERLOCK HOLMES: 'Elementary, my dear Watson!'\n");
    println!("🔬 INITIAL OBSERVATION:");
    println!("   'The suspect has reddish mud on their boots...'");

    let mut facts = Facts::new();
    facts.set("Physical", Value::Object({
        let mut p = HashMap::new();
        p.insert("BootsWorn".to_string(), Value::Boolean(true));
        p.insert("MudColor".to_string(), Value::String("Reddish".to_string()));
        p
    }));
    let config = BackwardConfig {
        max_depth: 10,
        strategy: SearchStrategy::DepthFirst,
        enable_memoization: true,
        max_solutions: 1,
    };

    let mut bc_engine = BackwardEngine::with_config(kb, config);

    println!("\n🎯 HOLMES' QUESTION: 'Where was the suspect?'");
    let result = bc_engine.query("Deduction.Location == 'Crime Scene Identified'", &mut facts)?;

    if result.provable {
        println!("\n🔍 DEDUCTIVE CHAIN:");
        println!("   1. Boots have reddish mud");
        println!("   2. Lab analysis: Red clay with iron oxide");
        println!("   3. This mud type is rare in London");
        println!("   4. Only found at: Old Brickworks, East End");
        println!("   5. Only 3 people work there");
        
        println!("\n💡 HOLMES' CONCLUSION:");
        println!("   'The crime must have occurred at the Old Brickworks!'");
        println!("   'We have narrowed our suspects to just 3 individuals.'");
        
        println!("\n📊 Reasoning Statistics:");
        println!("   • Deductive steps: {}", result.stats.goals_explored);
        println!("   • Logic depth: {}", result.stats.max_depth);
        
        println!("\n📜 Complete Reasoning Chain:");
        result.proof_trace.print();
    }

    println!("\n🎩 'When you have eliminated the impossible, whatever remains,");
    println!("    however improbable, must be the truth.' - Sherlock Holmes");

    Ok(())
}

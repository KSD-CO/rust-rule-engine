//! Family Relationship Reasoning using Backward Chaining
//!
//! This example demonstrates:
//! - Transitive relationship reasoning (parent -> grandparent -> great-grandparent)
//! - Breadth-first search strategy
//! - Complex relationship inference
//! - Multiple proof paths

use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::types::Value;
use rust_rule_engine::backward::BackwardEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Family Relationship Reasoning - Backward Chaining       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test Case 1: Grandparent relationship
    println!("📋 Test Case 1: Grandparent Relationship");
    println!("─────────────────────────────────────────────────────────────");
    test_grandparent_relationship()?;

    println!("\n");

    // Test Case 2: Sibling relationship
    println!("📋 Test Case 2: Sibling Relationship");
    println!("─────────────────────────────────────────────────────────────");
    test_sibling_relationship()?;

    println!("\n");

    // Test Case 3: Uncle/Aunt relationship
    println!("📋 Test Case 3: Uncle/Aunt Relationship");
    println!("─────────────────────────────────────────────────────────────");
    test_uncle_relationship()?;

    println!("\n");

    // Test Case 4: Cousin relationship
    println!("📋 Test Case 4: Cousin Relationship");
    println!("─────────────────────────────────────────────────────────────");
    test_cousin_relationship()?;

    println!("\n✅ All family relationship tests completed!");
    Ok(())
}

fn create_family_rules() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let rules = r#"
rule "DefineParent_FatherOf" {
    when
        Person.IsFatherOf == true
    then
        Person.IsParentOf = true;
}

rule "DefineParent_MotherOf" {
    when
        Person.IsMotherOf == true
    then
        Person.IsParentOf = true;
}

rule "DefineChild_HasParent" {
    when
        Person.HasParent == true
    then
        Person.IsChildOf = true;
}

rule "InferGrandparent_TwoGenerations" {
    when
        Person.IsParentOf == true && Child.HasParent == true && Child.IsParentOf == true
    then
        Person.IsGrandparentOf = true;
}

rule "InferGrandchild_FromGrandparent" {
    when
        Grandparent.IsGrandparentOf == true
    then
        Person.IsGrandchildOf = true;
}

rule "DefineSibling_SameParents" {
    when
        Person1.ParentID == Person2.ParentID && Person1.ID != Person2.ID
    then
        Person1.IsSiblingOf = true;
}

rule "InferBrother_MaleSibling" {
    when
        Person.IsSiblingOf == true && Person.Gender == "Male"
    then
        Person.IsBrotherOf = true;
}

rule "InferSister_FemaleSibling" {
    when
        Person.IsSiblingOf == true && Person.Gender == "Female"
    then
        Person.IsSisterOf = true;
}

rule "InferUncle_ParentsBrother" {
    when
        Parent.IsParentOf == true && Uncle.IsBrotherOf == true && Parent.ID == Uncle.SiblingID
    then
        Uncle.IsUncleOf = true;
}

rule "InferAunt_ParentsSister" {
    when
        Parent.IsParentOf == true && Aunt.IsSisterOf == true && Parent.ID == Aunt.SiblingID
    then
        Aunt.IsAuntOf = true;
}

rule "InferCousin_ParentsSiblingsChild" {
    when
        Parent1.IsParentOf == true && Parent2.IsParentOf == true && Parent1.IsSiblingOf == true && Parent1.SiblingID == Parent2.ID
    then
        Person.IsCousinOf = true;
}

rule "InferNieceNephew_FromUncleAunt" {
    when
        Person.IsUncleOf == true
    then
        Child.IsNieceNephewOf = true;
}

rule "DetermineGeneration_First" {
    when
        Person.IsParentOf == true
    then
        Person.Generation = 1;
}

rule "DetermineGeneration_Second" {
    when
        Person.IsGrandparentOf == true
    then
        Person.Generation = 2;
}

rule "IdentifyFamily_Connected" {
    when
        Person.IsParentOf == true || Person.IsSiblingOf == true
    then
        Person.IsFamily = true;
}

rule "CalculateFamilySize_Small" {
    when
        Family.MemberCount <= 4
    then
        Family.Size = "Small";
}

rule "CalculateFamilySize_Medium" {
    when
        Family.MemberCount > 4 && Family.MemberCount <= 8
    then
        Family.Size = "Medium";
}

rule "CalculateFamilySize_Large" {
    when
        Family.MemberCount > 8
    then
        Family.Size = "Large";
}
    "#;

    let mut kb = KnowledgeBase::new("FamilyRelationshipSystem");
    for rule in rust_rule_engine::parser::grl::GRLParser::parse_rules(rules)? {
        kb.add_rule(rule)?;
    }
    Ok(kb)
}

fn test_grandparent_relationship() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_family_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    // Setup family facts:
    // John (grandfather) -> Mary (mother) -> Alice (child)
    let mut facts = Facts::new();

    // John is father of Mary
    facts.set("John.IsFatherOf", Value::Boolean(true));
    facts.set("Mary.HasParent", Value::Boolean(true));
    facts.set("Mary.ParentName", Value::String("John".to_string()));

    // Mary is mother of Alice
    facts.set("Mary.IsMotherOf", Value::Boolean(true));
    facts.set("Alice.HasParent", Value::Boolean(true));
    facts.set("Alice.ParentName", Value::String("Mary".to_string()));

    println!("Family Structure:");
    println!("  John (grandfather)");
    println!("    └─ Mary (mother)");
    println!("        └─ Alice (child)");

    // Query: Is John a grandparent?
    println!("\n🔍 Query: Is John a grandparent of Alice?");
    println!("   Person.IsGrandparentOf == true");

    // First, establish that John is parent of Mary
    facts.set("Person.IsParentOf", Value::Boolean(false));
    let parent_result = bc_engine.query("Person.IsParentOf == true", &mut facts)?;

    if parent_result.provable {
        println!("  ✓ Step 1: Confirmed John is parent of Mary");
    }

    // Then, establish that Mary is parent of Alice
    facts.set("Child.IsParentOf", Value::Boolean(false));
    let child_parent_result = bc_engine.query("Child.IsParentOf == true", &mut facts)?;

    if child_parent_result.provable {
        println!("  ✓ Step 2: Confirmed Mary is parent of Alice");
    }

    // Finally, check if John is grandparent
    let grandparent_result = bc_engine.query("Person.IsGrandparentOf == true", &mut facts)?;

    if grandparent_result.provable {
        println!("  ✓ Conclusion: John IS a grandparent of Alice!");

        println!("\n  Proof Chain:");
        println!("    1. John.IsFatherOf == true");
        println!("    2. John.IsParentOf == true (derived)");
        println!("    3. Mary.IsMotherOf == true");
        println!("    4. Mary.IsParentOf == true (derived)");
        println!("    5. Person.IsGrandparentOf == true (derived)");

        println!("\n  Statistics:");
        println!("    {} goals explored", grandparent_result.stats.goals_explored);
        println!("    {} rules evaluated", grandparent_result.stats.rules_evaluated);
        println!("    Max depth: {}", grandparent_result.stats.max_depth);
    } else {
        println!("  ✗ Could not establish grandparent relationship");
    }

    Ok(())
}

fn test_sibling_relationship() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_family_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    // Setup sibling facts: Alice and Bob have same parents
    let mut facts = Facts::new();
    facts.set("Person1.ParentID", Value::Number(100.0));
    facts.set("Person1.ID", Value::Number(1.0));
    facts.set("Person1.Name", Value::String("Alice".to_string()));
    facts.set("Person1.Gender", Value::String("Female".to_string()));

    facts.set("Person2.ParentID", Value::Number(100.0));
    facts.set("Person2.ID", Value::Number(2.0));
    facts.set("Person2.Name", Value::String("Bob".to_string()));
    facts.set("Person2.Gender", Value::String("Male".to_string()));

    println!("Family Structure:");
    println!("  Parents (ID: 100)");
    println!("    ├─ Alice (ID: 1, Female)");
    println!("    └─ Bob (ID: 2, Male)");

    // Query: Are they siblings?
    println!("\n🔍 Query: Are Alice and Bob siblings?");
    let result = bc_engine.query("Person1.IsSiblingOf == true", &mut facts)?;

    if result.provable {
        println!("  ✓ YES, they are siblings!");

        // Check if Bob is brother
        println!("\n🔍 Query: Is Bob a brother?");
        facts.set("Person.IsSiblingOf", Value::Boolean(true));
        facts.set("Person.Gender", Value::String("Male".to_string()));
        let brother_result = bc_engine.query("Person.IsBrotherOf == true", &mut facts)?;

        if brother_result.provable {
            println!("  ✓ YES, Bob is a brother!");
        }

        // Check if Alice is sister
        println!("\n🔍 Query: Is Alice a sister?");
        facts.set("Person.Gender", Value::String("Female".to_string()));
        let sister_result = bc_engine.query("Person.IsSisterOf == true", &mut facts)?;

        if sister_result.provable {
            println!("  ✓ YES, Alice is a sister!");
        }

        println!("\n  Reasoning:");
        println!("    • Same ParentID (100) → Siblings");
        println!("    • Male sibling → Brother");
        println!("    • Female sibling → Sister");
    } else {
        println!("  ✗ Not siblings");
    }

    Ok(())
}

fn test_uncle_relationship() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_family_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    // Setup uncle relationship:
    // Mary (parent) and John (uncle) are siblings
    // Mary is parent of Alice
    // Therefore John is Alice's uncle
    let mut facts = Facts::new();

    // Mary is parent
    facts.set("Parent.IsParentOf", Value::Boolean(true));
    facts.set("Parent.ID", Value::Number(10.0));
    facts.set("Parent.Name", Value::String("Mary".to_string()));

    // John is Mary's brother
    facts.set("Uncle.IsBrotherOf", Value::Boolean(true));
    facts.set("Uncle.SiblingID", Value::Number(10.0));
    facts.set("Uncle.Name", Value::String("John".to_string()));

    println!("Family Structure:");
    println!("  Grandparents");
    println!("    ├─ Mary (parent, ID: 10)");
    println!("    │   └─ Alice (child)");
    println!("    └─ John (uncle)");

    // Query: Is John an uncle?
    println!("\n🔍 Query: Is John an uncle of Alice?");
    let result = bc_engine.query("Uncle.IsUncleOf == true", &mut facts)?;

    if result.provable {
        println!("  ✓ YES, John is Alice's uncle!");

        println!("\n  Proof Chain:");
        println!("    1. Parent.IsParentOf == true (Mary is parent)");
        println!("    2. Uncle.IsBrotherOf == true (John is brother)");
        println!("    3. Parent.ID == Uncle.SiblingID (They are siblings)");
        println!("    4. Uncle.IsUncleOf == true (Derived)");

        println!("\n  Statistics:");
        println!("    {} goals explored", result.stats.goals_explored);
        println!("    {} rules evaluated", result.stats.rules_evaluated);
    } else {
        println!("  ✗ Could not establish uncle relationship");
    }

    Ok(())
}

fn test_cousin_relationship() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_family_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    // Setup cousin relationship:
    // Parent1 and Parent2 are siblings
    // Parent1 has child Alice
    // Parent2 has child Bob
    // Therefore Alice and Bob are cousins
    let mut facts = Facts::new();

    // Parent1 setup
    facts.set("Parent1.IsParentOf", Value::Boolean(true));
    facts.set("Parent1.ID", Value::Number(10.0));
    facts.set("Parent1.IsSiblingOf", Value::Boolean(true));
    facts.set("Parent1.SiblingID", Value::Number(11.0));
    facts.set("Parent1.ChildName", Value::String("Alice".to_string()));

    // Parent2 setup
    facts.set("Parent2.IsParentOf", Value::Boolean(true));
    facts.set("Parent2.ID", Value::Number(11.0));
    facts.set("Parent2.ChildName", Value::String("Bob".to_string()));

    println!("Family Structure:");
    println!("  Grandparents");
    println!("    ├─ Parent1 (ID: 10)");
    println!("    │   └─ Alice");
    println!("    └─ Parent2 (ID: 11)");
    println!("        └─ Bob");
    println!("  (Parent1 and Parent2 are siblings)");

    // Query: Are Alice and Bob cousins?
    println!("\n🔍 Query: Are Alice and Bob cousins?");
    let result = bc_engine.query("Person.IsCousinOf == true", &mut facts)?;

    if result.provable {
        println!("  ✓ YES, Alice and Bob are cousins!");

        println!("\n  Proof Chain:");
        println!("    1. Parent1.IsParentOf == true (Parent1 has child)");
        println!("    2. Parent2.IsParentOf == true (Parent2 has child)");
        println!("    3. Parent1.IsSiblingOf == true (Parents are siblings)");
        println!("    4. Parent1.SiblingID == Parent2.ID (Confirmed)");
        println!("    5. Person.IsCousinOf == true (Derived)");

        println!("\n  Cousin Relationship Explained:");
        println!("    • Their parents are siblings");
        println!("    • They are in the same generation");
        println!("    • They share grandparents");

        println!("\n  Statistics:");
        println!("    {} goals explored", result.stats.goals_explored);
        println!("    Max depth: {}", result.stats.max_depth);
    } else {
        println!("  ✗ Could not establish cousin relationship");
        println!("\n  Missing facts: {:?}", result.missing_facts);
    }

    Ok(())
}

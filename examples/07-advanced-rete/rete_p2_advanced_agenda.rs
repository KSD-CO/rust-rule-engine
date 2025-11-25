/// Demo: Advanced Agenda Features (P2 - Drools-style)
///
/// This example demonstrates advanced agenda features similar to Drools:
/// - Activation Groups: Only one rule in a group fires
/// - Agenda Groups: Sequential execution of rule groups
/// - Ruleflow Groups: Workflow-based execution
/// - Auto-focus: Automatic agenda group switching
/// - Lock-on-active: Prevent re-activation
/// - Salience/Priority: Rule firing order

use rust_rule_engine::rete::agenda::{AdvancedAgenda, Activation};

fn main() {
    println!("\n🎯 Advanced Agenda Demo (Drools-style)");
    println!("========================================\n");

    // Example 1: Basic Priority (Salience)
    println!("📋 Example 1: Priority/Salience");
    println!("-------------------------------");

    let mut agenda = AdvancedAgenda::new();

    agenda.add_activation(Activation::new("LowPriority".to_string(), 1));
    agenda.add_activation(Activation::new("MediumPriority".to_string(), 5));
    agenda.add_activation(Activation::new("HighPriority".to_string(), 10));

    println!("Added 3 rules with different priorities (1, 5, 10)");
    println!("Firing order (should be by priority):");

    while let Some(activation) = agenda.get_next_activation() {
        println!("  🔥 {} (salience: {})", activation.rule_name, activation.salience);
        agenda.mark_rule_fired(&activation);
    }

    // Example 2: Activation Groups
    println!("\n📋 Example 2: Activation Groups");
    println!("-------------------------------");
    println!("Only ONE rule in an activation group can fire\n");

    let mut agenda2 = AdvancedAgenda::new();

    agenda2.add_activation(
        Activation::new("Rule1".to_string(), 10)
            .with_activation_group("discount_group".to_string())
    );
    agenda2.add_activation(
        Activation::new("Rule2".to_string(), 20)
            .with_activation_group("discount_group".to_string())
    );
    agenda2.add_activation(
        Activation::new("Rule3".to_string(), 5)
            .with_activation_group("discount_group".to_string())
    );

    println!("Added 3 rules in 'discount_group' with salience 10, 20, 5");
    println!("Only the highest priority rule should fire:\n");

    let mut fired_count = 0;
    while let Some(activation) = agenda2.get_next_activation() {
        println!("  🔥 {} (salience: {})", activation.rule_name, activation.salience);
        agenda2.mark_rule_fired(&activation);
        fired_count += 1;
    }

    println!("\nTotal rules fired: {} (should be 1)", fired_count);
    assert_eq!(fired_count, 1, "Only one rule in activation group should fire");

    // Example 3: Agenda Groups
    println!("\n📋 Example 3: Agenda Groups");
    println!("--------------------------");
    println!("Sequential execution of rule groups\n");

    let mut agenda3 = AdvancedAgenda::new();

    // Add rules to different agenda groups
    agenda3.add_activation(
        Activation::new("InitRule1".to_string(), 10)
            .with_agenda_group("initialization".to_string())
    );
    agenda3.add_activation(
        Activation::new("InitRule2".to_string(), 5)
            .with_agenda_group("initialization".to_string())
    );

    agenda3.add_activation(
        Activation::new("ProcessRule1".to_string(), 10)
            .with_agenda_group("processing".to_string())
    );
    agenda3.add_activation(
        Activation::new("ProcessRule2".to_string(), 5)
            .with_agenda_group("processing".to_string())
    );

    agenda3.add_activation(
        Activation::new("CleanupRule".to_string(), 10)
            .with_agenda_group("cleanup".to_string())
    );

    println!("Current focus: {}", agenda3.get_focus());
    println!("No activations in MAIN group, so nothing fires initially\n");

    // Set focus to initialization group
    println!("Setting focus to 'initialization'...");
    agenda3.set_focus("initialization".to_string());

    while let Some(activation) = agenda3.get_next_activation() {
        if activation.agenda_group == "initialization" {
            println!("  🔥 {} (group: {})", activation.rule_name, activation.agenda_group);
            agenda3.mark_rule_fired(&activation);
        } else {
            break;
        }
    }

    // Set focus to processing
    println!("\nSetting focus to 'processing'...");
    agenda3.set_focus("processing".to_string());

    while let Some(activation) = agenda3.get_next_activation() {
        if activation.agenda_group == "processing" {
            println!("  🔥 {} (group: {})", activation.rule_name, activation.agenda_group);
            agenda3.mark_rule_fired(&activation);
        } else {
            break;
        }
    }

    // Set focus to cleanup
    println!("\nSetting focus to 'cleanup'...");
    agenda3.set_focus("cleanup".to_string());

    while let Some(activation) = agenda3.get_next_activation() {
        println!("  🔥 {} (group: {})", activation.rule_name, activation.agenda_group);
        agenda3.mark_rule_fired(&activation);
    }

    // Example 4: Auto-Focus
    println!("\n📋 Example 4: Auto-Focus");
    println!("-----------------------");
    println!("Automatically switch to agenda group when rule is added\n");

    let mut agenda4 = AdvancedAgenda::new();

    println!("Current focus: {}", agenda4.get_focus());

    agenda4.add_activation(
        Activation::new("UrgentRule".to_string(), 100)
            .with_agenda_group("urgent".to_string())
            .with_auto_focus(true)
    );

    println!("Added rule with auto-focus to 'urgent' group");
    println!("Current focus: {} (automatically switched!)", agenda4.get_focus());

    if let Some(activation) = agenda4.get_next_activation() {
        println!("  🔥 {} fires immediately", activation.rule_name);
    }

    // Example 5: Ruleflow Groups
    println!("\n📋 Example 5: Ruleflow Groups");
    println!("----------------------------");
    println!("Workflow-based rule execution\n");

    let mut agenda5 = AdvancedAgenda::new();

    agenda5.add_activation(
        Activation::new("ValidationRule".to_string(), 10)
            .with_ruleflow_group("validation".to_string())
    );

    println!("Added rule to 'validation' ruleflow group");
    println!("Ruleflow group not active yet, so rule doesn't add to agenda");
    println!("Activations in agenda: {}", agenda5.stats().total_activations);

    // Activate the ruleflow group
    agenda5.activate_ruleflow_group("validation".to_string());
    println!("\nActivated 'validation' ruleflow group");

    agenda5.add_activation(
        Activation::new("ValidationRule2".to_string(), 5)
            .with_ruleflow_group("validation".to_string())
    );

    println!("Now the rule is added to agenda");
    println!("Activations in agenda: {}", agenda5.stats().total_activations);

    // Example 6: Lock-on-Active
    println!("\n📋 Example 6: Lock-on-Active");
    println!("---------------------------");
    println!("Prevent rules from re-activating while group is locked\n");

    let mut agenda6 = AdvancedAgenda::new();

    agenda6.add_activation(
        Activation::new("ProcessingRule".to_string(), 10)
            .with_lock_on_active(true)
    );

    if let Some(activation) = agenda6.get_next_activation() {
        println!("  🔥 {} fires (lock-on-active = true)", activation.rule_name);
        agenda6.mark_rule_fired(&activation);
        println!("  Agenda group '{}' is now locked", activation.agenda_group);
    }

    // Try to add another activation to the same group
    agenda6.add_activation(
        Activation::new("AnotherRule".to_string(), 20)
            .with_lock_on_active(true)
    );

    println!("\nAdded another rule with lock-on-active");
    if agenda6.get_next_activation().is_none() {
        println!("  ❌ Rule does not fire (group is locked)");
    }

    // Statistics
    println!("\n📊 Agenda Statistics");
    println!("===================");

    let stats = agenda5.stats();
    println!("{}", stats);
    println!("Total activations: {}", stats.total_activations);
    println!("Groups: {}", stats.groups);
    println!("Fired rules: {}", stats.fired_rules);
    println!("Active ruleflow groups: {}", stats.active_ruleflow_groups);

    // Summary
    println!("\n✨ Advanced Agenda Features");
    println!("===========================");
    println!("✅ Salience/Priority: Control firing order");
    println!("✅ Activation Groups: Only one rule in group fires");
    println!("✅ Agenda Groups: Sequential group execution");
    println!("✅ Auto-Focus: Automatic group switching");
    println!("✅ Ruleflow Groups: Workflow-based execution");
    println!("✅ Lock-on-Active: Prevent re-activation");
    println!("✅ No-Loop: Prevent infinite loops");
    println!("\n🚀 Similar to Drools agenda control attributes!");
}

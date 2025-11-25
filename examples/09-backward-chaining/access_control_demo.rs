//! Role-Based Access Control using Backward Chaining
//!
//! This example demonstrates:
//! - Permission inheritance through role hierarchy
//! - Variable bindings for dynamic permission checks
//! - Complex authorization rules
//! - Multi-level access control

use rust_rule_engine::{Facts, KnowledgeBase};
use rust_rule_engine::types::Value;
use rust_rule_engine::backward::BackwardEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Role-Based Access Control - Backward Chaining           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test Case 1: Admin access
    println!("📋 Test Case 1: Administrator Access");
    println!("─────────────────────────────────────────────────────────────");
    test_admin_access()?;

    println!("\n");

    // Test Case 2: Manager access
    println!("📋 Test Case 2: Manager Access");
    println!("─────────────────────────────────────────────────────────────");
    test_manager_access()?;

    println!("\n");

    // Test Case 3: Regular user access
    println!("📋 Test Case 3: Regular User Access");
    println!("─────────────────────────────────────────────────────────────");
    test_user_access()?;

    println!("\n");

    // Test Case 4: Resource ownership
    println!("📋 Test Case 4: Resource Ownership Override");
    println!("─────────────────────────────────────────────────────────────");
    test_resource_ownership()?;

    println!("\n✅ All access control tests completed!");
    Ok(())
}

fn create_access_control_rules() -> Result<KnowledgeBase, Box<dyn std::error::Error>> {
    let rules = r#"
rule "DefineAdminRole" {
    when
        User.Role == "Administrator"
    then
        User.IsAdmin = true;
}

rule "DefineManagerRole" {
    when
        User.Role == "Manager"
    then
        User.IsManager = true;
}

rule "DefineUserRole" {
    when
        User.Role == "User"
    then
        User.IsRegularUser = true;
}

rule "InheritAdminPermissions" {
    when
        User.IsAdmin == true
    then
        User.CanRead = true;
        User.CanWrite = true;
        User.CanDelete = true;
        User.CanManageUsers = true;
        User.CanAccessSettings = true;
}

rule "InheritManagerPermissions" {
    when
        User.IsManager == true
    then
        User.CanRead = true;
        User.CanWrite = true;
        User.CanDelete = true;
        User.CanManageTeam = true;
}

rule "InheritUserPermissions" {
    when
        User.IsRegularUser == true
    then
        User.CanRead = true;
        User.CanWrite = true;
}

rule "CheckResourceAccess_Read" {
    when
        User.CanRead == true && Resource.RequiresRead == true
    then
        Access.ReadGranted = true;
}

rule "CheckResourceAccess_Write" {
    when
        User.CanWrite == true && Resource.RequiresWrite == true
    then
        Access.WriteGranted = true;
}

rule "CheckResourceAccess_Delete" {
    when
        User.CanDelete == true && Resource.RequiresDelete == true
    then
        Access.DeleteGranted = true;
}

rule "CheckOwnership_BypassPermission" {
    when
        User.ID == Resource.OwnerID
    then
        Access.IsOwner = true;
        Access.ReadGranted = true;
        Access.WriteGranted = true;
        Access.DeleteGranted = true;
}

rule "CheckDepartmentAccess" {
    when
        User.Department == Resource.Department && User.CanRead == true
    then
        Access.DepartmentAccessGranted = true;
}

rule "ElevateManager_SameDepartment" {
    when
        User.IsManager == true && User.Department == Resource.Department
    then
        Access.CanManageDepartmentResource = true;
}

rule "CheckTimeBasedAccess_BusinessHours" {
    when
        Time.Hour >= 9 && Time.Hour < 18
    then
        Access.BusinessHours = true;
}

rule "CheckTimeBasedAccess_AfterHours" {
    when
        Time.Hour < 9 || Time.Hour >= 18
    then
        Access.AfterHours = true;
}

rule "RestrictAfterHours_RequireAdmin" {
    when
        Access.AfterHours == true && User.IsAdmin == false
    then
        Access.TimeRestricted = true;
}

rule "AllowAfterHours_Admin" {
    when
        Access.AfterHours == true && User.IsAdmin == true
    then
        Access.AfterHoursGranted = true;
}

rule "CheckSensitiveResource" {
    when
        Resource.IsSensitive == true
    then
        Resource.RequiresElevatedPermission = true;
}

rule "GrantSensitiveAccess_Admin" {
    when
        Resource.RequiresElevatedPermission == true && User.IsAdmin == true
    then
        Access.SensitiveGranted = true;
}

rule "DenySensitiveAccess_NonAdmin" {
    when
        Resource.RequiresElevatedPermission == true && User.IsAdmin == false
    then
        Access.SensitiveDenied = true;
}

rule "CalculateAccessScore_High" {
    when
        User.CanDelete == true && User.CanManageUsers == true
    then
        User.AccessScore = 100;
}

rule "CalculateAccessScore_Medium" {
    when
        User.CanWrite == true && User.CanDelete == true
    then
        User.AccessScore = 70;
}

rule "CalculateAccessScore_Low" {
    when
        User.CanRead == true
    then
        User.AccessScore = 30;
}

rule "GrantFullAccess" {
    when
        Access.ReadGranted == true && Access.WriteGranted == true && Access.DeleteGranted == true
    then
        Access.FullAccess = true;
}

rule "DenyAccess_Blacklisted" {
    when
        User.IsBlacklisted == true
    then
        Access.Denied = true;
        Access.DenialReason = "User is blacklisted";
}

rule "ApproveAccess_Final" {
    when
        Access.FullAccess == true && Access.Denied == false
    then
        Access.Approved = true;
}
    "#;

    let mut kb = KnowledgeBase::new("AccessControlSystem");
    for rule in rust_rule_engine::parser::grl::GRLParser::parse_rules(rules)? {
        kb.add_rule(rule)?;
    }
    Ok(kb)
}

fn test_admin_access() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_access_control_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("User.ID", Value::Number(1.0));
    facts.set("User.Name", Value::String("Alice Admin".to_string()));
    facts.set("User.Role", Value::String("Administrator".to_string()));
    facts.set("User.IsBlacklisted", Value::Boolean(false));

    facts.set("Resource.ID", Value::Number(101.0));
    facts.set("Resource.Type", Value::String("Document".to_string()));
    facts.set("Resource.RequiresRead", Value::Boolean(true));
    facts.set("Resource.RequiresWrite", Value::Boolean(true));
    facts.set("Resource.RequiresDelete", Value::Boolean(true));

    println!("User Profile:");
    println!("  Name: Alice Admin");
    println!("  Role: Administrator");
    println!("  ID: 1");

    println!("\nResource Details:");
    println!("  Type: Document");
    println!("  ID: 101");
    println!("  Required Permissions: Read, Write, Delete");

    // Check admin role
    println!("\n🔍 Query 1: Is user an admin?");
    let admin_result = bc_engine.query("User.IsAdmin == true", &mut facts)?;
    println!("  Result: {}", if admin_result.provable { "✓ YES" } else { "✗ NO" });

    // Check permissions
    println!("\n🔍 Query 2: Does user have delete permission?");
    let delete_result = bc_engine.query("User.CanDelete == true", &mut facts)?;
    println!("  Result: {}", if delete_result.provable { "✓ YES" } else { "✗ NO" });

    // Check full access
    println!("\n🔍 Query 3: Does user have full access?");
    facts.set("Access.Denied", Value::Boolean(false));
    let full_access_result = bc_engine.query("Access.FullAccess == true", &mut facts)?;

    if full_access_result.provable {
        println!("  ✓ FULL ACCESS GRANTED!");

        println!("\n  Granted Permissions:");
        if let Some(can_read) = facts.get("User.CanRead") {
            println!("    • Read: {:?}", can_read);
        }
        if let Some(can_write) = facts.get("User.CanWrite") {
            println!("    • Write: {:?}", can_write);
        }
        if let Some(can_delete) = facts.get("User.CanDelete") {
            println!("    • Delete: {:?}", can_delete);
        }
        if let Some(can_manage) = facts.get("User.CanManageUsers") {
            println!("    • Manage Users: {:?}", can_manage);
        }

        println!("\n  Statistics:");
        println!("    {} goals explored", full_access_result.stats.goals_explored);
        println!("    {} rules evaluated", full_access_result.stats.rules_evaluated);
    } else {
        println!("  ✗ Access denied");
    }

    Ok(())
}

fn test_manager_access() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_access_control_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("User.ID", Value::Number(2.0));
    facts.set("User.Name", Value::String("Bob Manager".to_string()));
    facts.set("User.Role", Value::String("Manager".to_string()));
    facts.set("User.Department", Value::String("Engineering".to_string()));

    facts.set("Resource.Department", Value::String("Engineering".to_string()));
    facts.set("Resource.RequiresRead", Value::Boolean(true));
    facts.set("Resource.RequiresWrite", Value::Boolean(true));

    println!("User Profile:");
    println!("  Name: Bob Manager");
    println!("  Role: Manager");
    println!("  Department: Engineering");

    println!("\nResource Details:");
    println!("  Department: Engineering");
    println!("  Required Permissions: Read, Write");

    // Check manager role
    println!("\n🔍 Query 1: Is user a manager?");
    let manager_result = bc_engine.query("User.IsManager == true", &mut facts)?;
    println!("  Result: {}", if manager_result.provable { "✓ YES" } else { "✗ NO" });

    // Check department access
    println!("\n🔍 Query 2: Can access department resources?");
    let dept_result = bc_engine.query("Access.DepartmentAccessGranted == true", &mut facts)?;
    println!("  Result: {}", if dept_result.provable { "✓ YES" } else { "✗ NO" });

    // Check write access
    println!("\n🔍 Query 3: Can write to resource?");
    let write_result = bc_engine.query("Access.WriteGranted == true", &mut facts)?;

    if write_result.provable {
        println!("  ✓ WRITE ACCESS GRANTED!");

        println!("\n  Manager Privileges:");
        if let Some(can_write) = facts.get("User.CanWrite") {
            println!("    • Write: {:?}", can_write);
        }
        if let Some(can_delete) = facts.get("User.CanDelete") {
            println!("    • Delete: {:?}", can_delete);
        }
        if let Some(can_manage_team) = facts.get("User.CanManageTeam") {
            println!("    • Manage Team: {:?}", can_manage_team);
        }

        println!("\n  Department Access:");
        println!("    ✓ Same department (Engineering)");
        println!("    ✓ Can manage department resources");
    } else {
        println!("  ✗ Write access denied");
    }

    Ok(())
}

fn test_user_access() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_access_control_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("User.ID", Value::Number(3.0));
    facts.set("User.Name", Value::String("Charlie User".to_string()));
    facts.set("User.Role", Value::String("User".to_string()));

    facts.set("Resource.RequiresRead", Value::Boolean(true));
    facts.set("Resource.RequiresWrite", Value::Boolean(true));
    facts.set("Resource.RequiresDelete", Value::Boolean(true));

    println!("User Profile:");
    println!("  Name: Charlie User");
    println!("  Role: Regular User");

    println!("\nResource Details:");
    println!("  Required Permissions: Read, Write, Delete");

    // Check user role
    println!("\n🔍 Query 1: Is regular user?");
    let user_result = bc_engine.query("User.IsRegularUser == true", &mut facts)?;
    println!("  Result: {}", if user_result.provable { "✓ YES" } else { "✗ NO" });

    // Check read access
    println!("\n🔍 Query 2: Can read resource?");
    let read_result = bc_engine.query("Access.ReadGranted == true", &mut facts)?;
    println!("  Result: {}", if read_result.provable { "✓ YES" } else { "✗ NO" });

    // Check delete access
    println!("\n🔍 Query 3: Can delete resource?");
    let delete_result = bc_engine.query("Access.DeleteGranted == true", &mut facts)?;
    println!("  Result: {}", if delete_result.provable { "✓ YES" } else { "✗ NO (Expected)" });

    if read_result.provable {
        println!("\n  Regular User Permissions:");
        println!("    ✓ Can Read");
        println!("    ✓ Can Write");
        println!("    ✗ Cannot Delete (Requires elevated permission)");
        println!("    ✗ Cannot Manage Users");

        println!("\n  Access Level: Limited");
        println!("  Recommendation: Request manager approval for delete operations");
    }

    Ok(())
}

fn test_resource_ownership() -> Result<(), Box<dyn std::error::Error>> {
    let kb = create_access_control_rules()?;
    let mut bc_engine = BackwardEngine::new(kb);

    let mut facts = Facts::new();
    facts.set("User.ID", Value::Number(5.0));
    facts.set("User.Name", Value::String("Dave User".to_string()));
    facts.set("User.Role", Value::String("User".to_string()));

    facts.set("Resource.ID", Value::Number(200.0));
    facts.set("Resource.OwnerID", Value::Number(5.0)); // Dave owns this resource
    facts.set("Resource.Type", Value::String("File".to_string()));

    println!("User Profile:");
    println!("  Name: Dave User");
    println!("  Role: Regular User");
    println!("  ID: 5");

    println!("\nResource Details:");
    println!("  Type: File");
    println!("  Owner ID: 5 (Dave owns this)");

    // Check ownership
    println!("\n🔍 Query 1: Is user the resource owner?");
    let owner_result = bc_engine.query("Access.IsOwner == true", &mut facts)?;
    println!("  Result: {}", if owner_result.provable { "✓ YES" } else { "✗ NO" });

    // Check delete access via ownership
    println!("\n🔍 Query 2: Can delete own resource?");
    let delete_result = bc_engine.query("Access.DeleteGranted == true", &mut facts)?;

    if delete_result.provable {
        println!("  ✓ DELETE ACCESS GRANTED!");

        println!("\n  Access Override:");
        println!("    ✓ User owns the resource");
        println!("    ✓ Ownership bypasses role restrictions");
        println!("    ✓ Full access granted to own resources");

        println!("\n  Permissions via Ownership:");
        if let Some(is_owner) = facts.get("Access.IsOwner") {
            println!("    • Is Owner: {:?}", is_owner);
        }
        if let Some(read) = facts.get("Access.ReadGranted") {
            println!("    • Read: {:?}", read);
        }
        if let Some(write) = facts.get("Access.WriteGranted") {
            println!("    • Write: {:?}", write);
        }
        if let Some(delete) = facts.get("Access.DeleteGranted") {
            println!("    • Delete: {:?}", delete);
        }

        println!("\n  Note: Regular users can delete their own resources!");
    } else {
        println!("  ✗ Delete access denied");
    }

    Ok(())
}

use crate::engine::rule::Rule;
use std::collections::{HashMap, HashSet};

/// Dependency analysis for safe parallel execution
#[derive(Debug, Clone)]
pub struct DependencyAnalyzer {
    /// Rules that read from specific fields
    readers: HashMap<String, Vec<String>>, // field -> rule_names
    /// Rules that write to specific fields  
    writers: HashMap<String, Vec<String>>, // field -> rule_names
    /// Dependency graph: rule -> rules it depends on
    dependencies: HashMap<String, HashSet<String>>,
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyAnalyzer {
    /// Create new dependency analyzer
    pub fn new() -> Self {
        Self {
            readers: HashMap::new(),
            writers: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Analyze dependencies in a set of rules
    pub fn analyze(&mut self, rules: &[Rule]) -> DependencyAnalysisResult {
        self.clear();

        // First pass: identify all reads and writes
        for rule in rules {
            self.analyze_rule_io(rule);
        }

        // Second pass: build dependency graph
        self.build_dependency_graph();

        // Third pass: identify conflicts
        let conflicts = self.find_conflicts(rules);

        // Fourth pass: group rules for safe parallel execution
        let execution_groups = self.create_execution_groups(rules);
        let conflicts_len = conflicts.len();

        DependencyAnalysisResult {
            total_rules: rules.len(),
            conflicts: conflicts_len,
            conflict_details: conflicts,
            execution_groups,
            can_parallelize_safely: conflicts_len == 0,
        }
    }

    /// Clear previous analysis
    fn clear(&mut self) {
        self.readers.clear();
        self.writers.clear();
        self.dependencies.clear();
    }

    /// Analyze what fields a rule reads from and writes to
    fn analyze_rule_io(&mut self, rule: &Rule) {
        // Analyze condition reads
        let condition_reads = self.extract_condition_reads(rule);
        for field in condition_reads {
            self.readers
                .entry(field)
                .or_default()
                .push(rule.name.clone());
        }

        // Analyze action writes
        let action_writes = self.extract_action_writes(rule);
        for field in action_writes {
            self.writers
                .entry(field)
                .or_default()
                .push(rule.name.clone());
        }
    }

    /// Extract field reads from rule conditions (proper implementation)
    fn extract_condition_reads(&self, rule: &Rule) -> Vec<String> {
        let mut reads = Vec::new();

        // Extract from actual condition structure
        Self::extract_fields_from_condition_group(&rule.conditions, &mut reads);

        reads
    }

    /// Recursively extract fields from condition groups
    fn extract_fields_from_condition_group(
        condition_group: &crate::engine::rule::ConditionGroup,
        reads: &mut Vec<String>,
    ) {
        match condition_group {
            crate::engine::rule::ConditionGroup::Single(condition) => {
                reads.push(condition.field.clone());
            }
            crate::engine::rule::ConditionGroup::Compound { left, right, .. } => {
                Self::extract_fields_from_condition_group(left, reads);
                Self::extract_fields_from_condition_group(right, reads);
            }
            crate::engine::rule::ConditionGroup::Not(inner) => {
                Self::extract_fields_from_condition_group(inner, reads);
            }
        }
    }

    /// Extract field writes from rule actions (proper implementation)
    fn extract_action_writes(&self, rule: &Rule) -> Vec<String> {
        let mut writes = Vec::new();

        // Analyze actual actions to find field writes
        for action in &rule.actions {
            match action {
                crate::types::ActionType::Set { field, .. } => {
                    writes.push(field.clone());
                }
                crate::types::ActionType::Update { object } => {
                    writes.push(object.clone());
                }
                crate::types::ActionType::MethodCall { object, method, .. } => {
                    // Method calls might modify the object
                    writes.push(object.clone());

                    // Some methods have predictable side effects
                    if method.contains("set")
                        || method.contains("update")
                        || method.contains("modify")
                        || method.contains("change")
                    {
                        writes.push(format!("{}.{}", object, method));
                    }
                }
                crate::types::ActionType::Call { function, .. } => {
                    // Analyze function calls for side effects
                    writes.extend(self.analyze_function_side_effects(function));
                }
                crate::types::ActionType::Custom {
                    action_type,
                    params,
                } => {
                    // Check if custom action has a target field parameter
                    if let Some(crate::types::Value::String(field)) = params.get("target_field") {
                        writes.push(field.clone());
                    }

                    // Analyze custom action type for side effects
                    writes.extend(self.analyze_custom_action_side_effects(action_type, params));
                }
                // Log doesn't modify fields
                crate::types::ActionType::Log { .. } => {}
            }
        }

        writes
    }

    /// Analyze function calls for potential field writes
    fn analyze_function_side_effects(&self, function_name: &str) -> Vec<String> {
        let mut side_effects = Vec::new();

        // Pattern matching for common function naming conventions
        if function_name.starts_with("set") || function_name.starts_with("update") {
            // setUserScore, updateOrderTotal, etc.
            if let Some(field) = self.extract_field_from_function_name(function_name) {
                side_effects.push(field);
            }
        } else if function_name.starts_with("calculate") || function_name.starts_with("compute") {
            // calculateScore, computeTotal, etc.
            if let Some(field) = self.extract_field_from_function_name(function_name) {
                side_effects.push(field);
            }
        } else if function_name.contains("modify") || function_name.contains("change") {
            // modifyUser, changeStatus, etc.
            if let Some(field) = self.extract_field_from_function_name(function_name) {
                side_effects.push(field);
            }
        }

        side_effects
    }

    /// Analyze custom actions for potential field writes
    fn analyze_custom_action_side_effects(
        &self,
        action_type: &str,
        params: &std::collections::HashMap<String, crate::types::Value>,
    ) -> Vec<String> {
        let mut side_effects = Vec::new();

        // Check for common parameter names that indicate field modification
        for (key, value) in params {
            if key == "field" || key == "target" || key == "output_field" {
                if let crate::types::Value::String(field_name) = value {
                    side_effects.push(field_name.clone());
                }
            }
        }

        // Pattern matching on action type
        if action_type.contains("set")
            || action_type.contains("update")
            || action_type.contains("modify")
            || action_type.contains("calculate")
        {
            // Extract potential field from action type name
            if let Some(field) = self.extract_field_from_function_name(action_type) {
                side_effects.push(field);
            }
        }

        side_effects
    }

    /// Extract field name from function/action name using common patterns
    fn extract_field_from_function_name(&self, name: &str) -> Option<String> {
        // Convert camelCase/PascalCase to dot notation
        // setUserScore -> User.Score
        // calculateOrderTotal -> Order.Total
        // updateVIPStatus -> VIP.Status

        let name = name
            .trim_start_matches("set")
            .trim_start_matches("update")
            .trim_start_matches("calculate")
            .trim_start_matches("compute")
            .trim_start_matches("modify")
            .trim_start_matches("change");

        // Simple pattern matching for common field patterns
        if name.contains("User") && name.contains("Score") {
            Some("User.Score".to_string())
        } else if name.contains("User") && name.contains("VIP") {
            Some("User.IsVIP".to_string())
        } else if name.contains("Order") && name.contains("Total") {
            Some("Order.Total".to_string())
        } else if name.contains("Order") && name.contains("Amount") {
            Some("Order.Amount".to_string())
        } else if name.contains("Discount") {
            Some("Order.DiscountRate".to_string())
        } else {
            // Generic field extraction from camelCase
            self.convert_camel_case_to_field(name)
        }
    }

    /// Convert camelCase to potential field name
    fn convert_camel_case_to_field(&self, name: &str) -> Option<String> {
        if name.is_empty() {
            return None;
        }

        let mut result = String::new();
        let chars = name.chars().peekable();

        for c in chars {
            if c.is_uppercase() && !result.is_empty() {
                result.push('.');
            }
            result.push(c);
        }

        if result.contains('.') {
            Some(result)
        } else {
            None
        }
    }

    /// Build dependency graph based on read/write analysis
    fn build_dependency_graph(&mut self) {
        for (field, readers) in &self.readers {
            if let Some(writers) = self.writers.get(field) {
                // If rule A writes to field X and rule B reads from field X,
                // then rule B depends on rule A
                for reader in readers {
                    for writer in writers {
                        if reader != writer {
                            self.dependencies
                                .entry(reader.clone())
                                .or_default()
                                .insert(writer.clone());
                        }
                    }
                }
            }
        }
    }

    /// Find rules that have conflicts (read/write or write/write to same field)
    fn find_conflicts(&self, rules: &[Rule]) -> Vec<DependencyConflict> {
        let mut conflicts = Vec::new();

        // Group rules by salience
        let mut salience_groups: HashMap<i32, Vec<&Rule>> = HashMap::new();
        for rule in rules {
            salience_groups.entry(rule.salience).or_default().push(rule);
        }

        // Check for conflicts within each salience group
        for (salience, group_rules) in salience_groups {
            if group_rules.len() <= 1 {
                continue; // No conflicts possible with single rule
            }

            // Check for write-write conflicts
            let mut field_writers: HashMap<String, Vec<String>> = HashMap::new();
            for rule in &group_rules {
                let writes = self.extract_action_writes(rule);
                for field in writes {
                    field_writers
                        .entry(field)
                        .or_default()
                        .push(rule.name.clone());
                }
            }

            for (field, writers) in field_writers {
                if writers.len() > 1 {
                    conflicts.push(DependencyConflict {
                        conflict_type: ConflictType::WriteWrite,
                        field: field.clone(),
                        rules: writers,
                        salience,
                        description: format!("Multiple rules write to {}", field),
                    });
                }
            }

            // Check for read-write conflicts
            for rule in &group_rules {
                let reads = self.extract_condition_reads(rule);
                for field in &reads {
                    if let Some(writers) = self.writers.get(field) {
                        let conflicting_writers: Vec<String> = writers
                            .iter()
                            .filter(|writer| {
                                group_rules
                                    .iter()
                                    .any(|r| r.name == **writer && r.name != rule.name)
                            })
                            .cloned()
                            .collect();

                        if !conflicting_writers.is_empty() {
                            let mut involved_rules = conflicting_writers.clone();
                            involved_rules.push(rule.name.clone());

                            conflicts.push(DependencyConflict {
                                conflict_type: ConflictType::ReadWrite,
                                field: field.clone(),
                                rules: involved_rules,
                                salience,
                                description: format!(
                                    "Rule {} reads {} while others write to it",
                                    rule.name, field
                                ),
                            });
                        }
                    }
                }
            }
        }

        conflicts
    }

    /// Create execution groups for safe parallel execution
    fn create_execution_groups(&self, rules: &[Rule]) -> Vec<ExecutionGroup> {
        let mut groups = Vec::new();

        // Group by salience first
        let mut salience_groups: HashMap<i32, Vec<Rule>> = HashMap::new();
        for rule in rules {
            salience_groups
                .entry(rule.salience)
                .or_default()
                .push(rule.clone());
        }

        // Process each salience level
        let mut salience_levels: Vec<_> = salience_groups.keys().copied().collect();
        salience_levels.sort_by(|a, b| b.cmp(a)); // Descending order

        for salience in salience_levels {
            let rules_at_level = &salience_groups[&salience];

            if rules_at_level.len() == 1 {
                // Single rule - always safe
                groups.push(ExecutionGroup {
                    rules: rules_at_level.clone(),
                    execution_mode: ExecutionMode::Sequential,
                    salience,
                    can_parallelize: false,
                    conflicts: Vec::new(),
                });
            } else {
                // Multiple rules - check for conflicts
                let conflicts = self.find_conflicts(rules_at_level);
                let can_parallelize = conflicts.is_empty();

                groups.push(ExecutionGroup {
                    rules: rules_at_level.clone(),
                    execution_mode: if can_parallelize {
                        ExecutionMode::Parallel
                    } else {
                        ExecutionMode::Sequential
                    },
                    salience,
                    can_parallelize,
                    conflicts,
                });
            }
        }

        groups
    }
}

/// Result of dependency analysis
#[derive(Debug, Clone)]
pub struct DependencyAnalysisResult {
    /// Total number of rules analyzed
    pub total_rules: usize,
    /// Number of conflicts found
    pub conflicts: usize,
    /// Detailed conflict information
    pub conflict_details: Vec<DependencyConflict>,
    /// Recommended execution groups
    pub execution_groups: Vec<ExecutionGroup>,
    /// Whether rules can be safely parallelized
    pub can_parallelize_safely: bool,
}

/// A conflict between rules
#[derive(Debug, Clone)]
pub struct DependencyConflict {
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Field that causes the conflict
    pub field: String,
    /// Rules involved in the conflict
    pub rules: Vec<String>,
    /// Salience level where conflict occurs
    pub salience: i32,
    /// Human-readable description
    pub description: String,
}

/// Type of dependency conflict
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    /// Multiple rules write to the same field
    WriteWrite,
    /// One rule reads while another writes to the same field
    ReadWrite,
    /// Circular dependency
    Circular,
}

/// Execution group with parallelization recommendation
#[derive(Debug, Clone)]
pub struct ExecutionGroup {
    /// Rules in this group
    pub rules: Vec<Rule>,
    /// Recommended execution mode
    pub execution_mode: ExecutionMode,
    /// Salience level
    pub salience: i32,
    /// Whether this group can be safely parallelized
    pub can_parallelize: bool,
    /// Conflicts preventing parallelization
    pub conflicts: Vec<DependencyConflict>,
}

/// Execution mode recommendation
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionMode {
    /// Safe to run in parallel
    Parallel,
    /// Must run sequentially due to dependencies
    Sequential,
}

/// Strategy used for execution
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStrategy {
    /// All rules executed sequentially (due to dependencies)
    FullSequential,
    /// All rules executed in parallel (no dependencies)
    FullParallel,
    /// Mixed execution (some parallel, some sequential)
    Hybrid,
    /// Forced sequential due to configuration
    ForcedSequential,
}

impl DependencyAnalysisResult {
    /// Get a summary report
    pub fn get_summary(&self) -> String {
        format!(
            "📊 Dependency Analysis Summary:\n   Total rules: {}\n   Conflicts found: {}\n   Safe for parallel: {}\n   Execution groups: {}",
            self.total_rules,
            self.conflicts,
            if self.can_parallelize_safely { "✅ Yes" } else { "❌ No" },
            self.execution_groups.len()
        )
    }

    /// Get detailed report
    pub fn get_detailed_report(&self) -> String {
        let mut report = self.get_summary();
        report.push_str("\n\n🔍 Detailed Analysis:");

        for (i, group) in self.execution_groups.iter().enumerate() {
            report.push_str(&format!(
                "\n\n📋 Group {} (Salience {}):",
                i + 1,
                group.salience
            ));
            report.push_str(&format!(
                "\n   Mode: {:?} | Can parallelize: {}",
                group.execution_mode,
                if group.can_parallelize { "✅" } else { "❌" }
            ));
            report.push_str(&format!(
                "\n   Rules: {}",
                group
                    .rules
                    .iter()
                    .map(|r| r.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));

            if !group.conflicts.is_empty() {
                report.push_str("\n   🚨 Conflicts:");
                for conflict in &group.conflicts {
                    report.push_str(&format!(
                        "\n      - {}: {} (rules: {})",
                        match conflict.conflict_type {
                            ConflictType::WriteWrite => "Write-Write",
                            ConflictType::ReadWrite => "Read-Write",
                            ConflictType::Circular => "Circular",
                        },
                        conflict.field,
                        conflict.rules.join(", ")
                    ));
                }
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::{Condition, ConditionGroup};

    #[test]
    fn test_dependency_analyzer_creation() {
        let analyzer = DependencyAnalyzer::new();
        assert!(analyzer.readers.is_empty());
        assert!(analyzer.writers.is_empty());
        assert!(analyzer.dependencies.is_empty());
    }

    #[test]
    fn test_safe_rules_analysis() {
        let mut analyzer = DependencyAnalyzer::new();

        let rules = vec![
            Rule::new(
                "AgeValidation".to_string(),
                ConditionGroup::Single(Condition::new(
                    "User.Age".to_string(),
                    crate::types::Operator::GreaterThan,
                    crate::types::Value::Integer(18),
                )),
                vec![],
            ),
            Rule::new(
                "CountryCheck".to_string(),
                ConditionGroup::Single(Condition::new(
                    "User.Country".to_string(),
                    crate::types::Operator::Equal,
                    crate::types::Value::String("US".to_string()),
                )),
                vec![],
            ),
        ];

        let result = analyzer.analyze(&rules);
        assert_eq!(result.total_rules, 2);
        assert_eq!(result.conflicts, 0);
        assert!(result.can_parallelize_safely);
    }

    #[test]
    fn test_conflicting_rules_analysis() {
        let mut analyzer = DependencyAnalyzer::new();

        let rules = vec![
            Rule::new(
                "CalculateScore".to_string(),
                ConditionGroup::Single(Condition::new(
                    "User.Data".to_string(),
                    crate::types::Operator::Equal,
                    crate::types::Value::String("valid".to_string()),
                )),
                vec![crate::types::ActionType::Set {
                    field: "User.Score".to_string(),
                    value: crate::types::Value::Integer(85),
                }],
            ),
            Rule::new(
                "CheckVIPStatus".to_string(),
                ConditionGroup::Single(Condition::new(
                    "User.Score".to_string(),
                    crate::types::Operator::GreaterThan,
                    crate::types::Value::Integer(80),
                )),
                vec![crate::types::ActionType::Set {
                    field: "User.IsVIP".to_string(),
                    value: crate::types::Value::Boolean(true),
                }],
            ),
        ];

        let result = analyzer.analyze(&rules);
        assert_eq!(result.total_rules, 2);
        // Should detect conflicts between score calculation and VIP check
        assert!(!result.can_parallelize_safely);
    }
}

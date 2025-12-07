// Explanation Generation for Backward Chaining
//
// This module provides functionality for generating human-readable
// explanations of how backward chaining queries are proven.
//
// Version: 1.9.0

use super::goal::{Goal, GoalStatus};
use super::proof_tree::{ProofNode, ProofNodeType, ProofStats, ProofTree};
use super::unification::Bindings;
use std::collections::HashMap;

/// Explanation builder that tracks the proof process
#[derive(Debug, Clone)]
pub struct ExplanationBuilder {
    /// Stack of nodes being built (for hierarchy)
    node_stack: Vec<ProofNode>,

    /// Statistics tracking
    goals_explored: usize,
    rules_evaluated: usize,
    facts_checked: usize,
    max_depth: usize,

    /// Enable/disable tracking
    enabled: bool,
}

impl ExplanationBuilder {
    /// Create a new explanation builder
    pub fn new() -> Self {
        ExplanationBuilder {
            node_stack: Vec::new(),
            goals_explored: 0,
            rules_evaluated: 0,
            facts_checked: 0,
            max_depth: 0,
            enabled: false,
        }
    }

    /// Enable explanation tracking
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable explanation tracking
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if tracking is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Start tracking a goal
    pub fn start_goal(&mut self, goal: &Goal) {
        if !self.enabled {
            return;
        }

        self.goals_explored += 1;
        self.max_depth = self.max_depth.max(goal.depth);

        let node = ProofNode::new(goal.pattern.clone(), goal.depth);
        self.node_stack.push(node);
    }

    /// Mark goal as proven by a fact
    pub fn goal_proven_by_fact(&mut self, goal: &Goal, bindings: &Bindings) {
        if !self.enabled || self.node_stack.is_empty() {
            return;
        }

        self.facts_checked += 1;

        if let Some(node) = self.node_stack.last_mut() {
            node.proven = true;
            node.node_type = ProofNodeType::Fact;
            node.set_bindings(bindings);
        }
    }

    /// Mark goal as proven by a rule
    pub fn goal_proven_by_rule(&mut self, goal: &Goal, rule_name: &str, bindings: &Bindings) {
        if !self.enabled || self.node_stack.is_empty() {
            return;
        }

        self.rules_evaluated += 1;

        if let Some(node) = self.node_stack.last_mut() {
            node.proven = true;
            node.node_type = ProofNodeType::Rule;
            node.rule_name = Some(rule_name.to_string());
            node.set_bindings(bindings);
        }
    }

    /// Mark goal as negation
    pub fn goal_negation(&mut self, goal: &Goal, proven: bool) {
        if !self.enabled || self.node_stack.is_empty() {
            return;
        }

        if let Some(node) = self.node_stack.last_mut() {
            node.proven = proven;
            node.node_type = ProofNodeType::Negation;
        }
    }

    /// Mark goal as failed
    pub fn goal_failed(&mut self) {
        if !self.enabled || self.node_stack.is_empty() {
            return;
        }

        if let Some(node) = self.node_stack.last_mut() {
            node.proven = false;
            node.node_type = ProofNodeType::Failed;
        }
    }

    /// Finish tracking a goal and add to parent
    pub fn finish_goal(&mut self) {
        if !self.enabled || self.node_stack.is_empty() {
            return;
        }

        let finished_node = self.node_stack.pop().unwrap();

        // Add as child to parent if there is one
        if let Some(parent) = self.node_stack.last_mut() {
            parent.add_child(finished_node);
        } else {
            // Root node - push it back
            self.node_stack.push(finished_node);
        }
    }

    /// Build the final proof tree
    pub fn build(self, query: String) -> Option<ProofTree> {
        if !self.enabled || self.node_stack.is_empty() {
            return None;
        }

        let root = self.node_stack.into_iter().next().unwrap();
        let mut tree = ProofTree::new(root, query);

        tree.set_stats(ProofStats {
            goals_explored: self.goals_explored,
            rules_evaluated: self.rules_evaluated,
            facts_checked: self.facts_checked,
            max_depth: self.max_depth,
            total_nodes: tree.root.node_count(),
        });

        Some(tree)
    }

    /// Get current statistics
    pub fn stats(&self) -> ProofStats {
        ProofStats {
            goals_explored: self.goals_explored,
            rules_evaluated: self.rules_evaluated,
            facts_checked: self.facts_checked,
            max_depth: self.max_depth,
            total_nodes: self.node_stack.len(),
        }
    }

    /// Reset the builder
    pub fn reset(&mut self) {
        self.node_stack.clear();
        self.goals_explored = 0;
        self.rules_evaluated = 0;
        self.facts_checked = 0;
        self.max_depth = 0;
    }
}

impl Default for ExplanationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Step-by-step explanation of reasoning
#[derive(Debug, Clone)]
pub struct ExplanationStep {
    /// Step number
    pub step_number: usize,

    /// Goal being proven
    pub goal: String,

    /// Rule used (if any)
    pub rule_name: Option<String>,

    /// Condition evaluated
    pub condition: String,

    /// Variable bindings at this step
    pub bindings: HashMap<String, String>,

    /// Result of this step
    pub result: StepResult,

    /// Depth in reasoning tree
    pub depth: usize,
}

/// Result of an explanation step
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    /// Step succeeded
    Success,

    /// Step failed
    Failed,

    /// Step was skipped
    Skipped,
}

impl ExplanationStep {
    /// Create a new explanation step
    pub fn new(
        step_number: usize,
        goal: String,
        rule_name: Option<String>,
        condition: String,
        bindings: HashMap<String, String>,
        result: StepResult,
        depth: usize,
    ) -> Self {
        ExplanationStep {
            step_number,
            goal,
            rule_name,
            condition,
            bindings,
            result,
            depth,
        }
    }

    /// Format step as string
    pub fn format(&self) -> String {
        let mut s = String::new();

        s.push_str(&format!("Step {}: {}\n", self.step_number, self.goal));

        if let Some(rule) = &self.rule_name {
            s.push_str(&format!("  Rule: {}\n", rule));
        }

        s.push_str(&format!("  Condition: {}\n", self.condition));

        if !self.bindings.is_empty() {
            s.push_str(&format!("  Bindings: {:?}\n", self.bindings));
        }

        s.push_str(&format!("  Result: {:?}\n", self.result));

        s
    }
}

/// Complete explanation with steps
#[derive(Debug, Clone)]
pub struct Explanation {
    /// Original query
    pub query: String,

    /// Proof tree
    pub proof_tree: ProofTree,

    /// Step-by-step explanation
    pub steps: Vec<ExplanationStep>,

    /// Summary
    pub summary: String,
}

impl Explanation {
    /// Create a new explanation
    pub fn new(query: String, proof_tree: ProofTree) -> Self {
        let steps = Self::tree_to_steps(&proof_tree.root);
        let summary = Self::generate_summary(&proof_tree);

        Explanation {
            query,
            proof_tree,
            steps,
            summary,
        }
    }

    /// Convert proof tree to step-by-step explanation
    fn tree_to_steps(root: &ProofNode) -> Vec<ExplanationStep> {
        let mut steps = Vec::new();
        Self::collect_steps(root, &mut steps, 1);
        steps
    }

    /// Recursively collect steps from tree
    fn collect_steps(node: &ProofNode, steps: &mut Vec<ExplanationStep>, mut step_num: usize) -> usize {
        let result = if node.proven {
            StepResult::Success
        } else {
            StepResult::Failed
        };

        let condition = match &node.node_type {
            ProofNodeType::Fact => format!("{} [FACT]", node.goal),
            ProofNodeType::Rule => node.goal.clone(),
            ProofNodeType::Negation => format!("NOT {}", node.goal),
            ProofNodeType::Failed => format!("{} [FAILED]", node.goal),
        };

        let step = ExplanationStep::new(
            step_num,
            node.goal.clone(),
            node.rule_name.clone(),
            condition,
            node.bindings.clone(),
            result,
            node.depth,
        );

        steps.push(step);
        step_num += 1;

        // Process children
        for child in &node.children {
            step_num = Self::collect_steps(child, steps, step_num);
        }

        step_num
    }

    /// Generate summary text
    fn generate_summary(tree: &ProofTree) -> String {
        if tree.success {
            format!(
                "Query '{}' was successfully proven using {} rules and {} facts.",
                tree.query,
                tree.stats.rules_evaluated,
                tree.stats.facts_checked
            )
        } else {
            format!(
                "Query '{}' could not be proven. Explored {} goals, evaluated {} rules, and checked {} facts.",
                tree.query,
                tree.stats.goals_explored,
                tree.stats.rules_evaluated,
                tree.stats.facts_checked
            )
        }
    }

    /// Print full explanation
    pub fn print(&self) {
        println!("================================================================================");
        println!("EXPLANATION");
        println!("================================================================================");
        println!("\nQuery: {}", self.query);
        println!("Result: {}", if self.proof_tree.success { "✓ Proven" } else { "✗ Unprovable" });
        println!("\n{}\n", self.summary);

        println!("Step-by-Step Reasoning:");
        println!("{}", "-".repeat(80));
        for step in &self.steps {
            print!("{}", step.format());
        }

        println!("\n{}", "=".repeat(80));
        self.proof_tree.print_stats();
        println!("{}", "=".repeat(80));
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(serde::Serialize)]
        struct ExplanationJson<'a> {
            query: &'a str,
            success: bool,
            summary: &'a str,
            proof_tree: &'a ProofTree,
            stats: &'a ProofStats,
        }

        let json = ExplanationJson {
            query: &self.query,
            success: self.proof_tree.success,
            summary: &self.summary,
            proof_tree: &self.proof_tree,
            stats: &self.proof_tree.stats,
        };

        serde_json::to_string_pretty(&json)
    }

    /// Export to Markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Query Explanation\n\n");
        md.push_str(&format!("**Query:** `{}`\n\n", self.query));
        md.push_str(&format!(
            "**Result:** {}\n\n",
            if self.proof_tree.success { "✓ Proven" } else { "✗ Unprovable" }
        ));
        md.push_str(&format!("**Summary:** {}\n\n", self.summary));

        md.push_str("## Proof Tree\n\n");
        md.push_str(&self.proof_tree.to_markdown());

        md
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explanation_builder_creation() {
        let builder = ExplanationBuilder::new();
        assert!(!builder.is_enabled());
        assert_eq!(builder.goals_explored, 0);
    }

    #[test]
    fn test_enable_disable() {
        let mut builder = ExplanationBuilder::new();
        builder.enable();
        assert!(builder.is_enabled());

        builder.disable();
        assert!(!builder.is_enabled());
    }

    #[test]
    fn test_tracking_goal() {
        let mut builder = ExplanationBuilder::new();
        builder.enable();

        let goal = Goal::new("test".to_string());
        builder.start_goal(&goal);

        assert_eq!(builder.goals_explored, 1);
        assert_eq!(builder.node_stack.len(), 1);
    }

    #[test]
    fn test_goal_proven_by_fact() {
        let mut builder = ExplanationBuilder::new();
        builder.enable();

        let goal = Goal::new("test".to_string());
        builder.start_goal(&goal);

        let bindings = Bindings::new();
        builder.goal_proven_by_fact(&goal, &bindings);

        assert_eq!(builder.facts_checked, 1);
        assert!(builder.node_stack[0].proven);
        assert_eq!(builder.node_stack[0].node_type, ProofNodeType::Fact);
    }

    #[test]
    fn test_build_proof_tree() {
        let mut builder = ExplanationBuilder::new();
        builder.enable();

        let goal = Goal::new("test".to_string());
        builder.start_goal(&goal);

        let bindings = Bindings::new();
        builder.goal_proven_by_fact(&goal, &bindings);

        let tree = builder.build("test query".to_string());
        assert!(tree.is_some());

        let tree = tree.unwrap();
        assert_eq!(tree.query, "test query");
        assert!(tree.success);
    }

    #[test]
    fn test_explanation_step() {
        let step = ExplanationStep::new(
            1,
            "test_goal".to_string(),
            Some("test_rule".to_string()),
            "condition".to_string(),
            HashMap::new(),
            StepResult::Success,
            0,
        );

        assert_eq!(step.step_number, 1);
        assert_eq!(step.result, StepResult::Success);
    }
}

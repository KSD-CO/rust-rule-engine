// Proof Tree for Backward Chaining Explanations
//
// This module provides data structures for capturing and visualizing
// the reasoning process in backward chaining queries.
//
// Version: 1.9.0

use super::unification::Bindings;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a single node in the proof tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofNode {
    /// The goal that was proven at this node
    pub goal: String,

    /// Name of the rule that was used (if any)
    pub rule_name: Option<String>,

    /// Variable bindings at this node
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub bindings: HashMap<String, String>,

    /// Child nodes (sub-goals that were proven)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ProofNode>,

    /// Depth in the proof tree
    pub depth: usize,

    /// Whether this goal was proven successfully
    pub proven: bool,

    /// Type of proof node
    pub node_type: ProofNodeType,
}

/// Type of proof node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProofNodeType {
    /// Goal proven by a fact
    Fact,

    /// Goal proven by a rule
    Rule,

    /// Negated goal (NOT)
    Negation,

    /// Goal failed to prove
    Failed,
}

impl ProofNode {
    /// Create a new proof node
    pub fn new(goal: String, depth: usize) -> Self {
        ProofNode {
            goal,
            rule_name: None,
            bindings: HashMap::new(),
            children: Vec::new(),
            depth,
            proven: false,
            node_type: ProofNodeType::Failed,
        }
    }

    /// Create a fact node
    pub fn fact(goal: String, depth: usize) -> Self {
        ProofNode {
            goal,
            rule_name: None,
            bindings: HashMap::new(),
            children: Vec::new(),
            depth,
            proven: true,
            node_type: ProofNodeType::Fact,
        }
    }

    /// Create a rule node
    pub fn rule(goal: String, rule_name: String, depth: usize) -> Self {
        ProofNode {
            goal,
            rule_name: Some(rule_name),
            bindings: HashMap::new(),
            children: Vec::new(),
            depth,
            proven: true,
            node_type: ProofNodeType::Rule,
        }
    }

    /// Create a negation node
    pub fn negation(goal: String, depth: usize, proven: bool) -> Self {
        ProofNode {
            goal,
            rule_name: None,
            bindings: HashMap::new(),
            children: Vec::new(),
            depth,
            proven,
            node_type: ProofNodeType::Negation,
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: ProofNode) {
        self.children.push(child);
    }

    /// Set bindings from Bindings object
    pub fn set_bindings(&mut self, bindings: &Bindings) {
        // Convert Bindings to HashMap using to_map() method
        let binding_map = bindings.to_map();
        self.bindings = binding_map
            .iter()
            .map(|(k, v)| (k.clone(), format!("{:?}", v)))
            .collect();
    }

    /// Set bindings from HashMap
    pub fn set_bindings_map(&mut self, bindings: HashMap<String, String>) {
        self.bindings = bindings;
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Print the proof tree
    pub fn print_tree(&self, indent: usize) {
        let prefix = "  ".repeat(indent);
        let status = if self.proven { "✓" } else { "✗" };

        println!("{}{} {}", prefix, status, self.goal);

        if let Some(rule) = &self.rule_name {
            println!("{}  [Rule: {}]", prefix, rule);
        }

        match self.node_type {
            ProofNodeType::Fact => println!("{}  [FACT]", prefix),
            ProofNodeType::Negation => println!("{}  [NEGATION]", prefix),
            _ => {}
        }

        if !self.bindings.is_empty() {
            println!("{}  Bindings: {:?}", prefix, self.bindings);
        }

        for child in &self.children {
            child.print_tree(indent + 1);
        }
    }

    /// Get tree height
    pub fn height(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.height()).max().unwrap_or(0)
        }
    }

    /// Count total nodes
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }
}

/// Complete proof tree with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofTree {
    /// Root node of the proof tree
    pub root: ProofNode,

    /// Whether the query was proven
    pub success: bool,

    /// Original query string
    pub query: String,

    /// Statistics
    pub stats: ProofStats,
}

/// Statistics about the proof
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProofStats {
    /// Total number of goals explored
    pub goals_explored: usize,

    /// Total number of rules evaluated
    pub rules_evaluated: usize,

    /// Total number of facts checked
    pub facts_checked: usize,

    /// Maximum depth reached
    pub max_depth: usize,

    /// Total nodes in proof tree
    pub total_nodes: usize,
}

impl ProofTree {
    /// Create a new proof tree
    pub fn new(root: ProofNode, query: String) -> Self {
        let success = root.proven;
        let total_nodes = root.node_count();
        let max_depth = root.height();

        ProofTree {
            root,
            success,
            query,
            stats: ProofStats {
                goals_explored: 0,
                rules_evaluated: 0,
                facts_checked: 0,
                max_depth,
                total_nodes,
            },
        }
    }

    /// Set statistics
    pub fn set_stats(&mut self, stats: ProofStats) {
        self.stats = stats;
    }

    /// Print the entire tree
    pub fn print(&self) {
        println!("Query: {}", self.query);
        println!("Result: {}", if self.success { "✓ Proven" } else { "✗ Unprovable" });
        println!("\nProof Tree:");
        println!("{}", "=".repeat(80));
        self.root.print_tree(0);
        println!("{}", "=".repeat(80));
        self.print_stats();
    }

    /// Print statistics
    pub fn print_stats(&self) {
        println!("\nStatistics:");
        println!("  Goals explored: {}", self.stats.goals_explored);
        println!("  Rules evaluated: {}", self.stats.rules_evaluated);
        println!("  Facts checked: {}", self.stats.facts_checked);
        println!("  Max depth: {}", self.stats.max_depth);
        println!("  Total nodes: {}", self.stats.total_nodes);
    }

    /// Convert to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert to Markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Proof Explanation\n\n");
        md.push_str(&format!("**Query:** `{}`\n\n", self.query));
        md.push_str(&format!("**Result:** {}\n\n",
            if self.success { "✓ Proven" } else { "✗ Unprovable" }
        ));

        md.push_str("## Proof Tree\n\n");
        self.node_to_markdown(&self.root, &mut md, 0);

        md.push_str("\n## Statistics\n\n");
        md.push_str(&format!("- **Goals explored:** {}\n", self.stats.goals_explored));
        md.push_str(&format!("- **Rules evaluated:** {}\n", self.stats.rules_evaluated));
        md.push_str(&format!("- **Facts checked:** {}\n", self.stats.facts_checked));
        md.push_str(&format!("- **Max depth:** {}\n", self.stats.max_depth));
        md.push_str(&format!("- **Total nodes:** {}\n", self.stats.total_nodes));

        md
    }

    /// Convert node to markdown recursively
    fn node_to_markdown(&self, node: &ProofNode, md: &mut String, depth: usize) {
        let prefix = "  ".repeat(depth);
        let status = if node.proven { "✓" } else { "✗" };

        md.push_str(&format!("{}* {} `{}`", prefix, status, node.goal));

        if let Some(rule) = &node.rule_name {
            md.push_str(&format!(" **[Rule: {}]**", rule));
        }

        match node.node_type {
            ProofNodeType::Fact => md.push_str(" *[FACT]*"),
            ProofNodeType::Negation => md.push_str(" *[NEGATION]*"),
            _ => {}
        }

        md.push('\n');

        if !node.bindings.is_empty() {
            md.push_str(&format!("{}  * Bindings: `{:?}`\n", prefix, node.bindings));
        }

        for child in &node.children {
            self.node_to_markdown(child, md, depth + 1);
        }
    }

    /// Convert to HTML
    pub fn to_html(&self) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("  <title>Proof Explanation</title>\n");
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: 'Courier New', monospace; margin: 20px; }\n");
        html.push_str("    .proven { color: green; }\n");
        html.push_str("    .failed { color: red; }\n");
        html.push_str("    .node { margin-left: 20px; }\n");
        html.push_str("    .rule { color: blue; font-style: italic; }\n");
        html.push_str("    .bindings { color: gray; font-size: 0.9em; }\n");
        html.push_str("    .stats { margin-top: 20px; padding: 10px; background: #f0f0f0; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n<body>\n");

        html.push_str(&format!("<h1>Proof Explanation</h1>\n"));
        html.push_str(&format!("<p><strong>Query:</strong> <code>{}</code></p>\n", self.query));
        html.push_str(&format!("<p><strong>Result:</strong> <span class=\"{}\">{}</span></p>\n",
            if self.success { "proven" } else { "failed" },
            if self.success { "✓ Proven" } else { "✗ Unprovable" }
        ));

        html.push_str("<h2>Proof Tree</h2>\n");
        self.node_to_html(&self.root, &mut html);

        html.push_str("<div class=\"stats\">\n");
        html.push_str("<h2>Statistics</h2>\n");
        html.push_str(&format!("<p>Goals explored: {}</p>\n", self.stats.goals_explored));
        html.push_str(&format!("<p>Rules evaluated: {}</p>\n", self.stats.rules_evaluated));
        html.push_str(&format!("<p>Facts checked: {}</p>\n", self.stats.facts_checked));
        html.push_str(&format!("<p>Max depth: {}</p>\n", self.stats.max_depth));
        html.push_str(&format!("<p>Total nodes: {}</p>\n", self.stats.total_nodes));
        html.push_str("</div>\n");

        html.push_str("</body>\n</html>");
        html
    }

    /// Convert node to HTML recursively
    fn node_to_html(&self, node: &ProofNode, html: &mut String) {
        let status = if node.proven { "✓" } else { "✗" };
        let class = if node.proven { "proven" } else { "failed" };

        html.push_str("<div class=\"node\">\n");
        html.push_str(&format!("  <span class=\"{}\">{} {}</span>",
            class, status, node.goal));

        if let Some(rule) = &node.rule_name {
            html.push_str(&format!(" <span class=\"rule\">[Rule: {}]</span>", rule));
        }

        match node.node_type {
            ProofNodeType::Fact => html.push_str(" <em>[FACT]</em>"),
            ProofNodeType::Negation => html.push_str(" <em>[NEGATION]</em>"),
            _ => {}
        }

        if !node.bindings.is_empty() {
            html.push_str(&format!("<br><span class=\"bindings\">Bindings: {:?}</span>",
                node.bindings));
        }

        html.push_str("\n");

        for child in &node.children {
            self.node_to_html(child, html);
        }

        html.push_str("</div>\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_node_creation() {
        let node = ProofNode::new("test_goal".to_string(), 0);
        assert_eq!(node.goal, "test_goal");
        assert_eq!(node.depth, 0);
        assert!(!node.proven);
        assert_eq!(node.node_type, ProofNodeType::Failed);
    }

    #[test]
    fn test_fact_node() {
        let node = ProofNode::fact("fact_goal".to_string(), 1);
        assert!(node.proven);
        assert_eq!(node.node_type, ProofNodeType::Fact);
        assert!(node.is_leaf());
    }

    #[test]
    fn test_rule_node() {
        let node = ProofNode::rule("rule_goal".to_string(), "test_rule".to_string(), 2);
        assert!(node.proven);
        assert_eq!(node.node_type, ProofNodeType::Rule);
        assert_eq!(node.rule_name, Some("test_rule".to_string()));
    }

    #[test]
    fn test_add_child() {
        let mut parent = ProofNode::rule("parent".to_string(), "rule1".to_string(), 0);
        let child = ProofNode::fact("child".to_string(), 1);

        parent.add_child(child);
        assert_eq!(parent.children.len(), 1);
        assert!(!parent.is_leaf());
    }

    #[test]
    fn test_tree_height() {
        let mut root = ProofNode::rule("root".to_string(), "rule1".to_string(), 0);
        let mut child1 = ProofNode::rule("child1".to_string(), "rule2".to_string(), 1);
        let child2 = ProofNode::fact("child2".to_string(), 2);

        child1.add_child(child2);
        root.add_child(child1);

        assert_eq!(root.height(), 3);
    }

    #[test]
    fn test_node_count() {
        let mut root = ProofNode::rule("root".to_string(), "rule1".to_string(), 0);
        let child1 = ProofNode::fact("child1".to_string(), 1);
        let child2 = ProofNode::fact("child2".to_string(), 1);

        root.add_child(child1);
        root.add_child(child2);

        assert_eq!(root.node_count(), 3);
    }

    #[test]
    fn test_proof_tree_creation() {
        let root = ProofNode::fact("test".to_string(), 0);
        let tree = ProofTree::new(root, "test query".to_string());

        assert!(tree.success);
        assert_eq!(tree.query, "test query");
        assert_eq!(tree.stats.total_nodes, 1);
    }

    #[test]
    fn test_json_serialization() {
        let root = ProofNode::fact("test".to_string(), 0);
        let tree = ProofTree::new(root, "test query".to_string());

        let json = tree.to_json().unwrap();
        assert!(json.contains("test query"));
        assert!(json.contains("Fact"));
    }

    #[test]
    fn test_markdown_generation() {
        let root = ProofNode::fact("test".to_string(), 0);
        let tree = ProofTree::new(root, "test query".to_string());

        let md = tree.to_markdown();
        assert!(md.contains("# Proof Explanation"));
        assert!(md.contains("test query"));
        assert!(md.contains("✓"));
    }

    #[test]
    fn test_html_generation() {
        let root = ProofNode::fact("test".to_string(), 0);
        let tree = ProofTree::new(root, "test query".to_string());

        let html = tree.to_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("test query"));
        assert!(html.contains("✓"));
    }
}

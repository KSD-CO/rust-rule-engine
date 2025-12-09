//! Disjunction (OR) support for backward chaining queries
//!
//! This module implements OR patterns in queries, allowing multiple alternative
//! conditions to be specified. The query succeeds if ANY of the alternatives succeed.
//!
//! # Examples
//!
//! ```rust,ignore
//! // Find people who are either managers OR seniors
//! let results = engine.query(
//!     "eligible(?person) WHERE (manager(?person) OR senior(?person))",
//!     &mut facts
//! )?;
//!
//! // Complex OR with multiple conditions
//! let results = engine.query(
//!     "discount(?customer) WHERE (vip(?customer) OR total_spent(?customer, ?amt) > 10000)",
//!     &mut facts
//! )?;
//! ```

use super::goal::Goal;
use super::unification::Bindings;
use std::collections::HashSet;

/// Represents a disjunction (OR) of goals
#[derive(Debug, Clone)]
pub struct Disjunction {
    /// Alternative goals - at least one must succeed
    pub branches: Vec<Goal>,

    /// Original pattern string
    pub pattern: String,
}

impl Disjunction {
    /// Create a new disjunction from a list of goals
    pub fn new(branches: Vec<Goal>, pattern: String) -> Self {
        assert!(!branches.is_empty(), "Disjunction must have at least one branch");
        Self { branches, pattern }
    }

    /// Create a disjunction from two goals
    pub fn from_pair(left: Goal, right: Goal) -> Self {
        let pattern = format!("({} OR {})", left.pattern, right.pattern);
        Self {
            branches: vec![left, right],
            pattern,
        }
    }

    /// Add another branch to this disjunction
    pub fn add_branch(&mut self, goal: Goal) {
        self.branches.push(goal);
    }

    /// Get the number of branches
    pub fn branch_count(&self) -> usize {
        self.branches.len()
    }
}

/// Result of evaluating a disjunction
#[derive(Debug, Clone)]
pub struct DisjunctionResult {
    /// All solutions from all branches
    pub solutions: Vec<Bindings>,

    /// Which branches succeeded (by index)
    pub successful_branches: Vec<usize>,

    /// Whether the disjunction as a whole succeeded
    pub success: bool,
}

impl DisjunctionResult {
    /// Create a new result
    pub fn new() -> Self {
        Self {
            solutions: Vec::new(),
            successful_branches: Vec::new(),
            success: false,
        }
    }

    /// Create a successful result
    pub fn success(solutions: Vec<Bindings>, successful_branches: Vec<usize>) -> Self {
        Self {
            solutions,
            successful_branches,
            success: true,
        }
    }

    /// Create a failed result
    pub fn failure() -> Self {
        Self {
            solutions: Vec::new(),
            successful_branches: Vec::new(),
            success: false,
        }
    }

    /// Add solutions from a branch
    pub fn add_branch_solutions(&mut self, branch_index: usize, solutions: Vec<Bindings>) {
        if !solutions.is_empty() {
            self.successful_branches.push(branch_index);
            self.solutions.extend(solutions);
            self.success = true;
        }
    }

    /// Deduplicate solutions based on variable bindings
    pub fn deduplicate(&mut self) {
        // Use a set to track unique binding combinations
        let mut seen = HashSet::new();
        let mut unique_solutions = Vec::new();

        for solution in &self.solutions {
            let binding_map = solution.to_map();
            let key = format!("{:?}", binding_map);

            if seen.insert(key) {
                unique_solutions.push(solution.clone());
            }
        }

        self.solutions = unique_solutions;
    }

    /// Get the total number of solutions
    pub fn solution_count(&self) -> usize {
        self.solutions.len()
    }
}

impl Default for DisjunctionResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser for OR patterns in queries
pub struct DisjunctionParser;

impl DisjunctionParser {
    /// Parse a pattern that might contain OR
    ///
    /// Examples:
    /// - "(A OR B)" -> Disjunction with 2 branches
    /// - "(A OR B OR C)" -> Disjunction with 3 branches
    /// - "A" -> None (no OR, single goal)
    pub fn parse(pattern: &str) -> Option<Disjunction> {
        let pattern = pattern.trim();

        // Check if pattern starts with '(' and ends with ')'
        if !pattern.starts_with('(') || !pattern.ends_with(')') {
            return None;
        }

        // Remove outer parentheses
        let inner = &pattern[1..pattern.len()-1];

        // Split by OR (naive implementation - TODO: handle nested parens)
        if !inner.contains(" OR ") {
            return None;
        }

        let branches: Vec<Goal> = inner
            .split(" OR ")
            .map(|s| Goal::new(s.trim().to_string()))
            .collect();

        if branches.len() < 2 {
            return None;
        }

        Some(Disjunction::new(branches, pattern.to_string()))
    }

    /// Check if a pattern contains OR
    pub fn contains_or(pattern: &str) -> bool {
        pattern.contains(" OR ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disjunction_creation() {
        let goal1 = Goal::new("manager(?person)".to_string());
        let goal2 = Goal::new("senior(?person)".to_string());

        let disj = Disjunction::from_pair(goal1, goal2);

        assert_eq!(disj.branch_count(), 2);
        assert!(disj.pattern.contains("OR"));
    }

    #[test]
    fn test_disjunction_add_branch() {
        let goal1 = Goal::new("manager(?person)".to_string());
        let goal2 = Goal::new("senior(?person)".to_string());
        let goal3 = Goal::new("director(?person)".to_string());

        let mut disj = Disjunction::from_pair(goal1, goal2);
        disj.add_branch(goal3);

        assert_eq!(disj.branch_count(), 3);
    }

    #[test]
    fn test_disjunction_result_success() {
        let mut result = DisjunctionResult::new();

        let bindings1 = Bindings::new();
        let bindings2 = Bindings::new();

        result.add_branch_solutions(0, vec![bindings1]);
        result.add_branch_solutions(1, vec![bindings2]);

        assert!(result.success);
        assert_eq!(result.solution_count(), 2);
        assert_eq!(result.successful_branches.len(), 2);
    }

    #[test]
    fn test_disjunction_result_empty() {
        let mut result = DisjunctionResult::new();

        result.add_branch_solutions(0, vec![]);
        result.add_branch_solutions(1, vec![]);

        assert!(!result.success);
        assert_eq!(result.solution_count(), 0);
    }

    #[test]
    fn test_parser_simple_or() {
        let pattern = "(manager(?person) OR senior(?person))";
        let disj = DisjunctionParser::parse(pattern);

        assert!(disj.is_some());
        let disj = disj.unwrap();
        assert_eq!(disj.branch_count(), 2);
    }

    #[test]
    fn test_parser_triple_or() {
        let pattern = "(A OR B OR C)";
        let disj = DisjunctionParser::parse(pattern);

        assert!(disj.is_some());
        let disj = disj.unwrap();
        assert_eq!(disj.branch_count(), 3);
    }

    #[test]
    fn test_parser_no_or() {
        let pattern = "manager(?person)";
        let disj = DisjunctionParser::parse(pattern);

        assert!(disj.is_none());
    }

    #[test]
    fn test_parser_contains_or() {
        assert!(DisjunctionParser::contains_or("(A OR B)"));
        assert!(!DisjunctionParser::contains_or("A AND B"));
    }

    #[test]
    fn test_deduplication() {
        let mut result = DisjunctionResult::new();

        // Add duplicate solutions
        let bindings = Bindings::new();
        result.add_branch_solutions(0, vec![bindings.clone(), bindings.clone()]);

        assert_eq!(result.solution_count(), 2);

        result.deduplicate();

        assert_eq!(result.solution_count(), 1);
    }
}

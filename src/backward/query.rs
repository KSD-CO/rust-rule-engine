//! Query interface for backward chaining

use super::goal::Goal;
use super::search::Solution;
use crate::types::Value;
use std::collections::HashMap;

/// Result of a query operation
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Whether the query goal is provable
    pub provable: bool,

    /// Variable bindings that satisfy the query
    pub bindings: HashMap<String, Value>,

    /// Trace of how the goal was proven
    pub proof_trace: ProofTrace,

    /// Facts that are missing to prove the goal
    pub missing_facts: Vec<String>,

    /// Execution statistics
    pub stats: QueryStats,

    /// All solutions found (when max_solutions > 1)
    pub solutions: Vec<Solution>,
}

/// Trace showing how a goal was proven
#[derive(Debug, Clone)]
pub struct ProofTrace {
    /// Root goal that was proven
    pub goal: String,
    
    /// Steps taken to prove the goal
    pub steps: Vec<ProofStep>,
}

/// Single step in a proof
#[derive(Debug, Clone)]
pub struct ProofStep {
    /// Rule that was applied
    pub rule_name: String,
    
    /// Goal this step proved
    pub goal: String,
    
    /// Sub-steps (for nested proofs)
    pub sub_steps: Vec<ProofStep>,
    
    /// Depth in the proof tree
    pub depth: usize,
}

/// Statistics about query execution
#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    /// Number of goals explored
    pub goals_explored: usize,
    
    /// Number of rules evaluated
    pub rules_evaluated: usize,
    
    /// Maximum depth reached
    pub max_depth: usize,
    
    /// Time taken (if measured)
    pub duration_ms: Option<u64>,
}

impl QueryResult {
    /// Create a successful query result
    pub fn success(bindings: HashMap<String, Value>, proof: ProofTrace, stats: QueryStats) -> Self {
        Self {
            provable: true,
            bindings,
            proof_trace: proof,
            missing_facts: Vec::new(),
            stats,
            solutions: Vec::new(),
        }
    }

    /// Create a successful query result with multiple solutions
    pub fn success_with_solutions(
        bindings: HashMap<String, Value>,
        proof: ProofTrace,
        stats: QueryStats,
        solutions: Vec<Solution>,
    ) -> Self {
        Self {
            provable: true,
            bindings,
            proof_trace: proof,
            missing_facts: Vec::new(),
            stats,
            solutions,
        }
    }

    /// Create a failed query result
    pub fn failure(missing: Vec<String>, stats: QueryStats) -> Self {
        Self {
            provable: false,
            bindings: HashMap::new(),
            proof_trace: ProofTrace::empty(),
            missing_facts: missing,
            stats,
            solutions: Vec::new(),
        }
    }
}

impl ProofTrace {
    /// Create an empty proof trace
    pub fn empty() -> Self {
        Self {
            goal: String::new(),
            steps: Vec::new(),
        }
    }
    
    /// Create a new proof trace
    pub fn new(goal: String) -> Self {
        Self {
            goal,
            steps: Vec::new(),
        }
    }
    
    /// Add a step to the proof
    pub fn add_step(&mut self, step: ProofStep) {
        self.steps.push(step);
    }
    
    /// Build trace from a goal tree
    pub fn from_goal(goal: &Goal) -> Self {
        let mut trace = Self::new(goal.pattern.clone());
        
        for (i, rule_name) in goal.candidate_rules.iter().enumerate() {
            let step = ProofStep {
                rule_name: rule_name.clone(),
                goal: goal.pattern.clone(),
                sub_steps: goal.sub_goals.iter()
                    .map(|sg| ProofStep::from_goal(sg, i + 1))
                    .collect(),
                depth: goal.depth,
            };
            trace.add_step(step);
        }
        
        trace
    }
    
    /// Print the proof trace in a readable format
    pub fn print(&self) {
        println!("Proof for goal: {}", self.goal);
        for step in &self.steps {
            step.print(0);
        }
    }
}

impl ProofStep {
    /// Create from a goal
    fn from_goal(goal: &Goal, depth: usize) -> Self {
        Self {
            rule_name: goal.candidate_rules.first()
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            goal: goal.pattern.clone(),
            sub_steps: goal.sub_goals.iter()
                .map(|sg| Self::from_goal(sg, depth + 1))
                .collect(),
            depth,
        }
    }
    
    /// Print this step with indentation
    fn print(&self, indent: usize) {
        let prefix = "  ".repeat(indent);
        println!("{}→ [{}] {}", prefix, self.rule_name, self.goal);
        for sub in &self.sub_steps {
            sub.print(indent + 1);
        }
    }
}

/// Query parser for converting strings to goals
pub struct QueryParser;

impl QueryParser {
    /// Parse a query string into a Goal
    ///
    /// Examples:
    /// - "User.IsVIP == true"
    /// - "Order.Total > 1000"
    /// - "User.IsVIP == true && Order.Amount > 1000"
    pub fn parse(query: &str) -> Result<Goal, String> {
        use super::expression::ExpressionParser;

        // Simple parsing for now
        if query.is_empty() {
            return Err("Empty query".to_string());
        }

        // Parse expression using ExpressionParser
        match ExpressionParser::parse(query) {
            Ok(expr) => {
                Ok(Goal::with_expression(query.to_string(), expr))
            }
            Err(e) => {
                Err(format!("Failed to parse query: {}", e))
            }
        }
    }

    /// Validate query syntax
    pub fn validate(query: &str) -> Result<(), String> {
        if query.is_empty() {
            return Err("Query cannot be empty".to_string());
        }

        // Try to parse to validate
        Self::parse(query).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_query_result_creation() {
        let stats = QueryStats::default();
        
        let success = QueryResult::success(
            HashMap::new(),
            ProofTrace::empty(),
            stats.clone(),
        );
        assert!(success.provable);
        
        let failure = QueryResult::failure(vec!["fact".to_string()], stats);
        assert!(!failure.provable);
        assert_eq!(failure.missing_facts.len(), 1);
    }
    
    #[test]
    fn test_proof_trace() {
        let mut trace = ProofTrace::new("User.IsVIP == true".to_string());
        assert_eq!(trace.goal, "User.IsVIP == true");
        assert!(trace.steps.is_empty());
        
        let step = ProofStep {
            rule_name: "VIPRule".to_string(),
            goal: "User.IsVIP == true".to_string(),
            sub_steps: Vec::new(),
            depth: 0,
        };
        
        trace.add_step(step);
        assert_eq!(trace.steps.len(), 1);
    }
    
    #[test]
    fn test_proof_step() {
        let step = ProofStep {
            rule_name: "TestRule".to_string(),
            goal: "test".to_string(),
            sub_steps: Vec::new(),
            depth: 0,
        };
        
        assert_eq!(step.rule_name, "TestRule");
        assert_eq!(step.depth, 0);
    }
    
    #[test]
    fn test_query_parser() {
        let result = QueryParser::parse("User.IsVIP == true");
        assert!(result.is_ok());
        
        let empty = QueryParser::parse("");
        assert!(empty.is_err());
    }
    
    #[test]
    fn test_query_validation() {
        assert!(QueryParser::validate("User.Age > 18").is_ok());
        assert!(QueryParser::validate("User.IsVIP == true").is_ok());
        assert!(QueryParser::validate("").is_err());
        // Note: "invalid" is now valid as a field name
        // Use empty string or malformed syntax for error cases
        assert!(QueryParser::validate("(unclosed").is_err());
    }
    
    #[test]
    fn test_query_stats() {
        let stats = QueryStats {
            goals_explored: 5,
            rules_evaluated: 3,
            max_depth: 2,
            duration_ms: Some(100),
        };

        assert_eq!(stats.goals_explored, 5);
        assert_eq!(stats.duration_ms, Some(100));
    }

    #[test]
    fn test_query_stats_default() {
        let stats = QueryStats::default();
        assert_eq!(stats.goals_explored, 0);
        assert_eq!(stats.rules_evaluated, 0);
        assert_eq!(stats.max_depth, 0);
        assert_eq!(stats.duration_ms, None);
    }

    #[test]
    fn test_query_result_with_bindings() {
        let mut bindings = HashMap::new();
        bindings.insert("X".to_string(), Value::String("VIP".to_string()));
        bindings.insert("Y".to_string(), Value::Number(1000.0));

        let stats = QueryStats::default();
        let result = QueryResult::success(bindings, ProofTrace::empty(), stats);

        assert!(result.provable);
        assert_eq!(result.bindings.len(), 2);
        assert_eq!(result.bindings.get("X"), Some(&Value::String("VIP".to_string())));
        assert_eq!(result.bindings.get("Y"), Some(&Value::Number(1000.0)));
    }

    #[test]
    fn test_query_result_failure_with_missing_facts() {
        let missing = vec![
            "User.IsVIP".to_string(),
            "Order.Total".to_string(),
        ];

        let stats = QueryStats::default();
        let result = QueryResult::failure(missing, stats);

        assert!(!result.provable);
        assert_eq!(result.missing_facts.len(), 2);
        assert!(result.bindings.is_empty());
    }

    #[test]
    fn test_proof_trace_from_goal() {
        let mut goal = Goal::new("User.IsVIP == true".to_string());
        goal.depth = 1;
        goal.add_candidate_rule("VIPRule".to_string());

        let mut subgoal = Goal::new("User.Points > 1000".to_string());
        subgoal.depth = 2;
        subgoal.add_candidate_rule("PointsRule".to_string());

        goal.add_subgoal(subgoal);

        let trace = ProofTrace::from_goal(&goal);

        assert_eq!(trace.goal, "User.IsVIP == true");
        assert_eq!(trace.steps.len(), 1);
        assert_eq!(trace.steps[0].rule_name, "VIPRule");
        assert_eq!(trace.steps[0].sub_steps.len(), 1);
    }

    #[test]
    fn test_proof_step_nested() {
        let sub_step = ProofStep {
            rule_name: "SubRule".to_string(),
            goal: "subgoal".to_string(),
            sub_steps: Vec::new(),
            depth: 2,
        };

        let step = ProofStep {
            rule_name: "MainRule".to_string(),
            goal: "main".to_string(),
            sub_steps: vec![sub_step],
            depth: 1,
        };

        assert_eq!(step.sub_steps.len(), 1);
        assert_eq!(step.sub_steps[0].rule_name, "SubRule");
        assert_eq!(step.sub_steps[0].depth, 2);
    }

    #[test]
    fn test_query_parser_complex_expressions() {
        // Test AND expression
        let and_result = QueryParser::parse("User.IsVIP == true && Order.Total > 1000");
        assert!(and_result.is_ok());

        // Test OR expression
        let or_result = QueryParser::parse("User.Points > 500 || User.IsVIP == true");
        assert!(or_result.is_ok());

        // Test NOT expression
        let not_result = QueryParser::parse("!(User.IsBanned == true)");
        assert!(not_result.is_ok());
    }

    #[test]
    fn test_query_parser_invalid_syntax() {
        // Empty query
        assert!(QueryParser::parse("").is_err());

        // Unclosed parenthesis
        assert!(QueryParser::parse("(User.IsVIP == true").is_err());

        // Invalid operator sequence
        assert!(QueryParser::parse("User.IsVIP == == true").is_err());
    }

    #[test]
    fn test_proof_trace_empty() {
        let trace = ProofTrace::empty();
        assert!(trace.goal.is_empty());
        assert!(trace.steps.is_empty());
    }
}

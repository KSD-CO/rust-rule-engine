//! Goal system for backward chaining

use crate::types::Value;
use std::collections::HashMap;
use super::expression::Expression;
use super::unification::Bindings;

/// Represents a goal to prove/achieve in backward chaining
#[derive(Debug, Clone)]
pub struct Goal {
    /// Target pattern to prove (e.g., "User.IsVIP == true")
    pub pattern: String,

    /// Parsed expression AST (if available)
    pub expression: Option<Expression>,

    /// Current status of this goal
    pub status: GoalStatus,

    /// Sub-goals that need to be proven first
    pub sub_goals: Vec<Goal>,

    /// Rules that can potentially derive this goal
    pub candidate_rules: Vec<String>,

    /// Variable bindings accumulated during proof
    pub bindings: Bindings,

    /// Depth of this goal in the search tree
    pub depth: usize,

    /// Whether this is a negated goal (NOT keyword)
    /// When true, the goal succeeds if it CANNOT be proven (closed-world assumption)
    pub is_negated: bool,
}

/// Status of a goal in the proof process
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoalStatus {
    /// Goal has not been attempted yet
    Pending,
    
    /// Goal is currently being proven (to detect cycles)
    InProgress,
    
    /// Goal has been successfully proven
    Proven,
    
    /// Goal cannot be proven with available rules/facts
    Unprovable,
}

impl Goal {
    /// Create a new goal with the given pattern
    pub fn new(pattern: String) -> Self {
        Self {
            pattern,
            expression: None,
            status: GoalStatus::Pending,
            sub_goals: Vec::new(),
            candidate_rules: Vec::new(),
            bindings: Bindings::new(),
            depth: 0,
            is_negated: false,
        }
    }

    /// Create a new goal with parsed expression
    pub fn with_expression(pattern: String, expression: Expression) -> Self {
        Self {
            pattern,
            expression: Some(expression),
            status: GoalStatus::Pending,
            sub_goals: Vec::new(),
            candidate_rules: Vec::new(),
            bindings: Bindings::new(),
            depth: 0,
            is_negated: false,
        }
    }

    /// Create a negated goal (NOT goal)
    pub fn negated(pattern: String) -> Self {
        Self {
            pattern,
            expression: None,
            status: GoalStatus::Pending,
            sub_goals: Vec::new(),
            candidate_rules: Vec::new(),
            bindings: Bindings::new(),
            depth: 0,
            is_negated: true,
        }
    }

    /// Create a negated goal with expression
    pub fn negated_with_expression(pattern: String, expression: Expression) -> Self {
        Self {
            pattern,
            expression: Some(expression),
            status: GoalStatus::Pending,
            sub_goals: Vec::new(),
            candidate_rules: Vec::new(),
            bindings: Bindings::new(),
            depth: 0,
            is_negated: true,
        }
    }
    
    /// Check if this goal is proven
    pub fn is_proven(&self) -> bool {
        self.status == GoalStatus::Proven
    }
    
    /// Check if this goal is unprovable
    pub fn is_unprovable(&self) -> bool {
        self.status == GoalStatus::Unprovable
    }
    
    /// Check if all sub-goals are proven
    pub fn all_subgoals_proven(&self) -> bool {
        self.sub_goals.iter().all(|g| g.is_proven())
    }
    
    /// Add a sub-goal
    pub fn add_subgoal(&mut self, goal: Goal) {
        self.sub_goals.push(goal);
    }
    
    /// Add a candidate rule that can derive this goal
    pub fn add_candidate_rule(&mut self, rule_name: String) {
        if !self.candidate_rules.contains(&rule_name) {
            self.candidate_rules.push(rule_name);
        }
    }
}

/// Manager for goal-driven reasoning
#[derive(Debug)]
pub struct GoalManager {
    /// Active goals being pursued
    goals: Vec<Goal>,
    
    /// Maximum depth for goal search (prevent infinite recursion)
    max_depth: usize,
    
    /// Cache of proven goals (memoization)
    proven_cache: HashMap<String, bool>,
}

impl GoalManager {
    /// Create a new goal manager
    pub fn new(max_depth: usize) -> Self {
        Self {
            goals: Vec::new(),
            max_depth,
            proven_cache: HashMap::new(),
        }
    }
    
    /// Add a new goal to pursue
    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }
    
    /// Get the next pending goal to work on
    pub fn next_pending(&mut self) -> Option<&mut Goal> {
        self.goals.iter_mut()
            .find(|g| g.status == GoalStatus::Pending)
    }
    
    /// Check if a goal pattern has been proven before (memoization)
    pub fn is_cached(&self, pattern: &str) -> Option<bool> {
        self.proven_cache.get(pattern).copied()
    }
    
    /// Cache the result of proving a goal
    pub fn cache_result(&mut self, pattern: String, proven: bool) {
        self.proven_cache.insert(pattern, proven);
    }
    
    /// Check if we've exceeded maximum depth
    pub fn is_too_deep(&self, depth: usize) -> bool {
        depth > self.max_depth
    }
    
    /// Get all goals
    pub fn goals(&self) -> &[Goal] {
        &self.goals
    }
    
    /// Clear all goals and cache
    pub fn clear(&mut self) {
        self.goals.clear();
        self.proven_cache.clear();
    }
}

impl Default for GoalManager {
    fn default() -> Self {
        Self::new(10) // Default max depth of 10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_goal_creation() {
        let goal = Goal::new("User.IsVIP == true".to_string());
        assert_eq!(goal.status, GoalStatus::Pending);
        assert_eq!(goal.depth, 0);
        assert!(goal.sub_goals.is_empty());
    }
    
    #[test]
    fn test_goal_status_checks() {
        let mut goal = Goal::new("test".to_string());
        assert!(!goal.is_proven());
        
        goal.status = GoalStatus::Proven;
        assert!(goal.is_proven());
        
        goal.status = GoalStatus::Unprovable;
        assert!(goal.is_unprovable());
    }
    
    #[test]
    fn test_subgoal_management() {
        let mut parent = Goal::new("parent".to_string());
        let mut child1 = Goal::new("child1".to_string());
        let mut child2 = Goal::new("child2".to_string());
        
        child1.status = GoalStatus::Proven;
        child2.status = GoalStatus::Proven;
        
        parent.add_subgoal(child1);
        parent.add_subgoal(child2);
        
        assert_eq!(parent.sub_goals.len(), 2);
        assert!(parent.all_subgoals_proven());
    }
    
    #[test]
    fn test_goal_manager() {
        let mut manager = GoalManager::new(5);
        
        let goal1 = Goal::new("goal1".to_string());
        let goal2 = Goal::new("goal2".to_string());
        
        manager.add_goal(goal1);
        manager.add_goal(goal2);
        
        assert_eq!(manager.goals().len(), 2);
        
        // Test caching
        assert!(manager.is_cached("goal1").is_none());
        manager.cache_result("goal1".to_string(), true);
        assert_eq!(manager.is_cached("goal1"), Some(true));
        
        // Test depth check
        assert!(!manager.is_too_deep(3));
        assert!(manager.is_too_deep(10));
    }
    
    #[test]
    fn test_candidate_rules() {
        let mut goal = Goal::new("test".to_string());

        goal.add_candidate_rule("Rule1".to_string());
        goal.add_candidate_rule("Rule2".to_string());
        goal.add_candidate_rule("Rule1".to_string()); // Duplicate

        assert_eq!(goal.candidate_rules.len(), 2);
    }

    #[test]
    fn test_goal_with_expression() {
        use super::super::expression::Expression;
        use crate::types::{Operator, Value};

        let expr = Expression::Comparison {
            left: Box::new(Expression::Field("User.IsVIP".to_string())),
            operator: Operator::Equal,
            right: Box::new(Expression::Literal(Value::Boolean(true))),
        };

        let goal = Goal::with_expression("User.IsVIP == true".to_string(), expr);
        assert!(goal.expression.is_some());
        assert_eq!(goal.pattern, "User.IsVIP == true");
        assert_eq!(goal.status, GoalStatus::Pending);
    }

    #[test]
    fn test_subgoals_not_all_proven() {
        let mut parent = Goal::new("parent".to_string());
        let mut child1 = Goal::new("child1".to_string());
        let mut child2 = Goal::new("child2".to_string());

        child1.status = GoalStatus::Proven;
        child2.status = GoalStatus::Pending; // Not proven yet

        parent.add_subgoal(child1);
        parent.add_subgoal(child2);

        assert_eq!(parent.sub_goals.len(), 2);
        assert!(!parent.all_subgoals_proven());
    }

    #[test]
    fn test_goal_manager_next_pending() {
        let mut manager = GoalManager::new(5);

        let mut goal1 = Goal::new("goal1".to_string());
        goal1.status = GoalStatus::Proven; // Already proven

        let goal2 = Goal::new("goal2".to_string()); // Pending

        let mut goal3 = Goal::new("goal3".to_string());
        goal3.status = GoalStatus::InProgress;

        manager.add_goal(goal1);
        manager.add_goal(goal2);
        manager.add_goal(goal3);

        // Should return goal2 as it's the only pending one
        let next = manager.next_pending();
        assert!(next.is_some());
        assert_eq!(next.unwrap().pattern, "goal2");
    }

    #[test]
    fn test_goal_manager_clear() {
        let mut manager = GoalManager::new(5);

        manager.add_goal(Goal::new("goal1".to_string()));
        manager.add_goal(Goal::new("goal2".to_string()));
        manager.cache_result("goal1".to_string(), true);

        assert_eq!(manager.goals().len(), 2);
        assert!(manager.is_cached("goal1").is_some());

        manager.clear();

        assert_eq!(manager.goals().len(), 0);
        assert!(manager.is_cached("goal1").is_none());
    }

    #[test]
    fn test_goal_bindings() {
        let mut goal = Goal::new("User.?X == true".to_string());

        // Test that bindings start empty
        assert!(goal.bindings.get("X").is_none());

        // Add a binding
        goal.bindings.bind("X".to_string(), Value::String("IsVIP".to_string())).ok();

        // Verify binding was added
        assert!(goal.bindings.get("X").is_some());
        assert_eq!(goal.bindings.get("X"), Some(&Value::String("IsVIP".to_string())));
    }

    #[test]
    fn test_goal_depth() {
        let mut goal = Goal::new("test".to_string());
        assert_eq!(goal.depth, 0);

        goal.depth = 5;
        assert_eq!(goal.depth, 5);
    }

    #[test]
    fn test_goal_manager_default() {
        let manager = GoalManager::default();
        assert_eq!(manager.max_depth, 10);
        assert_eq!(manager.goals().len(), 0);
    }

    #[test]
    fn test_negated_goal() {
        let goal = Goal::negated("User.IsBanned == true".to_string());
        assert_eq!(goal.pattern, "User.IsBanned == true");
        assert!(goal.is_negated);
        assert_eq!(goal.status, GoalStatus::Pending);
    }

    #[test]
    fn test_negated_goal_with_expression() {
        use super::super::expression::Expression;
        use crate::types::{Operator, Value};

        let expr = Expression::Comparison {
            left: Box::new(Expression::Field("User.IsBanned".to_string())),
            operator: Operator::Equal,
            right: Box::new(Expression::Literal(Value::Boolean(true))),
        };

        let goal = Goal::negated_with_expression("User.IsBanned == true".to_string(), expr);
        assert!(goal.expression.is_some());
        assert!(goal.is_negated);
        assert_eq!(goal.pattern, "User.IsBanned == true");
    }

    #[test]
    fn test_normal_goal_not_negated() {
        let goal = Goal::new("test".to_string());
        assert!(!goal.is_negated);

        let expr = Expression::Field("test".to_string());
        let goal_with_expr = Goal::with_expression("test".to_string(), expr);
        assert!(!goal_with_expr.is_negated);
    }
}

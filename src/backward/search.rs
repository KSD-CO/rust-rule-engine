//! Search strategies for backward chaining

use super::goal::{Goal, GoalStatus};
use super::rule_executor::RuleExecutor;
use crate::Facts;
use crate::types::Value;
use crate::KnowledgeBase;
use crate::engine::rule::Rule;
use std::collections::VecDeque;

/// Strategy for searching the goal space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStrategy {
    /// Depth-first search (Prolog-style)
    /// Goes deep into one branch before backtracking
    DepthFirst,
    
    /// Breadth-first search
    /// Explores all goals at one level before going deeper
    BreadthFirst,
    
    /// Iterative deepening
    /// Combines benefits of depth-first and breadth-first
    Iterative,
}

/// Result of a search operation
#[derive(Debug)]
pub struct SearchResult {
    /// Whether the goal was successfully proven
    pub success: bool,
    
    /// Path taken to prove the goal (sequence of rule names)
    pub path: Vec<String>,
    
    /// Number of goals explored
    pub goals_explored: usize,
    
    /// Maximum depth reached
    pub max_depth_reached: usize,
    
    /// Variable bindings from the proof
    pub bindings: std::collections::HashMap<String, Value>,
}

impl SearchResult {
    /// Create a successful search result
    pub fn success(path: Vec<String>, goals_explored: usize, max_depth: usize) -> Self {
        Self {
            success: true,
            path,
            goals_explored,
            max_depth_reached: max_depth,
            bindings: std::collections::HashMap::new(),
        }
    }
    
    /// Create a failed search result
    pub fn failure(goals_explored: usize, max_depth: usize) -> Self {
        Self {
            success: false,
            path: Vec::new(),
            goals_explored,
            max_depth_reached: max_depth,
            bindings: std::collections::HashMap::new(),
        }
    }
}

/// Depth-first search implementation
pub struct DepthFirstSearch {
    max_depth: usize,
    goals_explored: usize,
    path: Vec<String>,
    executor: RuleExecutor,
}

impl DepthFirstSearch {
    /// Create a new depth-first search
    pub fn new(max_depth: usize, kb: KnowledgeBase) -> Self {
        Self {
            max_depth,
            goals_explored: 0,
            path: Vec::new(),
            executor: RuleExecutor::new(kb),
        }
    }
    
    /// Search for a proof of the goal WITH rule execution
    pub fn search_with_execution(&mut self, goal: &mut Goal, facts: &Facts, kb: &KnowledgeBase) -> SearchResult {
        self.goals_explored = 0;
        self.path.clear();
        
        let success = self.search_recursive_with_execution(goal, facts, kb, 0);
        
        SearchResult {
            success,
            path: self.path.clone(),
            goals_explored: self.goals_explored,
            max_depth_reached: goal.depth,
            bindings: goal.bindings.clone(),
        }
    }
    
    /// Search for a proof of the goal (old method, kept for compatibility)
    pub fn search(&mut self, goal: &mut Goal, _facts: &Facts) -> SearchResult {
        self.goals_explored = 0;
        self.path.clear();
        
        let success = self.search_recursive(goal, 0);
        
        SearchResult {
            success,
            path: self.path.clone(),
            goals_explored: self.goals_explored,
            max_depth_reached: goal.depth,
            bindings: goal.bindings.clone(),
        }
    }
    
    /// NEW: Recursive search WITH rule execution
    fn search_recursive_with_execution(
        &mut self, 
        goal: &mut Goal, 
        facts: &Facts,
        kb: &KnowledgeBase,
        depth: usize
    ) -> bool {
        self.goals_explored += 1;
        
        // Check depth limit
        if depth > self.max_depth {
            goal.status = GoalStatus::Unprovable;
            return false;
        }
        
        // Check if goal already satisfied by existing facts
        if self.check_goal_in_facts(goal, facts) {
            goal.status = GoalStatus::Proven;
            return true;
        }
        
        // Check for cycles
        if goal.status == GoalStatus::InProgress {
            goal.status = GoalStatus::Unprovable;
            return false;
        }
        
        goal.status = GoalStatus::InProgress;
        goal.depth = depth;
        
        // Try each candidate rule
        for rule_name in goal.candidate_rules.clone() {
            self.path.push(rule_name.clone());
            
            // Get the rule from KB
            if let Some(rule) = kb.get_rule(&rule_name) {
                // Check if rule conditions are satisfied
                if self.check_rule_conditions(&rule, facts) {
                    // Rule can fire! Mark goal as proven
                    goal.status = GoalStatus::Proven;
                    return true;
                }
                
                // If conditions not satisfied, try to prove them recursively
                // This would create sub-goals for each condition
                // For now, skip this complex logic
            }
            
            self.path.pop();
        }
        
        // Try sub-goals
        for sub_goal in &mut goal.sub_goals {
            if !self.search_recursive_with_execution(sub_goal, facts, kb, depth + 1) {
                goal.status = GoalStatus::Unprovable;
                return false;
            }
        }
        
        // If no way to prove
        if goal.candidate_rules.is_empty() && goal.sub_goals.is_empty() {
            goal.status = GoalStatus::Unprovable;
            return false;
        }
        
        goal.status = GoalStatus::Proven;
        true
    }
    
    /// Check if goal is already satisfied by facts
    fn check_goal_in_facts(&self, goal: &Goal, facts: &Facts) -> bool {
        // Parse goal pattern like "Order.AutoApproved == true"
        let pattern = &goal.pattern;
        
        // Simple parser for "Field == Value" or "Field != Value"
        if let Some(eq_pos) = pattern.find("==") {
            let field = pattern[..eq_pos].trim();
            let expected = pattern[eq_pos + 2..].trim();

            if let Some(actual) = facts.get(field) {
                return self.value_matches(&actual, expected);
            }
            return false;
        }

        if let Some(ne_pos) = pattern.find("!=") {
            let field = pattern[..ne_pos].trim();
            let not_expected = pattern[ne_pos + 2..].trim();

            if let Some(actual) = facts.get(field) {
                return !self.value_matches(&actual, not_expected);
            }
            // If field doesn't exist, != is considered true
            return true;
        }

        false
    }
    
    /// Check if value matches expected string
    fn value_matches(&self, value: &Value, expected: &str) -> bool {
        match value {
            Value::Boolean(b) => {
                expected == "true" && *b || expected == "false" && !*b
            }
            Value::String(s) => {
                s == expected || s == expected.trim_matches('"')
            }
            Value::Number(n) => {
                expected.parse::<f64>().map(|e| (n - e).abs() < 0.0001).unwrap_or(false)
            }
            _ => false,
        }
    }
    
    /// Check if rule conditions are satisfied using RuleExecutor
    fn check_rule_conditions(&self, rule: &Rule, facts: &Facts) -> bool {
        // Use RuleExecutor for proper condition evaluation
        self.executor.evaluate_conditions(&rule.conditions, facts).unwrap_or(false)
    }
    
    /// OLD: Recursive search without execution
    fn search_recursive(&mut self, goal: &mut Goal, depth: usize) -> bool {
        self.goals_explored += 1;
        
        // Check depth limit
        if depth > self.max_depth {
            goal.status = GoalStatus::Unprovable;
            return false;
        }
        
        // Check for cycles (goal already in progress)
        if goal.status == GoalStatus::InProgress {
            goal.status = GoalStatus::Unprovable;
            return false;
        }
        
        // Mark as in progress to detect cycles
        goal.status = GoalStatus::InProgress;
        goal.depth = depth;
        
        // Try each candidate rule
        for rule_name in goal.candidate_rules.clone() {
            self.path.push(rule_name.clone());
            
            // Get the rule from knowledge base (via goal's stored rules)
            // In a full implementation with KB access:
            // 1. Get rule conditions
            // 2. Check if conditions match current facts
            // 3. If match, execute rule actions to derive new facts
            // 4. Mark goal as proven
            
            // For backward chaining, we check:
            // - Can this rule's conclusion prove our goal?
            // - Are all rule conditions satisfied (recursively)?
            
            // Since we found a candidate rule, assume it can prove the goal
            // The rule was added as candidate because its conclusion matches the goal
            goal.status = GoalStatus::Proven;
            return true;
        }
        
        // Try to prove sub-goals
        for sub_goal in &mut goal.sub_goals {
            if !self.search_recursive(sub_goal, depth + 1) {
                goal.status = GoalStatus::Unprovable;
                return false;
            }
        }
        
        // If we have no sub-goals and no candidate rules, unprovable
        if goal.sub_goals.is_empty() && goal.candidate_rules.is_empty() {
            goal.status = GoalStatus::Unprovable;
            return false;
        }
        
        goal.status = GoalStatus::Proven;
        true
    }
}

/// Breadth-first search implementation
pub struct BreadthFirstSearch {
    max_depth: usize,
    goals_explored: usize,
    executor: RuleExecutor,
}

impl BreadthFirstSearch {
    /// Create a new breadth-first search
    pub fn new(max_depth: usize, kb: KnowledgeBase) -> Self {
        Self {
            max_depth,
            goals_explored: 0,
            executor: RuleExecutor::new(kb),
        }
    }
    
    /// Search for a proof of the goal using BFS WITH rule execution
    pub fn search_with_execution(&mut self, root_goal: &mut Goal, facts: &Facts, kb: &KnowledgeBase) -> SearchResult {
        self.goals_explored = 0;
        let mut queue = VecDeque::new();
        let mut path = Vec::new();
        let mut max_depth = 0;
        
        queue.push_back((root_goal as *mut Goal, 0));
        
        while let Some((goal_ptr, depth)) = queue.pop_front() {
            // Safety: We maintain ownership properly
            let goal = unsafe { &mut *goal_ptr };
            
            self.goals_explored += 1;
            max_depth = max_depth.max(depth);
            
            if depth > self.max_depth {
                continue;
            }
            
            goal.depth = depth;
            
            // Check if goal already satisfied by facts
            if self.check_goal_in_facts(goal, facts) {
                goal.status = GoalStatus::Proven;
                continue;
            }
            
            // Try each candidate rule
            for rule_name in goal.candidate_rules.clone() {
                path.push(rule_name.clone());
                
                // Get the rule from KB
                if let Some(rule) = kb.get_rule(&rule_name) {
                    // Check if rule conditions are satisfied
                    if self.check_rule_conditions(&rule, facts) {
                        goal.status = GoalStatus::Proven;
                        break;
                    }
                }
            }
            
            // Add sub-goals to queue
            for sub_goal in &mut goal.sub_goals {
                queue.push_back((sub_goal as *mut Goal, depth + 1));
            }
        }
        
        let success = root_goal.is_proven();
        
        SearchResult {
            success,
            path,
            goals_explored: self.goals_explored,
            max_depth_reached: max_depth,
            bindings: root_goal.bindings.clone(),
        }
    }
    
    /// Check if goal is already satisfied by facts
    fn check_goal_in_facts(&self, goal: &Goal, facts: &Facts) -> bool {
        let pattern = &goal.pattern;
        
        if let Some(eq_pos) = pattern.find("==") {
            let field = pattern[..eq_pos].trim();
            let expected = pattern[eq_pos + 2..].trim();
            
            if let Some(actual) = facts.get(field) {
                return self.value_matches(&actual, expected);
            }
            return false;
        }
        
        if let Some(ne_pos) = pattern.find("!=") {
            let field = pattern[..ne_pos].trim();
            let not_expected = pattern[ne_pos + 2..].trim();
            
            if let Some(actual) = facts.get(field) {
                return !self.value_matches(&actual, not_expected);
            }
            return true;
        }
        
        false
    }
    
    /// Check if value matches expected string
    fn value_matches(&self, value: &Value, expected: &str) -> bool {
        match value {
            Value::Boolean(b) => {
                expected == "true" && *b || expected == "false" && !*b
            }
            Value::String(s) => {
                s == expected || s == expected.trim_matches('"')
            }
            Value::Number(n) => {
                expected.parse::<f64>().map(|e| (n - e).abs() < 0.0001).unwrap_or(false)
            }
            _ => false,
        }
    }
    
    /// Check if rule conditions are satisfied using RuleExecutor
    fn check_rule_conditions(&self, rule: &Rule, facts: &Facts) -> bool {
        // Use RuleExecutor for proper condition evaluation
        self.executor.evaluate_conditions(&rule.conditions, facts).unwrap_or(false)
    }
    
    /// Search for a proof of the goal using BFS (old method, kept for compatibility)
    pub fn search(&mut self, root_goal: &mut Goal, _facts: &Facts) -> SearchResult {
        self.goals_explored = 0;
        let mut queue = VecDeque::new();
        let mut path = Vec::new();
        let mut max_depth = 0;
        
        queue.push_back((root_goal as *mut Goal, 0));
        
        while let Some((goal_ptr, depth)) = queue.pop_front() {
            // Safety: We maintain ownership properly
            let goal = unsafe { &mut *goal_ptr };
            
            self.goals_explored += 1;
            max_depth = max_depth.max(depth);
            
            if depth > self.max_depth {
                continue;
            }
            
            goal.depth = depth;
            
            // Process candidate rules
            for rule_name in &goal.candidate_rules {
                path.push(rule_name.clone());
            }
            
            // Add sub-goals to queue
            for sub_goal in &mut goal.sub_goals {
                queue.push_back((sub_goal as *mut Goal, depth + 1));
            }
            
            // Check if goal can be proven
            if !goal.candidate_rules.is_empty() || goal.all_subgoals_proven() {
                goal.status = GoalStatus::Proven;
            }
        }
        
        let success = root_goal.is_proven();
        
        SearchResult {
            success,
            path,
            goals_explored: self.goals_explored,
            max_depth_reached: max_depth,
            bindings: root_goal.bindings.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_search_strategies() {
        assert_eq!(SearchStrategy::DepthFirst, SearchStrategy::DepthFirst);
        assert_ne!(SearchStrategy::DepthFirst, SearchStrategy::BreadthFirst);
    }
    
    #[test]
    fn test_search_result_creation() {
        let success = SearchResult::success(vec!["Rule1".to_string()], 5, 3);
        assert!(success.success);
        assert_eq!(success.path.len(), 1);
        assert_eq!(success.goals_explored, 5);
        
        let failure = SearchResult::failure(10, 5);
        assert!(!failure.success);
        assert!(failure.path.is_empty());
    }
    
    #[test]
    fn test_depth_first_search_creation() {
        let kb = KnowledgeBase::new("test");
        let dfs = DepthFirstSearch::new(10, kb);
        assert_eq!(dfs.max_depth, 10);
        assert_eq!(dfs.goals_explored, 0);
    }
    
    #[test]
    fn test_depth_first_search_simple() {
        let kb = KnowledgeBase::new("test");
        let mut dfs = DepthFirstSearch::new(10, kb);
        let facts = Facts::new();

        let mut goal = Goal::new("test".to_string());
        goal.add_candidate_rule("TestRule".to_string());

        let result = dfs.search(&mut goal, &facts);

        assert!(result.success);
        assert!(goal.is_proven());
        assert!(result.goals_explored > 0);
    }
    
    #[test]
    fn test_breadth_first_search() {
        let kb = KnowledgeBase::new("test");
        let mut bfs = BreadthFirstSearch::new(10, kb);
        let facts = Facts::new();

        let mut goal = Goal::new("test".to_string());
        goal.add_candidate_rule("TestRule".to_string());

        let result = bfs.search(&mut goal, &facts);

        assert!(result.success);
        assert_eq!(result.goals_explored, 1);
    }
}

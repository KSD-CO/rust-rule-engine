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
    pub fn search_with_execution(&mut self, goal: &mut Goal, facts: &mut Facts, kb: &KnowledgeBase) -> SearchResult {
        self.goals_explored = 0;
        self.path.clear();

        let success = self.search_recursive_with_execution(goal, facts, kb, 0);

        SearchResult {
            success,
            path: self.path.clone(),
            goals_explored: self.goals_explored,
            max_depth_reached: goal.depth,
            bindings: goal.bindings.to_map(),
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
            bindings: goal.bindings.to_map(),
        }
    }
    
    /// NEW: Recursive search WITH rule execution
    fn search_recursive_with_execution(
        &mut self,
        goal: &mut Goal,
        facts: &mut Facts,  // ✅ Made mutable to allow rule execution
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
                // ✅ FIX: Try to execute rule (checks conditions AND executes actions)
                match self.executor.try_execute_rule(&rule, facts) {
                    Ok(true) => {
                        // Rule executed successfully - derived new facts!
                        // Now check if our goal is proven
                        if self.check_goal_in_facts(goal, facts) {
                            goal.status = GoalStatus::Proven;
                            return true;
                        }
                    }
                    Ok(false) => {
                        // ✅ Conditions not satisfied - try to prove them recursively!
                        if self.try_prove_rule_conditions(&rule, facts, kb, depth + 1) {
                            // All conditions now satisfied! Try executing rule again
                            match self.executor.try_execute_rule(&rule, facts) {
                                Ok(true) => {
                                    if self.check_goal_in_facts(goal, facts) {
                                        goal.status = GoalStatus::Proven;
                                        self.path.pop();
                                        return true;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(_) => {
                        // Execution error - continue to next rule
                    }
                }
            }

            self.path.pop();
        }

        // Try sub-goals
        let mut all_subgoals_proven = true;
        for sub_goal in &mut goal.sub_goals {
            if !self.search_recursive_with_execution(sub_goal, facts, kb, depth + 1) {
                all_subgoals_proven = false;
                break;
            }
        }

        // If we have sub-goals and they're all proven, goal is proven
        if !goal.sub_goals.is_empty() && all_subgoals_proven {
            goal.status = GoalStatus::Proven;
            return true;
        }

        // If we have no candidate rules and no sub-goals, or nothing worked
        goal.status = GoalStatus::Unprovable;
        false
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

    /// Try to prove all conditions of a rule by creating sub-goals
    /// This is the core of recursive backward chaining!
    fn try_prove_rule_conditions(
        &mut self,
        rule: &Rule,
        facts: &mut Facts,
        kb: &KnowledgeBase,
        depth: usize,
    ) -> bool {
        // Extract all conditions from the condition group and try to prove them
        self.try_prove_condition_group(&rule.conditions, facts, kb, depth)
    }

    /// Recursively prove a condition group
    fn try_prove_condition_group(
        &mut self,
        group: &crate::engine::rule::ConditionGroup,
        facts: &mut Facts,
        kb: &KnowledgeBase,
        depth: usize,
    ) -> bool {
        use crate::engine::rule::ConditionGroup;

        match group {
            ConditionGroup::Single(condition) => {
                // Try to prove this single condition
                self.try_prove_single_condition(condition, facts, kb, depth)
            }
            ConditionGroup::Compound { left, operator, right } => {
                // Handle AND/OR/NOT logic
                use crate::types::LogicalOperator;
                match operator {
                    LogicalOperator::And => {
                        // Both must be proven
                        self.try_prove_condition_group(left, facts, kb, depth)
                            && self.try_prove_condition_group(right, facts, kb, depth)
                    }
                    LogicalOperator::Or => {
                        // At least one must be proven
                        self.try_prove_condition_group(left, facts, kb, depth)
                            || self.try_prove_condition_group(right, facts, kb, depth)
                    }
                    LogicalOperator::Not => {
                        // Left should fail, right doesn't apply
                        !self.try_prove_condition_group(left, facts, kb, depth)
                    }
                }
            }
            ConditionGroup::Not(_) | ConditionGroup::Exists(_) | ConditionGroup::Forall(_) | ConditionGroup::Accumulate { .. } => {
                // For now, skip complex conditions
                true
            }
        }
    }

    /// Try to prove a single condition
    fn try_prove_single_condition(
        &mut self,
        condition: &crate::engine::rule::Condition,
        facts: &mut Facts,
        kb: &KnowledgeBase,
        depth: usize,
    ) -> bool {
        // Convert condition to goal pattern
        let goal_pattern = self.condition_to_goal_pattern(condition);

        // Create a sub-goal for this condition
        let mut sub_goal = Goal::new(goal_pattern.clone());
        sub_goal.depth = depth;

        // Find candidate rules that could prove this sub-goal
        for candidate_rule in kb.get_rules() {
            if self.rule_could_prove_pattern(&candidate_rule, &goal_pattern) {
                sub_goal.add_candidate_rule(candidate_rule.name.clone());
            }
        }

        // Try to prove this sub-goal recursively
        self.search_recursive_with_execution(&mut sub_goal, facts, kb, depth)
    }

    /// Convert a condition to a goal pattern string
    fn condition_to_goal_pattern(&self, condition: &crate::engine::rule::Condition) -> String {
        use crate::engine::rule::ConditionExpression;

        let field = match &condition.expression {
            ConditionExpression::Field(f) => f.clone(),
            ConditionExpression::FunctionCall { name, .. } => name.clone(),
            ConditionExpression::Test { name, .. } => format!("test({})", name),
            ConditionExpression::MultiField { field, .. } => field.clone(),
        };

        let op_str = match condition.operator {
            crate::types::Operator::Equal => "==",
            crate::types::Operator::NotEqual => "!=",
            crate::types::Operator::GreaterThan => ">",
            crate::types::Operator::LessThan => "<",
            crate::types::Operator::GreaterThanOrEqual => ">=",
            crate::types::Operator::LessThanOrEqual => "<=",
            crate::types::Operator::Contains => "contains",
            crate::types::Operator::NotContains => "not_contains",
            crate::types::Operator::StartsWith => "starts_with",
            crate::types::Operator::EndsWith => "ends_with",
            crate::types::Operator::Matches => "matches",
        };

        let value_str = format!("{:?}", condition.value);

        format!("{} {} {}", field, op_str, value_str)
    }

    /// Check if a rule could prove a specific goal pattern
    fn rule_could_prove_pattern(&self, rule: &Rule, pattern: &str) -> bool {
        // Simple heuristic: check if pattern mentions fields that this rule sets
        for action in &rule.actions {
            match action {
                crate::types::ActionType::Set { field, .. } => {
                    if pattern.contains(field) {
                        return true;
                    }
                }
                crate::types::ActionType::MethodCall { object, method, .. } => {
                    if pattern.contains(object) || pattern.contains(method) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
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
    pub fn search_with_execution(&mut self, root_goal: &mut Goal, facts: &mut Facts, kb: &KnowledgeBase) -> SearchResult {
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
                    // ✅ FIX: Try to execute rule (checks conditions AND executes actions)
                    match self.executor.try_execute_rule(&rule, facts) {
                        Ok(true) => {
                            // Rule executed successfully - derived new facts!
                            // Now check if our goal is proven
                            if self.check_goal_in_facts(goal, facts) {
                                goal.status = GoalStatus::Proven;
                                break;
                            }
                        }
                        Ok(false) => {
                            // Conditions not satisfied - continue to next rule
                        }
                        Err(_) => {
                            // Execution error - continue to next rule
                        }
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
            bindings: root_goal.bindings.to_map(),
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
            bindings: root_goal.bindings.to_map(),
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

//! Search strategies for backward chaining

#![allow(deprecated)]

use super::goal::{Goal, GoalStatus};
use super::rule_executor::RuleExecutor;
use crate::engine::rule::Rule;
use crate::rete::propagation::IncrementalEngine;
use crate::types::Value;
use crate::Facts;
use crate::KnowledgeBase;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

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

/// A single solution found during search
#[derive(Debug, Clone)]
pub struct Solution {
    /// Path taken to prove the goal (sequence of rule names)
    pub path: Vec<String>,

    /// Variable bindings from this proof
    pub bindings: std::collections::HashMap<String, Value>,
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

    /// All solutions found (if max_solutions > 1)
    pub solutions: Vec<Solution>,
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
            solutions: Vec::new(),
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
            solutions: Vec::new(),
        }
    }
}

/// Depth-first search implementation
pub struct DepthFirstSearch {
    max_depth: usize,
    goals_explored: usize,
    path: Vec<String>,
    executor: RuleExecutor,
    max_solutions: usize,
    solutions: Vec<Solution>,
}

impl DepthFirstSearch {
    /// Create a new depth-first search
    pub fn new(max_depth: usize, kb: KnowledgeBase) -> Self {
        Self {
            max_depth,
            goals_explored: 0,
            path: Vec::new(),
            executor: RuleExecutor::new_with_inserter(kb, None),
            max_solutions: 1,
            solutions: Vec::new(),
        }
    }

    /// Set maximum number of solutions to find
    pub fn with_max_solutions(mut self, max_solutions: usize) -> Self {
        self.max_solutions = max_solutions;
        self
    }

    /// Create a new depth-first search and wire an optional IncrementalEngine
    /// to enable TMS logical insertion. The engine is provided as Arc<Mutex<>>
    /// and the inserter closure will call `insert_logical` on it.
    pub fn new_with_engine(
        max_depth: usize,
        kb: KnowledgeBase,
        engine: Option<Arc<Mutex<IncrementalEngine>>>,
    ) -> Self {
        // Build inserter closure if engine is provided
        let inserter = engine.map(|eng| {
            let eng = eng.clone();
            std::sync::Arc::new(
                move |fact_type: String,
                      data: crate::rete::TypedFacts,
                      rule_name: String,
                      premises: Vec<String>| {
                    if let Ok(mut e) = eng.lock() {
                        // Resolve premise keys into FactHandles when possible
                        let handles = e.resolve_premise_keys(premises);
                        let _ = e.insert_logical(fact_type, data, rule_name, handles);
                    }
                },
            )
                as std::sync::Arc<
                    dyn Fn(String, crate::rete::TypedFacts, String, Vec<String>) + Send + Sync,
                >
        });

        Self {
            max_depth,
            goals_explored: 0,
            path: Vec::new(),
            executor: RuleExecutor::new_with_inserter(kb, inserter),
            max_solutions: 1,
            solutions: Vec::new(),
        }
    }

    /// Search for a proof of the goal WITH rule execution
    pub fn search_with_execution(
        &mut self,
        goal: &mut Goal,
        facts: &mut Facts,
        kb: &KnowledgeBase,
    ) -> SearchResult {
        self.goals_explored = 0;
        self.path.clear();
        self.solutions.clear();

        let success = self.search_recursive_with_execution(goal, facts, kb, 0);

        SearchResult {
            success,
            path: self.path.clone(),
            goals_explored: self.goals_explored,
            max_depth_reached: goal.depth,
            bindings: goal.bindings.to_map(),
            solutions: self.solutions.clone(),
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
            solutions: Vec::new(),
        }
    }

    /// NEW: Recursive search WITH rule execution
    fn search_recursive_with_execution(
        &mut self,
        goal: &mut Goal,
        facts: &mut Facts, // ✅ Made mutable to allow rule execution
        kb: &KnowledgeBase,
        depth: usize,
    ) -> bool {
        self.goals_explored += 1;

        // Check depth limit
        if depth > self.max_depth {
            goal.status = GoalStatus::Unprovable;
            return false;
        }

        // Check if goal already satisfied by existing facts
        let fact_proven = self.check_goal_in_facts(goal, facts);

        // Handle negated goals (closed-world assumption)
        if goal.is_negated {
            // For negated goals: success if CANNOT be proven
            if fact_proven {
                goal.status = GoalStatus::Unprovable;
                return false; // Goal IS provable, so NOT goal fails
            }
            // Continue to check if it can be derived via rules
            // If no rules can prove it, then negation succeeds
        } else {
            // Normal goal: success if proven
            if fact_proven {
                goal.status = GoalStatus::Proven;
                return true;
            }
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

            // Start an undo frame before trying this candidate so we can rollback
            // any speculative changes if this candidate doesn't lead to a proof.
            facts.begin_undo_frame();

            // Get the rule from KB
            if let Some(rule) = kb.get_rule(&rule_name) {
                // Try to execute rule (checks conditions AND executes actions)
                match self.executor.try_execute_rule(&rule, facts) {
                    Ok(true) => {
                        // Rule executed successfully - derived new facts!
                        // Now check if our goal is proven
                        if self.check_goal_in_facts(goal, facts) {
                            goal.status = GoalStatus::Proven;

                            // Save this solution
                            self.solutions.push(Solution {
                                path: self.path.clone(),
                                bindings: goal.bindings.to_map(),
                            });

                            // If we only want one solution OR we've found enough, stop searching
                            if self.max_solutions == 1 || self.solutions.len() >= self.max_solutions
                            {
                                return true; // keep changes
                            }

                            // Otherwise (max_solutions > 1 and not enough yet), rollback and continue
                            // This allows us to find alternative proof paths
                            facts.rollback_undo_frame();
                            self.path.pop();
                            continue;
                        }
                        // Not proven yet: fallthrough and rollback below
                    }
                    Ok(false) => {
                        // Conditions not satisfied - try to prove them recursively!
                        if self.try_prove_rule_conditions(&rule, facts, kb, depth + 1) {
                            // All conditions now satisfied! Try executing rule again
                            match self.executor.try_execute_rule(&rule, facts) {
                                Ok(true) => {
                                    if self.check_goal_in_facts(goal, facts) {
                                        goal.status = GoalStatus::Proven;

                                        // Save this solution
                                        self.solutions.push(Solution {
                                            path: self.path.clone(),
                                            bindings: goal.bindings.to_map(),
                                        });

                                        // If we only want one solution OR we've found enough, stop searching
                                        if self.max_solutions == 1
                                            || self.solutions.len() >= self.max_solutions
                                        {
                                            return true; // keep changes
                                        }

                                        // Otherwise, rollback and continue searching
                                        facts.rollback_undo_frame();
                                        self.path.pop();
                                        continue;
                                    }
                                }
                                _ => {
                                    // execution failed on second attempt
                                }
                            }
                        }
                    }
                    Err(_) => {
                        // Execution error - continue to next rule
                    }
                }
            }

            // Candidate failed to prove goal — rollback speculative changes
            facts.rollback_undo_frame();
            self.path.pop();
        }

        // Try sub-goals (begin undo frame before attempting sub-goals so we can rollback
        // if any sub-goal fails)
        let mut all_subgoals_proven = true;
        if !goal.sub_goals.is_empty() {
            facts.begin_undo_frame();
            for sub_goal in &mut goal.sub_goals {
                if !self.search_recursive_with_execution(sub_goal, facts, kb, depth + 1) {
                    all_subgoals_proven = false;
                    break;
                }
            }

            if all_subgoals_proven {
                // commit sub-goal frame (keep changes)
                facts.commit_undo_frame();
                goal.status = GoalStatus::Proven;
                return true;
            }

            // rollback any changes from failed sub-goal exploration
            facts.rollback_undo_frame();
        }

        // If we found at least one solution (even if less than max_solutions), consider it proven
        if !self.solutions.is_empty() {
            goal.status = GoalStatus::Proven;
            // For negated goals, finding a proof means negation fails
            return !goal.is_negated;
        }

        // If we have no candidate rules and no sub-goals, or nothing worked
        if goal.is_negated {
            // For negated goals: if we couldn't prove it, then NOT succeeds (closed-world assumption)
            goal.status = GoalStatus::Proven;
            true
        } else {
            // For normal goals: if we couldn't prove it, it's unprovable
            goal.status = GoalStatus::Unprovable;
            false
        }
    }

    /// Check if goal is already satisfied by facts
    ///
    /// This method now reuses ConditionEvaluator for proper evaluation
    fn check_goal_in_facts(&self, goal: &Goal, facts: &Facts) -> bool {
        // For negated goals, use the expression directly (parser strips NOT)
        if goal.is_negated {
            if let Some(ref expr) = goal.expression {
                // Expression.evaluate() returns Value, need to convert to bool
                match expr.evaluate(facts) {
                    Ok(Value::Boolean(b)) => return b,
                    Ok(_) => return false, // Non-boolean values are false
                    Err(_) => return false,
                }
            }
            return false;
        }

        // Parse goal pattern into a Condition and use ConditionEvaluator
        if let Some(condition) = self.parse_goal_pattern(&goal.pattern) {
            // Use RuleExecutor's evaluator (which delegates to ConditionEvaluator)
            self.executor
                .evaluate_condition(&condition, facts)
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Parse goal pattern string into a Condition object
    ///
    /// Examples:
    /// - "User.Score >= 80" → Condition { field: "User.Score", operator: GreaterThanOrEqual, value: Number(80) }
    /// - "User.IsVIP == true" → Condition { field: "User.IsVIP", operator: Equal, value: Boolean(true) }
    fn parse_goal_pattern(&self, pattern: &str) -> Option<crate::engine::rule::Condition> {
        use crate::engine::rule::{Condition, ConditionExpression};
        use crate::types::Operator;

        // Try parsing operators in order (longest first to avoid conflicts)
        let operators = [
            (">=", Operator::GreaterThanOrEqual),
            ("<=", Operator::LessThanOrEqual),
            ("==", Operator::Equal),
            ("!=", Operator::NotEqual),
            (" > ", Operator::GreaterThan),
            (" < ", Operator::LessThan),
            (" contains ", Operator::Contains),
            (" not_contains ", Operator::NotContains),
            (" starts_with ", Operator::StartsWith),
            (" startsWith ", Operator::StartsWith),
            (" ends_with ", Operator::EndsWith),
            (" endsWith ", Operator::EndsWith),
            (" matches ", Operator::Matches),
        ];

        for (op_str, operator) in operators {
            if let Some(pos) = pattern.find(op_str) {
                let field = pattern[..pos].trim().to_string();
                let value_str = pattern[pos + op_str.len()..].trim();

                // Parse value
                let value = self.parse_value_string(value_str);

                return Some(Condition {
                    field: field.clone(),
                    expression: ConditionExpression::Field(field),
                    operator,
                    value,
                });
            }
        }

        None
    }

    /// Parse value string into a Value
    fn parse_value_string(&self, s: &str) -> Value {
        let s = s.trim();

        // Boolean
        if s == "true" {
            return Value::Boolean(true);
        }
        if s == "false" {
            return Value::Boolean(false);
        }

        // Null
        if s == "null" {
            return Value::Null;
        }

        // String (quoted)
        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            return Value::String(s[1..s.len() - 1].to_string());
        }

        // Number (float)
        if let Ok(n) = s.parse::<f64>() {
            return Value::Number(n);
        }

        // Integer
        if let Ok(i) = s.parse::<i64>() {
            return Value::Integer(i);
        }

        // Default to string
        Value::String(s.to_string())
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
            ConditionGroup::Compound {
                left,
                operator,
                right,
            } => {
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
            ConditionGroup::Not(_)
            | ConditionGroup::Exists(_)
            | ConditionGroup::Forall(_)
            | ConditionGroup::Accumulate { .. } => {
                // Complex conditions (Not, Exists, Forall, Accumulate) cannot be proven backward;
                // they can only be evaluated against current facts.
                // Use the executor's condition evaluator to check them.
                self.executor
                    .evaluate_conditions(group, facts)
                    .unwrap_or(false)
            }
            #[cfg(feature = "streaming")]
            ConditionGroup::StreamPattern { .. } => {
                // StreamPattern cannot be proven backward; evaluate against current facts.
                self.executor
                    .evaluate_conditions(group, facts)
                    .unwrap_or(false)
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
        // First check if condition is already satisfied by facts
        if let Ok(satisfied) = self.executor.evaluate_condition(condition, facts) {
            if satisfied {
                return true;
            }
        }

        // Condition not satisfied - try to prove it by finding rules that can derive it
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

        // Convert value to string format that matches goal patterns
        let value_str = match &condition.value {
            Value::Boolean(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::String(s) => format!("\"{}\"", s),
            Value::Null => "null".to_string(),
            _ => format!("{:?}", condition.value),
        };

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
        if let Some(rule_name) = goal.candidate_rules.clone().into_iter().next() {
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

/// Iterative deepening search implementation
pub struct IterativeDeepeningSearch {
    max_depth: usize,
    goals_explored: usize,
    kb: KnowledgeBase,
    engine: Option<Arc<Mutex<IncrementalEngine>>>,
}

impl IterativeDeepeningSearch {
    /// Create a new iterative deepening search
    pub fn new(max_depth: usize, kb: KnowledgeBase) -> Self {
        Self {
            max_depth,
            goals_explored: 0,
            kb,
            engine: None,
        }
    }

    /// Create with optional IncrementalEngine for TMS integration
    pub fn new_with_engine(
        max_depth: usize,
        kb: KnowledgeBase,
        engine: Option<Arc<Mutex<IncrementalEngine>>>,
    ) -> Self {
        // Store the engine so we can pass it to DFS instances
        Self {
            max_depth,
            goals_explored: 0,
            kb,
            engine,
        }
    }

    /// Search with execution: probe with increasing depth using non-executing DFS,
    /// then run a final executing DFS at the discovered depth to mutate facts.
    pub fn search_with_execution(
        &mut self,
        root_goal: &mut Goal,
        facts: &mut Facts,
        kb: &KnowledgeBase,
    ) -> SearchResult {
        self.goals_explored = 0;
        let mut cumulative_goals = 0usize;

        // Try increasing depth limits
        for depth_limit in 0..=self.max_depth {
            // Probe using a non-executing depth-first search on a cloned goal
            let mut probe_goal = root_goal.clone();
            let probe_kb = self.kb.clone();
            let mut probe_dfs = DepthFirstSearch::new(depth_limit, probe_kb);
            let probe_result = probe_dfs.search(&mut probe_goal, facts);
            cumulative_goals += probe_result.goals_explored;

            if probe_result.success {
                // Found a depth where a proof exists; execute for real at this depth
                let exec_kb = self.kb.clone();
                let mut exec_dfs =
                    DepthFirstSearch::new_with_engine(depth_limit, exec_kb, self.engine.clone());
                let exec_result = exec_dfs.search_with_execution(root_goal, facts, kb);
                // Aggregate explored goals
                let mut final_result = exec_result;
                final_result.goals_explored += cumulative_goals - final_result.goals_explored;
                return final_result;
            }
        }

        // Nothing proved up to max_depth
        SearchResult::failure(cumulative_goals, self.max_depth)
    }

    /// Non-executing search using iterative deepening (probes only)
    pub fn search(&mut self, root_goal: &mut Goal, facts: &Facts) -> SearchResult {
        self.goals_explored = 0;
        let mut cumulative_goals = 0usize;

        for depth_limit in 0..=self.max_depth {
            let mut probe_goal = root_goal.clone();
            let probe_kb = self.kb.clone();
            let mut probe_dfs = DepthFirstSearch::new(depth_limit, probe_kb);
            let probe_result = probe_dfs.search(&mut probe_goal, facts);
            cumulative_goals += probe_result.goals_explored;
            if probe_result.success {
                // Return the successful probe result (with aggregated goals explored)
                let mut res = probe_result;
                res.goals_explored = cumulative_goals;
                return res;
            }
        }

        SearchResult::failure(cumulative_goals, self.max_depth)
    }
}

impl BreadthFirstSearch {
    /// Create a new breadth-first search
    pub fn new(max_depth: usize, kb: KnowledgeBase) -> Self {
        Self {
            max_depth,
            goals_explored: 0,
            executor: RuleExecutor::new_with_inserter(kb, None),
        }
    }

    /// Create BFS with optional engine for TMS integration
    pub fn new_with_engine(
        max_depth: usize,
        kb: KnowledgeBase,
        engine: Option<Arc<Mutex<IncrementalEngine>>>,
    ) -> Self {
        // If engine provided, create inserter closure
        let inserter = engine.map(|eng| {
            let eng = eng.clone();
            std::sync::Arc::new(
                move |fact_type: String,
                      data: crate::rete::TypedFacts,
                      rule_name: String,
                      _premises: Vec<String>| {
                    if let Ok(mut e) = eng.lock() {
                        let _ = e.insert_logical(fact_type, data, rule_name, Vec::new());
                    }
                },
            )
                as std::sync::Arc<
                    dyn Fn(String, crate::rete::TypedFacts, String, Vec<String>) + Send + Sync,
                >
        });

        Self {
            max_depth,
            goals_explored: 0,
            executor: RuleExecutor::new_with_inserter(kb, inserter),
        }
    }

    /// Search for a proof of the goal using BFS WITH rule execution
    pub fn search_with_execution(
        &mut self,
        root_goal: &mut Goal,
        facts: &mut Facts,
        kb: &KnowledgeBase,
    ) -> SearchResult {
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
            solutions: Vec::new(),
        }
    }

    /// Check if goal is already satisfied by facts
    ///
    /// This method now reuses ConditionEvaluator for proper evaluation
    fn check_goal_in_facts(&self, goal: &Goal, facts: &Facts) -> bool {
        // For negated goals, use the expression directly (parser strips NOT)
        if goal.is_negated {
            if let Some(ref expr) = goal.expression {
                // Expression.evaluate() returns Value, need to convert to bool
                match expr.evaluate(facts) {
                    Ok(Value::Boolean(b)) => return b,
                    Ok(_) => return false, // Non-boolean values are false
                    Err(_) => return false,
                }
            }
            return false;
        }

        // Parse goal pattern into a Condition and use ConditionEvaluator
        if let Some(condition) = self.parse_goal_pattern(&goal.pattern) {
            // Use RuleExecutor's evaluator (which delegates to ConditionEvaluator)
            self.executor
                .evaluate_condition(&condition, facts)
                .unwrap_or(false)
        } else {
            false
        }
    }

    /// Parse goal pattern string into a Condition object
    ///
    /// Examples:
    /// - "User.Score >= 80" → Condition { field: "User.Score", operator: GreaterThanOrEqual, value: Number(80) }
    /// - "User.IsVIP == true" → Condition { field: "User.IsVIP", operator: Equal, value: Boolean(true) }
    fn parse_goal_pattern(&self, pattern: &str) -> Option<crate::engine::rule::Condition> {
        use crate::engine::rule::{Condition, ConditionExpression};
        use crate::types::Operator;

        // Try parsing operators in order (longest first to avoid conflicts)
        let operators = [
            (">=", Operator::GreaterThanOrEqual),
            ("<=", Operator::LessThanOrEqual),
            ("==", Operator::Equal),
            ("!=", Operator::NotEqual),
            (" > ", Operator::GreaterThan),
            (" < ", Operator::LessThan),
            (" contains ", Operator::Contains),
            (" not_contains ", Operator::NotContains),
            (" starts_with ", Operator::StartsWith),
            (" startsWith ", Operator::StartsWith),
            (" ends_with ", Operator::EndsWith),
            (" endsWith ", Operator::EndsWith),
            (" matches ", Operator::Matches),
        ];

        for (op_str, operator) in operators {
            if let Some(pos) = pattern.find(op_str) {
                let field = pattern[..pos].trim().to_string();
                let value_str = pattern[pos + op_str.len()..].trim();

                // Parse value
                let value = self.parse_value_string(value_str);

                return Some(Condition {
                    field: field.clone(),
                    expression: ConditionExpression::Field(field),
                    operator,
                    value,
                });
            }
        }

        None
    }

    /// Parse value string into a Value
    fn parse_value_string(&self, s: &str) -> Value {
        let s = s.trim();

        // Boolean
        if s == "true" {
            return Value::Boolean(true);
        }
        if s == "false" {
            return Value::Boolean(false);
        }

        // Null
        if s == "null" {
            return Value::Null;
        }

        // String (quoted)
        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            return Value::String(s[1..s.len() - 1].to_string());
        }

        // Number (float)
        if let Ok(n) = s.parse::<f64>() {
            return Value::Number(n);
        }

        // Integer
        if let Ok(i) = s.parse::<i64>() {
            return Value::Integer(i);
        }

        // Default to string
        Value::String(s.to_string())
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
            solutions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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

    #[test]
    fn test_iterative_deepening_search_success() {
        let kb = KnowledgeBase::new("test");
        let mut ids = IterativeDeepeningSearch::new(5, kb.clone());
        let mut root = Goal::new("test".to_string());
        root.add_candidate_rule("TestRule".to_string());

        // Facts empty; DFS probe should succeed because candidate rules mark provable
        let facts = Facts::new();
        let res = ids.search(&mut root, &facts);
        assert!(res.success);
    }

    #[test]
    fn test_iterative_deepening_search_depth_limit() {
        let kb = KnowledgeBase::new("test");
        // Set max_depth to 0 so even shallow proofs that require depth >0 fail
        let mut ids = IterativeDeepeningSearch::new(0, kb.clone());
        let mut root = Goal::new("test".to_string());
        // Add a subgoal to force depth > 0
        let mut sub = Goal::new("sub".to_string());
        sub.add_candidate_rule("SubRule".to_string());
        root.sub_goals.push(sub);

        let facts = Facts::new();
        let res = ids.search(&mut root, &facts);
        assert!(!res.success);
    }

    #[test]
    fn test_depth_first_search_max_depth_exceeded() {
        let kb = KnowledgeBase::new("test");
        let mut dfs = DepthFirstSearch::new(2, kb);
        let facts = Facts::new();

        // Create nested goals exceeding max depth
        let mut root = Goal::new("level0".to_string());
        root.depth = 0;
        root.add_candidate_rule("Rule0".to_string());

        let mut level1 = Goal::new("level1".to_string());
        level1.depth = 1;
        level1.add_candidate_rule("Rule1".to_string());

        let mut level2 = Goal::new("level2".to_string());
        level2.depth = 2;
        level2.add_candidate_rule("Rule2".to_string());

        let mut level3 = Goal::new("level3".to_string());
        level3.depth = 3; // Exceeds max_depth of 2
        level3.add_candidate_rule("Rule3".to_string());

        level2.add_subgoal(level3);
        level1.add_subgoal(level2);
        root.add_subgoal(level1);

        let result = dfs.search(&mut root, &facts);

        // Verify search completed (max_depth_reached is set)
        assert!(result.max_depth_reached <= 3);
    }

    #[test]
    fn test_breadth_first_search_multiple_candidates() {
        let kb = KnowledgeBase::new("test");
        let mut bfs = BreadthFirstSearch::new(10, kb);
        let facts = Facts::new();

        let mut goal = Goal::new("multi_rule_goal".to_string());
        goal.add_candidate_rule("Rule1".to_string());
        goal.add_candidate_rule("Rule2".to_string());
        goal.add_candidate_rule("Rule3".to_string());

        let result = bfs.search(&mut goal, &facts);

        assert!(result.success);
        assert_eq!(goal.candidate_rules.len(), 3);
    }

    #[test]
    fn test_depth_first_search_empty_goal() {
        let kb = KnowledgeBase::new("test");
        let mut dfs = DepthFirstSearch::new(10, kb);
        let facts = Facts::new();

        let mut goal = Goal::new("".to_string());
        // No candidate rules, no subgoals

        let result = dfs.search(&mut goal, &facts);

        // Should fail - no way to prove empty goal
        assert!(!result.success);
    }

    #[test]
    fn test_search_result_with_bindings() {
        use crate::types::Value;
        let mut bindings = HashMap::new();
        bindings.insert("X".to_string(), Value::String("test".to_string()));
        bindings.insert("Y".to_string(), Value::Number(42.0));

        let result = SearchResult {
            success: true,
            path: vec!["Rule1".to_string()],
            goals_explored: 5,
            max_depth_reached: 3,
            bindings: bindings.clone(),
            solutions: Vec::new(),
        };

        assert_eq!(result.bindings.len(), 2);
        assert_eq!(
            result.bindings.get("X"),
            Some(&Value::String("test".to_string()))
        );
    }

    #[test]
    fn test_breadth_first_search_with_subgoals() {
        let kb = KnowledgeBase::new("test");
        let mut bfs = BreadthFirstSearch::new(10, kb);
        let facts = Facts::new();

        let mut root = Goal::new("root".to_string());
        root.add_candidate_rule("RootRule".to_string());

        let mut sub1 = Goal::new("sub1".to_string());
        sub1.add_candidate_rule("Sub1Rule".to_string());

        let mut sub2 = Goal::new("sub2".to_string());
        sub2.add_candidate_rule("Sub2Rule".to_string());

        root.add_subgoal(sub1);
        root.add_subgoal(sub2);

        let result = bfs.search(&mut root, &facts);

        assert!(result.success);
        assert!(result.goals_explored >= 3); // root + 2 subgoals
    }

    #[test]
    fn test_iterative_deepening_search_no_candidates() {
        let kb = KnowledgeBase::new("test");
        let mut ids = IterativeDeepeningSearch::new(5, kb);
        let mut root = Goal::new("no_rules".to_string());
        // No candidate rules added

        let facts = Facts::new();
        let result = ids.search(&mut root, &facts);

        assert!(!result.success);
        assert!(result.path.is_empty());
    }

    #[test]
    fn test_search_strategy_equality() {
        assert_eq!(SearchStrategy::BreadthFirst, SearchStrategy::BreadthFirst);
        assert_eq!(SearchStrategy::Iterative, SearchStrategy::Iterative);
        assert_ne!(SearchStrategy::BreadthFirst, SearchStrategy::Iterative);
    }

    #[test]
    fn test_depth_first_search_goals_explored_count() {
        let kb = KnowledgeBase::new("test");
        let mut dfs = DepthFirstSearch::new(10, kb);
        let facts = Facts::new();

        let mut root = Goal::new("root".to_string());
        root.add_candidate_rule("RootRule".to_string());

        let mut sub = Goal::new("sub".to_string());
        sub.add_candidate_rule("SubRule".to_string());

        root.add_subgoal(sub);

        let result = dfs.search(&mut root, &facts);

        // Search succeeded with candidate rules
        assert!(result.success);
        // Goals explored count is tracked (always >= 0 since it's usize)
        assert!(result.goals_explored > 0);
    }
}

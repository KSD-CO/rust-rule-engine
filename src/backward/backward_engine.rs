//! Backward chaining engine implementation

use super::goal::{Goal, GoalManager, GoalStatus};
use super::search::{DepthFirstSearch, BreadthFirstSearch, IterativeDeepeningSearch, SearchStrategy, SearchResult};
use super::query::{QueryParser, QueryResult, QueryStats, ProofTrace};
use super::conclusion_index::ConclusionIndex;
use crate::{Facts, KnowledgeBase};
use crate::errors::Result;
use std::sync::Arc;

/// Configuration for backward chaining engine
#[derive(Debug, Clone)]
pub struct BackwardConfig {
    /// Maximum depth for goal search
    pub max_depth: usize,
    
    /// Search strategy to use
    pub strategy: SearchStrategy,
    
    /// Enable memoization of proven goals
    pub enable_memoization: bool,
    
    /// Maximum number of solutions to find
    pub max_solutions: usize,
}

impl Default for BackwardConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            strategy: SearchStrategy::DepthFirst,
            enable_memoization: true,
            max_solutions: 1,
        }
    }
}

/// Backward chaining engine for goal-driven reasoning
pub struct BackwardEngine {
    knowledge_base: Arc<KnowledgeBase>,
    config: BackwardConfig,
    goal_manager: GoalManager,
    /// RETE-style conclusion index for O(1) rule lookup
    conclusion_index: ConclusionIndex,
}

impl BackwardEngine {
    /// Create a new backward chaining engine
    pub fn new(kb: KnowledgeBase) -> Self {
        let rules = kb.get_rules();
        let conclusion_index = ConclusionIndex::from_rules(&rules);

        Self {
            knowledge_base: Arc::new(kb),
            config: BackwardConfig::default(),
            goal_manager: GoalManager::default(),
            conclusion_index,
        }
    }

    /// Create with custom configuration
    pub fn with_config(kb: KnowledgeBase, config: BackwardConfig) -> Self {
        let rules = kb.get_rules();
        let conclusion_index = ConclusionIndex::from_rules(&rules);

        Self {
            knowledge_base: Arc::new(kb),
            goal_manager: GoalManager::new(config.max_depth),
            config,
            conclusion_index,
        }
    }
    
    /// Update configuration
    pub fn set_config(&mut self, config: BackwardConfig) {
        self.goal_manager = GoalManager::new(config.max_depth);
        self.config = config;
    }
    
    /// Query whether a goal can be proven
    ///
    /// # Example
    ///
    /// ```ignore
    /// let engine = BackwardEngine::new(kb);
    /// let result = engine.query("User.IsVIP == true", &mut facts)?;
    ///
    /// if result.provable {
    ///     println!("User is VIP!");
    /// }
    /// ```
    pub fn query(&mut self, query_str: &str, facts: &mut Facts) -> Result<QueryResult> {
        // Backward-compatible: no RETE engine provided
        self.query_with_rete_engine(query_str, facts, None)
    }

    /// Query with optional RETE IncrementalEngine for TMS integration
    pub fn query_with_rete_engine(
        &mut self,
        query_str: &str,
        facts: &mut Facts,
        rete_engine: Option<std::sync::Arc<std::sync::Mutex<crate::rete::propagation::IncrementalEngine>>>,
    ) -> Result<QueryResult> {
        // Parse query into goal
        let mut goal = QueryParser::parse(query_str)
            .map_err(|e| crate::errors::RuleEngineError::ParseError { message: e })?;

        // Check cache if memoization enabled
        if self.config.enable_memoization {
            if let Some(cached) = self.goal_manager.is_cached(query_str) {
                return Ok(if cached {
                    QueryResult::success(
                        goal.bindings.to_map(), // Convert Bindings to HashMap
                        ProofTrace::from_goal(&goal),
                        QueryStats::default(),
                    )
                } else {
                    QueryResult::failure(vec![], QueryStats::default())
                });
            }
        }

        // Find candidate rules that can prove this goal
        self.find_candidate_rules(&mut goal)?;

        // Execute search strategy with optional rete_engine
        let search_result = match self.config.strategy {
            SearchStrategy::DepthFirst => {
                let mut dfs = DepthFirstSearch::new_with_engine(self.config.max_depth, (*self.knowledge_base).clone(), rete_engine.clone())
                    .with_max_solutions(self.config.max_solutions);
                dfs.search_with_execution(&mut goal, facts, &self.knowledge_base)
            }
            SearchStrategy::BreadthFirst => {
                let mut bfs = BreadthFirstSearch::new_with_engine(self.config.max_depth, (*self.knowledge_base).clone(), rete_engine.clone());
                bfs.search_with_execution(&mut goal, facts, &self.knowledge_base)
            }
            SearchStrategy::Iterative => {
                let mut ids = IterativeDeepeningSearch::new_with_engine(self.config.max_depth, (*self.knowledge_base).clone(), rete_engine.clone());
                ids.search_with_execution(&mut goal, facts, &self.knowledge_base)
            }
        };

        // Cache result if enabled
        if self.config.enable_memoization {
            self.goal_manager.cache_result(query_str.to_string(), search_result.success);
        }

        // Build query result
        let stats = QueryStats {
            goals_explored: search_result.goals_explored,
            rules_evaluated: search_result.path.len(),
            max_depth: search_result.max_depth_reached,
            duration_ms: None,
        };

        Ok(if search_result.success {
            QueryResult::success(
                search_result.bindings,
                ProofTrace::from_goal(&goal),
                stats,
            )
        } else {
            QueryResult::failure(self.find_missing_facts(&goal), stats)
        })
    }
    
    /// Find all candidate rules that could prove a goal
    ///
    /// This uses the RETE-style conclusion index for O(1) lookup
    /// instead of O(n) iteration through all rules.
    fn find_candidate_rules(&self, goal: &mut Goal) -> Result<()> {
        // Use conclusion index for O(1) lookup
        let candidates = self.conclusion_index.find_candidates(&goal.pattern);

        // Add candidate rules to goal
        for rule_name in candidates {
            goal.add_candidate_rule(rule_name);
        }

        // If no candidates found via index, fall back to checking all rules
        // This handles edge cases where index might miss some patterns
        if goal.candidate_rules.is_empty() {
            for rule in self.knowledge_base.get_rules() {
                if self.rule_could_prove_goal(&rule, goal) {
                    goal.add_candidate_rule(rule.name.clone());
                }
            }
        }

        Ok(())
    }
    
    /// Check if a rule could potentially prove a goal
    fn rule_could_prove_goal(&self, rule: &crate::engine::rule::Rule, goal: &Goal) -> bool {
        // Simple heuristic: check if goal pattern appears in rule actions
        // In a full implementation, this would be more sophisticated

        // Check rule name
        if goal.pattern.contains(&rule.name) {
            return true;
        }

        // Check actions
        for action in &rule.actions {
            match action {
                crate::types::ActionType::Set { field, .. } => {
                    if goal.pattern.contains(field) {
                        return true;
                    }
                }
                crate::types::ActionType::MethodCall { object, method, .. } => {
                    if goal.pattern.contains(object) || goal.pattern.contains(method) {
                        return true;
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Find what facts are missing to prove a goal
    fn find_missing_facts(&self, goal: &Goal) -> Vec<String> {
        let mut missing = Vec::new();
        
        // Collect unprovable sub-goals
        self.collect_missing_recursive(goal, &mut missing);
        
        missing
    }
    
    fn collect_missing_recursive(&self, goal: &Goal, missing: &mut Vec<String>) {
        if goal.status == GoalStatus::Unprovable {
            missing.push(goal.pattern.clone());
        }
        
        for sub_goal in &goal.sub_goals {
            self.collect_missing_recursive(sub_goal, missing);
        }
    }
    
    /// Explain why a goal was proven (or not)
    pub fn explain_why(&mut self, query_str: &str, facts: &mut Facts) -> Result<String> {
        let result = self.query(query_str, facts)?;

        if result.provable {
            Ok(format!(
                "Goal '{}' is PROVABLE\nProof trace:\n{:#?}",
                query_str, result.proof_trace
            ))
        } else {
            Ok(format!(
                "Goal '{}' is NOT provable\nMissing facts: {:?}",
                query_str, result.missing_facts
            ))
        }
    }
    
    /// Get the configuration
    pub fn config(&self) -> &BackwardConfig {
        &self.config
    }
    
    /// Get the knowledge base
    pub fn knowledge_base(&self) -> &KnowledgeBase {
        &self.knowledge_base
    }

    /// Get conclusion index statistics
    pub fn index_stats(&self) -> super::conclusion_index::IndexStats {
        self.conclusion_index.stats()
    }

    /// Rebuild the conclusion index (call after modifying knowledge base)
    pub fn rebuild_index(&mut self) {
        let rules = self.knowledge_base.get_rules();
        self.conclusion_index = ConclusionIndex::from_rules(&rules);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::KnowledgeBase;
    use crate::types::Value;
    
    #[test]
    fn test_engine_creation() {
        let kb = KnowledgeBase::new("test");
        let engine = BackwardEngine::new(kb);
        
        assert_eq!(engine.config.max_depth, 10);
        assert_eq!(engine.config.strategy, SearchStrategy::DepthFirst);
    }
    
    #[test]
    fn test_config_customization() {
        let kb = KnowledgeBase::new("test");
        let config = BackwardConfig {
            max_depth: 5,
            strategy: SearchStrategy::BreadthFirst,
            enable_memoization: false,
            max_solutions: 10,
        };
        
        let engine = BackwardEngine::with_config(kb, config);
        assert_eq!(engine.config.max_depth, 5);
        assert_eq!(engine.config.strategy, SearchStrategy::BreadthFirst);
        assert!(!engine.config.enable_memoization);
    }
    
    #[test]
    fn test_query_simple() {
        let kb = KnowledgeBase::new("test");
        let mut engine = BackwardEngine::new(kb);
        let mut facts = Facts::new();

        // This will fail because no rules, but tests the flow
        let result = engine.query("User.IsVIP == true", &mut facts);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_call_condition_len() {
        use crate::engine::rule::{Rule, Condition, ConditionGroup};
        use crate::types::{Operator, ActionType};

        let mut kb = KnowledgeBase::new("test");

        // Rule: If User.Name length > 3, then User.HasLongName = true
        let conditions = ConditionGroup::Single(
            Condition::with_function(
                "len".to_string(),
                vec!["User.Name".to_string()],
                Operator::GreaterThan,
                Value::Number(3.0),
            )
        );
        let actions = vec![ActionType::Set {
            field: "User.HasLongName".to_string(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new("CheckNameLength".to_string(), conditions, actions);
        kb.add_rule(rule);

        let mut engine = BackwardEngine::new(kb);
        let mut facts = Facts::new();
        facts.set("User.Name", Value::String("John".to_string()));

        // Query if User.HasLongName == true
        let result = engine.query("User.HasLongName == true", &mut facts);
        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should be provable because "John".len() = 4 > 3
        assert!(query_result.provable, "Query should be provable with len() function");
    }

    #[test]
    fn test_function_call_condition_isempty() {
        use crate::engine::rule::{Rule, Condition, ConditionGroup};
        use crate::types::{Operator, ActionType};

        let mut kb = KnowledgeBase::new("test");

        // Rule: If User.Description is NOT empty, then User.HasDescription = true
        let conditions = ConditionGroup::Single(
            Condition::with_function(
                "isEmpty".to_string(),
                vec!["User.Description".to_string()],
                Operator::Equal,
                Value::Boolean(false),
            )
        );
        let actions = vec![ActionType::Set {
            field: "User.HasDescription".to_string(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new("CheckDescription".to_string(), conditions, actions);
        kb.add_rule(rule);

        let mut engine = BackwardEngine::new(kb);
        let mut facts = Facts::new();
        facts.set("User.Description", Value::String("A great user".to_string()));

        let result = engine.query("User.HasDescription == true", &mut facts);
        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should be provable because description is not empty
        assert!(query_result.provable, "Query should be provable with isEmpty() function");
    }

    #[test]
    fn test_test_expression_exists() {
        use crate::engine::rule::{Rule, Condition, ConditionGroup, ConditionExpression};
        use crate::types::{Operator, ActionType};

        let mut kb = KnowledgeBase::new("test");

        // Rule: If User.Email exists, then User.HasEmail = true
        let condition = Condition {
            field: "User.Email".to_string(),  // Required field
            expression: ConditionExpression::Test {
                name: "exists".to_string(),
                args: vec!["User.Email".to_string()],
            },
            operator: Operator::Equal,
            value: Value::Boolean(true),
        };
        let conditions = ConditionGroup::Single(condition);
        let actions = vec![ActionType::Set {
            field: "User.HasEmail".to_string(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new("CheckEmail".to_string(), conditions, actions);
        kb.add_rule(rule);

        let mut engine = BackwardEngine::new(kb);
        let mut facts = Facts::new();
        facts.set("User.Email", Value::String("user@example.com".to_string()));

        let result = engine.query("User.HasEmail == true", &mut facts);
        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should be provable because User.Email exists
        assert!(query_result.provable, "Query should be provable with exists test");
    }

    #[test]
    fn test_multifield_count_operation() {
        use crate::engine::rule::{Rule, Condition, ConditionGroup, ConditionExpression};
        use crate::types::{Operator, ActionType};

        let mut kb = KnowledgeBase::new("test");

        // Rule: If User.Orders count > 5, then User.IsFrequentBuyer = true
        let condition = Condition {
            field: "User.Orders".to_string(),  // Required field
            expression: ConditionExpression::MultiField {
                field: "User.Orders".to_string(),
                operation: "count".to_string(),
                variable: None,
            },
            operator: Operator::GreaterThan,
            value: Value::Number(5.0),
        };
        let conditions = ConditionGroup::Single(condition);
        let actions = vec![ActionType::Set {
            field: "User.IsFrequentBuyer".to_string(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new("CheckFrequentBuyer".to_string(), conditions, actions);
        kb.add_rule(rule);

        let mut engine = BackwardEngine::new(kb);
        let mut facts = Facts::new();

        // Create array of 6 orders
        let orders = Value::Array(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
            Value::Number(4.0),
            Value::Number(5.0),
            Value::Number(6.0),
        ]);
        facts.set("User.Orders", orders);

        let result = engine.query("User.IsFrequentBuyer == true", &mut facts);
        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should be provable because orders.count() = 6 > 5
        assert!(query_result.provable, "Query should be provable with count multifield operation");
    }

    #[test]
    fn test_fact_derivation_basic() {
        use crate::engine::rule::{Rule, Condition, ConditionGroup};
        use crate::types::{Operator, ActionType};

        let mut kb = KnowledgeBase::new("test");

        // Rule: If User.Age > 18, then User.IsAdult = true
        let conditions = ConditionGroup::Single(
            Condition::new(
                "User.Age".to_string(),
                Operator::GreaterThan,
                Value::Number(18.0),
            )
        );
        let actions = vec![ActionType::Set {
            field: "User.IsAdult".to_string(),
            value: Value::Boolean(true),
        }];

        let rule = Rule::new("DetermineAdult".to_string(), conditions, actions);
        kb.add_rule(rule);

        let mut engine = BackwardEngine::new(kb);
        let mut facts = Facts::new();
        facts.set("User.Age", Value::Number(25.0));

        // Query will trigger rule execution which should set User.IsAdult
        let result = engine.query("User.IsAdult == true", &mut facts);
        assert!(result.is_ok());
        let query_result = result.unwrap();

        assert!(query_result.provable, "Fact should be derived by rule action");
    }

    #[test]
    fn test_rule_chaining_two_levels() {
        use crate::engine::rule::{Rule, Condition, ConditionGroup};
        use crate::types::{Operator, ActionType, LogicalOperator};

        let mut kb = KnowledgeBase::new("test");

        // Rule 1: If User.LoyaltyPoints > 100, then User.IsVIP = true
        let conditions1 = ConditionGroup::Single(
            Condition::new(
                "User.LoyaltyPoints".to_string(),
                Operator::GreaterThan,
                Value::Number(100.0),
            )
        );
        let actions1 = vec![ActionType::Set {
            field: "User.IsVIP".to_string(),
            value: Value::Boolean(true),
        }];
        let rule1 = Rule::new("DetermineVIP".to_string(), conditions1, actions1);

        // Rule 2: If User.IsVIP == true AND Order.Amount < 10000, then Order.AutoApproved = true
        let conditions2 = ConditionGroup::Compound {
            left: Box::new(ConditionGroup::Single(
                Condition::new(
                    "User.IsVIP".to_string(),
                    Operator::Equal,
                    Value::Boolean(true),
                )
            )),
            operator: LogicalOperator::And,
            right: Box::new(ConditionGroup::Single(
                Condition::new(
                    "Order.Amount".to_string(),
                    Operator::LessThan,
                    Value::Number(10000.0),
                )
            )),
        };
        let actions2 = vec![ActionType::Set {
            field: "Order.AutoApproved".to_string(),
            value: Value::Boolean(true),
        }];
        let rule2 = Rule::new("AutoApproveVIP".to_string(), conditions2, actions2);

        kb.add_rule(rule1);
        kb.add_rule(rule2);

        let mut engine = BackwardEngine::new(kb);
        let mut facts = Facts::new();
        facts.set("User.LoyaltyPoints", Value::Number(150.0));
        facts.set("Order.Amount", Value::Number(5000.0));

        // Query Order.AutoApproved - should chain through:
        // 1. Check Order.AutoApproved rule (rule2)
        // 2. Needs User.IsVIP == true
        // 3. Check User.IsVIP rule (rule1)
        // 4. Verify User.LoyaltyPoints > 100 (satisfied)
        // 5. Execute rule1 action -> sets User.IsVIP = true
        // 6. Now rule2 conditions satisfied -> sets Order.AutoApproved = true
        let result = engine.query("Order.AutoApproved == true", &mut facts);
        assert!(result.is_ok());
        let query_result = result.unwrap();

        assert!(query_result.provable, "Query should be provable through 2-level rule chaining");
    }

    #[test]
    fn test_fact_derivation_with_log_action() {
        use crate::engine::rule::{Rule, Condition, ConditionGroup};
        use crate::types::{Operator, ActionType};

        let mut kb = KnowledgeBase::new("test");

        // Rule: If User.Score > 90, log message and set HighScore
        let conditions = ConditionGroup::Single(
            Condition::new(
                "User.Score".to_string(),
                Operator::GreaterThan,
                Value::Number(90.0),
            )
        );
        let actions = vec![
            ActionType::Log {
                message: "High score achieved!".to_string(),
            },
            ActionType::Set {
                field: "User.HasHighScore".to_string(),
                value: Value::Boolean(true),
            }
        ];

        let rule = Rule::new("HighScoreRule".to_string(), conditions, actions);
        kb.add_rule(rule);

        let mut engine = BackwardEngine::new(kb);
        let mut facts = Facts::new();
        facts.set("User.Score", Value::Number(95.0));

        let result = engine.query("User.HasHighScore == true", &mut facts);
        assert!(result.is_ok());
        let query_result = result.unwrap();

        // Should be provable and log action should have been executed
        assert!(query_result.provable, "Query should be provable with log action");
    }
}

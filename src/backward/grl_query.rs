//! GRL Query Syntax Implementation
//!
//! This module provides parsing and execution of backward chaining queries defined in GRL
//! (Goal-driven Rule Language) syntax. GRL queries allow you to define goal-driven reasoning
//! tasks with configurable search strategies and action handlers.
//!
//! # Features
//!
//! - **Declarative query syntax** - Define queries in a readable, structured format
//! - **Multiple search strategies** - Choose between depth-first, breadth-first, or iterative deepening
//! - **Action handlers** - Execute actions on query success, failure, or missing facts
//! - **Conditional execution** - Use `when` clauses to conditionally execute queries
//! - **Parameterized queries** - Support for query parameters (future enhancement)
//!
//! # GRL Query Syntax
//!
//! ```grl
//! query "QueryName" {
//!     goal: <expression>                    // Required: Goal to prove
//!     strategy: <depth-first|breadth-first|iterative>  // Optional: Search strategy
//!     max-depth: <number>                   // Optional: Maximum search depth
//!     max-solutions: <number>               // Optional: Maximum solutions to find
//!     enable-memoization: <true|false>      // Optional: Enable result caching
//!
//!     when: <condition>                     // Optional: Only execute if condition is true
//!
//!     on-success: {                         // Optional: Actions on successful proof
//!         <variable> = <value>;
//!         <FunctionName>(<args>);
//!     }
//!
//!     on-failure: {                         // Optional: Actions on proof failure
//!         <actions>
//!     }
//!
//!     on-missing: {                         // Optional: Actions when facts are missing
//!         <actions>
//!     }
//! }
//! ```
//!
//! # Example
//!
//! ```rust
//! use rust_rule_engine::backward::grl_query::{GRLQueryParser, GRLQueryExecutor};
//! use rust_rule_engine::backward::BackwardEngine;
//! use rust_rule_engine::{KnowledgeBase, Facts, Value};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let query_str = r#"
//! query "CheckVIPStatus" {
//!     goal: User.IsVIP == true
//!     strategy: depth-first
//!     max-depth: 10
//!     on-success: {
//!         User.DiscountRate = 0.2;
//!         LogMessage("VIP confirmed");
//!     }
//!     on-failure: {
//!         LogMessage("Not a VIP user");
//!     }
//! }
//! "#;
//!
//! let query = GRLQueryParser::parse(query_str)?;
//! let mut bc_engine = BackwardEngine::new(KnowledgeBase::new("kb"));
//! let mut facts = Facts::new();
//! facts.set("User.LoyaltyPoints", Value::Number(1500.0));
//!
//! let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts)?;
//!
//! if result.provable {
//!     println!("Goal proven!");
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Supported Functions in Actions
//!
//! - `LogMessage(message)` - Print a log message
//! - `Request(message)` - Send a request message
//! - `Print(message)` - Print output
//! - `Debug(message)` - Print debug output to stderr

use crate::errors::RuleEngineError;
use crate::{Facts, Value};
use super::backward_engine::{BackwardEngine, BackwardConfig};
use super::search::SearchStrategy;
use super::query::{QueryResult, QueryStats, ProofTrace};
use super::optimizer::QueryOptimizer;

use std::collections::HashMap;

/// Search strategy option for queries
#[derive(Debug, Clone, PartialEq)]
pub enum GRLSearchStrategy {
    DepthFirst,
    BreadthFirst,
    Iterative,
}

impl Default for GRLSearchStrategy {
    fn default() -> Self {
        GRLSearchStrategy::DepthFirst
    }
}

/// Action to execute based on query result
#[derive(Debug, Clone)]
pub struct QueryAction {
    /// Assignment: Variable = Value (as string to be parsed)
    pub assignments: Vec<(String, String)>,
    /// Function/method calls
    pub calls: Vec<String>,
}

impl QueryAction {
    pub fn new() -> Self {
        QueryAction {
            assignments: Vec::new(),
            calls: Vec::new(),
        }
    }

    /// Execute the action on the given facts
    pub fn execute(&self, facts: &mut Facts) -> Result<(), RuleEngineError> {
        // Execute assignments - for now just log them
        for (var_name, value_str) in &self.assignments {
            // Simple value parsing
            let value = if value_str == "true" {
                Value::Boolean(true)
            } else if value_str == "false" {
                Value::Boolean(false)
            } else if let Ok(n) = value_str.parse::<f64>() {
                Value::Number(n)
            } else {
                // Remove quotes if present
                let cleaned = value_str.trim_matches('"');
                Value::String(cleaned.to_string())
            };
            
            facts.set(var_name, value);
        }

        // Execute function calls
        for call in &self.calls {
            self.execute_function_call(call)?;
        }

        Ok(())
    }

    /// Execute a single function call
    fn execute_function_call(&self, call: &str) -> Result<(), RuleEngineError> {
        let call = call.trim();

        // Parse function name and arguments
        if let Some(open_paren) = call.find('(') {
            let func_name = call[..open_paren].trim();

            // Extract arguments (everything between first ( and last ))
            if let Some(close_paren) = call.rfind(')') {
                let args_str = &call[open_paren + 1..close_paren];

                match func_name {
                    "LogMessage" => {
                        // Parse string argument (remove quotes if present)
                        let message = args_str.trim().trim_matches('"').trim_matches('\'');
                        println!("[LOG] {}", message);
                    }
                    "Request" => {
                        // Parse request call
                        let message = args_str.trim().trim_matches('"').trim_matches('\'');
                        println!("[REQUEST] {}", message);
                    }
                    "Print" => {
                        // Generic print function
                        let message = args_str.trim().trim_matches('"').trim_matches('\'');
                        println!("{}", message);
                    }
                    "Debug" => {
                        // Debug output
                        let message = args_str.trim().trim_matches('"').trim_matches('\'');
                        eprintln!("[DEBUG] {}", message);
                    }
                    other => {
                        // Unknown function - log warning but don't fail
                        eprintln!("[WARNING] Unknown function call in query action: {}({})", other, args_str);
                    }
                }
            } else {
                return Err(RuleEngineError::ParseError {
                    message: format!("Malformed function call (missing closing paren): {}", call),
                });
            }
        } else {
            return Err(RuleEngineError::ParseError {
                message: format!("Malformed function call (missing opening paren): {}", call),
            });
        }

        Ok(())
    }
}

/// A GRL Query definition
#[derive(Debug, Clone)]
pub struct GRLQuery {
    /// Query name
    pub name: String,

    /// Goal pattern to prove (as string expression)
    pub goal: String,

    /// Search strategy
    pub strategy: GRLSearchStrategy,

    /// Maximum search depth
    pub max_depth: usize,

    /// Maximum number of solutions
    pub max_solutions: usize,

    /// Enable memoization
    pub enable_memoization: bool,

    /// Enable query optimization (goal reordering, etc.)
    pub enable_optimization: bool,

    /// Action on success
    pub on_success: Option<QueryAction>,

    /// Action on failure
    pub on_failure: Option<QueryAction>,

    /// Action on missing facts
    pub on_missing: Option<QueryAction>,

    /// Parameters for parameterized queries
    pub params: HashMap<String, String>, // param_name -> type

    /// Conditional execution (as string condition)
    pub when_condition: Option<String>,
}

impl GRLQuery {
    /// Create a new query with defaults
    pub fn new(name: String, goal: String) -> Self {
        GRLQuery {
            name,
            goal,
            strategy: GRLSearchStrategy::default(),
            max_depth: 10,
            max_solutions: 1,
            enable_memoization: true,
            enable_optimization: true,
            on_success: None,
            on_failure: None,
            on_missing: None,
            params: HashMap::new(),
            when_condition: None,
        }
    }

    /// Set search strategy
    pub fn with_strategy(mut self, strategy: GRLSearchStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set max depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Set max solutions
    pub fn with_max_solutions(mut self, max_solutions: usize) -> Self {
        self.max_solutions = max_solutions;
        self
    }

    /// Set memoization
    pub fn with_memoization(mut self, enable: bool) -> Self {
        self.enable_memoization = enable;
        self
    }

    /// Set query optimization
    pub fn with_optimization(mut self, enable: bool) -> Self {
        self.enable_optimization = enable;
        self
    }

    /// Add success action
    pub fn with_on_success(mut self, action: QueryAction) -> Self {
        self.on_success = Some(action);
        self
    }

    /// Add failure action
    pub fn with_on_failure(mut self, action: QueryAction) -> Self {
        self.on_failure = Some(action);
        self
    }

    /// Add missing facts action
    pub fn with_on_missing(mut self, action: QueryAction) -> Self {
        self.on_missing = Some(action);
        self
    }

    /// Add parameter
    pub fn with_param(mut self, name: String, type_name: String) -> Self {
        self.params.insert(name, type_name);
        self
    }

    /// Set conditional execution
    pub fn with_when(mut self, condition: String) -> Self {
        self.when_condition = Some(condition);
        self
    }

    /// Check if query should execute based on when condition
    pub fn should_execute(&self, _facts: &Facts) -> Result<bool, RuleEngineError> {
        // If there's no when condition, execute by default
        if self.when_condition.is_none() {
            return Ok(true);
        }

        // Parse and evaluate the when condition expression against the current facts
        if let Some(ref cond_str) = self.when_condition {
            use crate::backward::expression::ExpressionParser;

            match ExpressionParser::parse(cond_str) {
                Ok(expr) => Ok(expr.is_satisfied(_facts)),
                Err(e) => Err(e),
            }
        } else {
            Ok(true)
        }
    }

    /// Execute success actions
    pub fn execute_success_actions(&self, facts: &mut Facts) -> Result<(), RuleEngineError> {
        if let Some(ref action) = self.on_success {
            action.execute(facts)?;
        }
        Ok(())
    }

    /// Execute failure actions
    pub fn execute_failure_actions(&self, facts: &mut Facts) -> Result<(), RuleEngineError> {
        if let Some(ref action) = self.on_failure {
            action.execute(facts)?;
        }
        Ok(())
    }

    /// Execute missing facts actions
    pub fn execute_missing_actions(&self, facts: &mut Facts) -> Result<(), RuleEngineError> {
        if let Some(ref action) = self.on_missing {
            action.execute(facts)?;
        }
        Ok(())
    }

    /// Convert to BackwardConfig
    pub fn to_config(&self) -> BackwardConfig {
        let search_strategy = match self.strategy {
            GRLSearchStrategy::DepthFirst => SearchStrategy::DepthFirst,
            GRLSearchStrategy::BreadthFirst => SearchStrategy::BreadthFirst,
            GRLSearchStrategy::Iterative => SearchStrategy::Iterative,
        };

        BackwardConfig {
            strategy: search_strategy,
            max_depth: self.max_depth,
            enable_memoization: self.enable_memoization,
            max_solutions: self.max_solutions,
        }
    }
}

/// Parser for GRL Query syntax
pub struct GRLQueryParser;

impl GRLQueryParser {
    /// Parse a query from string
    /// 
    /// # Example
    /// ```
    /// let query_str = r#"
    /// query "CheckVIP" {
    ///     goal: User.IsVIP == true
    ///     strategy: depth-first
    /// }
    /// "#;
    /// let query = rust_rule_engine::backward::GRLQueryParser::parse(query_str).unwrap();
    /// ```
    pub fn parse(input: &str) -> Result<GRLQuery, RuleEngineError> {
        let input = input.trim();

        // Extract query name
        let name = Self::extract_query_name(input)?;

        // Extract goal
        let goal = Self::extract_goal(input)?;

        // Create base query
        let mut query = GRLQuery::new(name, goal);

        // Parse optional attributes
        if let Some(strategy) = Self::extract_strategy(input) {
            query.strategy = strategy;
        }

        if let Some(max_depth) = Self::extract_max_depth(input) {
            query.max_depth = max_depth;
        }

        if let Some(max_solutions) = Self::extract_max_solutions(input) {
            query.max_solutions = max_solutions;
        }

        if let Some(enable_memo) = Self::extract_memoization(input) {
            query.enable_memoization = enable_memo;
        }

        if let Some(enable_opt) = Self::extract_optimization(input) {
            query.enable_optimization = enable_opt;
        }

        // Parse actions
        if let Some(action) = Self::extract_on_success(input)? {
            query.on_success = Some(action);
        }

        if let Some(action) = Self::extract_on_failure(input)? {
            query.on_failure = Some(action);
        }

        if let Some(action) = Self::extract_on_missing(input)? {
            query.on_missing = Some(action);
        }

        // Parse when condition
        if let Some(condition) = Self::extract_when_condition(input)? {
            query.when_condition = Some(condition);
        }

        Ok(query)
    }

    fn extract_query_name(input: &str) -> Result<String, RuleEngineError> {
        let re = regex::Regex::new(r#"query\s+"([^"]+)"\s*\{"#).unwrap();
        if let Some(caps) = re.captures(input) {
            Ok(caps[1].to_string())
        } else {
            Err(RuleEngineError::ParseError {
                message: "Invalid query syntax: missing query name".to_string(),
            })
        }
    }

    fn extract_goal(input: &str) -> Result<String, RuleEngineError> {
        // Find goal: line
        if let Some(goal_start) = input.find("goal:") {
            let after_goal = &input[goal_start + 5..]; // Skip "goal:"

            // Find the end of the goal line (newline or end of attributes section)
            // Goal ends at newline, but we need to handle parentheses carefully
            let goal_end = Self::find_goal_end(after_goal)?;
            let goal_str = after_goal[..goal_end].trim().to_string();

            if goal_str.is_empty() {
                return Err(RuleEngineError::ParseError {
                    message: "Invalid query syntax: empty goal".to_string(),
                });
            }

            Ok(goal_str)
        } else {
            Err(RuleEngineError::ParseError {
                message: "Invalid query syntax: missing goal".to_string(),
            })
        }
    }

    fn find_goal_end(input: &str) -> Result<usize, RuleEngineError> {
        let mut paren_depth = 0;
        let mut in_string = false;
        let mut escape_next = false;

        for (i, ch) in input.chars().enumerate() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '(' if !in_string => paren_depth += 1,
                ')' if !in_string => {
                    if paren_depth == 0 {
                        return Err(RuleEngineError::ParseError {
                            message: format!("Parse error: Unexpected closing parenthesis at position {}", i),
                        });
                    }
                    paren_depth -= 1;
                }
                '\n' if !in_string && paren_depth == 0 => return Ok(i),
                _ => {}
            }
        }

        if in_string {
            return Err(RuleEngineError::ParseError {
                message: "Parse error: Unclosed string in goal".to_string(),
            });
        }

        if paren_depth > 0 {
            return Err(RuleEngineError::ParseError {
                message: format!("Parse error: {} unclosed parentheses in goal", paren_depth),
            });
        }

        // If we reach here, goal extends to end of input
        Ok(input.len())
    }

    fn extract_strategy(input: &str) -> Option<GRLSearchStrategy> {
        let re = regex::Regex::new(r"strategy:\s*([a-z-]+)").unwrap();
        re.captures(input).and_then(|caps| {
            match caps[1].trim() {
                "depth-first" => Some(GRLSearchStrategy::DepthFirst),
                "breadth-first" => Some(GRLSearchStrategy::BreadthFirst),
                "iterative" => Some(GRLSearchStrategy::Iterative),
                _ => None,
            }
        })
    }

    fn extract_max_depth(input: &str) -> Option<usize> {
        let re = regex::Regex::new(r"max-depth:\s*(\d+)").unwrap();
        re.captures(input)
            .and_then(|caps| caps[1].parse().ok())
    }

    fn extract_max_solutions(input: &str) -> Option<usize> {
        let re = regex::Regex::new(r"max-solutions:\s*(\d+)").unwrap();
        re.captures(input)
            .and_then(|caps| caps[1].parse().ok())
    }

    fn extract_memoization(input: &str) -> Option<bool> {
        let re = regex::Regex::new(r"enable-memoization:\s*(true|false)").unwrap();
        re.captures(input).and_then(|caps| {
            match caps[1].trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            }
        })
    }

    fn extract_optimization(input: &str) -> Option<bool> {
        let re = regex::Regex::new(r"enable-optimization:\s*(true|false)").unwrap();
        re.captures(input).and_then(|caps| {
            match caps[1].trim() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            }
        })
    }

    fn extract_on_success(input: &str) -> Result<Option<QueryAction>, RuleEngineError> {
        Self::extract_action_block(input, "on-success")
    }

    fn extract_on_failure(input: &str) -> Result<Option<QueryAction>, RuleEngineError> {
        Self::extract_action_block(input, "on-failure")
    }

    fn extract_on_missing(input: &str) -> Result<Option<QueryAction>, RuleEngineError> {
        Self::extract_action_block(input, "on-missing")
    }

    fn extract_action_block(input: &str, action_name: &str) -> Result<Option<QueryAction>, RuleEngineError> {
        let pattern = format!(r"{}:\s*\{{([^}}]+)\}}", action_name);
        let re = regex::Regex::new(&pattern).unwrap();
        
        if let Some(caps) = re.captures(input) {
            let block = caps[1].trim();
            let mut action = QueryAction::new();

            // Parse assignments: Variable = Value
            let assign_re = regex::Regex::new(r"([A-Za-z_][A-Za-z0-9_.]*)\s*=\s*([^;]+);").unwrap();
            for caps in assign_re.captures_iter(block) {
                let var_name = caps[1].trim().to_string();
                let value_str = caps[2].trim().to_string();
                action.assignments.push((var_name, value_str));
            }

            // Parse function calls: Function(...)
            let call_re = regex::Regex::new(r"([A-Za-z_][A-Za-z0-9_]*\([^)]*\));").unwrap();
            for caps in call_re.captures_iter(block) {
                action.calls.push(caps[1].trim().to_string());
            }

            Ok(Some(action))
        } else {
            Ok(None)
        }
    }

    fn extract_when_condition(input: &str) -> Result<Option<String>, RuleEngineError> {
        let re = regex::Regex::new(r"when:\s*([^\n}]+)").unwrap();
        if let Some(caps) = re.captures(input) {
            let condition_str = caps[1].trim().to_string();
            Ok(Some(condition_str))
        } else {
            Ok(None)
        }
    }

    /// Parse multiple queries from a file
    pub fn parse_queries(input: &str) -> Result<Vec<GRLQuery>, RuleEngineError> {
        let mut queries = Vec::new();
        
        // Find all query blocks - use simpler approach
        // Split by "query" keyword and process each block
        let parts: Vec<&str> = input.split("query").collect();
        
        for part in parts.iter().skip(1) { // Skip first empty part
            let query_str = format!("query{}", part);
            // Find the matching closing brace
            if let Some(end_idx) = find_matching_brace(&query_str) {
                let complete_query = &query_str[..end_idx];
                if let Ok(query) = Self::parse(complete_query) {
                    queries.push(query);
                }
            }
        }

        Ok(queries)
    }
}

// Helper function to find matching closing brace
fn find_matching_brace(input: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    
    for (i, ch) in input.chars().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }
        
        match ch {
            '\\' => escape_next = true,
            '"' => in_string = !in_string,
            '{' if !in_string => depth += 1,
            '}' if !in_string => {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
    }
    
    None
}

/// Executor for GRL queries
pub struct GRLQueryExecutor;

impl GRLQueryExecutor {
    /// Execute a single query
    pub fn execute(
        query: &GRLQuery,
        bc_engine: &mut BackwardEngine,
        facts: &mut Facts,
    ) -> Result<QueryResult, RuleEngineError> {
        // Check when condition
        if !query.should_execute(facts)? {
            return Ok(QueryResult {
                provable: false,
                bindings: HashMap::new(),
                proof_trace: ProofTrace { goal: String::new(), steps: Vec::new() },
                missing_facts: Vec::new(),
                stats: QueryStats::default(),
                solutions: Vec::new(),
            });
        }

        // Apply config
        bc_engine.set_config(query.to_config());

        // Parse compound goals (support &&, ||, and !=)
        let result = if query.goal.contains("&&") && query.goal.contains("||") {
            // Complex expression with both AND and OR - need proper parsing
            // For now, evaluate left-to-right with precedence: AND before OR
            Self::execute_complex_goal(&query.goal, bc_engine, facts)?
        } else if query.goal.contains("||") {
            // Split on || and check any goal (OR logic)
            Self::execute_compound_or_goal(&query.goal, bc_engine, facts)?
        } else if query.goal.contains("&&") {
            // Split on && and check all goals (AND logic)
            Self::execute_compound_and_goal(&query.goal, bc_engine, facts)?
        } else {
            // Single goal
            bc_engine.query(&query.goal, facts)?
        };

        // Execute appropriate actions
        if result.provable {
            query.execute_success_actions(facts)?;
        } else if !result.missing_facts.is_empty() {
            query.execute_missing_actions(facts)?;
        } else {
            query.execute_failure_actions(facts)?;
        }

        Ok(result)
    }

    /// Execute compound AND goal (all must be true)
    fn execute_compound_and_goal(
        goal_expr: &str,
        bc_engine: &mut BackwardEngine,
        facts: &mut Facts,
    ) -> Result<QueryResult, RuleEngineError> {
        let sub_goals: Vec<&str> = goal_expr.split("&&").map(|s| s.trim()).collect();

        let mut all_provable = true;
        let mut combined_bindings = HashMap::new();
        let mut all_missing = Vec::new();
        let mut combined_stats = QueryStats::default();

        for (i, sub_goal) in sub_goals.iter().enumerate() {
            // Handle != by using expression parser directly
            let goal_satisfied = if sub_goal.contains("!=") {
                // Parse and evaluate the expression directly
                use crate::backward::expression::ExpressionParser;

                match ExpressionParser::parse(sub_goal) {
                    Ok(expr) => expr.is_satisfied(facts),
                    Err(_) => false,
                }
            } else {
                // Normal == comparison, use backward chaining
                let result = bc_engine.query(sub_goal, facts)?;
                result.provable
            };

            if !goal_satisfied {
                all_provable = false;
            }

            // Note: For compound goals with !=, we don't track missing facts well yet
            // This is a simplification for now
        }

        Ok(QueryResult {
            provable: all_provable,
            bindings: combined_bindings,
            proof_trace: ProofTrace {
                goal: goal_expr.to_string(),
                steps: Vec::new()
            },
            missing_facts: all_missing,
            stats: combined_stats,
            solutions: Vec::new(),
        })
    }

    /// Execute compound OR goal (any must be true)
    fn execute_compound_or_goal(
        goal_expr: &str,
        bc_engine: &mut BackwardEngine,
        facts: &mut Facts,
    ) -> Result<QueryResult, RuleEngineError> {
        let sub_goals: Vec<&str> = goal_expr.split("||").map(|s| s.trim()).collect();

        let mut any_provable = false;
        let mut combined_bindings = HashMap::new();
        let mut all_missing = Vec::new();
        let mut combined_stats = QueryStats::default();
        let mut all_solutions = Vec::new();

        for sub_goal in sub_goals.iter() {
            // Handle != by using expression parser directly
            let (goal_satisfied, result_opt) = if sub_goal.contains("!=") {
                // Parse and evaluate the expression directly
                use crate::backward::expression::ExpressionParser;

                match ExpressionParser::parse(sub_goal) {
                    Ok(expr) => (expr.is_satisfied(facts), None),
                    Err(_) => (false, None),
                }
            } else {
                // Normal == comparison, use backward chaining
                let result = bc_engine.query(sub_goal, facts)?;
                let provable = result.provable;
                (provable, Some(result))
            };

            if goal_satisfied {
                any_provable = true;

                // Merge results from successful branch
                if let Some(result) = result_opt {
                    combined_bindings.extend(result.bindings);
                    all_missing.extend(result.missing_facts);
                    combined_stats.goals_explored += result.stats.goals_explored;
                    combined_stats.rules_evaluated += result.stats.rules_evaluated;
                    if let Some(dur) = result.stats.duration_ms {
                        combined_stats.duration_ms = Some(combined_stats.duration_ms.unwrap_or(0) + dur);
                    }
                    all_solutions.extend(result.solutions);
                }
            }
        }

        Ok(QueryResult {
            provable: any_provable,
            bindings: combined_bindings,
            proof_trace: ProofTrace {
                goal: goal_expr.to_string(),
                steps: Vec::new()
            },
            missing_facts: all_missing,
            stats: combined_stats,
            solutions: all_solutions,
        })
    }

    /// Strip outer parentheses from expression
    fn strip_outer_parens(expr: &str) -> &str {
        let trimmed = expr.trim();
        if trimmed.starts_with('(') && trimmed.ends_with(')') {
            // Check if these are matching outer parens
            let inner = &trimmed[1..trimmed.len()-1];
            let mut depth = 0;
            for ch in inner.chars() {
                match ch {
                    '(' => depth += 1,
                    ')' => {
                        depth -= 1;
                        if depth < 0 {
                            // Closing paren in middle, so outer parens don't match
                            return trimmed;
                        }
                    }
                    _ => {}
                }
            }
            if depth == 0 {
                // Outer parens match, return inner
                return inner.trim();
            }
        }
        trimmed
    }

    /// Execute complex goal with both AND and OR operators
    /// Precedence: AND is evaluated before OR (like multiplication before addition)
    /// Example: "A || B && C" is evaluated as "A || (B && C)"
    fn execute_complex_goal(
        goal_expr: &str,
        bc_engine: &mut BackwardEngine,
        facts: &mut Facts,
    ) -> Result<QueryResult, RuleEngineError> {
        // Strip outer parentheses first
        let cleaned_expr = Self::strip_outer_parens(goal_expr);

        // Split by || first (lowest precedence)
        let or_parts: Vec<&str> = cleaned_expr.split("||").map(|s| s.trim()).collect();

        let mut any_provable = false;
        let mut combined_bindings = HashMap::new();
        let mut all_missing = Vec::new();
        let mut combined_stats = QueryStats::default();
        let mut all_solutions = Vec::new();

        for or_part in or_parts.iter() {
            // Strip parentheses from each part
            let cleaned_part = Self::strip_outer_parens(or_part);

            // Each OR part might contain AND clauses
            let result = if cleaned_part.contains("&&") {
                Self::execute_compound_and_goal(cleaned_part, bc_engine, facts)?
            } else {
                bc_engine.query(cleaned_part, facts)?
            };

            if result.provable {
                any_provable = true;
                combined_bindings.extend(result.bindings);
                all_missing.extend(result.missing_facts);
                combined_stats.goals_explored += result.stats.goals_explored;
                combined_stats.rules_evaluated += result.stats.rules_evaluated;
                if let Some(dur) = result.stats.duration_ms {
                    combined_stats.duration_ms = Some(combined_stats.duration_ms.unwrap_or(0) + dur);
                }
                all_solutions.extend(result.solutions);
            }
        }

        Ok(QueryResult {
            provable: any_provable,
            bindings: combined_bindings,
            proof_trace: ProofTrace {
                goal: goal_expr.to_string(),
                steps: Vec::new()
            },
            missing_facts: all_missing,
            stats: combined_stats,
            solutions: all_solutions,
        })
    }

    /// Execute multiple queries
    pub fn execute_queries(
        queries: &[GRLQuery],
        bc_engine: &mut BackwardEngine,
        facts: &mut Facts,
    ) -> Result<Vec<QueryResult>, RuleEngineError> {
        let mut results = Vec::new();

        for query in queries {
            let result = Self::execute(query, bc_engine, facts)?;
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_query() {
        let input = r#"
        query "TestQuery" {
            goal: User.IsVIP == true
        }
        "#;

        let query = GRLQueryParser::parse(input).unwrap();
        assert_eq!(query.name, "TestQuery");
        assert_eq!(query.strategy, GRLSearchStrategy::DepthFirst);
        assert_eq!(query.max_depth, 10);
    }

    #[test]
    fn test_parse_query_with_strategy() {
        let input = r#"
        query "TestQuery" {
            goal: User.IsVIP == true
            strategy: breadth-first
            max-depth: 5
        }
        "#;

        let query = GRLQueryParser::parse(input).unwrap();
        assert_eq!(query.strategy, GRLSearchStrategy::BreadthFirst);
        assert_eq!(query.max_depth, 5);
    }

    #[test]
    fn test_parse_query_with_actions() {
        let input = r#"
        query "TestQuery" {
            goal: User.IsVIP == true
            on-success: {
                User.DiscountRate = 0.2;
                LogMessage("VIP confirmed");
            }
        }
        "#;

        let query = GRLQueryParser::parse(input).unwrap();
        assert!(query.on_success.is_some());
        
        let action = query.on_success.unwrap();
        assert_eq!(action.assignments.len(), 1);
        assert_eq!(action.calls.len(), 1);
    }

    #[test]
    fn test_parse_query_with_when_condition() {
        let input = r#"
        query "TestQuery" {
            goal: User.IsVIP == true
            when: Environment.Mode == "Production"
        }
        "#;

        let query = GRLQueryParser::parse(input).unwrap();
        assert!(query.when_condition.is_some());
    }

    #[test]
    fn test_parse_multiple_queries() {
        let input = r#"
        query "Query1" {
            goal: A == true
        }
        
        query "Query2" {
            goal: B == true
            strategy: breadth-first
        }
        "#;

        let queries = GRLQueryParser::parse_queries(input).unwrap();
        assert_eq!(queries.len(), 2);
        assert_eq!(queries[0].name, "Query1");
        assert_eq!(queries[1].name, "Query2");
    }

    #[test]
    fn test_query_config_conversion() {
        let query = GRLQuery::new(
            "Test".to_string(),
            "X == true".to_string(),
        )
        .with_strategy(GRLSearchStrategy::BreadthFirst)
        .with_max_depth(15)
        .with_memoization(false);

        let config = query.to_config();
        assert_eq!(config.max_depth, 15);
        assert_eq!(config.enable_memoization, false);
    }

    #[test]
    fn test_action_execution() {
        let mut facts = Facts::new();

        let mut action = QueryAction::new();
        action.assignments.push((
            "User.DiscountRate".to_string(),
            "0.2".to_string(),
        ));

        action.execute(&mut facts).unwrap();

        // Check that assignment was executed
        let value = facts.get("User.DiscountRate");
        assert!(value.is_some());
    }

    #[test]
    fn test_should_execute_no_condition() {
        let query = GRLQuery::new("Q".to_string(), "X == true".to_string());
        let facts = Facts::new();
        // No when condition -> should execute
        let res = query.should_execute(&facts).unwrap();
        assert!(res);
    }

    #[test]
    fn test_should_execute_condition_true() {
        let mut facts = Facts::new();
        facts.set("Environment.Mode", Value::String("Production".to_string()));

        let query = GRLQuery::new("Q".to_string(), "X == true".to_string())
            .with_when("Environment.Mode == \"Production\"".to_string());

        let res = query.should_execute(&facts).unwrap();
        assert!(res, "expected when condition to be satisfied");
    }

    #[test]
    fn test_should_execute_condition_false() {
        let mut facts = Facts::new();
        facts.set("Environment.Mode", Value::String("Development".to_string()));

        let query = GRLQuery::new("Q".to_string(), "X == true".to_string())
            .with_when("Environment.Mode == \"Production\"".to_string());

        let res = query.should_execute(&facts).unwrap();
        assert!(!res, "expected when condition to be unsatisfied");
    }

    #[test]
    fn test_should_execute_parse_error_propagates() {
        let facts = Facts::new();
        // Use an unterminated string literal to force a parse error from the expression parser
        let query = GRLQuery::new("Q".to_string(), "X == true".to_string())
            .with_when("Environment.Mode == \"Production".to_string());

        let res = query.should_execute(&facts);
        assert!(res.is_err(), "expected parse error to propagate");
    }

    #[test]
    fn test_parse_query_with_or_goal() {
        let input = r#"
        query "TestOR" {
            goal: User.IsVIP == true || User.TotalSpent > 10000
        }
        "#;

        let query = GRLQueryParser::parse(input).unwrap();
        assert_eq!(query.name, "TestOR");
        assert!(query.goal.contains("||"));
    }

    #[test]
    fn test_parse_query_with_complex_goal() {
        let input = r#"
        query "ComplexQuery" {
            goal: (User.IsVIP == true && User.Active == true) || User.TotalSpent > 10000
        }
        "#;

        let query = GRLQueryParser::parse(input).unwrap();
        assert!(query.goal.contains("||"));
        assert!(query.goal.contains("&&"));
    }

    #[test]
    fn test_parse_query_with_multiple_or_branches() {
        let input = r#"
        query "MultiOR" {
            goal: Employee.IsManager == true || Employee.IsSenior == true || Employee.IsDirector == true
        }
        "#;

        let query = GRLQueryParser::parse(input).unwrap();
        let branches: Vec<&str> = query.goal.split("||").collect();
        assert_eq!(branches.len(), 3);
    }

    #[test]
    fn test_parse_query_with_parentheses() {
        let input = r#"
        query "ParenQuery" {
            goal: (User.IsVIP == true && User.Active == true) || User.TotalSpent > 10000
        }
        "#;

        let query = GRLQueryParser::parse(input).unwrap();
        assert!(query.goal.contains("("));
        assert!(query.goal.contains(")"));
        assert!(query.goal.contains("||"));
        assert!(query.goal.contains("&&"));
    }

    #[test]
    fn test_parse_query_with_nested_parentheses() {
        let input = r#"
        query "NestedParen" {
            goal: ((A == true && B == true) || C == true) && D == true
        }
        "#;

        let query = GRLQueryParser::parse(input).unwrap();
        assert_eq!(query.name, "NestedParen");
        // Check that parentheses are preserved
        assert!(query.goal.starts_with("(("));
    }

    #[test]
    fn test_parse_query_unclosed_parenthesis() {
        let input = r#"
        query "BadParen" {
            goal: (User.IsVIP == true && User.Active == true
        }
        "#;

        let result = GRLQueryParser::parse(input);
        assert!(result.is_err());
        if let Err(e) = result {
            let msg = format!("{:?}", e);
            assert!(msg.contains("unclosed parentheses") || msg.contains("parenthesis"));
        }
    }
}

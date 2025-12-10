//! Nested query support for backward chaining
//!
//! This module implements nested/subqueries in backward chaining queries.
//! Subqueries allow using the results of one query as input to another query.
//!
//! # Examples
//!
//! ```rust,ignore
//! // Find grandparents: people who are parents of parents
//! let results = engine.query(
//!     "grandparent(?x, ?z) WHERE
//!         parent(?x, ?y) AND
//!         (parent(?y, ?z) WHERE child(?z, ?y))",
//!     &mut facts
//! )?;
//!
//! // Find high-value customers: VIPs or those with large purchases
//! let results = engine.query(
//!     "high_value(?customer) WHERE
//!         (vip(?customer) OR
//!          (purchase(?customer, ?item, ?amount) WHERE ?amount > 1000))",
//!     &mut facts
//! )?;
//! ```

use super::goal::Goal;
use super::unification::Bindings;

/// Represents a nested query (subquery) within a larger query
#[derive(Debug, Clone)]
pub struct NestedQuery {
    /// The outer goal that depends on the subquery
    pub outer_goal: Goal,

    /// The inner subquery to evaluate first
    pub subquery: Box<Query>,

    /// Variables shared between outer and inner queries
    pub shared_variables: Vec<String>,
}

/// Represents a query that may contain nested subqueries
#[derive(Debug, Clone)]
pub struct Query {
    /// Main goals to evaluate
    pub goals: Vec<Goal>,

    /// Any nested subqueries
    pub nested: Vec<NestedQuery>,

    /// Original query string
    pub pattern: String,
}

impl Query {
    /// Create a new query
    pub fn new(pattern: String) -> Self {
        Self {
            goals: Vec::new(),
            nested: Vec::new(),
            pattern,
        }
    }

    /// Add a goal to this query
    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.push(goal);
    }

    /// Add a nested subquery
    pub fn add_nested(&mut self, nested: NestedQuery) {
        self.nested.push(nested);
    }

    /// Check if this query has nested subqueries
    pub fn has_nested(&self) -> bool {
        !self.nested.is_empty()
    }

    /// Get all variables used in this query
    pub fn variables(&self) -> Vec<String> {
        let mut vars = Vec::new();

        for goal in &self.goals {
            // Extract variables from pattern (simple heuristic: look for ?var)
            let pattern_vars = Self::extract_variables(&goal.pattern);
            for var in pattern_vars {
                if !vars.contains(&var) {
                    vars.push(var);
                }
            }
        }

        vars
    }

    /// Extract variables from a pattern string (simple regex-like extraction)
    fn extract_variables(pattern: &str) -> Vec<String> {
        let mut vars = Vec::new();
        let chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '?' {
                // Found a variable, extract until non-alphanumeric
                let mut var = String::from("?");
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    var.push(chars[i]);
                    i += 1;
                }
                if !vars.contains(&var) {
                    vars.push(var);
                }
            } else {
                i += 1;
            }
        }

        vars
    }
}

impl NestedQuery {
    /// Create a new nested query
    pub fn new(outer_goal: Goal, subquery: Query) -> Self {
        // Find shared variables between outer goal and subquery
        let outer_vars = Query::extract_variables(&outer_goal.pattern);
        let subquery_vars = subquery.variables();

        let shared_variables: Vec<String> = outer_vars
            .iter()
            .filter(|v| subquery_vars.contains(v))
            .cloned()
            .collect();

        Self {
            outer_goal,
            subquery: Box::new(subquery),
            shared_variables,
        }
    }

    /// Check if this nested query shares variables with its parent
    pub fn has_shared_variables(&self) -> bool {
        !self.shared_variables.is_empty()
    }
}

/// Result of evaluating a nested query
#[derive(Debug, Clone)]
pub struct NestedQueryResult {
    /// Solutions from the subquery
    pub subquery_solutions: Vec<Bindings>,

    /// Final solutions after applying to outer goal
    pub final_solutions: Vec<Bindings>,

    /// Whether the nested query succeeded
    pub success: bool,

    /// Number of subquery evaluations
    pub subquery_count: usize,
}

impl NestedQueryResult {
    /// Create a new result
    pub fn new() -> Self {
        Self {
            subquery_solutions: Vec::new(),
            final_solutions: Vec::new(),
            success: false,
            subquery_count: 0,
        }
    }

    /// Create a successful result
    pub fn success(subquery_solutions: Vec<Bindings>, final_solutions: Vec<Bindings>) -> Self {
        Self {
            success: !final_solutions.is_empty(),
            subquery_count: subquery_solutions.len(),
            subquery_solutions,
            final_solutions,
        }
    }

    /// Create a failed result
    pub fn failure() -> Self {
        Self {
            subquery_solutions: Vec::new(),
            final_solutions: Vec::new(),
            success: false,
            subquery_count: 0,
        }
    }
}

impl Default for NestedQueryResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Parser for nested queries
pub struct NestedQueryParser;

impl NestedQueryParser {
    /// Parse a query that might contain nested subqueries
    ///
    /// Syntax: "outer_goal WHERE (subquery WHERE conditions)"
    ///
    /// Examples:
    /// - "grandparent(?x, ?z) WHERE parent(?x, ?y) AND (parent(?y, ?z) WHERE child(?z, ?y))"
    /// - "eligible(?p) WHERE (manager(?p) WHERE senior(?p))"
    pub fn parse(query_str: &str) -> Query {
        let mut query = Query::new(query_str.to_string());

        // Simple parsing - look for WHERE clauses with nested parentheses
        // This is a simplified parser - production would use a proper parser

        if let Some(where_idx) = query_str.find(" WHERE ") {
            let conditions = &query_str[where_idx + 7..].trim();

            // Parse conditions (simplified - doesn't handle complex nesting)
            let goals = Self::parse_conditions(conditions);
            for goal in goals {
                query.add_goal(goal);
            }
        }

        query
    }

    /// Parse conditions into goals
    fn parse_conditions(conditions: &str) -> Vec<Goal> {
        let mut goals = Vec::new();

        // Split by AND (simplified - doesn't handle parentheses properly)
        for condition in conditions.split(" AND ") {
            let condition = condition.trim();

            // Check if this is a nested query (contains WHERE)
            if condition.contains(" WHERE ") {
                // Skip nested queries for now - would need recursive parsing
                continue;
            }

            // Create a simple goal
            if !condition.is_empty() && !condition.starts_with('(') {
                goals.push(Goal::new(condition.to_string()));
            }
        }

        goals
    }

    /// Check if a query contains nested subqueries
    pub fn has_nested(query_str: &str) -> bool {
        // Look for WHERE inside parentheses - indicates nested query
        let mut paren_depth = 0;
        let mut in_parens = false;

        let chars: Vec<char> = query_str.chars().collect();
        for i in 0..chars.len() {
            match chars[i] {
                '(' => {
                    paren_depth += 1;
                    in_parens = true;
                }
                ')' => {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        in_parens = false;
                    }
                }
                'W' if in_parens && paren_depth > 0 => {
                    // Check if this starts "WHERE"
                    if i + 5 < chars.len() {
                        let substr: String = chars[i..i+5].iter().collect();
                        if substr == "WHERE" {
                            return true;
                        }
                    }
                }
                _ => {}
            }
        }

        false
    }
}

/// Evaluator for nested queries
pub struct NestedQueryEvaluator;

impl NestedQueryEvaluator {
    /// Evaluate a nested query
    ///
    /// Algorithm:
    /// 1. Evaluate the subquery to get intermediate results
    /// 2. For each subquery result, bind variables in outer goal
    /// 3. Evaluate outer goal with those bindings
    /// 4. Collect and merge all solutions
    pub fn evaluate(
        nested: &NestedQuery,
        initial_bindings: &Bindings,
    ) -> NestedQueryResult {
        // This is a placeholder - actual implementation would recursively
        // call the backward chaining engine

        // For now, return empty result
        NestedQueryResult::new()
    }

    /// Merge bindings from subquery into outer goal
    fn merge_bindings(
        outer_bindings: &Bindings,
        subquery_bindings: &Bindings,
        shared_vars: &[String],
    ) -> Bindings {
        let mut merged = outer_bindings.clone();

        // Add bindings from subquery for shared variables
        for var in shared_vars {
            if let Some(value) = subquery_bindings.get(var) {
                // Use bind method (which returns Result)
                let _ = merged.bind(var.clone(), value.clone());
            }
        }

        merged
    }
}

/// Statistics for nested query evaluation
#[derive(Debug, Clone, Default)]
pub struct NestedQueryStats {
    /// Total number of nested queries evaluated
    pub total_nested: usize,

    /// Total number of subquery evaluations
    pub total_subquery_evals: usize,

    /// Maximum nesting depth encountered
    pub max_depth: usize,

    /// Average number of solutions per subquery
    pub avg_subquery_solutions: f64,
}

impl NestedQueryStats {
    /// Create new stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a nested query evaluation
    pub fn record_evaluation(&mut self, depth: usize, subquery_solutions: usize) {
        self.total_nested += 1;
        self.total_subquery_evals += subquery_solutions;
        self.max_depth = self.max_depth.max(depth);

        // Update average
        self.avg_subquery_solutions =
            self.total_subquery_evals as f64 / self.total_nested as f64;
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        format!(
            "Nested queries: {} | Subquery evals: {} | Max depth: {} | Avg solutions: {:.2}",
            self.total_nested,
            self.total_subquery_evals,
            self.max_depth,
            self.avg_subquery_solutions
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_creation() {
        let query = Query::new("test(?x)".to_string());
        assert_eq!(query.pattern, "test(?x)");
        assert!(query.goals.is_empty());
        assert!(!query.has_nested());
    }

    #[test]
    fn test_query_add_goal() {
        let mut query = Query::new("test".to_string());
        let goal = Goal::new("parent(?x, ?y)".to_string());
        query.add_goal(goal);

        assert_eq!(query.goals.len(), 1);
    }

    #[test]
    fn test_query_variables() {
        let mut query = Query::new("test".to_string());

        let goal1 = Goal::new("parent(?x, ?y)".to_string());
        let goal2 = Goal::new("age(?x, ?age)".to_string());

        query.add_goal(goal1);
        query.add_goal(goal2);

        let vars = query.variables();
        assert!(vars.contains(&"?x".to_string()));
        assert!(vars.contains(&"?y".to_string()));
        assert!(vars.contains(&"?age".to_string()));
    }

    #[test]
    fn test_nested_query_creation() {
        let outer_goal = Goal::new("grandparent(?x, ?z)".to_string());
        let mut subquery = Query::new("parent(?y, ?z)".to_string());

        let sub_goal = Goal::new("parent(?y, ?z)".to_string());
        subquery.add_goal(sub_goal);

        let nested = NestedQuery::new(outer_goal, subquery);

        assert!(!nested.subquery.goals.is_empty());
    }

    #[test]
    fn test_nested_query_shared_variables() {
        let outer_goal = Goal::new("grandparent(?x, ?z)".to_string());

        let mut subquery = Query::new("parent(?y, ?z)".to_string());
        let sub_goal = Goal::new("parent(?y, ?z)".to_string());
        subquery.add_goal(sub_goal);

        let nested = NestedQuery::new(outer_goal, subquery);

        // ?z is shared between outer and subquery
        assert!(nested.has_shared_variables());
        assert!(nested.shared_variables.contains(&"?z".to_string()));
    }

    #[test]
    fn test_nested_query_result() {
        let result = NestedQueryResult::new();
        assert!(!result.success);
        assert_eq!(result.subquery_count, 0);

        let success = NestedQueryResult::success(vec![], vec![]);
        assert!(!success.success); // Empty solutions = not successful
    }

    #[test]
    fn test_nested_query_parser_has_nested() {
        assert!(NestedQueryParser::has_nested(
            "grandparent(?x, ?z) WHERE parent(?x, ?y) AND (parent(?y, ?z) WHERE child(?z, ?y))"
        ));

        assert!(!NestedQueryParser::has_nested(
            "parent(?x, ?y) WHERE person(?x) AND person(?y)"
        ));
    }

    #[test]
    fn test_nested_query_stats() {
        let mut stats = NestedQueryStats::new();

        stats.record_evaluation(1, 5);
        stats.record_evaluation(2, 10);

        assert_eq!(stats.total_nested, 2);
        assert_eq!(stats.total_subquery_evals, 15);
        assert_eq!(stats.max_depth, 2);
        assert_eq!(stats.avg_subquery_solutions, 7.5);
    }

    #[test]
    fn test_parser_simple_query() {
        let query = NestedQueryParser::parse("parent(?x, ?y) WHERE person(?x) AND person(?y)");
        assert!(!query.goals.is_empty());
    }
}

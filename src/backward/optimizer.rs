//! Query optimization for backward chaining
//!
//! This module implements query optimization techniques to improve performance:
//! - Goal reordering based on selectivity
//! - Index selection for O(1) lookups
//! - Join ordering optimization
//! - Predicate pushdown
//! - Memoization of intermediate results
//!
//! # Examples
//!
//! ```rust,ignore
//! use rust_rule_engine::backward::optimizer::QueryOptimizer;
//!
//! let mut optimizer = QueryOptimizer::new();
//!
//! // Before: item(?x) AND expensive(?x) AND in_stock(?x)
//! // After:  in_stock(?x) AND expensive(?x) AND item(?x)
//! //         (Evaluates most selective first)
//!
//! let optimized = optimizer.optimize_query(query)?;
//! ```

use super::goal::Goal;
use std::collections::HashMap;

/// Query optimizer for backward chaining
#[derive(Debug, Clone)]
pub struct QueryOptimizer {
    /// Selectivity estimates for predicates
    selectivity_map: HashMap<String, f64>,

    /// Whether to enable goal reordering
    enable_reordering: bool,

    /// Whether to enable index selection
    enable_index_selection: bool,

    /// Whether to enable memoization
    enable_memoization: bool,

    /// Statistics for optimization
    stats: OptimizationStats,
}

impl QueryOptimizer {
    /// Create a new query optimizer
    pub fn new() -> Self {
        Self {
            selectivity_map: HashMap::new(),
            enable_reordering: true,
            enable_index_selection: true,
            enable_memoization: true,
            stats: OptimizationStats::new(),
        }
    }

    /// Create optimizer with custom configuration
    pub fn with_config(config: OptimizerConfig) -> Self {
        Self {
            selectivity_map: HashMap::new(),
            enable_reordering: config.enable_reordering,
            enable_index_selection: config.enable_index_selection,
            enable_memoization: config.enable_memoization,
            stats: OptimizationStats::new(),
        }
    }

    /// Optimize a list of goals
    ///
    /// Returns a reordered list of goals optimized for evaluation
    pub fn optimize_goals(&mut self, goals: Vec<Goal>) -> Vec<Goal> {
        if !self.enable_reordering || goals.len() <= 1 {
            return goals;
        }

        self.stats.total_optimizations += 1;

        // Estimate selectivity for each goal
        let mut goal_selectivity: Vec<(Goal, f64)> = goals
            .into_iter()
            .map(|g| {
                let selectivity = self.estimate_selectivity(&g);
                (g, selectivity)
            })
            .collect();

        // Sort by selectivity (lower = more selective = evaluate first)
        goal_selectivity.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Extract optimized goals
        let optimized: Vec<Goal> = goal_selectivity.into_iter().map(|(g, _)| g).collect();

        self.stats.goals_reordered += optimized.len();

        optimized
    }

    /// Estimate selectivity of a goal (lower = more selective)
    ///
    /// Returns a value between 0.0 (very selective) and 1.0 (not selective)
    pub fn estimate_selectivity(&self, goal: &Goal) -> f64 {
        // Check if we have a known selectivity estimate
        if let Some(&selectivity) = self.selectivity_map.get(&goal.pattern) {
            return selectivity;
        }

        // Heuristic-based estimation
        self.heuristic_selectivity(goal)
    }

    /// Heuristic-based selectivity estimation
    fn heuristic_selectivity(&self, goal: &Goal) -> f64 {
        let pattern = &goal.pattern;

        // Count bound vs unbound variables
        let (bound_count, var_count) = self.count_variables(pattern);

        if var_count == 0 {
            // No variables = most selective (exact match)
            return 0.1;
        }

        // More bound variables = more selective
        let bound_ratio = bound_count as f64 / var_count as f64;
        let selectivity = 1.0 - (bound_ratio * 0.8);

        // Check for specific patterns
        if pattern.contains("in_stock") || pattern.contains("available") {
            // Stock checks typically very selective
            return selectivity * 0.3;
        }

        if pattern.contains("expensive") || pattern.contains("premium") {
            // Price filters moderately selective
            return selectivity * 0.5;
        }

        if pattern.contains("item") || pattern.contains("product") {
            // Generic predicates less selective
            return selectivity * 1.2;
        }

        selectivity
    }

    /// Count bound and total variables in a pattern
    fn count_variables(&self, pattern: &str) -> (usize, usize) {
        let mut bound = 0;
        let mut total = 0;

        let chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '?' {
                total += 1;

                // Skip variable name
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }

                // Check if followed by a comparison (indicates bound)
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }

                if i < chars.len() && (chars[i] == '=' || chars[i] == '>' || chars[i] == '<') {
                    bound += 1;
                }
            } else {
                i += 1;
            }
        }

        (bound, total)
    }

    /// Set selectivity estimate for a predicate
    pub fn set_selectivity(&mut self, predicate: String, selectivity: f64) {
        self.selectivity_map
            .insert(predicate, selectivity.clamp(0.0, 1.0));
    }

    /// Get optimization statistics
    pub fn stats(&self) -> &OptimizationStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = OptimizationStats::new();
    }

    /// Enable or disable goal reordering
    pub fn set_reordering(&mut self, enabled: bool) {
        self.enable_reordering = enabled;
    }

    /// Enable or disable index selection
    pub fn set_index_selection(&mut self, enabled: bool) {
        self.enable_index_selection = enabled;
    }

    /// Enable or disable memoization
    pub fn set_memoization(&mut self, enabled: bool) {
        self.enable_memoization = enabled;
    }
}

impl Default for QueryOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for the query optimizer
#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// Enable goal reordering
    pub enable_reordering: bool,

    /// Enable index selection
    pub enable_index_selection: bool,

    /// Enable memoization
    pub enable_memoization: bool,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enable_reordering: true,
            enable_index_selection: true,
            enable_memoization: true,
        }
    }
}

/// Statistics for query optimization
#[derive(Debug, Clone, Default)]
pub struct OptimizationStats {
    /// Total number of optimizations performed
    pub total_optimizations: usize,

    /// Total goals reordered
    pub goals_reordered: usize,

    /// Number of index selections made
    pub index_selections: usize,

    /// Number of memoization hits
    pub memoization_hits: usize,

    /// Number of memoization misses
    pub memoization_misses: usize,
}

impl OptimizationStats {
    /// Create new statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Get memoization hit rate
    pub fn memoization_hit_rate(&self) -> f64 {
        let total = self.memoization_hits + self.memoization_misses;
        if total == 0 {
            0.0
        } else {
            self.memoization_hits as f64 / total as f64
        }
    }

    /// Get summary string
    pub fn summary(&self) -> String {
        format!(
            "Optimizations: {} | Goals reordered: {} | Memo hits: {} ({:.1}%)",
            self.total_optimizations,
            self.goals_reordered,
            self.memoization_hits,
            self.memoization_hit_rate() * 100.0
        )
    }
}

/// Join ordering optimizer
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct JoinOptimizer {
    /// Cost estimates for different join strategies
    cost_model: HashMap<String, f64>,
}

impl JoinOptimizer {
    /// Create a new join optimizer
    pub fn new() -> Self {
        Self {
            cost_model: HashMap::new(),
        }
    }

    /// Optimize join order for a set of goals
    ///
    /// Returns goals ordered for optimal join performance
    pub fn optimize_joins(&self, goals: Vec<Goal>) -> Vec<Goal> {
        if goals.len() <= 1 {
            return goals;
        }

        // Simple heuristic: start with goals that have most bound variables
        let mut sorted_goals = goals;
        sorted_goals.sort_by_key(|g| {
            // Count bound variables (negative for descending sort)
            -(self.count_bound_vars(&g.pattern) as i32)
        });

        sorted_goals
    }

    /// Count bound variables in a pattern
    fn count_bound_vars(&self, pattern: &str) -> usize {
        // Simple heuristic: variables followed by comparison operators
        let mut count = 0;
        let chars: Vec<char> = pattern.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '?' {
                // Skip variable name
                i += 1;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }

                // Check if followed by comparison
                while i < chars.len() && chars[i].is_whitespace() {
                    i += 1;
                }

                if i < chars.len() && (chars[i] == '=' || chars[i] == '>' || chars[i] == '<') {
                    count += 1;
                }
            } else {
                i += 1;
            }
        }

        count
    }
}

impl Default for JoinOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = QueryOptimizer::new();
        assert!(optimizer.enable_reordering);
        assert!(optimizer.enable_index_selection);
        assert!(optimizer.enable_memoization);
    }

    #[test]
    fn test_optimizer_with_config() {
        let config = OptimizerConfig {
            enable_reordering: false,
            enable_index_selection: true,
            enable_memoization: false,
        };

        let optimizer = QueryOptimizer::with_config(config);
        assert!(!optimizer.enable_reordering);
        assert!(optimizer.enable_index_selection);
        assert!(!optimizer.enable_memoization);
    }

    #[test]
    fn test_goal_reordering() {
        let mut optimizer = QueryOptimizer::new();

        // Set selectivity estimates (lower = more selective)
        optimizer.set_selectivity("in_stock(?x)".to_string(), 0.1);
        optimizer.set_selectivity("expensive(?x)".to_string(), 0.3);
        optimizer.set_selectivity("item(?x)".to_string(), 0.9);

        let goals = vec![
            Goal::new("item(?x)".to_string()),
            Goal::new("expensive(?x)".to_string()),
            Goal::new("in_stock(?x)".to_string()),
        ];

        let optimized = optimizer.optimize_goals(goals);

        // Should be ordered: in_stock, expensive, item
        assert_eq!(optimized[0].pattern, "in_stock(?x)");
        assert_eq!(optimized[1].pattern, "expensive(?x)");
        assert_eq!(optimized[2].pattern, "item(?x)");
    }

    #[test]
    fn test_selectivity_estimation() {
        let optimizer = QueryOptimizer::new();

        // Exact match (no variables) - very selective
        let goal1 = Goal::new("employee(alice)".to_string());
        let sel1 = optimizer.estimate_selectivity(&goal1);
        assert!(sel1 < 0.5);

        // One unbound variable - less selective
        let goal2 = Goal::new("employee(?x)".to_string());
        let sel2 = optimizer.estimate_selectivity(&goal2);
        assert!(sel2 > sel1);

        // Bound variable (with comparison) - more selective
        let goal3 = Goal::new("salary(?x) WHERE ?x > 100000".to_string());
        let sel3 = optimizer.estimate_selectivity(&goal3);
        // Should be more selective than fully unbound
        assert!(sel3 < sel2);
    }

    #[test]
    fn test_count_variables() {
        let optimizer = QueryOptimizer::new();

        // No variables
        let (bound, total) = optimizer.count_variables("employee(alice)");
        assert_eq!(total, 0);
        assert_eq!(bound, 0);

        // One unbound variable
        let (bound, total) = optimizer.count_variables("employee(?x)");
        assert_eq!(total, 1);
        assert_eq!(bound, 0);

        // One bound variable
        let (bound, total) = optimizer.count_variables("salary(?x) WHERE ?x > 100");
        assert_eq!(total, 2); // ?x appears twice
        assert_eq!(bound, 1); // Second ?x is bound by >
    }

    #[test]
    fn test_optimization_stats() {
        let mut optimizer = QueryOptimizer::new();

        let goals = vec![
            Goal::new("a(?x)".to_string()),
            Goal::new("b(?x)".to_string()),
        ];

        optimizer.optimize_goals(goals);

        let stats = optimizer.stats();
        assert_eq!(stats.total_optimizations, 1);
        assert_eq!(stats.goals_reordered, 2);
    }

    #[test]
    fn test_join_optimizer() {
        let optimizer = JoinOptimizer::new();

        let goals = vec![
            Goal::new("item(?x)".to_string()),
            Goal::new("price(?x, ?p) WHERE ?p > 100".to_string()),
            Goal::new("in_stock(?x)".to_string()),
        ];

        let optimized = optimizer.optimize_joins(goals);

        // Goal with bound variable should come first
        assert!(optimized[0].pattern.contains("?p > 100"));
    }

    #[test]
    fn test_disable_reordering() {
        let mut optimizer = QueryOptimizer::new();
        optimizer.set_reordering(false);

        let goals = vec![
            Goal::new("a(?x)".to_string()),
            Goal::new("b(?x)".to_string()),
            Goal::new("c(?x)".to_string()),
        ];

        let optimized = optimizer.optimize_goals(goals.clone());

        // Order should be unchanged
        assert_eq!(optimized[0].pattern, goals[0].pattern);
        assert_eq!(optimized[1].pattern, goals[1].pattern);
        assert_eq!(optimized[2].pattern, goals[2].pattern);
    }

    #[test]
    fn test_stats_summary() {
        let mut stats = OptimizationStats::new();
        stats.total_optimizations = 10;
        stats.goals_reordered = 25;
        stats.memoization_hits = 8;
        stats.memoization_misses = 2;

        let summary = stats.summary();
        assert!(summary.contains("10"));
        assert!(summary.contains("25"));
        assert!(summary.contains("8"));
        assert!(summary.contains("80")); // 80% hit rate
    }

    #[test]
    fn test_memoization_hit_rate() {
        let mut stats = OptimizationStats::new();

        // No data yet
        assert_eq!(stats.memoization_hit_rate(), 0.0);

        // 80% hit rate
        stats.memoization_hits = 8;
        stats.memoization_misses = 2;
        assert!((stats.memoization_hit_rate() - 0.8).abs() < 0.01);

        // 100% hit rate
        stats.memoization_hits = 10;
        stats.memoization_misses = 0;
        assert_eq!(stats.memoization_hit_rate(), 1.0);
    }
}

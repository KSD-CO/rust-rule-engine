use serde::{Deserialize, Serialize};
/// Advanced analytics and performance monitoring for rule engine
/// This module provides comprehensive metrics collection, analysis,
/// and performance insights for rule execution.
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// Trend direction for performance metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Performance is improving over time
    Improving,
    /// Performance is degrading over time  
    Degrading,
    /// Performance is stable over time
    Stable,
}

/// Individual rule execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetrics {
    /// Name of the rule
    pub rule_name: String,
    /// Total number of times this rule was evaluated
    pub total_evaluations: u64,
    /// Total number of times this rule fired (condition was true)
    pub total_fires: u64,
    /// Total number of successful executions (no errors)
    pub total_successes: u64,
    /// Total number of failed executions (with errors)
    pub total_failures: u64,
    /// Sum of all execution times for averaging
    pub total_execution_time: Duration,
    /// Fastest execution time recorded
    pub min_execution_time: Duration,
    /// Slowest execution time recorded
    pub max_execution_time: Duration,
    /// Estimated memory usage for this rule
    pub estimated_memory_usage: usize,
    /// Last time this rule was executed
    pub last_executed: Option<SystemTime>,
    /// Recent execution times (for trend analysis)
    pub recent_execution_times: Vec<Duration>,
}

impl RuleMetrics {
    /// Create new metrics for a rule
    pub fn new(rule_name: String) -> Self {
        Self {
            rule_name,
            total_evaluations: 0,
            total_fires: 0,
            total_successes: 0,
            total_failures: 0,
            total_execution_time: Duration::ZERO,
            min_execution_time: Duration::MAX,
            max_execution_time: Duration::ZERO,
            estimated_memory_usage: 0,
            last_executed: None,
            recent_execution_times: Vec::new(),
        }
    }

    /// Record a successful rule execution
    pub fn record_execution(&mut self, duration: Duration, fired: bool, memory_usage: usize) {
        self.total_evaluations += 1;
        if fired {
            self.total_fires += 1;
        }
        self.total_successes += 1;
        self.total_execution_time += duration;

        // Update min/max times
        if duration < self.min_execution_time {
            self.min_execution_time = duration;
        }
        if duration > self.max_execution_time {
            self.max_execution_time = duration;
        }

        self.estimated_memory_usage = memory_usage;
        self.last_executed = Some(SystemTime::now());

        // Keep last 100 execution times for trend analysis
        self.recent_execution_times.push(duration);
        if self.recent_execution_times.len() > 100 {
            self.recent_execution_times.remove(0);
        }
    }

    /// Record a failed rule execution
    pub fn record_failure(&mut self, duration: Duration) {
        self.total_evaluations += 1;
        self.total_failures += 1;
        self.total_execution_time += duration;
        self.last_executed = Some(SystemTime::now());
    }

    /// Calculate average execution time
    pub fn avg_execution_time(&self) -> Duration {
        if self.total_evaluations > 0 {
            self.total_execution_time / self.total_evaluations as u32
        } else {
            Duration::ZERO
        }
    }

    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_evaluations > 0 {
            (self.total_successes as f64 / self.total_evaluations as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Calculate fire rate as percentage
    pub fn fire_rate(&self) -> f64 {
        if self.total_evaluations > 0 {
            (self.total_fires as f64 / self.total_evaluations as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if this rule is performing poorly
    pub fn is_problematic(&self) -> bool {
        self.success_rate() < 95.0
            || self.avg_execution_time() > Duration::from_millis(50)
            || self.total_failures > 10
    }
}

/// Configuration for analytics collection
#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    /// Whether to track detailed execution timing
    pub track_execution_time: bool,
    /// Whether to estimate memory usage
    pub track_memory_usage: bool,
    /// Whether to track success/failure rates
    pub track_success_rate: bool,
    /// Sampling rate (0.0 to 1.0) - 1.0 means track everything
    pub sampling_rate: f64,
    /// How long to retain detailed metrics
    pub retention_period: Duration,
    /// Maximum number of recent execution times to keep per rule
    pub max_recent_samples: usize,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            track_execution_time: true,
            track_memory_usage: true,
            track_success_rate: true,
            sampling_rate: 1.0,
            retention_period: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            max_recent_samples: 100,
        }
    }
}

impl AnalyticsConfig {
    /// Production-ready configuration with reasonable sampling
    pub fn production() -> Self {
        Self {
            track_execution_time: true,
            track_memory_usage: false, // Expensive in production
            track_success_rate: true,
            sampling_rate: 0.1, // Sample 10% of executions
            retention_period: Duration::from_secs(24 * 60 * 60), // 1 day
            max_recent_samples: 50,
        }
    }

    /// Development configuration with full tracking
    pub fn development() -> Self {
        Self {
            track_execution_time: true,
            track_memory_usage: true,
            track_success_rate: true,
            sampling_rate: 1.0,                             // Track everything
            retention_period: Duration::from_secs(60 * 60), // 1 hour
            max_recent_samples: 100,
        }
    }
}

/// Execution event for timeline analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    /// When this event occurred
    pub timestamp: SystemTime,
    /// Name of the rule that was executed
    pub rule_name: String,
    /// Whether the rule fired
    pub fired: bool,
    /// Execution time
    pub duration: Duration,
    /// Whether the execution was successful
    pub success: bool,
    /// Error message if execution failed
    pub error: Option<String>,
}

/// Performance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrend {
    /// Rule name
    pub rule_name: String,
    /// Trend direction: Improving, Degrading, Stable
    pub trend: TrendDirection,
    /// Percentage change in performance
    pub change_percentage: f64,
    /// Time period of this trend
    pub period: Duration,
}

/// Main analytics collector and analyzer
#[derive(Debug)]
pub struct RuleAnalytics {
    /// Configuration for analytics collection
    config: AnalyticsConfig,
    /// Metrics for each rule
    rule_metrics: HashMap<String, RuleMetrics>,
    /// Timeline of execution events
    execution_timeline: Vec<ExecutionEvent>,
    /// When analytics collection started
    start_time: SystemTime,
    /// Total number of rule executions tracked
    total_executions: u64,
}

impl RuleAnalytics {
    /// Create new analytics collector
    pub fn new(config: AnalyticsConfig) -> Self {
        Self {
            config,
            rule_metrics: HashMap::new(),
            execution_timeline: Vec::new(),
            start_time: SystemTime::now(),
            total_executions: 0,
        }
    }

    /// Record a rule execution
    pub fn record_execution(
        &mut self,
        rule_name: &str,
        duration: Duration,
        fired: bool,
        success: bool,
        error: Option<String>,
        memory_usage: usize,
    ) {
        // Apply sampling rate
        if !self.should_sample() {
            return;
        }

        self.total_executions += 1;

        // Update rule metrics
        let metrics = self
            .rule_metrics
            .entry(rule_name.to_string())
            .or_insert_with(|| RuleMetrics::new(rule_name.to_string()));

        if success {
            metrics.record_execution(duration, fired, memory_usage);
        } else {
            metrics.record_failure(duration);
        }

        // Add to timeline
        self.execution_timeline.push(ExecutionEvent {
            timestamp: SystemTime::now(),
            rule_name: rule_name.to_string(),
            fired,
            duration,
            success,
            error,
        });

        // Clean up old events
        self.cleanup_old_data();
    }

    /// Get metrics for a specific rule
    pub fn get_rule_metrics(&self, rule_name: &str) -> Option<&RuleMetrics> {
        self.rule_metrics.get(rule_name)
    }

    /// Get all rule metrics
    pub fn get_all_metrics(&self) -> &HashMap<String, RuleMetrics> {
        &self.rule_metrics
    }

    /// Get the slowest rules
    pub fn slowest_rules(&self, limit: usize) -> Vec<&RuleMetrics> {
        let mut rules: Vec<&RuleMetrics> = self.rule_metrics.values().collect();
        rules.sort_by_key(|b| std::cmp::Reverse(b.avg_execution_time()));
        rules.into_iter().take(limit).collect()
    }

    /// Get the most frequently fired rules
    pub fn most_fired_rules(&self, limit: usize) -> Vec<&RuleMetrics> {
        let mut rules: Vec<&RuleMetrics> = self.rule_metrics.values().collect();
        rules.sort_by(|a, b| b.total_fires.cmp(&a.total_fires));
        rules.into_iter().take(limit).collect()
    }

    /// Get problematic rules (low success rate, high execution time, etc.)
    pub fn problematic_rules(&self) -> Vec<&RuleMetrics> {
        self.rule_metrics
            .values()
            .filter(|metrics| metrics.is_problematic())
            .collect()
    }

    /// Calculate overall performance statistics
    pub fn overall_stats(&self) -> OverallStats {
        let total_time: Duration = self
            .rule_metrics
            .values()
            .map(|m| m.total_execution_time)
            .sum();

        let total_evaluations: u64 = self
            .rule_metrics
            .values()
            .map(|m| m.total_evaluations)
            .sum();

        let total_fires: u64 = self.rule_metrics.values().map(|m| m.total_fires).sum();

        let total_successes: u64 = self.rule_metrics.values().map(|m| m.total_successes).sum();

        let avg_execution_time = if total_evaluations > 0 {
            total_time / total_evaluations as u32
        } else {
            Duration::ZERO
        };

        let rules_per_second = if total_time.as_secs_f64() > 0.0 {
            total_evaluations as f64 / total_time.as_secs_f64()
        } else {
            0.0
        };

        let success_rate = if total_evaluations > 0 {
            (total_successes as f64 / total_evaluations as f64) * 100.0
        } else {
            0.0
        };

        OverallStats {
            total_rules: self.rule_metrics.len(),
            total_evaluations,
            total_fires,
            total_successes,
            avg_execution_time,
            rules_per_second,
            success_rate,
            uptime: self.start_time.elapsed().unwrap_or(Duration::ZERO),
        }
    }

    /// Check if we should sample this execution based on sampling rate
    fn should_sample(&self) -> bool {
        if self.config.sampling_rate >= 1.0 {
            return true;
        }

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.total_executions.hash(&mut hasher);
        let hash = hasher.finish();

        (hash as f64 / u64::MAX as f64) < self.config.sampling_rate
    }

    /// Clean up old data based on retention period
    fn cleanup_old_data(&mut self) {
        let cutoff = SystemTime::now()
            .checked_sub(self.config.retention_period)
            .unwrap_or(SystemTime::UNIX_EPOCH);

        // Remove old timeline events
        self.execution_timeline
            .retain(|event| event.timestamp >= cutoff);
    }

    /// Get configuration reference
    pub fn config(&self) -> &AnalyticsConfig {
        &self.config
    }

    /// Get all rule metrics as a map
    pub fn get_all_rule_metrics(&self) -> &HashMap<String, RuleMetrics> {
        &self.rule_metrics
    }

    /// Generate optimization recommendations based on analytics data
    pub fn generate_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        for (rule_name, metrics) in &self.rule_metrics {
            // Check for slow rules
            if metrics.avg_execution_time().as_millis() > 100 {
                recommendations.push(format!(
                    "Consider optimizing '{}' - average execution time is {:.2}ms",
                    rule_name,
                    metrics.avg_execution_time().as_secs_f64() * 1000.0
                ));
            }

            // Check for low success rates
            if metrics.success_rate() < 50.0 && metrics.total_evaluations > 10 {
                recommendations.push(format!(
                    "Rule '{}' has low success rate ({:.1}%) - review conditions",
                    rule_name,
                    metrics.success_rate()
                ));
            }

            // Check for rules that never fire
            if metrics.total_fires == 0 && metrics.total_evaluations > 20 {
                recommendations.push(format!(
                    "Rule '{}' never fires despite {} evaluations - review logic",
                    rule_name, metrics.total_evaluations
                ));
            }
        }

        recommendations
    }

    /// Get recent execution events
    pub fn get_recent_events(&self, limit: usize) -> Vec<&ExecutionEvent> {
        self.execution_timeline.iter().rev().take(limit).collect()
    }

    /// Get overall performance statistics
    pub fn get_overall_stats(&self) -> OverallStats {
        self.overall_stats()
    }
}

/// Overall performance statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallStats {
    /// Total number of unique rules
    pub total_rules: usize,
    /// Total rule evaluations
    pub total_evaluations: u64,
    /// Total rule fires
    pub total_fires: u64,
    /// Total successful executions
    pub total_successes: u64,
    /// Average execution time across all rules
    pub avg_execution_time: Duration,
    /// Rules processed per second
    pub rules_per_second: f64,
    /// Overall success rate percentage
    pub success_rate: f64,
    /// How long analytics has been running
    pub uptime: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_metrics_creation() {
        let metrics = RuleMetrics::new("TestRule".to_string());
        assert_eq!(metrics.rule_name, "TestRule");
        assert_eq!(metrics.total_evaluations, 0);
        assert_eq!(metrics.success_rate(), 0.0);
    }

    #[test]
    fn test_rule_metrics_recording() {
        let mut metrics = RuleMetrics::new("TestRule".to_string());

        // Record successful execution
        metrics.record_execution(Duration::from_millis(10), true, 1024);

        assert_eq!(metrics.total_evaluations, 1);
        assert_eq!(metrics.total_fires, 1);
        assert_eq!(metrics.total_successes, 1);
        assert_eq!(metrics.success_rate(), 100.0);
        assert_eq!(metrics.fire_rate(), 100.0);
    }

    #[test]
    fn test_analytics_config() {
        let config = AnalyticsConfig::production();
        assert!(config.sampling_rate < 1.0);
        assert!(!config.track_memory_usage);

        let dev_config = AnalyticsConfig::development();
        assert_eq!(dev_config.sampling_rate, 1.0);
        assert!(dev_config.track_memory_usage);
    }

    #[test]
    fn test_analytics_recording() {
        let config = AnalyticsConfig::development();
        let mut analytics = RuleAnalytics::new(config);

        analytics.record_execution("TestRule", Duration::from_millis(5), true, true, None, 1024);

        assert_eq!(analytics.total_executions, 1);
        assert!(analytics.get_rule_metrics("TestRule").is_some());
    }
}

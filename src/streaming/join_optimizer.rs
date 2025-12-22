use crate::rete::stream_join_node::{JoinStrategy, JoinType};
use std::time::Duration;

/// Join optimization strategies
#[derive(Debug, Clone)]
pub enum JoinOptimization {
    /// Build smaller stream as the hash table (for hash joins)
    BuildSmaller,
    /// Pre-partition streams by join key
    PrePartition { partition_count: usize },
    /// Use bloom filters to skip non-matching events early
    BloomFilter { false_positive_rate: f64 },
    /// Index the join key for faster lookups
    IndexJoinKey,
    /// Merge overlapping time windows
    MergeWindows,
}

/// Statistics about a stream for optimization decisions
#[derive(Debug, Clone)]
pub struct StreamStats {
    pub stream_name: String,
    pub estimated_event_rate: f64,    // events per second
    pub estimated_cardinality: usize, // unique join key values
    pub average_event_size: usize,    // bytes
}

/// Join plan with optimization recommendations
#[derive(Debug, Clone)]
pub struct OptimizedJoinPlan {
    pub left_stream: String,
    pub right_stream: String,
    pub join_type: JoinType,
    pub join_strategy: JoinStrategy,
    pub optimizations: Vec<JoinOptimization>,
    pub estimated_cost: f64,
    pub explanation: String,
}

/// Join optimizer - analyzes stream characteristics and suggests optimizations
pub struct JoinOptimizer {
    /// Statistics for known streams
    stream_stats: Vec<StreamStats>,
}

impl JoinOptimizer {
    /// Create a new join optimizer
    pub fn new() -> Self {
        Self {
            stream_stats: Vec::new(),
        }
    }

    /// Register stream statistics for optimization
    pub fn register_stream_stats(&mut self, stats: StreamStats) {
        // Remove existing stats for this stream
        self.stream_stats
            .retain(|s| s.stream_name != stats.stream_name);
        self.stream_stats.push(stats);
    }

    /// Optimize a join plan based on stream characteristics
    pub fn optimize_join(
        &self,
        left_stream: &str,
        right_stream: &str,
        join_type: JoinType,
        join_strategy: JoinStrategy,
    ) -> OptimizedJoinPlan {
        let left_stats = self
            .stream_stats
            .iter()
            .find(|s| s.stream_name == left_stream);
        let right_stats = self
            .stream_stats
            .iter()
            .find(|s| s.stream_name == right_stream);

        let mut optimizations = Vec::new();
        let mut explanation = String::new();
        let mut estimated_cost = 1.0;

        // Optimization 1: Build smaller stream
        if let (Some(left), Some(right)) = (left_stats, right_stats) {
            let left_size = left.estimated_event_rate * left.average_event_size as f64;
            let right_size = right.estimated_event_rate * right.average_event_size as f64;

            if left_size < right_size * 0.7 {
                optimizations.push(JoinOptimization::BuildSmaller);
                explanation.push_str("Using left stream as build side (smaller). ");
                estimated_cost *= 0.8;
            } else if right_size < left_size * 0.7 {
                optimizations.push(JoinOptimization::BuildSmaller);
                explanation.push_str("Using right stream as build side (smaller). ");
                estimated_cost *= 0.8;
            }
        }

        // Optimization 2: Pre-partitioning for high-cardinality joins
        if let (Some(left), Some(right)) = (left_stats, right_stats) {
            let max_cardinality = left.estimated_cardinality.max(right.estimated_cardinality);
            if max_cardinality > 1000 {
                let partition_count = (max_cardinality / 100).min(32);
                optimizations.push(JoinOptimization::PrePartition { partition_count });
                explanation.push_str(&format!(
                    "Pre-partitioning into {} partitions for high cardinality. ",
                    partition_count
                ));
                estimated_cost *= 0.7;
            }
        }

        // Optimization 3: Bloom filter for sparse joins
        if let (Some(left), Some(right)) = (left_stats, right_stats) {
            let join_selectivity = (left.estimated_cardinality.min(right.estimated_cardinality)
                as f64)
                / (left.estimated_cardinality.max(right.estimated_cardinality) as f64);

            if join_selectivity < 0.1 {
                // Very sparse join
                optimizations.push(JoinOptimization::BloomFilter {
                    false_positive_rate: 0.01,
                });
                explanation.push_str("Using bloom filter for sparse join (< 10% selectivity). ");
                estimated_cost *= 0.6;
            }
        }

        // Optimization 4: Index join key for frequent lookups
        if let (Some(left), Some(right)) = (left_stats, right_stats) {
            if left.estimated_event_rate > 100.0 || right.estimated_event_rate > 100.0 {
                optimizations.push(JoinOptimization::IndexJoinKey);
                explanation.push_str("Indexing join key for high-frequency streams. ");
                estimated_cost *= 0.85;
            }
        }

        // Optimization 5: Window merging for tumbling windows
        if let JoinStrategy::TimeWindow { duration } = &join_strategy {
            if duration.as_secs() >= 60 {
                optimizations.push(JoinOptimization::MergeWindows);
                explanation.push_str("Merging adjacent windows for efficiency. ");
                estimated_cost *= 0.9;
            }
        }

        // Adjust cost based on join type
        match join_type {
            JoinType::Inner => {
                explanation.push_str("Inner join - most efficient. ");
            }
            JoinType::LeftOuter | JoinType::RightOuter => {
                explanation.push_str("Outer join - tracking unmatched events. ");
                estimated_cost *= 1.2;
            }
            JoinType::FullOuter => {
                explanation.push_str("Full outer join - tracking all unmatched events. ");
                estimated_cost *= 1.4;
            }
        }

        if optimizations.is_empty() {
            explanation.push_str("No specific optimizations recommended - using default strategy.");
        }

        OptimizedJoinPlan {
            left_stream: left_stream.to_string(),
            right_stream: right_stream.to_string(),
            join_type,
            join_strategy,
            optimizations,
            estimated_cost,
            explanation: explanation.trim().to_string(),
        }
    }

    /// Estimate memory usage for a join
    pub fn estimate_memory_usage(
        &self,
        left_stream: &str,
        right_stream: &str,
        window_duration: Duration,
    ) -> usize {
        let left_stats = self
            .stream_stats
            .iter()
            .find(|s| s.stream_name == left_stream);
        let right_stats = self
            .stream_stats
            .iter()
            .find(|s| s.stream_name == right_stream);

        let mut total_memory = 0;

        if let Some(left) = left_stats {
            let events_in_window =
                (left.estimated_event_rate * window_duration.as_secs_f64()) as usize;
            total_memory += events_in_window * left.average_event_size;
        }

        if let Some(right) = right_stats {
            let events_in_window =
                (right.estimated_event_rate * window_duration.as_secs_f64()) as usize;
            total_memory += events_in_window * right.average_event_size;
        }

        // Add overhead for hash tables and tracking structures (roughly 50%)
        (total_memory as f64 * 1.5) as usize
    }

    /// Recommend optimal window size based on stream characteristics
    pub fn recommend_window_size(
        &self,
        left_stream: &str,
        right_stream: &str,
        max_memory_bytes: usize,
    ) -> Duration {
        let left_stats = self
            .stream_stats
            .iter()
            .find(|s| s.stream_name == left_stream);
        let right_stats = self
            .stream_stats
            .iter()
            .find(|s| s.stream_name == right_stream);

        // Default to 5 minutes if no stats
        let default_window = Duration::from_secs(300);

        match (left_stats, right_stats) {
            (Some(left), Some(right)) => {
                let combined_rate = left.estimated_event_rate + right.estimated_event_rate;
                let avg_event_size = (left.average_event_size + right.average_event_size) / 2;

                // Calculate maximum window duration that fits in memory
                let max_events = max_memory_bytes / avg_event_size;
                let max_duration_secs = (max_events as f64 / combined_rate) as u64;

                // Use 80% of max to leave buffer
                let recommended_secs = (max_duration_secs as f64 * 0.8) as u64;

                // Clamp between 10 seconds and 1 hour
                let clamped_secs = recommended_secs.max(10).min(3600);

                Duration::from_secs(clamped_secs)
            }
            _ => default_window,
        }
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
    fn test_build_smaller_optimization() {
        let mut optimizer = JoinOptimizer::new();

        optimizer.register_stream_stats(StreamStats {
            stream_name: "small".to_string(),
            estimated_event_rate: 10.0,
            estimated_cardinality: 100,
            average_event_size: 100,
        });

        optimizer.register_stream_stats(StreamStats {
            stream_name: "large".to_string(),
            estimated_event_rate: 100.0,
            estimated_cardinality: 1000,
            average_event_size: 100,
        });

        let plan = optimizer.optimize_join(
            "small",
            "large",
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(60),
            },
        );

        assert!(plan
            .optimizations
            .iter()
            .any(|opt| matches!(opt, JoinOptimization::BuildSmaller)));
        assert!(plan.estimated_cost < 1.0); // Should be optimized
    }

    #[test]
    fn test_pre_partition_optimization() {
        let mut optimizer = JoinOptimizer::new();

        optimizer.register_stream_stats(StreamStats {
            stream_name: "high_cardinality".to_string(),
            estimated_event_rate: 50.0,
            estimated_cardinality: 5000,
            average_event_size: 200,
        });

        optimizer.register_stream_stats(StreamStats {
            stream_name: "other".to_string(),
            estimated_event_rate: 50.0,
            estimated_cardinality: 100,
            average_event_size: 200,
        });

        let plan = optimizer.optimize_join(
            "high_cardinality",
            "other",
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(60),
            },
        );

        assert!(plan
            .optimizations
            .iter()
            .any(|opt| matches!(opt, JoinOptimization::PrePartition { .. })));
    }

    #[test]
    fn test_bloom_filter_for_sparse_join() {
        let mut optimizer = JoinOptimizer::new();

        optimizer.register_stream_stats(StreamStats {
            stream_name: "sparse_left".to_string(),
            estimated_event_rate: 50.0,
            estimated_cardinality: 50,
            average_event_size: 200,
        });

        optimizer.register_stream_stats(StreamStats {
            stream_name: "sparse_right".to_string(),
            estimated_event_rate: 50.0,
            estimated_cardinality: 1000,
            average_event_size: 200,
        });

        let plan = optimizer.optimize_join(
            "sparse_left",
            "sparse_right",
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(60),
            },
        );

        assert!(plan
            .optimizations
            .iter()
            .any(|opt| matches!(opt, JoinOptimization::BloomFilter { .. })));
    }

    #[test]
    fn test_memory_estimation() {
        let mut optimizer = JoinOptimizer::new();

        optimizer.register_stream_stats(StreamStats {
            stream_name: "stream1".to_string(),
            estimated_event_rate: 100.0, // 100 events/sec
            estimated_cardinality: 100,
            average_event_size: 1000, // 1KB per event
        });

        optimizer.register_stream_stats(StreamStats {
            stream_name: "stream2".to_string(),
            estimated_event_rate: 100.0,
            estimated_cardinality: 100,
            average_event_size: 1000,
        });

        let memory = optimizer.estimate_memory_usage(
            "stream1",
            "stream2",
            Duration::from_secs(10), // 10 second window
        );

        // Should be roughly: 100 events/sec * 10 sec * 1KB * 2 streams * 1.5 overhead
        // = 3MB
        assert!(memory > 2_000_000 && memory < 4_000_000);
    }

    #[test]
    fn test_window_size_recommendation() {
        let mut optimizer = JoinOptimizer::new();

        optimizer.register_stream_stats(StreamStats {
            stream_name: "stream1".to_string(),
            estimated_event_rate: 100.0,
            estimated_cardinality: 100,
            average_event_size: 1000,
        });

        optimizer.register_stream_stats(StreamStats {
            stream_name: "stream2".to_string(),
            estimated_event_rate: 100.0,
            estimated_cardinality: 100,
            average_event_size: 1000,
        });

        let window = optimizer.recommend_window_size(
            "stream1", "stream2", 10_000_000, // 10MB max memory
        );

        // With 200 events/sec at 1KB each, 10MB should allow ~50 seconds
        // But we use 80% of that, so ~40 seconds
        assert!(window.as_secs() >= 30 && window.as_secs() <= 50);
    }

    #[test]
    fn test_outer_join_cost_adjustment() {
        let optimizer = JoinOptimizer::new();

        let inner_plan = optimizer.optimize_join(
            "left",
            "right",
            JoinType::Inner,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(60),
            },
        );

        let full_outer_plan = optimizer.optimize_join(
            "left",
            "right",
            JoinType::FullOuter,
            JoinStrategy::TimeWindow {
                duration: Duration::from_secs(60),
            },
        );

        // Full outer join should be more expensive than inner join
        assert!(full_outer_plan.estimated_cost > inner_plan.estimated_cost);
    }
}

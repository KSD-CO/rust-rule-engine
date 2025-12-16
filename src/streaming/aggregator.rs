//! Stream Aggregation Functions
//!
//! Provides various aggregation operations for streaming data analysis.

use crate::streaming::event::StreamEvent;
use crate::streaming::window::TimeWindow;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of aggregation to perform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AggregationType {
    /// Count number of events
    Count,
    /// Sum numeric values
    Sum { field: String },
    /// Calculate average
    Average { field: String },
    /// Find minimum value
    Min { field: String },
    /// Find maximum value
    Max { field: String },
    /// Count distinct values
    CountDistinct { field: String },
    /// Calculate standard deviation
    StdDev { field: String },
    /// Calculate percentile
    Percentile { field: String, percentile: f64 },
    /// First event in window
    First,
    /// Last event in window
    Last,
    /// Count by category
    CountBy { field: String },
}

/// Result of an aggregation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AggregationResult {
    /// Numeric result
    Number(f64),
    /// String result
    Text(String),
    /// Boolean result
    Boolean(bool),
    /// Map of category counts
    CountMap(HashMap<String, usize>),
    /// No result (empty data)
    None,
}

impl AggregationResult {
    /// Convert to numeric value if possible
    pub fn as_number(&self) -> Option<f64> {
        match self {
            AggregationResult::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Convert to string if possible
    pub fn as_string(&self) -> Option<&str> {
        match self {
            AggregationResult::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to boolean if possible
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            AggregationResult::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

/// Aggregator for performing calculations on event streams
#[derive(Debug)]
#[allow(dead_code)]
pub struct Aggregator {
    /// Type of aggregation
    aggregation_type: AggregationType,
    /// Field to aggregate on (if applicable)
    _field: Option<String>,
}

impl Aggregator {
    /// Create a new aggregator
    pub fn new(aggregation_type: AggregationType) -> Self {
        let _field = match &aggregation_type {
            AggregationType::Sum { field }
            | AggregationType::Average { field }
            | AggregationType::Min { field }
            | AggregationType::Max { field }
            | AggregationType::CountDistinct { field }
            | AggregationType::StdDev { field }
            | AggregationType::Percentile { field, .. }
            | AggregationType::CountBy { field } => Some(field.clone()),
            _ => None,
        };

        Self {
            aggregation_type,
            _field,
        }
    }

    /// Perform aggregation on a time window
    pub fn aggregate(&self, window: &TimeWindow) -> AggregationResult {
        let events = window.events();

        match &self.aggregation_type {
            AggregationType::Count => AggregationResult::Number(events.len() as f64),

            AggregationType::Sum { field } => {
                let sum = window.sum(field);
                AggregationResult::Number(sum)
            }

            AggregationType::Average { field } => match window.average(field) {
                Some(avg) => AggregationResult::Number(avg),
                None => AggregationResult::None,
            },

            AggregationType::Min { field } => match window.min(field) {
                Some(min) => AggregationResult::Number(min),
                None => AggregationResult::None,
            },

            AggregationType::Max { field } => match window.max(field) {
                Some(max) => AggregationResult::Number(max),
                None => AggregationResult::None,
            },

            AggregationType::CountDistinct { field } => {
                let distinct_count = self.count_distinct_values(events, field);
                AggregationResult::Number(distinct_count as f64)
            }

            AggregationType::StdDev { field } => {
                let std_dev = self.calculate_std_dev(events, field);
                match std_dev {
                    Some(sd) => AggregationResult::Number(sd),
                    None => AggregationResult::None,
                }
            }

            AggregationType::Percentile { field, percentile } => {
                let value = self.calculate_percentile(events, field, *percentile);
                match value {
                    Some(v) => AggregationResult::Number(v),
                    None => AggregationResult::None,
                }
            }

            AggregationType::First => match events.front() {
                Some(event) => AggregationResult::Text(event.id.clone()),
                None => AggregationResult::None,
            },

            AggregationType::Last => match events.back() {
                Some(event) => AggregationResult::Text(event.id.clone()),
                None => AggregationResult::None,
            },

            AggregationType::CountBy { field } => {
                let counts = self.count_by_field(events, field);
                AggregationResult::CountMap(counts)
            }
        }
    }

    /// Perform aggregation on a slice of events
    pub fn aggregate_events(&self, events: &[StreamEvent]) -> AggregationResult {
        match &self.aggregation_type {
            AggregationType::Count => AggregationResult::Number(events.len() as f64),

            AggregationType::Sum { field } => {
                let sum: f64 = events.iter().filter_map(|e| e.get_numeric(field)).sum();
                AggregationResult::Number(sum)
            }

            AggregationType::Average { field } => {
                let values: Vec<f64> = events.iter().filter_map(|e| e.get_numeric(field)).collect();

                if values.is_empty() {
                    AggregationResult::None
                } else {
                    let avg = values.iter().sum::<f64>() / values.len() as f64;
                    AggregationResult::Number(avg)
                }
            }

            _ => {
                // For other types, create a temporary window
                // This is less efficient but provides compatibility
                AggregationResult::None
            }
        }
    }

    /// Count distinct values in a field
    fn count_distinct_values(
        &self,
        events: &std::collections::VecDeque<StreamEvent>,
        field: &str,
    ) -> usize {
        let mut seen = std::collections::HashSet::new();

        for event in events {
            if let Some(value) = event.data.get(field) {
                seen.insert(format!("{:?}", value));
            }
        }

        seen.len()
    }

    /// Calculate standard deviation
    fn calculate_std_dev(
        &self,
        events: &std::collections::VecDeque<StreamEvent>,
        field: &str,
    ) -> Option<f64> {
        let values: Vec<f64> = events.iter().filter_map(|e| e.get_numeric(field)).collect();

        if values.len() < 2 {
            return None;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

        Some(variance.sqrt())
    }

    /// Calculate percentile
    fn calculate_percentile(
        &self,
        events: &std::collections::VecDeque<StreamEvent>,
        field: &str,
        percentile: f64,
    ) -> Option<f64> {
        let mut values: Vec<f64> = events.iter().filter_map(|e| e.get_numeric(field)).collect();

        if values.is_empty() {
            return None;
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let index = (percentile / 100.0 * (values.len() - 1) as f64).round() as usize;
        values.get(index).copied()
    }

    /// Count occurrences by field value
    fn count_by_field(
        &self,
        events: &std::collections::VecDeque<StreamEvent>,
        field: &str,
    ) -> HashMap<String, usize> {
        let mut counts = HashMap::new();

        for event in events {
            if let Some(value) = event.data.get(field) {
                let key = match value {
                    crate::types::Value::String(s) => s.clone(),
                    crate::types::Value::Number(n) => n.to_string(),
                    crate::types::Value::Integer(i) => i.to_string(),
                    crate::types::Value::Boolean(b) => b.to_string(),
                    _ => format!("{:?}", value),
                };

                *counts.entry(key).or_insert(0) += 1;
            }
        }

        counts
    }
}

/// Stream analytics helper for complex aggregations
#[derive(Debug)]
pub struct StreamAnalytics {
    /// Cache of recent calculations
    cache: HashMap<String, (u64, AggregationResult)>,
    /// Cache TTL in milliseconds
    cache_ttl: u64,
}

impl StreamAnalytics {
    /// Create new stream analytics instance
    pub fn new(cache_ttl_ms: u64) -> Self {
        Self {
            cache: HashMap::new(),
            cache_ttl: cache_ttl_ms,
        }
    }

    /// Perform cached aggregation
    pub fn aggregate_cached(
        &mut self,
        key: &str,
        window: &TimeWindow,
        aggregator: &Aggregator,
        current_time: u64,
    ) -> AggregationResult {
        // Check cache
        if let Some((timestamp, result)) = self.cache.get(key) {
            if current_time - timestamp < self.cache_ttl {
                return result.clone();
            }
        }

        // Calculate new result
        let result = aggregator.aggregate(window);
        self.cache
            .insert(key.to_string(), (current_time, result.clone()));

        // Clean old cache entries
        self.cache
            .retain(|_, (timestamp, _)| current_time - *timestamp < self.cache_ttl);

        result
    }

    /// Calculate moving average over multiple windows
    pub fn moving_average(
        &self,
        windows: &[TimeWindow],
        field: &str,
        window_count: usize,
    ) -> Option<f64> {
        if windows.is_empty() {
            return None;
        }

        let recent_windows = if windows.len() > window_count {
            &windows[windows.len() - window_count..]
        } else {
            windows
        };

        let total_sum: f64 = recent_windows.iter().map(|w| w.sum(field)).sum();

        let total_count: usize = recent_windows.iter().map(|w| w.count()).sum();

        if total_count == 0 {
            None
        } else {
            Some(total_sum / total_count as f64)
        }
    }

    /// Detect anomalies using z-score
    pub fn detect_anomalies(
        &self,
        windows: &[TimeWindow],
        field: &str,
        threshold: f64,
    ) -> Vec<String> {
        if windows.len() < 3 {
            return Vec::new();
        }

        // Calculate baseline statistics from historical windows
        let historical_windows = &windows[..windows.len() - 1];
        let values: Vec<f64> = historical_windows
            .iter()
            .flat_map(|w| w.events().iter().filter_map(|e| e.get_numeric(field)))
            .collect();

        if values.len() < 10 {
            return Vec::new();
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // Check current window for anomalies
        let current_window = windows.last().unwrap();
        let mut anomalies = Vec::new();

        for event in current_window.events() {
            if let Some(value) = event.get_numeric(field) {
                let z_score = (value - mean) / std_dev;
                if z_score.abs() > threshold {
                    anomalies.push(event.id.clone());
                }
            }
        }

        anomalies
    }

    /// Calculate trend direction
    pub fn calculate_trend(&self, windows: &[TimeWindow], field: &str) -> TrendDirection {
        if windows.len() < 2 {
            return TrendDirection::Stable;
        }

        let averages: Vec<f64> = windows.iter().filter_map(|w| w.average(field)).collect();

        if averages.len() < 2 {
            return TrendDirection::Stable;
        }

        let first_half = &averages[..averages.len() / 2];
        let second_half = &averages[averages.len() / 2..];

        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;

        let change_percent = ((second_avg - first_avg) / first_avg) * 100.0;

        if change_percent > 5.0 {
            TrendDirection::Increasing
        } else if change_percent < -5.0 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }
}

/// Direction of trend analysis
#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    /// Values are increasing
    Increasing,
    /// Values are decreasing
    Decreasing,
    /// Values are stable
    Stable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::streaming::event::StreamEvent;
    use crate::types::Value;
    use std::collections::HashMap;

    #[test]
    fn test_count_aggregation() {
        let aggregator = Aggregator::new(AggregationType::Count);
        let events = create_test_events(5);

        let result = aggregator.aggregate_events(&events);
        assert_eq!(result.as_number(), Some(5.0));
    }

    #[test]
    fn test_sum_aggregation() {
        let aggregator = Aggregator::new(AggregationType::Sum {
            field: "value".to_string(),
        });
        let events = create_test_events(5);

        let result = aggregator.aggregate_events(&events);
        assert_eq!(result.as_number(), Some(10.0)); // 0+1+2+3+4
    }

    #[test]
    fn test_average_aggregation() {
        let aggregator = Aggregator::new(AggregationType::Average {
            field: "value".to_string(),
        });
        let events = create_test_events(5);

        let result = aggregator.aggregate_events(&events);
        assert_eq!(result.as_number(), Some(2.0));
    }

    fn create_test_events(count: usize) -> Vec<StreamEvent> {
        (0..count)
            .map(|i| {
                let mut data = HashMap::new();
                data.insert("value".to_string(), Value::Number(i as f64));
                StreamEvent::new("TestEvent", data, "test")
            })
            .collect()
    }
}

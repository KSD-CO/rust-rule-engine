//! Stream Operators - Fluent API for Stream Processing
//!
//! This module provides a fluent, composable API for building stream processing pipelines.
//! Inspired by Apache Flink, Kafka Streams, and Rust iterators.
//!
//! ## Example
//!
//! ```rust,ignore
//! use rust_rule_engine::streaming::*;
//!
//! let result = DataStream::from_events(events)
//!     .filter(|e| e.get_numeric("amount").unwrap_or(0.0) > 100.0)
//!     .map(|e| enhance_event(e))
//!     .key_by(|e| e.get_string("user_id").unwrap_or("unknown").to_string())
//!     .window(WindowConfig::sliding(Duration::from_secs(60)))
//!     .reduce(|acc, e| {
//!         let sum = acc.get_numeric("total").unwrap_or(0.0);
//!         let amount = e.get_numeric("amount").unwrap_or(0.0);
//!         acc.data.insert("total".to_string(), Value::Number(sum + amount));
//!         acc
//!     })
//!     .collect();
//! ```

use crate::streaming::event::StreamEvent;
use crate::streaming::window::{TimeWindow, WindowType};
use crate::types::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// A stream of events with chainable operators
#[derive(Clone)]
pub struct DataStream {
    events: Vec<StreamEvent>,
}

impl DataStream {
    /// Create a new data stream from events
    pub fn from_events(events: Vec<StreamEvent>) -> Self {
        Self { events }
    }

    /// Create an empty data stream
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Add an event to the stream
    pub fn push(&mut self, event: StreamEvent) {
        self.events.push(event);
    }

    /// Get the number of events in the stream
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if stream is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Filter events based on a predicate
    ///
    /// # Example
    /// ```rust,ignore
    /// stream.filter(|e| e.get_numeric("amount").unwrap_or(0.0) > 100.0)
    /// ```
    pub fn filter<F>(self, predicate: F) -> Self
    where
        F: Fn(&StreamEvent) -> bool,
    {
        let filtered_events = self.events.into_iter().filter(predicate).collect();
        Self {
            events: filtered_events,
        }
    }

    /// Transform each event using a mapping function
    ///
    /// # Example
    /// ```rust,ignore
    /// stream.map(|mut e| {
    ///     e.add_tag("processed", "true");
    ///     e
    /// })
    /// ```
    pub fn map<F>(self, mapper: F) -> Self
    where
        F: Fn(StreamEvent) -> StreamEvent,
    {
        let mapped_events = self.events.into_iter().map(mapper).collect();
        Self {
            events: mapped_events,
        }
    }

    /// Transform each event into multiple events (flatMap)
    ///
    /// # Example
    /// ```rust,ignore
    /// stream.flat_map(|e| {
    ///     // Split event into multiple events
    ///     vec![e.clone(), e]
    /// })
    /// ```
    pub fn flat_map<F>(self, mapper: F) -> Self
    where
        F: Fn(StreamEvent) -> Vec<StreamEvent>,
    {
        let flat_mapped_events = self
            .events
            .into_iter()
            .flat_map(mapper)
            .collect();
        Self {
            events: flat_mapped_events,
        }
    }

    /// Key events by a specific field or function
    ///
    /// # Example
    /// ```rust,ignore
    /// stream.key_by(|e| e.get_string("user_id").unwrap_or("default").to_string())
    /// ```
    pub fn key_by<F, K>(self, key_selector: F) -> KeyedStream<K>
    where
        F: Fn(&StreamEvent) -> K,
        K: std::hash::Hash + Eq + Clone,
    {
        let mut keyed_events: HashMap<K, Vec<StreamEvent>> = HashMap::new();

        for event in self.events {
            let key = key_selector(&event);
            keyed_events.entry(key).or_insert_with(Vec::new).push(event);
        }

        KeyedStream { keyed_events }
    }

    /// Apply a window to the stream
    ///
    /// # Example
    /// ```rust,ignore
    /// stream.window(WindowConfig::sliding(Duration::from_secs(60)))
    /// ```
    pub fn window(self, config: WindowConfig) -> WindowedStream {
        WindowedStream::new(self.events, config)
    }

    /// Reduce events to a single result
    ///
    /// # Example
    /// ```rust,ignore
    /// stream.reduce(|acc, e| {
    ///     // Accumulate values
    ///     acc
    /// })
    /// ```
    pub fn reduce<F>(self, reducer: F) -> Option<StreamEvent>
    where
        F: Fn(StreamEvent, StreamEvent) -> StreamEvent,
    {
        self.events.into_iter().reduce(reducer)
    }

    /// Count the number of events
    pub fn count(self) -> usize {
        self.events.len()
    }

    /// Collect events into a vector
    pub fn collect(self) -> Vec<StreamEvent> {
        self.events
    }

    /// Take only the first n events
    pub fn take(self, n: usize) -> Self {
        Self {
            events: self.events.into_iter().take(n).collect(),
        }
    }

    /// Skip the first n events
    pub fn skip(self, n: usize) -> Self {
        Self {
            events: self.events.into_iter().skip(n).collect(),
        }
    }

    /// Process each event with a side effect (doesn't modify the stream)
    ///
    /// # Example
    /// ```rust,ignore
    /// stream.for_each(|e| {
    ///     println!("Processing: {:?}", e);
    /// })
    /// ```
    pub fn for_each<F>(self, action: F) -> Self
    where
        F: Fn(&StreamEvent),
    {
        for event in &self.events {
            action(event);
        }
        self
    }

    /// Union with another stream
    pub fn union(mut self, other: DataStream) -> Self {
        self.events.extend(other.events);
        Self {
            events: self.events,
        }
    }

    /// Find events matching a pattern
    pub fn find<F>(self, predicate: F) -> Option<StreamEvent>
    where
        F: Fn(&StreamEvent) -> bool,
    {
        self.events.into_iter().find(predicate)
    }

    /// Check if any event matches the predicate
    pub fn any<F>(&self, predicate: F) -> bool
    where
        F: Fn(&StreamEvent) -> bool,
    {
        self.events.iter().any(predicate)
    }

    /// Check if all events match the predicate
    pub fn all<F>(&self, predicate: F) -> bool
    where
        F: Fn(&StreamEvent) -> bool,
    {
        self.events.iter().all(predicate)
    }

    /// Sort events by a key function
    pub fn sort_by<F, K>(mut self, key_fn: F) -> Self
    where
        F: Fn(&StreamEvent) -> K,
        K: Ord,
    {
        self.events.sort_by_key(key_fn);
        Self {
            events: self.events,
        }
    }

    /// Group events by a key and apply aggregation
    pub fn group_by<F, K>(self, key_selector: F) -> GroupedStream<K>
    where
        F: Fn(&StreamEvent) -> K,
        K: std::hash::Hash + Eq + Clone,
    {
        let mut grouped: HashMap<K, Vec<StreamEvent>> = HashMap::new();

        for event in self.events {
            let key = key_selector(&event);
            grouped.entry(key).or_insert_with(Vec::new).push(event);
        }

        GroupedStream { groups: grouped }
    }

    /// Apply an aggregation function
    pub fn aggregate<A>(self, aggregator: A) -> AggregateResult
    where
        A: Aggregation,
    {
        aggregator.aggregate(&self.events)
    }
}

impl Default for DataStream {
    fn default() -> Self {
        Self::new()
    }
}

/// A stream of events keyed by a specific field
pub struct KeyedStream<K>
where
    K: std::hash::Hash + Eq,
{
    keyed_events: HashMap<K, Vec<StreamEvent>>,
}

impl<K> KeyedStream<K>
where
    K: std::hash::Hash + Eq + Clone,
{
    /// Reduce events within each key
    pub fn reduce<F>(self, reducer: F) -> HashMap<K, StreamEvent>
    where
        F: Fn(StreamEvent, StreamEvent) -> StreamEvent,
    {
        self.keyed_events
            .into_iter()
            .filter_map(|(key, events)| {
                events
                    .into_iter()
                    .reduce(|acc, e| reducer(acc, e))
                    .map(|result| (key, result))
            })
            .collect()
    }

    /// Apply aggregation to each key group
    pub fn aggregate<A>(self, aggregator: A) -> HashMap<K, AggregateResult>
    where
        A: Aggregation + Clone,
    {
        self.keyed_events
            .into_iter()
            .map(|(key, events)| (key, aggregator.clone().aggregate(&events)))
            .collect()
    }

    /// Apply a window to each key group
    pub fn window(self, config: WindowConfig) -> KeyedWindowedStream<K> {
        KeyedWindowedStream {
            keyed_events: self.keyed_events,
            config,
        }
    }

    /// Count events per key
    pub fn count(self) -> HashMap<K, usize> {
        self.keyed_events
            .into_iter()
            .map(|(key, events)| (key, events.len()))
            .collect()
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<K> {
        self.keyed_events.keys().cloned().collect()
    }

    /// Flatten back to a regular stream
    pub fn flatten(self) -> DataStream {
        let events: Vec<StreamEvent> = self
            .keyed_events
            .into_iter()
            .flat_map(|(_, events)| events)
            .collect();

        DataStream { events }
    }
}

/// Window configuration for stream processing
#[derive(Debug, Clone)]
pub struct WindowConfig {
    pub window_type: WindowType,
    pub duration: Duration,
    pub max_events: usize,
}

impl WindowConfig {
    /// Create a sliding window configuration
    pub fn sliding(duration: Duration) -> Self {
        Self {
            window_type: WindowType::Sliding,
            duration,
            max_events: 10000,
        }
    }

    /// Create a tumbling window configuration
    pub fn tumbling(duration: Duration) -> Self {
        Self {
            window_type: WindowType::Tumbling,
            duration,
            max_events: 10000,
        }
    }

    /// Create a session window configuration
    pub fn session(timeout: Duration) -> Self {
        Self {
            window_type: WindowType::Session { timeout },
            duration: timeout,
            max_events: 10000,
        }
    }

    /// Set maximum events per window
    pub fn with_max_events(mut self, max_events: usize) -> Self {
        self.max_events = max_events;
        self
    }
}

/// A stream with windowing applied
pub struct WindowedStream {
    windows: Vec<TimeWindow>,
}

impl WindowedStream {
    /// Create a new windowed stream
    pub fn new(events: Vec<StreamEvent>, config: WindowConfig) -> Self {
        let mut windows = Vec::new();

        if events.is_empty() {
            return Self { windows };
        }

        // Group events into windows based on configuration
        match config.window_type {
            WindowType::Tumbling => {
                // Calculate window boundaries
                let window_ms = config.duration.as_millis() as u64;
                let mut window_map: HashMap<u64, Vec<StreamEvent>> = HashMap::new();

                for event in events {
                    let window_start = (event.metadata.timestamp / window_ms) * window_ms;
                    window_map
                        .entry(window_start)
                        .or_insert_with(Vec::new)
                        .push(event);
                }

                // Create windows
                for (start_time, mut window_events) in window_map {
                    let mut window = TimeWindow::new(
                        config.window_type.clone(),
                        config.duration,
                        start_time,
                        config.max_events,
                    );

                    for event in window_events.drain(..) {
                        window.add_event(event);
                    }

                    windows.push(window);
                }
            }
            WindowType::Sliding | WindowType::Session { .. } => {
                // For sliding windows, create overlapping windows
                // Simplified implementation: create one window per unique timestamp
                let window_ms = config.duration.as_millis() as u64;

                if !events.is_empty() {
                    let min_time = events.iter().map(|e| e.metadata.timestamp).min().unwrap();
                    let max_time = events.iter().map(|e| e.metadata.timestamp).max().unwrap();

                    let mut current_start = min_time;

                    while current_start <= max_time {
                        let mut window = TimeWindow::new(
                            config.window_type.clone(),
                            config.duration,
                            current_start,
                            config.max_events,
                        );

                        for event in &events {
                            if event.metadata.timestamp >= current_start
                                && event.metadata.timestamp < current_start + window_ms
                            {
                                window.add_event(event.clone());
                            }
                        }

                        if window.count() > 0 {
                            windows.push(window);
                        }

                        // Slide forward (overlap 50%)
                        current_start += window_ms / 2;
                    }
                }
            }
        }

        Self { windows }
    }

    /// Apply aggregation to each window
    pub fn aggregate<A>(self, aggregator: A) -> Vec<AggregateResult>
    where
        A: Aggregation,
    {
        self.windows
            .iter()
            .map(|window| {
                let events: Vec<StreamEvent> = window.events().iter().cloned().collect();
                aggregator.aggregate(&events)
            })
            .collect()
    }

    /// Reduce events within each window
    pub fn reduce<F>(self, reducer: F) -> Vec<StreamEvent>
    where
        F: Fn(StreamEvent, StreamEvent) -> StreamEvent + Clone,
    {
        self.windows
            .into_iter()
            .filter_map(|window| {
                let events: Vec<StreamEvent> = window.events().iter().cloned().collect();
                events.into_iter().reduce(|acc, e| reducer(acc, e))
            })
            .collect()
    }

    /// Get all windows
    pub fn windows(&self) -> &[TimeWindow] {
        &self.windows
    }

    /// Count events in each window
    pub fn counts(self) -> Vec<usize> {
        self.windows.iter().map(|w| w.count()).collect()
    }

    /// Flatten all windows back into a stream
    pub fn flatten(self) -> DataStream {
        let events: Vec<StreamEvent> = self
            .windows
            .into_iter()
            .flat_map(|w| w.events().iter().cloned().collect::<Vec<_>>())
            .collect();

        DataStream { events }
    }
}

/// Keyed stream with windowing
pub struct KeyedWindowedStream<K>
where
    K: std::hash::Hash + Eq,
{
    keyed_events: HashMap<K, Vec<StreamEvent>>,
    config: WindowConfig,
}

impl<K> KeyedWindowedStream<K>
where
    K: std::hash::Hash + Eq + Clone,
{
    /// Apply aggregation to each key's window
    pub fn aggregate<A>(self, aggregator: A) -> HashMap<K, Vec<AggregateResult>>
    where
        A: Aggregation + Clone,
    {
        self.keyed_events
            .into_iter()
            .map(|(key, events)| {
                let windowed = WindowedStream::new(events, self.config.clone());
                let results = windowed.aggregate(aggregator.clone());
                (key, results)
            })
            .collect()
    }

    /// Reduce events within each key's window
    pub fn reduce<F>(self, reducer: F) -> HashMap<K, Vec<StreamEvent>>
    where
        F: Fn(StreamEvent, StreamEvent) -> StreamEvent + Clone,
    {
        self.keyed_events
            .into_iter()
            .map(|(key, events)| {
                let windowed = WindowedStream::new(events, self.config.clone());
                let results = windowed.reduce(reducer.clone());
                (key, results)
            })
            .collect()
    }
}

/// Grouped stream for aggregations
pub struct GroupedStream<K>
where
    K: std::hash::Hash + Eq,
{
    groups: HashMap<K, Vec<StreamEvent>>,
}

impl<K> GroupedStream<K>
where
    K: std::hash::Hash + Eq + Clone,
{
    /// Apply aggregation to each group
    pub fn aggregate<A>(self, aggregator: A) -> HashMap<K, AggregateResult>
    where
        A: Aggregation + Clone,
    {
        self.groups
            .into_iter()
            .map(|(key, events)| (key, aggregator.clone().aggregate(&events)))
            .collect()
    }

    /// Count events in each group
    pub fn count(self) -> HashMap<K, usize> {
        self.groups
            .into_iter()
            .map(|(key, events)| (key, events.len()))
            .collect()
    }

    /// Get the first event in each group
    pub fn first(self) -> HashMap<K, StreamEvent> {
        self.groups
            .into_iter()
            .filter_map(|(key, mut events)| {
                if !events.is_empty() {
                    Some((key, events.remove(0)))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the last event in each group
    pub fn last(self) -> HashMap<K, StreamEvent> {
        self.groups
            .into_iter()
            .filter_map(|(key, mut events)| events.pop().map(|e| (key, e)))
            .collect()
    }
}

/// Trait for aggregation functions
pub trait Aggregation: Send + Sync {
    fn aggregate(&self, events: &[StreamEvent]) -> AggregateResult;
}

/// Result of an aggregation operation
#[derive(Debug, Clone)]
pub enum AggregateResult {
    Number(f64),
    String(String),
    Map(HashMap<String, Value>),
    List(Vec<Value>),
    None,
}

impl AggregateResult {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            AggregateResult::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            AggregateResult::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            AggregateResult::Map(m) => Some(m),
            _ => None,
        }
    }
}

// Built-in aggregators

/// Count aggregator
#[derive(Clone)]
pub struct Count;

impl Aggregation for Count {
    fn aggregate(&self, events: &[StreamEvent]) -> AggregateResult {
        AggregateResult::Number(events.len() as f64)
    }
}

/// Sum aggregator
#[derive(Clone)]
pub struct Sum {
    pub field: String,
}

impl Sum {
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
        }
    }
}

impl Aggregation for Sum {
    fn aggregate(&self, events: &[StreamEvent]) -> AggregateResult {
        let sum: f64 = events
            .iter()
            .filter_map(|e| e.get_numeric(&self.field))
            .sum();
        AggregateResult::Number(sum)
    }
}

/// Average aggregator
#[derive(Clone)]
pub struct Average {
    pub field: String,
}

impl Average {
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
        }
    }
}

impl Aggregation for Average {
    fn aggregate(&self, events: &[StreamEvent]) -> AggregateResult {
        let values: Vec<f64> = events
            .iter()
            .filter_map(|e| e.get_numeric(&self.field))
            .collect();

        if values.is_empty() {
            AggregateResult::None
        } else {
            let avg = values.iter().sum::<f64>() / values.len() as f64;
            AggregateResult::Number(avg)
        }
    }
}

/// Min aggregator
#[derive(Clone)]
pub struct Min {
    pub field: String,
}

impl Min {
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
        }
    }
}

impl Aggregation for Min {
    fn aggregate(&self, events: &[StreamEvent]) -> AggregateResult {
        events
            .iter()
            .filter_map(|e| e.get_numeric(&self.field))
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .map(AggregateResult::Number)
            .unwrap_or(AggregateResult::None)
    }
}

/// Max aggregator
#[derive(Clone)]
pub struct Max {
    pub field: String,
}

impl Max {
    pub fn new(field: impl Into<String>) -> Self {
        Self {
            field: field.into(),
        }
    }
}

impl Aggregation for Max {
    fn aggregate(&self, events: &[StreamEvent]) -> AggregateResult {
        events
            .iter()
            .filter_map(|e| e.get_numeric(&self.field))
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .map(AggregateResult::Number)
            .unwrap_or(AggregateResult::None)
    }
}

/// Custom aggregator using a closure
pub struct CustomAggregator<F>
where
    F: Fn(&[StreamEvent]) -> AggregateResult + Send + Sync,
{
    func: Arc<F>,
}

impl<F> CustomAggregator<F>
where
    F: Fn(&[StreamEvent]) -> AggregateResult + Send + Sync,
{
    pub fn new(func: F) -> Self {
        Self {
            func: Arc::new(func),
        }
    }
}

impl<F> Clone for CustomAggregator<F>
where
    F: Fn(&[StreamEvent]) -> AggregateResult + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            func: Arc::clone(&self.func),
        }
    }
}

impl<F> Aggregation for CustomAggregator<F>
where
    F: Fn(&[StreamEvent]) -> AggregateResult + Send + Sync,
{
    fn aggregate(&self, events: &[StreamEvent]) -> AggregateResult {
        (self.func)(events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;
    use std::collections::HashMap;

    fn create_test_events(count: usize) -> Vec<StreamEvent> {
        (0..count)
            .map(|i| {
                let mut data = HashMap::new();
                data.insert("value".to_string(), Value::Number(i as f64));
                data.insert("user_id".to_string(), Value::String(format!("user_{}", i % 3)));
                StreamEvent::new("TestEvent", data, "test")
            })
            .collect()
    }

    #[test]
    fn test_filter_operator() {
        let events = create_test_events(10);
        let stream = DataStream::from_events(events);

        let filtered = stream.filter(|e| e.get_numeric("value").unwrap_or(0.0) > 5.0);

        assert_eq!(filtered.len(), 4); // 6, 7, 8, 9
    }

    #[test]
    fn test_map_operator() {
        let events = create_test_events(5);
        let stream = DataStream::from_events(events);

        let mapped = stream.map(|mut e| {
            if let Some(value) = e.get_numeric("value") {
                e.data.insert("doubled".to_string(), Value::Number(value * 2.0));
            }
            e
        });

        let collected = mapped.collect();
        assert_eq!(collected[0].get_numeric("doubled"), Some(0.0));
        assert_eq!(collected[1].get_numeric("doubled"), Some(2.0));
    }

    #[test]
    fn test_key_by_operator() {
        let events = create_test_events(9);
        let stream = DataStream::from_events(events);

        let keyed = stream.key_by(|e| e.get_string("user_id").unwrap_or("").to_string());

        let counts = keyed.count();
        assert_eq!(counts.len(), 3); // 3 unique users
        assert_eq!(*counts.get("user_0").unwrap(), 3);
        assert_eq!(*counts.get("user_1").unwrap(), 3);
        assert_eq!(*counts.get("user_2").unwrap(), 3);
    }

    #[test]
    fn test_reduce_operator() {
        let events = create_test_events(5);
        let stream = DataStream::from_events(events);

        let result = stream.reduce(|mut acc, e| {
            let acc_val = acc.get_numeric("value").unwrap_or(0.0);
            let e_val = e.get_numeric("value").unwrap_or(0.0);
            acc.data.insert("value".to_string(), Value::Number(acc_val + e_val));
            acc
        });

        assert!(result.is_some());
        assert_eq!(result.unwrap().get_numeric("value"), Some(10.0)); // 0+1+2+3+4
    }

    #[test]
    fn test_count_aggregator() {
        let events = create_test_events(10);
        let result = Count.aggregate(&events);

        assert_eq!(result.as_number(), Some(10.0));
    }

    #[test]
    fn test_sum_aggregator() {
        let events = create_test_events(5);
        let result = Sum::new("value").aggregate(&events);

        assert_eq!(result.as_number(), Some(10.0)); // 0+1+2+3+4
    }

    #[test]
    fn test_average_aggregator() {
        let events = create_test_events(5);
        let result = Average::new("value").aggregate(&events);

        assert_eq!(result.as_number(), Some(2.0)); // (0+1+2+3+4)/5
    }

    #[test]
    fn test_group_by() {
        let events = create_test_events(9);
        let stream = DataStream::from_events(events);

        let grouped = stream.group_by(|e| e.get_string("user_id").unwrap_or("").to_string());

        let counts = grouped.count();
        assert_eq!(counts.len(), 3);
    }

    #[test]
    fn test_chaining_operators() {
        let events = create_test_events(20);
        let stream = DataStream::from_events(events);

        let result = stream
            .filter(|e| e.get_numeric("value").unwrap_or(0.0) >= 5.0)
            .map(|mut e| {
                if let Some(v) = e.get_numeric("value") {
                    e.data.insert("doubled".to_string(), Value::Number(v * 2.0));
                }
                e
            })
            .take(5)
            .collect();

        assert_eq!(result.len(), 5);
        assert_eq!(result[0].get_numeric("doubled"), Some(10.0)); // 5 * 2
    }

    #[test]
    fn test_windowed_stream() {
        let events = create_test_events(10);
        let stream = DataStream::from_events(events);

        let windowed = stream.window(WindowConfig::tumbling(Duration::from_secs(60)));

        assert!(!windowed.windows().is_empty());
    }
}

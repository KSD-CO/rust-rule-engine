//! Streaming Rule Engine
//!
//! Core engine for processing real-time event streams with rule evaluation.

use crate::engine::facts::Facts;
use crate::engine::knowledge_base::KnowledgeBase;
use crate::engine::RustRuleEngine;
use crate::parser::grl::GRLParser;
use crate::streaming::aggregator::StreamAnalytics;
use crate::streaming::event::StreamEvent;
use crate::streaming::window::{TimeWindow, WindowManager, WindowType};
use crate::types::Value;
use crate::{Result, RuleEngineError};

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;

/// Configuration for stream rule engine
#[derive(Debug, Clone)]
pub struct StreamConfig {
    /// Buffer size for incoming events
    pub buffer_size: usize,
    /// Window duration for aggregations
    pub window_duration: Duration,
    /// Maximum events per window
    pub max_events_per_window: usize,
    /// Maximum number of windows to keep
    pub max_windows: usize,
    /// Window type (sliding, tumbling, etc.)
    pub window_type: WindowType,
    /// Cache TTL for analytics
    pub analytics_cache_ttl_ms: u64,
    /// Processing interval for rule evaluation
    pub processing_interval: Duration,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            window_duration: Duration::from_secs(60),
            max_events_per_window: 1000,
            max_windows: 100,
            window_type: WindowType::Sliding,
            analytics_cache_ttl_ms: 30000,
            processing_interval: Duration::from_millis(100),
        }
    }
}

/// Result of stream rule execution
#[derive(Debug, Clone)]
pub struct StreamExecutionResult {
    /// Number of rules that fired
    pub rules_fired: usize,
    /// Number of events processed
    pub events_processed: usize,
    /// Processing duration
    pub processing_time_ms: u64,
    /// Triggered actions
    pub actions: Vec<StreamAction>,
    /// Analytics results
    pub analytics: HashMap<String, Value>,
}

/// Action triggered by stream rules
#[derive(Debug, Clone)]
pub struct StreamAction {
    /// Action type identifier
    pub action_type: String,
    /// Action parameters
    pub parameters: HashMap<String, Value>,
    /// Timestamp when action was triggered
    pub timestamp: u64,
    /// Rule that triggered this action
    pub rule_name: String,
}

/// Main streaming rule engine
pub struct StreamRuleEngine {
    /// Configuration
    config: StreamConfig,
    /// Regular rule engine for rule evaluation
    rule_engine: RustRuleEngine,
    /// Window manager for time-based processing
    window_manager: Arc<RwLock<WindowManager>>,
    /// Stream analytics
    analytics: Arc<RwLock<StreamAnalytics>>,
    /// Event sender
    event_sender: Option<mpsc::Sender<StreamEvent>>,
    /// Action callbacks
    action_handlers: Arc<RwLock<HashMap<String, Box<dyn Fn(&StreamAction) + Send + Sync>>>>,
    /// Running state
    is_running: Arc<RwLock<bool>>,
}

impl StreamRuleEngine {
    /// Create a new stream rule engine
    pub fn new() -> Self {
        let config = StreamConfig::default();
        let kb = KnowledgeBase::new("StreamKB");
        let rule_engine = RustRuleEngine::new(kb);

        let window_manager = Arc::new(RwLock::new(WindowManager::new(
            config.window_type.clone(),
            config.window_duration,
            config.max_events_per_window,
            config.max_windows,
        )));

        let analytics = Arc::new(RwLock::new(StreamAnalytics::new(
            config.analytics_cache_ttl_ms,
        )));

        Self {
            config,
            rule_engine,
            window_manager,
            analytics,
            event_sender: None,
            action_handlers: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: StreamConfig) -> Self {
        let kb = KnowledgeBase::new("StreamKB");
        let rule_engine = RustRuleEngine::new(kb);

        let window_manager = Arc::new(RwLock::new(WindowManager::new(
            config.window_type.clone(),
            config.window_duration,
            config.max_events_per_window,
            config.max_windows,
        )));

        let analytics = Arc::new(RwLock::new(StreamAnalytics::new(
            config.analytics_cache_ttl_ms,
        )));

        Self {
            config,
            rule_engine,
            window_manager,
            analytics,
            event_sender: None,
            action_handlers: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Add streaming rule from GRL string
    pub async fn add_rule(&mut self, grl_rule: &str) -> Result<()> {
        let rules = GRLParser::parse_rules(grl_rule)?;

        for rule in rules {
            self.rule_engine.knowledge_base_mut().add_rule(rule)?;
        }

        Ok(())
    }

    /// Add streaming rule from file
    pub async fn add_rule_file<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.add_rule(&content).await
    }

    /// Register action handler
    pub async fn register_action_handler<F>(&self, action_type: &str, handler: F)
    where
        F: Fn(&StreamAction) + Send + Sync + 'static,
    {
        let mut handlers = self.action_handlers.write().await;
        handlers.insert(action_type.to_string(), Box::new(handler));
    }

    /// Start the streaming engine
    pub async fn start(&mut self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<StreamEvent>(self.config.buffer_size);
        self.event_sender = Some(tx);

        // Set running state
        {
            let mut running = self.is_running.write().await;
            *running = true;
        }

        // Clone shared components for the processing task
        let window_manager = Arc::clone(&self.window_manager);
        let _analytics = Arc::clone(&self.analytics);
        let _action_handlers = Arc::clone(&self.action_handlers);
        let is_running = Arc::clone(&self.is_running);
        let processing_interval = self.config.processing_interval;

        // Start event processing task
        let _processing_task = tokio::spawn(async move {
            let mut interval_timer = interval(processing_interval);
            let mut event_batch = Vec::new();

            loop {
                tokio::select! {
                    // Process incoming events
                    event = rx.recv() => {
                        match event {
                            Some(event) => {
                                event_batch.push(event);

                                // Process batch when full or on timer
                                if event_batch.len() >= 100 {
                                    Self::process_event_batch(&window_manager, &event_batch).await;
                                    event_batch.clear();
                                }
                            }
                            None => break, // Channel closed
                        }
                    }

                    // Timer tick for processing
                    _ = interval_timer.tick() => {
                        if !event_batch.is_empty() {
                            Self::process_event_batch(&window_manager, &event_batch).await;
                            event_batch.clear();
                        }

                        // Check if still running
                        let running = is_running.read().await;
                        if !*running {
                            break;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the streaming engine
    pub async fn stop(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
    }

    /// Send event to stream for processing
    pub async fn send_event(&self, event: StreamEvent) -> Result<()> {
        if let Some(ref sender) = self.event_sender {
            sender.send(event).await.map_err(|_| {
                RuleEngineError::ExecutionError("Failed to send event to stream".to_string())
            })?;
        }
        Ok(())
    }

    /// Process a batch of events
    async fn process_event_batch(
        window_manager: &Arc<RwLock<WindowManager>>,
        events: &[StreamEvent],
    ) {
        let mut manager = window_manager.write().await;
        for event in events {
            manager.process_event(event.clone());
        }
    }

    /// Execute rules against current window state
    pub async fn execute_rules(&mut self) -> Result<StreamExecutionResult> {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let window_manager = self.window_manager.read().await;
        let _analytics = self.analytics.read().await;

        // Get current windows
        let windows = window_manager.active_windows();
        let mut total_events_processed = 0;
        let mut rules_fired = 0;
        let actions = Vec::new();
        let mut analytics_results = HashMap::new();

        // Process each window
        for window in windows {
            total_events_processed += window.count();

            // Create facts from window data
            let facts = Facts::new();

            // Add window aggregations to facts
            self.add_window_aggregations_to_facts(&facts, window)
                .await?;

            // Execute rules on this window
            let result = self.rule_engine.execute(&facts)?;
            rules_fired += result.rules_fired;

            // Note: Traditional rule engine doesn't return actions,
            // we'd need to extend it for streaming action capture
            // For now, we create empty actions list
        }

        // Calculate analytics
        if !windows.is_empty() {
            let latest_window = windows.last().unwrap();
            analytics_results.insert(
                "total_events".to_string(),
                Value::Number(total_events_processed as f64),
            );
            analytics_results.insert(
                "window_count".to_string(),
                Value::Number(windows.len() as f64),
            );
            analytics_results.insert(
                "latest_window_events".to_string(),
                Value::Number(latest_window.count() as f64),
            );
        }

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Ok(StreamExecutionResult {
            rules_fired,
            events_processed: total_events_processed,
            processing_time_ms: end_time - start_time,
            actions,
            analytics: analytics_results,
        })
    }

    /// Add window aggregations to facts
    async fn add_window_aggregations_to_facts(
        &self,
        facts: &Facts,
        window: &TimeWindow,
    ) -> Result<()> {
        // Add basic window stats
        facts.add_value("WindowEventCount", Value::Number(window.count() as f64))?;
        facts.add_value("WindowStartTime", Value::Number(window.start_time as f64))?;
        facts.add_value("WindowEndTime", Value::Number(window.end_time as f64))?;
        facts.add_value(
            "WindowDurationMs",
            Value::Number(window.duration_ms() as f64),
        )?;

        // Add common aggregations for numeric fields
        let numeric_fields = self.detect_numeric_fields(window);
        for field in numeric_fields {
            if let Some(sum) = window
                .events()
                .iter()
                .filter_map(|e| e.get_numeric(&field))
                .reduce(|a, b| a + b)
            {
                facts.add_value(&format!("{}Sum", field), Value::Number(sum))?;
            }

            if let Some(avg) = window.average(&field) {
                facts.add_value(&format!("{}Average", field), Value::Number(avg))?;
            }

            if let Some(min) = window.min(&field) {
                facts.add_value(&format!("{}Min", field), Value::Number(min))?;
            }

            if let Some(max) = window.max(&field) {
                facts.add_value(&format!("{}Max", field), Value::Number(max))?;
            }
        }

        Ok(())
    }

    /// Detect numeric fields in window events
    fn detect_numeric_fields(&self, window: &TimeWindow) -> Vec<String> {
        let mut fields = std::collections::HashSet::new();

        for event in window.events() {
            for (key, value) in &event.data {
                match value {
                    Value::Number(_) | Value::Integer(_) => {
                        fields.insert(key.clone());
                    }
                    _ => {}
                }
            }
        }

        fields.into_iter().collect()
    }

    /// Get current window statistics
    pub async fn get_window_statistics(&self) -> crate::streaming::window::WindowStatistics {
        let window_manager = self.window_manager.read().await;
        window_manager.get_statistics()
    }

    /// Get analytics for a specific field
    pub async fn get_field_analytics(&self, field: &str) -> HashMap<String, Value> {
        let window_manager = self.window_manager.read().await;
        let mut results = HashMap::new();

        let windows = window_manager.active_windows();
        if windows.is_empty() {
            return results;
        }

        // Calculate aggregations across all windows
        let total_sum: f64 = windows.iter().map(|w| w.sum(field)).sum();
        let total_count: usize = windows.iter().map(|w| w.count()).sum();

        results.insert("total_sum".to_string(), Value::Number(total_sum));
        results.insert("total_count".to_string(), Value::Number(total_count as f64));

        if total_count > 0 {
            results.insert(
                "overall_average".to_string(),
                Value::Number(total_sum / total_count as f64),
            );
        }

        // Get min/max across all windows
        let all_values: Vec<f64> = windows
            .iter()
            .flat_map(|w| w.events().iter().filter_map(|e| e.get_numeric(field)))
            .collect();

        if !all_values.is_empty() {
            let min = all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = all_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

            results.insert("global_min".to_string(), Value::Number(min));
            results.insert("global_max".to_string(), Value::Number(max));
        }

        results
    }

    /// Check if engine is running
    pub async fn is_running(&self) -> bool {
        let running = self.is_running.read().await;
        *running
    }
}

impl Default for StreamRuleEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Value;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_stream_engine_creation() {
        let engine = StreamRuleEngine::new();
        assert!(!engine.is_running().await);
    }

    #[tokio::test]
    async fn test_add_streaming_rule() {
        let mut engine = StreamRuleEngine::new();

        let rule = r#"
        rule "TestStreamRule" salience 10 {
            when
                WindowEventCount > 5
            then
                log("High event count detected");
        }
        "#;

        assert!(engine.add_rule(rule).await.is_ok());
    }

    #[tokio::test]
    async fn test_event_processing() {
        let mut engine = StreamRuleEngine::new();
        engine.start().await.unwrap();

        let mut data = HashMap::new();
        data.insert("value".to_string(), Value::Number(10.0));

        let event = StreamEvent::new("TestEvent", data, "test_source");
        assert!(engine.send_event(event).await.is_ok());

        engine.stop().await;
    }
}

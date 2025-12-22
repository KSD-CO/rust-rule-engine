//! Rule Streaming Engine - Real-time Event Processing
//!
//! This module provides real-time rule execution capabilities for streaming data,
//! including time-based windows, event aggregation, and continuous rule evaluation.
//!
//! ## Features
//!
//! - **🔄 Continuous Processing**: Non-stop rule evaluation on incoming events
//! - **⏰ Time Windows**: Sliding and tumbling window aggregations
//! - **📊 Stream Analytics**: Count, sum, average, min/max over time windows
//! - **🎯 Event Filtering**: Pattern matching and event correlation
//! - **⚡ High Throughput**: Async processing with backpressure handling
//!
//! ## Example
//!

#![allow(missing_docs)]
//! ```rust,ignore
//! use rust_rule_engine::streaming::*;
//!
//! let mut stream_engine = StreamRuleEngine::new()
//!     .with_window_size(Duration::from_secs(60))
//!     .with_buffer_size(1000);
//!
//! // Define streaming rule
//! let rule = r#"
//! rule "HighFrequencyTrading" {
//!     when
//!         stream(TradeEvent, 5s).count() > 100
//!     then
//!         AlertService.trigger("High frequency detected");
//! }
//! "#;
//!
//! stream_engine.add_rule(rule).await?;
//! stream_engine.start().await?;
//! ```

#[cfg(feature = "streaming")]
pub mod aggregator;
#[cfg(feature = "streaming")]
pub mod engine;
#[cfg(feature = "streaming")]
pub mod event;
#[cfg(feature = "streaming")]
pub mod join_manager;
#[cfg(feature = "streaming")]
pub mod join_optimizer;
#[cfg(feature = "streaming")]
pub mod operators;
#[cfg(feature = "streaming")]
pub mod state;
#[cfg(feature = "streaming")]
pub mod watermark;
#[cfg(feature = "streaming")]
pub mod window;

#[cfg(feature = "streaming")]
pub use aggregator::{AggregationType, Aggregator};
#[cfg(feature = "streaming")]
pub use engine::StreamRuleEngine;
#[cfg(feature = "streaming")]
pub use event::{EventMetadata, StreamEvent};
#[cfg(feature = "streaming")]
pub use join_manager::StreamJoinManager;
#[cfg(feature = "streaming")]
pub use join_optimizer::{JoinOptimization, JoinOptimizer, OptimizedJoinPlan, StreamStats};
#[cfg(feature = "streaming")]
pub use operators::{
    AggregateResult, Aggregation, Average, Count, CustomAggregator, DataStream, GroupedStream,
    KeyedStream, Max, Min, Sum, WindowConfig, WindowedStream,
};
#[cfg(feature = "streaming")]
pub use state::{
    CheckpointMetadata, StateBackend, StateConfig, StateStatistics, StateStore, StatefulOperator,
};
#[cfg(feature = "streaming")]
pub use watermark::{
    LateDataHandler, LateDataStats, LateDataStrategy, LateEventDecision, Watermark,
    WatermarkGenerator, WatermarkStrategy, WatermarkedStream,
};
#[cfg(feature = "streaming")]
pub use window::{TimeWindow, WindowManager, WindowType};

/// Re-export for non-streaming builds
#[cfg(not(feature = "streaming"))]
pub struct StreamRuleEngine;

#[cfg(not(feature = "streaming"))]
impl StreamRuleEngine {
    /// Create a new stream rule engine (non-streaming placeholder)
    pub fn new() -> Self {
        StreamRuleEngine
    }

    /// Create with custom configuration (requires streaming feature)
    pub fn with_config(_config: StreamConfig) -> Self {
        panic!("StreamRuleEngine configuration methods require the 'streaming' feature to be enabled. Enable it in Cargo.toml: features = [\"streaming\"]");
    }

    /// Add streaming rule from GRL string (requires streaming feature)
    pub async fn add_rule(&mut self, _grl_rule: &str) -> Result<()> {
        Err(crate::RuleEngineError::FeatureNotEnabled {
            feature: "streaming".to_string(),
            message: "Streaming rule engine requires the 'streaming' feature to be enabled"
                .to_string(),
        })
    }

    /// Add streaming rule from file (requires streaming feature)
    pub async fn add_rule_file<P: AsRef<std::path::Path>>(&mut self, _path: P) -> Result<()> {
        Err(crate::RuleEngineError::FeatureNotEnabled {
            feature: "streaming".to_string(),
            message: "Streaming rule engine requires the 'streaming' feature to be enabled"
                .to_string(),
        })
    }

    /// Register action handler (requires streaming feature)
    pub async fn register_action_handler<F>(&self, _action_type: &str, _handler: F)
    where
        F: Fn(&StreamAction) + Send + Sync + 'static,
    {
        panic!("StreamRuleEngine action handlers require the 'streaming' feature to be enabled. Enable it in Cargo.toml: features = [\"streaming\"]");
    }

    /// Start the streaming engine (requires streaming feature)
    pub async fn start(&mut self) -> Result<()> {
        Err(crate::RuleEngineError::FeatureNotEnabled {
            feature: "streaming".to_string(),
            message: "Streaming rule engine requires the 'streaming' feature to be enabled"
                .to_string(),
        })
    }

    /// Stop the streaming engine (requires streaming feature)
    pub async fn stop(&self) {
        panic!("StreamRuleEngine stop method requires the 'streaming' feature to be enabled. Enable it in Cargo.toml: features = [\"streaming\"]");
    }

    /// Send event to streaming engine (requires streaming feature)
    pub async fn send_event(&self, _event: StreamEvent) -> Result<()> {
        Err(crate::RuleEngineError::FeatureNotEnabled {
            feature: "streaming".to_string(),
            message: "Streaming rule engine requires the 'streaming' feature to be enabled"
                .to_string(),
        })
    }

    /// Execute rules manually (requires streaming feature)
    pub async fn execute_rules(&mut self) -> Result<StreamExecutionResult> {
        Err(crate::RuleEngineError::FeatureNotEnabled {
            feature: "streaming".to_string(),
            message: "Streaming rule engine requires the 'streaming' feature to be enabled"
                .to_string(),
        })
    }

    /// Get window statistics (requires streaming feature)
    pub async fn get_window_statistics(&self) -> crate::streaming::window::WindowStatistics {
        panic!("StreamRuleEngine window statistics require the 'streaming' feature to be enabled. Enable it in Cargo.toml: features = [\"streaming\"]");
    }

    /// Get field analytics (requires streaming feature)
    pub async fn get_field_analytics(
        &self,
        _field: &str,
    ) -> std::collections::HashMap<String, crate::types::Value> {
        panic!("StreamRuleEngine field analytics require the 'streaming' feature to be enabled. Enable it in Cargo.toml: features = [\"streaming\"]");
    }

    /// Check if engine is running (requires streaming feature)
    pub async fn is_running(&self) -> bool {
        panic!("StreamRuleEngine running status requires the 'streaming' feature to be enabled. Enable it in Cargo.toml: features = [\"streaming\"]");
    }
}

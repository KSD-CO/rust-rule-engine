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
pub mod window;

#[cfg(feature = "streaming")]
pub use aggregator::{AggregationType, Aggregator};
#[cfg(feature = "streaming")]
pub use engine::StreamRuleEngine;
#[cfg(feature = "streaming")]
pub use event::{EventMetadata, StreamEvent};
#[cfg(feature = "streaming")]
pub use window::{TimeWindow, WindowManager, WindowType};

/// Re-export for non-streaming builds
#[cfg(not(feature = "streaming"))]
pub struct StreamRuleEngine;

#[cfg(not(feature = "streaming"))]
impl StreamRuleEngine {
    /// Placeholder for non-streaming builds
    pub fn new() -> Self {
        StreamRuleEngine
    }
}

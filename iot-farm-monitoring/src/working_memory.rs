// Working Memory - Central fact storage for RETE network
//
// In classic RETE architecture, Working Memory is a separate layer between
// Alpha nodes (filters) and Beta nodes (joins). It stores all facts that have
// passed through Alpha node filters and are available for pattern matching.
//
// NOTE: This is a facade/wrapper implementation. The actual storage is inside
// StreamAlphaNode buffers for performance reasons, but this class provides
// the conceptual RETE architecture interface.

use rust_rule_engine::streaming::event::StreamEvent;
use rust_rule_engine::rete::stream_alpha_node::StreamAlphaNode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Working Memory stores filtered facts from Alpha nodes
/// 
/// This is the central storage layer in RETE architecture:
/// - Alpha nodes write filtered events here (via their internal buffers)
/// - Beta nodes read from here to perform joins
/// - Time-windowed: automatically expires old facts
/// - Indexed by stream name for fast access
/// 
/// **Implementation Note**: This is a facade over Alpha node buffers.
/// The actual VecDeque<StreamEvent> storage is inside each StreamAlphaNode
/// for performance optimization, but this interface maintains RETE architecture clarity.
pub struct WorkingMemory {
    /// References to Alpha nodes (which hold the actual fact buffers)
    alpha_nodes: HashMap<String, Arc<Mutex<StreamAlphaNode>>>,
}

impl WorkingMemory {
    /// Create a new empty Working Memory
    pub fn new() -> Self {
        Self {
            alpha_nodes: HashMap::new(),
        }
    }

    /// Register an Alpha node (which holds facts for a stream)
    pub fn register_alpha_node(&mut self, stream_name: String, alpha_node: Arc<Mutex<StreamAlphaNode>>) {
        self.alpha_nodes.insert(stream_name, alpha_node);
    }

    /// Get all facts for a stream (reads from Alpha node buffer)
    /// 
    /// This is called by Beta nodes when performing joins
    pub fn get_facts(&self, stream_name: &str) -> Vec<StreamEvent> {
        self.alpha_nodes.get(stream_name)
            .map(|node| {
                let n = node.lock().unwrap();
                n.get_events().iter().cloned().collect()
            })
            .unwrap_or_default()
    }

    /// Get facts count for a stream
    pub fn fact_count(&self, stream_name: &str) -> usize {
        self.alpha_nodes.get(stream_name)
            .map(|node| {
                let n = node.lock().unwrap();
                n.event_count()
            })
            .unwrap_or(0)
    }

    /// Clear all facts from a specific stream (via its Alpha node)
    pub fn clear_stream(&self, stream_name: &str) {
        if let Some(node) = self.alpha_nodes.get(stream_name) {
            let mut n = node.lock().unwrap();
            n.clear();
        }
    }

    /// Clear all facts from Working Memory (clears all Alpha node buffers)
    pub fn clear_all(&self) {
        for node in self.alpha_nodes.values() {
            let mut n = node.lock().unwrap();
            n.clear();
        }
    }

    /// Get statistics about Working Memory
    pub fn stats(&self) -> WorkingMemoryStats {
        let mut total_facts = 0;
        let mut stream_stats = HashMap::new();

        for (stream_name, node) in &self.alpha_nodes {
            let n = node.lock().unwrap();
            let count = n.event_count();
            total_facts += count;
            stream_stats.insert(stream_name.clone(), count);
        }

        WorkingMemoryStats {
            total_facts,
            stream_count: self.alpha_nodes.len(),
            stream_stats,
        }
    }

    /// Get list of all registered streams
    pub fn streams(&self) -> Vec<String> {
        self.alpha_nodes.keys().cloned().collect()
    }
}

impl Default for WorkingMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about Working Memory state
#[derive(Debug, Clone)]
pub struct WorkingMemoryStats {
    pub total_facts: usize,
    pub stream_count: usize,
    pub stream_stats: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rule_engine::rete::stream_alpha_node::{StreamAlphaNode, WindowSpec};
    use rust_rule_engine::streaming::window::WindowType;
    use rust_rule_engine::types::Value;
    use std::collections::HashMap as StdHashMap;

    fn create_test_event(stream: &str, timestamp: u64) -> StreamEvent {
        let mut data = StdHashMap::new();
        data.insert("zone_id".to_string(), Value::String("zone_1".to_string()));
        StreamEvent::with_timestamp("TestEvent", data, stream, timestamp)
    }

    #[test]
    fn test_working_memory_basic() {
        let mut wm = WorkingMemory::new();
        let alpha = Arc::new(Mutex::new(StreamAlphaNode::new("test-stream", None, None)));
        wm.register_alpha_node("test-stream".to_string(), alpha.clone());

        // Add event through Alpha node
        let event = create_test_event("test-stream", 1000);
        {
            let mut a = alpha.lock().unwrap();
            a.process_event(&event);
        }

        assert_eq!(wm.fact_count("test-stream"), 1);
        
        let facts = wm.get_facts("test-stream");
        assert_eq!(facts.len(), 1);
    }

    #[test]
    fn test_working_memory_stats() {
        let mut wm = WorkingMemory::new();
        
        let alpha1 = Arc::new(Mutex::new(StreamAlphaNode::new("stream1", None, None)));
        let alpha2 = Arc::new(Mutex::new(StreamAlphaNode::new("stream2", None, None)));
        
        wm.register_alpha_node("stream1".to_string(), alpha1.clone());
        wm.register_alpha_node("stream2".to_string(), alpha2.clone());

        // Add events
        {
            let mut a1 = alpha1.lock().unwrap();
            a1.process_event(&create_test_event("stream1", 1000));
            a1.process_event(&create_test_event("stream1", 2000));
        }
        {
            let mut a2 = alpha2.lock().unwrap();
            a2.process_event(&create_test_event("stream2", 3000));
        }

        let stats = wm.stats();
        assert_eq!(stats.total_facts, 3);
        assert_eq!(stats.stream_count, 2);
        assert_eq!(stats.stream_stats.get("stream1"), Some(&2));
        assert_eq!(stats.stream_stats.get("stream2"), Some(&1));
    }

    #[test]
    fn test_working_memory_clear() {
        let mut wm = WorkingMemory::new();
        let alpha = Arc::new(Mutex::new(StreamAlphaNode::new("test-stream", None, None)));
        wm.register_alpha_node("test-stream".to_string(), alpha.clone());

        // Add events
        {
            let mut a = alpha.lock().unwrap();
            a.process_event(&create_test_event("test-stream", 1000));
            a.process_event(&create_test_event("test-stream", 2000));
        }

        assert_eq!(wm.fact_count("test-stream"), 2);

        // Clear
        wm.clear_stream("test-stream");
        assert_eq!(wm.fact_count("test-stream"), 0);
    }
}

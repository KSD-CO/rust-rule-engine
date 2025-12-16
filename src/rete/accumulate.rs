//! Accumulate Functions for RETE-UL Engine
//!
//! This module implements Drools/CLIPS-style accumulate functions for aggregating
//! data across multiple facts in rule conditions.
//!
//! # Examples
//!
//! ```grl
//! rule "TotalSales" {
//!     when
//!         $total: accumulate(
//!             Order($amount: amount, status == "completed"),
//!             sum($amount)
//!         )
//!         $total > 10000
//!     then
//!         Report.highValue = true;
//! }
//! ```

use super::facts::FactValue;
use std::collections::HashMap;

/// Accumulate function trait - defines how to aggregate values
pub trait AccumulateFunction: Send + Sync {
    /// Initialize the accumulator
    fn init(&self) -> Box<dyn AccumulateState>;

    /// Get the function name
    fn name(&self) -> &str;

    /// Clone the function
    fn clone_box(&self) -> Box<dyn AccumulateFunction>;
}

/// State maintained during accumulation
pub trait AccumulateState: Send {
    /// Accumulate a new value
    fn accumulate(&mut self, value: &FactValue);

    /// Get the final result
    fn get_result(&self) -> FactValue;

    /// Reset the state
    fn reset(&mut self);

    /// Clone the state
    fn clone_box(&self) -> Box<dyn AccumulateState>;
}

// ============================================================================
// Built-in Accumulate Functions
// ============================================================================

/// Sum accumulator - adds up numeric values
#[derive(Debug, Clone)]
pub struct SumFunction;

impl AccumulateFunction for SumFunction {
    fn init(&self) -> Box<dyn AccumulateState> {
        Box::new(SumState { total: 0.0 })
    }

    fn name(&self) -> &str {
        "sum"
    }

    fn clone_box(&self) -> Box<dyn AccumulateFunction> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct SumState {
    total: f64,
}

impl AccumulateState for SumState {
    fn accumulate(&mut self, value: &FactValue) {
        match value {
            FactValue::Integer(i) => self.total += *i as f64,
            FactValue::Float(f) => self.total += f,
            _ => {} // Ignore non-numeric values
        }
    }

    fn get_result(&self) -> FactValue {
        FactValue::Float(self.total)
    }

    fn reset(&mut self) {
        self.total = 0.0;
    }

    fn clone_box(&self) -> Box<dyn AccumulateState> {
        Box::new(self.clone())
    }
}

/// Count accumulator - counts number of matching facts
#[derive(Debug, Clone)]
pub struct CountFunction;

impl AccumulateFunction for CountFunction {
    fn init(&self) -> Box<dyn AccumulateState> {
        Box::new(CountState { count: 0 })
    }

    fn name(&self) -> &str {
        "count"
    }

    fn clone_box(&self) -> Box<dyn AccumulateFunction> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct CountState {
    count: i64,
}

impl AccumulateState for CountState {
    fn accumulate(&mut self, _value: &FactValue) {
        self.count += 1;
    }

    fn get_result(&self) -> FactValue {
        FactValue::Integer(self.count)
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn clone_box(&self) -> Box<dyn AccumulateState> {
        Box::new(self.clone())
    }
}

/// Average accumulator - calculates mean of numeric values
#[derive(Debug, Clone)]
pub struct AverageFunction;

impl AccumulateFunction for AverageFunction {
    fn init(&self) -> Box<dyn AccumulateState> {
        Box::new(AverageState { sum: 0.0, count: 0 })
    }

    fn name(&self) -> &str {
        "average"
    }

    fn clone_box(&self) -> Box<dyn AccumulateFunction> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct AverageState {
    sum: f64,
    count: usize,
}

impl AccumulateState for AverageState {
    fn accumulate(&mut self, value: &FactValue) {
        match value {
            FactValue::Integer(i) => {
                self.sum += *i as f64;
                self.count += 1;
            }
            FactValue::Float(f) => {
                self.sum += f;
                self.count += 1;
            }
            _ => {} // Ignore non-numeric values
        }
    }

    fn get_result(&self) -> FactValue {
        if self.count == 0 {
            FactValue::Float(0.0)
        } else {
            FactValue::Float(self.sum / self.count as f64)
        }
    }

    fn reset(&mut self) {
        self.sum = 0.0;
        self.count = 0;
    }

    fn clone_box(&self) -> Box<dyn AccumulateState> {
        Box::new(self.clone())
    }
}

/// Minimum accumulator - finds minimum numeric value
#[derive(Debug, Clone)]
pub struct MinFunction;

impl AccumulateFunction for MinFunction {
    fn init(&self) -> Box<dyn AccumulateState> {
        Box::new(MinState { min: None })
    }

    fn name(&self) -> &str {
        "min"
    }

    fn clone_box(&self) -> Box<dyn AccumulateFunction> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct MinState {
    min: Option<f64>,
}

impl AccumulateState for MinState {
    fn accumulate(&mut self, value: &FactValue) {
        let num = match value {
            FactValue::Integer(i) => Some(*i as f64),
            FactValue::Float(f) => Some(*f),
            _ => None,
        };

        if let Some(n) = num {
            self.min = Some(match self.min {
                Some(current) => current.min(n),
                None => n,
            });
        }
    }

    fn get_result(&self) -> FactValue {
        match self.min {
            Some(m) => FactValue::Float(m),
            None => FactValue::Float(0.0),
        }
    }

    fn reset(&mut self) {
        self.min = None;
    }

    fn clone_box(&self) -> Box<dyn AccumulateState> {
        Box::new(self.clone())
    }
}

/// Maximum accumulator - finds maximum numeric value
#[derive(Debug, Clone)]
pub struct MaxFunction;

impl AccumulateFunction for MaxFunction {
    fn init(&self) -> Box<dyn AccumulateState> {
        Box::new(MaxState { max: None })
    }

    fn name(&self) -> &str {
        "max"
    }

    fn clone_box(&self) -> Box<dyn AccumulateFunction> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone)]
struct MaxState {
    max: Option<f64>,
}

impl AccumulateState for MaxState {
    fn accumulate(&mut self, value: &FactValue) {
        let num = match value {
            FactValue::Integer(i) => Some(*i as f64),
            FactValue::Float(f) => Some(*f),
            _ => None,
        };

        if let Some(n) = num {
            self.max = Some(match self.max {
                Some(current) => current.max(n),
                None => n,
            });
        }
    }

    fn get_result(&self) -> FactValue {
        match self.max {
            Some(m) => FactValue::Float(m),
            None => FactValue::Float(0.0),
        }
    }

    fn reset(&mut self) {
        self.max = None;
    }

    fn clone_box(&self) -> Box<dyn AccumulateState> {
        Box::new(self.clone())
    }
}

// ============================================================================
// Accumulate Pattern - for use in RETE conditions
// ============================================================================

/// Accumulate pattern in a rule condition
pub struct AccumulatePattern {
    /// Variable to bind the result to (e.g., "$total")
    pub result_var: String,

    /// Source pattern to match facts (e.g., "Order")
    pub source_pattern: String,

    /// Field to extract from matching facts (e.g., "amount")
    pub extract_field: String,

    /// Conditions on the source pattern (e.g., "status == 'completed'")
    pub source_conditions: Vec<String>,

    /// Accumulate function to apply (sum, avg, count, etc.)
    pub function: Box<dyn AccumulateFunction>,
}

impl Clone for AccumulatePattern {
    fn clone(&self) -> Self {
        Self {
            result_var: self.result_var.clone(),
            source_pattern: self.source_pattern.clone(),
            extract_field: self.extract_field.clone(),
            source_conditions: self.source_conditions.clone(),
            function: self.function.clone_box(),
        }
    }
}

impl std::fmt::Debug for AccumulatePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccumulatePattern")
            .field("result_var", &self.result_var)
            .field("source_pattern", &self.source_pattern)
            .field("extract_field", &self.extract_field)
            .field("source_conditions", &self.source_conditions)
            .field("function", &self.function.name())
            .finish()
    }
}

impl AccumulatePattern {
    /// Create a new accumulate pattern
    pub fn new(
        result_var: String,
        source_pattern: String,
        extract_field: String,
        function: Box<dyn AccumulateFunction>,
    ) -> Self {
        Self {
            result_var,
            source_pattern,
            extract_field,
            source_conditions: Vec::new(),
            function,
        }
    }

    /// Add a condition to the source pattern
    pub fn with_condition(mut self, condition: String) -> Self {
        self.source_conditions.push(condition);
        self
    }
}

// ============================================================================
// Accumulate Function Registry
// ============================================================================

/// Registry of available accumulate functions
pub struct AccumulateFunctionRegistry {
    functions: HashMap<String, Box<dyn AccumulateFunction>>,
}

impl AccumulateFunctionRegistry {
    /// Create a new registry with built-in functions
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };

        // Register built-in functions
        registry.register(Box::new(SumFunction));
        registry.register(Box::new(CountFunction));
        registry.register(Box::new(AverageFunction));
        registry.register(Box::new(MinFunction));
        registry.register(Box::new(MaxFunction));

        registry
    }

    /// Register a custom accumulate function
    pub fn register(&mut self, function: Box<dyn AccumulateFunction>) {
        self.functions.insert(function.name().to_string(), function);
    }

    /// Get a function by name
    pub fn get(&self, name: &str) -> Option<Box<dyn AccumulateFunction>> {
        self.functions.get(name).map(|f| f.clone_box())
    }

    /// Get all available function names
    pub fn available_functions(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

impl Default for AccumulateFunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum_function() {
        let sum = SumFunction;
        let mut state = sum.init();

        state.accumulate(&FactValue::Integer(10));
        state.accumulate(&FactValue::Integer(20));
        state.accumulate(&FactValue::Float(15.5));

        match state.get_result() {
            FactValue::Float(f) => assert_eq!(f, 45.5),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_count_function() {
        let count = CountFunction;
        let mut state = count.init();

        state.accumulate(&FactValue::Integer(10));
        state.accumulate(&FactValue::String("test".to_string()));
        state.accumulate(&FactValue::Boolean(true));

        match state.get_result() {
            FactValue::Integer(i) => assert_eq!(i, 3),
            _ => panic!("Expected Integer"),
        }
    }

    #[test]
    fn test_average_function() {
        let avg = AverageFunction;
        let mut state = avg.init();

        state.accumulate(&FactValue::Integer(10));
        state.accumulate(&FactValue::Integer(20));
        state.accumulate(&FactValue::Integer(30));

        match state.get_result() {
            FactValue::Float(f) => assert_eq!(f, 20.0),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_min_max_functions() {
        let min = MinFunction;
        let max = MaxFunction;

        let mut min_state = min.init();
        let mut max_state = max.init();

        for value in &[
            FactValue::Integer(15),
            FactValue::Integer(5),
            FactValue::Integer(25),
        ] {
            min_state.accumulate(value);
            max_state.accumulate(value);
        }

        match min_state.get_result() {
            FactValue::Float(f) => assert_eq!(f, 5.0),
            _ => panic!("Expected Float"),
        }

        match max_state.get_result() {
            FactValue::Float(f) => assert_eq!(f, 25.0),
            _ => panic!("Expected Float"),
        }
    }

    #[test]
    fn test_registry() {
        let registry = AccumulateFunctionRegistry::new();

        assert!(registry.get("sum").is_some());
        assert!(registry.get("count").is_some());
        assert!(registry.get("average").is_some());
        assert!(registry.get("min").is_some());
        assert!(registry.get("max").is_some());
        assert!(registry.get("unknown").is_none());

        let functions = registry.available_functions();
        assert_eq!(functions.len(), 5);
    }
}

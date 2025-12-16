//! Pattern Matching with Variable Binding (P3 Feature - Advanced)
//!
//! This module implements Drools-style pattern matching with:
//! - Variable binding ($var)
//! - Multi-object patterns
//! - Join conditions between patterns
//! - Field constraints with variables

use super::facts::{FactValue, TypedFacts};
use super::multifield::MultifieldOp;
use super::working_memory::{FactHandle, WorkingMemory};
use std::collections::HashMap;

/// Variable name (e.g., "$name", "$age")
pub type Variable = String;

/// Pattern constraint with optional variable binding
#[derive(Debug, Clone)]
pub enum PatternConstraint {
    /// Simple constraint: field op value
    Simple {
        field: String,
        operator: String,
        value: FactValue,
    },
    /// Binding constraint: field = $var (binds value to variable)
    Binding { field: String, variable: Variable },
    /// Variable constraint: field op $var (compare with bound variable)
    Variable {
        field: String,
        operator: String,
        variable: Variable,
    },
    /// Multi-field constraint: pattern matching on arrays/collections
    ///
    /// Examples:
    /// - `Order.items $?all_items` - Collect all items (Collect)
    /// - `Product.tags contains "electronics"` - Check containment (Contains)
    /// - `Order.items count > 0` - Get array length (Count)
    MultiField {
        field: String,
        variable: Option<Variable>, // $?var for multi-field binding
        operator: MultifieldOp,
        value: Option<FactValue>, // For operations like Contains
    },
}

impl PatternConstraint {
    /// Create simple constraint
    pub fn simple(field: String, operator: String, value: FactValue) -> Self {
        Self::Simple {
            field,
            operator,
            value,
        }
    }

    /// Create binding constraint
    pub fn binding(field: String, variable: Variable) -> Self {
        Self::Binding { field, variable }
    }

    /// Create variable constraint
    pub fn variable(field: String, operator: String, variable: Variable) -> Self {
        Self::Variable {
            field,
            operator,
            variable,
        }
    }

    /// Create multi-field constraint
    pub fn multifield(
        field: String,
        operator: MultifieldOp,
        variable: Option<Variable>,
        value: Option<FactValue>,
    ) -> Self {
        Self::MultiField {
            field,
            operator,
            variable,
            value,
        }
    }

    /// Evaluate constraint against facts and bindings
    pub fn evaluate(
        &self,
        facts: &TypedFacts,
        bindings: &HashMap<Variable, FactValue>,
    ) -> Option<HashMap<Variable, FactValue>> {
        match self {
            PatternConstraint::Simple {
                field,
                operator,
                value,
            } => {
                if facts.evaluate_condition(field, operator, value) {
                    Some(HashMap::new())
                } else {
                    None
                }
            }
            PatternConstraint::Binding { field, variable } => {
                if let Some(fact_value) = facts.get(field) {
                    let mut new_bindings = HashMap::new();
                    new_bindings.insert(variable.clone(), fact_value.clone());
                    Some(new_bindings)
                } else {
                    None
                }
            }
            PatternConstraint::Variable {
                field,
                operator,
                variable,
            } => {
                if let Some(bound_value) = bindings.get(variable) {
                    if facts.evaluate_condition(field, operator, bound_value) {
                        Some(HashMap::new())
                    } else {
                        None
                    }
                } else {
                    None // Variable not bound yet
                }
            }
            PatternConstraint::MultiField {
                field,
                operator,
                variable,
                value,
            } => {
                // Delegate to multifield evaluation helper
                super::multifield::evaluate_multifield_pattern(
                    facts,
                    field,
                    operator,
                    variable.as_deref(),
                    value.as_ref(),
                    bindings,
                )
            }
        }
    }
}

/// A pattern matches facts of a specific type with constraints
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Fact type (e.g., "Person", "Order")
    pub fact_type: String,
    /// List of constraints
    pub constraints: Vec<PatternConstraint>,
    /// Optional pattern name for reference
    pub name: Option<String>,
}

impl Pattern {
    /// Create new pattern
    pub fn new(fact_type: String) -> Self {
        Self {
            fact_type,
            constraints: Vec::new(),
            name: None,
        }
    }

    /// Add constraint
    pub fn with_constraint(mut self, constraint: PatternConstraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Set pattern name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Match this pattern against a fact
    pub fn matches(
        &self,
        facts: &TypedFacts,
        bindings: &HashMap<Variable, FactValue>,
    ) -> Option<HashMap<Variable, FactValue>> {
        let mut new_bindings = bindings.clone();

        for constraint in &self.constraints {
            match constraint.evaluate(facts, &new_bindings) {
                Some(additional_bindings) => {
                    new_bindings.extend(additional_bindings);
                }
                None => return None,
            }
        }

        Some(new_bindings)
    }

    /// Match pattern against working memory
    pub fn match_in_working_memory(
        &self,
        wm: &WorkingMemory,
        bindings: &HashMap<Variable, FactValue>,
    ) -> Vec<(FactHandle, HashMap<Variable, FactValue>)> {
        let mut results = Vec::new();

        for fact in wm.get_by_type(&self.fact_type) {
            if let Some(new_bindings) = self.matches(&fact.data, bindings) {
                results.push((fact.handle, new_bindings));
            }
        }

        results
    }
}

/// Multi-pattern rule with joins
#[derive(Debug, Clone)]
pub struct MultiPattern {
    /// List of patterns that must all match
    pub patterns: Vec<Pattern>,
    /// Rule name
    pub name: String,
}

impl MultiPattern {
    /// Create new multi-pattern rule
    pub fn new(name: String) -> Self {
        Self {
            patterns: Vec::new(),
            name,
        }
    }

    /// Add pattern
    pub fn with_pattern(mut self, pattern: Pattern) -> Self {
        self.patterns.push(pattern);
        self
    }

    /// Match all patterns (with variable binding across patterns)
    pub fn match_all(
        &self,
        wm: &WorkingMemory,
    ) -> Vec<(Vec<FactHandle>, HashMap<Variable, FactValue>)> {
        if self.patterns.is_empty() {
            return Vec::new();
        }

        // Start with first pattern
        let mut results = Vec::new();
        let first_pattern = &self.patterns[0];
        let empty_bindings = HashMap::new();

        for (handle, bindings) in first_pattern.match_in_working_memory(wm, &empty_bindings) {
            results.push((vec![handle], bindings));
        }

        // Join with remaining patterns
        for pattern in &self.patterns[1..] {
            let mut new_results = Vec::new();

            for (handles, bindings) in results {
                for (handle, new_bindings) in pattern.match_in_working_memory(wm, &bindings) {
                    let mut combined_handles = handles.clone();
                    combined_handles.push(handle);
                    new_results.push((combined_handles, new_bindings));
                }
            }

            results = new_results;

            if results.is_empty() {
                break; // No matches, stop early
            }
        }

        results
    }
}

/// Pattern builder for easier construction (Drools-style DSL)
pub struct PatternBuilder {
    pattern: Pattern,
}

impl PatternBuilder {
    /// Start building pattern for a fact type
    pub fn for_type(fact_type: impl Into<String>) -> Self {
        Self {
            pattern: Pattern::new(fact_type.into()),
        }
    }

    /// Add simple constraint (field op value)
    pub fn where_field(
        mut self,
        field: impl Into<String>,
        operator: impl Into<String>,
        value: FactValue,
    ) -> Self {
        self.pattern.constraints.push(PatternConstraint::Simple {
            field: field.into(),
            operator: operator.into(),
            value,
        });
        self
    }

    /// Bind field to variable ($var)
    pub fn bind(mut self, field: impl Into<String>, variable: impl Into<String>) -> Self {
        self.pattern.constraints.push(PatternConstraint::Binding {
            field: field.into(),
            variable: variable.into(),
        });
        self
    }

    /// Compare field with variable ($var)
    pub fn where_var(
        mut self,
        field: impl Into<String>,
        operator: impl Into<String>,
        variable: impl Into<String>,
    ) -> Self {
        self.pattern.constraints.push(PatternConstraint::Variable {
            field: field.into(),
            operator: operator.into(),
            variable: variable.into(),
        });
        self
    }

    /// Set pattern name
    pub fn named(mut self, name: impl Into<String>) -> Self {
        self.pattern.name = Some(name.into());
        self
    }

    /// Build the pattern
    pub fn build(self) -> Pattern {
        self.pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pattern() {
        let pattern = PatternBuilder::for_type("Person")
            .where_field("age", ">", FactValue::Integer(18))
            .where_field("status", "==", FactValue::String("active".to_string()))
            .build();

        let mut facts = TypedFacts::new();
        facts.set("age", 25i64);
        facts.set("status", "active");

        let bindings = HashMap::new();
        let result = pattern.matches(&facts, &bindings);
        assert!(result.is_some());
    }

    #[test]
    fn test_variable_binding() {
        let pattern = PatternBuilder::for_type("Person")
            .bind("name", "$personName")
            .bind("age", "$personAge")
            .build();

        let mut facts = TypedFacts::new();
        facts.set("name", "John");
        facts.set("age", 25i64);

        let bindings = HashMap::new();
        let result = pattern.matches(&facts, &bindings).unwrap();

        assert_eq!(result.get("$personName").unwrap().as_string(), "John");
        assert_eq!(result.get("$personAge").unwrap().as_integer(), Some(25));
    }

    #[test]
    fn test_variable_constraint() {
        // First bind $minAge
        let mut bindings = HashMap::new();
        bindings.insert("$minAge".to_string(), FactValue::Integer(18));

        // Then use $minAge in constraint
        let pattern = PatternBuilder::for_type("Person")
            .where_var("age", ">=", "$minAge")
            .build();

        let mut facts = TypedFacts::new();
        facts.set("age", 25i64);

        let result = pattern.matches(&facts, &bindings);
        assert!(result.is_some());
    }

    #[test]
    fn test_multi_pattern_join() {
        let mut wm = WorkingMemory::new();

        // Insert Person
        let mut person = TypedFacts::new();
        person.set("name", "John");
        person.set("age", 25i64);
        wm.insert("Person".to_string(), person);

        // Insert Order for John
        let mut order = TypedFacts::new();
        order.set("customer", "John");
        order.set("amount", 1000.0);
        wm.insert("Order".to_string(), order);

        // Multi-pattern: Person($name) AND Order(customer == $name)
        let person_pattern = PatternBuilder::for_type("Person")
            .bind("name", "$name")
            .build();

        let order_pattern = PatternBuilder::for_type("Order")
            .where_var("customer", "==", "$name")
            .build();

        let multi = MultiPattern::new("PersonWithOrder".to_string())
            .with_pattern(person_pattern)
            .with_pattern(order_pattern);

        let matches = multi.match_all(&wm);
        assert_eq!(matches.len(), 1);

        let (handles, bindings) = &matches[0];
        assert_eq!(handles.len(), 2);
        assert_eq!(bindings.get("$name").unwrap().as_string(), "John");
    }
}

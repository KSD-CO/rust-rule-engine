//! Template System (inspired by CLIPS deftemplate)
//!
//! Provides type-safe structured facts with schema validation.
//! Similar to CLIPS deftemplate and Drools declared types.

use crate::rete::facts::{FactValue, TypedFacts};
use crate::errors::{Result, RuleEngineError};
use std::collections::HashMap;

/// Field definition in a template
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDef {
    pub name: String,
    pub field_type: FieldType,
    pub default_value: Option<FactValue>,
    pub required: bool,
}

/// Supported field types
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<FieldType>),
    Any,
}

impl FieldType {
    /// Check if a value matches this field type
    pub fn matches(&self, value: &FactValue) -> bool {
        match (self, value) {
            (FieldType::String, FactValue::String(_)) => true,
            (FieldType::Integer, FactValue::Integer(_)) => true,
            (FieldType::Float, FactValue::Float(_)) => true,
            (FieldType::Boolean, FactValue::Boolean(_)) => true,
            (FieldType::Array(inner), FactValue::Array(arr)) => {
                // Check if all elements match the inner type
                arr.iter().all(|v| inner.matches(v))
            }
            (FieldType::Any, _) => true,
            _ => false,
        }
    }

    /// Get default value for this type
    pub fn default_value(&self) -> FactValue {
        match self {
            FieldType::String => FactValue::String(String::new()),
            FieldType::Integer => FactValue::Integer(0),
            FieldType::Float => FactValue::Float(0.0),
            FieldType::Boolean => FactValue::Boolean(false),
            FieldType::Array(_) => FactValue::Array(Vec::new()),
            FieldType::Any => FactValue::Null,
        }
    }
}

/// Template definition (like CLIPS deftemplate)
#[derive(Debug, Clone)]
pub struct Template {
    pub name: String,
    pub fields: Vec<FieldDef>,
    field_map: HashMap<String, usize>,
}

impl Template {
    /// Create a new template
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            field_map: HashMap::new(),
        }
    }

    /// Add a field to the template
    pub fn add_field(&mut self, field: FieldDef) -> &mut Self {
        let idx = self.fields.len();
        self.field_map.insert(field.name.clone(), idx);
        self.fields.push(field);
        self
    }

    /// Validate that facts conform to this template
    pub fn validate(&self, facts: &TypedFacts) -> Result<()> {
        // Check required fields
        for field in &self.fields {
            let value = facts.get(&field.name);

            if field.required && value.is_none() {
                return Err(RuleEngineError::EvaluationError {
                    message: format!(
                        "Required field '{}' missing in template '{}'",
                        field.name, self.name
                    ),
                });
            }

            // Check type if value exists
            if let Some(val) = value {
                if !field.field_type.matches(val) {
                    return Err(RuleEngineError::EvaluationError {
                        message: format!(
                            "Field '{}' has wrong type. Expected {:?}, got {:?}",
                            field.name, field.field_type, val
                        ),
                    });
                }
            }
        }

        Ok(())
    }

    /// Create facts from template with default values
    pub fn create_instance(&self) -> TypedFacts {
        let mut facts = TypedFacts::new();

        for field in &self.fields {
            let value = field.default_value.clone()
                .unwrap_or_else(|| field.field_type.default_value());
            facts.set(&field.name, value);
        }

        facts
    }

    /// Get field definition by name
    pub fn get_field(&self, name: &str) -> Option<&FieldDef> {
        self.field_map.get(name).and_then(|idx| self.fields.get(*idx))
    }
}

/// Template builder for fluent API
pub struct TemplateBuilder {
    template: Template,
}

impl TemplateBuilder {
    /// Start building a template
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            template: Template::new(name),
        }
    }

    /// Add a string field
    pub fn string_field(mut self, name: impl Into<String>) -> Self {
        self.template.add_field(FieldDef {
            name: name.into(),
            field_type: FieldType::String,
            default_value: None,
            required: false,
        });
        self
    }

    /// Add a required string field
    pub fn required_string(mut self, name: impl Into<String>) -> Self {
        self.template.add_field(FieldDef {
            name: name.into(),
            field_type: FieldType::String,
            default_value: None,
            required: true,
        });
        self
    }

    /// Add an integer field
    pub fn integer_field(mut self, name: impl Into<String>) -> Self {
        self.template.add_field(FieldDef {
            name: name.into(),
            field_type: FieldType::Integer,
            default_value: None,
            required: false,
        });
        self
    }

    /// Add a float field
    pub fn float_field(mut self, name: impl Into<String>) -> Self {
        self.template.add_field(FieldDef {
            name: name.into(),
            field_type: FieldType::Float,
            default_value: None,
            required: false,
        });
        self
    }

    /// Add a boolean field
    pub fn boolean_field(mut self, name: impl Into<String>) -> Self {
        self.template.add_field(FieldDef {
            name: name.into(),
            field_type: FieldType::Boolean,
            default_value: None,
            required: false,
        });
        self
    }

    /// Add a field with custom default
    pub fn field_with_default(
        mut self,
        name: impl Into<String>,
        field_type: FieldType,
        default: FactValue,
    ) -> Self {
        self.template.add_field(FieldDef {
            name: name.into(),
            field_type,
            default_value: Some(default),
            required: false,
        });
        self
    }

    /// Add an array field
    pub fn array_field(mut self, name: impl Into<String>, element_type: FieldType) -> Self {
        self.template.add_field(FieldDef {
            name: name.into(),
            field_type: FieldType::Array(Box::new(element_type)),
            default_value: None,
            required: false,
        });
        self
    }

    /// Build the template
    pub fn build(self) -> Template {
        self.template
    }
}

/// Template registry for managing templates
pub struct TemplateRegistry {
    templates: HashMap<String, Template>,
}

impl TemplateRegistry {
    /// Create a new template registry
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template
    pub fn register(&mut self, template: Template) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template by name
    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    /// Create an instance from a template
    pub fn create_instance(&self, template_name: &str) -> Result<TypedFacts> {
        let template = self.get(template_name).ok_or_else(|| {
            RuleEngineError::EvaluationError {
                message: format!("Template '{}' not found", template_name),
            }
        })?;

        Ok(template.create_instance())
    }

    /// Validate facts against a template
    pub fn validate(&self, template_name: &str, facts: &TypedFacts) -> Result<()> {
        let template = self.get(template_name).ok_or_else(|| {
            RuleEngineError::EvaluationError {
                message: format!("Template '{}' not found", template_name),
            }
        })?;

        template.validate(facts)
    }

    /// List all registered templates
    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_builder() {
        let template = TemplateBuilder::new("Person")
            .required_string("name")
            .integer_field("age")
            .boolean_field("is_adult")
            .build();

        assert_eq!(template.name, "Person");
        assert_eq!(template.fields.len(), 3);
        assert!(template.get_field("name").unwrap().required);
    }

    #[test]
    fn test_create_instance() {
        let template = TemplateBuilder::new("Person")
            .string_field("name")
            .integer_field("age")
            .build();

        let instance = template.create_instance();
        assert_eq!(instance.get("name"), Some(&FactValue::String(String::new())));
        assert_eq!(instance.get("age"), Some(&FactValue::Integer(0)));
    }

    #[test]
    fn test_validation_success() {
        let template = TemplateBuilder::new("Person")
            .required_string("name")
            .integer_field("age")
            .build();

        let mut facts = TypedFacts::new();
        facts.set("name", FactValue::String("Alice".to_string()));
        facts.set("age", FactValue::Integer(30));

        assert!(template.validate(&facts).is_ok());
    }

    #[test]
    fn test_validation_missing_required() {
        let template = TemplateBuilder::new("Person")
            .required_string("name")
            .integer_field("age")
            .build();

        let mut facts = TypedFacts::new();
        facts.set("age", FactValue::Integer(30));

        assert!(template.validate(&facts).is_err());
    }

    #[test]
    fn test_validation_wrong_type() {
        let template = TemplateBuilder::new("Person")
            .string_field("name")
            .integer_field("age")
            .build();

        let mut facts = TypedFacts::new();
        facts.set("name", FactValue::String("Alice".to_string()));
        facts.set("age", FactValue::String("thirty".to_string())); // Wrong type!

        assert!(template.validate(&facts).is_err());
    }

    #[test]
    fn test_template_registry() {
        let mut registry = TemplateRegistry::new();

        let template = TemplateBuilder::new("Order")
            .required_string("order_id")
            .float_field("amount")
            .build();

        registry.register(template);

        assert!(registry.get("Order").is_some());
        assert!(registry.create_instance("Order").is_ok());
        assert_eq!(registry.list_templates(), vec!["Order"]);
    }

    #[test]
    fn test_array_field() {
        let template = TemplateBuilder::new("ShoppingCart")
            .array_field("items", FieldType::String)
            .build();

        let mut facts = TypedFacts::new();
        facts.set("items", FactValue::Array(vec![
            FactValue::String("item1".to_string()),
            FactValue::String("item2".to_string()),
        ]));

        assert!(template.validate(&facts).is_ok());
    }

    #[test]
    fn test_field_with_default() {
        let template = TemplateBuilder::new("Config")
            .field_with_default(
                "timeout",
                FieldType::Integer,
                FactValue::Integer(30),
            )
            .build();

        let instance = template.create_instance();
        assert_eq!(instance.get("timeout"), Some(&FactValue::Integer(30)));
    }
}

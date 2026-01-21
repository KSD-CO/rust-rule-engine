use crate::engine::rule::Rule;
use crate::errors::{Result, RuleEngineError};
use crate::parser::GRLParser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameter types for rule templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    /// String parameter type
    String,
    /// Numeric parameter type
    Number,
    /// Boolean parameter type
    Boolean,
    /// Array parameter type
    Array,
}

/// A parameter definition for a rule template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Default value for the parameter
    pub default_value: Option<String>,
    /// Human-readable description
    pub description: Option<String>,
}

/// Rule template that can generate multiple rules with different parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTemplate {
    /// Template name
    pub name: String,
    /// Template description
    pub description: Option<String>,
    /// List of parameters this template accepts
    pub parameters: Vec<ParameterDef>,
    /// Condition template with parameter placeholders
    pub condition_template: String,
    /// Action template with parameter placeholders
    pub action_template: String,
    /// Rule salience/priority
    pub salience: Option<i32>,
}

/// Builder for creating rule template instances
pub struct TemplateInstance {
    template: RuleTemplate,
    rule_name: String,
    parameter_values: HashMap<String, String>,
}

/// Manager for rule templates
pub struct TemplateManager {
    templates: HashMap<String, RuleTemplate>,
}

impl RuleTemplate {
    /// Create a new rule template
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            parameters: Vec::new(),
            condition_template: String::new(),
            action_template: String::new(),
            salience: None,
        }
    }

    /// Add a parameter to the template
    pub fn with_parameter(mut self, name: &str, param_type: ParameterType) -> Self {
        self.parameters.push(ParameterDef {
            name: name.to_string(),
            param_type,
            default_value: None,
            description: None,
        });
        self
    }

    /// Set the condition template
    pub fn with_condition(mut self, condition: &str) -> Self {
        self.condition_template = condition.to_string();
        self
    }

    /// Set the action template
    pub fn with_action(mut self, action: &str) -> Self {
        self.action_template = action.to_string();
        self
    }

    /// Set the salience for generated rules
    pub fn with_salience(mut self, salience: i32) -> Self {
        self.salience = Some(salience);
        self
    }

    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Create a template instance for generating a specific rule
    pub fn instantiate(&self, rule_name: &str) -> TemplateInstance {
        TemplateInstance {
            template: self.clone(),
            rule_name: rule_name.to_string(),
            parameter_values: HashMap::new(),
        }
    }

    /// Validate that all required parameters are provided
    pub fn validate_parameters(&self, params: &HashMap<String, String>) -> Result<()> {
        for param_def in &self.parameters {
            if !params.contains_key(&param_def.name) && param_def.default_value.is_none() {
                return Err(RuleEngineError::ParseError {
                    message: format!("Missing required parameter: {}", param_def.name),
                });
            }
        }
        Ok(())
    }

    /// Replace template placeholders with actual values (public for demo)
    pub fn substitute_placeholders(&self, text: &str, params: &HashMap<String, String>) -> String {
        let mut result = text.to_string();

        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Apply default values for missing parameters
        for param_def in &self.parameters {
            if !params.contains_key(&param_def.name) {
                if let Some(default_value) = &param_def.default_value {
                    let placeholder = format!("{{{{{}}}}}", param_def.name);
                    result = result.replace(&placeholder, default_value);
                }
            }
        }

        result
    }
}

impl TemplateInstance {
    /// Set a parameter value
    pub fn with_param(mut self, name: &str, value: impl ToString) -> Self {
        self.parameter_values
            .insert(name.to_string(), value.to_string());
        self
    }

    /// Build the actual rule from the template
    pub fn build(self) -> Result<Rule> {
        // Validate parameters
        self.template.validate_parameters(&self.parameter_values)?;

        // Substitute placeholders
        let condition = self
            .template
            .substitute_placeholders(&self.template.condition_template, &self.parameter_values);
        let action = self
            .template
            .substitute_placeholders(&self.template.action_template, &self.parameter_values);

        // Generate GRL rule text
        let grl_rule = if let Some(salience) = self.template.salience {
            format!(
                r#"rule "{}" salience {} {{
when
{}
then
{};
}}"#,
                self.rule_name, salience, condition, action
            )
        } else {
            format!(
                r#"rule "{}" {{
when
{}
then
{};
}}"#,
                self.rule_name, condition, action
            )
        };

        // Parse the generated GRL
        let rules = GRLParser::parse_rules(&grl_rule)?;

        if rules.is_empty() {
            return Err(RuleEngineError::ParseError {
                message: "Failed to generate rule from template".to_string(),
            });
        }

        Ok(rules.into_iter().next().unwrap())
    }
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template
    pub fn register_template(&mut self, template: RuleTemplate) {
        self.templates.insert(template.name.clone(), template);
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&RuleTemplate> {
        self.templates.get(name)
    }

    /// Generate multiple rules from a template with different parameter sets
    pub fn generate_rules(
        &self,
        template_name: &str,
        rule_configs: Vec<(String, HashMap<String, String>)>,
    ) -> Result<Vec<Rule>> {
        let template =
            self.get_template(template_name)
                .ok_or_else(|| RuleEngineError::ParseError {
                    message: format!("Template not found: {}", template_name),
                })?;

        let mut rules = Vec::new();

        for (rule_name, params) in rule_configs {
            let mut instance = template.instantiate(&rule_name);
            instance.parameter_values = params;

            rules.push(instance.build()?);
        }

        Ok(rules)
    }

    /// Load templates from JSON file
    pub fn load_from_json(&mut self, json_content: &str) -> Result<()> {
        let templates: Vec<RuleTemplate> =
            serde_json::from_str(json_content).map_err(|e| RuleEngineError::ParseError {
                message: format!("Failed to parse template JSON: {}", e),
            })?;

        for template in templates {
            self.register_template(template);
        }

        Ok(())
    }

    /// Save templates to JSON
    pub fn save_to_json(&self) -> Result<String> {
        let templates: Vec<&RuleTemplate> = self.templates.values().collect();
        serde_json::to_string_pretty(&templates).map_err(|e| RuleEngineError::ParseError {
            message: format!("Failed to serialize templates: {}", e),
        })
    }

    /// List all available templates
    pub fn list_templates(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for TemplateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_creation() {
        let template = RuleTemplate::new("VIPCheck")
            .with_parameter("country", ParameterType::String)
            .with_parameter("threshold", ParameterType::Number)
            .with_condition(
                "User.Country == \"{{country}}\" && User.SpendingTotal >= {{threshold}}",
            )
            .with_action("User.setIsVIP(true)")
            .with_salience(10);

        assert_eq!(template.name, "VIPCheck");
        assert_eq!(template.parameters.len(), 2);
        assert_eq!(template.salience, Some(10));
    }

    #[test]
    fn test_template_instantiation() {
        let template = RuleTemplate::new("VIPCheck")
            .with_parameter("country", ParameterType::String)
            .with_parameter("threshold", ParameterType::Number)
            .with_condition(
                "User.Country == \"{{country}}\" && User.SpendingTotal >= {{threshold}}",
            )
            .with_action("User.setIsVIP(true)");

        let rule = template
            .instantiate("VIPCheck_US")
            .with_param("country", "US")
            .with_param("threshold", "1000")
            .build()
            .unwrap();

        assert_eq!(rule.name, "VIPCheck_US");
    }

    #[test]
    fn test_template_manager() {
        let mut manager = TemplateManager::new();

        let template = RuleTemplate::new("TestTemplate")
            .with_parameter("value", ParameterType::String)
            .with_condition("User.Field == \"{{value}}\"")
            .with_action("User.setResult(true)");

        manager.register_template(template);

        assert!(manager.get_template("TestTemplate").is_some());
        assert_eq!(manager.list_templates().len(), 1);
    }
}

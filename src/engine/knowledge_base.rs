use crate::engine::rule::Rule;
use crate::errors::{Result, RuleEngineError};
use crate::parser::grl::GRLParser;
use crate::types::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Knowledge Base - manages collections of rules and facts
/// Similar to Grule's KnowledgeBase concept
#[derive(Debug)]
pub struct KnowledgeBase {
    name: String,
    rules: Arc<RwLock<Vec<Rule>>>,
    rule_index: Arc<RwLock<HashMap<String, usize>>>,
    version: Arc<RwLock<u64>>,
}

impl KnowledgeBase {
    /// Create a new knowledge base
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rules: Arc::new(RwLock::new(Vec::new())),
            rule_index: Arc::new(RwLock::new(HashMap::new())),
            version: Arc::new(RwLock::new(0)),
        }
    }

    /// Get the knowledge base name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the current version of the knowledge base
    pub fn version(&self) -> u64 {
        *self.version.read().unwrap()
    }

    /// Add a rule to the knowledge base
    pub fn add_rule(&self, rule: Rule) -> Result<()> {
        let mut rules = self.rules.write().unwrap();
        let mut index = self.rule_index.write().unwrap();
        let mut version = self.version.write().unwrap();

        // Check for duplicate rule names
        if index.contains_key(&rule.name) {
            return Err(RuleEngineError::ParseError {
                message: format!("Rule '{}' already exists", rule.name),
            });
        }

        let rule_position = rules.len();
        index.insert(rule.name.clone(), rule_position);
        rules.push(rule);

        // Sort rules by priority (salience)
        rules.sort_by(|a, b| b.salience.cmp(&a.salience));

        // Rebuild index after sorting
        index.clear();
        for (pos, rule) in rules.iter().enumerate() {
            index.insert(rule.name.clone(), pos);
        }

        *version += 1;
        Ok(())
    }

    /// Add multiple rules from GRL text
    pub fn add_rules_from_grl(&self, grl_text: &str) -> Result<usize> {
        let rules = GRLParser::parse_rules(grl_text)?;
        let count = rules.len();

        for rule in rules {
            self.add_rule(rule)?;
        }

        Ok(count)
    }

    /// Remove a rule by name
    pub fn remove_rule(&self, rule_name: &str) -> Result<bool> {
        let mut rules = self.rules.write().unwrap();
        let mut index = self.rule_index.write().unwrap();
        let mut version = self.version.write().unwrap();

        if let Some(&position) = index.get(rule_name) {
            rules.remove(position);

            // Rebuild index
            index.clear();
            for (pos, rule) in rules.iter().enumerate() {
                index.insert(rule.name.clone(), pos);
            }

            *version += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get a rule by name
    pub fn get_rule(&self, rule_name: &str) -> Option<Rule> {
        let rules = self.rules.read().unwrap();
        let index = self.rule_index.read().unwrap();

        if let Some(&position) = index.get(rule_name) {
            rules.get(position).cloned()
        } else {
            None
        }
    }

    /// Get all rules
    pub fn get_rules(&self) -> Vec<Rule> {
        let rules = self.rules.read().unwrap();
        rules.clone()
    }

    /// Get all rule names
    pub fn get_rule_names(&self) -> Vec<String> {
        let index = self.rule_index.read().unwrap();
        index.keys().cloned().collect()
    }

    /// Get rule count
    pub fn rule_count(&self) -> usize {
        let rules = self.rules.read().unwrap();
        rules.len()
    }

    /// Enable or disable a rule
    pub fn set_rule_enabled(&self, rule_name: &str, enabled: bool) -> Result<bool> {
        let mut rules = self.rules.write().unwrap();
        let index = self.rule_index.read().unwrap();
        let mut version = self.version.write().unwrap();

        if let Some(&position) = index.get(rule_name) {
            if let Some(rule) = rules.get_mut(position) {
                rule.enabled = enabled;
                *version += 1;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// Clear all rules
    pub fn clear(&self) {
        let mut rules = self.rules.write().unwrap();
        let mut index = self.rule_index.write().unwrap();
        let mut version = self.version.write().unwrap();

        rules.clear();
        index.clear();
        *version += 1;
    }

    /// Get a snapshot of all rules (for execution)
    pub fn get_rules_snapshot(&self) -> Vec<Rule> {
        let rules = self.rules.read().unwrap();
        rules.clone()
    }

    /// Get knowledge base statistics
    pub fn get_statistics(&self) -> KnowledgeBaseStats {
        let rules = self.rules.read().unwrap();

        let enabled_count = rules.iter().filter(|r| r.enabled).count();
        let disabled_count = rules.len() - enabled_count;

        let mut priority_distribution = HashMap::new();
        for rule in rules.iter() {
            *priority_distribution.entry(rule.salience).or_insert(0) += 1;
        }

        KnowledgeBaseStats {
            name: self.name.clone(),
            version: self.version(),
            total_rules: rules.len(),
            enabled_rules: enabled_count,
            disabled_rules: disabled_count,
            priority_distribution,
        }
    }

    /// Export rules to GRL format
    pub fn export_to_grl(&self) -> String {
        let rules = self.rules.read().unwrap();
        let mut grl_output = String::new();

        grl_output.push_str(&format!("// Knowledge Base: {}\n", self.name));
        grl_output.push_str(&format!("// Version: {}\n", self.version()));
        grl_output.push_str(&format!("// Rules: {}\n\n", rules.len()));

        for rule in rules.iter() {
            grl_output.push_str(&rule.to_grl());
            grl_output.push_str("\n\n");
        }

        grl_output
    }
}

impl Clone for KnowledgeBase {
    fn clone(&self) -> Self {
        let rules = self.rules.read().unwrap();
        let new_kb = KnowledgeBase::new(&self.name);

        for rule in rules.iter() {
            let _ = new_kb.add_rule(rule.clone());
        }

        new_kb
    }
}

/// Statistics about a Knowledge Base
#[derive(Debug, Clone)]
pub struct KnowledgeBaseStats {
    /// The name of the knowledge base
    pub name: String,
    /// The version number of the knowledge base
    pub version: u64,
    /// Total number of rules in the knowledge base
    pub total_rules: usize,
    /// Number of enabled rules
    pub enabled_rules: usize,
    /// Number of disabled rules
    pub disabled_rules: usize,
    /// Distribution of rules by priority/salience
    pub priority_distribution: HashMap<i32, usize>,
}

impl std::fmt::Display for KnowledgeBaseStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Knowledge Base: {}", self.name)?;
        writeln!(f, "Version: {}", self.version)?;
        writeln!(f, "Total Rules: {}", self.total_rules)?;
        writeln!(f, "Enabled Rules: {}", self.enabled_rules)?;
        writeln!(f, "Disabled Rules: {}", self.disabled_rules)?;
        writeln!(f, "Priority Distribution:")?;

        let mut priorities: Vec<_> = self.priority_distribution.iter().collect();
        priorities.sort_by(|a, b| b.0.cmp(a.0));

        for (priority, count) in priorities {
            writeln!(f, "  Priority {}: {} rules", priority, count)?;
        }

        Ok(())
    }
}

/// Extension trait to add GRL export functionality to Rule
trait RuleGRLExport {
    fn to_grl(&self) -> String;
}

impl RuleGRLExport for Rule {
    fn to_grl(&self) -> String {
        let mut grl = String::new();

        // Rule declaration
        grl.push_str(&format!("rule {}", self.name));

        if let Some(ref description) = self.description {
            grl.push_str(&format!(" \"{}\"", description));
        }

        if self.salience != 0 {
            grl.push_str(&format!(" salience {}", self.salience));
        }

        grl.push_str(" {\n");

        // When clause
        grl.push_str("    when\n");
        grl.push_str(&format!("        {}\n", self.conditions.to_grl()));

        // Then clause
        grl.push_str("    then\n");
        for action in &self.actions {
            grl.push_str(&format!("        {};\n", action.to_grl()));
        }

        grl.push('}');

        if !self.enabled {
            grl = format!("// DISABLED\n{}", grl);
        }

        grl
    }
}

/// Extension trait for ConditionGroup GRL export
trait ConditionGroupGRLExport {
    fn to_grl(&self) -> String;
}

impl ConditionGroupGRLExport for crate::engine::rule::ConditionGroup {
    fn to_grl(&self) -> String {
        match self {
            crate::engine::rule::ConditionGroup::Single(condition) => {
                format!(
                    "{} {} {}",
                    condition.field,
                    condition.operator.to_grl(),
                    condition.value.to_grl()
                )
            }
            crate::engine::rule::ConditionGroup::Compound {
                left,
                operator,
                right,
            } => {
                let op_str = match operator {
                    crate::types::LogicalOperator::And => "&&",
                    crate::types::LogicalOperator::Or => "||",
                    crate::types::LogicalOperator::Not => "!",
                };
                format!("{} {} {}", left.to_grl(), op_str, right.to_grl())
            }
            crate::engine::rule::ConditionGroup::Not(condition) => {
                format!("!{}", condition.to_grl())
            }
            crate::engine::rule::ConditionGroup::Exists(condition) => {
                format!("exists({})", condition.to_grl())
            }
            crate::engine::rule::ConditionGroup::Forall(condition) => {
                format!("forall({})", condition.to_grl())
            }
            crate::engine::rule::ConditionGroup::Accumulate {
                source_pattern,
                extract_field,
                source_conditions,
                function,
                function_arg,
                ..
            } => {
                let conditions_str = if source_conditions.is_empty() {
                    String::new()
                } else {
                    format!(", {}", source_conditions.join(", "))
                };
                format!(
                    "accumulate({}(${}: {}{}), {}({}))",
                    source_pattern,
                    function_arg.trim_start_matches('$'),
                    extract_field,
                    conditions_str,
                    function,
                    function_arg
                )
            }
        }
    }
}

/// Extension trait for Operator GRL export
trait OperatorGRLExport {
    fn to_grl(&self) -> &'static str;
}

impl OperatorGRLExport for crate::types::Operator {
    fn to_grl(&self) -> &'static str {
        match self {
            crate::types::Operator::Equal => "==",
            crate::types::Operator::NotEqual => "!=",
            crate::types::Operator::GreaterThan => ">",
            crate::types::Operator::GreaterThanOrEqual => ">=",
            crate::types::Operator::LessThan => "<",
            crate::types::Operator::LessThanOrEqual => "<=",
            crate::types::Operator::Contains => "contains",
            crate::types::Operator::NotContains => "not_contains",
            crate::types::Operator::StartsWith => "starts_with",
            crate::types::Operator::EndsWith => "ends_with",
            crate::types::Operator::Matches => "matches",
        }
    }
}

/// Extension trait for Value GRL export
trait ValueGRLExport {
    fn to_grl(&self) -> String;
}

impl ValueGRLExport for Value {
    fn to_grl(&self) -> String {
        match self {
            Value::String(s) => format!("\"{}\"", s),
            Value::Number(n) => n.to_string(),
            Value::Integer(i) => i.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::Null => "null".to_string(),
            Value::Array(_) => "[array]".to_string(),
            Value::Object(_) => "{object}".to_string(),
            Value::Expression(expr) => expr.clone(), // Export as-is
        }
    }
}

/// Extension trait for ActionType GRL export
trait ActionTypeGRLExport {
    fn to_grl(&self) -> String;
}

impl ActionTypeGRLExport for crate::types::ActionType {
    fn to_grl(&self) -> String {
        match self {
            crate::types::ActionType::Set { field, value } => {
                format!("{} = {}", field, value.to_grl())
            }
            crate::types::ActionType::Log { message } => {
                format!("Log(\"{}\")", message)
            }
            crate::types::ActionType::MethodCall {
                object,
                method,
                args,
            } => {
                let args_str = args
                    .iter()
                    .map(|arg| arg.to_grl())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}.{}({})", object, method, args_str)
            }
            crate::types::ActionType::Retract { object } => {
                format!("retract(${})", object)
            }
            crate::types::ActionType::Custom { action_type, .. } => {
                format!("Custom(\"{}\")", action_type)
            }
            crate::types::ActionType::ActivateAgendaGroup { group } => {
                format!("ActivateAgendaGroup(\"{}\")", group)
            }
            crate::types::ActionType::ScheduleRule {
                rule_name,
                delay_ms,
            } => {
                format!("ScheduleRule({}, \"{}\")", delay_ms, rule_name)
            }
            crate::types::ActionType::CompleteWorkflow { workflow_name } => {
                format!("CompleteWorkflow(\"{}\")", workflow_name)
            }
            crate::types::ActionType::SetWorkflowData { key, value } => {
                format!("SetWorkflowData(\"{}={}\")", key, value.to_grl())
            }
        }
    }
}

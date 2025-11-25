# Backward Chaining - Detailed Implementation Plan

> **Goal:** Transform backward chaining module from prototype (70% complete) to production-ready (100%)
> **Total Timeline:** 7-10 weeks
> **Priority:** Get to production in shortest time with highest quality

---

## Phase 1: Core Fixes (CRITICAL) 🔴

**Duration:** 2-3 weeks
**Goal:** Fix critical bugs and missing core functionality
**Priority:** MUST HAVE for production

### Task 1.1: Proper Expression Parser & AST (Week 1)

**Problem:**
```rust
// Current: String manipulation - fragile
if let Some(eq_pos) = pattern.find("==") {
    let field = pattern[..eq_pos].trim();
    let expected = pattern[eq_pos + 2..].trim();
}
```

**Solution:**
Create proper AST-based expression parser

**Files to create/modify:**
- `src/backward/expression.rs` (NEW)
- `src/backward/query.rs` (MODIFY)

**Implementation:**

```rust
// src/backward/expression.rs

use crate::types::{Value, ComparisonOperator};
use crate::errors::Result;

/// Expression AST for backward chaining queries
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Field reference (e.g., "User.IsVIP")
    Field(String),

    /// Literal value
    Literal(Value),

    /// Binary comparison (e.g., "X == Y", "A > B")
    Comparison {
        left: Box<Expression>,
        operator: ComparisonOperator,
        right: Box<Expression>,
    },

    /// Logical operations (e.g., "A && B", "X || Y")
    Logical {
        left: Box<Expression>,
        operator: LogicalOperator,
        right: Box<Expression>,
    },

    /// Negation (e.g., "!X", "NOT Y")
    Not(Box<Expression>),

    /// Variable (for future unification support)
    Variable(String), // e.g., "?X", "?Customer"
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
}

/// Expression parser using recursive descent
pub struct ExpressionParser {
    input: String,
    position: usize,
}

impl ExpressionParser {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.trim().to_string(),
            position: 0,
        }
    }

    /// Parse expression from string
    pub fn parse(input: &str) -> Result<Expression> {
        let mut parser = Self::new(input);
        parser.parse_expression()
    }

    /// Parse full expression (handles ||)
    fn parse_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_and_expression()?;

        while self.peek_operator("||") {
            self.consume_operator("||");
            let right = self.parse_and_expression()?;
            left = Expression::Logical {
                left: Box::new(left),
                operator: LogicalOperator::Or,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse AND expression (handles &&)
    fn parse_and_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison()?;

        while self.peek_operator("&&") {
            self.consume_operator("&&");
            let right = self.parse_comparison()?;
            left = Expression::Logical {
                left: Box::new(left),
                operator: LogicalOperator::And,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /// Parse comparison (e.g., "X == Y", "A > 5")
    fn parse_comparison(&mut self) -> Result<Expression> {
        let left = self.parse_primary()?;

        // Check for comparison operators
        let operator = if self.peek_operator("==") {
            self.consume_operator("==");
            ComparisonOperator::Equal
        } else if self.peek_operator("!=") {
            self.consume_operator("!=");
            ComparisonOperator::NotEqual
        } else if self.peek_operator(">=") {
            self.consume_operator(">=");
            ComparisonOperator::GreaterOrEqual
        } else if self.peek_operator("<=") {
            self.consume_operator("<=");
            ComparisonOperator::LessOrEqual
        } else if self.peek_operator(">") {
            self.consume_operator(">");
            ComparisonOperator::Greater
        } else if self.peek_operator("<") {
            self.consume_operator("<");
            ComparisonOperator::Less
        } else {
            return Ok(left);
        };

        let right = self.parse_primary()?;

        Ok(Expression::Comparison {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    /// Parse primary expression (field, literal, variable, or parenthesized)
    fn parse_primary(&mut self) -> Result<Expression> {
        self.skip_whitespace();

        // Handle negation
        if self.peek_char() == Some('!') {
            self.consume_char();
            let expr = self.parse_primary()?;
            return Ok(Expression::Not(Box::new(expr)));
        }

        // Handle parentheses
        if self.peek_char() == Some('(') {
            self.consume_char();
            let expr = self.parse_expression()?;
            if self.peek_char() != Some(')') {
                return Err(crate::errors::RuleEngineError::ParseError {
                    message: "Expected closing parenthesis".to_string(),
                });
            }
            self.consume_char();
            return Ok(expr);
        }

        // Handle variables (?X, ?Customer)
        if self.peek_char() == Some('?') {
            self.consume_char();
            let name = self.consume_identifier()?;
            return Ok(Expression::Variable(format!("?{}", name)));
        }

        // Handle literals
        if let Some(value) = self.try_parse_literal() {
            return Ok(Expression::Literal(value));
        }

        // Handle field reference
        let field_name = self.consume_field_path()?;
        Ok(Expression::Field(field_name))
    }

    fn consume_field_path(&mut self) -> Result<String> {
        let mut path = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                path.push(ch);
                self.consume_char();
            } else {
                break;
            }
        }

        if path.is_empty() {
            return Err(crate::errors::RuleEngineError::ParseError {
                message: "Expected field name".to_string(),
            });
        }

        Ok(path)
    }

    fn consume_identifier(&mut self) -> Result<String> {
        let mut ident = String::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.consume_char();
            } else {
                break;
            }
        }

        if ident.is_empty() {
            return Err(crate::errors::RuleEngineError::ParseError {
                message: "Expected identifier".to_string(),
            });
        }

        Ok(ident)
    }

    fn try_parse_literal(&mut self) -> Option<Value> {
        self.skip_whitespace();

        // Boolean literals
        if self.peek_word("true") {
            self.consume_word("true");
            return Some(Value::Boolean(true));
        }
        if self.peek_word("false") {
            self.consume_word("false");
            return Some(Value::Boolean(false));
        }

        // String literals
        if self.peek_char() == Some('"') {
            self.consume_char();
            let mut s = String::new();
            while let Some(ch) = self.peek_char() {
                if ch == '"' {
                    self.consume_char();
                    return Some(Value::String(s));
                }
                if ch == '\\' {
                    self.consume_char();
                    if let Some(escaped) = self.peek_char() {
                        s.push(escaped);
                        self.consume_char();
                    }
                } else {
                    s.push(ch);
                    self.consume_char();
                }
            }
            return None; // Unterminated string
        }

        // Number literals
        if let Some(ch) = self.peek_char() {
            if ch.is_numeric() || ch == '-' {
                let mut num_str = String::new();
                while let Some(ch) = self.peek_char() {
                    if ch.is_numeric() || ch == '.' || ch == '-' {
                        num_str.push(ch);
                        self.consume_char();
                    } else {
                        break;
                    }
                }
                if let Ok(n) = num_str.parse::<f64>() {
                    return Some(Value::Number(n));
                }
            }
        }

        None
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn consume_char(&mut self) {
        if self.position < self.input.len() {
            self.position += 1;
        }
    }

    fn peek_operator(&self, op: &str) -> bool {
        self.skip_whitespace();
        self.input[self.position..].starts_with(op)
    }

    fn consume_operator(&mut self, op: &str) {
        self.skip_whitespace();
        if self.input[self.position..].starts_with(op) {
            self.position += op.len();
        }
    }

    fn peek_word(&self, word: &str) -> bool {
        self.skip_whitespace();
        let remaining = &self.input[self.position..];
        if remaining.starts_with(word) {
            // Make sure it's a complete word (not prefix)
            let next_pos = self.position + word.len();
            if next_pos >= self.input.len() {
                return true;
            }
            let next_char = self.input.chars().nth(next_pos);
            next_char.map(|c| !c.is_alphanumeric() && c != '_').unwrap_or(true)
        } else {
            false
        }
    }

    fn consume_word(&mut self, word: &str) {
        self.skip_whitespace();
        if self.peek_word(word) {
            self.position += word.len();
        }
    }

    fn skip_whitespace(&self) {
        // Note: This should mutate position, but we need to keep it const for peek
        // We'll handle whitespace in the main parsing logic
    }
}

/// Expression evaluator
impl Expression {
    /// Evaluate expression against facts
    pub fn evaluate(&self, facts: &crate::Facts) -> Result<Value> {
        match self {
            Expression::Field(name) => {
                facts.get(name)
                    .or_else(|| facts.get_nested(name))
                    .ok_or_else(|| crate::errors::RuleEngineError::ExecutionError {
                        message: format!("Field not found: {}", name),
                    })
            }

            Expression::Literal(value) => Ok(value.clone()),

            Expression::Comparison { left, operator, right } => {
                let left_val = left.evaluate(facts)?;
                let right_val = right.evaluate(facts)?;

                let result = operator.evaluate(&left_val, &right_val);
                Ok(Value::Boolean(result))
            }

            Expression::Logical { left, operator, right } => {
                let left_val = left.evaluate(facts)?;

                match operator {
                    LogicalOperator::And => {
                        if !left_val.as_bool() {
                            return Ok(Value::Boolean(false));
                        }
                        let right_val = right.evaluate(facts)?;
                        Ok(Value::Boolean(right_val.as_bool()))
                    }
                    LogicalOperator::Or => {
                        if left_val.as_bool() {
                            return Ok(Value::Boolean(true));
                        }
                        let right_val = right.evaluate(facts)?;
                        Ok(Value::Boolean(right_val.as_bool()))
                    }
                }
            }

            Expression::Not(expr) => {
                let value = expr.evaluate(facts)?;
                Ok(Value::Boolean(!value.as_bool()))
            }

            Expression::Variable(_) => {
                Err(crate::errors::RuleEngineError::ExecutionError {
                    message: "Cannot evaluate unbound variable".to_string(),
                })
            }
        }
    }

    /// Check if expression is satisfied (returns true/false)
    pub fn is_satisfied(&self, facts: &crate::Facts) -> bool {
        self.evaluate(facts)
            .map(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Extract all field references from expression
    pub fn extract_fields(&self) -> Vec<String> {
        let mut fields = Vec::new();
        self.extract_fields_recursive(&mut fields);
        fields
    }

    fn extract_fields_recursive(&self, fields: &mut Vec<String>) {
        match self {
            Expression::Field(name) => fields.push(name.clone()),
            Expression::Comparison { left, right, .. } => {
                left.extract_fields_recursive(fields);
                right.extract_fields_recursive(fields);
            }
            Expression::Logical { left, right, .. } => {
                left.extract_fields_recursive(fields);
                right.extract_fields_recursive(fields);
            }
            Expression::Not(expr) => {
                expr.extract_fields_recursive(fields);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_comparison() {
        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        match expr {
            Expression::Comparison { operator, .. } => {
                assert_eq!(operator, ComparisonOperator::Equal);
            }
            _ => panic!("Expected comparison"),
        }
    }

    #[test]
    fn test_parse_logical_and() {
        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        match expr {
            Expression::Logical { operator, .. } => {
                assert_eq!(operator, LogicalOperator::And);
            }
            _ => panic!("Expected logical AND"),
        }
    }

    #[test]
    fn test_parse_negation() {
        let expr = ExpressionParser::parse("!User.IsBanned").unwrap();
        match expr {
            Expression::Not(_) => {}
            _ => panic!("Expected negation"),
        }
    }

    #[test]
    fn test_evaluate_simple() {
        let mut facts = crate::Facts::new();
        facts.set("User.IsVIP", Value::Boolean(true));

        let expr = ExpressionParser::parse("User.IsVIP == true").unwrap();
        let result = expr.evaluate(&facts).unwrap();

        assert_eq!(result, Value::Boolean(true));
    }

    #[test]
    fn test_extract_fields() {
        let expr = ExpressionParser::parse("User.IsVIP == true && Order.Amount > 1000").unwrap();
        let fields = expr.extract_fields();

        assert_eq!(fields.len(), 2);
        assert!(fields.contains(&"User.IsVIP".to_string()));
        assert!(fields.contains(&"Order.Amount".to_string()));
    }
}
```

**Update existing files:**

```rust
// src/backward/query.rs - Update QueryParser to use Expression

use super::expression::{Expression, ExpressionParser};

impl QueryParser {
    /// Parse a query string into a Goal
    pub fn parse(query: &str) -> Result<Goal, String> {
        if query.is_empty() {
            return Err("Empty query".to_string());
        }

        // Parse query into Expression
        let expr = ExpressionParser::parse(query)
            .map_err(|e| format!("Failed to parse query: {}", e))?;

        // Create goal from expression
        let mut goal = Goal::new(query.to_string());
        goal.expression = Some(expr);

        Ok(goal)
    }
}
```

**Tests to add:**
```bash
tests/backward/expression_parser_test.rs
- test_parse_simple_field
- test_parse_comparison_operators
- test_parse_logical_operators
- test_parse_nested_expressions
- test_parse_variables
- test_evaluate_expressions
- test_error_handling
```

**Success Criteria:**
- ✅ All comparison operators work (`==`, `!=`, `>`, `<`, `>=`, `<=`)
- ✅ Logical operators work (`&&`, `||`, `!`)
- ✅ Handles nested expressions with parentheses
- ✅ Proper error messages for invalid syntax
- ✅ 100% test coverage for parser

**Estimated Time:** 5-7 days

---

### Task 1.2: Rule Execution & Fact Derivation (Week 2)

**Problem:**
```rust
// Current: Only checks conditions, doesn't execute actions
if self.check_rule_conditions(&rule, facts) {
    goal.status = GoalStatus::Proven;
    return true;  // ❌ No new facts derived!
}
```

**Solution:**
Execute rule actions to derive new facts during backward chaining

**Files to modify:**
- `src/backward/search.rs`
- `src/backward/backward_engine.rs`

**Implementation:**

```rust
// src/backward/search.rs

use crate::engine::Engine; // Need access to action executor

impl DepthFirstSearch {
    /// Execute rule and derive new facts
    fn execute_rule_if_conditions_match(
        &self,
        rule: &Rule,
        facts: &mut Facts,
        kb: &KnowledgeBase,
    ) -> Result<bool> {
        // Check conditions
        if !self.check_rule_conditions(rule, facts) {
            return Ok(false);
        }

        // Conditions satisfied - execute actions
        println!("[BC] Executing rule '{}' to derive facts", rule.name);

        // Create a temporary engine to execute actions
        // (We need this because action execution needs engine context)
        let temp_engine = crate::RustRuleEngine::new(kb.clone());

        // Execute each action
        for action in &rule.actions {
            match action {
                crate::types::ActionType::Set { field, value } => {
                    // Evaluate value expression
                    let evaluated_value = self.evaluate_value_expression(value, facts)?;

                    // Set the fact
                    facts.set(field, evaluated_value);
                    println!("[BC]   Derived: {} = {:?}", field, facts.get(field));
                }

                crate::types::ActionType::MethodCall { object, method, args } => {
                    // Execute method call
                    temp_engine.execute_method_call(object, method, args, facts)?;
                }

                crate::types::ActionType::Request { .. } => {
                    // Request actions might need user input - skip in BC for now
                    println!("[BC]   Skipping Request action in backward chaining");
                }

                _ => {
                    // Handle other action types as needed
                }
            }
        }

        Ok(true)
    }

    /// Evaluate value expression (support calculations, etc.)
    fn evaluate_value_expression(
        &self,
        value: &crate::types::Value,
        facts: &Facts,
    ) -> Result<crate::types::Value> {
        // For now, just return the value
        // In future, handle expressions like "Order.Price * 0.1"
        Ok(value.clone())
    }
}
```

**Update search logic:**

```rust
// src/backward/search.rs - Update search_recursive_with_execution

fn search_recursive_with_execution(
    &mut self,
    goal: &mut Goal,
    facts: &mut Facts,  // Now mutable!
    kb: &KnowledgeBase,
    depth: usize
) -> bool {
    self.goals_explored += 1;

    // Check depth limit
    if depth > self.max_depth {
        goal.status = GoalStatus::Unprovable;
        return false;
    }

    // Check if goal already satisfied by existing facts
    if let Some(ref expr) = goal.expression {
        if expr.is_satisfied(facts) {
            println!("[BC] Goal already satisfied: {}", goal.pattern);
            goal.status = GoalStatus::Proven;
            return true;
        }
    }

    // Check for cycles
    if goal.status == GoalStatus::InProgress {
        goal.status = GoalStatus::Unprovable;
        return false;
    }

    goal.status = GoalStatus::InProgress;
    goal.depth = depth;

    // Try each candidate rule
    for rule_name in goal.candidate_rules.clone() {
        self.path.push(rule_name.clone());

        println!("[BC] Trying rule: {}", rule_name);

        // Get the rule from KB
        if let Some(rule) = kb.get_rule(&rule_name) {
            // Execute rule if conditions match
            if let Ok(executed) = self.execute_rule_if_conditions_match(&rule, facts, kb) {
                if executed {
                    // Rule executed successfully!
                    // Check if goal is now satisfied
                    if let Some(ref expr) = goal.expression {
                        if expr.is_satisfied(facts) {
                            println!("[BC] ✓ Rule '{}' proved goal!", rule_name);
                            goal.status = GoalStatus::Proven;
                            return true;
                        }
                    }
                }
            }
        }

        self.path.pop();
    }

    // If no rule proved the goal directly, try sub-goals
    for sub_goal in &mut goal.sub_goals {
        if !self.search_recursive_with_execution(sub_goal, facts, kb, depth + 1) {
            goal.status = GoalStatus::Unprovable;
            return false;
        }
    }

    // Check one more time after sub-goals
    if let Some(ref expr) = goal.expression {
        if expr.is_satisfied(facts) {
            goal.status = GoalStatus::Proven;
            return true;
        }
    }

    // If no way to prove
    goal.status = GoalStatus::Unprovable;
    false
}
```

**Tests to add:**
```bash
tests/backward/rule_execution_test.rs
- test_rule_derives_fact
- test_multiple_rules_chain
- test_actions_execute_in_order
- test_method_calls_work
- test_fact_derivation_satisfies_goal
```

**Success Criteria:**
- ✅ Rules execute actions when conditions match
- ✅ New facts are derived from rule actions
- ✅ Chained reasoning works (Rule A derives fact → Rule B uses it)
- ✅ Facts state is updated correctly
- ✅ Integration test with real rules passes

**Estimated Time:** 4-6 days

---

### Task 1.3: RETE Integration (Week 2-3)

**Problem:**
```rust
// Current: Naive candidate finding - O(n) over all rules
for rule in self.knowledge_base.get_rules() {
    if self.rule_could_prove_goal(&rule, goal) {
        goal.add_candidate_rule(rule.name.clone());
    }
}
```

**Solution:**
Integrate with RETE network to find candidate rules efficiently

**Files to modify:**
- `src/backward/backward_engine.rs`
- `src/backward/search.rs`
- Create `src/backward/rete_integration.rs`

**Implementation:**

```rust
// src/backward/rete_integration.rs

use crate::rete::network::ReteNetwork;
use crate::rete::alpha_node::AlphaNode;
use crate::backward::goal::Goal;
use crate::backward::expression::Expression;
use std::collections::HashSet;

/// Integrates backward chaining with RETE network
pub struct BackwardReteIntegration {
    /// Reference to RETE network (if available)
    rete_network: Option<Arc<ReteNetwork>>,

    /// Index: field_name -> rules that conclude on that field
    conclusion_index: HashMap<String, Vec<String>>,
}

impl BackwardReteIntegration {
    pub fn new() -> Self {
        Self {
            rete_network: None,
            conclusion_index: HashMap::new(),
        }
    }

    /// Build index of rule conclusions
    pub fn build_conclusion_index(&mut self, kb: &KnowledgeBase) {
        self.conclusion_index.clear();

        for rule in kb.get_rules() {
            // Extract fields that this rule concludes (sets)
            let concluded_fields = self.extract_conclusion_fields(rule);

            for field in concluded_fields {
                self.conclusion_index
                    .entry(field)
                    .or_insert_with(Vec::new)
                    .push(rule.name.clone());
            }
        }

        println!("[BC-RETE] Built conclusion index with {} entries",
                 self.conclusion_index.len());
    }

    /// Extract fields that a rule concludes (from its actions)
    fn extract_conclusion_fields(&self, rule: &Rule) -> HashSet<String> {
        let mut fields = HashSet::new();

        for action in &rule.actions {
            match action {
                crate::types::ActionType::Set { field, .. } => {
                    fields.insert(field.clone());

                    // Also index base object (e.g., "Order" from "Order.Status")
                    if let Some(dot_pos) = field.find('.') {
                        let base = &field[..dot_pos];
                        fields.insert(base.to_string());
                    }
                }
                crate::types::ActionType::MethodCall { object, .. } => {
                    fields.insert(object.clone());
                }
                _ => {}
            }
        }

        fields
    }

    /// Find candidate rules that can prove a goal (using index)
    pub fn find_candidate_rules(&self, goal: &Goal) -> Vec<String> {
        let mut candidates = Vec::new();

        // Extract fields from goal expression
        let goal_fields = if let Some(ref expr) = goal.expression {
            expr.extract_fields()
        } else {
            // Fallback: parse from pattern string
            self.extract_fields_from_pattern(&goal.pattern)
        };

        println!("[BC-RETE] Goal fields: {:?}", goal_fields);

        // Find rules that conclude on any of these fields
        for field in goal_fields {
            if let Some(rules) = self.conclusion_index.get(&field) {
                candidates.extend(rules.iter().cloned());
            }

            // Also check base object
            if let Some(dot_pos) = field.find('.') {
                let base = &field[..dot_pos];
                if let Some(rules) = self.conclusion_index.get(base) {
                    candidates.extend(rules.iter().cloned());
                }
            }
        }

        // Remove duplicates
        let mut unique_candidates: Vec<String> = candidates.into_iter().collect();
        unique_candidates.sort();
        unique_candidates.dedup();

        println!("[BC-RETE] Found {} candidate rules", unique_candidates.len());

        unique_candidates
    }

    /// Extract fields from pattern string (fallback)
    fn extract_fields_from_pattern(&self, pattern: &str) -> Vec<String> {
        let mut fields = Vec::new();

        // Simple regex to extract field references
        let re = regex::Regex::new(r"([A-Za-z_][A-Za-z0-9_.]+)").unwrap();
        for cap in re.captures_iter(pattern) {
            let field = cap[1].to_string();
            // Filter out keywords
            if !["true", "false", "and", "or", "not"].contains(&field.as_str()) {
                fields.push(field);
            }
        }

        fields
    }

    /// Check if RETE can help with pattern matching
    pub fn can_use_rete_matching(&self) -> bool {
        self.rete_network.is_some()
    }

    /// Use RETE alpha network for pattern matching
    pub fn match_pattern_with_rete(
        &self,
        pattern: &str,
        facts: &Facts,
    ) -> bool {
        if let Some(ref network) = self.rete_network {
            // Use RETE for efficient pattern matching
            // TODO: Implement when RETE API is ready
            false
        } else {
            false
        }
    }
}
```

**Update BackwardEngine to use integration:**

```rust
// src/backward/backward_engine.rs

use super::rete_integration::BackwardReteIntegration;

pub struct BackwardEngine {
    knowledge_base: Arc<KnowledgeBase>,
    config: BackwardConfig,
    goal_manager: GoalManager,
    rete_integration: BackwardReteIntegration,  // NEW
}

impl BackwardEngine {
    pub fn new(kb: KnowledgeBase) -> Self {
        let mut rete_integration = BackwardReteIntegration::new();

        // Build conclusion index for fast lookup
        rete_integration.build_conclusion_index(&kb);

        Self {
            knowledge_base: Arc::new(kb),
            config: BackwardConfig::default(),
            goal_manager: GoalManager::default(),
            rete_integration,  // NEW
        }
    }

    /// Find candidate rules using RETE integration
    fn find_candidate_rules(&self, goal: &mut Goal) -> Result<()> {
        // Use RETE integration for efficient lookup
        let candidates = self.rete_integration.find_candidate_rules(goal);

        println!("[BC] Found {} candidates for goal: {}",
                 candidates.len(), goal.pattern);

        for rule_name in candidates {
            goal.add_candidate_rule(rule_name);
        }

        Ok(())
    }
}
```

**Tests to add:**
```bash
tests/backward/rete_integration_test.rs
- test_build_conclusion_index
- test_find_candidates_by_field
- test_find_candidates_for_nested_field
- test_index_efficiency (benchmark)
- test_integration_with_real_kb
```

**Success Criteria:**
- ✅ Conclusion index built correctly
- ✅ Candidate finding is O(1) instead of O(n)
- ✅ Integration with existing RETE (if available)
- ✅ Benchmark shows >10x speedup for large rule sets
- ✅ All existing tests still pass

**Estimated Time:** 5-7 days

---

### Task 1.4: Variable Bindings & Unification (Week 3)

**Problem:**
```rust
// Current: No variable binding support
pub bindings: HashMap<String, Value>,  // Always empty
```

**Solution:**
Implement basic variable binding and unification

**Files to create/modify:**
- Create `src/backward/unification.rs`
- Modify `src/backward/goal.rs`
- Modify `src/backward/expression.rs`

**Implementation:**

```rust
// src/backward/unification.rs

use crate::types::Value;
use std::collections::HashMap;

/// Variable bindings during proof
#[derive(Debug, Clone)]
pub struct Bindings {
    /// Map from variable name to value
    bindings: HashMap<String, Value>,
}

impl Bindings {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Bind a variable to a value
    pub fn bind(&mut self, var_name: String, value: Value) -> Result<(), String> {
        // Check if already bound
        if let Some(existing) = self.bindings.get(&var_name) {
            // Must match existing binding
            if existing != &value {
                return Err(format!(
                    "Variable {} already bound to {:?}, cannot rebind to {:?}",
                    var_name, existing, value
                ));
            }
        } else {
            self.bindings.insert(var_name, value);
        }
        Ok(())
    }

    /// Get binding for a variable
    pub fn get(&self, var_name: &str) -> Option<&Value> {
        self.bindings.get(var_name)
    }

    /// Check if variable is bound
    pub fn is_bound(&self, var_name: &str) -> bool {
        self.bindings.contains_key(var_name)
    }

    /// Merge bindings from another set
    pub fn merge(&mut self, other: &Bindings) -> Result<(), String> {
        for (var, val) in &other.bindings {
            self.bind(var.clone(), val.clone())?;
        }
        Ok(())
    }

    /// Get all bindings
    pub fn as_map(&self) -> &HashMap<String, Value> {
        &self.bindings
    }
}

/// Unification algorithm for pattern matching
pub struct Unifier;

impl Unifier {
    /// Unify two expressions with variable bindings
    pub fn unify(
        left: &Expression,
        right: &Expression,
        bindings: &mut Bindings,
    ) -> Result<bool, String> {
        match (left, right) {
            // Variable on left
            (Expression::Variable(var), expr) => {
                if let Some(bound_value) = bindings.get(var) {
                    // Variable already bound - check if it matches
                    Self::unify(
                        &Expression::Literal(bound_value.clone()),
                        expr,
                        bindings
                    )
                } else {
                    // Bind variable to expression value
                    // (Assumes expression evaluates to a value)
                    // For now, only support binding to literals
                    if let Expression::Literal(val) = expr {
                        bindings.bind(var.clone(), val.clone())?;
                        Ok(true)
                    } else {
                        Err(format!("Cannot bind variable to non-literal expression"))
                    }
                }
            }

            // Variable on right
            (expr, Expression::Variable(var)) => {
                Self::unify(&Expression::Variable(var.clone()), expr, bindings)
            }

            // Two literals - must be equal
            (Expression::Literal(v1), Expression::Literal(v2)) => {
                Ok(v1 == v2)
            }

            // Two fields - must be same field
            (Expression::Field(f1), Expression::Field(f2)) => {
                Ok(f1 == f2)
            }

            // Comparison - both sides must unify
            (
                Expression::Comparison { left: l1, operator: op1, right: r1 },
                Expression::Comparison { left: l2, operator: op2, right: r2 }
            ) => {
                if op1 != op2 {
                    return Ok(false);
                }

                let left_match = Self::unify(l1, l2, bindings)?;
                let right_match = Self::unify(r1, r2, bindings)?;

                Ok(left_match && right_match)
            }

            _ => Ok(false),
        }
    }

    /// Match expression against facts and extract bindings
    pub fn match_expression(
        expr: &Expression,
        facts: &crate::Facts,
        bindings: &mut Bindings,
    ) -> Result<bool, String> {
        match expr {
            Expression::Variable(var) => {
                // Cannot match unbound variable
                if !bindings.is_bound(var) {
                    return Err(format!("Unbound variable: {}", var));
                }
                Ok(true)
            }

            Expression::Field(field_name) => {
                // Field must exist in facts
                Ok(facts.get(field_name).is_some())
            }

            Expression::Literal(_) => {
                // Literals always match
                Ok(true)
            }

            Expression::Comparison { left, operator, right } => {
                // Evaluate both sides with bindings
                let left_val = Self::evaluate_with_bindings(left, facts, bindings)?;
                let right_val = Self::evaluate_with_bindings(right, facts, bindings)?;

                let result = operator.evaluate(&left_val, &right_val);
                Ok(result)
            }

            Expression::Logical { left, operator, right } => {
                let left_match = Self::match_expression(left, facts, bindings)?;

                match operator {
                    LogicalOperator::And => {
                        if !left_match {
                            return Ok(false);
                        }
                        Self::match_expression(right, facts, bindings)
                    }
                    LogicalOperator::Or => {
                        if left_match {
                            return Ok(true);
                        }
                        Self::match_expression(right, facts, bindings)
                    }
                }
            }

            Expression::Not(expr) => {
                let result = Self::match_expression(expr, facts, bindings)?;
                Ok(!result)
            }
        }
    }

    /// Evaluate expression with variable bindings
    fn evaluate_with_bindings(
        expr: &Expression,
        facts: &crate::Facts,
        bindings: &Bindings,
    ) -> Result<Value, String> {
        match expr {
            Expression::Variable(var) => {
                bindings.get(var)
                    .cloned()
                    .ok_or_else(|| format!("Unbound variable: {}", var))
            }

            Expression::Field(field) => {
                facts.get(field)
                    .or_else(|| facts.get_nested(field))
                    .ok_or_else(|| format!("Field not found: {}", field))
            }

            Expression::Literal(val) => Ok(val.clone()),

            _ => expr.evaluate(facts)
                .map_err(|e| format!("Evaluation error: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backward::expression::{Expression, ExpressionParser};

    #[test]
    fn test_bind_variable() {
        let mut bindings = Bindings::new();

        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();

        assert_eq!(bindings.get("X"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_unify_variable_with_literal() {
        let mut bindings = Bindings::new();

        let var = Expression::Variable("X".to_string());
        let lit = Expression::Literal(Value::Number(42.0));

        let result = Unifier::unify(&var, &lit, &mut bindings).unwrap();

        assert!(result);
        assert_eq!(bindings.get("X"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn test_unify_bound_variable() {
        let mut bindings = Bindings::new();
        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();

        let var = Expression::Variable("X".to_string());
        let lit = Expression::Literal(Value::Number(42.0));

        let result = Unifier::unify(&var, &lit, &mut bindings).unwrap();

        assert!(result);
    }

    #[test]
    fn test_unify_conflicting_bindings() {
        let mut bindings = Bindings::new();
        bindings.bind("X".to_string(), Value::Number(42.0)).unwrap();

        let var = Expression::Variable("X".to_string());
        let lit = Expression::Literal(Value::Number(100.0));

        let result = Unifier::unify(&var, &lit, &mut bindings);

        assert!(result.is_err());
    }
}
```

**Update Goal to use Bindings:**

```rust
// src/backward/goal.rs

use super::unification::Bindings;
use super::expression::Expression;

pub struct Goal {
    pub pattern: String,
    pub status: GoalStatus,
    pub sub_goals: Vec<Goal>,
    pub candidate_rules: Vec<String>,
    pub bindings: Bindings,  // Changed from HashMap
    pub depth: usize,
    pub expression: Option<Expression>,  // NEW
}

impl Goal {
    pub fn new(pattern: String) -> Self {
        Self {
            pattern,
            status: GoalStatus::Pending,
            sub_goals: Vec::new(),
            candidate_rules: Vec::new(),
            bindings: Bindings::new(),  // Use new Bindings type
            depth: 0,
            expression: None,
        }
    }
}
```

**Tests to add:**
```bash
tests/backward/unification_test.rs
- test_bind_single_variable
- test_bind_multiple_variables
- test_unify_variables
- test_unify_with_bindings
- test_merge_bindings
- test_conflicting_bindings
```

**Success Criteria:**
- ✅ Variable binding works correctly
- ✅ Unification algorithm implemented
- ✅ Bindings propagate through proof tree
- ✅ Conflicts detected and reported
- ✅ Integration with expression evaluation

**Estimated Time:** 4-5 days

---

## Phase 1 Summary

**Total Duration:** 2-3 weeks (18-25 days)

**Deliverables:**
- ✅ Proper expression parser with AST
- ✅ Rule execution with fact derivation
- ✅ RETE integration for efficient candidate finding
- ✅ Basic variable binding and unification

**Success Criteria:**
- All Phase 1 tests pass (100+ new tests)
- Existing examples still work
- Performance improvement measurable
- Code coverage > 80%

---

## Phase 2: Quality & Testing (IMPORTANT) 🟡

**Duration:** 2 weeks
**Goal:** Production-grade quality, testing, and error handling
**Priority:** MUST HAVE for production confidence

### Task 2.1: Comprehensive Testing (Week 4)

**Goal:** Achieve >90% test coverage and catch edge cases

**Test categories to add:**

```bash
# Unit tests (expand existing)
tests/backward/
  ├── expression_parser_test.rs       (50+ tests)
  ├── unification_test.rs             (30+ tests)
  ├── goal_manager_test.rs            (20+ tests)
  ├── search_strategies_test.rs       (40+ tests)
  └── backward_engine_test.rs         (60+ tests)

# Integration tests (NEW)
tests/integration/backward/
  ├── simple_queries_test.rs          # Basic query scenarios
  ├── complex_queries_test.rs         # Nested, compound queries
  ├── rule_chaining_test.rs           # Multi-step reasoning
  ├── grl_query_syntax_test.rs        # GRL query parsing
  └── hybrid_mode_test.rs             # Forward + Backward

# Property-based tests (NEW using proptest)
tests/property/backward/
  ├── parser_properties.rs            # Parse → Print → Parse = identity
  ├── unification_properties.rs       # Unify properties
  └── search_properties.rs            # Search completeness

# Stress tests (NEW)
tests/stress/backward/
  ├── deep_recursion_test.rs          # Test max depth limits
  ├── many_rules_test.rs              # 1000+ rules
  ├── complex_expressions_test.rs     # Deep nesting
  └── memory_leak_test.rs             # Long-running queries
```

**Example comprehensive test:**

```rust
// tests/integration/backward/rule_chaining_test.rs

#[test]
fn test_multi_step_backward_chaining() {
    // Setup: Chain of 5 rules
    // Rule1: A && B → C
    // Rule2: C && D → E
    // Rule3: E → F
    // Rule4: F && G → H
    // Rule5: H → Goal

    let rules = r#"
        rule "Rule1" {
            when {
                A == true && B == true
            }
            then {
                C = true;
            }
        }

        rule "Rule2" {
            when {
                C == true && D == true
            }
            then {
                E = true;
            }
        }

        rule "Rule3" {
            when {
                E == true
            }
            then {
                F = true;
            }
        }

        rule "Rule4" {
            when {
                F == true && G == true
            }
            then {
                H = true;
            }
        }

        rule "Rule5" {
            when {
                H == true
            }
            then {
                Goal = true;
            }
        }
    "#;

    let mut kb = KnowledgeBase::new("ChainTest");
    let parsed_rules = GRLParser::parse_rules(rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    // Initial facts: Only A, B, D, G
    let mut facts = Facts::new();
    facts.set("A", Value::Boolean(true));
    facts.set("B", Value::Boolean(true));
    facts.set("D", Value::Boolean(true));
    facts.set("G", Value::Boolean(true));

    // Query: Can we prove Goal?
    let mut bc_engine = BackwardEngine::new(kb);
    let result = bc_engine.query("Goal == true", &mut facts).unwrap();

    // Should be provable through chain
    assert!(result.provable, "Goal should be provable through rule chain");

    // Check that intermediate facts were derived
    assert_eq!(facts.get("C"), Some(&Value::Boolean(true)));
    assert_eq!(facts.get("E"), Some(&Value::Boolean(true)));
    assert_eq!(facts.get("F"), Some(&Value::Boolean(true)));
    assert_eq!(facts.get("H"), Some(&Value::Boolean(true)));
    assert_eq!(facts.get("Goal"), Some(&Value::Boolean(true)));

    // Check proof trace
    assert!(result.proof_trace.steps.len() > 0);

    // Check stats
    assert!(result.stats.goals_explored >= 5); // At least 5 goals
    assert_eq!(result.stats.max_depth, 5); // Depth of 5
}

#[test]
fn test_backward_chaining_with_missing_facts() {
    let rules = r#"
        rule "VIPRule" {
            when {
                User.Points > 100
            }
            then {
                User.IsVIP = true;
            }
        }
    "#;

    let mut kb = KnowledgeBase::new("MissingTest");
    let parsed_rules = GRLParser::parse_rules(rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    // No facts provided
    let mut facts = Facts::new();

    // Query: Is user VIP?
    let mut bc_engine = BackwardEngine::new(kb);
    let result = bc_engine.query("User.IsVIP == true", &mut facts).unwrap();

    // Should not be provable
    assert!(!result.provable);

    // Should report missing facts
    assert!(!result.missing_facts.is_empty());
    assert!(result.missing_facts.contains(&"User.Points".to_string()) ||
            result.missing_facts.contains(&"User.Points > 100".to_string()));
}

#[test]
fn test_circular_dependency_detection() {
    let rules = r#"
        rule "Rule1" {
            when { B == true }
            then { A = true; }
        }

        rule "Rule2" {
            when { A == true }
            then { B = true; }
        }
    "#;

    let mut kb = KnowledgeBase::new("CircularTest");
    let parsed_rules = GRLParser::parse_rules(rules).unwrap();
    for rule in parsed_rules {
        kb.add_rule(rule).unwrap();
    }

    let mut facts = Facts::new();

    let mut bc_engine = BackwardEngine::new(kb);
    let result = bc_engine.query("A == true", &mut facts).unwrap();

    // Should detect circular dependency and not infinite loop
    assert!(!result.provable);

    // Should complete within reasonable time (test timeout)
    // If we get here, no infinite loop occurred
}
```

**Property-based testing:**

```rust
// tests/property/backward/parser_properties.rs

use proptest::prelude::*;
use rust_rule_engine::backward::expression::{Expression, ExpressionParser};

proptest! {
    #[test]
    fn test_parse_print_parse_identity(field in "[A-Z][a-z]{1,10}", value in 0.0f64..1000.0f64) {
        // Property: parse(print(expr)) == expr

        let expr_str = format!("{} == {}", field, value);
        let expr1 = ExpressionParser::parse(&expr_str).unwrap();

        // Print back to string
        let printed = format!("{:?}", expr1);

        // Parse again
        // (For this to work, we'd need Display impl for Expression)
        // For now, just check it parses
        let expr2 = ExpressionParser::parse(&expr_str).unwrap();

        // Should be equivalent
        prop_assert_eq!(format!("{:?}", expr1), format!("{:?}", expr2));
    }

    #[test]
    fn test_valid_expressions_parse(
        field in "[A-Z][a-zA-Z0-9.]{1,20}",
        value in any::<bool>()
    ) {
        // Property: All well-formed expressions should parse

        let expr_str = format!("{} == {}", field, value);
        let result = ExpressionParser::parse(&expr_str);

        prop_assert!(result.is_ok(), "Should parse: {}", expr_str);
    }
}
```

**Success Criteria:**
- ✅ >200 unit tests across all modules
- ✅ 30+ integration tests
- ✅ Property-based tests for parser & unification
- ✅ Stress tests pass (no crashes, no infinite loops)
- ✅ Code coverage >90%

**Estimated Time:** 5-7 days

---

### Task 2.2: Error Handling & Logging (Week 4-5)

**Goal:** Production-grade error handling with good diagnostics

**Files to create/modify:**
- Create `src/backward/errors.rs`
- Add logging throughout

**Implementation:**

```rust
// src/backward/errors.rs

use thiserror::Error;

/// Backward chaining specific errors
#[derive(Error, Debug)]
pub enum BackwardError {
    #[error("Query parse error: {message} at position {position}")]
    QueryParseError {
        message: String,
        position: usize,
        context: String,
    },

    #[error("Goal unprovable: {goal}. Missing facts: {missing_facts:?}")]
    GoalUnprovable {
        goal: String,
        missing_facts: Vec<String>,
        attempted_rules: Vec<String>,
    },

    #[error("Search depth exceeded: max_depth={max_depth}, goal={goal}")]
    MaxDepthExceeded {
        max_depth: usize,
        goal: String,
        current_depth: usize,
    },

    #[error("Circular dependency detected: {chain:?}")]
    CircularDependency {
        chain: Vec<String>,
    },

    #[error("Unification failed: cannot unify {left} with {right}")]
    UnificationError {
        left: String,
        right: String,
        reason: String,
    },

    #[error("Variable {var_name} is unbound")]
    UnboundVariable {
        var_name: String,
        context: String,
    },

    #[error("Execution timeout after {duration_ms}ms")]
    Timeout {
        duration_ms: u64,
        goal: String,
    },

    #[error("Invalid configuration: {message}")]
    InvalidConfig {
        message: String,
    },

    #[error(transparent)]
    EngineError(#[from] crate::errors::RuleEngineError),
}

impl BackwardError {
    /// Get user-friendly error message with suggestions
    pub fn user_message(&self) -> String {
        match self {
            BackwardError::QueryParseError { message, position, context } => {
                format!(
                    "Cannot parse query:\n  {}\n  {}^\n  Error: {}",
                    context,
                    " ".repeat(*position),
                    message
                )
            }

            BackwardError::GoalUnprovable { goal, missing_facts, attempted_rules } => {
                format!(
                    "Cannot prove goal: {}\n\nMissing information:\n{}\n\nAttempted rules:\n{}",
                    goal,
                    missing_facts.iter()
                        .map(|f| format!("  - {}", f))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    attempted_rules.iter()
                        .map(|r| format!("  - {}", r))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            }

            BackwardError::MaxDepthExceeded { max_depth, goal, current_depth } => {
                format!(
                    "Search too deep for goal: {}\n\
                     Exceeded maximum depth of {} (reached {})\n\
                     Suggestion: Increase max_depth or check for circular dependencies",
                    goal, max_depth, current_depth
                )
            }

            BackwardError::CircularDependency { chain } => {
                format!(
                    "Circular dependency detected:\n  {}\n\
                     These rules depend on each other, creating a loop.",
                    chain.join(" → ")
                )
            }

            _ => format!("{}", self),
        }
    }
}
```

**Add structured logging:**

```rust
// Add to Cargo.toml dependencies
// tracing = "0.1"
// tracing-subscriber = "0.3"

// src/backward/backward_engine.rs

use tracing::{info, warn, debug, error, span, Level};

impl BackwardEngine {
    pub fn query(&mut self, query_str: &str, facts: &Facts) -> Result<QueryResult, BackwardError> {
        let span = span!(Level::INFO, "backward_query", query = %query_str);
        let _enter = span.enter();

        info!("Starting backward chaining query");
        debug!("Initial facts: {} entries", facts.len());

        // Parse query
        let mut goal = match QueryParser::parse(query_str) {
            Ok(g) => {
                debug!("Query parsed successfully");
                g
            }
            Err(e) => {
                error!("Query parse failed: {}", e);
                return Err(BackwardError::QueryParseError {
                    message: e,
                    position: 0,
                    context: query_str.to_string(),
                });
            }
        };

        // Find candidates
        let start = std::time::Instant::now();
        self.find_candidate_rules(&mut goal)?;
        let elapsed = start.elapsed();

        info!(
            "Found {} candidate rules in {:?}",
            goal.candidate_rules.len(),
            elapsed
        );

        // Execute search
        let search_span = span!(Level::DEBUG, "search_execution");
        let _search_enter = search_span.enter();

        let search_result = self.execute_search(&mut goal, facts)?;

        if search_result.success {
            info!(
                "Query succeeded! Explored {} goals, max depth {}",
                search_result.goals_explored,
                search_result.max_depth_reached
            );
        } else {
            warn!(
                "Query failed. Explored {} goals",
                search_result.goals_explored
            );
        }

        // Build result
        let result = self.build_query_result(goal, search_result)?;

        Ok(result)
    }
}
```

**Success Criteria:**
- ✅ Specific error types for all failure modes
- ✅ User-friendly error messages with suggestions
- ✅ Structured logging with tracing
- ✅ Error context preserved through call stack
- ✅ No panics in production code

**Estimated Time:** 3-4 days

---

### Task 2.3: Performance Benchmarks (Week 5)

**Goal:** Measure and document performance characteristics

**Files to create:**
```bash
benches/backward_chaining_bench.rs
```

**Implementation:**

```rust
// benches/backward_chaining_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use rust_rule_engine::*;
use rust_rule_engine::backward::*;

fn benchmark_simple_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("simple_query");

    for num_rules in [10, 50, 100, 500, 1000].iter() {
        let (kb, facts) = setup_simple_kb(*num_rules);

        group.bench_with_input(
            BenchmarkId::from_parameter(num_rules),
            num_rules,
            |b, _| {
                let mut bc_engine = BackwardEngine::new(kb.clone());
                let mut facts_clone = facts.clone();

                b.iter(|| {
                    let result = bc_engine.query(
                        "Goal == true",
                        &mut facts_clone
                    );
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_deep_chaining(c: &mut Criterion) {
    let mut group = c.benchmark_group("deep_chaining");

    for depth in [5, 10, 15, 20].iter() {
        let (kb, facts) = setup_chain_kb(*depth);

        group.bench_with_input(
            BenchmarkId::from_parameter(depth),
            depth,
            |b, _| {
                let mut bc_engine = BackwardEngine::new(kb.clone());
                let mut facts_clone = facts.clone();

                b.iter(|| {
                    let result = bc_engine.query(
                        "FinalGoal == true",
                        &mut facts_clone
                    );
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_complex_expressions(c: &mut Criterion) {
    let mut group = c.benchmark_group("expression_parsing");

    let expressions = vec![
        "A == true",
        "A == true && B == true",
        "A == true && B == true && C == true",
        "A == true && (B == true || C == true)",
        "A == true && B == true && C == true && D == true && E == true",
    ];

    for expr in expressions {
        group.bench_with_input(
            BenchmarkId::from_parameter(expr),
            expr,
            |b, &expr| {
                b.iter(|| {
                    let result = ExpressionParser::parse(expr);
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

fn setup_simple_kb(num_rules: usize) -> (KnowledgeBase, Facts) {
    let mut kb = KnowledgeBase::new("BenchKB");

    for i in 0..num_rules {
        let rule_grl = format!(
            r#"
            rule "Rule{}" {{
                when {{ Fact{} == true }}
                then {{ Derived{} = true; }}
            }}
            "#,
            i, i, i
        );

        let rules = GRLParser::parse_rules(&rule_grl).unwrap();
        for rule in rules {
            kb.add_rule(rule).unwrap();
        }
    }

    // Add goal rule
    let goal_rule = format!(
        r#"
        rule "GoalRule" {{
            when {{ Derived{} == true }}
            then {{ Goal = true; }}
        }}
        "#,
        num_rules - 1
    );
    let rules = GRLParser::parse_rules(&goal_rule).unwrap();
    for rule in rules {
        kb.add_rule(rule).unwrap();
    }

    // Setup facts
    let mut facts = Facts::new();
    for i in 0..num_rules {
        facts.set(&format!("Fact{}", i), Value::Boolean(true));
    }

    (kb, facts)
}

fn setup_chain_kb(depth: usize) -> (KnowledgeBase, Facts) {
    let mut kb = KnowledgeBase::new("ChainKB");

    // Create chain: A0 → A1 → A2 → ... → An → FinalGoal
    for i in 0..depth {
        let rule_grl = format!(
            r#"
            rule "Chain{}" {{
                when {{ A{} == true }}
                then {{ A{} = true; }}
            }}
            "#,
            i, i, i + 1
        );

        let rules = GRLParser::parse_rules(&rule_grl).unwrap();
        for rule in rules {
            kb.add_rule(rule).unwrap();
        }
    }

    // Final rule
    let final_rule = format!(
        r#"
        rule "FinalRule" {{
            when {{ A{} == true }}
            then {{ FinalGoal = true; }}
        }}
        "#,
        depth
    );
    let rules = GRLParser::parse_rules(&final_rule).unwrap();
    for rule in rules {
        kb.add_rule(rule).unwrap();
    }

    // Only provide first fact
    let mut facts = Facts::new();
    facts.set("A0", Value::Boolean(true));

    (kb, facts)
}

criterion_group!(
    benches,
    benchmark_simple_query,
    benchmark_deep_chaining,
    benchmark_complex_expressions
);
criterion_main!(benches);
```

**Performance targets:**

| Operation | Target | Notes |
|-----------|--------|-------|
| Simple query (10 rules) | < 1ms | Basic lookup |
| Simple query (1000 rules) | < 10ms | With RETE index |
| Expression parsing | < 100μs | Small expressions |
| Deep chain (depth=10) | < 5ms | Sequential reasoning |
| Complex expression parse | < 500μs | Nested operators |

**Success Criteria:**
- ✅ All benchmarks documented
- ✅ Performance targets met
- ✅ Regression tests in CI
- ✅ Performance guide written

**Estimated Time:** 3-4 days

---

### Task 2.4: Documentation (Week 5)

**Goal:** Complete, production-ready documentation

**Documents to create/update:**

```bash
docs/
  ├── BACKWARD_CHAINING.md           # User guide (UPDATE)
  ├── BACKWARD_CHAINING_API.md       # API reference (NEW)
  ├── BACKWARD_CHAINING_PERFORMANCE.md  # Performance guide (NEW)
  ├── BACKWARD_CHAINING_EXAMPLES.md  # Example cookbook (NEW)
  └── TROUBLESHOOTING.md             # Common issues (NEW)
```

**Example documentation:**

```markdown
# Backward Chaining API Reference

## Core API

### BackwardEngine

Main engine for backward chaining queries.

#### Constructor

```rust
pub fn new(kb: KnowledgeBase) -> Self
```

Creates a new backward chaining engine.

**Example:**
```rust
let kb = KnowledgeBase::new("MyKB");
// ... add rules to kb ...

let bc_engine = BackwardEngine::new(kb);
```

#### query()

```rust
pub fn query(
    &mut self,
    query_str: &str,
    facts: &mut Facts
) -> Result<QueryResult, BackwardError>
```

Execute a backward chaining query.

**Parameters:**
- `query_str`: Query expression (e.g., "User.IsVIP == true")
- `facts`: Current facts (may be modified by rule execution)

**Returns:**
- `Ok(QueryResult)`: Query result with proof trace
- `Err(BackwardError)`: Query failed

**Example:**
```rust
let mut facts = Facts::new();
facts.set("User.Points", Value::Number(150.0));

let result = bc_engine.query("User.IsVIP == true", &mut facts)?;

if result.provable {
    println!("User is VIP!");
    println!("Proof: {:?}", result.proof_trace);
}
```

**Errors:**
- `BackwardError::QueryParseError`: Invalid query syntax
- `BackwardError::MaxDepthExceeded`: Search too deep
- `BackwardError::CircularDependency`: Rules depend on each other

### QueryResult

Result of a backward chaining query.

**Fields:**
```rust
pub struct QueryResult {
    pub provable: bool,              // Can goal be proven?
    pub bindings: HashMap<String, Value>,  // Variable bindings
    pub proof_trace: ProofTrace,     // How goal was proven
    pub missing_facts: Vec<String>,  // What's needed to prove
    pub stats: QueryStats,           // Execution statistics
}
```

**Example:**
```rust
let result = bc_engine.query("Order.Approved == true", &mut facts)?;

if !result.provable {
    println!("Cannot approve order. Missing:");
    for fact in result.missing_facts {
        println!("  - {}", fact);
    }
}

println!("Explored {} goals in {:?}ms",
         result.stats.goals_explored,
         result.stats.duration_ms);
```

## Configuration

### BackwardConfig

Configuration for backward chaining engine.

**Example:**
```rust
let config = BackwardConfig {
    max_depth: 15,                    // Maximum reasoning depth
    strategy: SearchStrategy::DepthFirst,  // Search strategy
    enable_memoization: true,         // Cache proven goals
    max_solutions: 10,                // Find multiple solutions
};

let bc_engine = BackwardEngine::with_config(kb, config);
```

## Error Handling

All errors implement `std::error::Error` and can be:
- Logged with `tracing`
- Converted to user messages with `.user_message()`
- Pattern matched for specific handling

**Example:**
```rust
match bc_engine.query("Goal == true", &mut facts) {
    Ok(result) => { /* handle result */ },

    Err(BackwardError::GoalUnprovable { goal, missing_facts, .. }) => {
        eprintln!("Cannot prove: {}", goal);
        eprintln!("Missing: {:?}", missing_facts);
    },

    Err(BackwardError::MaxDepthExceeded { max_depth, .. }) => {
        eprintln!("Search too deep (max: {})", max_depth);
        eprintln!("Try increasing max_depth config");
    },

    Err(e) => {
        eprintln!("Query error: {}", e.user_message());
    },
}
```

## Advanced Features

### Variable Bindings

Use variables in queries for pattern matching:

```rust
// Query with variable
let result = bc_engine.query("?Customer.IsVIP == true", &mut facts)?;

// Access bindings
if let Some(customer_value) = result.bindings.get("Customer") {
    println!("Found VIP customer: {:?}", customer_value);
}
```

### GRL Query Syntax

Define queries declaratively in GRL:

```grl
query "CheckApproval" {
    goal: Order.Approved == true && Order.Risk != "High"
    strategy: depth-first
    max-depth: 10

    on-success: {
        Order.Status = "Approved";
        LogMessage("Order approved");
    }

    on-failure: {
        Order.Status = "Rejected";
        LogMessage("Order rejected");
    }
}
```

Execute GRL queries:

```rust
let query_str = load_query_from_file("queries.grl", "CheckApproval");
let query = GRLQueryParser::parse(&query_str)?;

let result = GRLQueryExecutor::execute(&query, &mut bc_engine, &mut facts)?;
```
```

**Success Criteria:**
- ✅ Complete API documentation
- ✅ 10+ documented examples
- ✅ Performance guide with benchmarks
- ✅ Troubleshooting guide
- ✅ All public APIs have rustdoc

**Estimated Time:** 3-4 days

---

## Phase 2 Summary

**Total Duration:** 2 weeks (14 days)

**Deliverables:**
- ✅ >200 comprehensive tests
- ✅ Production-grade error handling
- ✅ Performance benchmarks
- ✅ Complete documentation

**Success Criteria:**
- >90% test coverage
- No panics in production code
- All benchmarks meet targets
- Documentation complete

---

## Phase 3: Optimization (IMPORTANT) 🟡

**Duration:** 1-2 weeks
**Goal:** Optimize for production workloads
**Priority:** IMPORTANT for performance at scale

### Task 3.1: Profile & Optimize Hot Paths (Week 6)

**Goal:** Identify and optimize performance bottlenecks

**Profiling plan:**

```bash
# 1. CPU profiling
cargo install cargo-flamegraph
cargo flamegraph --bench backward_chaining_bench

# 2. Memory profiling
cargo install heaptrack
heaptrack ./target/release/examples/ecommerce_approval_demo

# 3. Benchmark specific scenarios
cargo bench --features backward-chaining
```

**Common optimizations:**

```rust
// 1. Use Rc/Arc only where needed
// BEFORE:
pub struct BackwardEngine {
    knowledge_base: Arc<KnowledgeBase>,  // ❌ Always Arc
}

// AFTER:
pub struct BackwardEngine {
    knowledge_base: Rc<KnowledgeBase>,   // ✅ Rc for single-thread
}

#[cfg(feature = "parallel")]
pub struct BackwardEngine {
    knowledge_base: Arc<KnowledgeBase>,  // Arc only when needed
}

// 2. Reduce allocations
// BEFORE:
fn extract_fields(&self) -> Vec<String> {
    let mut fields = Vec::new();  // ❌ New allocation each time
    // ...
}

// AFTER:
fn extract_fields(&self, fields: &mut Vec<String>) {
    fields.clear();  // ✅ Reuse buffer
    // ...
}

// 3. Use SmallVec for small collections
use smallvec::{SmallVec, smallvec};

// BEFORE:
pub candidate_rules: Vec<String>,  // ❌ Heap allocation

// AFTER:
pub candidate_rules: SmallVec<[String; 8]>,  // ✅ Stack for ≤8 items

// 4. Lazy evaluation
// BEFORE:
let all_candidates = self.find_all_candidates();
for candidate in all_candidates {  // ❌ Find all, then iterate
    if self.try_prove(candidate) {
        break;  // Found one, rest wasted
    }
}

// AFTER:
for candidate in self.candidates_iter() {  // ✅ Lazy iterator
    if self.try_prove(candidate) {
        break;  // Short-circuit
    }
}
```

**Success Criteria:**
- ✅ Flamegraph analyzed
- ✅ Hot paths optimized
- ✅ Allocations reduced by >30%
- ✅ Benchmark improvements documented

**Estimated Time:** 4-5 days

---

### Task 3.2: Improve Memoization (Week 6-7)

**Goal:** Smarter caching for better performance

**Current implementation:**
```rust
// Simple boolean cache
proven_cache: HashMap<String, bool>  // ❌ Only caches success/failure
```

**Improved implementation:**

```rust
// src/backward/memoization.rs (NEW)

use std::collections::HashMap;
use crate::types::Value;
use super::unification::Bindings;

/// Cached query result
#[derive(Debug, Clone)]
struct CachedResult {
    /// Was goal provable?
    provable: bool,

    /// Variable bindings (if provable)
    bindings: Option<Bindings>,

    /// Timestamp (for cache invalidation)
    timestamp: u64,

    /// Number of times accessed
    access_count: usize,
}

/// Smart memoization cache
pub struct MemoizationCache {
    /// Cache storage
    cache: HashMap<String, CachedResult>,

    /// Maximum cache size
    max_size: usize,

    /// Current timestamp (incremented on facts change)
    current_timestamp: u64,

    /// Cache hit/miss stats
    hits: usize,
    misses: usize,
}

impl MemoizationCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            current_timestamp: 0,
            hits: 0,
            misses: 0,
        }
    }

    /// Get cached result
    pub fn get(&mut self, query: &str) -> Option<&CachedResult> {
        if let Some(result) = self.cache.get_mut(query) {
            // Check if still valid
            if result.timestamp >= self.current_timestamp {
                result.access_count += 1;
                self.hits += 1;
                return Some(result);
            } else {
                // Invalidate old entry
                self.cache.remove(query);
            }
        }

        self.misses += 1;
        None
    }

    /// Cache a result
    pub fn insert(
        &mut self,
        query: String,
        provable: bool,
        bindings: Option<Bindings>,
    ) {
        // Evict if cache full
        if self.cache.len() >= self.max_size {
            self.evict_lru();
        }

        let result = CachedResult {
            provable,
            bindings,
            timestamp: self.current_timestamp,
            access_count: 0,
        };

        self.cache.insert(query, result);
    }

    /// Invalidate cache (call when facts change)
    pub fn invalidate(&mut self) {
        self.current_timestamp += 1;

        // Optionally clear old entries immediately
        self.cache.retain(|_, v| v.timestamp >= self.current_timestamp);
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some((key, _)) = self.cache.iter()
            .min_by_key(|(_, v)| v.access_count)
        {
            let key = key.clone();
            self.cache.remove(&key);
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            hits: self.hits,
            misses: self.misses,
            hit_rate: if self.hits + self.misses > 0 {
                self.hits as f64 / (self.hits + self.misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.current_timestamp = 0;
        self.hits = 0;
        self.misses = 0;
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub size: usize,
    pub hits: usize,
    pub misses: usize,
    pub hit_rate: f64,
}
```

**Success Criteria:**
- ✅ Full result caching (not just boolean)
- ✅ Cache invalidation on facts change
- ✅ LRU eviction policy
- ✅ Cache hit rate >70% in benchmarks

**Estimated Time:** 3-4 days

---

### Task 3.3: Parallel Goal Proving (Week 7, Optional)

**Goal:** Prove independent goals in parallel

**Implementation:**

```rust
// Only if `parallel` feature enabled
#[cfg(feature = "parallel")]
use rayon::prelude::*;

impl BackwardEngine {
    /// Prove multiple independent goals in parallel
    #[cfg(feature = "parallel")]
    pub fn query_parallel(
        &mut self,
        queries: &[&str],
        facts: &Facts,
    ) -> Result<Vec<QueryResult>, BackwardError> {
        use rayon::prelude::*;

        // Clone engine for each thread
        let engines: Vec<_> = (0..queries.len())
            .map(|_| BackwardEngine::new(self.knowledge_base.as_ref().clone()))
            .collect();

        // Execute queries in parallel
        queries.par_iter()
            .zip(engines.par_iter_mut())
            .map(|(query, engine)| {
                let mut facts_clone = facts.clone();
                engine.query(query, &mut facts_clone)
            })
            .collect()
    }
}
```

**Success Criteria:**
- ✅ Parallel execution for independent goals
- ✅ Thread-safe implementation
- ✅ Linear speedup for N cores
- ✅ No data races

**Estimated Time:** 3-4 days (optional)

---

## Phase 3 Summary

**Total Duration:** 1-2 weeks (7-14 days)

**Deliverables:**
- ✅ Optimized hot paths
- ✅ Smart memoization
- ✅ (Optional) Parallel execution

**Success Criteria:**
- 2-3x performance improvement
- Cache hit rate >70%
- Memory usage optimized

---

## Phase 4: Advanced Features (NICE TO HAVE) 🟢

**Duration:** 2-3 weeks
**Goal:** Advanced reasoning capabilities
**Priority:** NICE TO HAVE - can ship without these

### Task 4.1: Negation-as-Failure (Week 8)
### Task 4.2: Better Explanation System (Week 8-9)
### Task 4.3: Hybrid Mode (Forward + Backward) (Week 9-10)

*(Details available on request - keeping plan focused on critical path)*

---

## Production Readiness Checklist

### Before Production Deployment

- [ ] **Phase 1 Complete**
  - [ ] Expression parser with AST
  - [ ] Rule execution with fact derivation
  - [ ] RETE integration
  - [ ] Variable bindings
  - [ ] All Phase 1 tests pass

- [ ] **Phase 2 Complete**
  - [ ] >90% test coverage
  - [ ] Integration tests pass
  - [ ] Property tests pass
  - [ ] Stress tests pass
  - [ ] Error handling complete
  - [ ] Logging added
  - [ ] Documentation complete

- [ ] **Phase 3 Complete** (at least basics)
  - [ ] Performance benchmarked
  - [ ] Hot paths optimized
  - [ ] Memoization improved
  - [ ] Performance targets met

- [ ] **Quality Gates**
  - [ ] No panics in production code
  - [ ] No unsafe code (or well-documented)
  - [ ] No known bugs
  - [ ] CI/CD passing
  - [ ] Security audit done

- [ ] **Operational Readiness**
  - [ ] Monitoring/metrics defined
  - [ ] Error alerting configured
  - [ ] Runbook created
  - [ ] Rollback plan ready

---

## Timeline Summary

| Phase | Duration | Start | End | Priority |
|-------|----------|-------|-----|----------|
| Phase 1: Core Fixes | 2-3 weeks | Week 1 | Week 3 | 🔴 CRITICAL |
| Phase 2: Quality | 2 weeks | Week 4 | Week 5 | 🟡 IMPORTANT |
| Phase 3: Optimization | 1-2 weeks | Week 6 | Week 7 | 🟡 IMPORTANT |
| Phase 4: Advanced | 2-3 weeks | Week 8 | Week 10 | 🟢 NICE TO HAVE |
| **Minimum for Production** | **5 weeks** | **Week 1** | **Week 5** | **Phase 1+2** |
| **Recommended for Production** | **7 weeks** | **Week 1** | **Week 7** | **Phase 1+2+3** |
| **Full Feature Complete** | **10 weeks** | **Week 1** | **Week 10** | **All Phases** |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| **Parser complexity exceeds estimate** | Start with simple recursive descent, can upgrade to parser combinator later if needed |
| **RETE integration breaks existing code** | Feature-flag integration, comprehensive regression tests |
| **Performance targets not met** | Phase 3 dedicated to optimization, can extend if needed |
| **Unforeseen bugs in production** | Comprehensive testing in Phase 2, phased rollout |
| **Timeline slips** | Phases 1+2 are minimum - can ship without Phase 3/4 |

---

## Success Metrics

### Phase 1 Success
- All critical bugs fixed
- Core functionality works correctly
- Tests pass

### Phase 2 Success
- >90% test coverage
- Zero panics
- Complete documentation

### Phase 3 Success
- Meets performance targets
- Cache hit rate >70%
- Memory optimized

### Production Success
- Used in production workload
- No critical bugs for 1 month
- Performance acceptable
- User feedback positive

---

## Next Steps

1. **Review & Approve Plan** - Team review this plan
2. **Set Up Tracking** - Create tasks in issue tracker
3. **Start Phase 1.1** - Begin expression parser implementation
4. **Weekly Check-ins** - Review progress, adjust timeline
5. **Quality Gates** - Don't proceed to next phase until current phase meets criteria

---

**Document Version:** 1.0
**Last Updated:** 2025-11-25
**Owner:** Rust Rule Engine Team
